//! Phase 3 — Stability Tests
//!
//! These tests verify:
//! - Restart recovery
//! - Database consistency
//! - Multi-node synchronization
//! - Fork handling
//! - Crash recovery
//! - Long-running stability

use num_bigint::BigUint;
use num_traits::Zero;
use tempfile::tempdir;
use udaya_core::{
    consensus::{ConsensusEngine, ConsensusParams},
    types::{Block, BlockHash, BlockLocator, MerkleRoot, Txid},
    validation::{TransactionValidator, UTXOSet},
    BLOCK_VERSION,
};
use udaya_storage::{BlockchainDB, StorageConfig};

// ============================================================
// RESTART RECOVERY TESTS
// ============================================================

#[test]
fn test_restart_recovery_block_persistence() {
    let dir = tempdir().unwrap();
    let mut config = StorageConfig::default();
    config.data_dir = dir.path().to_str().unwrap().to_string();

    {
        let db = BlockchainDB::open(&config).unwrap();
        let genesis = udaya_core::consensus::create_genesis_block();
        db.store_block(&genesis, 0).unwrap();

        let prev_hash = genesis.hash();
        let mut header = genesis.header.clone();
        header.previous_block_hash = prev_hash;
        header.timestamp += 600;
        header.nonce = 42;
        let block2 = Block::new(header, genesis.transactions.clone());
        db.store_block(&block2, 1).unwrap();
    }

    {
        let db = BlockchainDB::open(&config).unwrap();
        assert_eq!(db.get_chain_height().unwrap(), 1);
        assert!(db.get_chain_tip().unwrap().is_some());
        assert!(db.get_block_by_height(0).unwrap().is_some());
        assert!(db.get_block_by_height(1).unwrap().is_some());
    }
}

#[test]
fn test_restart_recovery_utxo_persistence() {
    let dir = tempdir().unwrap();
    let mut config = StorageConfig::default();
    config.data_dir = dir.path().to_str().unwrap().to_string();

    {
        let db = BlockchainDB::open(&config).unwrap();
        let genesis = udaya_core::consensus::create_genesis_block();
        db.store_block(&genesis, 0).unwrap();
        db.update_utxo_set_for_block(&genesis, 0).unwrap();
        assert_eq!(db.load_utxo_set().unwrap().len(), 1);
    }

    {
        let db = BlockchainDB::open(&config).unwrap();
        let utxo_set = db.load_utxo_set().unwrap();
        assert_eq!(utxo_set.len(), 1);
        assert!(utxo_set.len() > 0);
    }
}

#[test]
fn test_restart_recovery_transaction_index() {
    let dir = tempdir().unwrap();
    let mut config = StorageConfig::default();
    config.data_dir = dir.path().to_str().unwrap().to_string();

    {
        let db = BlockchainDB::open(&config).unwrap();
        let genesis = udaya_core::consensus::create_genesis_block();
        db.store_block(&genesis, 0).unwrap();
        let coinbase = genesis.coinbase_tx().unwrap();
        let txid = coinbase.txid();
        assert!(db.get_transaction(&txid).unwrap().is_some());
    }

    {
        let db = BlockchainDB::open(&config).unwrap();
        let genesis = db.get_block_by_height(0).unwrap().unwrap();
        let coinbase = genesis.coinbase_tx().unwrap();
        let txid = coinbase.txid();
        assert!(db.get_transaction(&txid).unwrap().is_some());
    }
}

// ============================================================
// DATABASE CONSISTENCY TESTS
// ============================================================

#[test]
fn test_database_consistency_block_hash_index() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let mut prev_hash = BlockHash::default();
    for height in 0..10 {
        let mut header = create_test_header(prev_hash, height);
        header.nonce = (height * 100) as u32;
        let coinbase = create_test_coinbase(height);
        let block = Block::new(header, vec![coinbase]);
        db.store_block(&block, height).unwrap();
        prev_hash = block.hash();
    }

    for height in 0..10 {
        let expected_hash = db.get_block_hash_by_height(height).unwrap();
        assert!(expected_hash.is_some());
        let block = db.get_block_by_height(height).unwrap();
        assert!(block.is_some());
        assert_eq!(expected_hash.unwrap(), block.unwrap().hash());
    }
}

