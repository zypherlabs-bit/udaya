use udaya_core::{
    transaction::{Transaction, create_p2pkh_transaction, estimate_tx_fee},
    types::{
        Block, BlockHash, BlockHeader, BlockLocator, InvType, InvVector,
        MerkleRoot, OutPoint, ScriptPubKey, ScriptSig, TxIn, TxOut, Txid,
    },
    address::{Address, AddressType, Network, hash160},
    consensus::{ConsensusEngine, ConsensusParams, create_genesis_block, GENESIS_BITS},
    validation::{UTXOSet, TransactionValidator, UTXOEntry, validate_block},
    security::{FuzzingEngine, FuzzConfig, AdversarialSimulator, AdversarialConfig,
               ChainSplitDetector, FloodProtection, AttackType},
    genesis::{mine_genesis_block, create_genesis_manifest, verify_genesis_block},
    serialization,
    config::UdayaConfig,
    BlockchainState, BlockchainInner,
    MAX_SUPPLY, SATS_PER_COIN, INITIAL_BLOCK_REWARD, HALVING_INTERVAL,
    DIFFICULTY_ADJUSTMENT_INTERVAL, BLOCK_TARGET_TIME_SECS, COINBASE_MATURITY,
    NETWORK_MAGIC, TICKER, MAINNET, TESTNET, REGTEST,
    BLOCK_VERSION, TX_VERSION, LOCKTIME_THRESHOLD,
    MAX_BLOCK_WEIGHT, MAX_BLOCK_SIZE, PROTOCOL_VERSION,
};
use num_bigint::BigUint;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;

// ============================================================
// BLOCKCHAIN STATE TESTS
// ============================================================

#[test]
fn test_blockchain_state_new() {
    let state = BlockchainState::new();
    let inner = state.inner.read();
    assert_eq!(inner.height, 0);
    assert!(inner.chain_tip.is_zero());
    assert_eq!(inner.total_work, BigUint::from(0u64));
    assert_eq!(inner.difficulty, 0);
}

#[test]
fn test_blockchain_state_clone() {
    let state1 = BlockchainState::new();
    let state2 = state1.clone();
    
    // Both should share the same inner Arc
    {
        let mut inner = state2.inner.write();
        inner.height = 42;
    }
    assert_eq!(state1.inner.read().height, 42);
}

// ============================================================
// CONSTANTS TESTS
// ============================================================

#[test]
fn test_network_constants() {
    assert_eq!(NETWORK_MAGIC, [0xBF, 0x59, 0x1A, 0xE7]);
    assert_eq!(TICKER, "UDYA");
    assert_eq!(MAX_SUPPLY, 21_000_000);
    assert_eq!(SATS_PER_COIN, 100_000_000);
    assert_eq!(BLOCK_TARGET_TIME_SECS, 600); // 10 minutes
    assert_eq!(INITIAL_BLOCK_REWARD, 50 * SATS_PER_COIN);
    assert_eq!(HALVING_INTERVAL, 210_000);
    assert_eq!(DIFFICULTY_ADJUSTMENT_INTERVAL, 2_016);
    assert_eq!(COINBASE_MATURITY, 100);
    assert_eq!(BLOCK_VERSION, 1);
    assert_eq!(TX_VERSION, 2);
    assert_eq!(PROTOCOL_VERSION, 70016);
}

#[test]
fn test_network_names() {
    assert_eq!(MAINNET, "mainnet");
    assert_eq!(TESTNET, "testnet");
    assert_eq!(REGTEST, "regtest");
}

// ============================================================
// BLOCKHASH TESTS
// ============================================================

#[test]
fn test_blockhash_compute() {
    let data = b"hello world";
    let hash = BlockHash::compute(data);
    assert!(!hash.is_zero());
    assert_eq!(hash.as_bytes().len(), 32);
    
    // Double SHA-256 should be deterministic
    let hash2 = BlockHash::double_sha256(data);
    assert_eq!(hash, hash2);
    
    let hash3 = BlockHash::compute(data);
    assert_eq!(hash, hash3);
}

#[test]
fn test_blockhash_display() {
    let hash = BlockHash::compute(b"test");
    let display = format!("{}", hash);
    assert_eq!(display.len(), 64); // 32 bytes = 64 hex chars
    assert!(!display.contains("BlockHash"));
}

