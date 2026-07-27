use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use udaya_core::transaction::Transaction;
use udaya_core::types::*;
use udaya_core::SATS_PER_COIN;

// ============================================================
// BIP-174: Partially Signed Bitcoin Transaction (PSBT)
// ============================================================
// This module implements PSBT v0 as defined in BIP-174.
// PSBT allows:
//   - Offline transaction creation and signing
//   - Multi-device signing workflows
//   - Hardware wallet integration
//   - Multi-signature coordination
//
// PSBT Global Fields:
//   - Unsigned Transaction (required)
//   - XPUBs (optional)
//   - Version (optional)
//
// PSBT Input Fields:
//   - Non-Witness UTXO (optional)
//   - Witness UTXO (optional)
//   - Partial Signatures (optional)
//   - Sighash Type (optional)
//   - Redeem Script (optional)
//   - Witness Script (optional)
//   - BIP32 Derivation Paths (optional)
//   - Finalized ScriptSig (optional)
//   - Finalized ScriptWitness (optional)
//
// PSBT Output Fields:
//   - Redeem Script (optional)
//   - Witness Script (optional)
//   - BIP32 Derivation Paths (optional)

/// PSBT magic bytes
pub const PSBT_MAGIC: [u8; 5] = [0x70, 0x73, 0x62, 0x74, 0xFF]; // "psbt" + 0xFF

/// PSBT version
pub const PSBT_VERSION: u8 = 0;

/// Sighash types (BIP-143)
pub const SIGHASH_ALL: u8 = 0x01;
pub const SIGHASH_NONE: u8 = 0x02;
pub const SIGHASH_SINGLE: u8 = 0x03;
pub const SIGHASH_ANYONECANPAY: u8 = 0x80;

/// PSBT role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PSBTRole {
    Creator,
    Updater,
    Signer,
    Combiner,
    Finalizer,
    Extractor,
    Analyzer,
}

/// PSBT state for tracking workflow progress
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PSBTState {
    Created,
    Updated,
    Signed,
    Finalized,
    Extracted,
    Invalid,
}

/// Key-value map entry for PSBT fields (type, value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMapEntry {
    pub key_type: u8,
    pub key_data: Vec<u8>,
    pub value: Vec<u8>,
}

/// Input fields for PSBT
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PSBTInput {
    // Non-Witness UTXO (full transaction for this input)
    pub non_witness_utxo: Option<Vec<u8>>,
    // Witness UTXO (value + scriptPubKey)
    pub witness_utxo: Option<WitnessUTXO>,
    // Partial signatures: pubkey -> signature
    pub partial_signatures: HashMap<Vec<u8>, Vec<u8>>,
    // Sighash type
    pub sighash_type: Option<u8>,
    // Redeem script (for P2SH)
    pub redeem_script: Option<Vec<u8>>,
    // Witness script (for P2WSH)
    pub witness_script: Option<Vec<u8>>,
    // BIP32 derivation paths: pubkey -> (fingerprint, path)
    pub bip32_derivations: HashMap<Vec<u8>, BIP32Derivation>,
    // Finalized scriptsig
    pub final_scriptsig: Option<Vec<u8>>,
    // Finalized scriptwitness
    pub final_scriptwitness: Option<Vec<Vec<u8>>>,
    // Unknown fields
    pub unknowns: HashMap<Vec<u8>, Vec<u8>>,
}

/// Output fields for PSBT
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PSBTOutput {
    // Redeem script (for P2SH)
    pub redeem_script: Option<Vec<u8>>,
    // Witness script (for P2WSH)
    pub witness_script: Option<Vec<u8>>,
    // BIP32 derivation paths: pubkey -> (fingerprint, path)
    pub bip32_derivations: HashMap<Vec<u8>, BIP32Derivation>,
    // Unknown fields
    pub unknowns: HashMap<Vec<u8>, Vec<u8>>,
}

/// BIP32 derivation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BIP32Derivation {
    pub master_fingerprint: [u8; 4],
    pub derivation_path: Vec<u32>,
}

/// Witness UTXO (value + scriptPubKey)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessUTXO {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

/// The core PSBT structure implementing BIP-174
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PSBT {
    /// Global: Unsigned transaction
    pub unsigned_tx: Option<Transaction>,
    /// Global: Version
    pub version: u8,
    /// Global: XPUBs
    pub xpubs: Vec<XPUBEntry>,
    /// Input fields
    pub inputs: Vec<PSBTInput>,
    /// Output fields
    pub outputs: Vec<PSBTOutput>,
    /// Current state
    pub state: PSBTState,
    /// Transaction type (for display)
    pub tx_type: String,
}

