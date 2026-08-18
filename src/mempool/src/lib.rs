use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use udaya_core::consensus::ConsensusEngine;
use udaya_core::transaction::Transaction;
use udaya_core::types::Txid;
use udaya_core::validation::{TransactionValidator, UTXOSet};

/// Mempool configuration
#[derive(Clone)]
pub struct MempoolConfig {
    pub max_tx_count: usize,
    pub max_tx_size: usize,
    pub min_fee_rate: u64,
    pub max_mempool_size_mb: usize,
    pub expiry_hours: u64,
    pub max_orphan_tx_count: usize,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_tx_count: 50_000,
            max_tx_size: 1_000_000,
            min_fee_rate: 1,
            max_mempool_size_mb: 300,
            expiry_hours: 72,
            max_orphan_tx_count: 10_000,
        }
    }
}

/// Transaction entry in the mempool with metadata
#[derive(Clone, Debug)]
pub struct TxEntry {
    pub tx: Transaction,
    pub txid: Txid,
    pub fee: u64,
    pub fee_rate: u64,
    pub size: usize,
    pub vsize: u64,
    pub time: u64,
    pub height: u64,
    pub ancestors: HashSet<Txid>,
    pub descendants: HashSet<Txid>,
}

/// The mempool (transaction memory pool)
pub struct Mempool {
    pub transactions: DashMap<Txid, TxEntry>,
    pub fee_index: parking_lot::RwLock<BTreeMap<u64, Vec<Txid>>>,
    pub orphans: DashMap<Txid, TxEntry>,
    pub utxo_set: Arc<RwLock<UTXOSet>>,
    pub config: MempoolConfig,
    pub validator: TransactionValidator,
    pub total_bytes: Arc<RwLock<usize>>,
}

impl Mempool {
    pub fn new(config: MempoolConfig, consensus: ConsensusEngine) -> Self {
        Self {
            transactions: DashMap::new(),
            fee_index: parking_lot::RwLock::new(BTreeMap::new()),
            orphans: DashMap::new(),
            utxo_set: Arc::new(RwLock::new(UTXOSet::new())),
            config,
            validator: TransactionValidator::new(consensus),
            total_bytes: Arc::new(RwLock::new(0)),
        }
    }

    /// Check if a transaction is in the mempool (or orphan pool)
    pub fn contains(&self, txid: &Txid) -> bool {
        self.transactions.contains_key(txid) || self.orphans.contains_key(txid)
    }

    pub fn submit_transaction(
        &self,
        tx: Transaction,
        current_height: u64,
        current_time: u64,
    ) -> anyhow::Result<Txid> {
        let txid = tx.txid();

        if self.transactions.contains_key(&txid) {
            anyhow::bail!("Transaction already in mempool");
        }

        self.validator.validate_transaction(&tx)?;

        if tx.size() > self.config.max_tx_size {
            anyhow::bail!("Transaction exceeds maximum size");
        }

        if self.transactions.len() >= self.config.max_tx_count {
            anyhow::bail!("Mempool full");
        }

        let vsize = tx.vsize();
        if vsize == 0 {
            anyhow::bail!("Transaction has zero virtual size");
        }

        {
            let utxo_set = self.utxo_set.read();
            self.validator.validate_transaction_context(
                &tx,
                &utxo_set,
                current_height,
                current_time,
            )?;
        }

        let mut missing_inputs = Vec::new();
        {
            let utxo_set = self.utxo_set.read();
            for input in &tx.inputs {
                if !utxo_set.contains(&input.previous_output) {
                    if !self.transactions.contains_key(&input.previous_output.txid) {
                        missing_inputs.push(input.previous_output.clone());
                    }
                }
            }
        }

        if !missing_inputs.is_empty() {
            if self.orphans.len() >= self.config.max_orphan_tx_count {
                anyhow::bail!("Orphan pool full");
            }

            let fee = self.calculate_fee(&tx, &self.utxo_set.read());
            let entry = TxEntry {
                tx: tx.clone(),
                txid,
                fee,
                fee_rate: if vsize > 0 { fee / vsize } else { 0 },
                size: tx.size(),
                vsize,
                time: current_time,
                height: current_height,
                ancestors: HashSet::new(),
                descendants: HashSet::new(),
            };

            self.orphans.insert(txid, entry);
            return Ok(txid);
        }

        self.add_transaction(tx, txid, current_height, current_time)?;

        Ok(txid)
    }

