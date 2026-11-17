//! Canonical UTXO state and atomic transaction execution.
//!
//! Every state transition is all-or-nothing. Inputs are resolved and every
//! signature is verified *before* any mutation. A missing input, a stale
//! outpoint (double spend), an invalid signature or an arithmetic error leaves
//! the set byte-for-byte unchanged.
//!
//! Udaya Global does not have a script VM and does not mint block subsidies.
//! Supply enters only through the explicitly configured genesis transaction,
//! capped by [`MAX_SUPPLY_SATS`]. Fees are burned, so applying transactions can
//! only reduce or preserve total supply.

use crate::codec::{Decoder, Encoder};
use crate::crypto::{self, Hash32};
use crate::error::{Error, Result};
use crate::tx::{OutPoint, Transaction};
use crate::{ChainParams, MAX_SUPPLY_SATS, MAX_TX_INPUTS};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A spendable output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utxo {
    pub value: u64,
    pub owner: [u8; 32],
    pub height: u64,
    pub is_genesis: bool,
}

/// The canonical unspent transaction output set.
///
/// `BTreeMap` is used deliberately: iteration order is deterministic across
/// platforms and Rust versions, which is required for identical state roots.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UtxoSet {
    map: BTreeMap<OutPoint, Utxo>,
    total_value: u64,
}

/// Receipt for one successfully applied transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedTx {
    pub txid: Hash32,
    pub fee: u64,
    pub input_value: u64,
    pub output_value: u64,
    pub created: Vec<OutPoint>,
}

