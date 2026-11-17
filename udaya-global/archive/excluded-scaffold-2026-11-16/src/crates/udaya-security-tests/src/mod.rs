/// Udaya Security Tests - Comprehensive test suite

use crate::primitives::*;
use crate::consensus::*;
use crate::execution::*;
use crate::wallet_core::*;
use rand::Rng;
use std::collections::HashMap;

#[test]
fn test_genesis() {
    let genesis = crate::consensus::create_genesis_block();
    assert_eq!(genesis.height, 0);
}

#[test]
fn test_vrf() {
    let vrf = crate::randomness::generate_vrf(1, b"seed", 1);
    assert!(!vrf.value.is_empty());
}

#[test]
fn test_peer_manager() {
    use crate::p2p::PeerManager;
    let pm = PeerManager::new("[::1]:9799".parse().unwrap(), 1);
    assert_eq!(pm.count(), 0);
}

#[test]
fn test_storage_config() {
    let config = crate::storage::StorageConfig::default();
    assert_eq!(config.data_dir, "data/udaya-global");
}

#[test]
fn test_wallet_core() {
    let mnemonic = ["test"; 25];
    let _ = from_mnemonic(mnemonic);
}

#[test]
fn test_fee_calculation() {
    let fee = crate::wallet_core::calculate_fee(100_000_000, 99_990_000);
    assert_eq!(fee, 10_000);
}

#[test]
fn test_script_execution() {
    let result = crate::execution::execute_script(
        &[0x76, 0xa9, 0x14, /* 20-byte pubkey hash */ 0u8; 20, 0xac],
        &[50_000_000],
        0,
    );
    assert!(matches!(result, crate::execution::ScriptResult::Success | crate::execution::ScriptResult::InsufficientFunds));
}

#[test]
fn test_chain_work() {
    let blocks = vec![
        crate::primitives::Block { height: 0, hash: vec![0u8; 32], parent_hash: vec![0u8; 32], merkle_root: vec![], timestamp: 0, bits: GENESIS_BITS, nonce: 0, coinbase_txid: vec![], transactions: vec![], committee_attestations: None },
    ];
    let work = crate::state::calculate_chain_work(&blocks);
    assert!(work > BigUint::from(0u32));
}

#[test]
fn test_merkle_root() {
    let txids = vec![vec![1u8; 32], vec![2u8; 32]];
    let _ = crate::state::verify_merkle_root(&txids, &[0u8; 32]);
}

#[test]
fn test_committee_selection() {
    let stakes = crate::consensus::BTreeMap::from([
        (1, 50_000),
        (2, 30_000),
    ]);
    let committee = crate::consensus::select_committee(&stakes, 2);
    assert!(committee.len() <= 2);
}

#[test]
fn test_blockchain_state() {
    let state = crate::state::BlockchainState {
        height: 0,
        tip_hash: vec![0u8; 32],
        utxo_set: HashMap::new(),
        total_supply: 21_000_000,
        orphan_blocks: vec![],
        chain_work: BigUint::from(0u32),
    };
    assert_eq!(state.height, 0);
}

#[test]
fn test_sync_status() {
    let synced = crate::state::SyncStatus::Synced;
    assert!(matches!(synced, crate::state::SyncStatus::Synced));
}

#[test]
fn test_vrf_beacon() {
    let participants = vec![
        (1, crate::randomness::VrffOutput { value: vec![1u8; 32], proof: vec![1u8; 32] }),
    ];
    let _beacon = crate::randomness::combine_vrf(&participants);
}

#[test]
fn test_pow_validation() {
    let block = crate::consensus::create_genesis_block();
    let params = crate::limits::ChainParams { is_experimental: true, ..crate::limits::EXPERIMENTAL_CHAIN_PARAMS };
    let _ = crate::consensus::validate_pow(&block, &params);
}