//! Udaya Global core: canonical encoding, transactions, UTXO state, blocks,
//! genesis and mempool.
//!
//! This crate is the consensus-critical heart of Udaya Global. It is isolated
//! from the pre-existing Udaya proof-of-work mainnet: the chain ID, network
//! magic, address HRP, ports and genesis are all independent, and the
//! constructor refuses the protected mainnet magic.

pub mod block;
pub mod codec;
pub mod crypto;
pub mod error;
pub mod fee;
pub mod genesis;
pub mod mempool;
pub mod rewards;
pub mod state;
pub mod tx;

pub use crypto::Hash32;
pub use error::{Error, Result};

/// Wire protocol version for Udaya Global P2P.
pub const PROTOCOL_VERSION: u32 = 1;

/// Magic bytes of the existing Udaya mainnet. Udaya Global must never use
/// these. Kept here so every entry point can refuse them explicitly.
pub const PROTECTED_MAINNET_MAGIC: [u8; 4] = [0xBF, 0x59, 0x1A, 0xE7];

/// Default Udaya Global network magic ("UGL" + version 1). Deliberately
/// different in all four bytes from the existing mainnet magic.
pub const GLOBAL_MAGIC: [u8; 4] = [0x55, 0x47, 0x4C, 0x01];

/// Default chain ID for the isolated development network.
pub const DEVNET_CHAIN_ID: &str = "udaya-global-devnet-1";

/// Default address human-readable part for the development network.
pub const DEVNET_ADDRESS_HRP: &str = "ugdev";

/// Default P2P port. Distinct from the existing mainnet port (9798).
pub const DEVNET_P2P_PORT: u16 = 29798;

/// Version byte inside the address payload.
pub const ADDRESS_VERSION: u8 = 0x01;

/// Hard cap on Udaya Global supply, in smallest units.
///
/// Udaya Global is a separate chain. Its genesis allocation is explicit and
/// configured per network; it never mints beyond this cap and never imports
/// balances from the existing mainnet.
pub const MAX_SUPPLY_SATS: u64 = 2_100_000_000_000_000;

/// Smallest unit per coin, for display only.
pub const SATS_PER_COIN: u64 = 100_000_000;

/// Maximum encoded block size accepted from peers.
pub const MAX_BLOCK_BYTES: usize = 4 * 1024 * 1024;

/// Maximum encoded transaction size accepted from peers.
pub const MAX_TX_BYTES: usize = 256 * 1024;

/// Maximum inputs in a transaction.
pub const MAX_TX_INPUTS: usize = 1024;

/// Maximum outputs in a transaction.
pub const MAX_TX_OUTPUTS: usize = 1024;

/// Maximum transactions in a block.
pub const MAX_BLOCK_TXS: usize = 8_192;

/// Default minimum fee per encoded byte.
pub const DEFAULT_MIN_FEE_PER_BYTE: u64 = 1;

/// Maximum accepted clock drift for a proposed block.
pub const MAX_FUTURE_DRIFT_SECS: u64 = 120;

/// Consensus/network parameters. Every hash that matters is bound to
/// `chain_id`, so parameters are consensus identity, not metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainParams {
    pub chain_id: String,
    pub network_magic: [u8; 4],
    pub address_hrp: String,
    pub p2p_port: u16,
    pub min_fee_per_byte: u64,
    pub max_block_bytes: usize,
    pub max_tx_bytes: usize,
}

impl Default for ChainParams {
    fn default() -> Self {
        Self::devnet()
    }
}

impl ChainParams {
    pub fn devnet() -> Self {
        Self {
            chain_id: DEVNET_CHAIN_ID.to_string(),
            network_magic: GLOBAL_MAGIC,
            address_hrp: DEVNET_ADDRESS_HRP.to_string(),
            p2p_port: DEVNET_P2P_PORT,
            min_fee_per_byte: DEFAULT_MIN_FEE_PER_BYTE,
            max_block_bytes: MAX_BLOCK_BYTES,
            max_tx_bytes: MAX_TX_BYTES,
        }
    }

