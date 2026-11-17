//! Udaya Global genesis configuration.
//!
//! The genesis is an explicit, human-inspectable file. It is the only place
//! where units enter the Udaya Global supply. It is separate from the existing
//! Udaya mainnet genesis and refuses the protected mainnet identity.
//!
//! Nodes must distribute the genesis hash out of band and pin it in
//! configuration. A node that receives a different genesis (even with the same
//! chain ID) will produce different block hashes and cannot interoperate.

use crate::block::{Block, BlockHeader, BLOCK_VERSION};
use crate::crypto::{self, Hash32};
use crate::error::{Error, Result};
use crate::state::UtxoSet;
use crate::tx::{Transaction, TxOut, GENESIS_TX_VERSION};
use crate::{ChainParams, MAX_SUPPLY_SATS};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Current genesis file format version.
pub const GENESIS_VERSION: u8 = 1;

/// Maximum size of a genesis JSON file.
pub const MAX_GENESIS_FILE_BYTES: u64 = 1_000_000;

/// One genesis allocation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Allocation {
    #[serde(with = "hex32")]
    pub recipient: [u8; 32],
    pub value: u64,
}

/// Genesis configuration as stored on disk.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub version: u8,
    pub chain_id: String,
    #[serde(with = "hex4")]
    pub network_magic: [u8; 4],
    pub address_hrp: String,
    pub p2p_port: u16,
    pub min_fee_per_byte: u64,
    pub timestamp: u64,
    pub allocations: Vec<Allocation>,
    #[serde(with = "hex32_vec")]
    pub validators: Vec<[u8; 32]>,
}

/// A validated genesis, its initial state and its height-0 block.
#[derive(Clone, Debug)]
pub struct Genesis {
    config: GenesisConfig,
    block: Block,
    state: UtxoSet,
}

impl Genesis {
    /// Validate the configuration and derive the genesis block and state.
    pub fn build(config: GenesisConfig) -> Result<Self> {
        config.validate()?;
        let chain = config.chain_params();

        let tx = Transaction {
            version: GENESIS_TX_VERSION,
            inputs: vec![],
            outputs: config
                .allocations
                .iter()
                .map(|a| TxOut::new(a.value, a.recipient))
                .collect(),
            fee: 0,
        };
        let txid = tx.txid(&chain);

        let mut state = UtxoSet::new();
        state.apply_genesis(&chain, &tx)?;
        let state_root = state.state_root(&chain);
        state.audit_supply()?;

        let header = BlockHeader {
            version: BLOCK_VERSION,
            height: 0,
            previous_hash: [0u8; 32],
            timestamp: config.timestamp,
            tx_root: crypto::merkle_root(&[txid]),
            state_root,
            proposer: [0u8; 32],
            round: 0,
        };
        let block = Block {
            header,
            transactions: vec![tx],
        };
        // Round-trip through the production validator so a malformed config
        // cannot create a genesis block that no node could decode.
        block.validate_structure(&chain)?;

        Ok(Self {
            config,
            block,
            state,
        })
    }

    /// Load and validate a genesis file. If `expected_chain_id` is supplied it
    /// must match; if `expected_hash` is supplied the computed genesis hash
    /// must match exactly.
    pub fn load(
        path: &Path,
        expected_chain_id: Option<&str>,
        expected_hash: Option<Hash32>,
    ) -> Result<Self> {
        let meta = std::fs::metadata(path)?;
        if meta.len() > MAX_GENESIS_FILE_BYTES {
            return Err(Error::Genesis("genesis file is too large".into()));
        }
        let data = std::fs::read(path)?;
        let config: GenesisConfig = serde_json::from_slice(&data)?;
        if let Some(expected) = expected_chain_id {
            if config.chain_id != expected {
                return Err(Error::Genesis(format!(
                    "genesis chain id {} does not match expected {}",
                    config.chain_id, expected
                )));
            }
        }
        let genesis = Self::build(config)?;
        if let Some(expected) = expected_hash {
            let actual = genesis.hash();
            if actual != expected {
                return Err(Error::Genesis(format!(
                    "genesis hash {} does not match pinned {}",
                    hex::encode(actual),
                    hex::encode(expected)
                )));
            }
        }
        Ok(genesis)
    }

    /// Read the raw config (used by tooling and tests).
    pub fn config(&self) -> &GenesisConfig {
        &self.config
    }

