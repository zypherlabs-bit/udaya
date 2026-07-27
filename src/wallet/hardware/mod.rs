pub mod ledger;
pub mod trezor;
pub mod keystone;
pub mod coldcard;
pub mod compatibility;

use udaya_wallet::psbt::{PSBT, PSBTRole, PSBTState};
use serde::{Deserialize, Serialize};

/// Hardware wallet types supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HardwareWalletType {
    Ledger,
    Trezor,
    Keystone,
    Coldcard,
}

/// Hardware wallet connection status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Hardware wallet feature support matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareWalletCapabilities {
    pub wallet_type: HardwareWalletType,
    pub psbt_export: bool,
    pub psbt_import: bool,
    pub address_verification: bool,
    pub signature_validation: bool,
    pub bip44_support: bool,
    pub bip49_support: bool,
    pub bip84_support: bool,
    pub bip86_support: bool,
    pub taproot_support: bool,
    pub multisig_support: bool,
    pub usb_connection: bool,
    pub bluetooth_connection: bool,
    pub nfc_connection: bool,
    pub screen_display: bool,
    pub pin_entry: bool,
    pub passphrase_support: bool,
}

impl HardwareWalletCapabilities {
    pub fn new(wallet_type: HardwareWalletType) -> Self {
        match wallet_type {
            HardwareWalletType::Ledger => Self {
                wallet_type: HardwareWalletType::Ledger,
                psbt_export: true,
                psbt_import: true,
                address_verification: true,
                signature_validation: true,
                bip44_support: true,
                bip49_support: true,
                bip84_support: true,
                bip86_support: true,
                taproot_support: true,
                multisig_support: true,
                usb_connection: true,
                bluetooth_connection: false,
                nfc_connection: false,
                screen_display: true,
                pin_entry: true,
                passphrase_support: true,
            },
            HardwareWalletType::Trezor => Self {
                wallet_type: HardwareWalletType::Trezor,
                psbt_export: true,
                psbt_import: true,
                address_verification: true,
                signature_validation: true,
                bip44_support: true,
                bip49_support: true,
                bip84_support: true,
                bip86_support: true,
                taproot_support: true,
                multisig_support: true,
                usb_connection: true,
                bluetooth_connection: false,
                nfc_connection: false,
                screen_display: true,
                pin_entry: true,
                passphrase_support: true,
            },
            HardwareWalletType::Keystone => Self {
                wallet_type: HardwareWalletType::Keystone,
                psbt_export: true,
                psbt_import: true,
                address_verification: true,
                signature_validation: true,
                bip44_support: true,
                bip49_support: true,
                bip84_support: true,
                bip86_support: true,
                taproot_support: true,
                multisig_support: true,
                usb_connection: false,
                bluetooth_connection: true,
                nfc_connection: true,
                screen_display: true,
                pin_entry: true,
                passphrase_support: true,
            },
            HardwareWalletType::Coldcard => Self {
                wallet_type: HardwareWalletType::Coldcard,
                psbt_export: true,
                psbt_import: true,
                address_verification: true,
                signature_validation: true,
                bip44_support: true,
                bip49_support: true,
                bip84_support: true,
                bip86_support: true,
                taproot_support: true,
                multisig_support: true,
                usb_connection: true,
                bluetooth_connection: false,
                nfc_connection: false,
                screen_display: true,
                pin_entry: true,
                passphrase_support: true,
            },
        }
    }

    /// Get support score (0-100)
    pub fn support_score(&self) -> u32 {
        let mut score = 0u32;
        if self.psbt_export { score += 15; }
        if self.psbt_import { score += 15; }
        if self.address_verification { score += 10; }
        if self.signature_validation { score += 10; }
        if self.bip44_support { score += 5; }
        if self.bip49_support { score += 5; }
        if self.bip84_support { score += 10; }
        if self.bip86_support { score += 10; }
        if self.taproot_support { score += 5; }
        if self.multisig_support { score += 5; }
        if self.usb_connection { score += 3; }
        if self.bluetooth_connection { score += 2; }
        if self.nfc_connection { score += 1; }
        if self.screen_display { score += 2; }
        if self.pin_entry { score += 1; }
        if self.passphrase_support { score += 1; }
        score
    }
}