#[test]
fn test_database_consistency_chain_state() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    assert_eq!(db.get_chain_height().unwrap(), 0);
    assert!(db.get_chain_tip().unwrap().is_none());

    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).unwrap();
    assert_eq!(db.get_chain_height().unwrap(), 0);
    assert_eq!(db.get_chain_tip().unwrap(), Some(genesis.hash()));

    let mut header = genesis.header.clone();
    header.previous_block_hash = genesis.hash();
    header.nonce = 1;
    let block = Block::new(header, vec![]);
    db.store_block(&block, 100).unwrap();
    assert_eq!(db.get_chain_height().unwrap(), 100);
    assert_eq!(db.get_chain_tip().unwrap(), Some(block.hash()));
}

#[test]
fn test_database_consistency_utxo_after_blocks() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).unwrap();
    db.update_utxo_set_for_block(&genesis, 0).unwrap();
    let utxo_count_after_genesis = db.load_utxo_set().unwrap().len();

    let mut header = genesis.header.clone();
    header.previous_block_hash = genesis.hash();
    header.nonce = 1;
    let coinbase = create_test_coinbase(1);
    let block = Block::new(header, vec![coinbase]);
    db.store_block(&block, 1).unwrap();
    db.update_utxo_set_for_block(&block, 1).unwrap();

    let utxo_count_after_block = db.load_utxo_set().unwrap().len();
    assert!(utxo_count_after_block >= utxo_count_after_genesis);
}

// ============================================================
// MULTI-NODE SYNCHRONIZATION TESTS
// ============================================================

#[test]
fn test_multi_node_block_sync() {
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    let config1 = StorageConfig {
        data_dir: dir1.path().to_str().unwrap().to_string(),
        ..Default::default()
    };
    let config2 = StorageConfig {
        data_dir: dir2.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db1 = BlockchainDB::open(&config1).unwrap();
    let genesis = udaya_core::consensus::create_genesis_block();

    let mut prev_hash = BlockHash::default();
    let mut blocks = Vec::new();

    for height in 0..5 {
        let mut header = create_test_header(prev_hash, height);
        header.nonce = (height * 1000) as u32;
        let coinbase = create_test_coinbase(height);
        let block = Block::new(header, vec![coinbase]);
        db1.store_block(&block, height).unwrap();
        prev_hash = block.hash();
        blocks.push((height, block));
    }

    let db2 = BlockchainDB::open(&config2).unwrap();
    for (height, block) in &blocks {
        db2.store_block(block, *height).unwrap();
    }

    assert_eq!(db2.get_chain_height().unwrap(), 4);
    assert_eq!(db2.get_chain_tip().unwrap(), Some(prev_hash));

    for (height, expected_block) in &blocks {
        let synced_block = db2.get_block_by_height(*height).unwrap();
        assert!(synced_block.is_some());
        assert_eq!(synced_block.unwrap().hash(), expected_block.hash());
    }
}

#[test]
fn test_block_locator_for_sync() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let genesis = udaya_core::consensus::create_genesis_block();

    let mut prev_hash = BlockHash::default();
    let mut hashes = Vec::new();

    for height in 0..100 {
        let mut header = create_test_header(prev_hash, height);
        header.nonce = height as u32;
        let coinbase = create_test_coinbase(height);
        let block = Block::new(header, vec![coinbase]);
        db.store_block(&block, height).unwrap();
        prev_hash = block.hash();
        hashes.push(prev_hash);
    }

    let locator = generate_block_locator(&hashes, Some(hashes[99]));

    fn generate_block_locator(hashes: &[BlockHash], stop: Option<BlockHash>) -> BlockLocator {
        let mut locator_hashes = Vec::new();
        let mut step = 1;
        let mut index = hashes.len() - 1;
        while index > 0 {
            locator_hashes.push(hashes[index]);
            index = index.saturating_sub(step);
            if index > 0 && step < 100 {
                step *= 2;
            }
        }
        locator_hashes.push(BlockHash::default());
        BlockLocator {
            hashes: locator_hashes,
            stop: stop.unwrap_or_default(),
        }
    }

    assert!(locator.hashes.len() < 50);
    assert!(locator.hashes.len() > 1);
    assert_eq!(locator.hashes[0], hashes[99]);
}

