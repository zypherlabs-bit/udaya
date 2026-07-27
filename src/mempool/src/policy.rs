// Udaya mempool policy module
// Transaction acceptance policy rules

use udaya_core::transaction::Transaction;
use udaya_core::types::Txid;

/// Policy configuration for mempool acceptance
pub struct MempoolPolicy {
    pub min_fee_rate: u64,
    pub max_tx_size: usize,
    pub min_standard_tx_size: usize,
    pub max_sigops_per_tx: usize,
    pub max_tx_chain_length: usize,
}

impl Default for MempoolPolicy {
    fn default() -> Self {
        Self {
            min_fee_rate: 1,
            max_tx_size: 100_000,
            min_standard_tx_size: 82,
            max_sigops_per_tx: 4_000,
            max_tx_chain_length: 25,
        }
    }
}

impl MempoolPolicy {
    /// Check if a transaction is standard according to mempool policy
    pub fn is_standard_tx(&self, tx: &Transaction) -> bool {
        // Must have at least one input and output
        if tx.inputs.is_empty() || tx.outputs.is_empty() {
            return false;
        }
        
        // Non-coinbase must be minimum size
        if !tx.is_coinbase() && tx.size() < self.min_standard_tx_size {
            return false;
        }
        
        // Size limit
        if tx.size() > self.max_tx_size {
            return false;
        }
        
        true
    }
}