pub mod address;
pub mod config;
pub mod consensus;
pub mod genesis;
pub mod script;
pub mod security;
pub mod serialization;
pub mod transaction;
pub mod types;
pub mod validation;

pub mod observability;

use parking_lot::RwLock;
use std::sync::Arc;

/// Udaya network magic bytes for message identification
pub const NETWORK_MAGIC: [u8; 4] = [0xBF, 0x59, 0x1A, 0xE7];

/// Udaya ticker symbol
pub const TICKER: &str = "UDYA";

/// Maximum supply (Bitcoin-inspired: 21 million)
pub const MAX_SUPPLY: u64 = 21_000_000;

/// Satoshis per coin (1 UDYA = 100,000,000 satoshis)
pub const SATS_PER_COIN: u64 = 100_000_000;

/// Block time target: 10 minutes
pub const BLOCK_TARGET_TIME_SECS: u64 = 600;

/// Initial block reward: 50 UDYA
pub const INITIAL_BLOCK_REWARD: u64 = 50 * SATS_PER_COIN;

/// Halving interval: 210,000 blocks
pub const HALVING_INTERVAL: u64 = 210_000;

/// Difficulty adjustment interval: 2016 blocks
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 2_016;

/// Maximum block weight (weight units)
pub const MAX_BLOCK_WEIGHT: u64 = 4_000_000;

/// Maximum block size in bytes
pub const MAX_BLOCK_SIZE: usize = 1_000_000;

/// Coinbase maturity (blocks before coinbase can be spent)
pub const COINBASE_MATURITY: u64 = 100;

/// Version byte for mainnet addresses
pub const MAINNET_ADDRESS_VERSION: u8 = 0x00;

/// Version byte for testnet addresses
pub const TESTNET_ADDRESS_VERSION: u8 = 0x6F;

/// Current protocol version
pub const PROTOCOL_VERSION: u32 = 70016;

/// Minimum protocol version supported
pub const MIN_PROTOCOL_VERSION: u32 = 70012;

/// Block version for Udaya
pub const BLOCK_VERSION: i32 = 1;

/// Transaction version
pub const TX_VERSION: i32 = 2;

/// Locktime flags
pub const LOCKTIME_THRESHOLD: u32 = 500_000_000;

/// Schnorr signature flag in witnesses
pub const SCHNORR_SIG_FLAG: u8 = 0x00;

/// Udaya network names
pub const MAINNET: &str = "mainnet";
pub const TESTNET: &str = "testnet";
pub const REGTEST: &str = "regtest";

/// The shared blockchain state
#[derive(Clone)]
pub struct BlockchainState {
    pub inner: Arc<RwLock<BlockchainInner>>,
}

pub struct BlockchainInner {
    pub chain_tip: types::BlockHash,
    pub height: u64,
    pub total_work: num_bigint::BigUint,
    pub difficulty: u64,
    pub median_time: u64,
    pub mempool_count: usize,
    pub last_block_time: u64,
}

impl Default for BlockchainState {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockchainState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(BlockchainInner {
                chain_tip: types::BlockHash::default(),
                height: 0,
                total_work: num_bigint::BigUint::from(0u64),
                difficulty: 0,
                median_time: 0,
                mempool_count: 0,
                last_block_time: 0,
            })),
        }
    }
}