// ============================================================
// FORK HANDLING TESTS
// ============================================================

#[test]
fn test_fork_detection() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).unwrap();

    let mut hash_a = genesis.hash();
    let header_a = create_test_header(hash_a, 1);
    let coinbase_a = create_test_coinbase(1);
    let block_a = Block::new(header_a, vec![coinbase_a]);
    db.store_block(&block_a, 1).unwrap();

    hash_a = block_a.hash();
    let header_b = create_test_header(hash_a, 2);
    let coinbase_b = create_test_coinbase(2);
    let block_b = Block::new(header_b, vec![coinbase_b]);
    db.store_block(&block_b, 2).unwrap();

    let header_x = create_test_header(genesis.hash(), 1);
    let coinbase_x = create_test_coinbase(1);
    let mut block_x = Block::new(header_x, vec![coinbase_x]);
    block_x.header.nonce = 999;

    let header_y = create_test_header(block_x.hash(), 2);
    let coinbase_y = create_test_coinbase(2);
    let mut block_y = Block::new(header_y, vec![coinbase_y]);
    block_y.header.nonce = 888;

    assert_eq!(
        block_a.header.previous_block_hash,
        block_x.header.previous_block_hash
    );
    assert_ne!(block_a.hash(), block_x.hash());
    assert_ne!(block_b.hash(), block_y.hash());
}

#[test]
fn test_reorg_safety() {
    let consensus = udaya_core::consensus::ConsensusEngine::new(
        udaya_core::consensus::ConsensusParams::default(),
    );
    let current_length = 100;
    let fork_length = 106;
    assert!(consensus.is_reorg_safe(fork_length, current_length));
}

// ============================================================
// CRASH RECOVERY TESTS
// ============================================================

#[test]
fn test_crash_recovery_incomplete_block() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    {
        let db = BlockchainDB::open(&config).unwrap();
        let genesis = udaya_core::consensus::create_genesis_block();
        db.store_block(&genesis, 0).unwrap();
    }

    {
        let db = BlockchainDB::open(&config).unwrap();
        assert_eq!(db.get_chain_height().unwrap(), 0);
        assert!(db.get_block_by_height(0).unwrap().is_some());

        let genesis = db.get_block_by_height(0).unwrap().unwrap();
        let mut header = genesis.header.clone();
        header.previous_block_hash = genesis.hash();
        header.nonce = 1;
        let block = Block::new(header, vec![]);
        db.store_block(&block, 1).unwrap();
        assert_eq!(db.get_chain_height().unwrap(), 1);
    }
}

#[test]
fn test_crash_recovery_database_integrity() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    {
        let db = BlockchainDB::open(&config).unwrap();
        for height in 0..10 {
            let genesis = udaya_core::consensus::create_genesis_block();
            let mut header = genesis.header.clone();
            header.nonce = height as u32;
            let block = Block::new(header, vec![]);
            db.store_block(&block, height).unwrap();
        }
    }

    {
        let db = BlockchainDB::open(&config).unwrap();
        assert_eq!(db.get_chain_height().unwrap(), 9);
        assert_eq!(db.block_count().unwrap(), 10);
        for height in 0..10 {
            assert!(db.get_block_by_height(height).unwrap().is_some());
        }
        assert!(db.get_chain_tip().unwrap().is_some());
    }
}

// ============================================================
// LONG-RUNNING STABILITY TESTS
// ============================================================

