use super::{HardwareDeviceInfo, HardwareTransport, HardwareWallet, HardwareWalletError, HardwareWalletErrorCode, HardwareWalletResult, HardwareDeviceType, PsbtData};

/// Keystone hardware wallet implementation (Bluetooth/NFC transport)
pub struct KeystoneWallet {
    connected: bool,
    info: Option<HardwareDeviceInfo>,
}

impl KeystoneWallet {
    pub fn new() -> Self {
        Self {
            connected: false,
            info: None,
        }
    }
}

impl HardwareWallet for KeystoneWallet {
    fn device_type(&self) -> HardwareDeviceType {
        HardwareDeviceType::Keystone
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn connect(&mut self) -> HardwareWalletResult<()> {
        if self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceBusy,
                message: "Keystone device already connected".to_string(),
            });
        }
        self.connected = true;
        self.info = Some(HardwareDeviceInfo {
            device_type: HardwareDeviceType::Keystone,
            transport: HardwareTransport::Bluetooth,
            manufacturer: "Keystone".to_string(),
            model: "Keystone 3 Pro".to_string(),
            firmware_version: "1.0.0".to_string(),
            serial_number: "KEYSTONE-0000".to_string(),
            is_connected: true,
        });
        Ok(())
    }

    fn disconnect(&mut self) -> HardwareWalletResult<()> {
        self.connected = false;
        self.info = None;
        Ok(())
    }

    fn get_device_info(&self) -> HardwareWalletResult<HardwareDeviceInfo> {
        self.info.clone().ok_or_else(|| HardwareWalletError {
            code: HardwareWalletErrorCode::DeviceNotFound,
            message: "Keystone device not connected".to_string(),
        })
    }

    fn get_master_fingerprint(&self) -> HardwareWalletResult<[u8; 4]> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Keystone device not connected".to_string(),
            });
        }
        Ok([0x56; 4])
    }

    fn get_extended_pubkey(&self, _path: &str) -> HardwareWalletResult<String> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Keystone device not connected".to_string(),
            });
        }
        Err(HardwareWalletError {
            code: HardwareWalletErrorCode::NotImplemented,
            message: "Keystone extended pubkey retrieval not yet implemented".to_string(),
        })
    }

    fn sign_transaction(&self, _psbt: &PsbtData) -> HardwareWalletResult<PsbtData> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Keystone device not connected".to_string(),
            });
        }
        Err(HardwareWalletError {
            code: HardwareWalletErrorCode::NotImplemented,
            message: "Keystone transaction signing not yet implemented".to_string(),
        })
    }
}