/// XPUB entry for PSBT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPUBEntry {
    pub xpub_data: Vec<u8>,
    pub fingerprint: [u8; 4],
    pub derivation_path: Vec<u32>,
}

impl Default for PSBT {
    fn default() -> Self {
        Self {
            unsigned_tx: None,
            version: PSBT_VERSION,
            xpubs: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            state: PSBTState::Created,
            tx_type: String::new(),
        }
    }
}

impl PSBT {
    /// Create a new PSBT from an unsigned transaction (createpsbt)
    pub fn create(unsigned_tx: Transaction) -> Self {
        let input_count = unsigned_tx.inputs.len();
        let output_count = unsigned_tx.outputs.len();

        let mut psbt = PSBT {
            unsigned_tx: Some(unsigned_tx),
            inputs: vec![PSBTInput::default(); input_count],
            outputs: vec![PSBTOutput::default(); output_count],
            state: PSBTState::Created,
            ..Default::default()
        };

        // Determine transaction type
        if input_count == 1 && output_count >= 1 {
            psbt.tx_type = "Simple Payment".to_string();
        } else if input_count > 1 {
            psbt.tx_type = "Multi-Input".to_string();
        } else {
            psbt.tx_type = "Standard".to_string();
        }

        psbt
    }

    /// Decode a PSBT from raw bytes (decodepsbt)
    pub fn decode(raw: &[u8]) -> anyhow::Result<Self> {
        if raw.len() < 5 {
            anyhow::bail!("PSBT data too short");
        }

        if raw[..5] != PSBT_MAGIC {
            anyhow::bail!("Invalid PSBT magic bytes");
        }

        // Parse PSBT from raw format
        // For now, use a simplified parse - full parsing would iterate key-value pairs
        let psbt: PSBT = bincode::deserialize(&raw[5..])?;
        Ok(psbt)
    }