#[test]
fn test_long_running_many_blocks() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).unwrap();

    let mut prev_hash = genesis.hash();

    for height in 1..1000 {
        let mut header = create_test_header(prev_hash, height);
        header.nonce = height as u32;
        let coinbase = create_test_coinbase(height);
        let block = Block::new(header, vec![coinbase]);
        db.store_block(&block, height).unwrap();
        prev_hash = block.hash();

        if height % 100 == 0 {
            assert_eq!(db.get_chain_height().unwrap(), height);
        }
    }

    assert_eq!(db.get_chain_height().unwrap(), 999);
    assert_eq!(db.block_count().unwrap(), 1000);
}

#[test]
fn test_long_running_utxo_stability() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).unwrap();
    db.update_utxo_set_for_block(&genesis, 0).unwrap();

    let mut prev_hash = genesis.hash();
    let mut utxo_count = db.load_utxo_set().unwrap().len();

    for height in 1..100 {
        let mut header = create_test_header(prev_hash, height);
        header.nonce = height as u32;
        let coinbase = create_test_coinbase(height);
        let block = Block::new(header, vec![coinbase]);

        db.store_block(&block, height).unwrap();
        db.update_utxo_set_for_block(&block, height).unwrap();
        prev_hash = block.hash();

        let new_count = db.load_utxo_set().unwrap().len();
        assert!(
            new_count >= utxo_count,
            "UTXO count decreased from {} to {} at height {}",
            utxo_count,
            new_count,
            height
        );
        utxo_count = new_count;
    }
}

#[test]
fn test_long_running_validation_consistency() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let consensus = ConsensusEngine::new(ConsensusParams::default());

    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).unwrap();

    let mut prev_hash = genesis.hash();
    let _validator = TransactionValidator::new(consensus);

    for height in 1..100 {
        let mut header = create_test_header(prev_hash, height);
        header.nonce = height as u32;
        let coinbase = create_test_coinbase(height);
        let txs = vec![coinbase];
        let txids: Vec<Txid> = txs.iter().map(|tx| tx.txid()).collect();
        header.merkle_root = MerkleRoot::compute(&txids);
        let block = Block::new(header, txs);

        db.store_block(&block, height).unwrap();
        let stored = db.get_block_by_height(height).unwrap().unwrap();
        assert!(stored.verify_merkle_root());
        assert_eq!(stored.hash(), block.hash());
        prev_hash = stored.hash();
    }
}

// ============================================================
// HELPER FUNCTIONS
// ============================================================

fn create_test_header(prev_hash: BlockHash, height: u64) -> udaya_core::types::BlockHeader {
    use udaya_core::consensus::GENESIS_BITS;
    udaya_core::types::BlockHeader {
        version: BLOCK_VERSION,
        previous_block_hash: prev_hash,
        merkle_root: MerkleRoot::compute(&[Txid::compute(format!("merkle_{}", height).as_bytes())]),
        timestamp: (1231006505 + (height * 600)) as u32,
        bits: GENESIS_BITS,
        nonce: 0,
    }
}

fn create_test_coinbase(height: u64) -> udaya_core::transaction::Transaction {
    use udaya_core::{
        transaction::Transaction, types::ScriptPubKey, types::TxOut, INITIAL_BLOCK_REWARD,
    };
    Transaction::new_coinbase(
        format!("Udaya Block {}", height).into_bytes(),
        vec![TxOut::new(
            INITIAL_BLOCK_REWARD,
            ScriptPubKey::new(vec![0x6a]),
        )],
        height,
    )
}

fn calculate_chain_work(hashes: &[BlockHash]) -> BigUint {
    use num_bigint::BigUint;
    use num_traits::Zero;
    let mut total_work = BigUint::zero();
    for hash in hashes {
        let hash_int = BigUint::from_bytes_be(&hash.0);
        let max_hash = BigUint::from(2u128).pow(256);
        if hash_int > BigUint::zero() {
            total_work += &max_hash / &hash_int;
        }
    }
    total_work
}

// ============================================================
// PERFORMANCE AND STRESS TESTS
// ============================================================

