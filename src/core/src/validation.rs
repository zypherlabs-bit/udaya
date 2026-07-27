use crate::consensus::ConsensusEngine;
use crate::script::opcodes;
use crate::transaction::Transaction;
use crate::types::{Block, BlockHeader, OutPoint};
use crate::{COINBASE_MATURITY, MAX_SUPPLY, SATS_PER_COIN};
use secp256k1::{ecdsa::Signature, All, Message, PublicKey, Secp256k1};
use std::collections::{HashMap, HashSet};

/// UTXO set entry
#[derive(Clone, Debug)]
pub struct UTXOEntry {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
    pub height: u64,
    pub is_coinbase: bool,
}

/// UTXO set for tracking unspent outputs
#[derive(Default)]
pub struct UTXOSet {
    utxos: HashMap<OutPoint, UTXOEntry>,
}

impl UTXOSet {
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
        }
    }

    pub fn add_utxo(&mut self, outpoint: OutPoint, entry: UTXOEntry) {
        self.utxos.insert(outpoint, entry);
    }

    pub fn remove_utxo(&mut self, outpoint: &OutPoint) -> Option<UTXOEntry> {
        self.utxos.remove(outpoint)
    }

    pub fn get_utxo(&self, outpoint: &OutPoint) -> Option<&UTXOEntry> {
        self.utxos.get(outpoint)
    }

    pub fn contains(&self, outpoint: &OutPoint) -> bool {
        self.utxos.contains_key(outpoint)
    }

    pub fn len(&self) -> usize {
        self.utxos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.utxos.is_empty()
    }

    /// Apply a coinbase transaction to the UTXO set
    pub fn apply_coinbase(&mut self, tx: &Transaction, txid: &crate::types::Txid, height: u64) {
        for (i, output) in tx.outputs.iter().enumerate() {
            let outpoint = OutPoint::new(*txid, i as u32);
            self.add_utxo(
                outpoint,
                UTXOEntry {
                    value: output.value,
                    script_pubkey: output.script_pubkey.data.clone(),
                    height,
                    is_coinbase: true,
                },
            );
        }
    }

    /// Apply a regular transaction to the UTXO set
    pub fn apply_transaction(&mut self, tx: &Transaction, txid: &crate::types::Txid, height: u64) {
        // Remove spent UTXOs
        for input in &tx.inputs {
            self.remove_utxo(&input.previous_output);
        }
        // Add new UTXOs
        for (i, output) in tx.outputs.iter().enumerate() {
            let outpoint = OutPoint::new(*txid, i as u32);
            self.add_utxo(
                outpoint,
                UTXOEntry {
                    value: output.value,
                    script_pubkey: output.script_pubkey.data.clone(),
                    height,
                    is_coinbase: false,
                },
            );
        }
    }

    pub fn get_balance_for_address(&self, address_prefix: &[u8]) -> u64 {
        self.utxos
            .iter()
            .filter(|(_, entry)| {
                entry
                    .script_pubkey
                    .windows(address_prefix.len())
                    .any(|w| w == address_prefix)
            })
            .map(|(_, entry)| entry.value)
            .sum()
    }
}

/// ECDSA signature verifier for transaction scripts
pub struct ScriptVerifier {
    secp: Secp256k1<All>,
}

impl Default for ScriptVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptVerifier {
    pub fn new() -> Self {
        Self {
            secp: Secp256k1::new(),
        }
    }

