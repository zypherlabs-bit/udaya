use crate::{column, StorageConfig};
use rocksdb::{IteratorMode, Options, WriteBatch, DB};
use std::path::Path;
use udaya_core::transaction::Transaction;
use udaya_core::types::{Block, BlockHash, BlockHeader, Txid};

/// Blockchain database using RocksDB
pub struct BlockchainDB {
    db: DB,
    #[allow(dead_code)]
    config: StorageConfig,
}

impl BlockchainDB {
    /// Open or create the blockchain database
    pub fn open(config: &StorageConfig) -> anyhow::Result<Self> {
        let db_path = Path::new(&config.data_dir).join("blockchain");
        std::fs::create_dir_all(&db_path)?;

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(config.max_open_files);
        opts.set_keep_log_file_num(1);

        if config.enable_compression {
            opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        }

        let cfs: Vec<&str> = vec![
            column::BLOCKS,
            column::BLOCK_HASHES,
            column::BLOCK_HEADERS,
            column::TRANSACTIONS,
            column::UTXO_SET,
            column::CHAIN_STATE,
        ];

        let cf_descriptors: Vec<rocksdb::ColumnFamilyDescriptor> = cfs
            .iter()
            .map(|name| {
                let cf_opts = Options::default();
                rocksdb::ColumnFamilyDescriptor::new(*name, cf_opts)
            })
            .collect();

        let db = DB::open_cf_descriptors(&opts, db_path, cf_descriptors)?;

        Ok(Self {
            db,
            config: config.clone(),
        })
    }

    /// Store a block with all its transactions
    pub fn store_block(&self, block: &Block, height: u64) -> anyhow::Result<()> {
        let hash = block.hash();
        let hash_key = hash.to_vec();
        let height_key = height.to_be_bytes();

        let mut batch = WriteBatch::default();

        // Store block by hash
        let block_data = bincode::serialize(block)?;
        batch.put_cf(self.cf(column::BLOCKS), &hash_key, &block_data);

        // Store block hash by height
        batch.put_cf(self.cf(column::BLOCK_HASHES), &height_key, &hash_key);

        // Store block header
        let header_data = bincode::serialize(&block.header)?;
        batch.put_cf(self.cf(column::BLOCK_HEADERS), &hash_key, &header_data);

        // Store individual transactions
        for tx in &block.transactions {
            let txid = tx.txid();
            let tx_data = bincode::serialize(tx)?;
            batch.put_cf(self.cf(column::TRANSACTIONS), txid.to_vec(), &tx_data);
        }

        // Update chain state
        let state_key = b"chain_tip";
        batch.put_cf(self.cf(column::CHAIN_STATE), state_key, &hash_key);

        let height_key_state = b"chain_height";
        batch.put_cf(
            self.cf(column::CHAIN_STATE),
            height_key_state,
            &height.to_be_bytes(),
        );

        self.db.write(batch)?;
        Ok(())
    }

    /// Get block by hash
    pub fn get_block(&self, hash: &BlockHash) -> anyhow::Result<Option<Block>> {
        let cf = self.cf(column::BLOCKS);
        match self.db.get_cf(cf, hash.to_vec())? {
            Some(data) => Ok(Some(bincode::deserialize(&data)?)),
            None => Ok(None),
        }
    }

    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> anyhow::Result<Option<Block>> {
        if let Some(hash) = self.get_block_hash_by_height(height)? {
            self.get_block(&hash)
        } else {
            Ok(None)
        }
    }

    /// Get block hash by height
    pub fn get_block_hash_by_height(&self, height: u64) -> anyhow::Result<Option<BlockHash>> {
        let cf = self.cf(column::BLOCK_HASHES);
        match self.db.get_cf(cf, height.to_be_bytes())? {
            Some(data) => {
                let mut hash = BlockHash::default();
                hash.0.copy_from_slice(&data[..32.min(data.len())]);
                Ok(Some(hash))
            }
            None => Ok(None),
        }
    }

    /// Get block header by hash
    pub fn get_block_header(&self, hash: &BlockHash) -> anyhow::Result<Option<BlockHeader>> {
        let cf = self.cf(column::BLOCK_HEADERS);
        match self.db.get_cf(cf, hash.to_vec())? {
            Some(data) => Ok(Some(bincode::deserialize(&data)?)),
            None => Ok(None),
        }
    }

    /// Get transaction by txid
    pub fn get_transaction(&self, txid: &Txid) -> anyhow::Result<Option<Transaction>> {
        let cf = self.cf(column::TRANSACTIONS);
        match self.db.get_cf(cf, txid.to_vec())? {
            Some(data) => Ok(Some(bincode::deserialize(&data)?)),
            None => Ok(None),
        }
    }

    /// Get chain tip hash
    pub fn get_chain_tip(&self) -> anyhow::Result<Option<BlockHash>> {
        let cf = self.cf(column::CHAIN_STATE);
        match self.db.get_cf(cf, b"chain_tip")? {
            Some(data) => {
                let mut hash = BlockHash::default();
                hash.0.copy_from_slice(&data[..32.min(data.len())]);
                Ok(Some(hash))
            }
            None => Ok(None),
        }
    }

    /// Get chain height
    pub fn get_chain_height(&self) -> anyhow::Result<u64> {
        let cf = self.cf(column::CHAIN_STATE);
        match self.db.get_cf(cf, b"chain_height")? {
            Some(data) => {
                let mut height_bytes = [0u8; 8];
                height_bytes.copy_from_slice(&data[..8.min(data.len())]);
                Ok(u64::from_be_bytes(height_bytes))
            }
            None => Ok(0),
        }
    }

    /// Check if a block exists
    pub fn block_exists(&self, hash: &BlockHash) -> anyhow::Result<bool> {
        let cf = self.cf(column::BLOCKS);
        Ok(self.db.get_cf(cf, hash.to_vec())?.is_some())
    }

    /// Get total number of blocks
    pub fn block_count(&self) -> anyhow::Result<u64> {
        let cf = self.cf(column::BLOCK_HASHES);
        let iter = self.db.iterator_cf(cf, IteratorMode::Start);
        Ok(iter.count() as u64)
    }

    /// Get all block hashes in order
    pub fn iter_blocks(&self) -> impl Iterator<Item = (u64, BlockHash)> + '_ {
        let cf = self.cf(column::BLOCK_HASHES);
        let iter = self.db.iterator_cf(&cf, IteratorMode::Start);
        iter.filter_map(|result| {
            result.ok().and_then(|(key, value)| {
                if key.len() == 8 {
                    let height = u64::from_be_bytes(key[..8].try_into().ok()?);
                    let mut hash = BlockHash::default();
                    hash.0.copy_from_slice(&value[..32.min(value.len())]);
                    Some((height, hash))
                } else {
                    None
                }
            })
        })
    }

    /// Flush the database
    pub fn flush(&self) -> anyhow::Result<()> {
        self.db.flush()?;
        Ok(())
    }

    /// Get column family handle
    fn cf(&self, name: &str) -> &rocksdb::ColumnFamily {
        self.db.cf_handle(name).expect("Column family should exist")
    }
}

impl Drop for BlockchainDB {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_blockchain_db_basic() {
        let dir = tempdir().unwrap();
        let mut config = StorageConfig::default();
        config.data_dir = dir.path().to_str().unwrap().to_string();

        let db = BlockchainDB::open(&config).unwrap();
        assert_eq!(db.get_chain_height().unwrap(), 0);
        assert_eq!(db.block_count().unwrap(), 0);
    }
}