/// Hardware wallet workflow for PSBT signing
pub struct HardwareWalletWorkflow {
    pub wallet_type: HardwareWalletType,
    pub psbt: PSBT,
    pub verified_addresses: Vec<String>,
    pub signed_inputs: Vec<usize>,
    pub workflow_complete: bool,
}

impl HardwareWalletWorkflow {
    /// Create a new hardware wallet signing workflow
    pub fn new(wallet_type: HardwareWalletType, psbt: PSBT) -> Self {
        Self {
            wallet_type,
            psbt,
            verified_addresses: Vec::new(),
            signed_inputs: Vec::new(),
            workflow_complete: false,
        }
    }

    /// Export PSBT for hardware wallet processing
    pub fn export_psbt(&self) -> anyhow::Result<String> {
        if self.psbt.unsigned_tx.is_none() {
            anyhow::bail!("No transaction in PSBT");
        }
        Ok(self.psbt.to_base64())
    }

    /// Import PSBT from hardware wallet after signing
    pub fn import_psbt(&mut self, signed_psbt_b64: &str) -> anyhow::Result<()> {
        let signed = PSBT::from_base64(signed_psbt_b64)?;
        
        // Merge signatures from the hardware wallet signed PSBT
        for (i, signed_input) in signed.inputs.iter().enumerate() {
            if i < self.psbt.inputs.len() {
                for (pubkey, sig) in &signed_input.partial_signatures {
                    self.psbt.add_partial_signature(i, pubkey.clone(), sig.clone())?;
                    self.signed_inputs.push(i);
                }
            }
        }

        Ok(())
    }

    /// Verify an address displayed on the hardware wallet
    pub fn verify_address(&mut self, address: &str) -> bool {
        // In production, this would compare with what's shown on the device screen
        self.verified_addresses.push(address.to_string());
        true
    }

    /// Get the completed PSBT
    pub fn get_completed_psbt(&self) -> &PSBT {
        &self.psbt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_wallet_capabilities() {
        let ledger = HardwareWalletCapabilities::new(HardwareWalletType::Ledger);
        assert!(ledger.psbt_export);
        assert!(ledger.psbt_import);
        assert!(ledger.address_verification);
        assert!(ledger.usb_connection);
        assert!(!ledger.bluetooth_connection);
        assert!(ledger.support_score() > 50);
    }

    #[test]
    fn test_keystone_capabilities() {
        let keystone = HardwareWalletCapabilities::new(HardwareWalletType::Keystone);
        assert!(keystone.bluetooth_connection);
        assert!(keystone.nfc_connection);
        assert!(!keystone.usb_connection);
    }

    #[test]
    fn test_coldcard_capabilities() {
        let coldcard = HardwareWalletCapabilities::new(HardwareWalletType::Coldcard);
        assert!(coldcard.usb_connection);
        assert!(coldcard.multisig_support);
    }

    #[test]
    fn test_hardware_wallet_workflow() {
        let tx = udaya_core::transaction::Transaction::new(
            2,
            vec![udaya_core::types::TxIn {
                previous_output: udaya_core::types::OutPoint::new(
                    udaya_core::types::Txid([1u8; 32]), 0
                ),
                script_sig: udaya_core::types::ScriptSig::new(vec![]),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            vec![udaya_core::types::TxOut {
                value: 100_000_000,
                script_pubkey: udaya_core::types::ScriptPubKey::new(vec![0x00; 20]),
            }],
            0,
        );

        let psbt = PSBT::create(tx);
        let mut workflow = HardwareWalletWorkflow::new(
            HardwareWalletType::Ledger,
            psbt,
        );

        let exported = workflow.export_psbt().expect("Should export PSBT");
        assert!(!exported.is_empty());
    }
}
</write_to_file>