/// Ledger hardware wallet integration
///
/// Ledger devices supported:
/// - Ledger Nano S
/// - Ledger Nano X
/// - Ledger Nano S Plus
/// - Ledger Stax
///
/// UDYA App Installation: Requires the Bitcoin app (supports PSBT natively)
///
/// Supported workflows:
/// - PSBT export/import via USB HID
/// - Address verification on device screen
/// - BIP44, BIP49, BIP84, BIP86 derivation paths
/// - Transaction signing with on-device key storage
///
/// Connection: USB HID (WebUSB for browser, node-hid for desktop)
///
/// # Security
/// - Private keys never leave the device
/// - Transaction details displayed on screen for verification
/// - PIN-protected access
/// - Passphrase support for hidden wallets

use super::{HardwareWalletType, HardwareWalletCapabilities, ConnectionStatus};

/// Ledger device information
pub struct LedgerDevice {
    pub device_type: String,
    pub firmware_version: String,
    pub connection: ConnectionStatus,
    pub app_version: Option<String>,
}

impl LedgerDevice {
    pub fn new() -> Self {
        Self {
            device_type: "Ledger".to_string(),
            firmware_version: String::new(),
            connection: ConnectionStatus::Disconnected,
            app_version: None,
        }
    }

    /// Get Ledger capabilities
    pub fn get_capabilities() -> HardwareWalletCapabilities {
        HardwareWalletCapabilities::new(HardwareWalletType::Ledger)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_capabilities() {
        let caps = LedgerDevice::get_capabilities();
        assert!(caps.psbt_export);
        assert!(caps.psbt_import);
        assert!(caps.address_verification);
        assert!(caps.usb_connection);
        assert!(!caps.bluetooth_connection);
    }

    #[test]
    fn test_ledger_device_creation() {
        let device = LedgerDevice::new();
        assert_eq!(device.device_type, "Ledger");
        assert_eq!(device.connection, ConnectionStatus::Disconnected);
    }
}
</write_to_file>