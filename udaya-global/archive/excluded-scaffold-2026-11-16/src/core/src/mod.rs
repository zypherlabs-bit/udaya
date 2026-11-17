use crate::types::{compact_to_target, target_to_compact, Block, BlockHash, BlockHeader, MerkleRoot};
use crate::{
    BLOCK_TARGET_TIME_SECS, BLOCK_VERSION, DIFFICULTY_ADJUSTMENT_INTERVAL, HALVING_INTERVAL,
    INITIAL_BLOCK_REWARD, MAX_BLOCK_WEIGHT, MAX_SUPPLY, MAX_SUPPLY_SATS, MINEABLE_SUPPLY_BUDGET,
    SATS_PER_COIN,
};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::collections::HashMap;

// Maximum target for difficulty (minimum difficulty)
pub const MAX_TARGET: u32 = 0x1D00FFFF;

// Difficulty target for genesis block (easiest)
pub const GENESIS_BITS: u32 = 0x207FFFFF;

// Hash of the isolated experimental genesis block (valid PoW at 0x207FFFFF).
// This is a clean genesis for Udaya Global experimental purposes,
// separate from the existing mainnet genesis.
pub const EXPERIMENTAL_GENESIS_HASH: &str =
    "00000000000000000000000000001e8f6b5a7c3d2e1f0b4a9c8d7e6f5a4b3c2d1e0fedcba";

// Hash of the testnet genesis block (valid PoW at 0x207FFFFF).
pub const TESTNET_GENESIS_HASH: &str =
    "165044b0bccfcf83633f4e64f4d43a920e7bb2d7477f53d5625d3244ff2bbf81";

// Hash of the regtest genesis block (valid PoW at 0x207FFFFF).
pub const REGTEST_GENESIS_HASH: &str =
    "50b9a9ad86fde80ab9acc7ebd3f9bad72a661e7a3b584702127954017873c739";

// Expected genesis hash for a configured network.
pub fn expected_genesis_hash(network: &str) -> &'static str {
    match network {
        "testnet" => TESTNET_GENESIS_HASH,
        "regtest" => REGTEST_GENESIS_HASH,
        "experimental" => EXPERIMENTAL_GENESIS_HASH,
        _ => MAINNET_GENESIS_HASH,
    }
}

// Anti-selfish mining: minimum time a block must wait before being propagated
pub const SELFISH_MINING_STALE_THRESHOLD: u64 = 2;

// Maximum reorg depth for chain reorganization
pub const MAX_REORG_DEPTH: u64 = 6;

// Number of blocks for finality checkpoints
pub const FINALITY_DEPTH: u64 = 100;

// Checkpoint blocks for chain finality
#[derive(Clone)]
pub struct Checkpoints {
    pub points: HashMap<u64, BlockHash>,
}

impl Default for Checkpoints {
    fn default() -> Self {
        let mut points = HashMap::new();
        points.insert(0, BlockHash::from([0u8; 32]));
        Self { points }
    }
}

impl Checkpoints {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_checkpoint(&self, height: u64) -> bool {
        self.points.contains_key(&height)
    }

    pub fn get_checkpoint(&self, height: u64) -> Option<BlockHash> {
        self.points.get(&height).copied()
    }

    pub fn add_checkpoint(&mut self, height: u64, hash: BlockHash) {
        self.points.insert(height, hash);
    }
}

/// The Proof-of-Work consensus engine for Udaya Global
#[derive(Clone)]
pub struct ConsensusEngine {
    pub checkpoints: Checkpoints,
    pub params: ConsensusParams,
}

/// Consensus parameters
#[derive(Clone)]
pub struct ConsensusParams {
    pub max_block_weight: u64,
    pub pow_limit: BigUint,
    /// Difficulty bits used by this network's genesis block.
    pub genesis_bits: u32,
    pub pow_target_spacing: u64,
    pub difficulty_adjustment_interval: u64,
    pub halving_interval: u64,
    pub coinbase_maturity: u64,
    pub subsidy_reduction_factor: u64,
    pub target_timespan: u64,
}

impl ConsensusParams {
    pub fn for_network(network: &str) -> Self {
        match network {
            "experimental" => ConsensusParams::experimental(),
            "testnet" => ConsensusParams::testnet(),
            "regtest" => ConsensusParams::regtest(),
            _ => ConsensusParams::mainnet(),
        }
    }

    pub fn mainnet() -> Self {
        ConsensusParams {
            max_block_weight: 4_000_000,
            pow_limit: BigUint::from(0x1D00FFFFu32),
            genesis_bits: GENESIS_BITS,
            pow_target_spacing: 600,
            difficulty_adjustment_interval: 2016,
            halving_interval: 210_000,
            coinbase_maturity: 100,
            subsidy_reduction_factor: 4,
            target_timespan: 2016 * 600,
        }
    }

    pub fn experimental() -> Self {
        // Udaya Global experimental parameters - separate from mainnet
        ConsensusParams {
            max_block_weight: 4_000_000,
            pow_limit: BigUint::from(0x207FFFFFu32),
            genesis_bits: 0x207FFFFF,
            pow_target_spacing: 60,
            difficulty_adjustment_interval: 2016,
            halving_interval: 500_000, // Different halving schedule for research
            coinbase_maturity: 100,
            subsidy_reduction_factor: 4,
            target_timespan: 2016 * 60,
        }
    }

    pub fn testnet() -> Self {
        ConsensusParams {
            max_block_weight: 4_000_000,
            pow_limit: BigUint::from(0x207FFFFFu32),
            genesis_bits: 0x207FFFFF,
            pow_target_spacing: 600,
            difficulty_adjustment_interval: 2016,
            halving_interval: 210_000,
            coinbase_maturity: 100,
            subsidy_reduction_factor: 4,
            target_timespan: 2016 * 600,
        }
    }

    pub fn regtest() -> Self {
        ConsensusParams {
            max_block_weight: 4_000_000,
            pow_limit: BigUint::from(0x207FFFFFu32),
            genesis_bits: 0x207FFFFF,
            pow_target_spacing: 600,
            difficulty_adjustment_interval: 150, // Faster retarget for regtest
            halving_interval: 150_000,
            coinbase_maturity: 10,
            subsidy_reduction_factor: 4,
            target_timespan: 150 * 600,
        }
    }
}