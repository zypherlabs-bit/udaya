//! Node configuration file and its validation.
//!
//! The configuration deliberately contains no private keys and no genesis
//! allocations. Keys live in a separate permission-restricted file, and the
//! genesis is a separately pinned JSON document.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;
use ug_consensus::ConsensusConfig;
use ug_core::mempool::MempoolConfig;

/// Current node configuration version.
pub const CONFIG_VERSION: u8 = 1;

/// Maximum size of `node.json`.
pub const MAX_CONFIG_BYTES: u64 = 256 * 1024;

/// Consensus timing settings (milliseconds).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConsensusSettings {
    pub propose_timeout_ms: u64,
    pub prevote_timeout_ms: u64,
    pub precommit_timeout_ms: u64,
}

impl Default for ConsensusSettings {
    fn default() -> Self {
        let config = ConsensusConfig::default();
        Self {
            propose_timeout_ms: config.propose_timeout_ms,
            prevote_timeout_ms: config.prevote_timeout_ms,
            precommit_timeout_ms: config.precommit_timeout_ms,
        }
    }
}

impl ConsensusSettings {
    pub fn to_core(&self) -> ConsensusConfig {
        ConsensusConfig {
            propose_timeout_ms: self.propose_timeout_ms,
            prevote_timeout_ms: self.prevote_timeout_ms,
            precommit_timeout_ms: self.precommit_timeout_ms,
        }
    }

    fn validate(&self) -> Result<()> {
        for (name, value) in [
            ("propose_timeout_ms", self.propose_timeout_ms),
            ("prevote_timeout_ms", self.prevote_timeout_ms),
            ("precommit_timeout_ms", self.precommit_timeout_ms),
        ] {
            if value == 0 || value > 600_000 {
                return Err(Error::Config(format!(
                    "{name} must be in 1..=600000, got {value}"
                )));
            }
        }
        Ok(())
    }
}

/// Mempool limits.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MempoolSettings {
    pub max_transactions: usize,
    pub max_bytes: usize,
    pub expiry_secs: u64,
}

impl Default for MempoolSettings {
    fn default() -> Self {
        let config = MempoolConfig::default();
        Self {
            max_transactions: config.max_transactions,
            max_bytes: config.max_bytes,
            expiry_secs: config.expiry_secs,
        }
    }
}

impl MempoolSettings {
    pub fn to_core(&self) -> MempoolConfig {
        MempoolConfig {
            max_transactions: self.max_transactions,
            max_bytes: self.max_bytes,
            expiry_secs: self.expiry_secs,
        }
    }

    fn validate(&self) -> Result<()> {
        if self.max_transactions == 0 || self.max_transactions > 1_000_000 {
            return Err(Error::Config(
                "mempool max_transactions must be in 1..=1000000".into(),
            ));
        }
        if self.max_bytes == 0 || self.max_bytes > 4 * 1024 * 1024 * 1024 {
            return Err(Error::Config(
                "mempool max_bytes must be in 1..=4 GiB".into(),
            ));
        }
        if self.expiry_secs == 0 || self.expiry_secs > 30 * 24 * 60 * 60 {
            return Err(Error::Config(
                "mempool expiry_secs must be in 1..=2592000".into(),
            ));
        }
        Ok(())
    }
}

/// Node configuration. Stored in `<data-dir>/node.json`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    pub version: u8,
    /// P2P listen address. Port 0 requests an ephemeral port (used by tests).
    pub listen: SocketAddr,
    #[serde(default)]
    pub bootstrap: Vec<SocketAddr>,
    pub max_peers: usize,
    pub ping_interval_secs: u64,
    pub peer_timeout_secs: u64,
    pub reconnect_interval_secs: u64,
    /// Optional pinned genesis hash (64 hex characters). If present, startup
    /// fails unless the locally loaded genesis matches exactly.
    #[serde(default)]
    pub genesis_hash: Option<String>,
    /// Maximum commit certificates requested per catch-up round trip.
    pub sync_limit: u32,
    /// Maximum out-of-order commit certificates buffered.
    pub max_pending_commits: usize,
    pub consensus: ConsensusSettings,
    pub mempool: MempoolSettings,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            listen: "127.0.0.1:29798".parse().expect("valid default address"),
            bootstrap: Vec::new(),
            max_peers: 64,
            ping_interval_secs: 15,
            peer_timeout_secs: 120,
            reconnect_interval_secs: 10,
            genesis_hash: None,
            sync_limit: 2,
            max_pending_commits: 256,
            consensus: ConsensusSettings::default(),
            mempool: MempoolSettings::default(),
        }
    }
}