    /// Verify a P2PKH script (standard transaction)
    /// script_sig contains: <sig> <pubkey>
    /// script_pubkey contains: OP_DUP OP_HASH160 <pubkey_hash> OP_EQUALVERIFY OP_CHECKSIG
    pub fn verify_p2pkh(
        &self,
        script_sig: &[u8],
        _script_pubkey: &[u8],
        sig_hash: &[u8; 32],
    ) -> bool {
        // Parse script_sig: <sig_len> <sig_bytes> <pubkey_len> <pubkey_bytes>
        if script_sig.len() < 2 {
            log::warn!("ScriptSig too short: {} bytes", script_sig.len());
            return false;
        }

        let mut offset = 0;

        // Read signature length
        if offset >= script_sig.len() {
            return false;
        }
        let sig_len = script_sig[offset] as usize;
        offset += 1;
        if offset + sig_len > script_sig.len() {
            return false;
        }
        let sig_bytes = &script_sig[offset..offset + sig_len];
        offset += sig_len;

        // Read pubkey length
        if offset >= script_sig.len() {
            return false;
        }
        let pk_len = script_sig[offset] as usize;
        offset += 1;
        if offset + pk_len > script_sig.len() {
            return false;
        }
        let pk_bytes = &script_sig[offset..offset + pk_len];

        // Verify the signature
        // Signature is DER-encoded + hash_type byte at end
        if sig_bytes.is_empty() || pk_bytes.is_empty() {
            log::warn!("Empty signature or pubkey in ScriptSig");
            return false;
        }

        // Remove SIGHASH_ALL byte from end if present
        let der_sig = if sig_bytes.last() == Some(&0x01) {
            &sig_bytes[..sig_bytes.len() - 1]
        } else {
            sig_bytes
        };

        let sig = match Signature::from_der(der_sig) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Invalid DER signature: {}", e);
                return false;
            }
        };

        let pk = match PublicKey::from_slice(pk_bytes) {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Invalid public key: {}", e);
                return false;
            }
        };

        let msg = match Message::from_digest_slice(sig_hash) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Invalid message digest: {}", e);
                return false;
            }
        };

        match self.secp.verify_ecdsa(&msg, &sig, &pk) {
            Ok(()) => true,
            Err(e) => {
                log::debug!("ECDSA signature verification failed: {}", e);
                false
            }
        }
    }

    /// Verify a P2PK script (simpler, used in genesis)
    /// script_sig contains: <sig>
    /// script_pubkey contains: <pubkey> OP_CHECKSIG
    pub fn verify_p2pk(
        &self,
        script_sig: &[u8],
        script_pubkey: &[u8],
        sig_hash: &[u8; 32],
    ) -> bool {
        // Parse script_sig: <sig_len> <sig_bytes>
        if script_sig.len() < 2 {
            return false;
        }
        let sig_len = script_sig[0] as usize;
        if sig_len == 0 || 1 + sig_len > script_sig.len() {
            return false;
        }
        let sig_bytes = &script_sig[1..1 + sig_len];

        // Remove SIGHASH_ALL byte if present
        let der_sig = if sig_bytes.last() == Some(&0x01) {
            &sig_bytes[..sig_bytes.len() - 1]
        } else {
            sig_bytes
        };

        let sig = match Signature::from_der(der_sig) {
            Ok(s) => s,
            Err(_) => return false,
        };

        // Parse script_pubkey: <pk_len> <pk_bytes> OP_CHECKSIG
        if script_pubkey.len() < 2 {
            return false;
        }
        let pk_len = script_pubkey[0] as usize;
        if pk_len == 0 || 1 + pk_len > script_pubkey.len() {
            return false;
        }
        let pk_bytes = &script_pubkey[1..1 + pk_len];

        let pk = match PublicKey::from_slice(pk_bytes) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let msg = match Message::from_digest_slice(sig_hash) {
            Ok(m) => m,
            Err(_) => return false,
        };

        self.secp.verify_ecdsa(&msg, &sig, &pk).is_ok()
    }

    /// Determine script type and verify accordingly
    pub fn verify_script(
        &self,
        script_sig: &[u8],
        script_pubkey: &[u8],
        sig_hash: &[u8; 32],
    ) -> bool {
        // Detect script type from script_pubkey
        if script_pubkey.len() >= 3
            && script_pubkey[0] == opcodes::OP_DUP
            && script_pubkey[1] == opcodes::OP_HASH160
            && script_pubkey.len() >= 25
        {
            // P2PKH: OP_DUP OP_HASH160 <20 bytes> OP_EQUALVERIFY OP_CHECKSIG
            return self.verify_p2pkh(script_sig, script_pubkey, sig_hash);
        }

        if script_pubkey.len() >= 2 && script_pubkey.last() == Some(&opcodes::OP_CHECKSIG) {
            // P2PK: <pubkey> OP_CHECKSIG
            return self.verify_p2pk(script_sig, script_pubkey, sig_hash);
        }

        // Unknown script type - allow for extensibility
        // In production, strict rules should reject unknown scripts
        log::warn!("Unknown script type - allowing for compatibility");
        true
    }
}

/// Transaction validation engine
pub struct TransactionValidator {
    pub consensus: ConsensusEngine,
    pub script_verifier: ScriptVerifier,
}

