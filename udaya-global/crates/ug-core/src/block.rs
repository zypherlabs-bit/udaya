//! Canonical blocks, header hashing and deterministic block execution.
//!
//! Udaya Global has no proof-of-work and no block subsidy. Blocks are ordered
//! by the U-ADC BFT consensus engine, and every block is fully executed and
//! state-root-checked before it can be accepted. A node never trusts a peer's
//! claimed height or state.

use crate::codec::{Decoder, Encoder};
use crate::crypto::{self, Hash32};
use crate::error::{Error, Result};
use crate::state::{AppliedTx, UtxoSet};
use crate::tx::Transaction;
use crate::{
    ChainParams, Height, Round, MAX_BLOCK_BYTES, MAX_BLOCK_TXS, MAX_FUTURE_DRIFT_SECS, MAX_TX_BYTES,
};

/// Current block version.
pub const BLOCK_VERSION: u8 = 1;

/// Block header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    pub version: u8,
    pub height: Height,
    pub previous_hash: Hash32,
    pub timestamp: u64,
    pub tx_root: Hash32,
    pub state_root: Hash32,
    pub proposer: [u8; 32],
    pub round: Round,
}

impl BlockHeader {
    pub fn encode(&self) -> Vec<u8> {
        let mut e = Encoder::with_capacity(1 + 8 + 32 + 8 + 32 + 32 + 32 + 4);
        e.u8(self.version);
        e.u64(self.height);
        e.fixed(&self.previous_hash);
        e.u64(self.timestamp);
        e.fixed(&self.tx_root);
        e.fixed(&self.state_root);
        e.fixed(&self.proposer);
        e.u32(self.round);
        e.finish()
    }

    pub fn decode(d: &mut Decoder<'_>) -> Result<Self> {
        Ok(Self {
            version: d.u8()?,
            height: d.u64()?,
            previous_hash: d.fixed::<32>()?,
            timestamp: d.u64()?,
            tx_root: d.fixed::<32>()?,
            state_root: d.fixed::<32>()?,
            proposer: d.fixed::<32>()?,
            round: d.u32()?,
        })
    }

    /// Chain-bound block hash.
    pub fn hash(&self, chain: &ChainParams) -> Hash32 {
        crypto::tagged_hash(
            "ug/block-header",
            &[chain.chain_id.as_bytes(), &self.encode()],
        )
    }
}

/// A complete block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

/// Result of executing a block.
#[derive(Clone, Debug)]
pub struct BlockReceipt {
    pub height: Height,
    pub block_hash: Hash32,
    pub state_root: Hash32,
    pub tx_count: usize,
    pub fees_burned: u64,
    pub applied: Vec<AppliedTx>,
}

impl Block {
    pub fn hash(&self, chain: &ChainParams) -> Hash32 {
        self.header.hash(chain)
    }

    pub fn txids(&self, chain: &ChainParams) -> Vec<Hash32> {
        self.transactions.iter().map(|tx| tx.txid(chain)).collect()
    }

    pub fn compute_tx_root(&self, chain: &ChainParams) -> Hash32 {
        crypto::merkle_root(&self.txids(chain))
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut e = Encoder::with_capacity(
            1 + 8 + 32 + 8 + 32 + 32 + 32 + 4 + 4 + self.transactions.len() * 128,
        );
        e.fixed(&self.header.encode());
        e.u32(self.transactions.len() as u32);
        for tx in &self.transactions {
            e.bytes(&tx.encode());
        }
        e.finish()
    }

    /// Strict decode with chain-specific limits.
    pub fn decode(data: &[u8], chain: &ChainParams) -> Result<Self> {
        if data.len() > chain.max_block_bytes {
            return Err(Error::InvalidBlock(format!(
                "block size {} exceeds limit {}",
                data.len(),
                chain.max_block_bytes
            )));
        }
        let mut d = Decoder::new(data);
        let header = BlockHeader::decode(&mut d)?;
        let count = d.len(MAX_BLOCK_TXS)?;
        let mut transactions = Vec::with_capacity(count);
        for _ in 0..count {
            let tx_bytes = d.bytes(chain.max_tx_bytes.min(MAX_TX_BYTES))?;
            let tx = if header.height == 0 {
                Transaction::decode_genesis(tx_bytes, chain)?
            } else {
                Transaction::decode(tx_bytes, chain)?
            };
            transactions.push(tx);
        }
        d.finish()?;
        let block = Self {
            header,
            transactions,
        };
        block.validate_structure(chain)?;
        Ok(block)
    }

