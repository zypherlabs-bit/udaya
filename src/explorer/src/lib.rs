use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use udaya_core::transaction::Transaction;
use udaya_core::types::*;

/// Udaya Blockchain Explorer Backend
/// Provides data for frontend explorers, mempool visualizers, and analytics dashboards.

/// Explorer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub block_height: u64,
    pub chain_tip: String,
    pub difficulty: u64,
    pub hash_rate: f64,
    pub total_transactions: u64,
    pub total_blocks: u64,
    pub mempool_tx_count: usize,
    pub mempool_size_bytes: usize,
    pub total_supply: f64,
    pub circulating_supply: f64,
    pub median_tx_fee: u64,
    pub avg_block_size: f64,
    pub avg_block_time_secs: f64,
    pub active_nodes: usize,
    pub network_hashrate_ths: f64,
}

impl Default for ChainStats {
    fn default() -> Self {
        Self {
            block_height: 0,
            chain_tip: String::new(),
            difficulty: 0,
            hash_rate: 0.0,
            total_transactions: 0,
            total_blocks: 0,
            mempool_tx_count: 0,
            mempool_size_bytes: 0,
            total_supply: 0.0,
            circulating_supply: 0.0,
            median_tx_fee: 0,
            avg_block_size: 0.0,
            avg_block_time_secs: 600.0,
            active_nodes: 0,
            network_hashrate_ths: 0.0,
        }
    }
}

/// Block summary for explorer display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub hash: String,
    pub height: u64,
    pub version: i32,
    pub previous_block_hash: String,
    pub merkle_root: String,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u32,
    pub tx_count: usize,
    pub size_kb: f64,
    pub weight: u64,
    pub miner: Option<String>,
    pub reward: u64,
    pub total_fees: u64,
    pub difficulty: u64,
}

/// Transaction summary for explorer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxSummary {
    pub txid: String,
    pub wtxid: String,
    pub version: i32,
    pub size_bytes: usize,
    pub vsize: u64,
    pub weight: u64,
    pub fee: u64,
    pub fee_rate: u64,
    pub inputs: Vec<TxInputSummary>,
    pub outputs: Vec<TxOutputSummary>,
    pub lock_time: u32,
    pub block_height: Option<u64>,
    pub block_hash: Option<String>,
    pub block_time: Option<u64>,
    pub confirmations: u64,
    pub is_coinbase: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInputSummary {
    pub txid: String,
    pub vout: u32,
    pub address: Option<String>,
    pub value: u64,
    pub script_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutputSummary {
    pub address: Option<String>,
    pub value: u64,
    pub script_type: String,
    pub spent: bool,
    pub spent_txid: Option<String>,
}

/// Address info for explorer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub balance: u64,
    pub total_received: u64,
    pub total_sent: u64,
    pub tx_count: usize,
    pub unconfirmed_balance: i64,
    pub unconfirmed_tx_count: usize,
}

/// Mempool snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolSnapshot {
    pub tx_count: usize,
    pub size_bytes: usize,
    pub total_fees: u64,
    pub min_fee_rate: u64,
    pub max_fee_rate: u64,
    pub median_fee_rate: u64,
    pub transactions: Vec<MempoolTxInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolTxInfo {
    pub txid: String,
    pub fee: u64,
    pub fee_rate: u64,
    pub size: usize,
    pub vsize: u64,
    pub time: u64,
    pub height: u64,
    pub inputs_count: usize,
    pub outputs_count: usize,
}

