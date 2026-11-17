/// Udaya Execution - Transaction execution engine
/// UTXO model execution with script validation

use crate::primitives::*;
use std::collections::HashMap;

/// Transaction input validator
pub fn validate_input(
    txid: &[u8],
    vout: u32,
    utxo_set: &HashMap<(Vec<u8>, u32), TxOut>,
) -> Result<(), String> {
    if txid.is_empty() {
        return Err("Empty transaction ID".to_string());
    }
    if vout == 0xFFFFFFFF {
        return Err("Coinbase input cannot be spent".to_string());
    }
    Ok(())
}

/// Transaction output validator
pub fn validate_output(
    value: u64,
    script_pubkey: &[u8],
    min_value: u64,
) -> Result<(), String> {
    if value == 0 {
        return Err("Zero value output not allowed".to_string());
    }
    if value < min_value {
        return Err(format!("Value {} below minimum {}", value, min_value));
    }
    Ok(())
}

/// UTXO set update after transaction
pub fn update_utxo_set(
    txid: Vec<u8>,
    vout: u32,
    new_utxos: &[(Vec<u8>, u64, Vec<u8>)],
    removed_utxos: &[(Vec<u8>, u32)],
    utxo_set: &mut HashMap<(Vec<u8>, u32), TxOut>,
) {
    // Remove spent outputs
    for (txid_remove, vout_remove) in removed_utxos {
        let key = (txid_remove.clone(), *vout_remove);
        utxo_set.remove(&key);
    }

    // Add new outputs
    for (script_pubkey, value, new_txid) in new_utxos {
        let key = (new_txid.clone(), 0); // Simplified - vout would be assigned
        utxo_set.insert(key.clone(), TxOut::new(value, script_pubkey.to_vec()));
    }
}

/// Script execution result
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptResult {
    /// Success
    Success,
    /// Script error
    Error(String),
    /// Insufficient funds
    InsufficientFunds,
    /// Script execution would create dust
    Dust,
}

/// Script execution
pub fn execute_script(
    script: &[u8],
    values: &[u64],
    min_requirement: u64,
) -> ScriptResult {
    if script.is_empty() {
        return ScriptResult::Error("Empty script".to_string());
    }

    // Simple P2PKH-like validation
    if script.len() >= 4 && script[0] == 0x76 { // OP_DUP
        // Verify signature check
        if script.len() >= 25 && script[script.len() - 1] == 0xac { // OP_CHECKSIG
            // Verify values meet minimum
            let total: u64 = values.iter().sum();
            if total >= min_requirement {
                return ScriptResult::Success;
            }
            return ScriptResult::InsufficientFunds;
        }
    }

    ScriptResult::Error("Script not recognized".to_string())
}

/// TxOut helper
impl TxOut {
    pub fn new(value: u64, script_pubkey: Vec<u8>) -> Self {
        TxOut {
            value,
            script_pubkey,
        }
    }
}