    fn add_transaction(
        &self,
        tx: Transaction,
        txid: Txid,
        current_height: u64,
        current_time: u64,
    ) -> anyhow::Result<()> {
        let fee = self.calculate_fee(&tx, &self.utxo_set.read());
        let vsize = tx.vsize();
        let fee_rate = if vsize > 0 { fee / vsize } else { 0 };

        {
            let mut utxo_set = self.utxo_set.write();
            utxo_set.apply_transaction(&tx, &txid, current_height);
        }

        let ancestors = self.find_ancestors(&tx);

        let entry = TxEntry {
            tx: tx.clone(),
            txid,
            fee,
            fee_rate,
            size: tx.size(),
            vsize,
            time: current_time,
            height: current_height,
            ancestors: ancestors.clone(),
            descendants: HashSet::new(),
        };

        for ancestor_id in &ancestors {
            if let Some(mut ancestor_entry) = self.transactions.get_mut(ancestor_id) {
                ancestor_entry.descendants.insert(txid);
            }
        }

        {
            let mut fee_index = self.fee_index.write();
            fee_index.entry(fee_rate).or_default().push(txid);
        }

        {
            let mut total_bytes = self.total_bytes.write();
            *total_bytes += tx.size();
        }

        self.transactions.insert(txid, entry);
        self.resolve_orphans(&txid, current_height, current_time);

        Ok(())
    }

    pub fn remove_transactions(&self, txs: &[Transaction]) {
        for tx in txs {
            let txid = tx.txid();
            if let Some((_, entry)) = self.transactions.remove(&txid) {
                let mut fee_index = self.fee_index.write();
                if let Some(txids) = fee_index.get_mut(&entry.fee_rate) {
                    txids.retain(|id| id != &txid);
                    if txids.is_empty() {
                        fee_index.remove(&entry.fee_rate);
                    }
                }

                let mut total_bytes = self.total_bytes.write();
                *total_bytes = total_bytes.saturating_sub(tx.size());
            }
        }
    }

    pub fn get_block_template(&self, max_weight: u64) -> Vec<Transaction> {
        let fee_index = self.fee_index.read();
        let mut selected = Vec::new();
        let mut total_weight = 0u64;

        for (_fee_rate, txids) in fee_index.iter().rev() {
            for txid in txids {
                if let Some(entry) = self.transactions.get(txid) {
                    let tx_weight = entry.tx.weight();
                    if total_weight + tx_weight <= max_weight {
                        selected.push(entry.tx.clone());
                        total_weight += tx_weight;
                    }
                }
            }
        }

        selected
    }

    pub fn get_stats(&self) -> MempoolStats {
        let total_tx = self.transactions.len();
        let total_bytes = *self.total_bytes.read();

        let mut total_fees = 0u64;
        let mut min_fee_rate = u64::MAX;
        let mut max_fee_rate = 0u64;

        for entry in self.transactions.iter() {
            total_fees += entry.fee;
            min_fee_rate = min_fee_rate.min(entry.fee_rate);
            max_fee_rate = max_fee_rate.max(entry.fee_rate);
        }

        MempoolStats {
            total_transactions: total_tx,
            total_bytes,
            total_fees,
            min_fee_rate: if min_fee_rate == u64::MAX {
                0
            } else {
                min_fee_rate
            },
            max_fee_rate,
            orphan_count: self.orphans.len(),
        }
    }

    fn calculate_fee(&self, tx: &Transaction, utxo_set: &UTXOSet) -> u64 {
        if tx.is_coinbase() {
            return 0;
        }

        let mut total_input = 0u64;
        for input in &tx.inputs {
            if let Some(utxo) = utxo_set.get_utxo(&input.previous_output) {
                total_input += utxo.value;
            } else if let Some(entry) = self.transactions.get(&input.previous_output.txid) {
                if let Some(txout) = entry.tx.outputs.get(input.previous_output.vout as usize) {
                    total_input += txout.value;
                }
            }
        }

        total_input.saturating_sub(tx.total_output())
    }