#[test]
fn test_block_iteration_efficiency() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).unwrap();
    let mut prev_hash = genesis.hash();

    for height in 1..500 {
        let mut header = create_test_header(prev_hash, height);
        header.nonce = height as u32;
        let block = Block::new(header, vec![]);
        db.store_block(&block, height).unwrap();
        prev_hash = block.hash();
    }

    let blocks: Vec<_> = db.iter_blocks().collect();
    assert_eq!(blocks.len(), 500);
    for (i, (height, _)) in blocks.iter().enumerate() {
        assert_eq!(*height, i as u64);
    }
}

#[test]
fn test_database_flush_and_durability() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).unwrap();
    db.flush().unwrap();
    drop(db);

    let db = BlockchainDB::open(&config).unwrap();
    assert_eq!(db.get_chain_height().unwrap(), 0);
    assert!(db.get_block_by_height(0).unwrap().is_some());
}

#[test]
fn test_sequential_access_stability() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let genesis = udaya_core::consensus::create_genesis_block();
    db.store_block(&genesis, 0).unwrap();
    let mut prev_hash = genesis.hash();

    for height in 1..50 {
        let mut header = create_test_header(prev_hash, height);
        header.nonce = height as u32;
        let block = Block::new(header, vec![]);
        db.store_block(&block, height).unwrap();
        let _ = db.get_chain_height().unwrap();
        let _ = db.get_chain_tip().unwrap();
        prev_hash = block.hash();
    }
    assert_eq!(db.get_chain_height().unwrap(), 49);
}

#[cfg(test)]
mod block_locator_tests {
    use super::*;

    #[test]
    fn test_generate_block_locator_logic() {
        let hashes: Vec<BlockHash> = (0..100)
            .map(|i| BlockHash::compute(format!("block_{}", i).as_bytes()))
            .collect();

        let mut locator = Vec::new();
        let mut step = 1;
        let mut index = hashes.len() - 1;
        while index > 0 {
            locator.push(hashes[index]);
            index = index.saturating_sub(step);
            if index > 0 && step < 100 {
                step *= 2;
            }
        }
        locator.push(BlockHash::default());
        assert!(locator.len() < 20);
        assert!(locator.len() > 1);
    }
}

// ============================================================
// EDGE CASE STABILITY TESTS
// ============================================================

#[test]
fn test_empty_database_operations() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    assert_eq!(db.get_chain_height().unwrap(), 0);
    assert!(db.get_chain_tip().unwrap().is_none());
    assert_eq!(db.block_count().unwrap(), 0);
    assert!(db.get_block_hash_by_height(0).unwrap().is_none());
    assert!(db.get_block(&BlockHash::default()).unwrap().is_none());
}

#[test]
fn test_large_utxo_set_stability() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let mut utxo_set = UTXOSet::new();
    for i in 0..1000 {
        let txid = Txid::compute(format!("tx_{}", i).as_bytes());
        let outpoint = udaya_core::types::OutPoint::new(txid, 0);
        utxo_set.add_utxo(
            outpoint,
            udaya_core::validation::UTXOEntry {
                value: 1000 + i,
                script_pubkey: vec![0x76, 0xa9, 0x14],
                height: i as u64,
                is_coinbase: i % 2 == 0,
            },
        );
    }

    db.store_utxo_set(&utxo_set).unwrap();
    let loaded = db.load_utxo_set().unwrap();
    assert_eq!(loaded.len(), 1000);
}

#[test]
fn test_max_size_block_storage() {
    let dir = tempdir().unwrap();
    let config = StorageConfig {
        data_dir: dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };

    let db = BlockchainDB::open(&config).unwrap();
    let genesis = udaya_core::consensus::create_genesis_block();

    let mut txs = vec![];
    for i in 0..1000 {
        let tx = create_test_coinbase(i);
        txs.push(tx);
    }

    let mut header = genesis.header.clone();
    header.previous_block_hash = genesis.hash();
    header.nonce = 1;
    let block = Block::new(header, txs);

    db.store_block(&block, 1).unwrap();
    let retrieved = db.get_block_by_height(1).unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().transactions.len(), 1000);
}

fn main() {
    println!("Udaya Stability Tests");
    println!("This crate contains stability test functions that should be run as part of the test suite.");
    println!("Run with: cargo test --package stability-tests");
}