    /// Stateless validation. This does not check the parent, state root or
    /// signatures; those require contextual execution.
    pub fn validate_structure(&self, chain: &ChainParams) -> Result<()> {
        if self.header.version != BLOCK_VERSION {
            return Err(Error::InvalidBlock(format!(
                "unsupported block version {}",
                self.header.version
            )));
        }
        if self.header.height > i64::MAX as u64 {
            return Err(Error::InvalidBlock("height out of range".into()));
        }
        if self.transactions.len() > MAX_BLOCK_TXS {
            return Err(Error::InvalidBlock("too many transactions".into()));
        }
        let size = self.encode().len();
        if size > chain.max_block_bytes || size > MAX_BLOCK_BYTES {
            return Err(Error::InvalidBlock(format!(
                "encoded block size {size} exceeds limit"
            )));
        }
        if self.header.timestamp == 0 {
            return Err(Error::InvalidBlock("zero timestamp".into()));
        }
        if self.header.round > 10_000_000 {
            return Err(Error::InvalidBlock("round out of range".into()));
        }
        if self.header.tx_root != self.compute_tx_root(chain) {
            return Err(Error::InvalidBlock("transaction root mismatch".into()));
        }

        let mut seen = std::collections::BTreeSet::new();
        for txid in self.txids(chain) {
            if !seen.insert(txid) {
                return Err(Error::InvalidBlock("duplicate transaction".into()));
            }
        }

        if self.header.height == 0 {
            if self.header.previous_hash != [0u8; 32] {
                return Err(Error::InvalidBlock("genesis has a parent".into()));
            }
            if self.header.proposer != [0u8; 32] {
                return Err(Error::InvalidBlock("genesis has a proposer".into()));
            }
            if self.transactions.len() != 1 || !self.transactions[0].is_genesis() {
                return Err(Error::InvalidBlock(
                    "genesis must contain exactly one genesis transaction".into(),
                ));
            }
        } else {
            if self.header.proposer == [0u8; 32] {
                return Err(Error::InvalidBlock("missing proposer".into()));
            }
            if self.transactions.iter().any(|tx| tx.is_genesis()) {
                return Err(Error::InvalidBlock(
                    "genesis transaction outside the genesis block".into(),
                ));
            }
        }
        Ok(())
    }

    /// Execute the genesis block and return the initial state.
    pub fn execute_genesis(&self, chain: &ChainParams) -> Result<(UtxoSet, BlockReceipt)> {
        self.validate_structure(chain)?;
        if self.header.height != 0 {
            return Err(Error::InvalidBlock("not a genesis block".into()));
        }
        let tx = &self.transactions[0];
        let mut state = UtxoSet::new();
        let txid = state.apply_genesis(chain, tx)?;
        let state_root = state.state_root(chain);
        if state_root != self.header.state_root {
            return Err(Error::InvalidBlock("genesis state root mismatch".into()));
        }
        let receipt = BlockReceipt {
            height: 0,
            block_hash: self.hash(chain),
            state_root,
            tx_count: 1,
            fees_burned: 0,
            applied: vec![AppliedTx {
                txid,
                fee: 0,
                input_value: 0,
                output_value: state.total_value(),
                created: state.iter().map(|(op, _)| *op).collect(),
            }],
        };
        Ok((state, receipt))
    }

