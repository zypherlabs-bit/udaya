pub mod blockchain_db;

pub use blockchain_db::BlockchainDB;

use serde::{Deserialize, Serialize};

// Database column families
pub mod column {
    pub const BLOCKS: &str = "blocks";
    pub const BLOCK_HASHES: &str = "block_hashes";
    pub const BLOCK_HEADERS: &str = "block_headers";
    pub const BLOCK_HEIGHT_BY_HASH: &str = "block_height_by_hash";
    pub const CHAIN_WORK: &str = "chain_work";
    pub const TRANSACTIONS: &str = "transactions";
    /// Canonical transaction location: txid -> (block hash, height).
    pub const TX_LOCATIONS: &str = "tx_locations";
    pub const UTXO_SET: &str = "utxo_set";
    pub const MEMPOOL: &str = "mempool";
    pub const CHAIN_STATE: &str = "chain_state";
    pub const PEERS: &str = "peers";
    pub const WALLETS: &str = "wallets";
    pub const EXPLORER: &str = "explorer";
}

/// Database configuration for Udaya Global experimental setup
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
            data_dir: "data/udaya-global".to_string(),
            db_cache_size_mb: 512,
            max_open_files: 1000,
            enable_compression: true,
            prune_blocks: false,
            prune_target_gb: 10,
        }
    }
}

/// Database column family descriptors
pub mod column_descriptors {
    use rocksdb::Options;

    pub fn blocks_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::BLOCKS, cf_opts)
    }

    pub fn block_hashes_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::BLOCK_HASHES, cf_opts)
    }

    pub fn block_headers_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::BLOCK_HEADERS, cf_opts)
    }

    pub fn block_height_by_hash_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::BLOCK_HEIGHT_BY_HASH, cf_opts)
    }

    pub fn chain_work_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::CHAIN_WORK, cf_opts)
    }

    pub fn transactions_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::TRANSACTIONS, cf_opts)
    }

    pub fn tx_locations_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::TX_LOCATIONS, cf_opts)
    }

    pub fn utxo_set_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::UTXO_SET, cf_opts)
    }

    pub fn mempool_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::MEMPOOL, cf_opts)
    }

    pub fn chain_state_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::CHAIN_STATE, cf_opts)
    }

    pub fn peers_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::PEERS, cf_opts)
    }

    pub fn wallets_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::WALLETS, cf_opts)
    }

    pub fn explorer_cf() -> rocksdb::ColumnFamilyDescriptor {
        let cf_opts = Options::default();
        rocksdb::ColumnFamilyDescriptor::new(column::EXPLORER, cf_opts)
    }
}