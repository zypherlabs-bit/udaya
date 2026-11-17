//! Udaya Global node: durable storage, commit catch-up, P2P orchestration and
//! the U-ADC consensus event loop.
//!
//! The node is deliberately conservative:
//!
//! * Every block is fully executed against the parent state before it is
//!   voted for or persisted.
//! * A block is only persisted as final together with a verified `2f+1`
//!   pre-commit quorum certificate.
//! * Block files are written before state snapshots. A crash between the two
//!   is recovered by replaying the block log.
//! * A state snapshot is accepted only if its recomputed state root matches the
//!   committed block header and its supply audit passes.
//! * The double-sign journal is fsynced before any vote signature is produced.
//! * A restarted validator resumes at a round strictly above any vote already
//!   journaled for the start height.
//!
//! Not implemented in this release: observer nodes, validator set rotation,
//! state sync without a certificate chain, RPC, QUIC and NAT traversal. See
//! `docs/releases/RELEASE_READINESS.md`.

pub mod config;
pub mod node;
pub mod store;
pub mod verifier;

pub use config::{ConsensusSettings, MempoolSettings, NodeConfig};
pub use node::{read_disk_status, tip_of, verify_data_dir, Node, NodeHandle, NodeStatus};
pub use store::{BlockStore, StateSnapshot, StorePaths};

/// Node-layer errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("core error: {0}")]
    Core(#[from] ug_core::Error),
    #[error("consensus error: {0}")]
    Consensus(#[from] ug_consensus::Error),
    #[error("network error: {0}")]
    Net(#[from] ug_net::Error),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("storage error: {0}")]
    Store(String),
    #[error("node error: {0}")]
    Node(String),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("background task failed: {0}")]
    Join(#[from] tokio::task::JoinError),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Read a validator/peer seed from a hex file.
///
/// The file must contain exactly 64 hexadecimal characters (32 bytes), with
/// optional surrounding whitespace. No other format is accepted, so an
/// accidentally truncated or extended key can never be used silently.
pub fn load_seed(path: &std::path::Path) -> Result<[u8; 32]> {
    let text = std::fs::read_to_string(path)?;
    let text = text.trim();
    if text.len() != 64 {
        return Err(Error::Config(format!(
            "key file {} must contain 64 hex characters",
            path.display()
        )));
    }
    let bytes = hex::decode(text)
        .map_err(|_| Error::Config(format!("key file {} is not hex", path.display())))?;
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&bytes);
    if seed == [0u8; 32] {
        return Err(Error::Config("refusing an all-zero key seed".into()));
    }
    Ok(seed)
}

/// Write a seed to `path` atomically, refusing to overwrite an existing file.
///
/// On Unix the file is created with mode 0600. Windows has no equivalent
/// POSIX mode; operators must protect the data directory with ACLs.
pub fn save_seed(path: &std::path::Path, seed: &[u8; 32]) -> Result<()> {
    if path.exists() {
        return Err(Error::Config(format!(
            "refusing to overwrite existing key file {}",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = format!("{}\n", hex::encode(seed));
    let tmp = path.with_extension(format!("key.tmp-{}", std::process::id()));
    {
        use std::io::Write;
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&tmp)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_files_are_strict_and_never_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("validator.key");
        let seed = [0x42u8; 32];
        save_seed(&path, &seed).unwrap();
        assert_eq!(load_seed(&path).unwrap(), seed);
        assert!(save_seed(&path, &[0x43u8; 32]).is_err());

        std::fs::write(&path, "abcd").unwrap();
        assert!(load_seed(&path).is_err());
        std::fs::write(&path, "00".repeat(32)).unwrap();
        assert!(load_seed(&path).is_err());
    }
}
