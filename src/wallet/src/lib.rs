pub mod crypto;
pub mod psbt;
pub mod threat_model;
pub mod treasury;
pub mod zeroize;

use crypto::{
    derive_bip44_path, derive_bip49_path, derive_bip84_path, derive_bip86_path, MAINNET_HRP,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use udaya_core::transaction::{create_p2pkh_transaction, Transaction};
use udaya_core::types::*;
use udaya_core::SATS_PER_COIN;

/// Udaya Wallet Module
/// Supports desktop, mobile, and browser extension wallets
///
/// BIP Standards Compliance:
/// - BIP44: Legacy P2PKH: m/44'/257'/0'/0/0
/// - BIP49: P2SH-SegWit:  m/49'/257'/0'/0/0
/// - BIP84: Native SegWit: m/84'/257'/0'/0/0 with bech32
/// - BIP86: Taproot:       m/86'/257'/0'/0/0 with bech32m

/// HD wallet path derivation level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DerivationPath {
    BIP44, // Legacy: m/44'/257'/0'/0/0
    BIP49, // SegWit: m/49'/257'/0'/0/0
    BIP84, // Native SegWit: m/84'/257'/0'/0/0
    BIP86, // Taproot: m/86'/257'/0'/0/0
    Custom(String),
}

/// Wallet type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalletType {
    Desktop,
    Mobile,
    BrowserExtension,
    Hardware,
    ColdStorage,
}

/// UTXO entry for wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletUTXO {
    pub txid: Txid,
    pub vout: u32,
    pub value: u64,
    pub address: String,
    pub script_pubkey: Vec<u8>,
    pub height: u64,
    pub confirmations: u64,
    pub is_coinbase: bool,
    pub is_spent: bool,
    pub is_frozen: bool,
}

