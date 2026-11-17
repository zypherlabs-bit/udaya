//! Mempool with duplicate and double-spend protection.
//!
//! The mempool is an overlay over the canonical UTXO set. It tracks every
//! outpoint spent by a pending transaction and every output created by one.
//! A second transaction spending the same outpoint is rejected immediately,
//! before it can be relayed or included in a block template.
//!
//! There is deliberately no replace-by-fee and no orphan pool: an orphan is
//! indistinguishable from a double-spend attempt until its parent is known, so
//! unspendable transactions are rejected and the parent is fetched by the P2P
//! layer instead. This is a safety-first policy.

use crate::crypto::Hash32;
use crate::error::{Error, Result};
use crate::state::UtxoSet;
use crate::tx::{OutPoint, Transaction};
use crate::{ChainParams, Height};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Mempool limits.
#[derive(Clone, Debug)]
pub struct MempoolConfig {
    pub max_transactions: usize,
    pub max_bytes: usize,
    pub expiry_secs: u64,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_transactions: 10_000,
            max_bytes: 32 * 1024 * 1024,
            expiry_secs: 2 * 60 * 60,
        }
    }
}

/// One pending transaction.
#[derive(Clone, Debug)]
pub struct MempoolEntry {
    pub tx: Transaction,
    pub txid: Hash32,
    pub fee: u64,
    pub size: usize,
    pub received_at: u64,
    pub parents: BTreeSet<Hash32>,
}

impl MempoolEntry {
    pub fn fee_rate(&self) -> u64 {
        self.fee / self.size.max(1) as u64
    }
}

/// Pending transaction pool.
#[derive(Default)]
pub struct Mempool {
    config: MempoolConfig,
    entries: BTreeMap<Hash32, MempoolEntry>,
    spent: HashMap<OutPoint, Hash32>,
    created: HashMap<OutPoint, crate::state::Utxo>,
    removed: HashSet<OutPoint>,
    total_bytes: usize,
}

