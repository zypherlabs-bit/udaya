use crate::types::{OutPoint, TxIn, TxOut, Txid};
use crate::{LOCKTIME_THRESHOLD, TX_VERSION};
use serde::{Deserialize, Serialize};

/// A Udaya transaction
#[derive(Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub version: i32,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub lock_time: u32,
}

impl Transaction {
    pub fn new(version: i32, inputs: Vec<TxIn>, outputs: Vec<TxOut>, lock_time: u32) -> Self {
        Self {
            version,
            inputs,
            outputs,
            lock_time,
        }
    }

    /// Create a new coinbase transaction
    pub fn new_coinbase(coinbase_data: Vec<u8>, outputs: Vec<TxOut>, block_height: u64) -> Self {
        // Add block height to coinbase for BIP-34 compliance
        let mut height_bytes = vec![];
        height_bytes.extend_from_slice(&block_height.to_le_bytes());
        // Trim trailing zeros
        while height_bytes.last() == Some(&0) {
            height_bytes.pop();
        }
        let mut script_data = vec![height_bytes.len() as u8];
        script_data.extend_from_slice(&height_bytes);
        script_data.extend_from_slice(&coinbase_data);

        let coinbase_input = TxIn::new_coinbase(script_data);

        Self {
            version: TX_VERSION,
            inputs: vec![coinbase_input],
            outputs,
            lock_time: 0,
        }
    }

    /// Compute transaction ID
    pub fn txid(&self) -> Txid {
        let data = self.serialize_without_witness();
        Txid::compute(&data)
    }

    /// Compute witness transaction ID (includes witness data)
    pub fn wtxid(&self) -> Txid {
        let data = self.serialize();
        Txid::compute(&data)
    }

    /// Serialize transaction without witness data
    fn serialize_without_witness(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap_or_default()
    }

    /// Serialize full transaction
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap_or_default()
    }

    /// Deserialize transaction
    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        Ok(bincode::deserialize(data)?)
    }

    /// Check if this is a coinbase transaction
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].is_coinbase()
    }

    /// Check if transaction is final (locktime requirements met)
    pub fn is_final(&self, block_height: u64, block_time: u64) -> bool {
        if self.lock_time == 0 {
            return true;
        }
        if self.lock_time < LOCKTIME_THRESHOLD {
            self.lock_time as u64 <= block_height
        } else {
            self.lock_time as u64 <= block_time
        }
    }

    /// Get total output value (uses saturating arithmetic for fuzz resilience)
    pub fn total_output(&self) -> u64 {
        self.outputs
            .iter()
            .fold(0u64, |acc, o| acc.saturating_add(o.value))
    }

    /// Get transaction size in bytes
    pub fn size(&self) -> usize {
        self.serialize().len()
    }

    /// Get transaction weight
    pub fn weight(&self) -> u64 {
        // Base size * 3 + total size
        let base_size = self.serialize_without_witness().len() as u64;
        let total_size = self.size() as u64;
        base_size * 3 + total_size
    }

    /// Get virtual size (weight / 4)
    pub fn vsize(&self) -> u64 {
        self.weight().div_ceil(4)
    }

    /// Check if transaction has valid structure
    pub fn is_valid_structure(&self) -> bool {
        // Must have at least one input and output
        if self.inputs.is_empty() || self.outputs.is_empty() {
            return false;
        }

        // Coinbase must have exactly one input
        if self.is_coinbase() && self.inputs.len() != 1 {
            return false;
        }

        // Non-coinbase must have no null outpoints
        if !self.is_coinbase() {
            for input in &self.inputs {
                if input.is_coinbase() {
                    return false;
                }
            }
        }

        // No output can have value exceeding max money
        let total = self.total_output();
        if total > crate::MAX_SUPPLY * crate::SATS_PER_COIN {
            return false;
        }

        true
    }
}

impl std::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transaction")
            .field("txid", &self.txid())
            .field("version", &self.version)
            .field("inputs", &self.inputs.len())
            .field("outputs", &self.outputs.len())
            .field("lock_time", &self.lock_time)
            .finish()
    }
}

/// Fee estimation for transaction
pub fn estimate_tx_fee(tx: &Transaction, fee_rate: u64) -> u64 {
    tx.vsize() * fee_rate
}

/// Create a simple P2PKH transaction
pub fn create_p2pkh_transaction(
    utxos: Vec<(OutPoint, TxOut)>,
    to_address_script: Vec<u8>,
    change_address_script: Vec<u8>,
    amount: u64,
    fee: u64,
    private_key: &secp256k1::SecretKey,
    secp: &secp256k1::Secp256k1<secp256k1::All>,
) -> anyhow::Result<Transaction> {
    let total_input: u64 = utxos.iter().map(|(_, txout)| txout.value).sum();
    let change = total_input - amount - fee;

    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    for (outpoint, _) in &utxos {
        inputs.push(TxIn {
            previous_output: outpoint.clone(),
            script_sig: crate::types::ScriptSig::new(vec![]), // Will be signed
            sequence: 0xFFFFFFFF,
            witness: Vec::new(),
        });
    }

    outputs.push(TxOut {
        value: amount,
        script_pubkey: crate::types::ScriptPubKey::new(to_address_script),
    });

    if change > 0 {
        outputs.push(TxOut {
            value: change,
            script_pubkey: crate::types::ScriptPubKey::new(change_address_script),
        });
    }

    let mut tx = Transaction::new(TX_VERSION, inputs, outputs, 0);

    // Sign each input
    for i in 0..tx.inputs.len() {
        let sighash = tx.signature_hash(i, crate::types::ScriptSig::new(vec![]), 0x01); // SIGHASH_ALL
        let msg = secp256k1::Message::from_digest_slice(&sighash.0)?;
        let sig = secp.sign_ecdsa(&msg, private_key);
        let mut sig_bytes = sig.serialize_der().to_vec();
        sig_bytes.push(0x01); // SIGHASH_ALL
        tx.inputs[i].script_sig = crate::types::ScriptSig::new(sig_bytes);
    }

    Ok(tx)
}

/// Signature hash for transaction signing
impl Transaction {
    pub fn signature_hash(
        &self,
        input_index: usize,
        script_code: crate::types::ScriptSig,
        hash_type: u8,
    ) -> crate::types::BlockHash {
        // Simplified signature hash (legacy format)
        let mut data = Vec::new();

        // Version
        data.extend_from_slice(&self.version.to_le_bytes());

        // Inputs (simplified)
        for (i, input) in self.inputs.iter().enumerate() {
            data.extend_from_slice(&input.previous_output.txid.0);
            data.extend_from_slice(&input.previous_output.vout.to_le_bytes());
            if i == input_index {
                data.extend_from_slice(&script_code.data);
            } else {
                data.push(0x00); // empty script
            }
            data.extend_from_slice(&input.sequence.to_le_bytes());
        }

        // Outputs
        for output in &self.outputs {
            data.extend_from_slice(&output.value.to_le_bytes());
            data.extend_from_slice(&(output.script_pubkey.data.len() as u64).to_le_bytes());
            data.extend_from_slice(&output.script_pubkey.data);
        }

        // Locktime
        data.extend_from_slice(&self.lock_time.to_le_bytes());

        // Hash type
        data.push(hash_type);

        crate::types::BlockHash::double_sha256(&data)
    }
}