/// Mining analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningAnalytics {
    pub hash_rate_ths: f64,
    pub difficulty: u64,
    pub estimated_next_difficulty: u64,
    pub block_time_avg_secs: f64,
    pub blocks_until_halving: u64,
    pub current_reward_udya: f64,
    pub next_halving_height: u64,
    pub miner_diversity: Vec<MinerShare>,
    pub pool_distribution: Vec<PoolShare>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerShare {
    pub miner_address: String,
    pub blocks_mined: u64,
    pub share_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolShare {
    pub pool_name: String,
    pub blocks_mined: u64,
    pub hash_rate_ths: f64,
    pub share_percent: f64,
}

/// Governance analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceAnalytics {
    pub proposals_total: u64,
    pub proposals_active: u64,
    pub proposals_passed: u64,
    pub total_votes_cast: u64,
    pub voter_participation_percent: f64,
    pub treasury_balance: u64,
    pub recent_proposals: Vec<ProposalSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub end_height: u64,
    pub created_at: u64,
}

/// Explorer engine
pub struct ExplorerEngine {
    stats: Arc<RwLock<ChainStats>>,
    block_cache: Arc<RwLock<HashMap<u64, BlockSummary>>>,
    tx_cache: Arc<RwLock<HashMap<String, TxSummary>>>,
    _address_cache: Arc<RwLock<HashMap<String, AddressInfo>>>,
}

impl Default for ExplorerEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExplorerEngine {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(ChainStats::default())),
            block_cache: Arc::new(RwLock::new(HashMap::new())),
            tx_cache: Arc::new(RwLock::new(HashMap::new())),
            _address_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update chain statistics
    pub fn update_stats(&self, stats: ChainStats) {
        let mut s = self.stats.write();
        *s = stats;
    }

    /// Get chain stats
    pub fn get_stats(&self) -> ChainStats {
        self.stats.read().clone()
    }

    /// Cache a block summary
    pub fn cache_block(&self, height: u64, summary: BlockSummary) {
        let mut cache = self.block_cache.write();
        cache.insert(height, summary);

        // Prune cache if too large
        if cache.len() > 1000 {
            let keys: Vec<u64> = cache.keys().copied().collect();
            for key in keys.iter().take(100) {
                cache.remove(key);
            }
        }
    }

    /// Get cached block
    pub fn get_block(&self, height: u64) -> Option<BlockSummary> {
        self.block_cache.read().get(&height).cloned()
    }

    /// Cache transaction
    pub fn cache_tx(&self, txid: String, summary: TxSummary) {
        let mut cache = self.tx_cache.write();
        cache.insert(txid, summary);

        if cache.len() > 10000 {
            cache.clear();
        }
    }

    /// Get cached transaction
    pub fn get_tx(&self, txid: &str) -> Option<TxSummary> {
        self.tx_cache.read().get(txid).cloned()
    }

    /// Load blockchain data from database
    pub fn load_from_database(&self, db: &udaya_storage::BlockchainDB) -> anyhow::Result<()> {
        let chain_height = db.get_chain_height()?;
        let mut stats = ChainStats::default();
        stats.block_height = chain_height;
        stats.total_blocks = chain_height;

        // Update stats from database
        if let Some(tip) = db.get_chain_tip()? {
            stats.chain_tip = tip.to_string();
        }

        // Count total transactions (including genesis block at height 0)
        let mut total_txs = 0u64;
        for height in 0..=chain_height {
            if let Some(block) = db.get_block_by_height(height)? {
                total_txs += block.transactions.len() as u64;

                // Cache recent blocks
                if height >= chain_height.saturating_sub(100) {
                    let summary = Self::block_to_summary(&block, height, 0, None);
                    self.cache_block(height, summary);
                }
            }
        }
        stats.total_transactions = total_txs;

        self.update_stats(stats);
        Ok(())
    }

    /// Convert block to summary
    pub fn block_to_summary(
        block: &Block,
        height: u64,
        total_fees: u64,
        miner: Option<String>,
    ) -> BlockSummary {
        let coinbase = block.coinbase_tx();
        let reward = coinbase.map(|tx| tx.total_output()).unwrap_or(0);

        BlockSummary {
            hash: block.hash().to_string(),
            height,
            version: block.header.version,
            previous_block_hash: block.header.previous_block_hash.to_string(),
            merkle_root: format!("{:?}", block.header.merkle_root),
            timestamp: block.header.timestamp as u64,
            bits: block.header.bits,
            nonce: block.header.nonce,
            tx_count: block.transactions.len(),
            size_kb: block.size() as f64 / 1000.0,
            weight: block.transactions.iter().map(|tx| tx.weight()).sum(),
            miner,
            reward,
            total_fees,
            difficulty: block.header.bits as u64,
        }
    }

    /// Convert transaction to summary
    pub fn tx_to_summary(
        tx: &Transaction,
        block_height: Option<u64>,
        block_hash: Option<String>,
        block_time: Option<u64>,
        confirmations: u64,
    ) -> TxSummary {
        let txid = tx.txid();

        TxSummary {
            txid: txid.to_string(),
            wtxid: tx.wtxid().to_string(),
            version: tx.version,
            size_bytes: tx.size(),
            vsize: tx.vsize(),
            weight: tx.weight(),
            fee: 0, // calculate from context
            fee_rate: 0,
            inputs: tx
                .inputs
                .iter()
                .map(|input| TxInputSummary {
                    txid: input.previous_output.txid.to_string(),
                    vout: input.previous_output.vout,
                    address: None,
                    value: 0,
                    script_type: "pubkeyhash".to_string(),
                })
                .collect(),
            outputs: tx
                .outputs
                .iter()
                .map(|output| TxOutputSummary {
                    address: output.script_pubkey.address.clone(),
                    value: output.value,
                    script_type: "pubkeyhash".to_string(),
                    spent: false,
                    spent_txid: None,
                })
                .collect(),
            lock_time: tx.lock_time,
            block_height,
            block_hash,
            block_time,
            confirmations,
            is_coinbase: tx.is_coinbase(),
            timestamp: block_time.unwrap_or(0),
        }
    }
}

