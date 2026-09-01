use std::fmt;

/// Hardware wallet device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareDeviceType {
    Ledger,
    Trezor,
    Keystone,
    Coldcard,
}

impl fmt::Display for HardwareDeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HardwareDeviceType::Ledger => write!(f, "Ledger"),
            HardwareDeviceType::Trezor => write!(f, "Trezor"),
            HardwareDeviceType::Keystone => write!(f, "Keystone"),
            HardwareDeviceType::Coldcard => write!(f, "Coldcard"),
        }
    }
}

/// Connection transport for hardware wallets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareTransport {
    Usb,
    Bluetooth,
    Nfc,
    #[allow(dead_code)]
    Hid,
}

/// Hardware wallet device information
#[derive(Debug, Clone)]
pub struct HardwareDeviceInfo {
    pub device_type: HardwareDeviceType,
    pub transport: HardwareTransport,
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: String,
    pub serial_number: String,
    pub is_connected: bool,
}

/// PSBT (Partially Signed Bitcoin Transaction) for hardware wallet workflow
#[derive(Debug, Clone)]
pub struct PsbtData {
    pub base64: String,
    pub inputs_count: usize,
    pub outputs_count: usize,
    pub is_finalized: bool,
}

/// Result type for hardware wallet operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareWalletErrorCode {
    DeviceNotFound,
    DeviceBusy,
    UserCancelled,
    InvalidPin,
    TransportError,
    NotImplemented,
}

#[derive(Debug, Clone)]
pub struct HardwareWalletError {
    pub code: HardwareWalletErrorCode,
    pub message: String,
}

impl fmt::Display for HardwareWalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HardwareWalletError({:?}): {}", self.code, self.message)
    }
}

impl std::error::Error for HardwareWalletError {}

pub type HardwareWalletResult<T> = Result<T, HardwareWalletError>;

/// Common hardware wallet trait
pub trait HardwareWallet: Send + Sync {
    fn device_type(&self) -> HardwareDeviceType;
    fn is_connected(&self) -> bool;
    fn connect(&mut self) -> HardwareWalletResult<()>;
    fn disconnect(&mut self) -> HardwareWalletResult<()>;
    fn get_device_info(&self) -> HardwareWalletResult<HardwareDeviceInfo>;
    fn get_master_fingerprint(&self) -> HardwareWalletResult<[u8; 4]>;
    fn get_extended_pubkey(&self, path: &str) -> HardwareWalletResult<String>;
    fn sign_transaction(&self, psbt: &PsbtData) -> HardwareWalletResult<PsbtData>;
}
