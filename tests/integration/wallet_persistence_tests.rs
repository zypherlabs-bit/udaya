//! Integration tests for wallet persistence (Phase 6)

use udaya_wallet::Wallet;
use udaya_storage::blockchain_db::BlockchainDB;
use udaya_storage::StorageConfig;

/// Test helper to create a unique temporary test directory
fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "udaya_wallet_test_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Test wallet state persistence across simulated restart
#[test]
fn test_wallet_persistence_across_restart() {
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

    // Create a new wallet and generate some state
    let wallet = Wallet::new("Test Wallet", "testnet");
    let address1 = wallet.generate_address();
    let address2 = wallet.generate_address();
    
    assert!(!address1.is_empty());
    assert!(!address2.is_empty());
    assert_ne!(address1, address2, "Each call should generate a new address");

    // Export wallet state and persist to database
    let wallet_state = wallet.export_state();
    assert_eq!(wallet_state.accounts.len(), 1);
    assert_eq!(wallet_state.accounts[0].external_keys.len(), 2);

    let serialized = bincode::serialize(&wallet_state).expect("Failed to serialize wallet state");
    db.store_wallet_raw(&serialized).expect("Failed to store wallet");

    // Simulate restart: drop wallet and create a new one
    drop(wallet);

    // Create a new wallet instance (simulates node restart)
    let new_wallet = Wallet::new("Test Wallet", "testnet");

    // Load wallet state from database
    let loaded_data = db.load_wallet_raw().expect("Failed to load wallet raw");
    assert!(loaded_data.is_some(), "Wallet state should be persisted");
    
    let loaded_state: udaya_wallet::WalletState = bincode::deserialize(&loaded_data.unwrap()).expect("Failed to deserialize wallet state");
    assert_eq!(loaded_state.name, "Test Wallet");
    assert_eq!(loaded_state.network, "testnet");
    assert_eq!(loaded_state.accounts.len(), 1);
    assert_eq!(loaded_state.accounts[0].external_keys.len(), 2);
    assert_eq!(loaded_state.accounts[0].external_keys[0], address1);
    assert_eq!(loaded_state.accounts[0].external_keys[1], address2);

    // Import loaded state into new wallet
    new_wallet.import_state(loaded_state);

    // Verify wallet recovered correctly
    let recovered_state = new_wallet.export_state();
    assert_eq!(recovered_state.accounts[0].external_keys.len(), 2);
    assert_eq!(recovered_state.accounts[0].external_keys[0], address1);
    assert_eq!(recovered_state.accounts[0].external_keys[1], address2);

    let _ = std::fs::remove_dir_all(&temp);
}

/// Test wallet persistence with empty database
#[test]
fn test_wallet_persistence_empty_db() {
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

    // Try to load wallet from empty database
    let loaded_data = db.load_wallet_raw().expect("Failed to load wallet raw");
    assert!(loaded_data.is_none(), "Empty database should have no wallet");

    let _ = std::fs::remove_dir_all(&temp);
}

/// Test wallet persistence after generating seed
#[test]
fn test_wallet_persistence_after_seed_generation() {
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

    // Create wallet and generate seed
    let wallet = Wallet::new("Seed Test Wallet", "testnet");
    let (mnemonic, address) = wallet.generate_seed();
    
    assert_eq!(mnemonic.len(), 12, "Should generate 12-word mnemonic");
    assert!(address.starts_with("btf1"), "Mainnet address should start with btf1");

    // Persist wallet state
    let wallet_state = wallet.export_state();
    let serialized = bincode::serialize(&wallet_state).expect("Failed to serialize wallet state");
    db.store_wallet_raw(&serialized).expect("Failed to store wallet");

    // Simulate restart
    drop(wallet);
    let new_wallet = Wallet::new("Seed Test Wallet", "testnet");

    // Load and import state
    let loaded_data = db.load_wallet_raw().expect("Failed to load wallet raw").unwrap();
    let loaded_state: udaya_wallet::WalletState = bincode::deserialize(&loaded_data).expect("Failed to deserialize wallet state");
    new_wallet.import_state(loaded_state);

    // Verify mnemonic and address persisted
    let recovered_state = new_wallet.export_state();
    assert_eq!(recovered_state.accounts[0].mnemonic.as_ref().unwrap().len(), 12);
    assert_eq!(recovered_state.accounts[0].external_keys.len(), 1);
    assert_eq!(recovered_state.accounts[0].external_keys[0], address);

    let _ = std::fs::remove_dir_all(&temp);
}
