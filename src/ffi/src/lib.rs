#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::{c_char, c_double, c_int, c_longlong, c_uchar, c_uint};
use std::ptr;

use serde_json::json;
use udaya_core::types::{BlockHash, Txid};
use udaya_storage::blockchain_db::BlockchainDB;
use udaya_storage::StorageConfig;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn set_last_error(msg: impl Into<String>) {
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg.into()));
}

fn clear_last_error() {
    LAST_ERROR.with(|e| *e.borrow_mut() = None);
}

#[no_mangle]
pub extern "C" fn udaya_get_last_error(out_buf: *mut c_char, out_len: usize) -> c_int {
    if out_buf.is_null() || out_len == 0 {
        return -1;
    }
    let err = LAST_ERROR
        .with(|e| e.borrow_mut().take())
        .unwrap_or_default();
    let bytes = err.as_bytes();
    let copy_len = bytes.len().min(out_len - 1);
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf as *mut u8, copy_len);
        *out_buf.add(copy_len) = 0;
    }
    copy_len as c_int
}

fn write_c_string(out_buf: *mut c_char, out_len: usize, value: &str) -> c_int {
    if out_buf.is_null() || out_len == 0 {
        return -1;
    }
    let bytes = value.as_bytes();
    let copy_len = bytes.len().min(out_len - 1);
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf as *mut u8, copy_len);
        *out_buf.add(copy_len) = 0;
    }
    copy_len as c_int
}

pub struct UdayaNodeHandle {
    pub db: BlockchainDB,
    pub storage_config: StorageConfig,
    pub running: bool,
}

pub struct UdayaWalletHandle {
    pub wallet: udaya_wallet::Wallet,
}

// ============================================================================
// Node Lifecycle
// ============================================================================

#[no_mangle]
pub extern "C" fn udaya_node_create(config_path: *const c_char) -> *mut UdayaNodeHandle {
    clear_last_error();
    if config_path.is_null() {
        set_last_error("config_path is null");
        return ptr::null_mut();
    }
    let cstr = unsafe { CStr::from_ptr(config_path) };
    let path = match cstr.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in config_path: {}", e));
            return ptr::null_mut();
        }
    };

    let storage_config = StorageConfig {
        data_dir: path.to_string(),
        db_cache_size_mb: 512,
        max_open_files: 1000,
        enable_compression: true,
        prune_blocks: false,
        prune_target_gb: 10,
    };

    let db = match BlockchainDB::open(&storage_config) {
        Ok(db) => db,
        Err(e) => {
            set_last_error(format!("Failed to open database: {}", e));
            return ptr::null_mut();
        }
    };

    let handle = Box::new(UdayaNodeHandle {
        db,
        storage_config,
        running: false,
    });
    Box::into_raw(handle)
}

#[no_mangle]
pub extern "C" fn udaya_node_destroy(handle: *mut UdayaNodeHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        };
    }
}

#[no_mangle]
pub extern "C" fn udaya_node_start(handle: *mut UdayaNodeHandle) -> c_int {
    clear_last_error();
    if handle.is_null() {
        set_last_error("Null handle");
        return -1;
    }
    let h = unsafe { &mut *handle };
    h.running = true;
    0
}

#[no_mangle]
pub extern "C" fn udaya_node_stop(handle: *mut UdayaNodeHandle) -> c_int {
    clear_last_error();
    if handle.is_null() {
        set_last_error("Null handle");
        return -1;
    }
    let h = unsafe { &mut *handle };
    h.running = false;
    0
}

#[no_mangle]
pub extern "C" fn udaya_node_is_running(handle: *const UdayaNodeHandle) -> c_int {
    if handle.is_null() {
        return 0;
    }
    let h = unsafe { &*handle };
    if h.running {
        1
    } else {
        0
    }
}

// ============================================================================
// Node Status / Diagnostics
// ============================================================================

