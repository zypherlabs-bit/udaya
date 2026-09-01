#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_longlong};
use std::ptr;

use udaya_wallet::Wallet;

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
    let bytes = value.as_bytes();
    let copy_len = bytes.len().min(out_len - 1);
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf as *mut u8, copy_len);
        *out_buf.add(copy_len) = 0;
    }
    copy_len as c_int
}

// ============================================================================
// Wallet Handle
// ============================================================================

pub struct UdayaWalletHandle {
    pub wallet: Wallet,
}

// ============================================================================
// Wallet Lifecycle
// ============================================================================

#[no_mangle]
pub extern "C" fn udaya_wallet_create(
    name: *const c_char,
    network: *const c_char,
) -> *mut UdayaWalletHandle {
    clear_last_error();
    if name.is_null() || network.is_null() {
        set_last_error("name or network is null");
        return ptr::null_mut();
    }
    let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in name: {}", e));
            return ptr::null_mut();
        }
    };
    let network_str = match unsafe { CStr::from_ptr(network) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in network: {}", e));
            return ptr::null_mut();
        }
    };

    let wallet = Wallet::new(name_str, network_str);
    let handle = Box::new(UdayaWalletHandle { wallet });
    Box::into_raw(handle)
}

#[no_mangle]
pub extern "C" fn udaya_wallet_destroy(handle: *mut UdayaWalletHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(handle);
    }
}

#[no_mangle]
pub extern "C" fn udaya_wallet_create_new(
    handle: *mut UdayaWalletHandle,
    out_seed: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_seed.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let h = unsafe { &mut *handle };
    let (words, _address) = h.wallet.generate_seed();
    let seed = words.join(" ");
    write_c_string(out_seed, out_len, &seed)
}

#[no_mangle]
pub extern "C" fn udaya_wallet_restore(
    handle: *mut UdayaWalletHandle,
    seed_phrase: *const c_char,
) -> c_int {
    clear_last_error();
    if handle.is_null() || seed_phrase.is_null() {
        set_last_error("Null handle or seed_phrase");
        return -1;
    }
    let seed = match unsafe { CStr::from_ptr(seed_phrase) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in seed_phrase: {}", e));
            return -1;
        }
    };
    let words: Vec<String> = seed.split_whitespace().map(|s| s.to_string()).collect();
    let h = unsafe { &mut *handle };
    match h.wallet.recover_from_mnemonic(&words, "") {
        Ok(_) => 0,
        Err(e) => {
            set_last_error(format!("Failed to restore wallet: {}", e));
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn udaya_wallet_generate_address(
    handle: *mut UdayaWalletHandle,
    out_address: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || out_address.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let h = unsafe { &mut *handle };
    let address = h.wallet.generate_address();
    write_c_string(out_address, out_len, &address)
}

#[no_mangle]
pub extern "C" fn udaya_wallet_get_balance(handle: *const UdayaWalletHandle) -> c_longlong {
    clear_last_error();
    if handle.is_null() {
        set_last_error("Null handle");
        return -1;
    }
    let h = unsafe { &*handle };
    let balance = h.wallet.get_balance();
    balance.satoshi_total as c_longlong
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
    let h = unsafe { &*handle };
    let txs = h.wallet.get_transactions(100, 0);
    let json = match serde_json::to_string(&txs) {
        Ok(j) => j,
        Err(e) => {
            set_last_error(format!("Failed to serialize transactions: {}", e));
            return -1;
        }
    };
    write_c_string(out_json, out_len, &json)
}

#[no_mangle]
pub extern "C" fn udaya_wallet_send(
    handle: *const UdayaWalletHandle,
    address: *const c_char,
    amount: c_longlong,
    _fee_rate: c_int,
    out_txid: *mut c_char,
    out_len: usize,
) -> c_int {
    clear_last_error();
    if handle.is_null() || address.is_null() || out_txid.is_null() || out_len == 0 {
        set_last_error("Null handle or buffer");
        return -1;
    }
    let addr = match unsafe { CStr::from_ptr(address) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("Invalid UTF-8 in address: {}", e));
            return -1;
        }
    };
    set_last_error(format!("Send not yet implemented: {} {}", addr, amount));
    -1
}

// ============================================================================
// Diagnostics
// ============================================================================

#[no_mangle]
pub extern "C" fn udaya_diagnostic_check() -> c_int {
    clear_last_error();
    0
}