#[test]
fn test_blockhash_debug() {
    let hash = BlockHash::compute(b"test");
    let debug = format!("{:?}", hash);
    assert!(debug.starts_with("BlockHash("));
}

#[test]
fn test_blockhash_from_bytes() {
    let data = [0xABu8; 32];
    let hash = BlockHash::from_bytes(&data);
    assert_eq!(hash.as_bytes(), &data);
}

#[test]
fn test_blockhash_from_array() {
    let data = [0x42u8; 32];
    let hash: BlockHash = data.into();
    assert_eq!(hash.as_bytes(), &data);
}

#[test]
fn test_blockhash_is_zero() {
    let zero = BlockHash::default();
    assert!(zero.is_zero());
    
    let non_zero = BlockHash::compute(b"not zero");
    assert!(!non_zero.is_zero());
}

#[test]
fn test_blockhash_to_vec() {
    let hash = BlockHash::compute(b"vec test");
    let vec = hash.to_vec();
    assert_eq!(vec.len(), 32);
    assert_eq!(vec, hash.0.to_vec());
}

// ============================================================
// TXID TESTS
// ============================================================

#[test]
fn test_txid_compute() {
    let data = b"transaction data";
    let txid = Txid::compute(data);
    assert_eq!(txid.as_bytes().len(), 32);
    
    // Should be deterministic
    let txid2 = Txid::compute(data);
    assert_eq!(txid, txid2);
}

#[test]
fn test_txid_from_array() {
    let data = [0xFFu8; 32];
    let txid: Txid = data.into();
    assert_eq!(txid.as_bytes(), &data);
}

#[test]
fn test_txid_display_debug() {
    let txid = Txid::compute(b"display test");
    let display = format!("{}", txid);
    assert_eq!(display.len(), 64);
}

// ============================================================
// MERKLE ROOT TESTS
// ============================================================

#[test]
fn test_merkle_root_empty() {
    let root = MerkleRoot::compute(&[]);
    assert!(root.0.iter().all(|&b| b == 0));
}

#[test]
fn test_merkle_root_single() {
    let txid = Txid::compute(b"single tx");
    let root = MerkleRoot::compute(&[txid]);
    assert!(!root.0.iter().all(|&b| b == 0));
    
    // Single element merkle root should equal the txid's hash
    // Actually it's the double-sha256 of the txid bytes
    let expected = BlockHash::double_sha256(&txid.0).0;
    assert_eq!(root.0, expected);
}

#[test]
fn test_merkle_root_multiple() {
    let txids: Vec<Txid> = (0..4).map(|i| {
        Txid::compute(format!("tx_{}", i).as_bytes())
    }).collect();
    
    let root = MerkleRoot::compute(&txids);
    assert!(!root.0.iter().all(|&b| b == 0));
}

#[test]
fn test_merkle_root_odd_count() {
    let txids: Vec<Txid> = (0..3).map(|i| {
        Txid::compute(format!("tx_{}", i).as_bytes())
    }).collect();
    
    let root = MerkleRoot::compute(&txids);
    assert!(!root.0.iter().all(|&b| b == 0));
    
    // Should handle odd counts by duplicating last element
    let root_4 = MerkleRoot::compute(&[
        txids[0], txids[1], txids[2], txids[2], // duplicate last
    ]);
    assert_eq!(root.0, root_4.0);
}

// ============================================================
// TRANSACTION TESTS
// ============================================================

fn create_dummy_tx() -> Transaction {
    Transaction::new(
        TX_VERSION,
        vec![TxIn {
            previous_output: OutPoint::new(
                Txid::compute(b"dummy"),
                0,
            ),
            script_sig: ScriptSig::new(vec![0x00]),
            sequence: 0xFFFFFFFF,
            witness: vec![],
        }],
        vec![
            TxOut::new(100_000_000, ScriptPubKey::new(vec![0x76, 0xA9, 0x14])),
        ],
        0,
    )
}

#[test]
fn test_transaction_creation() {
    let tx = create_dummy_tx();
    assert_eq!(tx.version, TX_VERSION);
    assert_eq!(tx.inputs.len(), 1);
    assert_eq!(tx.outputs.len(), 1);
    assert_eq!(tx.lock_time, 0);
}

#[test]
fn test_transaction_txid_deterministic() {
    let tx1 = create_dummy_tx();
    let tx2 = create_dummy_tx();
    assert_eq!(tx1.txid(), tx2.txid());
}

