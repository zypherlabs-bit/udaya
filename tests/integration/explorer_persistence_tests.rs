//! Integration tests for Phase 8 - Explorer Persistence

use udaya_core::consensus::ConsensusEngine;
use udaya_core::validation::UTXOSet;
use udaya_explorer::ExplorerEngine;
use udaya_storage::blockchain_db::BlockchainDB;
use udaya_storage::StorageConfig;

/// Test helper to create a unique temporary test directory
fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "udaya_explorer_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Test explorer block indexing persistence
#[test]
fn test_explorer_block_indexing_persistence() {
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

    // Create explorer and load from database
    let explorer = ExplorerEngine::new();
    explorer.load_from_database(&db).expect("Failed to load explorer data");

    // Verify block indexing
    let stats = explorer.get_stats();
    assert_eq!(stats.block_height, 0, "Explorer should index genesis block");
    assert_eq!(stats.total_blocks, 0, "Explorer should count blocks");
    assert_eq!(stats.total_transactions, 1, "Explorer should count coinbase tx");

    // Verify block cache
    let cached_block = explorer.get_block(0);
    assert!(cached_block.is_some(), "Explorer should cache genesis block");
    let summary = cached_block.unwrap();
    assert_eq!(summary.height, 0);
    assert_eq!(summary.hash, genesis.hash().to_string());

    // Persist explorer state
    explorer.persist_to_database(&db).expect("Failed to persist explorer state");

    // Simulate restart: create new explorer and restore from database
    drop(explorer);
    let new_explorer = ExplorerEngine::new();

    // Restore cached state
    new_explorer.restore_from_database(&db).expect("Failed to restore explorer state");

    // Verify restored state
    let restored_stats = new_explorer.get_stats();
    assert_eq!(restored_stats.block_height, 0, "Restored explorer should have genesis height");
    assert_eq!(restored_stats.total_transactions, 1, "Restored explorer should have tx count");

    let restored_block = new_explorer.get_block(0);
    assert!(restored_block.is_some(), "Restored explorer should have cached block");

    let _ = std::fs::remove_dir_all(&temp);
}

/// Test explorer transaction indexing persistence
#[test]
fn test_explorer_transaction_indexing_persistence() {
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

    // Get coinbase txid
    let coinbase = genesis.coinbase_tx().expect("Genesis should have coinbase");
    let txid = coinbase.txid();

    // Create explorer and load from database
    let explorer = ExplorerEngine::new();
    explorer.load_from_database(&db).expect("Failed to load explorer data");

    // Cache the transaction
    let tx_summary = ExplorerEngine::tx_to_summary(coinbase, Some(0), None, Some(0), 1);
    explorer.cache_tx(txid.to_string(), tx_summary);

    // Persist explorer state
    explorer.persist_to_database(&db).expect("Failed to persist explorer state");

    // Simulate restart
    drop(explorer);
    let new_explorer = ExplorerEngine::new();
    new_explorer.restore_from_database(&db).expect("Failed to restore explorer state");

    // Verify transaction was restored
    let cached_tx = new_explorer.get_tx(&txid.to_string());
    assert!(cached_tx.is_some(), "Restored explorer should have cached transaction");

    let _ = std::fs::remove_dir_all(&temp);
}

/// Test explorer reload after database changes
#[test]
fn test_explorer_reload_after_db_changes() {
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

    // Create explorer
    let explorer = ExplorerEngine::new();

    // Load with empty database
    explorer.load_from_database(&db).expect("Failed to load explorer data");
    let stats1 = explorer.get_stats();
    assert_eq!(stats1.block_height, 0);
    assert_eq!(stats1.total_transactions, 0);

    // Add genesis block to database
    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).expect("Failed to store genesis");

    // Reload explorer from database
    explorer.load_from_database(&db).expect("Failed to reload explorer data");
    let stats2 = explorer.get_stats();
    assert_eq!(stats2.block_height, 0);
    assert_eq!(stats2.total_transactions, 1, "Explorer should see new transaction after reload");

    let _ = std::fs::remove_dir_all(&temp);
}

/// Test explorer persistence with empty database
#[test]
fn test_explorer_persistence_empty_db() {
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

    // Try to restore explorer state from empty database
    let explorer = ExplorerEngine::new();
    let result = explorer.restore_from_database(&db);
    assert!(result.is_ok(), "Restoring from empty DB should succeed");

    let stats = explorer.get_stats();
    assert_eq!(stats.block_height, 0, "Empty DB should have 0 height");

    let _ = std::fs::remove_dir_all(&temp);
}