    fn find_ancestors(&self, tx: &Transaction) -> HashSet<Txid> {
        let mut ancestors = HashSet::new();
        for input in &tx.inputs {
            if self.transactions.contains_key(&input.previous_output.txid) {
                ancestors.insert(input.previous_output.txid);
                if let Some(entry) = self.transactions.get(&input.previous_output.txid) {
                    ancestors.extend(entry.ancestors.clone());
                }
            }
        }
        ancestors
    }

    fn resolve_orphans(&self, txid: &Txid, current_height: u64, current_time: u64) {
        let to_resolve: Vec<(Txid, TxEntry)> = self
            .orphans
            .iter()
            .filter(|entry| {
                entry
                    .tx
                    .inputs
                    .iter()
                    .any(|input| &input.previous_output.txid == txid)
            })
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();

        for (orphan_id, entry) in to_resolve {
            self.orphans.remove(&orphan_id);
            if let Err(e) =
                self.add_transaction(entry.tx.clone(), orphan_id, current_height, current_time)
            {
                log::warn!("Failed to resolve orphan {}: {}", orphan_id, e);
            }
        }
    }

    pub fn expire_old_transactions(&self, current_time: u64) -> Vec<Txid> {
        let expiry_seconds = self.config.expiry_hours * 3600;
        let mut expired = Vec::new();

        let to_remove: Vec<Txid> = self
            .transactions
            .iter()
            .filter(|entry| current_time.saturating_sub(entry.time) > expiry_seconds)
            .map(|entry| *entry.key())
            .collect();

        for txid in &to_remove {
            if let Some((_, entry)) = self.transactions.remove(txid) {
                expired.push(*txid);
                let mut fee_index = self.fee_index.write();
                if let Some(txids) = fee_index.get_mut(&entry.fee_rate) {
                    txids.retain(|id| id != txid);
                }
            }
        }

        expired
    }
}

#[derive(Debug, Clone)]
pub struct MempoolStats {
    pub total_transactions: usize,
    pub total_bytes: usize,
    pub total_fees: u64,
    pub min_fee_rate: u64,
    pub max_fee_rate: u64,
    pub orphan_count: usize,
}

/// Fee estimator for suggesting fee rates
pub struct FeeEstimator {
    history: Vec<FeeSample>,
    track_blocks: usize,
}

#[derive(Clone)]
struct FeeSample {
    _block_height: u64,
    _min_fee_rate: u64,
    _max_fee_rate: u64,
    avg_fee_rate: u64,
    _tx_count: usize,
}

impl FeeEstimator {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            track_blocks: 100,
        }
    }

    pub fn estimate_fee(&self, target_blocks: u64) -> u64 {
        if self.history.is_empty() {
            return 10;
        }

        let recent: Vec<&FeeSample> = self
            .history
            .iter()
            .rev()
            .take(target_blocks.min(10) as usize)
            .collect();

        if recent.is_empty() {
            return 10;
        }

        let avg: u64 = recent.iter().map(|s| s.avg_fee_rate).sum::<u64>() / recent.len() as u64;

        avg
    }

    pub fn record_block(&mut self, height: u64, txs: &[Transaction], mempool: &Mempool) {
        let mut min_fee = u64::MAX;
        let mut max_fee = 0u64;
        let mut total_fee = 0u64;
        let mut count = 0;

        for tx in txs {
            if let Some(entry) = mempool.transactions.get(&tx.txid()) {
                min_fee = min_fee.min(entry.fee_rate);
                max_fee = max_fee.max(entry.fee_rate);
                total_fee += entry.fee_rate;
                count += 1;
            }
        }

        if count > 0 {
            let sample = FeeSample {
                _block_height: height,
                _min_fee_rate: if min_fee == u64::MAX { 0 } else { min_fee },
                _max_fee_rate: max_fee,
                avg_fee_rate: total_fee / count as u64,
                _tx_count: count,
            };

            self.history.push(sample);

            if self.history.len() > self.track_blocks {
                self.history.remove(0);
            }
        }
    }
}