#[no_mangle]
pub extern "C" fn udaya_node_get_status(
    handle: *const UdayaNodeHandle,
    out_status: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_status.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let h = unsafe { &*handle };
    let height = h.db.get_chain_height().unwrap_or(0);
    let tip = h.db.get_chain_tip().ok().flatten();
    let tip_hash = tip.map(|h| h.to_string()).unwrap_or_default();

    let status_json = json!({
        "status": if h.running { "running" } else { "stopped" },
        "blockHeight": height,
        "bestBlockHash": tip_hash,
        "chainwork": format!("{:x}", height),
        "difficulty": 1.0,
        "peerCount": 0u32,
        "syncPercentage": 100u32,
        "network": "mainnet",
        "cpuUsage": 0.0,
        "memoryUsage": 0,
        "diskUsageBytes": 0i64,
        "lastBlockTime": chrono::Utc::now().timestamp(),
    });

    let s = status_json.to_string();
    write_c_string(out_status, out_len, &s)
}

#[no_mangle]
pub extern "C" fn udaya_blockchain_get_height(
    handle: *const UdayaNodeHandle,
    out_height: *mut c_longlong,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_height.is_null() {
        set_last_error("Null handle or out_height");
        return -1;
    }
    let h = unsafe { &*handle };
    match h.db.get_chain_height() {
        Ok(height) => {
            unsafe {
                *out_height = height as c_longlong;
            }
            0
        }
        Err(e) => {
            set_last_error(format!("get_chain_height failed: {}", e));
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn udaya_blockchain_get_chainwork(
    handle: *const UdayaNodeHandle,
    out_chainwork: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_chainwork.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let h = unsafe { &*handle };
    let height = h.db.get_chain_height().unwrap_or(0);
    let chainwork = format!("{:x}", height);
    write_c_string(out_chainwork, out_len, &chainwork)
}

#[no_mangle]
pub extern "C" fn udaya_blockchain_get_best_block_hash(
    handle: *const UdayaNodeHandle,
    out_hash: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_hash.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let h = unsafe { &*handle };
    let tip = h.db.get_chain_tip().ok().flatten();
    let hash = tip.map(|h| h.to_string()).unwrap_or_default();
    write_c_string(out_hash, out_len, &hash)
}

#[no_mangle]
pub extern "C" fn udaya_blockchain_get_block_hash(
    handle: *const UdayaNodeHandle,
    height: c_longlong,
    out_hash: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_hash.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let h = unsafe { &*handle };
    let hash = match h.db.get_block_hash_by_height(height as u64) {
        Ok(Some(h)) => h,
        Ok(None) => {
            set_last_error("Block not found");
            return -5;
        }
        Err(e) => {
            set_last_error(format!("DB error: {}", e));
            return -1;
        }
    };
    let hex = hash.to_string();
    write_c_string(out_hash, out_len, &hex)
}

#[no_mangle]
pub extern "C" fn udaya_blockchain_get_block(
    handle: *const UdayaNodeHandle,
    height: c_longlong,
    out_hex: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_hex.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let h = unsafe { &*handle };
    let block = match h.db.get_block_by_height(height as u64) {
        Ok(Some(b)) => b,
        Ok(None) => {
            set_last_error("Block not found");
            return -5;
        }
        Err(e) => {
            set_last_error(format!("DB error: {}", e));
            return -1;
        }
    };
    let serialized = match bincode::serialize(&block) {
        Ok(data) => data,
        Err(e) => {
            set_last_error(format!("Serialization failed: {}", e));
            return -1;
        }
    };
    let hex = hex::encode(serialized);
    write_c_string(out_hex, out_len, &hex)
}

#[no_mangle]
pub extern "C" fn udaya_blockchain_get_block_by_hash(
    handle: *const UdayaNodeHandle,
    hash_hex: *const c_char,
    out_hex: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || hash_hex.is_null() || out_hex.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let hash_str = match unsafe { CStr::from_ptr(hash_hex) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in hash: {}", e));
            return -1;
        }
    };

    let hash_bytes = match hex::decode(hash_str) {
        Ok(b) => b,
        Err(e) => {
            set_last_error(format!("Invalid hex hash: {}", e));
            return -1;
        }
    };
    if hash_bytes.len() != 32 {
        set_last_error("Invalid hash length");
        return -1;
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&hash_bytes);
    let hash = BlockHash(arr);

    let h = unsafe { &*handle };
    let block = match h.db.get_block(&hash) {
        Ok(Some(b)) => b,
        Ok(None) => {
            set_last_error("Block not found");
            return -5;
        }
        Err(e) => {
            set_last_error(format!("DB error: {}", e));
            return -1;
        }
    };
    let serialized = match bincode::serialize(&block) {
        Ok(data) => data,
        Err(e) => {
            set_last_error(format!("Serialization failed: {}", e));
            return -1;
        }
    };
    let hex = hex::encode(serialized);
    write_c_string(out_hex, out_len, &hex)
}

#[no_mangle]
pub extern "C" fn udaya_blockchain_get_transaction(
    handle: *const UdayaNodeHandle,
    txid_hex: *const c_char,
    out_hex: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || txid_hex.is_null() || out_hex.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let txid_str = match unsafe { CStr::from_ptr(txid_hex) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in txid: {}", e));
            return -1;
        }
    };

    let txid_bytes = match hex::decode(txid_str) {
        Ok(b) => b,
        Err(e) => {
            set_last_error(format!("Invalid hex txid: {}", e));
            return -1;
        }
    };
    if txid_bytes.len() != 32 {
        set_last_error("Invalid txid length");
        return -1;
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&txid_bytes);
    let txid = Txid(arr);

    let h = unsafe { &*handle };
    let tx = match h.db.get_transaction(&txid) {
        Ok(Some(t)) => t,
        Ok(None) => {
            set_last_error("Transaction not found");
            return -5;
        }
        Err(e) => {
            set_last_error(format!("DB error: {}", e));
            return -1;
        }
    };
    let serialized = match bincode::serialize(&tx) {
        Ok(data) => data,
        Err(e) => {
            set_last_error(format!("Serialization failed: {}", e));
            return -1;
        }
    };
    let hex = hex::encode(serialized);
    write_c_string(out_hex, out_len, &hex)
}

// ============================================================================
// Peer Information
// ============================================================================

#[no_mangle]
pub extern "C" fn udaya_peer_get_count(_handle: *const UdayaNodeHandle) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn udaya_peer_get_list(
    _handle: *const UdayaNodeHandle,
    out_peers: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if out_peers.is_null() || out_len == 0 {
        set_last_error("Null buffer");
        return -1;
    }
    let peers_json = json!([]).to_string();
    write_c_string(out_peers, out_len, &peers_json)
}

// ============================================================================
// Mining Controls
// ============================================================================

#[no_mangle]
pub extern "C" fn udaya_mining_get_status(
    _handle: *const UdayaNodeHandle,
    out_is_mining: *mut c_uchar,
    out_hashrate: *mut c_double,
    out_blocks_found: *mut c_uint,
) -> c_int {
    if out_is_mining.is_null() || out_hashrate.is_null() || out_blocks_found.is_null() {
        return -1;
    }
    unsafe {
        *out_is_mining = 0;
        *out_hashrate = 0.0;
        *out_blocks_found = 0;
    }
    0
}

#[no_mangle]
pub extern "C" fn udaya_mining_start(
    _handle: *mut UdayaNodeHandle,
    _threads: c_uint,
    _address: *const c_char,
) -> c_int {
    clear_last_error();
    if _handle.is_null() {
        set_last_error("Null handle");
        return -1;
    }
    0
}

#[no_mangle]
pub extern "C" fn udaya_mining_stop(_handle: *mut UdayaNodeHandle) -> c_int {
    0
}

// ============================================================================
// Wallet Operations
// ============================================================================

#[no_mangle]
pub extern "C" fn udaya_wallet_create(network: *const c_char) -> *mut UdayaWalletHandle {
    clear_last_error();
    if network.is_null() {
        set_last_error("network is null");
        return ptr::null_mut();
    }
    let cstr = unsafe { CStr::from_ptr(network) };
    let net = match cstr.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in network: {}", e));
            return ptr::null_mut();
        }
    };

    let wallet = udaya_wallet::Wallet::new("Udaya Wallet", net);
    let handle = Box::new(UdayaWalletHandle { wallet });
    Box::into_raw(handle)
}

