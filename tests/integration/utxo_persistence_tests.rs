//! Integration tests for Udaya node persistent state (Phase 5)

use udaya_core::types::{OutPoint, ScriptPubKey, ScriptSig, TxIn, TxOut};
use udaya_core::{consensus, transaction};
use udaya_storage::blockchain_db::BlockchainDB;
use udaya_storage::StorageConfig;

/// Test helper to create a unique temporary test directory
fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "udaya_utxo_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Test UTXO set persistence across simulated restart
#[test]
fn test_utxo_persistence_across_restart() {
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

    // Create a genesis block and store it at height 0
    let genesis = consensus::create_genesis_block();
    db.store_block(&genesis, 0).expect("Failed to store genesis");
    db.update_utxo_set_for_block(&genesis, 0).expect("Failed to update UTXO set");

    // Verify genesis UTXO was persisted
    let utxo_set_after_genesis = db.load_utxo_set().expect("Failed to load UTXO set");
    assert_eq!(utxo_set_after_genesis.len(), 1, "Genesis should create 1 UTXO");

    // Verify chain state before restart
    let chain_height_before = db.get_chain_height().expect("Failed to get chain height");
    assert_eq!(chain_height_before, 0, "Chain height should be 0 after genesis at height 0");

    // Create a transaction spending the coinbase
    let mut utxo_set = utxo_set_after_genesis;
    let coinbase_tx = genesis.coinbase_tx().unwrap();
    let coinbase_outpoint = OutPoint::new(coinbase_tx.txid(), 0);

    if utxo_set.contains(&coinbase_outpoint) {
        let tx = transaction::Transaction::new(
            1,
            vec![TxIn {
                previous_output: coinbase_outpoint.clone(),
                script_sig: ScriptSig::new(vec![0x51]), // OP_1
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            vec![TxOut::new(
                50_000_000_000 - 1000,
                ScriptPubKey::new(vec![0x76, 0xa9, 0x14, 0xab, 0xcd]),
            )],
            0,
        );
        utxo_set.apply_transaction(&tx, &tx.txid(), 1);
    }

    // Store the updated UTXO set
    db.store_utxo_set(&utxo_set).expect("Failed to store UTXO set");

    // Simulate restart: close and reopen database
    drop(db);
    let db2 = BlockchainDB::open(&config).expect("Failed to reopen database");

    // Verify chain state persisted across restart
    let chain_height_after = db2.get_chain_height().expect("Failed to get chain height");
    assert_eq!(chain_height_after, 0, "Chain height should persist across restart");

    // Load UTXO set after restart
    let loaded_utxo_set = db2.load_utxo_set().expect("Failed to load UTXO set after restart");

    // Verify UTXO set persisted correctly
    assert_eq!(loaded_utxo_set.len(), 1, "Should have 1 UTXO after spending coinbase");
    assert!(!loaded_utxo_set.contains(&coinbase_outpoint), "Coinbase UTXO should be spent");

    let _ = std::fs::remove_dir_all(&temp);
}

/// Test UTXO set persistence with multiple blocks
#[test]
fn test_utxo_persistence_multiple_blocks() {
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

    // Store genesis block at height 0
    let genesis = consensus::create_genesis_block();
    db.store_block(&genesis, 0).expect("Failed to store genesis");
    db.update_utxo_set_for_block(&genesis, 0).expect("Failed to update UTXO set");

    // Store second block at height 1
    let block2 = consensus::create_genesis_block(); // simplified - using another block
    db.store_block(&block2, 1).expect("Failed to store block 2");
    db.update_utxo_set_for_block(&block2, 1).expect("Failed to update UTXO set for block 2");

    // Check chain height before restart
    let chain_height_before = db.get_chain_height().expect("Failed to get chain height");
    assert_eq!(chain_height_before, 1, "Chain height should be 1 after storing 2 blocks");

    // Simulate restart
    drop(db);
    let db2 = BlockchainDB::open(&config).expect("Failed to reopen database");

    // Verify persistence
    let utxo_set = db2.load_utxo_set().expect("Failed to load UTXO set");
    let chain_height = db2.get_chain_height().expect("Failed to get chain height");

    assert_eq!(chain_height, 1, "Chain height should persist across restart");
    assert!(!utxo_set.is_empty(), "UTXO set should not be empty after restart");

    let _ = std::fs::remove_dir_all(&temp);
}