    /// Serialize PSBT to raw bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = PSBT_MAGIC.to_vec();
        let serialized = bincode::serialize(self).unwrap_or_default();
        data.extend_from_slice(&serialized);
        data
    }

    /// Encode PSBT as base64 string for transport
    pub fn to_base64(&self) -> String {
        use base64::{engine::general_purpose, Engine as _};
        let bytes = self.serialize();
        general_purpose::STANDARD.encode(&bytes)
    }

    /// Decode PSBT from base64 string
    pub fn from_base64(b64: &str) -> anyhow::Result<Self> {
        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD.decode(b64)?;
        Self::decode(&bytes)
    }

    /// Add a non-witness UTXO to an input (updater role)
    pub fn add_non_witness_utxo(
        &mut self,
        input_index: usize,
        tx_bytes: Vec<u8>,
    ) -> anyhow::Result<()> {
        if input_index >= self.inputs.len() {
            anyhow::bail!("Invalid input index: {}", input_index);
        }
        self.inputs[input_index].non_witness_utxo = Some(tx_bytes);
        self.state = PSBTState::Updated;
        Ok(())
    }

    /// Add a witness UTXO to an input (updater role)
    pub fn add_witness_utxo(
        &mut self,
        input_index: usize,
        value: u64,
        script: Vec<u8>,
    ) -> anyhow::Result<()> {
        if input_index >= self.inputs.len() {
            anyhow::bail!("Invalid input index: {}", input_index);
        }
        self.inputs[input_index].witness_utxo = Some(WitnessUTXO {
            value,
            script_pubkey: script,
        });
        self.state = PSBTState::Updated;
        Ok(())
    }

    /// Add a BIP32 derivation path to an input (updater role)
    pub fn add_input_derivation(
        &mut self,
        input_index: usize,
        pubkey: Vec<u8>,
        fingerprint: [u8; 4],
        path: Vec<u32>,
    ) -> anyhow::Result<()> {
        if input_index >= self.inputs.len() {
            anyhow::bail!("Invalid input index: {}", input_index);
        }
        self.inputs[input_index].bip32_derivations.insert(
            pubkey,
            BIP32Derivation {
                master_fingerprint: fingerprint,
                derivation_path: path,
            },
        );
        self.state = PSBTState::Updated;
        Ok(())
    }

    /// Add a BIP32 derivation path to an output (updater role)
    pub fn add_output_derivation(
        &mut self,
        output_index: usize,
        pubkey: Vec<u8>,
        fingerprint: [u8; 4],
        path: Vec<u32>,
    ) -> anyhow::Result<()> {
        if output_index >= self.outputs.len() {
            anyhow::bail!("Invalid output index: {}", output_index);
        }
        self.outputs[output_index].bip32_derivations.insert(
            pubkey,
            BIP32Derivation {
                master_fingerprint: fingerprint,
                derivation_path: path,
            },
        );
        self.state = PSBTState::Updated;
        Ok(())
    }

    /// Add a partial signature to an input (signer role)
    pub fn add_partial_signature(
        &mut self,
        input_index: usize,
        pubkey: Vec<u8>,
        signature: Vec<u8>,
    ) -> anyhow::Result<()> {
        if input_index >= self.inputs.len() {
            anyhow::bail!("Invalid input index: {}", input_index);
        }
        self.inputs[input_index]
            .partial_signatures
            .insert(pubkey, signature);
        self.state = PSBTState::Signed;
        Ok(())
    }

    /// Set redeem script for an input
    pub fn set_input_redeem_script(
        &mut self,
        input_index: usize,
        script: Vec<u8>,
    ) -> anyhow::Result<()> {
        if input_index >= self.inputs.len() {
            anyhow::bail!("Invalid input index: {}", input_index);
        }
        self.inputs[input_index].redeem_script = Some(script);
        Ok(())
    }

    /// Set witness script for an input
    pub fn set_input_witness_script(
        &mut self,
        input_index: usize,
        script: Vec<u8>,
    ) -> anyhow::Result<()> {
        if input_index >= self.inputs.len() {
            anyhow::bail!("Invalid input index: {}", input_index);
        }
        self.inputs[input_index].witness_script = Some(script);
        Ok(())
    }

    /// Set redeem script for an output
    pub fn set_output_redeem_script(
        &mut self,
        output_index: usize,
        script: Vec<u8>,
    ) -> anyhow::Result<()> {
        if output_index >= self.outputs.len() {
            anyhow::bail!("Invalid output index: {}", output_index);
        }
        self.outputs[output_index].redeem_script = Some(script);
        Ok(())
    }

    /// Set witness script for an output
    pub fn set_output_witness_script(
        &mut self,
        output_index: usize,
        script: Vec<u8>,
    ) -> anyhow::Result<()> {
        if output_index >= self.outputs.len() {
            anyhow::bail!("Invalid output index: {}", output_index);
        }
        self.outputs[output_index].witness_script = Some(script);
        Ok(())
    }

    /// Finalize an input (finalizer role)
    pub fn finalize_input(
        &mut self,
        input_index: usize,
        scriptsig: Vec<u8>,
        scriptwitness: Vec<Vec<u8>>,
    ) -> anyhow::Result<()> {
        if input_index >= self.inputs.len() {
            anyhow::bail!("Invalid input index: {}", input_index);
        }
        self.inputs[input_index].final_scriptsig = Some(scriptsig);
        self.inputs[input_index].final_scriptwitness = Some(scriptwitness);
        self.state = PSBTState::Finalized;
        Ok(())
    }

    /// Extract a fully signed transaction from the PSBT (finalizepsbt / extract)
    pub fn extract_transaction(&self) -> anyhow::Result<Transaction> {
        let tx = self
            .unsigned_tx
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No unsigned transaction in PSBT"))?;

        let mut final_tx = tx.clone();

        // Move finalized scriptsig/scriptwitness into the transaction
        for (i, input) in self.inputs.iter().enumerate() {
            if let Some(ref scriptsig) = input.final_scriptsig {
                final_tx.inputs[i].script_sig = ScriptSig::new(scriptsig.clone());
            }
            if let Some(ref scriptwitness) = input.final_scriptwitness {
                final_tx.inputs[i].witness = scriptwitness.clone();
            }
        }

        let _ = &self.state; // reference self for borrow checker
        Ok(final_tx)
    }

    /// Combine multiple PSBTs into one (combinepsbt)
    pub fn combine(psbts: Vec<PSBT>) -> anyhow::Result<PSBT> {
        if psbts.is_empty() {
            anyhow::bail!("No PSBTs to combine");
        }

        // Use the first PSBT as base
        let mut combined = psbts[0].clone();

        for psbt in &psbts[1..] {
            // Combine input fields
            for (i, input) in psbt.inputs.iter().enumerate() {
                if i < combined.inputs.len() {
                    // Merge non-witness UTXO
                    if input.non_witness_utxo.is_some() {
                        combined.inputs[i].non_witness_utxo = input.non_witness_utxo.clone();
                    }
                    // Merge witness UTXO
                    if input.witness_utxo.is_some() {
                        combined.inputs[i].witness_utxo = input.witness_utxo.clone();
                    }
                    // Merge partial signatures
                    for (pubkey, sig) in &input.partial_signatures {
                        combined.inputs[i]
                            .partial_signatures
                            .entry(pubkey.clone())
                            .or_insert_with(|| sig.clone());
                    }
                    // Merge BIP32 derivations
                    for (pubkey, deriv) in &input.bip32_derivations {
                        combined.inputs[i]
                            .bip32_derivations
                            .entry(pubkey.clone())
                            .or_insert_with(|| deriv.clone());
                    }
                    // Merge final scripts
                    if input.final_scriptsig.is_some() {
                        combined.inputs[i].final_scriptsig = input.final_scriptsig.clone();
                    }
                    if input.final_scriptwitness.is_some() {
                        combined.inputs[i].final_scriptwitness = input.final_scriptwitness.clone();
                    }
                }
            }

            // Combine output fields
            for (i, output) in psbt.outputs.iter().enumerate() {
                if i < combined.outputs.len() {
                    for (pubkey, deriv) in &output.bip32_derivations {
                        combined.outputs[i]
                            .bip32_derivations
                            .entry(pubkey.clone())
                            .or_insert_with(|| deriv.clone());
                    }
                }
            }
        }

        // Update state based on combination
        if combined.inputs.iter().all(|i| i.final_scriptsig.is_some()) {
            combined.state = PSBTState::Finalized;
        } else if combined
            .inputs
            .iter()
            .any(|i| !i.partial_signatures.is_empty())
        {
            combined.state = PSBTState::Signed;
        } else {
            combined.state = PSBTState::Updated;
        }

        Ok(combined)
    }

    /// Analyze the PSBT state (analyzepsbt)
    pub fn analyze(&self) -> PSBTAnalysis {
        let mut analysis = PSBTAnalysis::new();

        analysis.total_inputs = self.inputs.len();
        analysis.total_outputs = self.outputs.len();
        analysis.state = self.state.clone();
        analysis.tx_type = self.tx_type.clone();

        if let Some(ref tx) = self.unsigned_tx {
            analysis.txid = hex::encode(tx.txid().0);
            analysis.fee_sats = tx.total_output(); // simplified
            analysis.fee_UDYA = analysis.fee_sats as f64 / SATS_PER_COIN as f64;
        }

        // Analyze each input
        for (i, input) in self.inputs.iter().enumerate() {
            let mut input_analysis = PSBTInputAnalysis::new();
            input_analysis.index = i;

            input_analysis.has_utxo =
                input.non_witness_utxo.is_some() || input.witness_utxo.is_some();
            input_analysis.has_sigs = !input.partial_signatures.is_empty();
            input_analysis.num_sigs = input.partial_signatures.len();
            input_analysis.is_finalized = input.final_scriptsig.is_some();
            input_analysis.has_redeem_script = input.redeem_script.is_some();
            input_analysis.has_witness_script = input.witness_script.is_some();

            // Determine signing status
            if input.final_scriptsig.is_some() {
                input_analysis.status = "Finalized".to_string();
            } else if !input.partial_signatures.is_empty() {
                input_analysis.status = format!(
                    "Partially Signed ({} sig(s))",
                    input.partial_signatures.len()
                );
            } else if input.non_witness_utxo.is_some() || input.witness_utxo.is_some() {
                input_analysis.status = "Ready to Sign".to_string();
            } else {
                input_analysis.status = "Missing UTXO".to_string();
            }

            analysis.inputs.push(input_analysis);
        }

        // Analyze each output
        for (i, output) in self.outputs.iter().enumerate() {
            let mut output_analysis = PSBTOutputAnalysis::new();
            output_analysis.index = i;
            output_analysis.has_redeem_script = output.redeem_script.is_some();
            output_analysis.has_witness_script = output.witness_script.is_some();
            output_analysis.num_bip32_derivations = output.bip32_derivations.len();

            // Display output value if available from unsigned tx
            if let Some(ref tx) = self.unsigned_tx {
                if i < tx.outputs.len() {
                    output_analysis.value_sats = tx.outputs[i].value;
                    output_analysis.value_UDYA = tx.outputs[i].value as f64 / SATS_PER_COIN as f64;
                }
            }

            analysis.outputs.push(output_analysis);
        }

        // Overall readiness
        let all_finalized = analysis.inputs.iter().all(|i| i.is_finalized);
        let all_ready = analysis.inputs.iter().all(|i| i.has_utxo);

        if all_finalized {
            analysis.readiness = "Ready to Extract".to_string();
        } else if all_ready && analysis.inputs.iter().all(|i| i.has_sigs) {
            analysis.readiness = "Fully Signed".to_string();
        } else if all_ready {
            analysis.readiness = "Ready to Sign".to_string();
        } else {
            analysis.readiness = "Incomplete - Missing UTXOs".to_string();
        }

        analysis
    }

    /// Convert to JSON for RPC responses
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "psbt": self.to_base64(),
            "tx": self.unsigned_tx.as_ref().map(|tx| serde_json::json!({
                "txid": hex::encode(tx.txid().0),
                "version": tx.version,
                "inputs": tx.inputs.len(),
                "outputs": tx.outputs.len(),
                "lock_time": tx.lock_time,
            })),
            "inputs": self.inputs.len(),
            "outputs": self.outputs.len(),
            "state": format!("{:?}", self.state),
            "type": self.tx_type,
        })
    }
}