impl Mempool {
    pub fn new(config: MempoolConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &MempoolConfig {
        &self.config
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    pub fn contains(&self, txid: &Hash32) -> bool {
        self.entries.contains_key(txid)
    }

    pub fn get(&self, txid: &Hash32) -> Option<&MempoolEntry> {
        self.entries.get(txid)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Hash32, &MempoolEntry)> {
        self.entries.iter()
    }

    /// Add a transaction, atomically. Rejects duplicates, conflicts, bad
    /// signatures, insufficient fees and unknown inputs.
    pub fn submit(
        &mut self,
        chain: &ChainParams,
        base: &UtxoSet,
        tx: Transaction,
        height: Height,
        now: u64,
    ) -> Result<Hash32> {
        self.submit_inner(chain, base, tx, height, now, true)
    }

    fn submit_inner(
        &mut self,
        chain: &ChainParams,
        base: &UtxoSet,
        tx: Transaction,
        height: Height,
        now: u64,
        enforce_expiry: bool,
    ) -> Result<Hash32> {
        tx.validate_structure(chain)?;
        let txid = tx.txid(chain);
        if self.entries.contains_key(&txid) {
            return Err(Error::Mempool("duplicate transaction".into()));
        }
        let size = tx.encoded_size();
        if size > chain.max_tx_bytes {
            return Err(Error::Mempool("transaction is too large".into()));
        }
        if self.entries.len() >= self.config.max_transactions {
            return Err(Error::Mempool("mempool is full".into()));
        }
        if self.total_bytes.saturating_add(size) > self.config.max_bytes {
            return Err(Error::Mempool("mempool byte limit reached".into()));
        }
        if enforce_expiry {
            let _ = now; // expiry is enforced during reconciliation
        }

        // Resolve every input before mutating anything.
        let mut input_meta = Vec::with_capacity(tx.inputs.len());
        let mut parents = BTreeSet::new();
        let mut resolved = Vec::with_capacity(tx.inputs.len());
        for input in &tx.inputs {
            let op = input.previous_output;
            if let Some(existing) = self.spent.get(&op) {
                return Err(Error::DoubleSpend(format!(
                    "{} is already spent by pending transaction {}",
                    op,
                    hex::encode(existing)
                )));
            }
            if self.removed.contains(&op) {
                return Err(Error::DoubleSpend(format!(
                    "{} was spent by a transaction that is no longer pending",
                    op
                )));
            }
            if let Some(utxo) = base.get(&op) {
                input_meta.push((utxo.owner, utxo.value));
                resolved.push((op, utxo.clone(), None));
            } else if let Some(utxo) = self.created.get(&op) {
                let parent = op.txid;
                input_meta.push((utxo.owner, utxo.value));
                parents.insert(parent);
                resolved.push((op, utxo.clone(), Some(parent)));
            } else {
                return Err(Error::MissingInput(op.to_string()));
            }
        }

        let total_in = tx.verify_inputs(chain, &input_meta)?;
        let total_out = tx
            .outputs
            .iter()
            .try_fold(0u64, |acc, out| acc.checked_add(out.value))
            .ok_or_else(|| Error::Mempool("output overflow".into()))?;
        if total_in != total_out + tx.fee {
            return Err(Error::Mempool("value conservation mismatch".into()));
        }

        // Commit. Every operation below is infallible.
        for (op, utxo, parent) in resolved {
            self.spent.insert(op, txid);
            self.removed.insert(op);
            let _ = (utxo, parent);
        }
        for (index, output) in tx.outputs.iter().enumerate() {
            self.created.insert(
                OutPoint::new(txid, index as u32),
                crate::state::Utxo {
                    value: output.value,
                    owner: output.recipient,
                    height,
                    is_genesis: false,
                },
            );
        }
        let fee = tx.fee;
        self.total_bytes += size;
        self.entries.insert(
            txid,
            MempoolEntry {
                tx,
                txid,
                fee,
                size,
                received_at: now,
                parents,
            },
        );
        Ok(txid)
    }

    /// Drop expired transactions. Returns the removed ids.
    pub fn expire(&mut self, now: u64) -> Vec<Hash32> {
        let expired: Vec<Hash32> = self
            .entries
            .values()
            .filter(|entry| now.saturating_sub(entry.received_at) > self.config.expiry_secs)
            .map(|entry| entry.txid)
            .collect();
        if !expired.is_empty() {
            // Rebuilding is the only way to keep the overlay consistent when
            // arbitrary entries are removed.
            let remaining: Vec<MempoolEntry> = self
                .entries
                .values()
                .filter(|entry| !expired.contains(&entry.txid))
                .cloned()
                .collect();
            self.rebuild_from_entries(remaining);
        }
        expired
    }

    /// Reconcile against a new canonical state (typically after a block).
    ///
    /// All entries are re-validated against the new base in deterministic
    /// parent-before-child order. Transactions that became invalid (for
    /// example because a conflicting transaction was finalized, or because an
    /// input was spent) are dropped. This prevents stale overlay outputs from
    /// ever entering a block template.
    pub fn reconcile(&mut self, chain: &ChainParams, base: &UtxoSet, height: Height, now: u64) {
        let mut remaining: Vec<MempoolEntry> = self
            .entries
            .values()
            .filter(|entry| now.saturating_sub(entry.received_at) <= self.config.expiry_secs)
            .cloned()
            .collect();
        remaining.sort_by(|a, b| {
            b.fee_rate()
                .cmp(&a.fee_rate())
                .then_with(|| a.txid.cmp(&b.txid))
        });

        self.clear();
        let mut progress = true;
        while progress && !remaining.is_empty() {
            progress = false;
            let mut deferred = Vec::new();
            for entry in remaining {
                let received_at = entry.received_at;
                // Re-validate from the canonical base. A clone is used because
                // a rejected entry must be retained for the next dependency
                // pass; this path runs once per block, not per transaction.
                match self.submit_inner(chain, base, entry.tx.clone(), height, received_at, false) {
                    Ok(_) => progress = true,
                    Err(_) => deferred.push(entry),
                }
            }
            remaining = deferred;
            // Sort each pass so tie-breaking is independent of hash-map order.
            remaining.sort_by(|a, b| {
                b.fee_rate()
                    .cmp(&a.fee_rate())
                    .then_with(|| a.txid.cmp(&b.txid))
            });
        }
        // Any remaining entries are unspendable and are intentionally dropped.
    }

    fn rebuild_from_entries(&mut self, entries: Vec<MempoolEntry>) {
        // Used by `expire`, where the base state is not available. Rebuild the
        // overlay from the retained entries in dependency order.
        self.clear();
        let mut remaining = entries;
        let mut progress = true;
        while progress && !remaining.is_empty() {
            progress = false;
            let mut deferred = Vec::new();
            for entry in remaining {
                let mut ok = true;
                for input in &entry.tx.inputs {
                    if !self.created.contains_key(&input.previous_output)
                        && !self.spent.contains_key(&input.previous_output)
                    {
                        // Input belongs to the canonical base, which is not
                        // available here; keep the entry without re-verifying
                        // its signature. `reconcile` performs full validation
                        // against the canonical state before any template.
                        continue;
                    }
                    if self.spent.contains_key(&input.previous_output) {
                        ok = false;
                        break;
                    }
                }
                if !ok {
                    deferred.push(entry);
                    continue;
                }
                for input in &entry.tx.inputs {
                    self.spent.insert(input.previous_output, entry.txid);
                    self.removed.insert(input.previous_output);
                }
                for (index, output) in entry.tx.outputs.iter().enumerate() {
                    self.created.insert(
                        OutPoint::new(entry.txid, index as u32),
                        crate::state::Utxo {
                            value: output.value,
                            owner: output.recipient,
                            height: 0,
                            is_genesis: false,
                        },
                    );
                }
                self.total_bytes += entry.size;
                self.entries.insert(entry.txid, entry);
                progress = true;
            }
            remaining = deferred;
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.spent.clear();
        self.created.clear();
        self.removed.clear();
        self.total_bytes = 0;
    }

    /// Deterministic, dependency-aware block template.
    ///
    /// Selection order is fee rate descending, then transaction ID ascending.
    /// A child is never selected before its in-mempool parent. Every candidate
    /// is re-executed against a working copy of the canonical state, so an
    /// invalid transaction can never reach a block template even if the
    /// overlay were corrupted.
    pub fn template(
        &self,
        chain: &ChainParams,
        base: &UtxoSet,
        height: Height,
        max_block_bytes: usize,
    ) -> Vec<Transaction> {
        let overhead = 128usize;
        let budget = max_block_bytes.saturating_sub(overhead);
        let mut candidates: Vec<&MempoolEntry> = self.entries.values().collect();
        candidates.sort_by(|a, b| {
            b.fee_rate()
                .cmp(&a.fee_rate())
                .then_with(|| a.txid.cmp(&b.txid))
        });

        let mut selected: Vec<Transaction> = Vec::new();
        let mut selected_ids: HashSet<Hash32> = HashSet::new();
        let mut working = base.clone();
        let mut used = 0usize;
        let mut pending: Vec<&MempoolEntry> = candidates;

        loop {
            let mut progress = false;
            let mut deferred = Vec::new();
            for entry in pending {
                let parents_ready = entry.parents.iter().all(|p| selected_ids.contains(p));
                if !parents_ready {
                    deferred.push(entry);
                    continue;
                }
                if used.saturating_add(entry.size) > budget {
                    continue;
                }
                match working.apply_transaction(chain, &entry.tx, height) {
                    Ok(_) => {
                        used += entry.size;
                        selected_ids.insert(entry.txid);
                        selected.push(entry.tx.clone());
                        progress = true;
                    }
                    Err(_) => {
                        // Invalid against the current working state; drop it
                        // and let its descendants be dropped on the next pass.
                    }
                }
            }
            if deferred.is_empty() || !progress {
                break;
            }
            pending = deferred;
        }
        selected
    }

    /// Statistics for status output.
    pub fn stats(&self) -> MempoolStats {
        let mut total_fees = 0u64;
        let mut min_fee_rate = u64::MAX;
        let mut max_fee_rate = 0u64;
        for entry in self.entries.values() {
            total_fees = total_fees.saturating_add(entry.fee);
            min_fee_rate = min_fee_rate.min(entry.fee_rate());
            max_fee_rate = max_fee_rate.max(entry.fee_rate());
        }
        MempoolStats {
            transactions: self.entries.len(),
            bytes: self.total_bytes,
            total_fees,
            min_fee_rate: if min_fee_rate == u64::MAX {
                0
            } else {
                min_fee_rate
            },
            max_fee_rate,
        }
    }

    /// Persist pending transactions (crash recovery / restart duplicate
    /// rejection).
    pub fn snapshot_encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(1u8);
        out.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        for entry in self.entries.values() {
            out.extend_from_slice(&entry.received_at.to_le_bytes());
            let bytes = entry.tx.encode();
            out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            out.extend_from_slice(&bytes);
        }
        out
    }

    /// Strictly decode a mempool snapshot.
    pub fn snapshot_decode(
        chain: &ChainParams,
        data: &[u8],
        max_entries: usize,
    ) -> Result<Vec<(Transaction, u64)>> {
        use crate::codec::Decoder;
        let mut d = Decoder::new(data);
        let version = d.u8()?;
        if version != 1 {
            return Err(Error::Mempool(
                "unsupported mempool snapshot version".into(),
            ));
        }
        let count = d.len(max_entries)?;
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            let received_at = d.u64()?;
            let tx_bytes = d.bytes(chain.max_tx_bytes)?;
            let tx = Transaction::decode(tx_bytes, chain)?;
            out.push((tx, received_at));
        }
        d.finish()?;
        Ok(out)
    }
}

/// Mempool statistics.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MempoolStats {
    pub transactions: usize,
    pub bytes: usize,
    pub total_fees: u64,
    pub min_fee_rate: u64,
    pub max_fee_rate: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::public_key;
    use crate::state::Utxo;
    use crate::tx::{TxIn, TxOut};
    use ed25519_dalek::SigningKey;