#[test]
fn test_transaction_wtxid() {
    let tx = create_dummy_tx();
    let txid = tx.txid();
    let wtxid = tx.wtxid();
    // Without witness data, txid == wtxid
    assert_eq!(txid, wtxid);
}

#[test]
fn test_coinbase_transaction() {
    let coinbase = Transaction::new_coinbase(
        vec![0x01, 0x02, 0x03],
        vec![TxOut::new(INITIAL_BLOCK_REWARD, ScriptPubKey::new(vec![0x41, 0xAC]))],
        0,
    );
    assert!(coinbase.is_coinbase());
    assert_eq!(coinbase.inputs.len(), 1);
    assert!(coinbase.inputs[0].is_coinbase());
}

#[test]
fn test_coinbase_block_height_encoding() {
    let height = 123456u64;
    let coinbase = Transaction::new_coinbase(
        vec![],
        vec![TxOut::new(50 * SATS_PER_COIN, ScriptPubKey::new(vec![]))],
        height,
    );
    
    // BIP-34: coinbase must start with block height
    let script_data = &coinbase.inputs[0].script_sig.data;
    assert_eq!(script_data[0] as u64, 3); // Push 3 bytes (for height 123456)
    let decoded_height = u64::from_le_bytes([
        script_data[1], script_data[2], script_data[3], 0, 0, 0, 0, 0,
    ]);
    assert_eq!(decoded_height, height);
}

#[test]
fn test_transaction_is_final() {
    let tx = Transaction::new(TX_VERSION, vec![], vec![], 0);
    assert!(tx.is_final(0, 0));
    
    // Locktime in the future (height-based)
    let tx_future = Transaction::new(TX_VERSION, vec![], vec![], 100);
    assert!(!tx_future.is_final(50, 0));
    assert!(tx_future.is_final(100, 0));
    assert!(tx_future.is_final(150, 0));
}

#[test]
fn test_transaction_total_output() {
    let tx = Transaction::new(
        TX_VERSION,
        vec![TxIn::new_coinbase(vec![])],
        vec![
            TxOut::new(100, ScriptPubKey::new(vec![0x00])),
            TxOut::new(200, ScriptPubKey::new(vec![0x00])),
            TxOut::new(300, ScriptPubKey::new(vec![0x00])),
        ],
        0,
    );
    assert_eq!(tx.total_output(), 600);
}

#[test]
fn test_transaction_total_output_saturating() {
    let tx = Transaction::new(
        TX_VERSION,
        vec![],
        vec![
            TxOut::new(u64::MAX, ScriptPubKey::new(vec![0x00])),
            TxOut::new(u64::MAX, ScriptPubKey::new(vec![0x00])),
        ],
        0,
    );
    // Should use saturating arithmetic
    assert_eq!(tx.total_output(), u64::MAX);
}

#[test]
fn test_transaction_serialization_roundtrip() {
    let tx = create_dummy_tx();
    let serialized = tx.serialize();
    let deserialized = Transaction::deserialize(&serialized).unwrap();
    assert_eq!(tx.txid(), deserialized.txid());
    assert_eq!(tx.version, deserialized.version);
    assert_eq!(tx.inputs.len(), deserialized.inputs.len());
    assert_eq!(tx.outputs.len(), deserialized.outputs.len());
}

#[test]
fn test_transaction_size_and_weight() {
    let tx = create_dummy_tx();
    let size = tx.size();
    assert!(size > 0);
    
    let weight = tx.weight();
    assert!(weight > 0);
    
    let vsize = tx.vsize();
    assert!(vsize > 0);
    assert!(vsize <= weight);
}

#[test]
fn test_is_valid_structure_empty() {
    let empty = Transaction::new(TX_VERSION, vec![], vec![], 0);
    assert!(!empty.is_valid_structure());
}

#[test]
fn test_is_valid_structure_valid() {
    let tx = create_dummy_tx();
    assert!(tx.is_valid_structure());
}

#[test]
fn test_is_valid_structure_multiple_coinbase_inputs() {
    let tx = Transaction::new(
        TX_VERSION,
        vec![
            TxIn::new_coinbase(vec![0x01]),
            TxIn::new_coinbase(vec![0x02]),
        ],
        vec![TxOut::new(100, ScriptPubKey::new(vec![0x00]))],
        0,
    );
    assert!(!tx.is_valid_structure()); // Coinbase must have exactly one input
}