    /// Validate identity and isolate from the protected Udaya mainnet.
    pub fn validate(&self) -> Result<()> {
        if self.chain_id.len() < 4 || self.chain_id.len() > 64 {
            return Err(Error::Chain("chain_id must be 4..=64 bytes".into()));
        }
        if !self
            .chain_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-')
        {
            return Err(Error::Chain(
                "chain_id may contain only ASCII alphanumerics, '.', '_' and '-'".into(),
            ));
        }
        let lowered = self.chain_id.to_ascii_lowercase();
        if lowered == "mainnet" || lowered == "udaya-mainnet" || lowered == "udaya" {
            return Err(Error::Chain(
                "refusing to reuse the existing Udaya mainnet chain identity".into(),
            ));
        }
        if !lowered.starts_with("udaya-global") {
            return Err(Error::Chain(
                "Udaya Global chain_id must start with 'udaya-global'".into(),
            ));
        }
        if self.network_magic == PROTECTED_MAINNET_MAGIC {
            return Err(Error::Chain(
                "refusing the protected existing Udaya mainnet network magic".into(),
            ));
        }
        if self.network_magic == [0u8; 4] {
            return Err(Error::Chain("network magic must not be zero".into()));
        }
        if self.p2p_port == 9798 {
            return Err(Error::Chain(
                "refusing the existing Udaya mainnet P2P port 9798".into(),
            ));
        }
        if self.p2p_port == 0 {
            return Err(Error::Chain("P2P port must be non-zero".into()));
        }
        if self.address_hrp.is_empty() || self.address_hrp.len() > 16 {
            return Err(Error::Chain("address HRP must be 1..=16 bytes".into()));
        }
        if !self
            .address_hrp
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        {
            return Err(Error::Chain(
                "address HRP must be lowercase ASCII alphanumeric".into(),
            ));
        }
        if self.min_fee_per_byte == 0 {
            return Err(Error::Chain("minimum fee per byte must be non-zero".into()));
        }
        if self.max_block_bytes == 0
            || self.max_block_bytes > MAX_BLOCK_BYTES
            || self.max_tx_bytes == 0
            || self.max_tx_bytes > MAX_TX_BYTES
        {
            return Err(Error::Chain("size limits are invalid".into()));
        }
        Ok(())
    }

    /// Domain-separated chain tag used by hashes and signatures.
    pub fn chain_tag(&self) -> Hash32 {
        crypto::tagged_hash("ug/chain-tag", &[self.chain_id.as_bytes()])
    }

    /// Human-readable network name for logs and status output.
    pub fn network_name(&self) -> &str {
        &self.chain_id
    }
}

/// A monotonically increasing block height.
pub type Height = u64;

/// A consensus round within a height.
pub type Round = u32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_devnet_is_valid_and_isolated() {
        let p = ChainParams::devnet();
        p.validate().unwrap();
        assert_ne!(p.network_magic, PROTECTED_MAINNET_MAGIC);
        assert_ne!(p.p2p_port, 9798);
        assert!(p.chain_id.starts_with("udaya-global"));
    }

    #[test]
    fn protected_mainnet_identity_is_refused() {
        let mut p = ChainParams::devnet();
        p.network_magic = PROTECTED_MAINNET_MAGIC;
        assert!(p.validate().is_err());

        let mut p = ChainParams::devnet();
        p.p2p_port = 9798;
        assert!(p.validate().is_err());

        let mut p = ChainParams::devnet();
        p.chain_id = "mainnet".into();
        assert!(p.validate().is_err());

        let mut p = ChainParams::devnet();
        p.chain_id = "udaya-mainnet".into();
        assert!(p.validate().is_err());
    }

    #[test]
    fn chain_id_is_validated() {
        for bad in ["", "abc", "udaya-global space", "other-chain-1"] {
            let mut p = ChainParams::devnet();
            p.chain_id = bad.into();
            assert!(p.validate().is_err(), "accepted {bad:?}");
        }
    }
}
