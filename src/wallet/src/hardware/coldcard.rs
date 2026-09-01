use super::{HardwareDeviceInfo, HardwareTransport, HardwareWallet, HardwareWalletError, HardwareWalletErrorCode, HardwareWalletResult, HardwareDeviceType, PsbtData};

/// Coldcard hardware wallet implementation (USB transport)
pub struct ColdcardWallet {
    connected: bool,
    info: Option<HardwareDeviceInfo>,
}

impl ColdcardWallet {
    pub fn new() -> Self {
        Self {
            connected: false,
            info: None,
        }
    }
}

impl HardwareWallet for ColdcardWallet {
    fn device_type(&self) -> HardwareDeviceType {
        HardwareDeviceType::Coldcard
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn connect(&mut self) -> HardwareWalletResult<()> {
        if self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceBusy,
                message: "Coldcard device already connected".to_string(),
            });
        }
        self.connected = true;
        self.info = Some(HardwareDeviceInfo {
            device_type: HardwareDeviceType::Coldcard,
            transport: HardwareTransport::Usb,
            manufacturer: "Coinkite".to_string(),
            model: "Coldcard Mk4".to_string(),
            firmware_version: "5.0.0".to_string(),
            serial_number: "COLDCARD-0000".to_string(),
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
            message: "Coldcard device not connected".to_string(),
        })
    }

    fn get_master_fingerprint(&self) -> HardwareWalletResult<[u8; 4]> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Coldcard device not connected".to_string(),
            });
        }
        Ok([0x78; 4])
    }

    fn get_extended_pubkey(&self, _path: &str) -> HardwareWalletResult<String> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Coldcard device not connected".to_string(),
            });
        }
        Err(HardwareWalletError {
            code: HardwareWalletErrorCode::NotImplemented,
            message: "Coldcard extended pubkey retrieval not yet implemented".to_string(),
        })
    }

    fn sign_transaction(&self, _psbt: &PsbtData) -> HardwareWalletResult<PsbtData> {
        if !self.connected {
            return Err(HardwareWalletError {
                code: HardwareWalletErrorCode::DeviceNotFound,
                message: "Coldcard device not connected".to_string(),
            });
        }
        Err(HardwareWalletError {
            code: HardwareWalletErrorCode::NotImplemented,
            message: "Coldcard transaction signing not yet implemented".to_string(),
        })
    }
}
