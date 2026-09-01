use super::{HardwareDeviceInfo, HardwareTransport, HardwareWallet, HardwareWalletError, HardwareWalletErrorCode, HardwareWalletResult, HardwareDeviceType, PsbtData};

/// Trezor hardware wallet implementation (USB HID transport)
pub struct TrezorWallet {
    connected: bool,
    info: Option<HardwareDeviceInfo>,
}

impl TrezorWallet {
    pub fn new() -> Self {
        Self {
            connected: false,
            info: None,
        }
    }
}

impl HardwareWallet for TrezorWallet {
    fn device_type(&self) -> HardwareDeviceType {
        HardwareDeviceType::Trezor
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn connect(&mut self) -> HardwareWalletResult<()> {
        if self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceBusy,
                message: "Trezor device already connected".to_string(),
            });
        }
        self.connected = true;
        self.info = Some(HardwareDeviceInfo {
            device_type: HardwareDeviceType::Trezor,
            transport: HardwareTransport::Hid,
            manufacturer: "SatoshiLabs".to_string(),
            model: "Trezor Model T".to_string(),
            firmware_version: "2.8.0".to_string(),
            serial_number: "TREZOR-0000".to_string(),
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
            message: "Trezor device not connected".to_string(),
        })
    }

    fn get_master_fingerprint(&self) -> HardwareWalletResult<[u8; 4]> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Trezor device not connected".to_string(),
            });
        }
        Ok([0x34; 4])
    }

    fn get_extended_pubkey(&self, _path: &str) -> HardwareWalletResult<String> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Trezor device not connected".to_string(),
            });
        }
        Err(HardwareWalletError {
            code: HardwareWalletErrorCode::NotImplemented,
            message: "Trezor extended pubkey retrieval not yet implemented".to_string(),
        })
    }

    fn sign_transaction(&self, _psbt: &PsbtData) -> HardwareWalletResult<PsbtData> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Trezor device not connected".to_string(),
            });
        }
        Err(HardwareWalletError {
            code: HardwareWalletErrorCode::NotImplemented,
            message: "Trezor transaction signing not yet implemented".to_string(),
        })
    }
}