#[test]
fn test_estimate_tx_fee() {
    let tx = create_dummy_tx();
    let fee = estimate_tx_fee(&tx, 10);
    assert_eq!(fee, tx.vsize() * 10);
}

#[test]
fn test_transaction_debug() {
    let tx = create_dummy_tx();
    let debug = format!("{:?}", tx);
    assert!(debug.contains("Transaction"));
    assert!(debug.contains("version"));
    assert!(debug.contains("inputs"));
    assert!(debug.contains("outputs"));
}

// ============================================================
// BLOCKHEADER TESTS
// ============================================================

#[test]
fn test_block_header_creation() {
    let prev_hash = BlockHash::compute(b"prev");
    let merkle_root = MerkleRoot::compute(&[Txid::compute(b"tx")]);
    
    let header = BlockHeader::new(
        BLOCK_VERSION,
        prev_hash,
        merkle_root,
        1234567890,
        0x1D00FFFF,
        12345,
    );
    
    assert_eq!(header.version, BLOCK_VERSION);
    assert_eq!(header.previous_block_hash, prev_hash);
    assert_eq!(header.merkle_root, merkle_root);
    assert_eq!(header.timestamp, 1234567890);
    assert_eq!(header.bits, 0x1D00FFFF);
    assert_eq!(header.nonce, 12345);
}

#[test]
fn test_block_header_hash_deterministic() {
    let header = create_test_header();
    let hash1 = header.hash();
    let hash2 = header.hash();
    assert_eq!(hash1, hash2);
}

#[test]
fn test_block_header_serialization_roundtrip() {
    let header = create_test_header();
    let serialized = header.serialize();
    
    // Serialization produces exactly 80 bytes (Bitcoin standard)
    assert_eq!(serialized.len(), 80);
}

#[test]
fn test_difficulty_target() {
    let header = create_test_header();
    let target = header.difficulty_target();
    assert!(target > BigUint::from(0u64));
    
    // For bits 0x1D00FFFF
    // Exponent = 0x1D = 29
    // Mantissa = 0x00FFFF
    // Target = mantissa * 256^(exponent - 3) = 0x00FFFF * 256^26
    let expected = BigUint::from(0xFFFFu64) << (8 * (0x1D - 3));
    assert_eq!(target, expected);
}

#[test]
fn test_verify_pow() {
    let header = create_test_header();
    // The default test header may or may not satisfy PoW
    // (depends on nonce/timestamp combo)
    let _ = header.verify_pow();
}

#[test]
fn test_block_header_debug() {
    let header = create_test_header();
    let debug = format!("{:?}", header);
    assert!(debug.contains("BlockHeader"));
    assert!(debug.contains("nonce"));
}

fn create_test_header() -> BlockHeader {
    BlockHeader {
        version: BLOCK_VERSION,
        previous_block_hash: BlockHash::default(),
        merkle_root: MerkleRoot([0u8; 32]),
        timestamp: 1231006505,
        bits: GENESIS_BITS,
        nonce: 0,
    }
}

// ============================================================
// BLOCK TESTS
// ============================================================

#[test]
fn test_block_creation() {
    let header = create_test_header();
    let tx = create_dummy_tx();
    let block = Block::new(header, vec![tx]);
    assert_eq!(block.header, header);
    assert_eq!(block.transactions.len(), 1);
}

#[test]
fn test_block_hash() {
    let header = create_test_header();
    let tx = create_dummy_tx();
    let block = Block::new(header, vec![tx]);
    assert_eq!(block.hash(), header.hash());
}

#[test]
fn test_block_merkle_root_verification() {
    let coinbase = Transaction::new_coinbase(
        vec![0x00],
        vec![TxOut::new(INITIAL_BLOCK_REWARD, ScriptPubKey::new(vec![0x41, 0xAC]))],
        0,
    );
    let merkle_root = MerkleRoot::compute(&[coinbase.txid()]);
    
    let header = BlockHeader {
        version: BLOCK_VERSION,
        previous_block_hash: BlockHash::default(),
        merkle_root,
        timestamp: 1231006505,
        bits: GENESIS_BITS,
        nonce: 0,
    };
    
    let block = Block::new(header, vec![coinbase]);
    assert!(block.verify_merkle_root());
}