impl NodeConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let meta = std::fs::metadata(path)?;
        if meta.len() > MAX_CONFIG_BYTES {
            return Err(Error::Config("node.json is too large".into()));
        }
        let data = std::fs::read(path)?;
        let config: Self = serde_json::from_slice(&data)?;
        config.validate()?;
        Ok(config)
    }

    /// Atomic save: temporary file, fsync, rename.
    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self)?;
        let tmp = path.with_extension(format!("json.tmp-{}", std::process::id()));
        {
            use std::io::Write;
            let mut file = std::fs::File::create(&tmp)?;
            file.write_all(&json)?;
            file.sync_all()?;
        }
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != CONFIG_VERSION {
            return Err(Error::Config(format!(
                "unsupported node config version {}",
                self.version
            )));
        }
        if self.bootstrap.len() > 256 {
            return Err(Error::Config("too many bootstrap peers".into()));
        }
        if self.bootstrap.iter().any(|address| address.port() == 0) {
            return Err(Error::Config(
                "bootstrap peers must have non-zero ports".into(),
            ));
        }
        if self.max_peers == 0 || self.max_peers > 256 {
            return Err(Error::Config("max_peers must be in 1..=256".into()));
        }
        if self.ping_interval_secs == 0 || self.ping_interval_secs > 3_600 {
            return Err(Error::Config(
                "ping_interval_secs must be in 1..=3600".into(),
            ));
        }
        if self.peer_timeout_secs < self.ping_interval_secs * 2 || self.peer_timeout_secs > 86_400 {
            return Err(Error::Config(
                "peer_timeout_secs must be at least twice ping_interval_secs and at most 86400"
                    .into(),
            ));
        }
        if self.reconnect_interval_secs == 0 || self.reconnect_interval_secs > 3_600 {
            return Err(Error::Config(
                "reconnect_interval_secs must be in 1..=3600".into(),
            ));
        }
        if self.sync_limit == 0 || self.sync_limit > ug_net::MAX_COMMITS_PER_MESSAGE as u32 {
            return Err(Error::Config(format!(
                "sync_limit must be in 1..={}",
                ug_net::MAX_COMMITS_PER_MESSAGE
            )));
        }
        if self.max_pending_commits == 0 || self.max_pending_commits > 4_096 {
            return Err(Error::Config(
                "max_pending_commits must be in 1..=4096".into(),
            ));
        }
        if let Some(hash) = &self.genesis_hash {
            if hash.len() != 64 || hex::decode(hash).map(|b| b.len()) != Ok(32) {
                return Err(Error::Config(
                    "genesis_hash must be 64 hexadecimal characters".into(),
                ));
            }
        }
        self.consensus.validate()?;
        self.mempool.validate()?;
        Ok(())
    }

    /// Parsed pinned genesis hash, if configured.
    pub fn pinned_genesis(&self) -> Result<Option<ug_core::Hash32>> {
        let Some(hash) = &self.genesis_hash else {
            return Ok(None);
        };
        let bytes = hex::decode(hash)
            .map_err(|_| Error::Config("genesis_hash is not hexadecimal".into()))?;
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(Some(out))
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_roundtrip() {
        let config = NodeConfig::default();
        config.validate().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("node.json");
        config.save(&path).unwrap();
        assert_eq!(NodeConfig::load(&path).unwrap(), config);
    }

    #[test]
    fn invalid_settings_are_refused() {
        let mut config = NodeConfig::default();
        config.max_peers = 0;
        assert!(config.validate().is_err());

        let mut config = NodeConfig::default();
        config.consensus.prevote_timeout_ms = 0;
        assert!(config.validate().is_err());

        let mut config = NodeConfig::default();
        config.sync_limit = 99;
        assert!(config.validate().is_err());

        let mut config = NodeConfig::default();
        config.genesis_hash = Some("not-hex".into());
        assert!(config.validate().is_err());

        let mut config = NodeConfig::default();
        config.peer_timeout_secs = 1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let json = r#"{"version":1,"listen":"127.0.0.1:29798","max_peers":64,"ping_interval_secs":15,"peer_timeout_secs":120,"reconnect_interval_secs":10,"sync_limit":2,"max_pending_commits":256,"consensus":{"propose_timeout_ms":3000,"prevote_timeout_ms":2000,"precommit_timeout_ms":2000},"mempool":{"max_transactions":100,"max_bytes":1000,"expiry_secs":60},"unexpected":true}"#;
        assert!(serde_json::from_str::<NodeConfig>(json).is_err());
    }
}