impl UtxoSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn total_value(&self) -> u64 {
        self.total_value
    }

    pub fn get(&self, outpoint: &OutPoint) -> Option<&Utxo> {
        self.map.get(outpoint)
    }

    pub fn contains(&self, outpoint: &OutPoint) -> bool {
        self.map.contains_key(outpoint)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&OutPoint, &Utxo)> {
        self.map.iter()
    }

    /// Add an output. Used by genesis construction and tests.
    pub fn add_utxo(&mut self, outpoint: OutPoint, utxo: Utxo) -> Result<()> {
        if utxo.value == 0 {
            return Err(Error::State("zero-value UTXO".into()));
        }
        if self.map.contains_key(&outpoint) {
            return Err(Error::State(format!("duplicate UTXO {outpoint}")));
        }
        let new_total = self
            .total_value
            .checked_add(utxo.value)
            .ok_or_else(|| Error::State("supply overflow".into()))?;
        if new_total > MAX_SUPPLY_SATS {
            return Err(Error::State("supply cap exceeded".into()));
        }
        self.map.insert(outpoint, utxo);
        self.total_value = new_total;
        Ok(())
    }

    /// Apply the genesis transaction. Genesis is the only transaction allowed
    /// to create outputs without inputs, and it must fit inside the supply cap.
    pub fn apply_genesis(&mut self, chain: &ChainParams, tx: &Transaction) -> Result<Hash32> {
        if !tx.is_genesis() {
            return Err(Error::Genesis(
                "genesis transaction must have version 0 and no inputs".into(),
            ));
        }
        if tx.outputs.is_empty() {
            return Err(Error::Genesis("genesis has no outputs".into()));
        }
        if tx.outputs.len() > crate::MAX_TX_OUTPUTS {
            return Err(Error::Genesis("genesis has too many outputs".into()));
        }
        if tx.fee != 0 {
            return Err(Error::Genesis("genesis fee must be zero".into()));
        }
        let mut total = 0u64;
        for output in &tx.outputs {
            if output.value == 0 {
                return Err(Error::Genesis("genesis output has zero value".into()));
            }
            if output.recipient == [0u8; 32] {
                return Err(Error::Genesis("genesis output has a zero key".into()));
            }
            total = total
                .checked_add(output.value)
                .ok_or_else(|| Error::Genesis("genesis supply overflow".into()))?;
        }
        if total > MAX_SUPPLY_SATS {
            return Err(Error::Genesis(format!(
                "genesis allocation {total} exceeds supply cap {MAX_SUPPLY_SATS}"
            )));
        }

        let txid = tx.txid(chain);
        let mut additions = Vec::with_capacity(tx.outputs.len());
        for (index, output) in tx.outputs.iter().enumerate() {
            additions.push((
                OutPoint::new(txid, index as u32),
                Utxo {
                    value: output.value,
                    owner: output.recipient,
                    height: 0,
                    is_genesis: true,
                },
            ));
        }
        for (outpoint, utxo) in additions {
            self.add_utxo(outpoint, utxo)?;
        }
        Ok(txid)
    }

    /// Atomically execute one non-genesis transaction.
    pub fn apply_transaction(
        &mut self,
        chain: &ChainParams,
        tx: &Transaction,
        height: u64,
    ) -> Result<AppliedTx> {
        tx.validate_structure(chain)?;
        if tx.is_genesis() {
            return Err(Error::InvalidTransaction(
                "genesis transactions are only valid in the genesis block".into(),
            ));
        }
        if tx.inputs.len() > MAX_TX_INPUTS {
            return Err(Error::InvalidTransaction("too many inputs".into()));
        }

        // Phase 1: resolve and verify everything without mutating state.
        let mut input_utxos = Vec::with_capacity(tx.inputs.len());
        let mut input_meta = Vec::with_capacity(tx.inputs.len());
        for input in &tx.inputs {
            let utxo = self
                .map
                .get(&input.previous_output)
                .ok_or_else(|| Error::MissingInput(input.previous_output.to_string()))?
                .clone();
            input_meta.push((utxo.owner, utxo.value));
            input_utxos.push((input.previous_output, utxo));
        }

        let total_in = tx.verify_inputs(chain, &input_meta)?;
        let total_out = tx
            .outputs
            .iter()
            .try_fold(0u64, |acc, out| acc.checked_add(out.value))
            .ok_or_else(|| Error::InvalidTransaction("output overflow".into()))?;

        // Fees are burned. Total supply after the transition must remain valid.
        let after_removal = self
            .total_value
            .checked_sub(total_in)
            .ok_or_else(|| Error::State("supply accounting underflow".into()))?;
        let after_insertion = after_removal
            .checked_add(total_out)
            .ok_or_else(|| Error::State("supply accounting overflow".into()))?;
        if after_insertion > MAX_SUPPLY_SATS {
            return Err(Error::State("supply cap exceeded".into()));
        }

        let txid = tx.txid(chain);

        // Phase 2: commit. No fallible operations are allowed from here on.
        for (outpoint, _) in &input_utxos {
            self.map.remove(outpoint);
        }
        let mut created = Vec::with_capacity(tx.outputs.len());
        for (index, output) in tx.outputs.iter().enumerate() {
            let outpoint = OutPoint::new(txid, index as u32);
            self.map.insert(
                outpoint,
                Utxo {
                    value: output.value,
                    owner: output.recipient,
                    height,
                    is_genesis: false,
                },
            );
            created.push(outpoint);
        }
        self.total_value = after_insertion;

        Ok(AppliedTx {
            txid,
            fee: tx.fee,
            input_value: total_in,
            output_value: total_out,
            created,
        })
    }

    /// Deterministic state root over the sorted UTXO set.
    pub fn state_root(&self, chain: &ChainParams) -> Hash32 {
        let leaves: Vec<Hash32> = self
            .map
            .iter()
            .map(|(outpoint, utxo)| {
                let mut e = Encoder::with_capacity(36 + 8 + 32 + 8 + 1);
                outpoint.encode(&mut e);
                e.u64(utxo.value);
                e.fixed(&utxo.owner);
                e.u64(utxo.height);
                e.u8(utxo.is_genesis as u8);
                crypto::tagged_hash("ug/utxo-entry", &[chain.chain_id.as_bytes(), &e.finish()])
            })
            .collect();
        crypto::merkle_root(&leaves)
    }

    /// Canonical snapshot encoding used for crash recovery.
    pub fn snapshot_encode(&self) -> Vec<u8> {
        let mut e = Encoder::with_capacity(8 + self.map.len() * 85);
        e.u8(1); // snapshot version
        e.u32(self.map.len() as u32);
        for (outpoint, utxo) in &self.map {
            outpoint.encode(&mut e);
            e.u64(utxo.value);
            e.fixed(&utxo.owner);
            e.u64(utxo.height);
            e.u8(utxo.is_genesis as u8);
        }
        e.finish()
    }

    /// Strict snapshot decoding. Rejects duplicates, truncation, trailing
    /// bytes, zero values, and any supply-cap violation.
    pub fn snapshot_decode(chain: &ChainParams, data: &[u8], max_entries: usize) -> Result<Self> {
        let _ = chain;
        let mut d = Decoder::new(data);
        let version = d.u8()?;
        if version != 1 {
            return Err(Error::State(format!(
                "unsupported snapshot version {version}"
            )));
        }
        let count = d.len(max_entries)?;
        let mut set = UtxoSet::new();
        for _ in 0..count {
            let outpoint = OutPoint::decode(&mut d)?;
            let value = d.u64()?;
            let owner = d.fixed::<32>()?;
            let height = d.u64()?;
            let is_genesis = d.u8()? != 0;
            set.add_utxo(
                outpoint,
                Utxo {
                    value,
                    owner,
                    height,
                    is_genesis,
                },
            )?;
        }
        d.finish()?;
        Ok(set)
    }

    /// Supply audit: every output must be inside the cap and the sum must
    /// equal the tracked total.
    pub fn audit_supply(&self) -> Result<u64> {
        let mut sum = 0u64;
        for utxo in self.map.values() {
            if utxo.value == 0 {
                return Err(Error::State("zero-value UTXO in canonical set".into()));
            }
            sum = sum
                .checked_add(utxo.value)
                .ok_or_else(|| Error::State("supply overflow".into()))?;
        }
        if sum != self.total_value {
            return Err(Error::State(
                "tracked supply does not match UTXO sum".into(),
            ));
        }
        if sum > MAX_SUPPLY_SATS {
            return Err(Error::State("supply cap violated".into()));
        }
        Ok(sum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::public_key;
    use crate::tx::{TxIn, TxOut};
    use ed25519_dalek::SigningKey;

    fn chain() -> ChainParams {
        ChainParams::devnet()
    }

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn funded_set(seed: u8, value: u64) -> (UtxoSet, OutPoint, [u8; 32]) {
        let owner = public_key(&key(seed));
        let outpoint = OutPoint::new([seed; 32], 0);
        let mut set = UtxoSet::new();
        set.add_utxo(
            outpoint,
            Utxo {
                value,
                owner,
                height: 0,
                is_genesis: true,
            },
        )
        .unwrap();
        (set, outpoint, owner)
    }

    fn spend(
        chain: &ChainParams,
        seed: u8,
        outpoint: OutPoint,
        outputs: Vec<TxOut>,
        fee: u64,
    ) -> Transaction {
        let mut tx = Transaction::new(vec![TxIn::new(outpoint)], outputs, fee);
        tx.sign_input_with_chain(0, &key(seed), chain).unwrap();
        tx
    }

    #[test]
    fn applies_valid_transaction_and_burns_fee() {
        let chain = chain();
        let (mut set, outpoint, _) = funded_set(1, 100_000);
        let tx = spend(
            &chain,
            1,
            outpoint,
            vec![TxOut::new(90_000, public_key(&key(2)))],
            10_000,
        );
        let receipt = set.apply_transaction(&chain, &tx, 1).unwrap();
        assert_eq!(receipt.fee, 10_000);
        assert_eq!(set.total_value(), 90_000);
        assert!(!set.contains(&outpoint));
        assert_eq!(set.len(), 1);
        assert_eq!(set.audit_supply().unwrap(), 90_000);
    }

    #[test]
    fn double_spend_is_rejected_and_state_is_unchanged() {
        let chain = chain();
        let (mut set, outpoint, _) = funded_set(1, 100_000);
        let tx_a = spend(
            &chain,
            1,
            outpoint,
            vec![TxOut::new(90_000, public_key(&key(2)))],
            10_000,
        );
        let tx_b = spend(
            &chain,
            1,
            outpoint,
            vec![TxOut::new(90_000, public_key(&key(3)))],
            10_000,
        );
        set.apply_transaction(&chain, &tx_a, 1).unwrap();
        let before = set.clone();
        let err = set.apply_transaction(&chain, &tx_b, 1).unwrap_err();
        assert!(matches!(err, Error::MissingInput(_)));
        assert_eq!(set, before, "failed transaction must not mutate state");
    }

    #[test]
    fn partial_invalid_input_set_is_atomic() {
        let chain = chain();
        let (mut set, outpoint, _) = funded_set(1, 100_000);
        let missing = OutPoint::new([99u8; 32], 0);
        let mut tx = Transaction::new(
            vec![TxIn::new(outpoint), TxIn::new(missing)],
            vec![TxOut::new(150_000, public_key(&key(2)))],
            0,
        );
        tx.sign_input_with_chain(0, &key(1), &chain).unwrap();
        tx.sign_input_with_chain(1, &key(1), &chain).unwrap();
        let before = set.clone();
        assert!(set.apply_transaction(&chain, &tx, 1).is_err());
        assert_eq!(set, before);
    }

    #[test]
    fn changing_a_signature_cannot_change_state() {
        let chain = chain();
        let (mut set, outpoint, _) = funded_set(1, 100_000);
        let mut tx = spend(
            &chain,
            1,
            outpoint,
            vec![TxOut::new(90_000, public_key(&key(2)))],
            10_000,
        );
        tx.inputs[0].signature[0] ^= 0x01;
        let before = set.clone();
        assert!(set.apply_transaction(&chain, &tx, 1).is_err());
        assert_eq!(set, before);
    }

    #[test]
    fn state_root_is_independent_of_insertion_order() {
        let chain = chain();
        let mut a = UtxoSet::new();
        let mut b = UtxoSet::new();
        let entries = [
            (
                OutPoint::new([3u8; 32], 0),
                Utxo {
                    value: 30,
                    owner: [3u8; 32],
                    height: 1,
                    is_genesis: false,
                },
            ),
            (
                OutPoint::new([1u8; 32], 0),
                Utxo {
                    value: 10,
                    owner: [1u8; 32],
                    height: 1,
                    is_genesis: false,
                },
            ),
            (
                OutPoint::new([2u8; 32], 0),
                Utxo {
                    value: 20,
                    owner: [2u8; 32],
                    height: 1,
                    is_genesis: false,
                },
            ),
        ];
        for (op, u) in entries.iter() {
            a.add_utxo(*op, u.clone()).unwrap();
        }
        for (op, u) in entries.into_iter().rev() {
            b.add_utxo(op, u).unwrap();
        }
        assert_eq!(a.state_root(&chain), b.state_root(&chain));
    }

    #[test]
    fn state_root_changes_when_any_committed_field_changes() {
        let chain = chain();
        let (mut set, outpoint, _) = funded_set(1, 100_000);
        let root = set.state_root(&chain);
        let utxo = set.get(&outpoint).unwrap().clone();
        set.map.insert(
            outpoint,
            Utxo {
                value: utxo.value + 1,
                ..utxo
            },
        );
        assert_ne!(root, set.state_root(&chain));
    }

    #[test]
    fn snapshot_roundtrip_and_strict_decode() {
        let chain = chain();
        let (set, _, _) = funded_set(1, 100_000);
        let bytes = set.snapshot_encode();
        let restored = UtxoSet::snapshot_decode(&chain, &bytes, 1000).unwrap();
        assert_eq!(set, restored);
        assert!(UtxoSet::snapshot_decode(&chain, &bytes[..bytes.len() - 1], 1000).is_err());
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(UtxoSet::snapshot_decode(&chain, &trailing, 1000).is_err());
        assert!(UtxoSet::snapshot_decode(&chain, &bytes, 0).is_err());
    }

    #[test]
    fn supply_cap_is_enforced_for_genesis_and_additions() {
        let chain = chain();
        let mut set = UtxoSet::new();
        let tx = Transaction {
            version: crate::tx::GENESIS_TX_VERSION,
            inputs: vec![],
            outputs: vec![TxOut::new(MAX_SUPPLY_SATS, [1u8; 32])],
            fee: 0,
        };
        set.apply_genesis(&chain, &tx).unwrap();
        assert_eq!(set.total_value(), MAX_SUPPLY_SATS);

        let extra = OutPoint::new([9u8; 32], 0);
        let err = set
            .add_utxo(
                extra,
                Utxo {
                    value: 1,
                    owner: [1u8; 32],
                    height: 0,
                    is_genesis: false,
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("supply cap"));
    }

    #[test]
    fn replaying_a_confirmed_transaction_fails() {
        let chain = chain();
        let (mut set, outpoint, _) = funded_set(1, 100_000);
        let tx = spend(
            &chain,
            1,
            outpoint,
            vec![TxOut::new(90_000, public_key(&key(2)))],
            10_000,
        );
        set.apply_transaction(&chain, &tx, 1).unwrap();
        assert!(set.apply_transaction(&chain, &tx, 2).is_err());
    }
}