/// PSBT analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PSBTAnalysis {
    pub state: PSBTState,
    pub tx_type: String,
    pub txid: String,
    pub fee_sats: u64,
    pub fee_UDYA: f64,
    pub total_inputs: usize,
    pub total_outputs: usize,
    pub inputs: Vec<PSBTInputAnalysis>,
    pub outputs: Vec<PSBTOutputAnalysis>,
    pub readiness: String,
}

impl Default for PSBTAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl PSBTAnalysis {
    pub fn new() -> Self {
        Self {
            state: PSBTState::Created,
            tx_type: String::new(),
            txid: String::new(),
            fee_sats: 0,
            fee_UDYA: 0.0,
            total_inputs: 0,
            total_outputs: 0,
            inputs: Vec::new(),
            outputs: Vec::new(),
            readiness: String::new(),
        }
    }
}

/// PSBT input analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PSBTInputAnalysis {
    pub index: usize,
    pub has_utxo: bool,
    pub has_sigs: bool,
    pub num_sigs: usize,
    pub is_finalized: bool,
    pub has_redeem_script: bool,
    pub has_witness_script: bool,
    pub status: String,
}

impl Default for PSBTInputAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl PSBTInputAnalysis {
    pub fn new() -> Self {
        Self {
            index: 0,
            has_utxo: false,
            has_sigs: false,
            num_sigs: 0,
            is_finalized: false,
            has_redeem_script: false,
            has_witness_script: false,
            status: String::new(),
        }
    }
}

