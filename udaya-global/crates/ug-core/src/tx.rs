//! Udaya Global transaction model.
//!
//! The model is intentionally UTXO-based. Every input references exactly one
//! previous output and carries one Ed25519 signature. A signature commits to:
//!
//! * the chain ID (cross-chain and mainnet replay protection),
//! * the input index,
//! * the owner public key of the spent output,
//! * every input outpoint,
//! * every output value and recipient,
//! * the explicit fee.
//!
//! There is no script VM, so there is no script-related attack surface. An
//! output is spendable only by the Ed25519 key named in `recipient`.
//!
//! Fees are explicit. A valid transaction must satisfy
//! `sum(inputs) == sum(outputs) + fee` exactly, so there is no ambiguity about
//! how much was paid. The fee is burned (it is not credited to any account),
//! which means transaction processing can never create new units.

use crate::codec::{Decoder, Encoder};
use crate::crypto::{self, Hash32, SignatureBytes};
use crate::error::{Error, Result};
use crate::{ChainParams, MAX_SUPPLY_SATS, MAX_TX_INPUTS, MAX_TX_OUTPUTS};
use ed25519_dalek::SigningKey;
use std::fmt;

/// Current transaction version.
pub const TX_VERSION: u8 = 1;
/// Genesis transaction version; not valid in normal blocks.
pub const GENESIS_TX_VERSION: u8 = 0;

/// A reference to a specific transaction output.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutPoint {
    pub txid: Hash32,
    pub index: u32,
}

impl OutPoint {
    pub fn new(txid: Hash32, index: u32) -> Self {
        Self { txid, index }
    }

    pub fn encode(&self, e: &mut Encoder) {
        e.fixed(&self.txid);
        e.u32(self.index);
    }

    pub fn decode(d: &mut Decoder<'_>) -> Result<Self> {
        Ok(Self {
            txid: d.fixed::<32>()?,
            index: d.u32()?,
        })
    }

    pub fn to_bytes(&self) -> [u8; 36] {
        let mut out = [0u8; 36];
        out[..32].copy_from_slice(&self.txid);
        out[32..].copy_from_slice(&self.index.to_le_bytes());
        out
    }
}

impl fmt::Debug for OutPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", hex::encode(self.txid), self.index)
    }
}

impl fmt::Display for OutPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", hex::encode(self.txid), self.index)
    }
}

/// A transaction input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxIn {
    pub previous_output: OutPoint,
    pub signature: SignatureBytes,
}

impl TxIn {
    pub fn new(previous_output: OutPoint) -> Self {
        Self {
            previous_output,
            signature: [0u8; 64],
        }
    }
}

/// A transaction output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TxOut {
    pub value: u64,
    pub recipient: [u8; 32],
}

impl TxOut {
    pub fn new(value: u64, recipient: [u8; 32]) -> Self {
        Self { value, recipient }
    }
}

/// A Udaya Global transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub version: u8,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub fee: u64,
}

impl Transaction {
    pub fn new(inputs: Vec<TxIn>, outputs: Vec<TxOut>, fee: u64) -> Self {
        Self {
            version: TX_VERSION,
            inputs,
            outputs,
            fee,
        }
    }

    pub fn is_genesis(&self) -> bool {
        self.version == GENESIS_TX_VERSION && self.inputs.is_empty()
    }

