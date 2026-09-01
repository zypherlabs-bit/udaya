use super::{HardwareDeviceInfo, HardwareTransport, HardwareWallet, HardwareWalletError, HardwareWalletErrorCode, HardwareWalletResult, HardwareDeviceType, PsbtData};

/// Ledger hardware wallet implementation (USB HID transport)
pub struct LedgerWallet {
    connected: bool,
    info: Option<HardwareDeviceInfo>,
}

impl LedgerWallet {
    pub fn new() -> Self {
        Self {
            connected: false,
            info: None,
        }
    }
}

impl HardwareWallet for LedgerWallet {
    fn device_type(&self) -> HardwareDeviceType {
        HardwareDeviceType::Ledger
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn connect(&mut self) -> HardwareWalletResult<()> {
        if self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceBusy,
                message: "Ledger device already connected".to_string(),
            });
        }
        self.connected = true;
        self.info = Some(HardwareDeviceInfo {
            device_type: HardwareDeviceType::Ledger,
            transport: HardwareTransport::Hid,
            manufacturer: "Ledger".to_string(),
            model: "Nano S Plus".to_string(),
            firmware_version: "2.1.0".to_string(),
            serial_number: "LEDGER-0000".to_string(),
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
            message: "Ledger device not connected".to_string(),
        })
    }

    fn get_master_fingerprint(&self) -> HardwareWalletResult<[u8; 4]> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Ledger device not connected".to_string(),
            });
        }
        Ok([0x12; 4])
    }

    fn get_extended_pubkey(&self, _path: &str) -> HardwareWalletResult<String> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Ledger device not connected".to_string(),
            });
        }
        Err(HardwareWalletError {
            code: HardwareWalletErrorCode::NotImplemented,
            message: "Ledger extended pubkey retrieval not yet implemented".to_string(),
        })
    }

    fn sign_transaction(&self, _psbt: &PsbtData) -> HardwareWalletResult<PsbtData> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Ledger device not connected".to_string(),
            });
        }
        Err(HardwareWalletError {
            code: HardwareWalletErrorCode::NotImplemented,
            message: "Ledger transaction signing not yet implemented".to_string(),
        })
    }
}
