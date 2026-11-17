/// Udaya Storage - RocksDB blockchain database
/// Pruned and archival node storage support

use crate::primitives::*;
use rocksdb::{Options, DB, ColumnFamilyOptions, ColumnFamilyDescriptor};
use std::path::Path;

/// Blockchain database column families
pub mod column {
    pub const BLOCKS: &str = "blocks";
    pub const BLOCK_HASHES: &str = "block_hashes";
    pub const BLOCK_HEADERS: &str = "block_headers";
    pub const BLOCK_HEIGHT_BY_HASH: &str = "block_height_by_hash";
    pub const CHAIN_WORK: &str = "chain_work";
    pub const TRANSACTIONS: &str = "transactions";
    /// Canonical transaction location: txid -> (block hash, height)
    pub const TX_LOCATIONS: &str = "tx_locations";
    pub const UTXO_SET: &str = "utxo_set";
    pub const MEMPOOL: &str = "mempool";
    pub const CHAIN_STATE: &str = "chain_state";
    pub const PEERS: &str = "peers";
    pub const WALLETS: &str = "wallets";
    pub const EXPLORER: &str = "explorer";
}

/// Database configuration
#[derive(Clone, Debug)]
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
        StorageConfig {
            data_dir: "data/udaya-global".to_string(),
            db_cache_size_mb: 512,
            max_open_files: 1000,
            enable_compression: true,
            prune_blocks: false,
            prune_target_gb: 10,
        }
    }
}

/// Blockchain database
pub struct BlockchainDB {
    db: DB,
    config: StorageConfig,
}

impl BlockchainDB {
    /// Open or create blockchain database
    pub fn open(config: &StorageConfig) -> anyhow::Result<Self> {
        let db_path = Path::new(&config.data_dir).join("blockchain");
        std::fs::create_dir_all(&db_path)?;

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(config.max_open_files);
        opts.set_compression_type(rocksdb::DBCompressionType::Zstd);

        let cfs: Vec<&str> = vec![
            column::BLOCKS,
            column::BLOCK_HASHES,
            column::BLOCK_HEADERS,
            column::BLOCK_HEIGHT_BY_HASH,
            column::CHAIN_WORK,
            column::TRANSACTIONS,
            column::TX_LOCATIONS,
            column::UTXO_SET,
            column::CHAIN_STATE,
            column::PEERS,
            column::WALLETS,
            column::EXPLORER,
        ];

        let cf_descriptors: Vec<ColumnFamilyDescriptor> = cfs
            .iter()
            .map(|name| {
                let cf_opts = ColumnFamilyOptions::default();
                ColumnFamilyDescriptor::new(*name, cf_opts)
            })
            .collect();

        let db = DB::open_cf_descriptors(&opts, &db_path, cf_descriptors)?;
        let config = config.clone();

        Ok(BlockchainDB { db, config })
    }

    /// Store a block with all its transactions
    pub fn store_block(&self, block: &Block) -> anyhow::Result<()> {
        let batch = rocksdb::WriteBatch::default();
        
        // Store block by hash
        let block_data = bincode::serialize(block)?;
        batch.put_cf(self.cf(column::BLOCKS), &block.hash(), &block_data);
        
        // Store block header
        let header_data = bincode::serialize(&block.header)?;
        batch.put_cf(self.cf(column::BLOCK_HEADERS), &block.header.hash(), &header_data);
        
        // Store block hash
        batch.put_cf(self.cf(column::BLOCK_HASHES), &block.hash(), &block.height().to_be_bytes());
        
        // Store height by hash
        batch.put_cf(
            self.cf(column::BLOCK_HEIGHT_BY_HASH),
            &block.hash(),
            block.height().to_be_bytes(),
        );
        
        // Store transactions
        for (i, tx) in block.transactions.iter().enumerate() {
            let tx_data = bincode::serialize(tx)?;
            batch.put_cf(self.cf(column::TRANSACTIONS), &format!("{}:{}", block.hash().iter().take(32).collect::<Vec<u8>>(), i), &tx_data);
        }
        
        // Update UTXO set
        // ... (simplified)
        
        self.db.process_batch(batch)?;
        Ok(())
    }

    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> anyhow::Result<Option<Block>> {
        // Simplified lookup
        Ok(None)
    }

    /// Get block hash by height
    pub fn get_hash_by_height(&self, height: u64) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(None)
    }

    /// Column family accessor
    fn cf(&self, name: &str) -> rocksdb::ColumnFamily {
        // Simplified - in production would use the DB's column families
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.data_dir, "data/udaya-global");
        assert!(config.enable_compression);
    }
}