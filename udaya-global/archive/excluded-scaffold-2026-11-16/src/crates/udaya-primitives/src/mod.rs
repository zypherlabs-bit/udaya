/// Udaya Primitives - Foundational types and constants
/// 
/// Core data structures used throughout the Udaya Global protocol.

/// A 256-bit hash used throughout the system
pub type BlockHash = [u8; 32];

/// A 256-bit transaction ID
pub type Txid = BlockHash;

/// Maximum supply constant
pub const MAX_SUPPLY: u64 = 21_000_000;

/// Promoter allocation (included within max supply)
pub const PROMOTER_ALLOCATION: u64 = 500_000;

/// Mineable supply budget
pub const MINEABLE_SUPPLY_BUDGET: u64 = MAX_SUPPLY - PROMOTER_ALLOCATION;

/// SATs per coin (smallest unit)
pub const SATS_PER_COIN: u64 = 100_000_000;

/// Initial block reward
pub const INITIAL_BLOCK_REWARD: u64 = 50 * SATS_PER_COIN;

/// Difficulty target for genesis block (easiest)
pub const GENESIS_BITS: u32 = 0x207FFFFF;

/// Maximum target for difficulty (minimum difficulty)
pub const MAX_TARGET: u32 = 0x1D00FFFF;

/// Anti-selfish mining: minimum time a block must wait before being propagated
pub const SELFISH_MINING_STALE_THRESHOLD: u64 = 2;

/// Maximum reorg depth for chain reorganization
pub const MAX_REORG_DEPTH: u64 = 6;

/// Number of blocks for finality checkpoints
pub const FINALITY_DEPTH: u64 = 100;

/// Committee size for U-ADC validator selection
pub const COMMITTEE_SIZE: usize = 21;

/// Committee stake minimum (UDYA)
pub const MIN_COMMITTEE_STAKE: u64 = 100_000;

/// Committee maximum weight cap (percentage of total stake)
pub const MAX_COMMITTEE_WEIGHT_PCT: u64 = 1;

/// VRF rotation interval (blocks)
pub const COMMITTEE_ROTATION_INTERVAL: u64 = 1000;

/// Finality depth for U-ADC committee attestations
pub const COMMITTEE_DEPTH: usize = 50;

/// View change timeout (seconds, experimental)
pub const VIEW_CHANGE_TIMEOUT: u64 = 300;

/// Block target spacing experimental (seconds)
pub const EXPERIMENTAL_BLOCK_TARGET_SECS: u64 = 60;

/// Block target spacing mainnet (seconds)
pub const MAINNET_BLOCK_TARGET_SECS: u64 = 600;

/// U-ADC: Threshold signature requirement (2f+1 of n committee)
pub const THRESHOLD_SIGNATURE_REQUIREMENT: usize = 14; // 2/3 of 21

/// U-ADC: Slashing penalty percentage
pub const SLASHING_PENALTY_PERCENT: u64 = 5; // 5% of stake

/// U-ADC: Minimum stake to become validator (UDYA)
pub const MIN_VALIDATOR_STAKE: u64 = 100_000;

/// Cryptographic primitive types and constants
pub mod crypto {
    /// Private key size in bytes (Ed25519)
    pub const PRIVATE_KEY_SIZE: usize = 32;
    /// Public key size in bytes (Ed25519)
    pub const PUBLIC_KEY_SIZE: usize = 32;
    /// Signature size in bytes (Ed25519)
    pub const SIGNATURE_SIZE: usize = 64;
    /// VRF proof size in bytes
    pub const VRF_PROOF_SIZE: usize = 32;
    /// VRF output size in bytes
    pub const VRF_OUTPUT_SIZE: usize = 32;

    /// Error types for cryptographic operations
    #[derive(Debug, thiserror::Error)]
    pub enum CryptoError {
        #[error("Invalid signature")]
        InvalidSignature,
        #[error("Verification failed")]
        VerificationFailed,
        #[error("Key mismatch")]
        KeyMismatch,
        #[error("Invalid input")]
        InvalidInput,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(MAX_SUPPLY, 21_000_000);
        assert_eq!(PROMOTER_ALLOCATION, 500_000);
        assert_eq!(MINEABLE_SUPPLY_BUDGET, 20_500_000);
        assert_eq!(COMMITTEE_SIZE, 21);
        assert_eq!(COMMITTEE_DEPTH, 50);
    }
}