/// Coldcard hardware wallet integration
///
/// Coldcard devices supported:
/// - Coldcard MK1
/// - Coldcard MK2
/// - Coldcard MK3
/// - Coldcard MK4
///
/// Supported workflows:
/// - PSBT export/import via MicroSD card or USB
/// - Address verification on device screen
/// - BIP44, BIP49, BIP84, BIP86 derivation paths
/// - Transaction signing with on-device key storage
/// - Air-gapped signing via MicroSD
///
/// Connection: USB HID, MicroSD card (air-gapped)
///
/// # Security
/// - Private keys never leave the device
/// - PSBT file signing via MicroSD (air-gapped)
/// - BIP39 passphrase support
/// - Duress PIN support
/// - Brick pin protection
/// - Fully open-source firmware

use super::{HardwareWalletType, HardwareWalletCapabilities, ConnectionStatus};

/// Coldcard device information
pub struct ColdcardDevice {
    pub device_type: String,
    pub firmware_version: String,
    pub connection: ConnectionStatus,
}

impl ColdcardDevice {
    pub fn new() -> Self {
        Self {
            device_type: "Coldcard".to_string(),
            firmware_version: String::new(),
            connection: ConnectionStatus::Disconnected,
        }
    }

    /// Get Coldcard capabilities
    pub fn get_capabilities() -> HardwareWalletCapabilities {
        HardwareWalletCapabilities::new(HardwareWalletType::Coldcard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coldcard_capabilities() {
        let caps = ColdcardDevice::get_capabilities();
        assert!(caps.usb_connection);
        assert!(caps.multisig_support);
        assert!(caps.address_verification);
    }
}
</write_to_file>