    /// Canonical encoding. The same value always produces the same bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut e = Encoder::with_capacity(9 + self.inputs.len() * 100 + self.outputs.len() * 40);
        e.u8(self.version);
        e.u32(self.inputs.len() as u32);
        for input in &self.inputs {
            input.previous_output.encode(&mut e);
            e.fixed(&input.signature);
        }
        e.u32(self.outputs.len() as u32);
        for output in &self.outputs {
            e.u64(output.value);
            e.fixed(&output.recipient);
        }
        e.u64(self.fee);
        e.finish()
    }

    /// Strict decode. Rejects anything that is not the unique canonical
    /// encoding of a structurally valid transaction.
    pub fn decode(data: &[u8], chain: &ChainParams) -> Result<Self> {
        let tx = Self::decode_structural(data, chain)?;
        tx.validate_structure(chain)?;
        Ok(tx)
    }

    /// Decode a genesis transaction (version 0, no inputs). This is only
    /// reachable while decoding the height-0 block; normal callers must use
    /// [`Transaction::decode`], which rejects version 0.
    pub fn decode_genesis(data: &[u8], chain: &ChainParams) -> Result<Self> {
        let tx = Self::decode_structural(data, chain)?;
        if !tx.is_genesis() {
            return Err(Error::InvalidTransaction(
                "not a genesis transaction".into(),
            ));
        }
        Ok(tx)
    }

    fn decode_structural(data: &[u8], chain: &ChainParams) -> Result<Self> {
        if data.len() > chain.max_tx_bytes {
            return Err(Error::InvalidTransaction(format!(
                "encoded size {} exceeds limit {}",
                data.len(),
                chain.max_tx_bytes
            )));
        }
        // A transaction needs at least version+counts+fee.
        if data.len() < 1 + 4 + 4 + 8 {
            return Err(Error::InvalidTransaction("encoding is too short".into()));
        }
        let mut d = Decoder::new(data);
        let version = d.u8()?;
        let input_count = d.len(MAX_TX_INPUTS)?;
        let mut inputs = Vec::with_capacity(input_count);
        for _ in 0..input_count {
            let previous_output = OutPoint::decode(&mut d)?;
            let signature = d.fixed::<64>()?;
            inputs.push(TxIn {
                previous_output,
                signature,
            });
        }
        let output_count = d.len(MAX_TX_OUTPUTS)?;
        let mut outputs = Vec::with_capacity(output_count);
        for _ in 0..output_count {
            let value = d.u64()?;
            let recipient = d.fixed::<32>()?;
            outputs.push(TxOut { value, recipient });
        }
        let fee = d.u64()?;
        d.finish()?;
        Ok(Self {
            version,
            inputs,
            outputs,
            fee,
        })
    }

    /// Structural validation and the consensus size/fee rules.
    pub fn validate_structure(&self, chain: &ChainParams) -> Result<()> {
        if self.version != TX_VERSION {
            return Err(Error::InvalidTransaction(format!(
                "unsupported version {}",
                self.version
            )));
        }
        if self.inputs.is_empty() {
            return Err(Error::InvalidTransaction("no inputs".into()));
        }
        if self.inputs.len() > MAX_TX_INPUTS {
            return Err(Error::InvalidTransaction("too many inputs".into()));
        }
        if self.outputs.is_empty() {
            return Err(Error::InvalidTransaction("no outputs".into()));
        }
        if self.outputs.len() > MAX_TX_OUTPUTS {
            return Err(Error::InvalidTransaction("too many outputs".into()));
        }

        let mut seen = std::collections::BTreeSet::new();
        for input in &self.inputs {
            if !seen.insert(input.previous_output) {
                return Err(Error::InvalidTransaction(format!(
                    "duplicate input {}",
                    input.previous_output
                )));
            }
        }

        let mut total_out = 0u64;
        for output in &self.outputs {
            if output.value == 0 {
                return Err(Error::InvalidTransaction(
                    "zero-value output is not allowed".into(),
                ));
            }
            total_out = total_out
                .checked_add(output.value)
                .ok_or_else(|| Error::InvalidTransaction("output value overflow".into()))?;
        }
        let commitment = total_out
            .checked_add(self.fee)
            .ok_or_else(|| Error::InvalidTransaction("fee overflow".into()))?;
        if commitment > MAX_SUPPLY_SATS {
            return Err(Error::InvalidTransaction(
                "outputs plus fee exceed maximum supply".into(),
            ));
        }

        let size = self.encoded_size();
        if size > chain.max_tx_bytes {
            return Err(Error::InvalidTransaction(format!(
                "size {size} exceeds limit {}",
                chain.max_tx_bytes
            )));
        }
        let min_fee = (size as u64)
            .checked_mul(chain.min_fee_per_byte)
            .ok_or_else(|| Error::InvalidTransaction("fee calculation overflow".into()))?;
        if self.fee < min_fee {
            return Err(Error::InvalidTransaction(format!(
                "fee {} is below minimum {} for {size} bytes",
                self.fee, min_fee
            )));
        }
        Ok(())
    }

    pub fn encoded_size(&self) -> usize {
        self.encode().len()
    }

    pub fn vsize(&self) -> u64 {
        self.encoded_size() as u64
    }

    /// Chain-bound transaction identifier (SHA-256d of the canonical encoding,
    /// domain-separated and bound to the chain ID).
    pub fn txid(&self, chain: &ChainParams) -> Hash32 {
        crypto::tagged_hash("ug/txid", &[chain.chain_id.as_bytes(), &self.encode()])
    }

    /// Build the per-input signature digest.
    fn signature_digest(
        &self,
        chain: &ChainParams,
        input_index: usize,
        owner: &[u8; 32],
    ) -> Hash32 {
        let mut e = Encoder::with_capacity(64 + self.inputs.len() * 36 + self.outputs.len() * 40);
        e.u8(self.version);
        e.u32(input_index as u32);
        e.fixed(owner);
        e.u32(self.inputs.len() as u32);
        for input in &self.inputs {
            input.previous_output.encode(&mut e);
        }
        e.u32(self.outputs.len() as u32);
        for output in &self.outputs {
            e.u64(output.value);
            e.fixed(&output.recipient);
        }
        e.u64(self.fee);
        let body = e.finish();
        crypto::tagged_hash("ug/tx-signature", &[chain.chain_id.as_bytes(), &body])
    }

    /// Sign one input, binding the signature to the chain.
    pub fn sign_input_with_chain(
        &mut self,
        index: usize,
        key: &SigningKey,
        chain: &ChainParams,
    ) -> Result<()> {
        let owner = crypto::public_key(key);
        let digest = self.signature_digest(chain, index, &owner);
        let input = self.inputs.get_mut(index).ok_or_else(|| {
            Error::InvalidTransaction(format!("input index {index} out of range"))
        })?;
        use ed25519_dalek::Signer;
        input.signature = key.sign(&digest).to_bytes();
        Ok(())
    }

    /// Verify one input signature against its owner key and chain.
    pub fn verify_input_signature(
        &self,
        index: usize,
        owner: &[u8; 32],
        chain: &ChainParams,
    ) -> Result<()> {
        let input = self
            .inputs
            .get(index)
            .ok_or(Error::InvalidSignature(index))?;
        let digest = self.signature_digest(chain, index, owner);
        use ed25519_dalek::{Signature, VerifyingKey};
        let vk = VerifyingKey::from_bytes(owner).map_err(|_| Error::InvalidSignature(index))?;
        let sig = Signature::from_bytes(&input.signature);
        vk.verify_strict(&digest, &sig)
            .map_err(|_| Error::InvalidSignature(index))
    }

    /// Verify all inputs and the value-conservation rule.
    ///
    /// `input_meta` contains `(owner_pubkey, value)` for each input, in order.
    /// The caller is responsible for having resolved each outpoint against the
    /// canonical UTXO set (or the mempool overlay) first.
    pub fn verify_inputs(
        &self,
        chain: &ChainParams,
        input_meta: &[([u8; 32], u64)],
    ) -> Result<u64> {
        if input_meta.len() != self.inputs.len() {
            return Err(Error::InvalidTransaction(
                "input metadata length mismatch".into(),
            ));
        }
        let mut total_in = 0u64;
        for (i, (owner, value)) in input_meta.iter().enumerate() {
            total_in = total_in
                .checked_add(*value)
                .ok_or_else(|| Error::InvalidTransaction("input value overflow".into()))?;
            self.verify_input_signature(i, owner, chain)?;
        }
        let mut total_out = 0u64;
        for output in &self.outputs {
            total_out = total_out
                .checked_add(output.value)
                .ok_or_else(|| Error::InvalidTransaction("output value overflow".into()))?;
        }
        let committed = total_out
            .checked_add(self.fee)
            .ok_or_else(|| Error::InvalidTransaction("fee overflow".into()))?;
        if committed != total_in {
            return Err(Error::InvalidTransaction(format!(
                "inputs {total_in} != outputs {total_out} + fee {}",
                self.fee
            )));
        }
        Ok(total_in)
    }

    pub fn fee_rate(&self) -> u64 {
        let size = self.encoded_size().max(1) as u64;
        self.fee / size
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn decode_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
            let chain = ChainParams::devnet();
            let _ = Transaction::decode(&bytes, &chain);
            let _ = Transaction::decode_genesis(&bytes, &chain);
        }

        #[test]
        fn encode_decode_roundtrip(
            value in 1u64..1_000_000u64,
            fee in 0u64..1_000u64,
        ) {
            // Build a syntactically minimal transaction and check canonical roundtrip.
            let chain = ChainParams::devnet();
            let tx = Transaction::new(vec![], vec![TxOut::new(value, [7u8; 32])], fee);
            let bytes = tx.encode();
            let decoded = Transaction::decode(&bytes, &chain);
            // A coinbase-less transaction with no inputs is structurally a
            // genesis-shaped tx; strict decode may reject it, but encoding must
            // be stable and not panic.
            let _ = decoded;
            prop_assert_eq!(Transaction::decode_structural(&bytes, &chain).unwrap().encode(), bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Utxo;

    fn params() -> ChainParams {
        ChainParams::devnet()
    }

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn funding(seed: u8, value: u64) -> (OutPoint, Utxo) {
        let owner = crypto::public_key(&key(seed));
        let outpoint = OutPoint::new([seed; 32], 0);
        (
            outpoint,
            Utxo {
                value,
                owner,
                height: 0,
                is_genesis: true,
            },
        )
    }

    fn signed_tx(
        chain: &ChainParams,
        seed: u8,
        input_value: u64,
        outputs: Vec<TxOut>,
        fee: u64,
    ) -> (Transaction, Vec<([u8; 32], u64)>) {
        let (outpoint, utxo) = funding(seed, input_value);
        let mut tx = Transaction::new(vec![TxIn::new(outpoint)], outputs, fee);
        tx.sign_input_with_chain(0, &key(seed), chain).unwrap();
        (tx, vec![(utxo.owner, utxo.value)])
    }

    #[test]
    fn canonical_roundtrip_and_txid_stability() {
        let chain = params();
        let (tx, _) = signed_tx(&chain, 1, 10_000, vec![TxOut::new(9_000, [2u8; 32])], 1_000);
        let bytes = tx.encode();
        let decoded = Transaction::decode(&bytes, &chain).unwrap();
        assert_eq!(tx, decoded);
        assert_eq!(tx.txid(&chain), decoded.txid(&chain));
        assert_eq!(bytes, decoded.encode());
    }

    #[test]
    fn decode_rejects_truncation_trailing_and_oversized_counts() {
        let chain = params();
        let (tx, _) = signed_tx(&chain, 1, 10_000, vec![TxOut::new(9_000, [2u8; 32])], 1_000);
        let bytes = tx.encode();

        for cut in [0, 1, 5, bytes.len() - 1] {
            assert!(Transaction::decode(&bytes[..cut], &chain).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Transaction::decode(&trailing, &chain).is_err());

        let mut e = Encoder::new();
        e.u8(TX_VERSION);
        e.u32(u32::MAX);
        let evil = e.finish();
        assert!(Transaction::decode(&evil, &chain).is_err());

        // A count larger than the remaining bytes must fail without allocating
        // a container of that size.
        let mut evil = Vec::new();
        evil.push(TX_VERSION);
        evil.extend_from_slice(&(MAX_TX_INPUTS as u32 + 1).to_le_bytes());
        assert!(Transaction::decode(&evil, &chain).is_err());
    }

    #[test]
    fn signature_rejects_tampering_and_wrong_key() {
        let chain = params();
        let (mut tx, meta) =
            signed_tx(&chain, 1, 10_000, vec![TxOut::new(9_000, [2u8; 32])], 1_000);
        tx.verify_inputs(&chain, &meta).unwrap();

        tx.outputs[0].value = 8_999;
        assert!(tx.verify_inputs(&chain, &meta).is_err());

        // Restore the signed tx, then verify with the wrong owner key.
        let (tx, _) = signed_tx(&chain, 1, 10_000, vec![TxOut::new(9_000, [2u8; 32])], 1_000);
        let wrong = [(crypto::public_key(&key(9)), 10_000u64)];
        assert!(tx.verify_inputs(&chain, &wrong).is_err());
    }

    #[test]
    fn signatures_are_not_replayable_on_another_chain() {
        let chain = params();
        let (tx, meta) = signed_tx(&chain, 1, 10_000, vec![TxOut::new(9_000, [2u8; 32])], 1_000);
        let mut other = chain.clone();
        other.chain_id = "udaya-global-devnet-2".into();
        assert!(tx.verify_inputs(&other, &meta).is_err());
        assert_ne!(tx.txid(&chain), tx.txid(&other));
    }

    #[test]
    fn value_conservation_is_exact_and_overflow_safe() {
        let chain = params();
        let (tx, meta) = signed_tx(&chain, 1, 10_000, vec![TxOut::new(9_000, [2u8; 32])], 999);
        // 9,000 + 999 != 10,000
        assert!(tx.verify_inputs(&chain, &meta).is_err());

        let (tx, meta) = signed_tx(
            &chain,
            1,
            u64::MAX,
            vec![TxOut::new(u64::MAX, [2u8; 32])],
            1,
        );
        assert!(tx.verify_inputs(&chain, &meta).is_err());
    }

    #[test]
    fn duplicate_inputs_are_rejected() {
        let chain = params();
        let (outpoint, utxo) = funding(1, 10_000);
        let mut tx = Transaction::new(
            vec![TxIn::new(outpoint), TxIn::new(outpoint)],
            vec![TxOut::new(9_000, [2u8; 32])],
            1_000,
        );
        tx.sign_input_with_chain(0, &key(1), &chain).unwrap();
        tx.sign_input_with_chain(1, &key(1), &chain).unwrap();
        let err = tx.validate_structure(&chain).unwrap_err();
        assert!(err.to_string().contains("duplicate input"));
        let meta = [(utxo.owner, utxo.value), (utxo.owner, utxo.value)];
        assert!(tx.verify_inputs(&chain, &meta).is_err());
    }

    #[test]
    fn fee_below_minimum_is_rejected() {
        let chain = params();
        let (tx, _) = signed_tx(&chain, 1, 100_000, vec![TxOut::new(99_999, [2u8; 32])], 1);
        // 1 satoshi fee is below size * 1 sat/byte for this transaction.
        assert!(tx.validate_structure(&chain).is_err());
    }

    #[test]
    fn zero_value_output_is_rejected() {
        let chain = params();
        let (tx, _) = signed_tx(&chain, 1, 10_000, vec![TxOut::new(0, [2u8; 32])], 1_000);
        assert!(tx.validate_structure(&chain).is_err());
    }

    #[test]
    fn genesis_transaction_shape_is_distinct() {
        let tx = Transaction {
            version: GENESIS_TX_VERSION,
            inputs: vec![],
            outputs: vec![TxOut::new(1, [1u8; 32])],
            fee: 0,
        };
        assert!(tx.is_genesis());
        assert!(tx.validate_structure(&params()).is_err());
    }
}