/// Wallet transaction entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTx {
    pub txid: Txid,
    pub tx: Transaction,
    pub block_height: Option<u64>,
    pub block_hash: Option<BlockHash>,
    pub timestamp: u64,
    pub inputs: Vec<WalletInput>,
    pub outputs: Vec<WalletOutput>,
    pub fee: u64,
    pub total_input: u64,
    pub total_output: u64,
    pub confirmations: u64,
    pub direction: TxDirection,
    pub status: TxStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInput {
    pub txid: Txid,
    pub vout: u32,
    pub address: String,
    pub value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletOutput {
    pub address: String,
    pub value: u64,
    pub is_change: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TxDirection {
    Sent,
    Received,
    SelfTransfer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TxStatus {
    Pending,
    Confirmed,
    Failed,
}

/// Wallet account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccount {
    pub name: String,
    pub index: u32,
    pub derivation_path: DerivationPath,
    pub external_keys: Vec<String>,
    pub internal_keys: Vec<String>,
    pub next_external_index: u32,
    pub next_internal_index: u32,
}

/// Full wallet state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletState {
    pub name: String,
    pub version: String,
    pub network: String,
    pub encrypted: bool,
    pub accounts: Vec<WalletAccount>,
    pub active_account: u32,
    pub transactions: Vec<WalletTx>,
    pub utxos: Vec<WalletUTXO>,
    pub created_at: u64,
    pub last_updated: u64,
}

/// Wallet engine
pub struct Wallet {
    state: Arc<RwLock<WalletState>>,
    _encrypted_master_key: Option<Vec<u8>>,
}

impl Wallet {
    /// Create a new wallet
    pub fn new(name: &str, network: &str) -> Self {
        let state = WalletState {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            network: network.to_string(),
            encrypted: false,
            accounts: vec![WalletAccount {
                name: "Default".to_string(),
                index: 0,
                derivation_path: DerivationPath::BIP84,
                external_keys: Vec::new(),
                internal_keys: Vec::new(),
                next_external_index: 0,
                next_internal_index: 0,
            }],
            active_account: 0,
            transactions: Vec::new(),
            utxos: Vec::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_updated: 0,
        };

        Self {
            state: Arc::new(RwLock::new(state)),
            _encrypted_master_key: None,
        }
    }

    /// Get wallet balance
    pub fn get_balance(&self) -> WalletBalance {
        let state = self.state.read();
        let mut confirmed = 0u64;
        let mut unconfirmed = 0u64;
        let mut immature = 0u64;

        for utxo in &state.utxos {
            if utxo.is_spent {
                continue;
            }
            if utxo.confirmations >= 6 {
                confirmed += utxo.value;
            } else if utxo.confirmations >= 1 {
                unconfirmed += utxo.value;
            } else {
                immature += utxo.value;
            }
        }

        WalletBalance {
            confirmed: confirmed as f64 / SATS_PER_COIN as f64,
            unconfirmed: unconfirmed as f64 / SATS_PER_COIN as f64,
            immature: immature as f64 / SATS_PER_COIN as f64,
            total: (confirmed + unconfirmed + immature) as f64 / SATS_PER_COIN as f64,
            satoshi_confirmed: confirmed,
            satoshi_total: confirmed + unconfirmed + immature,
        }
    }

    /// Get all UTXOs
    pub fn get_utxos(&self) -> Vec<WalletUTXO> {
        let state = self.state.read();
        state
            .utxos
            .iter()
            .filter(|u| !u.is_spent)
            .cloned()
            .collect()
    }

    /// Get transaction history
    pub fn get_transactions(&self, count: usize, skip: usize) -> Vec<WalletTx> {
        let state = self.state.read();
        let mut txs: Vec<WalletTx> = state.transactions.clone();
        txs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        txs.into_iter().skip(skip).take(count).collect()
    }

    /// Add a UTXO to the wallet
    pub fn add_utxo(&self, utxo: WalletUTXO) {
        let mut state = self.state.write();
        state.utxos.push(utxo);
        state.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Mark UTXO as spent
    pub fn mark_utxo_spent(&self, txid: &Txid, vout: u32) {
        let mut state = self.state.write();
        if let Some(utxo) = state
            .utxos
            .iter_mut()
            .find(|u| u.txid == *txid && u.vout == vout)
        {
            utxo.is_spent = true;
        }
    }

    /// Generate a new wallet seed and derive first address (BIP-84 Native SegWit)
    pub fn generate_seed(&self) -> (Vec<String>, String) {
        use crypto::{entropy_to_mnemonic, EntropySource, ExtendedKey};

        let entropy = EntropySource::generate();
        let mnemonic = entropy_to_mnemonic(&entropy.entropy);
        let seed = crypto::mnemonic_to_seed(&mnemonic, "");
        let master = ExtendedKey::from_seed(&seed);

        // Derive BIP84 first address using Udaya coin type (257')
        let path = derive_bip84_path();
        let key = master.derive_from_path(&path);
        let address = key.to_native_segwit_address(MAINNET_HRP);

        let mut state = self.state.write();
        state.accounts[0].external_keys.push(address.clone());

        (mnemonic, address)
    }

    /// Recover wallet from mnemonic (BIP-84 Native SegWit)
    pub fn recover_from_mnemonic(
        &self,
        mnemonic_words: &[String],
        passphrase: &str,
    ) -> anyhow::Result<String> {
        use crypto::{mnemonic_to_seed, ExtendedKey};

        let seed = mnemonic_to_seed(mnemonic_words, passphrase);
        let master = ExtendedKey::from_seed(&seed);

        // Derive BIP84 first address using Udaya coin type (257')
        let path = derive_bip84_path();
        let key = master.derive_from_path(&path);
        let address = key.to_native_segwit_address(MAINNET_HRP);

        let mut state = self.state.write();
        state.accounts[0].external_keys.push(address.clone());

        Ok(address)
    }

    /// Recover wallet from mnemonic using a specific derivation path
    pub fn recover_from_mnemonic_with_path(
        &self,
        mnemonic_words: &[String],
        passphrase: &str,
        derivation: DerivationPath,
    ) -> anyhow::Result<String> {
        use crypto::{mnemonic_to_seed, ExtendedKey};

        let seed = mnemonic_to_seed(mnemonic_words, passphrase);
        let master = ExtendedKey::from_seed(&seed);

        let path = match derivation {
            DerivationPath::BIP44 => derive_bip44_path(),
            DerivationPath::BIP49 => derive_bip49_path(),
            DerivationPath::BIP84 => derive_bip84_path(),
            DerivationPath::BIP86 => derive_bip86_path(),
            DerivationPath::Custom(_) => derive_bip84_path(), // default
        };

        let key = master.derive_from_path(&path);
        let address = match derivation {
            DerivationPath::BIP44 => key.to_p2pkh_address(),
            DerivationPath::BIP49 => key.to_p2sh_segwit_address(),
            DerivationPath::BIP84 => key.to_native_segwit_address(MAINNET_HRP),
            DerivationPath::BIP86 => key.to_taproot_address(MAINNET_HRP),
            DerivationPath::Custom(_) => key.to_native_segwit_address(MAINNET_HRP),
        };

        let mut state = self.state.write();
        state.accounts[0].external_keys.push(address.clone());

        Ok(address)
    }

    /// Generate a Native SegWit (BIP-84 bech32) address
    pub fn generate_address(&self) -> String {
        use crypto::{entropy_to_mnemonic, EntropySource, ExtendedKey};
        let entropy = EntropySource::generate();
        let mnemonic = entropy_to_mnemonic(&entropy.entropy);
        let seed = crypto::mnemonic_to_seed(&mnemonic, "");
        let master = ExtendedKey::from_seed(&seed);
        let path = derive_bip84_path();
        let key = master.derive_from_path(&path);
        key.to_native_segwit_address(MAINNET_HRP)
    }

    /// Generate a Taproot (BIP-86) address
    pub fn generate_taproot_address(&self) -> String {
        use crypto::{entropy_to_mnemonic, EntropySource, ExtendedKey};
        let entropy = EntropySource::generate();
        let mnemonic = entropy_to_mnemonic(&entropy.entropy);
        let seed = crypto::mnemonic_to_seed(&mnemonic, "");
        let master = ExtendedKey::from_seed(&seed);
        let path = derive_bip86_path();
        let key = master.derive_from_path(&path);
        key.to_taproot_address(MAINNET_HRP)
    }

    /// Create a simple payment transaction
    pub fn create_payment(
        &self,
        to_script: &[u8],
        amount_sats: u64,
        fee_sats: u64,
    ) -> anyhow::Result<Transaction> {
        let utxos = self.get_utxos();
        let target = amount_sats + fee_sats;

        let mut selected_value = 0u64;
        let mut selected_utxos: Vec<(OutPoint, TxOut)> = Vec::new();

        for utxo in &utxos {
            let outpoint = OutPoint::new(utxo.txid, utxo.vout);
            let txout = TxOut::new(utxo.value, ScriptPubKey::new(utxo.script_pubkey.clone()));
            selected_utxos.push((outpoint, txout));
            selected_value += utxo.value;
            if selected_value >= target {
                break;
            }
        }

        if selected_value < target {
            anyhow::bail!(
                "Insufficient funds: have {} sats, need {} sats",
                selected_value,
                target
            );
        }

        // Generate signing key using BIP-84 with Udaya coin type
        use crypto::{entropy_to_mnemonic, mnemonic_to_seed, EntropySource, ExtendedKey};
        let entropy = EntropySource::generate();
        let mnemonic = entropy_to_mnemonic(&entropy.entropy);
        let seed = mnemonic_to_seed(&mnemonic, "");
        let master = ExtendedKey::from_seed(&seed);
        let path = derive_bip84_path();
        let signing_key = master.derive_from_path(&path);

        let secret_key = signing_key.to_secret_key()?;
        let secp = secp256k1::Secp256k1::new();

        let tx = create_p2pkh_transaction(
            selected_utxos,
            to_script.to_vec(),
            vec![], // change script (simplified)
            amount_sats,
            fee_sats,
            &secret_key,
            &secp,
        )?;

        Ok(tx)
    }

    /// Export private key as WIF
    pub fn export_wif(&self) -> anyhow::Result<String> {
        use crypto::ExtendedKey;
        // Generate a deterministic key for the current wallet state
        let entropy = [0u8; 16]; // Placeholder - real wallet would use stored seed
        let seed = crypto::mnemonic_to_seed(&crypto::entropy_to_mnemonic(&entropy), "");
        let master = ExtendedKey::from_seed(&seed);
        Ok(master.to_wif(true))
    }

    /// Get wallet state for serialization
    pub fn export_state(&self) -> WalletState {
        self.state.read().clone()
    }

    /// Import wallet state
    pub fn import_state(&self, state: WalletState) {
        let mut s = self.state.write();
        *s = state;
    }
}

/// Wallet balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub confirmed: f64,
    pub unconfirmed: f64,
    pub immature: f64,
    pub total: f64,
    pub satoshi_confirmed: u64,
    pub satoshi_total: u64,
}

impl Default for WalletBalance {
    fn default() -> Self {
        Self {
            confirmed: 0.0,
            unconfirmed: 0.0,
            immature: 0.0,
            total: 0.0,
            satoshi_confirmed: 0,
            satoshi_total: 0,
        }
    }
}

/// Wallet backup format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBackup {
    pub format_version: String,
    pub network: String,
    pub encrypted_master_key: Vec<u8>,
    pub wallet_data: Vec<u8>,
    pub checksum: [u8; 32],
    pub created_at: u64,
}

