/// Udaya Crypto - Cryptographic primitives and implementations
/// Uses established, audited standards only.

use sha2::{Sha256, Digest};

/// SHA-256 double hash (PoW and block hashing)
pub fn double_sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let first = hasher.finalize();
    
    let mut hasher2 = Sha256::new();
    hasher2.update(&first);
    let second = hasher2.finalize();
    
    second.to_vec()
}

/// SHA-256 single hash
pub fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// VRF output structure
pub struct VrffOutput {
    pub value: Vec<u8>,
    pub proof: Vec<u8>,
}

/// Coinbase maturity check
pub fn check_coinbase_maturity(block_height: u64, coinbase_height: u64, maturity: u64) -> bool {
    block_height >= coinbase_height + maturity
}

/// Consensus parameter limits
pub mod limits {
    pub const MAX_BLOCK_WEIGHT: u64 = 4_000_000;
    pub const MAX_BLOCK_SIZE: u64 = 1_000_000;
}

/// Currency units
pub mod units {
    pub const COIN: u64 = 100_000_000;
    pub const SAT: u64 = 1;
    pub const FEE_ECONOMICAL: u64 = 1_000;
    pub const FEE_STANDARD: u64 = 5_000;
    pub const FEE_FAST: u64 = 20_000;
}

/// Network protocol constants
pub mod network {
    pub const DEFAULT_LISTEN_PORT: u16 = 9799;
    pub const DEFAULT_RPC_PORT: u16 = 8080;
    pub const MAX_OUTBOUND_PEERS: usize = 8;
    pub const MIN_PEERS: usize = 4;
    pub const BAN_THRESHOLD: i32 = 50;
    pub const PING_INTERVAL: u64 = 120;
    pub const TIMEOUT_SECS: u64 = 300;
}

/// Consensus parameters
pub mod consensus {
    pub const EXPERIMENTIAL_GENESIS_BITS: u32 = 0x207FFFFF;
    pub const MAINNET_GENESIS_BITS: u32 = 0x1D00FFFF;
    pub const EXPERIMENTIAL_BLOCK_TARGET: u64 = 60;
    pub const MAINNET_BLOCK_TARGET: u64 = 600;
    pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 2016;
    pub const EXPERIMENTAL_HALVING_INTERVAL: u64 = 500_000;
    pub const MAINNET_HALVING_INTERVAL: u64 = 210_000;
    pub const UADC_COMMITTEE_SIZE: usize = 21;
    pub const UADC_ROTATION_INTERVAL: u64 = 1000;
    pub const UADC_FINALITY_DEPTH: usize = 50;
    pub const MIN_VALIDATOR_STAKE: u64 = 100_000;
    pub const MAX_VALIDATOR_WEIGHT_PCT: u64 = 1;
}

/// Threshold signature collection
pub struct ThresholdSignature {
    pub threshold: usize,
    pub total: usize,
    pub signatures: Vec<Vec<u8>>,
    pub block_hash: Vec<u8>,
    pub participants: Vec<usize>,
}

impl ThresholdSignature {
    pub fn is_met(&self) -> bool {
        self.signatures.len() >= self.threshold
    }
    pub fn needed(&self) -> usize {
        self.threshold - self.signatures.len()
    }
}

/// Cryptographic error type
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    #[error("Key mismatch: {0}")]
    KeyMismatch(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Randomness failure")]
    RandomnessFailure,
    #[error("Threshold not met: need {0}, have {1}")]
    ThresholdNotMet(usize, usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_double_sha256() {
        let data = b"test data";
        let hash = double_sha256(data);
        assert_eq!(hash.len(), 64); // 32 bytes * 2 for hex
    }
    
    #[test]
    fn test_sha256() {
        let data = b"test data";
        let hash = sha256(data);
        assert!(!hash.is_empty());
    }
    
    #[test]
    fn test_coinbase_maturity() {
        assert!(check_coinbase_maturity(101, 1, 100));
        assert!(!check_coinbase_maturity(50, 1, 100));
    }
    
    #[test]
    fn test_threshold_signature() {
        let ts = ThresholdSignature {
            threshold: 14,
            total: 21,
            signatures: vec![vec![0u8; 64]],
            block_hash: vec![0u8; 32],
            participants: vec![0],
        };
        assert!(!ts.is_met());
        assert_eq!(ts.needed(), 13);
    }
}