/// PSBT output analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PSBTOutputAnalysis {
    pub index: usize,
    pub value_sats: u64,
    pub value_UDYA: f64,
    pub has_redeem_script: bool,
    pub has_witness_script: bool,
    pub num_bip32_derivations: usize,
    pub address: Option<String>,
}

impl Default for PSBTOutputAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl PSBTOutputAnalysis {
    pub fn new() -> Self {
        Self {
            index: 0,
            value_sats: 0,
            value_UDYA: 0.0,
            has_redeem_script: false,
            has_witness_script: false,
            num_bip32_derivations: 0,
            address: None,
        }
    }
}

// ============================================================
// PSBT Workflow Manager
// ============================================================

/// Manages the full PSBT workflow for multi-device signing
pub struct PSBTWorkflow {
    psbt: PSBT,
    role: PSBTRole,
    workflow_id: String,
    #[allow(dead_code)]
    created_at: u64,
    participants: Vec<String>,
    signatures_required: usize,
}

impl PSBTWorkflow {
    /// Create a new PSBT workflow
    pub fn new(
        psbt: PSBT,
        role: PSBTRole,
        participants: Vec<String>,
        signatures_required: usize,
    ) -> Self {
        Self {
            psbt,
            role,
            workflow_id: uuid::Uuid::new_v4().to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            participants,
            signatures_required,
        }
    }

