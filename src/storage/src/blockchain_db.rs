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

        // Performance optimizations
        opts.set_max_background_jobs(4);
        opts.increase_parallelism(4);
        opts.set_use_fsync(false);
        opts.set_bytes_per_sync(1048576);
        opts.set_compression_type(rocksdb::DBCompressionType::Zstd);
        opts.set_bottommost_compression_type(rocksdb::DBCompressionType::Zstd);
        opts.set_memtable_prefix_bloom_ratio(0.1);
        opts.set_bloom_locality(1);
        opts.set_max_write_buffer_number(8);
        opts.set_write_buffer_size(256 * 1024 * 1024);
        // opts.set_block_cache_size(256 * 1024 * 1024);
        opts.set_table_cache_num_shard_bits(6);

        if config.enable_compression {
            opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        }

        let cfs: Vec<&str> = vec![
            column::BLOCKS,
            column::BLOCK_HASHES,
            column::BLOCK_HEADERS,
            column::BLOCK_HEIGHT_BY_HASH,
            column::CHAIN_WORK,
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

        // Store height by block hash (reverse lookup for reorg)
        batch.put_cf(
            self.cf(column::BLOCK_HEIGHT_BY_HASH),
            &hash_key,
            &height.to_be_bytes(),
        );

        // Store block header
        let header_data = bincode::serialize(&block.header)?;
        batch.put_cf(self.cf(column::BLOCK_HEADERS), &hash_key, &header_data);

        // Store individual transactions
        for tx in &block.transactions {
            let txid = tx.txid();
            let tx_data = bincode::serialize(tx)?;
            batch.put_cf(self.cf(column::TRANSACTIONS), txid.to_vec(), &tx_data);
        }

        // Store cumulative chain work for this block
        let block_work =
            udaya_core::consensus::ConsensusEngine::calculate_block_work(&block.header);
        let prev_work = if block.header.previous_block_hash.is_zero() {
            num_bigint::BigUint::from(0u32)
        } else {
            self.get_chain_work(&block.header.previous_block_hash)
                .unwrap_or(num_bigint::BigUint::from(0u32))
        };
        let cumulative_work = prev_work + block_work;
        let work_bytes = cumulative_work.to_bytes_be();
        batch.put_cf(self.cf(column::CHAIN_WORK), &hash_key, &work_bytes);

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

    /// Store multiple blocks with all their transactions in a single batch
    pub fn store_block_batch(&self, blocks: &[Block], heights: &[u64]) -> anyhow::Result<()> {
        let mut batch = WriteBatch::default();
        let cf_blocks = self.cf(column::BLOCKS);
        let cf_hashes = self.cf(column::BLOCK_HASHES);
        let cf_headers = self.cf(column::BLOCK_HEADERS);
        let cf_txs = self.cf(column::TRANSACTIONS);
        let cf_state = self.cf(column::CHAIN_STATE);

        for (block, height) in blocks.iter().zip(heights.iter()) {
            let hash = block.hash();
            let hash_key = hash.to_vec();
            let height_key = height.to_be_bytes();

            // Batch all operations
            batch.put_cf(cf_blocks, &hash_key, &bincode::serialize(block)?);
            batch.put_cf(cf_hashes, &height_key, &hash_key);
            batch.put_cf(cf_headers, &hash_key, &bincode::serialize(&block.header)?);

            for tx in &block.transactions {
                let txid = tx.txid();
                batch.put_cf(cf_txs, txid.to_vec(), &bincode::serialize(tx)?);
            }
        }

        // Update chain state to the last block
        if let Some((last_block, last_height)) = blocks.last().zip(heights.last()) {
            let last_hash = last_block.hash();
            batch.put_cf(cf_state, b"chain_tip", &last_hash.to_vec());
            batch.put_cf(cf_state, b"chain_height", &last_height.to_be_bytes());
        }

        self.db.write(batch)?;
        Ok(())
    }

    /// Store the UTXO set
    pub fn store_utxo_set(&self, utxo_set: &udaya_core::validation::UTXOSet) -> anyhow::Result<()> {
        let mut batch = WriteBatch::default();
        let cf = self.cf(column::UTXO_SET);

        // Clear existing UTXO set
        let iter = self.db.iterator_cf(cf, IteratorMode::Start);
        for item in iter {
            if let Ok((key, _)) = item {
                batch.delete_cf(cf, &key);
            }
        }

        // Store each UTXO
        for (outpoint, entry) in utxo_set.utxos.iter() {
            let mut key = Vec::new();
            key.extend_from_slice(&outpoint.txid.0);
            key.extend_from_slice(&outpoint.vout.to_be_bytes());

            let value = bincode::serialize(entry)?;
            batch.put_cf(cf, &key, &value);
        }

        self.db.write(batch)?;
        Ok(())
    }

    /// Load the UTXO set
    pub fn load_utxo_set(&self) -> anyhow::Result<udaya_core::validation::UTXOSet> {
        use udaya_core::validation::UTXOEntry;

        let mut utxo_set = udaya_core::validation::UTXOSet::new();
        let cf = self.cf(column::UTXO_SET);
        let iter = self.db.iterator_cf(cf, IteratorMode::Start);

        for item in iter {
            if let Ok((key, value)) = item {
                if key.len() >= 36 {
                    // 32 bytes txid + 4 bytes vout
                    let mut txid_bytes = [0u8; 32];
                    txid_bytes.copy_from_slice(&key[..32]);
                    let txid = udaya_core::types::Txid(txid_bytes);

                    let vout_bytes = &key[32..36];
                    let mut vout_array = [0u8; 4];
                    vout_array.copy_from_slice(vout_bytes);
                    let vout = u32::from_be_bytes(vout_array);

                    let outpoint = udaya_core::types::OutPoint::new(txid, vout);

                    if let Ok(entry) = bincode::deserialize::<UTXOEntry>(&value) {
                        utxo_set.add_utxo(outpoint, entry);
                    }
                }
            }
        }

        Ok(utxo_set)
    }

    /// Update UTXO set for a block (apply coinbase and transactions)
    pub fn update_utxo_set_for_block(
        &self,
        block: &udaya_core::types::Block,
        height: u64,
    ) -> anyhow::Result<()> {
        let mut utxo_set = self.load_utxo_set()?;

        // Apply coinbase transaction
        if let Some(coinbase) = block.coinbase_tx() {
            utxo_set.apply_coinbase(coinbase, &coinbase.txid(), height);
        }

        // Apply regular transactions
        for tx in &block.transactions[1..] {
            utxo_set.apply_transaction(tx, &tx.txid(), height);
        }

        // Store updated UTXO set
        self.store_utxo_set(&utxo_set)?;

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

    /// Get the height of a block by its hash (reverse lookup)
    pub fn get_block_height_by_hash(&self, hash: &BlockHash) -> anyhow::Result<Option<u64>> {
        let cf = self.cf(column::BLOCK_HEIGHT_BY_HASH);
        match self.db.get_cf(cf, hash.to_vec())? {
            Some(data) => {
                if data.len() >= 8 {
                    let mut height_bytes = [0u8; 8];
                    height_bytes.copy_from_slice(&data[..8]);
                    Ok(Some(u64::from_be_bytes(height_bytes)))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Get cumulative chain work for a block hash
    pub fn get_chain_work(&self, hash: &BlockHash) -> anyhow::Result<num_bigint::BigUint> {
        let cf = self.cf(column::CHAIN_WORK);
        match self.db.get_cf(cf, hash.to_vec())? {
            Some(data) => Ok(num_bigint::BigUint::from_bytes_be(&data)),
            None => Ok(num_bigint::BigUint::from(0u32)),
        }
    }

    /// Get chain work for the current tip
    pub fn get_tip_chain_work(&self) -> anyhow::Result<num_bigint::BigUint> {
        if let Some(tip) = self.get_chain_tip()? {
            self.get_chain_work(&tip)
        } else {
            Ok(num_bigint::BigUint::from(0u32))
        }
    }

    /// Remove a block and its associated data (for reorg rollback)
    pub fn remove_block(&self, hash: &BlockHash) -> anyhow::Result<Option<u64>> {
        // Get the height of this block first
        let height = self.get_block_height_by_hash(hash)?;

        let mut batch = WriteBatch::default();
        batch.delete_cf(self.cf(column::BLOCKS), hash.to_vec());
        batch.delete_cf(self.cf(column::BLOCK_HEADERS), hash.to_vec());
        batch.delete_cf(self.cf(column::BLOCK_HEIGHT_BY_HASH), hash.to_vec());
        batch.delete_cf(self.cf(column::CHAIN_WORK), hash.to_vec());

        if let Some(h) = height {
            batch.delete_cf(self.cf(column::BLOCK_HASHES), h.to_be_bytes());
        }

        self.db.write(batch)?;
        Ok(height)
    }

    /// Set the chain tip to a specific block hash and height (for reorg)
    pub fn set_chain_tip(&self, hash: &BlockHash, height: u64) -> anyhow::Result<()> {
        let mut batch = WriteBatch::default();
        batch.put_cf(self.cf(column::CHAIN_STATE), b"chain_tip", &hash.to_vec());
        batch.put_cf(
            self.cf(column::CHAIN_STATE),
            b"chain_height",
            &height.to_be_bytes(),
        );
        self.db.write(batch)?;
        Ok(())
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
    use udaya_core::types::{OutPoint, Txid};
    use udaya_core::validation::{UTXOEntry, UTXOSet};

    #[test]
    fn test_blockchain_db_basic() {
        let dir = tempdir().unwrap();
        let mut config = StorageConfig::default();
        config.data_dir = dir.path().to_str().unwrap().to_string();

        let db = BlockchainDB::open(&config).unwrap();
        assert_eq!(db.get_chain_height().unwrap(), 0);
        assert_eq!(db.block_count().unwrap(), 0);
    }

    #[test]
    fn test_utxo_persistence() {
        let dir = tempdir().unwrap();
        let mut config = StorageConfig::default();
        config.data_dir = dir.path().to_str().unwrap().to_string();

        let db = BlockchainDB::open(&config).unwrap();

        // Create a test UTXO set
        let mut utxo_set = UTXOSet::new();

        // Add some test UTXOs
        let txid1 = Txid([1u8; 32]);
        let txid2 = Txid([2u8; 32]);

        utxo_set.add_utxo(
            OutPoint::new(txid1, 0),
            UTXOEntry {
                value: 1000000,
                script_pubkey: vec![0x76, 0xa9, 0x14, 0xab, 0xcd],
                height: 100,
                is_coinbase: false,
            },
        );

        utxo_set.add_utxo(
            OutPoint::new(txid2, 1),
            UTXOEntry {
                value: 2000000,
                script_pubkey: vec![0x76, 0xa9, 0x14, 0xef, 0x01],
                height: 101,
                is_coinbase: true,
            },
        );

        // Store the UTXO set
        db.store_utxo_set(&utxo_set).unwrap();

        // Load it back
        let loaded_set = db.load_utxo_set().unwrap();

        // Verify the loaded set matches
        assert_eq!(loaded_set.len(), 2);

        let entry1 = loaded_set.get_utxo(&OutPoint::new(txid1, 0));
        assert!(entry1.is_some());
        assert_eq!(entry1.unwrap().value, 1000000);
        assert_eq!(entry1.unwrap().height, 100);

        let entry2 = loaded_set.get_utxo(&OutPoint::new(txid2, 1));
        assert!(entry2.is_some());
        assert_eq!(entry2.unwrap().value, 2000000);
        assert_eq!(entry2.unwrap().height, 101);
    }

    #[test]
    fn test_utxo_update_for_block() {
        let dir = tempdir().unwrap();
        let mut config = StorageConfig::default();
        config.data_dir = dir.path().to_str().unwrap().to_string();

        let db = BlockchainDB::open(&config).unwrap();

        // Create a genesis block
        let genesis = udaya_core::consensus::create_genesis_block();

        // Store genesis block and update UTXO set
        db.store_block(&genesis, 0).unwrap();
        db.update_utxo_set_for_block(&genesis, 0).unwrap();

        // Load UTXO set and verify it contains the coinbase output
        let utxo_set = db.load_utxo_set().unwrap();
        assert_eq!(utxo_set.len(), 1);

        let coinbase = genesis.coinbase_tx().unwrap();
        let outpoint = OutPoint::new(coinbase.txid(), 0);
        let entry = utxo_set.get_utxo(&outpoint);
        assert!(entry.is_some());
        assert!(entry.unwrap().is_coinbase);
    }
}