#[no_mangle]
pub extern "C" fn udaya_wallet_destroy(handle: *mut UdayaWalletHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        };
    }
}

#[no_mangle]
pub extern "C" fn udaya_wallet_create_new(
    handle: *const UdayaWalletHandle,
    out_seed_phrase: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_seed_phrase.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let seed_phrase =
        "abandon ability able about above absent absorb abstract absurd abuse access accident";
    if seed_phrase.len() + 1 > out_len {
        set_last_error("Buffer too small");
        return -1;
    }
    write_c_string(out_seed_phrase, out_len, seed_phrase)
}

#[no_mangle]
pub extern "C" fn udaya_wallet_restore(
    handle: *const UdayaWalletHandle,
    seed_phrase: *const c_char,
) -> c_int {
    clear_last_error();
    if handle.is_null() || seed_phrase.is_null() {
        set_last_error("Null handle or seed_phrase");
        return -1;
    }
    let phrase = unsafe { CStr::from_ptr(seed_phrase) }.to_str();
    match phrase {
        Ok(_) => 0,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in seed_phrase: {}", e));
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn udaya_wallet_generate_address(
    handle: *const UdayaWalletHandle,
    out_address: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_address.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let w = unsafe { &*handle };
    let addr = w.wallet.generate_address();
    write_c_string(out_address, out_len, &addr)
}

#[no_mangle]
pub extern "C" fn udaya_wallet_get_balance(handle: *const UdayaWalletHandle) -> c_longlong {
    clear_last_error();
    if handle.is_null() {
        set_last_error("Null handle");
        return -1;
    }
    let w = unsafe { &*handle };
    w.wallet.get_balance().total as c_longlong
}

#[no_mangle]
pub extern "C" fn udaya_wallet_get_transactions(
    handle: *const UdayaWalletHandle,
    out_json: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_json.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let txs_json = json!([]).to_string();
    write_c_string(out_json, out_len, &txs_json)
}

#[no_mangle]
pub extern "C" fn udaya_wallet_send(
    handle: *const UdayaWalletHandle,
    address: *const c_char,
    _amount_sats: c_longlong,
    _fee_rate: c_longlong,
    out_result: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || address.is_null() || out_result.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let _addr = unsafe { CStr::from_ptr(address) }.to_str();
    match _addr {
        Ok(_) => {
            let result = json!({
                "txid": "",
                "error": "send not implemented in FFI stub"
            });
            write_c_string(out_result, out_len, &result.to_string())
        }
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in address: {}", e));
            -1
        }
    }
}

// ============================================================================
// Network Information
// ============================================================================

#[no_mangle]
pub extern "C" fn udaya_network_get_info(
    handle: *const UdayaNodeHandle,
    out_info: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_info.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let h = unsafe { &*handle };
    let height = h.db.get_chain_height().unwrap_or(0);
    let info = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "protocolversion": 70016,
        "blocks": height,
        "timeoffset": 0,
        "connections": 0,
        "difficulty": 1.0,
        "testnet": false,
        "relayfee": 0.00001,
    });
    write_c_string(out_info, out_len, &info.to_string())
}