/// WebSocket event for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEvent {
    pub event_type: WsEventType,
    pub data: serde_json::Value,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsEventType {
    NewBlock,
    NewTransaction,
    MempoolUpdate,
    ChainReorg,
    DifficultyChange,
    GovernanceUpdate,
    PeerEvent,
}

impl WsEvent {
    pub fn new(event_type: WsEventType, data: serde_json::Value) -> Self {
        Self {
            event_type,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_explorer_creation() {
        let explorer = ExplorerEngine::new();
        let stats = explorer.get_stats();
        assert_eq!(stats.block_height, 0);
    }

    #[test]
    fn test_block_summary_creation() {
        use udaya_core::consensus::create_genesis_block;
        let block = create_genesis_block();

        let summary = ExplorerEngine::block_to_summary(&block, 0, 0, None);
        assert_eq!(summary.hash, block.hash().to_string());
        assert_eq!(summary.height, 0);
    }

    #[test]
    fn test_ws_event_creation() {
        let data = serde_json::json!({"test": true});
        let event = WsEvent::new(WsEventType::NewBlock, data);
        assert!(event.timestamp > 0);
    }

    #[test]
    fn test_explorer_database_connection() {
        use udaya_storage::blockchain_db::BlockchainDB;
        use udaya_storage::StorageConfig;

        let dir = tempdir().unwrap();
        let mut config = StorageConfig::default();
        config.data_dir = dir.path().to_str().unwrap().to_string();

        let db = BlockchainDB::open(&config).unwrap();
        let explorer = ExplorerEngine::new();

        // Test with empty database
        explorer.load_from_database(&db).unwrap();
        let stats = explorer.get_stats();
        assert_eq!(stats.block_height, 0);
        assert_eq!(stats.total_transactions, 0);

        // Add a genesis block
        let genesis = udaya_core::consensus::create_genesis_block();
        db.store_block(&genesis, 0).unwrap();
        db.update_utxo_set_for_block(&genesis, 0).unwrap();

        // Load data again - the explorer should now see the genesis block
        explorer.load_from_database(&db).unwrap();
        let stats = explorer.get_stats();
        assert_eq!(stats.block_height, 0);
        // Genesis block has 1 transaction, so total should be 1
        assert_eq!(stats.total_transactions, 1);
    }
}