    pub fn chain_params(&self) -> ChainParams {
        self.config.chain_params()
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn state(&self) -> &UtxoSet {
        &self.state
    }

    pub fn hash(&self) -> Hash32 {
        self.block.hash(&self.chain_params())
    }

    pub fn total_allocated(&self) -> u64 {
        self.state.total_value()
    }

    pub fn validators(&self) -> &[[u8; 32]] {
        &self.config.validators
    }

    /// Serialize the canonical configuration JSON.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.config).map_err(Error::from)
    }

    /// Atomic save: write a temporary file, fsync it, then rename.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = self.to_json()?;
        let tmp = path.with_extension("json.tmp");
        {
            use std::io::Write;
            let mut file = std::fs::File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
            file.sync_all()?;
        }
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

impl GenesisConfig {
    pub fn chain_params(&self) -> ChainParams {
        ChainParams {
            chain_id: self.chain_id.clone(),
            network_magic: self.network_magic,
            address_hrp: self.address_hrp.clone(),
            p2p_port: self.p2p_port,
            min_fee_per_byte: self.min_fee_per_byte,
            max_block_bytes: crate::MAX_BLOCK_BYTES,
            max_tx_bytes: crate::MAX_TX_BYTES,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != GENESIS_VERSION {
            return Err(Error::Genesis(format!(
                "unsupported genesis version {}",
                self.version
            )));
        }
        let params = self.chain_params();
        params.validate()?;

        if self.timestamp < 1_600_000_000 {
            return Err(Error::Genesis(
                "genesis timestamp is implausibly old".into(),
            ));
        }
        if self.timestamp > 4_102_444_800 {
            return Err(Error::Genesis(
                "genesis timestamp is implausibly far in the future".into(),
            ));
        }
        if self.allocations.is_empty() {
            return Err(Error::Genesis("genesis has no allocations".into()));
        }
        if self.allocations.len() > crate::MAX_TX_OUTPUTS {
            return Err(Error::Genesis("too many genesis allocations".into()));
        }
        let mut total = 0u64;
        for allocation in &self.allocations {
            if allocation.value == 0 {
                return Err(Error::Genesis("zero-value allocation".into()));
            }
            if allocation.recipient == [0u8; 32] {
                return Err(Error::Genesis("zero-key allocation".into()));
            }
            total = total
                .checked_add(allocation.value)
                .ok_or_else(|| Error::Genesis("allocation overflow".into()))?;
        }
        if total > MAX_SUPPLY_SATS {
            return Err(Error::Genesis(format!(
                "allocations total {total} exceed the {MAX_SUPPLY_SATS} cap"
            )));
        }

        if self.validators.len() < 4 {
            return Err(Error::Genesis(
                "U-ADC requires at least 4 validators (3f+1)".into(),
            ));
        }
        if self.validators.len() > 100 {
            return Err(Error::Genesis("too many validators".into()));
        }
        // Require 3f+1 membership so the documented fault bound is exact.
        let n = self.validators.len();
        let f = (n - 1) / 3;
        if n != 3 * f + 1 {
            return Err(Error::Genesis(format!(
                "validator count {n} is not of the form 3f+1"
            )));
        }
        let mut seen = std::collections::BTreeSet::new();
        for validator in &self.validators {
            if *validator == [0u8; 32] {
                return Err(Error::Genesis("zero validator key".into()));
            }
            if !seen.insert(*validator) {
                return Err(Error::Genesis("duplicate validator key".into()));
            }
        }
        Ok(())
    }
}

/// Hex encoding for a 32-byte key in JSON.
pub mod hex32 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(value))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let s = String::deserialize(d)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("expected 32 bytes"));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(out)
    }
}

/// Hex encoding for a 4-byte network magic in JSON.
pub mod hex4 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &[u8; 4], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(value))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 4], D::Error> {
        let s = String::deserialize(d)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 4 {
            return Err(serde::de::Error::custom("expected 4 bytes"));
        }
        let mut out = [0u8; 4];
        out.copy_from_slice(&bytes);
        Ok(out)
    }
}