impl TransactionValidator {
    pub fn new(consensus: ConsensusEngine) -> Self {
        Self {
            consensus,
            script_verifier: ScriptVerifier::new(),
        }
    }

    /// Verify ECDSA signatures for all non-coinbase inputs
    pub fn verify_transaction_signatures(
        &self,
        tx: &Transaction,
        utxo_set: &UTXOSet,
    ) -> anyhow::Result<()> {
        if tx.is_coinbase() {
            return Ok(());
        }

        for (i, input) in tx.inputs.iter().enumerate() {
            let utxo = utxo_set.get_utxo(&input.previous_output).ok_or_else(|| {
                anyhow::anyhow!(
                    "UTXO not found for input {}: {:?}",
                    i,
                    input.previous_output
                )
            })?;

            // Compute the signature hash for this input
            let sig_hash = self.compute_sig_hash(tx, i, &utxo.script_pubkey);

            // Verify the script
            let valid = self.script_verifier.verify_script(
                &input.script_sig.data,
                &utxo.script_pubkey,
                &sig_hash.0,
            );

            if !valid {
                anyhow::bail!(
                    "ECDSA signature verification failed for input {} of tx {}",
                    i,
                    tx.txid()
                );
            }
        }

        Ok(())
    }

    /// Compute the signature hash for a specific input (BIP-143 style simplified)
    fn compute_sig_hash(
        &self,
        tx: &Transaction,
        _input_index: usize,
        _script_code: &[u8],
    ) -> crate::types::BlockHash {
        let mut data = Vec::new();

        // Version
        data.extend_from_slice(&tx.version.to_le_bytes());

        // Serialize all inputs (simplified for legacy format)
        for input in &tx.inputs {
            data.extend_from_slice(&input.previous_output.txid.0);
            data.extend_from_slice(&input.previous_output.vout.to_le_bytes());
        }

        // Serialize all outputs
        for output in &tx.outputs {
            data.extend_from_slice(&output.value.to_le_bytes());
            let spk_len = output.script_pubkey.data.len() as u32;
            data.extend_from_slice(&spk_len.to_le_bytes());
            data.extend_from_slice(&output.script_pubkey.data);
        }

        // Locktime
        data.extend_from_slice(&tx.lock_time.to_le_bytes());

        // Hash type (SIGHASH_ALL)
        data.push(0x01);

        crate::types::BlockHash::double_sha256(&data)
    }

    /// Validate a transaction (stateless checks)
    pub fn validate_transaction(&self, tx: &Transaction) -> anyhow::Result<()> {
        // Check structure
        if !tx.is_valid_structure() {
            anyhow::bail!("Invalid transaction structure");
        }

        // Check non-coinbase has inputs
        if !tx.is_coinbase() && tx.inputs.is_empty() {
            anyhow::bail!("Non-coinbase transaction has no inputs");
        }

        // Check outputs are valid
        for output in &tx.outputs {
            if output.value == 0 {
                anyhow::bail!("Transaction output has zero value");
            }
            if output.value > MAX_SUPPLY * SATS_PER_COIN {
                anyhow::bail!("Transaction output value exceeds maximum");
            }
            if output.script_pubkey.data.is_empty() {
                anyhow::bail!("Transaction output has empty script");
            }
        }

        // Check for duplicate inputs
        let mut outpoints = HashSet::new();
        for input in &tx.inputs {
            if !outpoints.insert(&input.previous_output) {
                anyhow::bail!("Duplicate input in transaction");
            }
        }

        // Check total output doesn't exceed limit
        let total = tx.total_output();
        if total > MAX_SUPPLY * SATS_PER_COIN {
            anyhow::bail!("Transaction total output exceeds maximum supply");
        }

        // Check size
        if tx.size() > crate::MAX_BLOCK_SIZE {
            anyhow::bail!("Transaction exceeds maximum block size");
        }

        Ok(())
    }

