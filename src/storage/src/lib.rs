pub mod blockchain_db;

use serde::{Deserialize, Serialize};

/// Database column families
pub mod column {
    pub const BLOCKS: &str = "blocks";
    pub const BLOCK_HASHES: &str = "block_hashes";
    pub const BLOCK_HEADERS: &str = "block_headers";
    pub const TRANSACTIONS: &str = "transactions";
    pub const UTXO_SET: &str = "utxo_set";
    pub const MEMPOOL: &str = "mempool";
    pub const CHAIN_STATE: &str = "chain_state";
    pub const PEERS: &str = "peers";
    pub const WALLETS: &str = "wallets";
}

/// Database configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: String,
    pub db_cache_size_mb: usize,
    pub max_open_files: i32,
    pub enable_compression: bool,
    pub prune_blocks: bool,
    pub prune_target_gb: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: "data/Udaya".to_string(),
            db_cache_size_mb: 512,
            max_open_files: 1000,
            enable_compression: true,
            prune_blocks: false,
            prune_target_gb: 10,
        }
    }
}

/// Blockchain database error
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("RocksDB error: {0}")]
    RocksDB(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    #[error("UTXO not found")]
    UTXONotFound,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<rocksdb::Error> for DatabaseError {
    fn from(e: rocksdb::Error) -> Self {
        DatabaseError::RocksDB(e.to_string())
    }
}

impl From<Box<bincode::ErrorKind>> for DatabaseError {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        DatabaseError::Serialization(e.to_string())
    }
}
