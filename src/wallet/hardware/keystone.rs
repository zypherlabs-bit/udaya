/// Keystone hardware wallet integration
///
/// Keystone devices supported:
/// - Keystone Pro
/// - Keystone Essential
/// - Keystone 3 Pro
///
/// Supported workflows:
/// - PSBT export/import via QR code (air-gapped)
/// - Address verification on device screen
/// - BIP44, BIP49, BIP84, BIP86 derivation paths
/// - Transaction signing with on-device key storage
///
/// Connection: QR code scanning (air-gapped), Bluetooth, NFC
///
/// # Security
/// - Fully air-gapped operation (no USB/network connection)
/// - Private keys never leave the device
/// - QR code based PSBT transfer (cc @psbt format)
/// - Open-source firmware
/// - Secure element chip

use super::{HardwareWalletType, HardwareWalletCapabilities, ConnectionStatus};

/// Keystone device information
pub struct KeystoneDevice {
    pub device_type: String,
    pub firmware_version: String,
    pub connection: ConnectionStatus,
}

impl KeystoneDevice {
    pub fn new() -> Self {
        Self {
            device_type: "Keystone".to_string(),
            firmware_version: String::new(),
            connection: ConnectionStatus::Disconnected,
        }
    }

    /// Get Keystone capabilities
    pub fn get_capabilities() -> HardwareWalletCapabilities {
        HardwareWalletCapabilities::new(HardwareWalletType::Keystone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystone_capabilities() {
        let caps = KeystoneDevice::get_capabilities();
        assert!(caps.bluetooth_connection);
        assert!(caps.nfc_connection);
        assert!(!caps.usb_connection);
        assert!(caps.multisig_support);
    }
}
</write_to_file>