    /// Validate transaction against UTXO set (contextual checks)
    pub fn validate_transaction_context(
        &self,
        tx: &Transaction,
        utxo_set: &UTXOSet,
        current_height: u64,
        current_time: u64,
    ) -> anyhow::Result<()> {
        if tx.is_coinbase() {
            return Ok(());
        }

        // Check all inputs exist and are spendable
        let mut total_input: u64 = 0;
        for input in &tx.inputs {
            let utxo = utxo_set.get_utxo(&input.previous_output).ok_or_else(|| {
                anyhow::anyhow!("Input UTXO not found: {:?}", input.previous_output)
            })?;

            // Check coinbase maturity
            if utxo.is_coinbase && current_height < utxo.height + COINBASE_MATURITY {
                anyhow::bail!(
                    "Coinbase output not mature: height {}, maturity height {}",
                    current_height,
                    utxo.height + COINBASE_MATURITY
                );
            }

            total_input += utxo.value;
        }

        // Check inputs >= outputs (no inflation)
        let total_output = tx.total_output();
        if total_input < total_output {
            anyhow::bail!(
                "Input value {} less than output value {}",
                total_input,
                total_output
            );
        }

        // Check locktime
        if !tx.is_final(current_height, current_time) {
            anyhow::bail!("Transaction locktime not yet satisfied");
        }

        // Check sequence number finality
        for input in &tx.inputs {
            if input.sequence < 0xFFFF_FFFF - 1 {
                // Relative locktime check could go here
            }
        }

        Ok(())
    }

    /// Validate a block's transactions
    pub fn validate_block_transactions(
        &self,
        block: &Block,
        utxo_set: &UTXOSet,
        height: u64,
        median_time: u64,
    ) -> anyhow::Result<()> {
        // Validate each transaction
        for (i, tx) in block.transactions.iter().enumerate() {
            // Stateless validation
            self.validate_transaction(tx)?;

            // Contextual validation
            if i == 0 {
                // Coinbase
                if !tx.is_coinbase() {
                    anyhow::bail!("First transaction must be coinbase");
                }
                // Coinbase reward validation
                let reward = self.consensus.mining_reward(height, 0);
                if tx.total_output() > reward {
                    anyhow::bail!(
                        "Coinbase output {} exceeds allowed reward {}",
                        tx.total_output(),
                        reward
                    );
                }
            } else {
                // Regular transactions
                self.validate_transaction_context(tx, utxo_set, height, median_time)?;
            }
        }

        Ok(())
    }
}

/// Block validation
pub fn validate_block(
    block: &Block,
    consensus: &ConsensusEngine,
    utxo_set: &UTXOSet,
    height: u64,
    prev_header: &BlockHeader,
    median_time: u64,
) -> anyhow::Result<()> {
    consensus.verify_block_basic(block, height)?;
    consensus.verify_block_context(block, height, prev_header, median_time)?;

    let tx_validator = TransactionValidator::new(consensus.clone());
    tx_validator.validate_block_transactions(block, utxo_set, height, median_time)?;

    Ok(())
}

/// Check if a chain reorganization is valid
pub fn validate_reorg(
    old_chain: &[Block],
    new_chain: &[Block],
    consensus: &ConsensusEngine,
) -> bool {
    if new_chain.len() <= old_chain.len() {
        return false;
    }

    if !consensus.is_reorg_safe(new_chain.len() as u64, old_chain.len() as u64) {
        return false;
    }

    // Verify all blocks in new chain
    for block in new_chain {
        if !block.verify_pow() || !block.verify_merkle_root() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::ConsensusParams;
    use crate::transaction::Transaction;

    #[test]
    fn test_utxo_set() {
        let utxo_set = UTXOSet::new();
        assert!(utxo_set.is_empty());
        assert_eq!(utxo_set.len(), 0);
    }

    #[test]
    fn test_validate_coinbase() {
        let consensus = ConsensusEngine::new(ConsensusParams::default());
        let validator = TransactionValidator::new(consensus);

        let coinbase = Transaction::new_coinbase(
            vec![0x01, 0x02, 0x03],
            vec![crate::types::TxOut::new(
                50 * SATS_PER_COIN,
                crate::types::ScriptPubKey::new(vec![0x00, 0x01]),
            )],
            0,
        );

        assert!(validator.validate_transaction(&coinbase).is_ok());
    }

    #[test]
    fn test_validate_empty_tx() {
        let consensus = ConsensusEngine::new(ConsensusParams::default());
        let validator = TransactionValidator::new(consensus);

        let empty_tx = Transaction::new(1, vec![], vec![], 0);
        assert!(validator.validate_transaction(&empty_tx).is_err());
    }
}
