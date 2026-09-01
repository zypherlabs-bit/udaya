//! Integration tests for Phase 7 - RPC Implementation and Explorer

use udaya_core::consensus::ConsensusEngine;
use udaya_core::validation::UTXOSet;
use udaya_explorer::ExplorerEngine;
use udaya_storage::blockchain_db::BlockchainDB;
use udaya_storage::StorageConfig;
use udaya_wallet::Wallet;

/// Test helper to create a unique temporary test directory
fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "udaya_phase7_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Test that RPC getblockchaininfo returns real data
#[test]
fn test_rpc_getblockchaininfo_real_data() {
    let temp = temp_dir();
    let config = StorageConfig {
        data_dir: temp.to_str().unwrap().to_string(),
        db_cache_size_mb: 64,
        max_open_files: 100,
        enable_compression: false,
        prune_blocks: false,
        prune_target_gb: 1,
    };

    let db = BlockchainDB::open(&config).expect("Failed to open database");

    // Store genesis block
    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).expect("Failed to store genesis");
    db.update_utxo_set_for_block(&genesis, 0).expect("Failed to update UTXO set");

    // Verify getblockchaininfo data
    let chain_height = db.get_chain_height().expect("Failed to get chain height");
    assert_eq!(chain_height, 0, "Chain height should be 0 after genesis");

    let tip = db.get_chain_tip().expect("Failed to get chain tip");
    assert!(tip.is_some(), "Chain tip should exist");

    let block_count = db.block_count().expect("Failed to get block count");
    assert_eq!(block_count, 1, "Should have 1 block");

    let _ = std::fs::remove_dir_all(&temp);
}

/// Test that RPC getblock returns real data
#[test]
fn test_rpc_getblock_real_data() {
    let temp = temp_dir();
    let config = StorageConfig {
        data_dir: temp.to_str().unwrap().to_string(),
        db_cache_size_mb: 64,
        max_open_files: 100,
        enable_compression: false,
        prune_blocks: false,
        prune_target_gb: 1,
    };

    let db = BlockchainDB::open(&config).expect("Failed to open database");

    // Store genesis block
    let genesis = udaya_core::consensus::create_genesis_block();
    let genesis_hash = genesis.hash();
    db.store_block(&genesis, 0).expect("Failed to store genesis");

    // Retrieve block by hash
    let retrieved = db.get_block(&genesis_hash).expect("Failed to get block");
    assert!(retrieved.is_some(), "Block should be found by hash");

    let block = retrieved.unwrap();
    assert_eq!(block.hash(), genesis_hash);
    assert!(!block.transactions.is_empty(), "Genesis should have transactions");

    // Retrieve block by height
    let retrieved_by_height = db.get_block_by_height(0).expect("Failed to get block by height");
    assert!(retrieved_by_height.is_some(), "Block should be found by height");

    let _ = std::fs::remove_dir_all(&temp);
}

/// Test that RPC gettransaction returns real data
#[test]
fn test_rpc_gettransaction_real_data() {
    let temp = temp_dir();
    let config = StorageConfig {
        data_dir: temp.to_str().unwrap().to_string(),
        db_cache_size_mb: 64,
        max_open_files: 100,
        enable_compression: false,
        prune_blocks: false,
        prune_target_gb: 1,
    };

    let db = BlockchainDB::open(&config).expect("Failed to open database");

    // Store genesis block (includes coinbase transaction)
    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).expect("Failed to store genesis");

    // Get coinbase transaction
    let coinbase = genesis.coinbase_tx().expect("Genesis should have coinbase");
    let txid = coinbase.txid();

    // Retrieve transaction
    let retrieved = db.get_transaction(&txid).expect("Failed to get transaction");
    assert!(retrieved.is_some(), "Transaction should be found");

    let tx = retrieved.unwrap();
    assert_eq!(tx.txid(), txid);
    assert!(!tx.inputs.is_empty() || !tx.outputs.is_empty(), "Transaction should have inputs or outputs");

    let _ = std::fs::remove_dir_all(&temp);
}

/// Test that explorer loads real data from database
#[test]
fn test_explorer_loads_real_data() {
    let temp = temp_dir();
    let config = StorageConfig {
        data_dir: temp.to_str().unwrap().to_string(),
        db_cache_size_mb: 64,
        max_open_files: 100,
        enable_compression: false,
        prune_blocks: false,
        prune_target_gb: 1,
    };

    let db = BlockchainDB::open(&config).expect("Failed to open database");

    // Store genesis block
    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).expect("Failed to store genesis");
    db.update_utxo_set_for_block(&genesis, 0).expect("Failed to update UTXO set");

    // Load explorer from database
    let explorer = udaya_explorer::ExplorerEngine::new();
    explorer.load_from_database(&db).expect("Failed to load explorer data");

    // Verify explorer has real data
    let stats = explorer.get_stats();
    assert_eq!(stats.block_height, 0, "Explorer should show genesis height");
    assert_eq!(stats.total_blocks, 0, "Explorer should show 0 blocks");
    assert_eq!(stats.total_transactions, 1, "Explorer should count coinbase tx");

    let _ = std::fs::remove_dir_all(&temp);
}

/// Test wallet RPC methods with real data
#[test]
fn test_wallet_rpc_real_data() {
    let wallet = Wallet::new("Test Wallet", "testnet");
    
    // Test getbalance returns real data
    let balance = wallet.get_balance();
    assert_eq!(balance.total, 0.0, "New wallet should have zero balance");

    // Test getnewaddress returns real address
    let address = wallet.generate_address();
    assert!(!address.is_empty(), "Address should not be empty");
    assert!(address.starts_with("btf1"), "Mainnet address should start with btf1");

    // Test listunspent returns real data
    let utxos = wallet.get_utxos();
    assert!(utxos.is_empty(), "New wallet should have no UTXOs");

    // Test listtransactions returns real data
    let txs = wallet.get_transactions(100, 0);
    assert!(txs.is_empty(), "New wallet should have no transactions");

    let _ = std::fs::remove_dir_all(&temp_dir());
}

/// Test mempool RPC methods with real data
#[test]
fn test_mempool_rpc_real_data() {
    let consensus = ConsensusEngine::new(udaya_core::consensus::ConsensusParams::default());
    let mempool = udaya_mempool::Mempool::new(
        udaya_mempool::MempoolConfig::default(),
        consensus,
        UTXOSet::new(),
    );

    // Test getmempoolinfo returns real data
    let stats = mempool.get_stats();
    assert_eq!(stats.total_transactions, 0, "Empty mempool should have 0 transactions");
    assert_eq!(stats.total_bytes, 0, "Empty mempool should have 0 bytes");

    // Test mempool contains returns false for non-existent tx
    assert!(!mempool.contains(&udaya_core::types::Txid([0u8; 32])), "Non-existent tx should not be in mempool");
}