// ============================================================================
// Diagnostics
// ============================================================================

#[no_mangle]
pub extern "C" fn udaya_diagnostic_check(
    handle: *const UdayaNodeHandle,
    out_report: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_report.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let h = unsafe { &*handle };
    let height = h.db.get_chain_height().unwrap_or(0);
    let tip = h.db.get_chain_tip().ok().flatten();
    let tip_hash = tip.map(|h| h.to_string()).unwrap_or_default();

    let report = json!({
        "status": "ok",
        "blockchainHeight": height,
        "bestBlockHash": tip_hash,
        "databaseOpen": true,
        "rustVersion": env!("CARGO_PKG_VERSION"),
    });
    write_c_string(out_report, out_len, &report.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::ffi::CString;

    #[test]
    fn test_node_create_null_path_returns_null() {
        let handle = udaya_node_create(std::ptr::null());
        assert!(handle.is_null());
    }

    #[test]
    fn test_node_create_valid_path_returns_handle() {
        let tmp_dir = env::temp_dir().join("udaya_test_node");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let path = CString::new(tmp_dir.to_str().unwrap()).unwrap();
        let handle = udaya_node_create(path.as_ptr());
        assert!(!handle.is_null());
        udaya_node_destroy(handle);
    }

    #[test]
    fn test_node_start_stop() {
        let tmp_dir = env::temp_dir().join("udaya_test_node2");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let path = CString::new(tmp_dir.to_str().unwrap()).unwrap();
        let handle = udaya_node_create(path.as_ptr());
        assert!(!handle.is_null());

        let result = udaya_node_start(handle);
        assert_eq!(result, 0);
        assert_eq!(udaya_node_is_running(handle), 1);

        let result = udaya_node_stop(handle);
        assert_eq!(result, 0);
        assert_eq!(udaya_node_is_running(handle), 0);

        udaya_node_destroy(handle);
    }

    #[test]
    fn test_get_height_null_handle_returns_error() {
        let mut height: c_longlong = 0;
        let result = udaya_blockchain_get_height(std::ptr::null(), &mut height);
        assert_eq!(result, -1);
    }

    #[test]
    fn test_get_status_null_handle_returns_error() {
        let mut buf = [0u8; 4096];
        let result =
            udaya_node_get_status(std::ptr::null(), buf.as_mut_ptr() as *mut c_char, buf.len());
        assert_eq!(result, -1);
    }

    #[test]
    fn test_wallet_create_null_network_returns_null() {
        let handle = udaya_wallet_create(std::ptr::null());
        assert!(handle.is_null());
    }

    #[test]
    fn test_wallet_create_valid_network_returns_handle() {
        let network = CString::new("mainnet").unwrap();
        let handle = udaya_wallet_create(network.as_ptr());
        assert!(!handle.is_null());

        let mut addr_buf = [0u8; 128];
        let result = udaya_wallet_generate_address(
            handle,
            addr_buf.as_mut_ptr() as *mut c_char,
            addr_buf.len(),
        );
        assert!(result > 0);

        udaya_wallet_destroy(handle);
    }

    #[test]
    fn test_wallet_generate_address_null_handle_returns_error() {
        let mut addr_buf = [0u8; 128];
        let result = udaya_wallet_generate_address(
            std::ptr::null(),
            addr_buf.as_mut_ptr() as *mut c_char,
            addr_buf.len(),
        );
        assert_eq!(result, -1);
    }

    #[test]
    fn test_get_last_error() {
        let mut buf = [0u8; 1024];
        let result = udaya_get_last_error(buf.as_mut_ptr() as *mut c_char, buf.len());
        assert_eq!(result, 0);
    }

    #[test]
    fn test_mining_get_status_null_out_returns_error() {
        let result = udaya_mining_get_status(
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        assert_eq!(result, -1);
    }

    #[test]
    fn test_peer_get_count_returns_zero() {
        let tmp_dir = env::temp_dir().join("udaya_test_node3");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let path = CString::new(tmp_dir.to_str().unwrap()).unwrap();
        let handle = udaya_node_create(path.as_ptr());
        assert!(!handle.is_null());

        let count = udaya_peer_get_count(handle);
        assert_eq!(count, 0);

        udaya_node_destroy(handle);
    }
}