    /// Fully execute this block on top of `parent_state` and verify every
    /// consensus-critical commitment. Returns the new state only on success.
    pub fn execute(
        &self,
        chain: &ChainParams,
        parent: &BlockHeader,
        parent_state: &UtxoSet,
        now_unix: u64,
    ) -> Result<(UtxoSet, BlockReceipt)> {
        self.validate_structure(chain)?;
        if self.header.height != parent.height + 1 {
            return Err(Error::InvalidBlock(format!(
                "height {} does not follow parent height {}",
                self.header.height, parent.height
            )));
        }
        let parent_hash = parent.hash(chain);
        if self.header.previous_hash != parent_hash {
            return Err(Error::InvalidBlock("previous hash mismatch".into()));
        }
        if self.header.timestamp < parent.timestamp {
            return Err(Error::InvalidBlock("timestamp moved backwards".into()));
        }
        if self.header.timestamp > now_unix.saturating_add(MAX_FUTURE_DRIFT_SECS) {
            return Err(Error::InvalidBlock(
                "timestamp too far in the future".into(),
            ));
        }

        let mut state = parent_state.clone();
        let mut applied = Vec::with_capacity(self.transactions.len());
        let mut fees = 0u64;
        for tx in &self.transactions {
            let receipt = state.apply_transaction(chain, tx, self.header.height)?;
            fees = fees
                .checked_add(receipt.fee)
                .ok_or_else(|| Error::InvalidBlock("fee overflow".into()))?;
            applied.push(receipt);
        }

        let state_root = state.state_root(chain);
        if state_root != self.header.state_root {
            return Err(Error::InvalidBlock(format!(
                "state root mismatch: computed {}, header {}",
                hex::encode(state_root),
                hex::encode(self.header.state_root)
            )));
        }
        state.audit_supply()?;

        let receipt = BlockReceipt {
            height: self.header.height,
            block_hash: self.hash(chain),
            state_root,
            tx_count: self.transactions.len(),
            fees_burned: fees,
            applied,
        };
        Ok((state, receipt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::public_key;
    use crate::tx::{TxIn, TxOut, GENESIS_TX_VERSION};
    use ed25519_dalek::SigningKey;

    fn chain() -> ChainParams {
        ChainParams::devnet()
    }

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn genesis_tx() -> Transaction {
        Transaction {
            version: GENESIS_TX_VERSION,
            inputs: vec![],
            outputs: vec![
                TxOut::new(1_000_000, public_key(&key(1))),
                TxOut::new(1_000_000, public_key(&key(2))),
            ],
            fee: 0,
        }
    }

    fn genesis_block(chain: &ChainParams) -> Block {
        let tx = genesis_tx();
        let txid = tx.txid(chain);
        let mut state = UtxoSet::new();
        state.apply_genesis(chain, &tx).unwrap();
        let header = BlockHeader {
            version: BLOCK_VERSION,
            height: 0,
            previous_hash: [0u8; 32],
            timestamp: 1_700_000_000,
            tx_root: crypto::merkle_root(&[txid]),
            state_root: state.state_root(chain),
            proposer: [0u8; 32],
            round: 0,
        };
        Block {
            header,
            transactions: vec![tx],
        }
    }

    fn next_block(
        chain: &ChainParams,
        parent: &Block,
        parent_state: &UtxoSet,
        txs: Vec<Transaction>,
        proposer_seed: u8,
    ) -> Block {
        let mut state = parent_state.clone();
        for tx in &txs {
            state
                .apply_transaction(chain, tx, parent.header.height + 1)
                .unwrap();
        }
        let txids: Vec<Hash32> = txs.iter().map(|tx| tx.txid(chain)).collect();
        let header = BlockHeader {
            version: BLOCK_VERSION,
            height: parent.header.height + 1,
            previous_hash: parent.hash(chain),
            timestamp: parent.header.timestamp + 1,
            tx_root: crypto::merkle_root(&txids),
            state_root: state.state_root(chain),
            proposer: public_key(&key(proposer_seed)),
            round: 0,
        };
        Block {
            header,
            transactions: txs,
        }
    }

    fn spend(
        chain: &ChainParams,
        seed: u8,
        txid: Hash32,
        outputs: Vec<TxOut>,
        fee: u64,
    ) -> Transaction {
        let mut tx = Transaction::new(
            vec![TxIn::new(crate::tx::OutPoint::new(txid, 0))],
            outputs,
            fee,
        );
        tx.sign_input_with_chain(0, &key(seed), chain).unwrap();
        tx
    }

    #[test]
    fn genesis_roundtrip_hash_and_execution() {
        let chain = chain();
        let block = genesis_block(&chain);
        let bytes = block.encode();
        let decoded = Block::decode(&bytes, &chain).unwrap();
        assert_eq!(block, decoded);
        assert_eq!(block.hash(&chain), decoded.hash(&chain));

        let (state, receipt) = block.execute_genesis(&chain).unwrap();
        assert_eq!(receipt.height, 0);
        assert_eq!(state.total_value(), 2_000_000);
        assert_eq!(state.state_root(&chain), block.header.state_root);
    }

    #[test]
    fn genesis_state_root_tamper_is_rejected() {
        let chain = chain();
        let mut block = genesis_block(&chain);
        block.header.state_root[0] ^= 1;
        assert!(block.execute_genesis(&chain).is_err());
    }

    #[test]
    fn valid_child_block_executes_and_updates_state() {
        let chain = chain();
        let genesis = genesis_block(&chain);
        let (state, _) = genesis.execute_genesis(&chain).unwrap();
        let genesis_txid = genesis.transactions[0].txid(&chain);
        let tx = spend(
            &chain,
            1,
            genesis_txid,
            vec![TxOut::new(900_000, public_key(&key(3)))],
            100_000,
        );
        let block = next_block(&chain, &genesis, &state, vec![tx], 4);
        let (new_state, receipt) = block
            .execute(&chain, &genesis.header, &state, 2_000_000_000)
            .unwrap();
        assert_eq!(receipt.height, 1);
        assert_eq!(receipt.tx_count, 1);
        assert_eq!(receipt.fees_burned, 100_000);
        assert_eq!(new_state.total_value(), 1_900_000);
    }

    #[test]
    fn double_spend_in_one_block_is_rejected() {
        let chain = chain();
        let genesis = genesis_block(&chain);
        let (state, _) = genesis.execute_genesis(&chain).unwrap();
        let genesis_txid = genesis.transactions[0].txid(&chain);
        let tx_a = spend(
            &chain,
            1,
            genesis_txid,
            vec![TxOut::new(900_000, public_key(&key(3)))],
            100_000,
        );
        let tx_b = spend(
            &chain,
            1,
            genesis_txid,
            vec![TxOut::new(900_000, public_key(&key(5)))],
            100_000,
        );
        // Build the header by executing only tx_a, then include both. Header
        // construction must fail at execution time rather than producing a
        // block that two nodes would interpret differently.
        let mut draft = next_block(&chain, &genesis, &state, vec![tx_a.clone()], 4);
        draft.transactions = vec![tx_a, tx_b];
        draft.header.tx_root = draft.compute_tx_root(&chain);
        let err = draft
            .execute(&chain, &genesis.header, &state, 2_000_000_000)
            .unwrap_err();
        assert!(err.to_string().contains("missing input") || err.to_string().contains("Input"));
    }

    #[test]
    fn wrong_state_root_is_rejected() {
        let chain = chain();
        let genesis = genesis_block(&chain);
        let (state, _) = genesis.execute_genesis(&chain).unwrap();
        let genesis_txid = genesis.transactions[0].txid(&chain);
        let tx = spend(
            &chain,
            1,
            genesis_txid,
            vec![TxOut::new(900_000, public_key(&key(3)))],
            100_000,
        );
        let mut block = next_block(&chain, &genesis, &state, vec![tx], 4);
        block.header.state_root[0] ^= 1;
        assert!(block
            .execute(&chain, &genesis.header, &state, 2_000_000_000)
            .is_err());
    }

    #[test]
    fn wrong_parent_and_future_timestamp_are_rejected() {
        let chain = chain();
        let genesis = genesis_block(&chain);
        let (state, _) = genesis.execute_genesis(&chain).unwrap();
        let block = next_block(&chain, &genesis, &state, vec![], 4);
        assert!(block
            .execute(
                &chain,
                &genesis.header,
                &state,
                block.header.timestamp + 1000
            )
            .is_ok());
        // "Now" earlier than the block by more than the allowed drift.
        assert!(block
            .execute(
                &chain,
                &genesis.header,
                &state,
                block
                    .header
                    .timestamp
                    .saturating_sub(MAX_FUTURE_DRIFT_SECS + 1),
            )
            .is_err());

        let mut other = genesis.header;
        other.timestamp += 1;
        assert!(block
            .execute(&chain, &other, &state, block.header.timestamp + 1000)
            .is_err());
    }

    #[test]
    fn decode_rejects_truncation_trailing_and_huge_tx_count() {
        let chain = chain();
        let block = genesis_block(&chain);
        let bytes = block.encode();
        assert!(Block::decode(&bytes[..bytes.len() - 1], &chain).is_err());
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Block::decode(&trailing, &chain).is_err());

        let mut e = Encoder::new();
        e.fixed(&block.header.encode());
        e.u32(u32::MAX);
        assert!(Block::decode(&e.finish(), &chain).is_err());
    }

    #[test]
    fn genesis_transaction_is_rejected_in_non_genesis_block() {
        let chain = chain();
        let genesis = genesis_block(&chain);
        let (state, _) = genesis.execute_genesis(&chain).unwrap();
        let mut block = next_block(&chain, &genesis, &state, vec![], 4);
        block.transactions = vec![genesis_tx()];
        block.header.tx_root = block.compute_tx_root(&chain);
        assert!(block.validate_structure(&chain).is_err());
    }
}