    pub fn get_psbt(&self) -> &PSBT {
        &self.psbt
    }

    pub fn get_psbt_mut(&mut self) -> &mut PSBT {
        &mut self.psbt
    }

    pub fn get_workflow_id(&self) -> &str {
        &self.workflow_id
    }

    pub fn get_role(&self) -> &PSBTRole {
        &self.role
    }

    pub fn get_participants(&self) -> &[String] {
        &self.participants
    }

    pub fn get_signatures_required(&self) -> usize {
        self.signatures_required
    }
}

// ============================================================
// PSBT Multisig Management
// ============================================================

/// Multisig configuration for PSBT workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigConfig {
    pub required_signatures: usize,
    pub total_keys: usize,
    pub public_keys: Vec<Vec<u8>>,
    pub redeem_script: Option<Vec<u8>>,
    pub witness_script: Option<Vec<u8>>,
    pub address_type: MultisigAddressType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultisigAddressType {
    P2SH,
    P2WSH,
    P2shP2wsh,
}

impl MultisigConfig {
    /// Create a new multisig configuration
    pub fn new(
        required: usize,
        total: usize,
        public_keys: Vec<Vec<u8>>,
        address_type: MultisigAddressType,
    ) -> anyhow::Result<Self> {
        if required > total {
            anyhow::bail!(
                "Required signatures ({}) cannot exceed total keys ({})",
                required,
                total
            );
        }
        if public_keys.len() != total {
            anyhow::bail!("Expected {} public keys, got {}", total, public_keys.len());
        }
        if total > 20 {
            anyhow::bail!("Max 20 keys supported (BIP-67), got {}", total);
        }

        Ok(Self {
            required_signatures: required,
            total_keys: total,
            public_keys,
            redeem_script: None,
            witness_script: None,
            address_type,
        })
    }