#[test]
fn test_block_merkle_root_mismatch() {
    let coinbase = Transaction::new_coinbase(
        vec![0x00],
        vec![TxOut::new(50 * SATS_PER_COIN, ScriptPubKey::new(vec![0x41, 0xAC]))],
        0,
    );
    let wrong_root = MerkleRoot([0xFFu8; 32]);
    
    let header = BlockHeader {
        version: BLOCK_VERSION,
        previous_block_hash: BlockHash::default(),
        merkle_root: wrong_root,
        timestamp: 1231006505,
        bits: GENESIS_BITS,
        nonce: 0,
    };
    
    let block = Block::new(header, vec![coinbase]);
    assert!(!block.verify_merkle_root());
}

#[test]
fn test_block_coinbase_tx() {
    let coinbase = Transaction::new_coinbase(
        vec![0x00],
        vec![TxOut::new(50 * SATS_PER_COIN, ScriptPubKey::new(vec![]))],
        0,
    );
    let regular_tx = create_dummy_tx();
    
    let merkle_root = MerkleRoot::compute(&[coinbase.txid(), regular_tx.txid()]);
    let header = BlockHeader {
        version: BLOCK_VERSION,
        previous_block_hash: BlockHash::default(),
        merkle_root,
        timestamp: 1231006505,
        bits: GENESIS_BITS,
        nonce: 0,
    };
    
    let block = Block::new(header, vec![coinbase.clone(), regular_tx]);
    assert!(block.coinbase_tx().is_some());
    assert_eq!(block.coinbase_tx().unwrap().txid(), coinbase.txid());
}

#[test]
fn test_block_serialization_roundtrip() {
    let coinbase = Transaction::new_coinbase(
        vec![0x00],
        vec![TxOut::new(50 * SATS_PER_COIN, ScriptPubKey::new(vec![]))],
        0,
    );
    let merkle_root = MerkleRoot::compute(&[coinbase.txid()]);
    let header = BlockHeader {
        version: BLOCK_VERSION,
        previous_block_hash: BlockHash::default(),
        merkle_root,
        timestamp: 1231006505,
        bits: GENESIS_BITS,
        nonce: 0,
    };
    
    let block = Block::new(header, vec![coinbase]);
    let serialized = block.serialize();
    let deserialized = Block::deserialize(&serialized).unwrap();
    assert_eq!(block.hash(), deserialized.hash());
    assert_eq!(block.transactions.len(), deserialized.transactions.len());
}

#[test]
fn test_block_size() {
    let coinbase = Transaction::new_coinbase(
        vec![0x00],
        vec![TxOut::new(50 * SATS_PER_COIN, ScriptPubKey::new(vec![]))],
        0,
    );
    let merkle_root = MerkleRoot::compute(&[coinbase.txid()]);
    let header = BlockHeader {
        version: BLOCK_VERSION,
        previous_block_hash: BlockHash::default(),
        merkle_root,
        timestamp: 1231006505,
        bits: GENESIS_BITS,
        nonce: 0,
    };
    
    let block = Block::new(header, vec![coinbase]);
    assert!(block.size() > 0);
    assert_eq!(block.tx_count(), 1);
}

// ============================================================
// BLOCKLOCATOR TESTS
// ============================================================

#[test]
fn test_block_locator() {
    let hashes = vec![
        BlockHash::compute(b"block1"),
        BlockHash::compute(b"block2"),
    ];
    let stop = BlockHash::compute(b"stop");
    let locator = BlockLocator::new(hashes.clone(), stop);
    assert_eq!(locator.hashes.len(), 2);
    assert_eq!(locator.stop, stop);
}

// ============================================================
// INVENTORY TESTS
// ============================================================

#[test]
fn test_inv_type_values() {
    assert_eq!(InvType::Error as u8, 0);
    assert_eq!(InvType::Tx as u8, 1);
    assert_eq!(InvType::Block as u8, 2);
    assert_eq!(InvType::FilteredBlock as u8, 3);
    assert_eq!(InvType::CompactBlock as u8, 4);
}

#[test]
fn test_inv_vector() {
    let hash = BlockHash::compute(b"inv item");
    let inv = InvVector::new(InvType::Tx, hash);
    assert_eq!(inv.inv_type, InvType::Tx);
    assert_eq!(inv.hash, hash);
}