/// QR Payment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub address: String,
    pub amount: Option<u64>,
    pub label: Option<String>,
    pub message: Option<String>,
}

impl PaymentRequest {
    pub fn to_uri(&self) -> String {
        let mut uri = format!("Udaya:{}", self.address);
        let mut params = Vec::new();

        if let Some(amount) = self.amount {
            params.push(format!("amount={}", amount as f64 / SATS_PER_COIN as f64));
        }
        if let Some(label) = &self.label {
            let encoded: String = label
                .chars()
                .map(|c| match c {
                    ' ' => "%20".to_string(),
                    '%' => "%25".to_string(),
                    '&' => "%26".to_string(),
                    '=' => "%3D".to_string(),
                    '?' => "%3F".to_string(),
                    '#' => "%23".to_string(),
                    _ => c.to_string(),
                })
                .collect();
            params.push(format!("label={}", encoded));
        }
        if let Some(message) = &self.message {
            params.push(format!("message={}", message));
        }

        if !params.is_empty() {
            uri.push('?');
            uri.push_str(&params.join("&"));
        }

        uri
    }

    /// Simple URL percent-decoding
    fn url_decode(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                } else {
                    result.push('%');
                    result.push_str(&hex);
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    pub fn from_uri(uri: &str) -> anyhow::Result<Self> {
        let stripped = uri
            .strip_prefix("Udaya:")
            .ok_or_else(|| anyhow::anyhow!("Invalid UDYA URI"))?;

        let parts: Vec<&str> = stripped.split('?').collect();
        let address = parts[0].to_string();

        let mut amount = None;
        let mut label = None;
        let mut message = None;

        if parts.len() > 1 {
            for param in parts[1].split('&') {
                let kv: Vec<&str> = param.split('=').collect();
                if kv.len() == 2 {
                    match kv[0] {
                        "amount" => {
                            amount = Some((kv[1].parse::<f64>()? * SATS_PER_COIN as f64) as u64)
                        }
                        "label" => label = Some(Self::url_decode(kv[1])),
                        "message" => message = Some(Self::url_decode(kv[1])),
                        _ => {}
                    }
                }
            }
        }

        Ok(Self {
            address,
            amount,
            label,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new("Test Wallet", "testnet");
        let balance = wallet.get_balance();
        assert_eq!(balance.total, 0.0);
    }

    #[test]
    fn test_payment_request_uri() {
        let req = PaymentRequest {
            address: "tb1qtest123".to_string(),
            amount: Some(100_000_000), // 1 UDYA
            label: Some("Test Payment".to_string()),
            message: None,
        };

        let uri = req.to_uri();
        assert!(uri.starts_with("Udaya:"));
        assert!(uri.contains("amount=1"));
        assert!(uri.contains("label=Test%20Payment"));

        let parsed = PaymentRequest::from_uri(&uri).unwrap();
        assert_eq!(parsed.address, "tb1qtest123");
        assert_eq!(parsed.amount, Some(100_000_000));
    }

    #[test]
    fn test_wallet_balance_calculation() {
        let wallet = Wallet::new("Balance Test", "mainnet");

        wallet.add_utxo(WalletUTXO {
            txid: Txid([1u8; 32]),
            vout: 0,
            value: 50_000_000_000, // 500 UDYA
            address: "btf1test".to_string(),
            script_pubkey: vec![],
            height: 100,
            confirmations: 10,
            is_coinbase: false,
            is_spent: false,
            is_frozen: false,
        });

        let balance = wallet.get_balance();
        assert_eq!(balance.satoshi_confirmed, 50_000_000_000);
        assert_eq!(balance.total, 500.0);
    }

    #[test]
    fn test_generate_seed() {
        let wallet = Wallet::new("Seed Test", "mainnet");
        let (mnemonic, address) = wallet.generate_seed();
        assert_eq!(mnemonic.len(), 12, "Should generate 12-word mnemonic");
        assert!(
            address.starts_with("btf1"),
            "Address should start with btf1"
        );
    }

    #[test]
    fn test_generate_taproot_address() {
        let wallet = Wallet::new("Taproot Test", "mainnet");
        let addr = wallet.generate_taproot_address();
        assert!(
            addr.starts_with("btf1"),
            "Taproot address should start with btf1"
        );
        assert!(addr.len() > 40, "Taproot bech32m addresses are long");
    }
}