/// Hex encoding for a vector of 32-byte keys in JSON.
pub mod hex32_vec {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &[[u8; 32]], s: S) -> Result<S::Ok, S::Error> {
        use serde::Serialize as _;
        let encoded: Vec<String> = value.iter().map(hex::encode).collect();
        encoded.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<[u8; 32]>, D::Error> {
        let encoded = Vec::<String>::deserialize(d)?;
        let mut out = Vec::with_capacity(encoded.len());
        for item in encoded {
            let bytes = hex::decode(&item).map_err(serde::de::Error::custom)?;
            if bytes.len() != 32 {
                return Err(serde::de::Error::custom("expected 32 bytes"));
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            out.push(key);
        }
        Ok(out)
    }
}

/// Convenience constructor used by `ug-node init`.
pub fn create_config(
    timestamp: u64,
    allocations: Vec<Allocation>,
    validators: Vec<[u8; 32]>,
) -> GenesisConfig {
    GenesisConfig {
        version: GENESIS_VERSION,
        chain_id: crate::DEVNET_CHAIN_ID.to_string(),
        network_magic: crate::GLOBAL_MAGIC,
        address_hrp: crate::DEVNET_ADDRESS_HRP.to_string(),
        p2p_port: crate::DEVNET_P2P_PORT,
        min_fee_per_byte: crate::DEFAULT_MIN_FEE_PER_BYTE,
        timestamp,
        allocations,
        validators,
    }
}

/// Compute the canonical genesis hash without building a full node.
pub fn genesis_hash(config: &GenesisConfig) -> Result<Hash32> {
    Ok(Genesis::build(config.clone())?.hash())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::public_key;
    use ed25519_dalek::SigningKey;
    use tempfile::tempdir;

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn config() -> GenesisConfig {
        create_config(
            1_700_000_000,
            vec![
                Allocation {
                    recipient: public_key(&key(1)),
                    value: 1_000_000,
                },
                Allocation {
                    recipient: public_key(&key(2)),
                    value: 2_000_000,
                },
            ],
            vec![
                public_key(&key(10)),
                public_key(&key(11)),
                public_key(&key(12)),
                public_key(&key(13)),
            ],
        )
    }

    #[test]
    fn genesis_is_deterministic() {
        let a = Genesis::build(config()).unwrap();
        let b = Genesis::build(config()).unwrap();
        assert_eq!(a.hash(), b.hash());
        assert_eq!(
            a.state().state_root(&a.chain_params()),
            b.state().state_root(&b.chain_params())
        );
        assert_eq!(a.total_allocated(), 3_000_000);
    }

    #[test]
    fn different_allocations_produce_different_genesis() {
        let a = Genesis::build(config()).unwrap();
        let mut c = config();
        c.allocations[0].value += 1;
        let b = Genesis::build(c).unwrap();
        assert_ne!(a.hash(), b.hash());
    }

    #[test]
    fn supply_cap_and_duplicate_validators_are_enforced() {
        let mut c = config();
        c.allocations[0].value = MAX_SUPPLY_SATS;
        assert!(Genesis::build(c).is_err());

        let mut c = config();
        c.validators[1] = c.validators[0];
        assert!(Genesis::build(c).is_err());

        let mut c = config();
        c.validators.truncate(3);
        assert!(Genesis::build(c).is_err());

        let mut c = config();
        c.validators.push(public_key(&key(14)));
        // 5 validators is not 3f+1.
        assert!(Genesis::build(c).is_err());
    }

    #[test]
    fn protected_mainnet_identity_is_refused() {
        let mut c = config();
        c.network_magic = crate::PROTECTED_MAINNET_MAGIC;
        assert!(Genesis::build(c).is_err());

        let mut c = config();
        c.chain_id = "udaya-mainnet".into();
        assert!(Genesis::build(c).is_err());
    }

    #[test]
    fn save_load_roundtrip_and_pinning() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("genesis.json");
        let genesis = Genesis::build(config()).unwrap();
        genesis.save(&path).unwrap();

        let loaded = Genesis::load(
            &path,
            Some(genesis.config().chain_id.as_str()),
            Some(genesis.hash()),
        )
        .unwrap();
        assert_eq!(loaded.hash(), genesis.hash());
        assert_eq!(loaded.config(), genesis.config());

        assert!(Genesis::load(&path, Some("udaya-global-other"), None).is_err());
        assert!(Genesis::load(&path, None, Some([0u8; 32])).is_err());
    }

    #[test]
    fn genesis_json_is_human_readable_hex() {
        let json = Genesis::build(config()).unwrap().to_json().unwrap();
        assert!(json.contains("\"chain_id\": \"udaya-global-devnet-1\""));
        assert!(json.contains("\"network_magic\": \"55474c01\""));
        assert!(json.contains("\"validators\""));
        assert!(!json.contains("\"validators\": [["));
    }
}