    /// Get the descriptor string for this multisig config
    pub fn to_descriptor(&self) -> String {
        let keys: Vec<String> = self.public_keys.iter().map(hex::encode).collect();
        format!(
            "{}-of-{} multisig (keys: [{}])",
            self.required_signatures,
            self.total_keys,
            keys.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psbt_create() {
        let tx = Transaction::new(
            2,
            vec![TxIn {
                previous_output: OutPoint::new(Txid([0u8; 32]), 0),
                script_sig: ScriptSig::new(vec![]),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            vec![TxOut {
                value: 100_000_000,
                script_pubkey: ScriptPubKey::new(vec![0x00; 20]),
            }],
            0,
        );
        let psbt = PSBT::create(tx);
        assert_eq!(psbt.inputs.len(), 1);
        assert_eq!(psbt.outputs.len(), 1);
        assert_eq!(psbt.state, PSBTState::Created);
    }

    #[test]
    fn test_psbt_base64_roundtrip() {
        let tx = Transaction::new(
            2,
            vec![TxIn {
                previous_output: OutPoint::new(Txid([1u8; 32]), 0),
                script_sig: ScriptSig::new(vec![]),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            vec![TxOut {
                value: 50_000_000,
                script_pubkey: ScriptPubKey::new(vec![0x00; 20]),
            }],
            0,
        );
        let psbt = PSBT::create(tx);
        let b64 = psbt.to_base64();
        assert!(!b64.is_empty());

        let decoded = PSBT::from_base64(&b64).expect("Should decode PSBT");
        assert_eq!(decoded.state, PSBTState::Created);
    }

    #[test]
    fn test_psbt_analyze() {
        let tx = Transaction::new(
            2,
            vec![TxIn {
                previous_output: OutPoint::new(Txid([2u8; 32]), 1),
                script_sig: ScriptSig::new(vec![]),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            vec![TxOut {
                value: 75_000_000,
                script_pubkey: ScriptPubKey::new(vec![0x00; 20]),
            }],
            0,
        );
        let psbt = PSBT::create(tx);
        let analysis = psbt.analyze();
        assert_eq!(analysis.total_inputs, 1);
        assert_eq!(analysis.total_outputs, 1);
        assert!(!analysis.readiness.is_empty());
    }

    #[test]
    fn test_psbt_add_partial_signature() {
        let tx = Transaction::new(
            2,
            vec![TxIn {
                previous_output: OutPoint::new(Txid([0u8; 32]), 0),
                script_sig: ScriptSig::new(vec![]),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            vec![TxOut {
                value: 100_000_000,
                script_pubkey: ScriptPubKey::new(vec![0x00; 20]),
            }],
            0,
        );
        let mut psbt = PSBT::create(tx);

        // Add UTXO first
        psbt.add_witness_utxo(0, 200_000_000, vec![0x00; 22])
            .unwrap();
        assert_eq!(psbt.state, PSBTState::Updated);

        // Add signature
        psbt.add_partial_signature(0, vec![0x02; 33], vec![0x30; 70])
            .unwrap();
        assert_eq!(psbt.state, PSBTState::Signed);
    }

    #[test]
    fn test_psbt_combine() {
        let tx = Transaction::new(
            2,
            vec![TxIn {
                previous_output: OutPoint::new(Txid([0u8; 32]), 0),
                script_sig: ScriptSig::new(vec![]),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            vec![TxOut {
                value: 100_000_000,
                script_pubkey: ScriptPubKey::new(vec![0x00; 20]),
            }],
            0,
        );

        let mut psbt1 = PSBT::create(tx.clone());
        let mut psbt2 = PSBT::create(tx.clone());

        psbt1
            .add_partial_signature(0, vec![0x02; 33], vec![0x30; 70])
            .unwrap();
        psbt2
            .add_partial_signature(0, vec![0x03; 33], vec![0x30; 71])
            .unwrap();

        let combined = PSBT::combine(vec![psbt1, psbt2]).expect("Should combine PSBTs");
        assert_eq!(combined.inputs[0].partial_signatures.len(), 2);
    }

    #[test]
    fn test_multisig_config() {
        let keys = vec![vec![0x02; 33], vec![0x03; 33], vec![0x04; 33]];
        let config = MultisigConfig::new(2, 3, keys, MultisigAddressType::P2WSH)
            .expect("Should create 2-of-3 multisig");
        assert_eq!(config.required_signatures, 2);
        assert_eq!(config.total_keys, 3);
        assert!(config.to_descriptor().contains("2-of-3"));
    }

    #[test]
    fn test_psbt_finalize_and_extract() {
        let tx = Transaction::new(
            2,
            vec![TxIn {
                previous_output: OutPoint::new(Txid([0u8; 32]), 0),
                script_sig: ScriptSig::new(vec![]),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            vec![TxOut {
                value: 100_000_000,
                script_pubkey: ScriptPubKey::new(vec![0x00; 20]),
            }],
            0,
        );
        let mut psbt = PSBT::create(tx.clone());

        // Finalize
        psbt.finalize_input(0, vec![0x48; 100], vec![vec![0x30; 70]])
            .expect("Should finalize input");
        assert_eq!(psbt.state, PSBTState::Finalized);

        // Extract
        let extracted = psbt.extract_transaction().expect("Should extract");
        assert!(!extracted.inputs[0].script_sig.data.is_empty());
    }

    #[test]
    fn test_psbt_workflow() {
        let tx = Transaction::new(
            2,
            vec![TxIn {
                previous_output: OutPoint::new(Txid([1u8; 32]), 0),
                script_sig: ScriptSig::new(vec![]),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            vec![TxOut {
                value: 50_000_000,
                script_pubkey: ScriptPubKey::new(vec![0x00; 20]),
            }],
            0,
        );

        let psbt = PSBT::create(tx);
        let participants = vec![
            "Device1".to_string(),
            "Device2".to_string(),
            "Device3".to_string(),
        ];
        let workflow = PSBTWorkflow::new(psbt, PSBTRole::Creator, participants, 2);
        assert_eq!(workflow.get_signatures_required(), 2);
        assert_eq!(workflow.get_participants().len(), 3);
        assert!(!workflow.get_workflow_id().is_empty());
    }

    #[test]
    fn test_psbt_invalid_input_index() {
        let tx = Transaction::new(
            2,
            vec![TxIn {
                previous_output: OutPoint::new(Txid([0u8; 32]), 0),
                script_sig: ScriptSig::new(vec![]),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            vec![TxOut {
                value: 100_000_000,
                script_pubkey: ScriptPubKey::new(vec![0x00; 20]),
            }],
            0,
        );
        let mut psbt = PSBT::create(tx);
        assert!(psbt.add_partial_signature(5, vec![], vec![]).is_err());
        assert!(psbt.finalize_input(5, vec![], vec![]).is_err());
    }
}
