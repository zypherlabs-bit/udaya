/// Hardware wallet compatibility matrix
///
/// Comprehensive compatibility mapping for Udaya UDYA support
/// across all major hardware wallet vendors.
use super::{HardwareWalletType, HardwareWalletCapabilities};

/// Compatibility matrix entry
#[derive(Debug, Clone)]
pub struct CompatibilityEntry {
    pub wallet_type: HardwareWalletType,
    pub psbt_export: bool,
    pub psbt_import: bool,
    pub address_verify: bool,
    pub signature_validate: bool,
    pub bip44: bool,
    pub bip49: bool,
    pub bip84: bool,
    pub bip86: bool,
    pub taproot: bool,
    pub multisig: bool,
    pub connection_usb: bool,
    pub connection_bt: bool,
    pub connection_nfc: bool,
    pub connection_qr: bool,
    pub connection_sd: bool,
    pub screen: bool,
    pub pin: bool,
    pub passphrase: bool,
    pub notes: &'static str,
}

/// Get the full compatibility matrix for all supported hardware wallets
pub fn get_compatibility_matrix() -> Vec<CompatibilityEntry> {
    vec![
        CompatibilityEntry {
            wallet_type: HardwareWalletType::Ledger,
            psbt_export: true,
            psbt_import: true,
            address_verify: true,
            signature_validate: true,
            bip44: true,
            bip49: true,
            bip84: true,
            bip86: true,
            taproot: true,
            multisig: true,
            connection_usb: true,
            connection_bt: false,
            connection_nfc: false,
            connection_qr: false,
            connection_sd: false,
            screen: true,
            pin: true,
            passphrase: true,
            notes: "Requires Bitcoin app installed. USB HID connection via WebUSB/node-hid. Best supported across all platforms.",
        },
        CompatibilityEntry {
            wallet_type: HardwareWalletType::Trezor,
            psbt_export: true,
            psbt_import: true,
            address_verify: true,
            signature_validate: true,
            bip44: true,
            bip49: true,
            bip84: true,
            bip86: true,
            taproot: true,
            multisig: true,
            connection_usb: true,
            connection_bt: false,
            connection_nfc: false,
            connection_qr: false,
            connection_sd: false,
            screen: true,
            pin: true,
            passphrase: true,
            notes: "Trezor Bridge required on desktop. Model T supports Shamir backup. Open-source firmware.",
        },
        CompatibilityEntry {
            wallet_type: HardwareWalletType::Keystone,
            psbt_export: true,
            psbt_import: true,
            address_verify: true,
            signature_validate: true,
            bip44: true,
            bip49: true,
            bip84: true,
            bip86: true,
            taproot: true,
            multisig: true,
            connection_usb: false,
            connection_bt: true,
            connection_nfc: true,
            connection_qr: true,
            connection_sd: true,
            screen: true,
            pin: true,
            passphrase: true,
            notes: "Air-gapped QR code based PSBT transfer. Best for cold storage. Open-source firmware with secure element.",
        },
        CompatibilityEntry {
            wallet_type: HardwareWalletType::Coldcard,
            psbt_export: true,
            psbt_import: true,
            address_verify: true,
            signature_validate: true,
            bip44: true,
            bip49: true,
            bip84: true,
            bip86: true,
            taproot: true,
            multisig: true,
            connection_usb: true,
            connection_bt: false,
            connection_nfc: false,
            connection_qr: false,
            connection_sd: true,
            screen: true,
            pin: true,
            passphrase: true,
            notes: "Air-gapped MicroSD signing. PSBT filename format. Duress PIN. Best security features.",
        },
    ]
}

/// Generate a JSON-compatible compatibility report
pub fn generate_compatibility_report() -> serde_json::Value {
    let matrix = get_compatibility_matrix();
    let entries: Vec<serde_json::Value> = matrix.iter().map(|entry| {
        serde_json::json!({
            "wallet": format!("{:?}", entry.wallet_type),
            "features": {
                "psbt_export": entry.psbt_export,
                "psbt_import": entry.psbt_import,
                "address_verification": entry.address_verify,
                "signature_validation": entry.signature_validate,
                "bip44": entry.bip44,
                "bip49": entry.bip49,
                "bip84": entry.bip84,
                "bip86": entry.bip86,
                "taproot": entry.taproot,
                "multisig": entry.multisig,
            },
            "connections": {
                "usb": entry.connection_usb,
                "bluetooth": entry.connection_bt,
                "nfc": entry.connection_nfc,
                "qr_code": entry.connection_qr,
                "sd_card": entry.connection_sd,
            },
            "security": {
                "screen_display": entry.screen,
                "pin_protection": entry.pin,
                "passphrase": entry.passphrase,
            },
            "notes": entry.notes,
        })
    }).collect();

    serde_json::json!({
        "report_type": "hardware_wallet_compatibility_matrix",
        "version": "1.0.0",
        "network": "Udaya Mainnet",
        "coin_type": "UDYA (257')",
        "supported_devices": entries.len(),
        "entries": entries,
        "summary": {
            "all_support_psbt": matrix.iter().all(|e| e.psbt_export && e.psbt_import),
            "all_support_bip84": matrix.iter().all(|e| e.bip84),
            "all_support_multisig": matrix.iter().all(|e| e.multisig),
            "air_gapped_options": matrix.iter().filter(|e| e.connection_qr || e.connection_sd).count(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compatibility_matrix() {
        let matrix = get_compatibility_matrix();
        assert_eq!(matrix.len(), 4, "Should have 4 hardware wallet entries");
    }

    #[test]
    fn test_all_support_core_features() {
        let matrix = get_compatibility_matrix();
        assert!(matrix.iter().all(|e| e.bip84), "All wallets should support BIP84");
        assert!(matrix.iter().all(|e| e.bip86), "All wallets should support BIP86");
        assert!(matrix.iter().all(|e| e.psbt_export), "All wallets should support PSBT export");
        assert!(matrix.iter().all(|e| e.psbt_import), "All wallets should support PSBT import");
    }

    #[test]
    fn test_report_generation() {
        let report = generate_compatibility_report();
        assert_eq!(report["supported_devices"], 4);
        assert!(report["summary"]["all_support_psbt"].as_bool().unwrap());
    }
}
</write_to_file>