use crate::types::{BlockHash};
use num_bigint::BigUint;
use std::collections::HashMap;

// Isolated experimental network identity for Udaya Global / U-ADC prototype (Phase 4).
// MUST remain distinct from protected existing Udaya mainnet and MUST refuse mainnet connections.
pub const EXPERIMENTAL_NETWORK_NAME: &str = "udaya-global-experimental";
pub const EXPERIMENTAL_CHAIN_ID: &str = "uadc-experimental-0001";
pub const EXPERIMENTAL_NETWORK_MAGIC: [u8; 4] = [0x55, 0x41, 0x44, 0x43]; // "UADC"
pub const PROTECTED_MAINNET_NETWORK_MAGIC: [u8; 4] = [0xBF, 0x59, 0x1A, 0xE7];
pub const EXPERIMENTAL_DEFAULT_P2P_PORT: u16 = 19799;
pub const EXPERIMENTAL_DEFAULT_RPC_PORT: u16 = 28332;
pub const EXPERIMENTAL_DEFAULT_DATADIR: &str = "data/udaya-global-experimental";

// Placeholder genesis hash for isolated prototype bootstrap only.
// NOT a verified/mainnet genesis; must be recomputed when a real experimental genesis block is defined/mined.
pub const EXPERIMENTAL_GENESIS_HASH: &str =
    "0000000000000000uadc0000experimental0000genesis0000placeholder00000000";

// Difficulty target for easiest/experimental genesis (research-only).
pub const GENESIS_BITS: u32 = 0x207FFFFF;
pub const MAX_TARGET: u32 = 0x1D00FFFF;

// Consensus/finality research parameters (candidate, not audited/proven).
pub const SELFISH_MINING_STALE_THRESHOLD: u64 = 2;
pub const MAX_REORG_DEPTH: u64 = 6;
pub const FINALITY_DEPTH: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Experimental,
}

impl Network {
    pub fn parse(name: &str) -> Result<Self, IsolationError> {
        match name {
            "experimental" | "udaya-global-experimental" | "uadc-experimental" => Ok(Network::Experimental),
            "mainnet" | "udaya-mainnet" | "existing-mainnet" => Err(IsolationError::ProtectedMainnetRefused),
            _ => Err(IsolationError::UnknownNetwork(name.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IsolationError {
    ProtectedMainnetRefused,
    ProtectedMainnetMagicRefused,
    UnknownNetwork(String),
}

impl std::fmt::Display for IsolationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsolationError::ProtectedMainnetRefused => write!(f, "protected existing Udaya mainnet is refused in udaya-global prototype"),
            IsolationError::ProtectedMainnetMagicRefused => write!(f, "protected existing Udaya mainnet magic [BF,59,1A,E7] is refused"),
            IsolationError::UnknownNetwork(n) => write!(f, "unknown or non-isolated network '{n}' refused (experimental only)"),
        }
    }
}

impl std::error::Error for IsolationError {}

pub fn ensure_not_protected_magic(magic: [u8; 4]) -> Result<(), IsolationError> {
    if magic == PROTECTED_MAINNET_NETWORK_MAGIC {
        Err(IsolationError::ProtectedMainnetMagicRefused)
    } else {
        Ok(())
    }
}

pub fn expected_genesis_hash(network: &str) -> Result<&'static str, IsolationError> {
    let _ = Network::parse(network)?;
    Ok(EXPERIMENTAL_GENESIS_HASH)
}

// Checkpoint blocks for chain finality (experimental scaffold).
#[derive(Clone, Default)]
pub struct Checkpoints {
    pub points: HashMap<u64, BlockHash>,
}

impl Checkpoints {
    pub fn new() -> Self { Self::default() }
    pub fn is_checkpoint(&self, height: u64) -> bool { self.points.contains_key(&height) }
    pub fn get_checkpoint(&self, height: u64) -> Option<BlockHash> { self.points.get(&height).copied() }
    pub fn add_checkpoint(&mut self, height: u64, hash: BlockHash) { self.points.insert(height, hash); }
}

/// Consensus parameters (experimental scaffold; U-ADC BFT rules are modeled separately).
#[derive(Clone, Debug)]
pub struct ConsensusParams {
    pub network_name: &'static str,
    pub chain_id: &'static str,
    pub network_magic: [u8; 4],
    pub max_block_weight: u64,
    pub pow_limit: BigUint,
    pub genesis_bits: u32,
    pub block_target_spacing_secs: u64,
    pub difficulty_adjustment_interval: u64,
    pub halving_interval: u64,
    pub coinbase_maturity: u64,
    pub target_timespan: u64,
}

impl ConsensusParams {
    pub fn for_network(network: &str) -> Result<Self, IsolationError> {
        let _ = Network::parse(network)?;
        Ok(Self::experimental())
    }

    pub fn experimental() -> Self {
        ConsensusParams {
            network_name: EXPERIMENTAL_NETWORK_NAME,
            chain_id: EXPERIMENTAL_CHAIN_ID,
            network_magic: EXPERIMENTAL_NETWORK_MAGIC,
            max_block_weight: 4_000_000,
            pow_limit: BigUint::from(GENESIS_BITS),
            genesis_bits: GENESIS_BITS,
            block_target_spacing_secs: 60,
            difficulty_adjustment_interval: 2016,
            halving_interval: 500_000,
            coinbase_maturity: 100,
            target_timespan: 2016 * 60,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConsensusEngine {
    pub checkpoints: Checkpoints,
    pub params: ConsensusParams,
}

impl ConsensusEngine {
    pub fn experimental() -> Self {
        Self { checkpoints: Checkpoints::new(), params: ConsensusParams::experimental() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_protected_mainnet() {
        assert!(matches!(Network::parse("mainnet"), Err(IsolationError::ProtectedMainnetRefused)));
        assert!(ensure_not_protected_magic(PROTECTED_MAINNET_NETWORK_MAGIC).is_err());
        assert!(expected_genesis_hash("mainnet").is_err());
    }
    #[test]
    fn experimental_identity_is_distinct() {
        let p = ConsensusParams::experimental();
        assert_eq!(p.network_magic, EXPERIMENTAL_NETWORK_MAGIC);
        assert_ne!(p.network_magic, PROTECTED_MAINNET_NETWORK_MAGIC);
        assert_eq!(p.chain_id, EXPERIMENTAL_CHAIN_ID);
    }
}