    fn chain() -> ChainParams {
        ChainParams::devnet()
    }

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn base(seed: u8, value: u64) -> (UtxoSet, OutPoint) {
        let op = OutPoint::new([seed; 32], 0);
        let mut set = UtxoSet::new();
        set.add_utxo(
            op,
            Utxo {
                value,
                owner: public_key(&key(seed)),
                height: 0,
                is_genesis: true,
            },
        )
        .unwrap();
        (set, op)
    }

    fn tx(chain: &ChainParams, seed: u8, op: OutPoint, value: u64, fee: u64) -> Transaction {
        let mut tx = Transaction::new(
            vec![TxIn::new(op)],
            vec![TxOut::new(value, public_key(&key(seed + 1)))],
            fee,
        );
        tx.sign_input_with_chain(0, &key(seed), chain).unwrap();
        tx
    }

    #[test]
    fn accepts_valid_and_rejects_duplicate() {
        let chain = chain();
        let (base, op) = base(1, 1_000_000);
        let mut pool = Mempool::new(MempoolConfig::default());
        let tx = tx(&chain, 1, op, 900_000, 100_000);
        let id = pool.submit(&chain, &base, tx.clone(), 1, 10).unwrap();
        assert!(pool.contains(&id));
        assert_eq!(
            pool.submit(&chain, &base, tx, 1, 10)
                .unwrap_err()
                .to_string(),
            "mempool rejected transaction: duplicate transaction"
        );
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn conflicting_second_spend_is_rejected_without_mutation() {
        let chain = chain();
        let (base, op) = base(1, 1_000_000);
        let mut pool = Mempool::new(MempoolConfig::default());
        let a = tx(&chain, 1, op, 900_000, 100_000);
        let b = tx(&chain, 1, op, 800_000, 200_000);
        pool.submit(&chain, &base, a, 1, 10).unwrap();
        let before = pool.snapshot_encode();
        let err = pool.submit(&chain, &base, b, 1, 10).unwrap_err();
        assert!(matches!(err, Error::DoubleSpend(_)));
        assert_eq!(before, pool.snapshot_encode());
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn parent_child_admission_and_template_order() {
        let chain = chain();
        let (base, op) = base(1, 1_000_000);
        let mut pool = Mempool::new(MempoolConfig::default());
        let parent = tx(&chain, 1, op, 900_000, 100_000);
        let parent_id = pool.submit(&chain, &base, parent, 1, 10).unwrap();
        let mut child = Transaction::new(
            vec![TxIn::new(OutPoint::new(parent_id, 0))],
            vec![TxOut::new(800_000, public_key(&key(3)))],
            100_000,
        );
        child.sign_input_with_chain(0, &key(2), &chain).unwrap();
        pool.submit(&chain, &base, child, 1, 11).unwrap();

        let template = pool.template(&chain, &base, 1, crate::MAX_BLOCK_BYTES);
        assert_eq!(template.len(), 2);
        assert_eq!(template[0].txid(&chain), parent_id);
    }

    #[test]
    fn template_is_deterministic_and_fee_ordered() {
        let chain = chain();
        let mut base = UtxoSet::new();
        let mut ops = Vec::new();
        for seed in 1..=4u8 {
            let op = OutPoint::new([seed; 32], 0);
            base.add_utxo(
                op,
                Utxo {
                    value: 1_000_000,
                    owner: public_key(&key(seed)),
                    height: 0,
                    is_genesis: true,
                },
            )
            .unwrap();
            ops.push((seed, op));
        }
        let mut pool = Mempool::new(MempoolConfig::default());
        let mut ids = Vec::new();
        for (seed, op) in ops {
            let fee = 10_000 * seed as u64;
            let tx = tx(&chain, seed, op, 1_000_000 - fee, fee);
            ids.push(pool.submit(&chain, &base, tx, 1, 10).unwrap());
        }
        let template = pool.template(&chain, &base, 1, crate::MAX_BLOCK_BYTES);
        assert_eq!(template.len(), 4);
        let fees: Vec<u64> = template.iter().map(|tx| tx.fee).collect();
        assert_eq!(fees, vec![40_000, 30_000, 20_000, 10_000]);
        assert_eq!(
            template,
            pool.template(&chain, &base, 1, crate::MAX_BLOCK_BYTES)
        );
    }

    #[test]
    fn expiry_removes_entries() {
        let chain = chain();
        let (base, op) = base(1, 1_000_000);
        let mut pool = Mempool::new(MempoolConfig {
            expiry_secs: 10,
            ..MempoolConfig::default()
        });
        pool.submit(&chain, &base, tx(&chain, 1, op, 900_000, 100_000), 1, 100)
            .unwrap();
        assert!(pool.expire(105).is_empty());
        assert_eq!(pool.expire(111).len(), 1);
        assert!(pool.is_empty());
    }

    #[test]
    fn reconcile_drops_conflicting_and_keeps_valid_children() {
        let chain = chain();
        let (canonical, op) = base(1, 1_000_000);
        let mut pool = Mempool::new(MempoolConfig::default());
        let a = tx(&chain, 1, op, 900_000, 100_000);
        let a_id = pool.submit(&chain, &canonical, a.clone(), 1, 10).unwrap();

        // A block confirms a conflicting spend of the same outpoint.
        let mut new_base = canonical.clone();
        new_base
            .apply_transaction(&chain, &tx(&chain, 1, op, 700_000, 300_000), 1)
            .unwrap();
        pool.reconcile(&chain, &new_base, 2, 20);
        assert!(!pool.contains(&a_id));
        assert!(pool.is_empty());

        // Now test the parent-child case across a commit.
        let (base2, op2) = base(2, 1_000_000);
        let mut pool2 = Mempool::new(MempoolConfig::default());
        let parent = tx(&chain, 2, op2, 900_000, 100_000);
        let parent_id = pool2.submit(&chain, &base2, parent, 1, 10).unwrap();
        let mut child = Transaction::new(
            vec![TxIn::new(OutPoint::new(parent_id, 0))],
            vec![TxOut::new(800_000, public_key(&key(9)))],
            100_000,
        );
        child.sign_input_with_chain(0, &key(3), &chain).unwrap();
        let child_id = pool2.submit(&chain, &base2, child, 1, 11).unwrap();
        let mut new_base2 = base2.clone();
        new_base2
            .apply_transaction(&chain, &pool2.get(&parent_id).unwrap().tx.clone(), 1)
            .unwrap();
        // Nonce: the confirmed parent is now in base, so remove it from the
        // pending set by reconciling; the child input exists in base.
        pool2.reconcile(&chain, &new_base2, 2, 20);
        assert!(!pool2.contains(&parent_id));
        assert!(pool2.contains(&child_id));
    }

    #[test]
    fn snapshot_roundtrip_and_strict_decode() {
        let chain = chain();
        let (base, op) = base(1, 1_000_000);
        let mut pool = Mempool::new(MempoolConfig::default());
        pool.submit(&chain, &base, tx(&chain, 1, op, 900_000, 100_000), 1, 100)
            .unwrap();
        let bytes = pool.snapshot_encode();
        let decoded = Mempool::snapshot_decode(&chain, &bytes, 100).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].1, 100);
        assert!(Mempool::snapshot_decode(&chain, &bytes[..bytes.len() - 1], 100).is_err());
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Mempool::snapshot_decode(&chain, &trailing, 100).is_err());
        assert!(Mempool::snapshot_decode(&chain, &bytes, 0).is_err());
    }

    #[test]
    fn unknown_input_is_rejected_not_parked() {
        let chain = chain();
        let (base, _) = base(1, 1_000_000);
        let mut pool = Mempool::new(MempoolConfig::default());
        let orphan = tx(&chain, 1, OutPoint::new([99u8; 32], 0), 900_000, 100_000);
        assert!(matches!(
            pool.submit(&chain, &base, orphan, 1, 10).unwrap_err(),
            Error::MissingInput(_)
        ));
        assert!(pool.is_empty());
    }

    #[test]
    fn forged_signature_is_rejected_before_admission() {
        let chain = chain();
        let (base, op) = base(1, 1_000_000);
        let mut pool = Mempool::new(MempoolConfig::default());
        let mut tx = tx(&chain, 1, op, 900_000, 100_000);
        tx.inputs[0].signature[7] ^= 1;
        assert!(pool.submit(&chain, &base, tx, 1, 10).is_err());
        assert!(pool.is_empty());
    }
}
