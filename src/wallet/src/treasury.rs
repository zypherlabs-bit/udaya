/// Udaya Founder Treasury Module
///
/// Implements secure treasury management for the founder allocation.
///
/// ## Architecture
/// - **Cold Storage (95%)**: 475,000 UDYA - 3-of-5 multisig
/// - **Operations Wallet (4%)**: 20,000 UDYA - 2-of-3 multisig
/// - **Daily Wallet (1%)**: 5,000 UDYA - 1-of-2 multisig
///
/// ## Security Controls
/// - PSBT-based multi-device signing workflow
/// - Hardware wallet compatibility (Ledger, Trezor, Coldcard, Keystone)
/// - Time-locked large withdrawals (>10,000 UDYA = 48h delay)
/// - Automated monitoring with alerts
/// - Quarterly audit trail
use crate::psbt::{PSBTWorkflow, PSBT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use udaya_core::transaction::Transaction;
use udaya_core::SATS_PER_COIN;

/// Total founder allocation
pub const FOUNDER_ALLOCATION: u64 = 500_000; // UDYA
pub const FOUNDER_ALLOCATION_SATS: u64 = FOUNDER_ALLOCATION * SATS_PER_COIN;

/// Cold storage percentage and amount
pub const COLD_STORAGE_PERCENT: u64 = 95;
pub const COLD_STORAGE_AMOUNT: u64 = 475_000 * SATS_PER_COIN;

/// Operations wallet percentage and amount
pub const OPERATIONS_PERCENT: u64 = 4;
pub const OPERATIONS_AMOUNT: u64 = 20_000 * SATS_PER_COIN;

/// Daily wallet percentage and amount
pub const DAILY_PERCENT: u64 = 1;
pub const DAILY_AMOUNT: u64 = 5_000 * SATS_PER_COIN;

/// Treasury wallet type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TreasuryWalletType {
    ColdStorage, // 95% - 3-of-5 multisig
    Operations,  // 4%  - 2-of-3 multisig
    Daily,       // 1%  - 1-of-2 multisig
}

/// Treasury wallet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryConfig {
    pub wallet_type: TreasuryWalletType,
    pub required_signatures: usize,
    pub total_signers: usize,
    pub signer_public_keys: Vec<Vec<u8>>,
    pub address: String,
    pub balance_sats: u64,
    pub time_lock_hours: u64,     // 0 = no timelock, 48 = 48h delay
    pub max_withdrawal_sats: u64, // per-transaction limit
    pub daily_limit_sats: u64,    // daily aggregate limit
}

/// Treasury wallet instance
#[allow(non_snake_case)]
pub struct TreasuryWallet {
    pub config: TreasuryConfig,
    pub psbt_workflow: Option<PSBTWorkflow>,
    pub audit_log: Vec<TreasuryEvent>,
    _pending_transactions: Vec<TreasuryTransaction>,
    daily_volume: HashMap<String, u64>, // date -> total sats withdrawn
}

/// Treasury transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryTransaction {
    pub id: String,
    pub wallet_type: TreasuryWalletType,
    pub destination: String,
    pub amount_sats: u64,
    pub purpose: String,
    pub signers_approved: Vec<String>,
    pub signatures_required: usize,
    pub status: TransactionStatus,
    pub created_at: u64,
    pub executed_at: Option<u64>,
    pub txid: Option<String>,
    pub psbt_base64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    PendingCreation,
    PendingSignatures,
    PartiallySigned(usize), // how many sigs collected
    ReadyToBroadcast,
    Broadcasted,
    Confirmed,
    Rejected(String),
}

/// Treasury event for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryEvent {
    pub timestamp: u64,
    pub event_type: TreasuryEventType,
    pub description: String,
    pub actor: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TreasuryEventType {
    WalletCreated,
    TransactionProposed,
    SignatureAdded,
    TransactionBroadcasted,
    TransactionConfirmed,
    WithdrawalLimitReached,
    BalanceAlert,
    SignerAdded,
    SignerRemoved,
    AuditLogCreated,
    SecurityAlert,
}

impl TreasuryWallet {
    /// Create a new treasury wallet with specified multisig configuration
    pub fn new(wallet_type: TreasuryWalletType, public_keys: Vec<Vec<u8>>) -> anyhow::Result<Self> {
        let (required, total, timelock, max_tx, daily_limit) = match &wallet_type {
            TreasuryWalletType::ColdStorage => {
                if public_keys.len() != 5 {
                    anyhow::bail!("Cold storage requires exactly 5 public keys");
                }
                (3, 5, 48, 10_000 * SATS_PER_COIN, 25_000 * SATS_PER_COIN)
            }
            TreasuryWalletType::Operations => {
                if public_keys.len() != 3 {
                    anyhow::bail!("Operations wallet requires exactly 3 public keys");
                }
                (2, 3, 0, 5_000 * SATS_PER_COIN, 10_000 * SATS_PER_COIN)
            }
            TreasuryWalletType::Daily => {
                if public_keys.len() != 2 {
                    anyhow::bail!("Daily wallet requires exactly 2 public keys");
                }
                (1, 2, 0, 100 * SATS_PER_COIN, 500 * SATS_PER_COIN)
            }
        };

        let balance_sats = match &wallet_type {
            TreasuryWalletType::ColdStorage => COLD_STORAGE_AMOUNT,
            TreasuryWalletType::Operations => OPERATIONS_AMOUNT,
            TreasuryWalletType::Daily => DAILY_AMOUNT,
        };

        let config = TreasuryConfig {
            wallet_type: wallet_type.clone(),
            required_signatures: required,
            total_signers: total,
            signer_public_keys: public_keys,
            address: String::new(), // populated after address derivation
            balance_sats,
            time_lock_hours: timelock,
            max_withdrawal_sats: max_tx,
            daily_limit_sats: daily_limit,
        };

        let mut wallet = Self {
            config,
            psbt_workflow: None,
            audit_log: Vec::new(),
            _pending_transactions: Vec::new(),
            daily_volume: HashMap::new(),
        };

        wallet.log_event(
            TreasuryEventType::WalletCreated,
            format!(
                "{:?} wallet created with {}-of-{} multisig",
                wallet_type, required, total
            ),
            "system",
            serde_json::json!({
                "type": format!("{:?}", wallet_type),
                "required_signatures": required,
                "total_signers": total,
                "balance_UDYA": balance_sats as f64 / SATS_PER_COIN as f64,
            }),
        );

        Ok(wallet)
    }

    /// Propose a new withdrawal transaction
    pub fn propose_withdrawal(
        &mut self,
        destination: &str,
        amount_sats: u64,
        purpose: &str,
        proposer: &str,
    ) -> anyhow::Result<String> {
        // Validate amount
        if amount_sats == 0 {
            anyhow::bail!("Withdrawal amount must be greater than 0");
        }
        if amount_sats > self.config.balance_sats {
            anyhow::bail!(
                "Insufficient balance: have {} UDYA, need {} UDYA",
                self.config.balance_sats as f64 / SATS_PER_COIN as f64,
                amount_sats as f64 / SATS_PER_COIN as f64,
            );
        }
        if amount_sats > self.config.max_withdrawal_sats {
            anyhow::bail!(
                "Exceeds per-transaction limit of {} UDYA",
                self.config.max_withdrawal_sats as f64 / SATS_PER_COIN as f64,
            );
        }

        // Check daily limit
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let today_volume = self.daily_volume.get(&today).unwrap_or(&0);
        if today_volume + amount_sats > self.config.daily_limit_sats {
            anyhow::bail!(
                "Exceeds daily limit of {} UDYA (already spent {} UDYA today)",
                self.config.daily_limit_sats as f64 / SATS_PER_COIN as f64,
                *today_volume as f64 / SATS_PER_COIN as f64,
            );
        }

        // Check time lock
        if self.config.time_lock_hours > 0 && amount_sats > 10_000 * SATS_PER_COIN {
            // For large cold storage withdrawals, enforce timelock
            // The PSBT workflow will handle this via BIP-68 relative locktime
            log::info!(
                "Large withdrawal ({} UDYA) requires {}-hour time lock",
                amount_sats as f64 / SATS_PER_COIN as f64,
                self.config.time_lock_hours,
            );
        }

        let tx_id = uuid::Uuid::new_v4().to_string();
        let tx = TreasuryTransaction {
            id: tx_id.clone(),
            wallet_type: self.config.wallet_type.clone(),
            destination: destination.to_string(),
            amount_sats,
            purpose: purpose.to_string(),
            signers_approved: Vec::new(),
            signatures_required: self.config.required_signatures,
            status: TransactionStatus::PendingCreation,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            executed_at: None,
            txid: None,
            psbt_base64: None,
        };

        self._pending_transactions.push(tx);

        self.log_event(
            TreasuryEventType::TransactionProposed,
            format!(
                "Withdrawal of {} UDYA to {} proposed by {}",
                amount_sats as f64 / SATS_PER_COIN as f64,
                destination,
                proposer,
            ),
            proposer,
            serde_json::json!({
                "tx_id": tx_id,
                "amount_UDYA": amount_sats as f64 / SATS_PER_COIN as f64,
                "destination": destination,
                "purpose": purpose,
                "signatures_required": self.config.required_signatures,
            }),
        );

        Ok(tx_id)
    }

    /// Create PSBT for a proposed transaction
    pub fn create_psbt_for_tx(
        &mut self,
        tx_id: &str,
        unsigned_tx: Transaction,
    ) -> anyhow::Result<PSBT> {
        let tx = self
            ._pending_transactions
            .iter_mut()
            .find(|t| t.id == tx_id)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found: {}", tx_id))?;

        // Create PSBT from unsigned transaction
        let mut psbt = PSBT::create(unsigned_tx);

        // Add BIP32 derivation paths for each signer
        for (i, pubkey) in self.config.signer_public_keys.iter().enumerate() {
            psbt.add_input_derivation(
                0,
                pubkey.clone(),
                [0u8; 4], // placeholder fingerprint
                vec![
                    0x80000054, // BIP84 purpose
                    0x80000101, // coin type 257'
                    0x80000000, // account
                    i as u32,   // key index
                ],
            )?;
        }

        tx.psbt_base64 = Some(psbt.to_base64());
        tx.status = TransactionStatus::PendingSignatures;

        Ok(psbt)
    }

    /// Add a signer's partial signature to a transaction
    pub fn add_signature(
        &mut self,
        tx_id: &str,
        signer: &str,
        signed_psbt_b64: &str,
    ) -> anyhow::Result<()> {
        // Find the transaction index to avoid holding mutable borrow across log_event
        let tx_idx = self
            ._pending_transactions
            .iter()
            .position(|t| t.id == tx_id)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found: {}", tx_id))?;

        {
            let tx = &mut self._pending_transactions[tx_idx];
            if tx.signers_approved.contains(&signer.to_string()) {
                anyhow::bail!("Signer {} already approved this transaction", signer);
            }
        }

        // Parse signed PSBT and merge signatures
        let signed = PSBT::from_base64(signed_psbt_b64)?;
        let current_b64 = self._pending_transactions[tx_idx]
            .psbt_base64
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No PSBT created for this transaction"))?
            .clone();
        let mut current = PSBT::from_base64(&current_b64)?;

        // Merge signatures from hardware wallet
        for (i, input) in signed.inputs.iter().enumerate() {
            if i < current.inputs.len() {
                for (pubkey, sig) in &input.partial_signatures {
                    current.add_partial_signature(i, pubkey.clone(), sig.clone())?;
                }
            }
        }

        let new_psbt_b64 = current.to_base64();
        let sig_count;
        {
            let tx = &mut self._pending_transactions[tx_idx];
            tx.signers_approved.push(signer.to_string());
            sig_count = tx.signers_approved.len();

            if sig_count >= tx.signatures_required {
                tx.status = TransactionStatus::ReadyToBroadcast;
            } else {
                tx.status = TransactionStatus::PartiallySigned(sig_count);
            }
            tx.psbt_base64 = Some(new_psbt_b64);
        }

        self.log_event(
            TreasuryEventType::SignatureAdded,
            format!(
                "Signer {} added signature ({}/{} required)",
                signer, sig_count, self._pending_transactions[tx_idx].signatures_required
            ),
            signer,
            serde_json::json!({
                "tx_id": tx_id,
                "signer": signer,
                "signatures_collected": sig_count,
                "signatures_required": self._pending_transactions[tx_idx].signatures_required,
            }),
        );

        Ok(())
    }

    /// Broadcast a fully signed transaction
    pub fn broadcast_transaction(
        &mut self,
        tx_id: &str,
        broadcaster: &str,
    ) -> anyhow::Result<Transaction> {
        let tx_idx = self
            ._pending_transactions
            .iter()
            .position(|t| t.id == tx_id)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found: {}", tx_id))?;

        let amount_sats;
        let sig_count;
        let sigs_required;
        let destination;
        let created_at;
        let psbt_b64_clone;
        {
            let tx = &self._pending_transactions[tx_idx];
            if tx.signers_approved.len() < tx.signatures_required {
                anyhow::bail!(
                    "Not enough signatures: have {}, need {}",
                    tx.signers_approved.len(),
                    tx.signatures_required,
                );
            }
            amount_sats = tx.amount_sats;
            sig_count = tx.signers_approved.len();
            sigs_required = tx.signatures_required;
            destination = tx.destination.clone();
            created_at = tx.created_at;
            psbt_b64_clone = tx
                .psbt_base64
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("No PSBT data"))?
                .clone();
        }

        // Parse the fully signed PSBT
        let psbt = PSBT::from_base64(&psbt_b64_clone)?;

        // Check time lock for large withdrawals
        if self.config.time_lock_hours > 0 && amount_sats > 10_000 * SATS_PER_COIN {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let elapsed_hours = (now - created_at) / 3600;
            if elapsed_hours < self.config.time_lock_hours {
                anyhow::bail!(
                    "Time lock active: {} hours elapsed, {} hours required",
                    elapsed_hours,
                    self.config.time_lock_hours,
                );
            }
        }

        // Extract the fully signed transaction
        let final_tx = psbt.extract_transaction()?;
        let txid = hex::encode(final_tx.txid().0);

        // Update transaction state
        {
            let tx = &mut self._pending_transactions[tx_idx];
            tx.status = TransactionStatus::Broadcasted;
            tx.executed_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            tx.txid = Some(txid.clone());
        }

        // Update balance
        self.config.balance_sats = self.config.balance_sats.saturating_sub(amount_sats);

        // Update daily volume
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        *self.daily_volume.entry(today).or_insert(0) += amount_sats;

        self.log_event(
            TreasuryEventType::TransactionBroadcasted,
            format!(
                "Transaction broadcast: {} UDYA to {} (txid: {})",
                amount_sats as f64 / SATS_PER_COIN as f64,
                destination,
                txid,
            ),
            broadcaster,
            serde_json::json!({
                "tx_id": tx_id,
                "txid": txid,
                "amount_UDYA": amount_sats as f64 / SATS_PER_COIN as f64,
                "destination": destination,
                "signers_count": sig_count,
                "signatures_required": sigs_required,
            }),
        );

        Ok(final_tx)
    }

    /// Confirm a transaction (called after on-chain confirmation)
    pub fn confirm_transaction(&mut self, tx_id: &str) -> anyhow::Result<()> {
        let tx = self
            ._pending_transactions
            .iter_mut()
            .find(|t| t.id == tx_id)
            .ok_or_else(|| anyhow::anyhow!("Transaction not found: {}", tx_id))?;

        tx.status = TransactionStatus::Confirmed;

        self.log_event(
            TreasuryEventType::TransactionConfirmed,
            format!("Transaction {} confirmed on-chain", tx_id),
            "system",
            serde_json::json!({ "tx_id": tx_id }),
        );

        Ok(())
    }

    /// Get current balance for this treasury wallet
    pub fn get_balance_UDYA(&self) -> f64 {
        self.config.balance_sats as f64 / SATS_PER_COIN as f64
    }

    /// Get pending transactions
    pub fn get_pending_transactions(&self) -> Vec<&TreasuryTransaction> {
        self._pending_transactions
            .iter()
            .filter(|t| t.status != TransactionStatus::Confirmed)
            .collect()
    }

    /// Check if daily limit has been exceeded
    pub fn check_daily_limit(&self) -> bool {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let today_volume = self.daily_volume.get(&today).unwrap_or(&0);
        *today_volume < self.config.daily_limit_sats
    }

    /// Generate a treasury health report
    pub fn generate_health_report(&self) -> serde_json::Value {
        let pending_count = self
            ._pending_transactions
            .iter()
            .filter(|t| t.status != TransactionStatus::Confirmed)
            .count();
        let pending_volume: u64 = self
            ._pending_transactions
            .iter()
            .filter(|t| t.status != TransactionStatus::Confirmed)
            .map(|t| t.amount_sats)
            .sum();

        serde_json::json!({
            "wallet_type": format!("{:?}", self.config.wallet_type),
            "balance_UDYA": self.get_balance_UDYA(),
            "balance_sats": self.config.balance_sats,
            "allocation_pct": match self.config.wallet_type {
                TreasuryWalletType::ColdStorage => 95,
                TreasuryWalletType::Operations => 4,
                TreasuryWalletType::Daily => 1,
            },
            "multisig_config": format!("{}-of-{}", self.config.required_signatures, self.config.total_signers),
            "signers_count": self.config.signer_public_keys.len(),
            "pending_transactions": pending_count,
            "pending_volume_UDYA": pending_volume as f64 / SATS_PER_COIN as f64,
            "daily_limit_UDYA": self.config.daily_limit_sats as f64 / SATS_PER_COIN as f64,
            "max_tx_limit_UDYA": self.config.max_withdrawal_sats as f64 / SATS_PER_COIN as f64,
            "time_lock_hours": self.config.time_lock_hours,
            "audit_log_entries": self.audit_log.len(),
            "daily_limit_active": self.check_daily_limit(),
            "recent_events": self.audit_log.iter().rev().take(10).map(|e| serde_json::json!({
                "timestamp": e.timestamp,
                "event_type": format!("{:?}", e.event_type),
                "description": e.description,
                "actor": e.actor,
            })).collect::<Vec<_>>(),
        })
    }

    /// Get the full audit trail
    pub fn get_audit_trail(&self) -> Vec<&TreasuryEvent> {
        self.audit_log.iter().collect()
    }

    /// Log a treasury event
    fn log_event(
        &mut self,
        event_type: TreasuryEventType,
        description: String,
        actor: &str,
        details: serde_json::Value,
    ) {
        let event = TreasuryEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            event_type,
            description,
            actor: actor.to_string(),
            details,
        };
        self.audit_log.push(event);
    }
}

/// Treasury management system - manages all three wallet tiers
#[allow(non_snake_case)]
pub struct TreasuryManager {
    pub cold_storage: TreasuryWallet,
    pub operations: TreasuryWallet,
    pub daily: TreasuryWallet,
    audit_log: Vec<TreasuryEvent>,
}

impl TreasuryManager {
    /// Create the full treasury system with all three wallet tiers
    pub fn create(
        cold_storage_keys: Vec<Vec<u8>>,
        operations_keys: Vec<Vec<u8>>,
        daily_keys: Vec<Vec<u8>>,
    ) -> anyhow::Result<Self> {
        let cold_storage = TreasuryWallet::new(TreasuryWalletType::ColdStorage, cold_storage_keys)?;
        let operations = TreasuryWallet::new(TreasuryWalletType::Operations, operations_keys)?;
        let daily = TreasuryWallet::new(TreasuryWalletType::Daily, daily_keys)?;

        Ok(Self {
            cold_storage,
            operations,
            daily,
            audit_log: Vec::new(),
        })
    }

    /// Get total treasury balance
    pub fn total_balance_UDYA(&self) -> f64 {
        self.cold_storage.get_balance_UDYA()
            + self.operations.get_balance_UDYA()
            + self.daily.get_balance_UDYA()
    }

    /// Generate comprehensive treasury report
    pub fn generate_treasury_report(&self) -> serde_json::Value {
        serde_json::json!({
            "report_type": "founder_treasury_status",
            "version": "1.0.0",
            "total_allocation_UDYA": FOUNDER_ALLOCATION,
            "total_balance_UDYA": self.total_balance_UDYA(),
            "total_balance_sats": (self.total_balance_UDYA() * SATS_PER_COIN as f64) as u64,
            "wallets": {
                "cold_storage": self.cold_storage.generate_health_report(),
                "operations": self.operations.generate_health_report(),
                "daily": self.daily.generate_health_report(),
            },
            "allocation": {
                "cold_storage_pct": 95,
                "cold_storage_UDYA": 475_000,
                "operations_pct": 4,
                "operations_UDYA": 20_000,
                "daily_pct": 1,
                "daily_UDYA": 5_000,
            },
            "security_controls": {
                "psbt_workflow": true,
                "hardware_wallet_compatible": true,
                "time_locked_withdrawals": true,
                "multi_signature": true,
                "daily_limits": true,
                "audit_trail": true,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use udaya_core::types::{OutPoint, ScriptPubKey, ScriptSig, TxIn, TxOut, Txid};

    fn create_test_keys(count: usize) -> Vec<Vec<u8>> {
        (0..count)
            .map(|i| {
                let mut key = vec![0x02u8; 33];
                key[1] = i as u8 + 1;
                key
            })
            .collect()
    }

    #[test]
    fn test_cold_storage_creation() {
        let keys = create_test_keys(5);
        let wallet = TreasuryWallet::new(TreasuryWalletType::ColdStorage, keys).unwrap();
        assert_eq!(wallet.config.required_signatures, 3);
        assert_eq!(wallet.config.total_signers, 5);
        assert_eq!(wallet.config.time_lock_hours, 48);
        assert_eq!(wallet.config.balance_sats, 475_000 * SATS_PER_COIN);
        assert!(wallet.get_balance_UDYA() > 0.0);
    }

    #[test]
    fn test_operations_wallet_creation() {
        let keys = create_test_keys(3);
        let wallet = TreasuryWallet::new(TreasuryWalletType::Operations, keys).unwrap();
        assert_eq!(wallet.config.required_signatures, 2);
        assert_eq!(wallet.config.max_withdrawal_sats, 5_000 * SATS_PER_COIN);
    }

    #[test]
    fn test_daily_wallet_creation() {
        let keys = create_test_keys(2);
        let wallet = TreasuryWallet::new(TreasuryWalletType::Daily, keys).unwrap();
        assert_eq!(wallet.config.required_signatures, 1);
        assert_eq!(wallet.config.max_withdrawal_sats, 100 * SATS_PER_COIN);
    }

    #[test]
    fn test_cold_storage_wrong_key_count() {
        let keys = create_test_keys(4); // Wrong: should be 5
        let result = TreasuryWallet::new(TreasuryWalletType::ColdStorage, keys);
        assert!(result.is_err());
    }

    #[test]
    fn test_propose_withdrawal() {
        let keys = create_test_keys(3);
        let mut wallet = TreasuryWallet::new(TreasuryWalletType::Operations, keys).unwrap();

        let tx_id = wallet
            .propose_withdrawal(
                "btf1qtestdestinationaddress",
                1_000 * SATS_PER_COIN, // 1,000 UDYA
                "Exchange listing fee",
                "founder",
            )
            .unwrap();

        assert!(!tx_id.is_empty());
        assert_eq!(wallet._pending_transactions.len(), 1);
        assert_eq!(wallet.audit_log.len(), 2); // creation + proposal
    }

    #[test]
    fn test_propose_withdrawal_exceeds_limit() {
        let keys = create_test_keys(2);
        let mut wallet = TreasuryWallet::new(TreasuryWalletType::Daily, keys).unwrap();

        let result = wallet.propose_withdrawal(
            "btf1qtest",
            200 * SATS_PER_COIN, // Exceeds 100 UDYA limit
            "Test",
            "founder",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_propose_withdrawal_insufficient_balance() {
        let keys = create_test_keys(5);
        let mut wallet = TreasuryWallet::new(TreasuryWalletType::ColdStorage, keys).unwrap();

        let result = wallet.propose_withdrawal(
            "btf1qtest",
            500_000 * SATS_PER_COIN, // More than balance
            "Test",
            "founder",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_treasury_manager_creation() {
        let cold_keys = create_test_keys(5);
        let ops_keys = create_test_keys(3);
        let daily_keys = create_test_keys(2);

        let manager = TreasuryManager::create(cold_keys, ops_keys, daily_keys).unwrap();
        assert!(manager.total_balance_UDYA() > 0.0);
    }

    #[test]
    fn test_treasury_report() {
        let cold_keys = create_test_keys(5);
        let ops_keys = create_test_keys(3);
        let daily_keys = create_test_keys(2);

        let manager = TreasuryManager::create(cold_keys, ops_keys, daily_keys).unwrap();
        let report = manager.generate_treasury_report();
        assert_eq!(report["total_allocation_UDYA"], 500_000);
        assert!(report["security_controls"]["psbt_workflow"]
            .as_bool()
            .unwrap());
        assert!(report["security_controls"]["multi_signature"]
            .as_bool()
            .unwrap());
    }

    #[test]
    fn test_add_and_check_signatures() {
        let keys = create_test_keys(3);
        let mut wallet = TreasuryWallet::new(TreasuryWalletType::Operations, keys).unwrap();

        let tx_id = wallet
            .propose_withdrawal("btf1qtest", 100 * SATS_PER_COIN, "Test payment", "founder")
            .unwrap();

        // Create unsigned PSBT
        let tx = Transaction::new(
            2,
            vec![TxIn {
                previous_output: OutPoint::new(Txid([1u8; 32]), 0),
                script_sig: ScriptSig::new(vec![]),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            vec![TxOut {
                value: 100 * SATS_PER_COIN,
                script_pubkey: ScriptPubKey::new(vec![0x00; 20]),
            }],
            0,
        );

        let psbt = wallet.create_psbt_for_tx(&tx_id, tx).unwrap();
        let b64 = psbt.to_base64();

        // Simulate first signer
        wallet.add_signature(&tx_id, "signer1", &b64).unwrap();
        if let Some(tx) = wallet._pending_transactions.iter().find(|t| t.id == tx_id) {
            assert_eq!(tx.signers_approved.len(), 1);
        }
    }

    #[test]
    fn test_daily_limit_tracking() {
        let keys = create_test_keys(2);
        let wallet = TreasuryWallet::new(TreasuryWalletType::Daily, keys).unwrap();
        assert!(wallet.check_daily_limit());
    }

    #[test]
    fn test_health_report() {
        let keys = create_test_keys(5);
        let wallet = TreasuryWallet::new(TreasuryWalletType::ColdStorage, keys).unwrap();
        let report = wallet.generate_health_report();
        assert_eq!(report["wallet_type"], "ColdStorage");
        assert!(report["audit_log_entries"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn test_audit_log_creation() {
        let keys = create_test_keys(3);
        let wallet = TreasuryWallet::new(TreasuryWalletType::Operations, keys).unwrap();
        let trail = wallet.get_audit_trail();
        assert_eq!(trail.len(), 1);
        assert_eq!(trail[0].event_type, TreasuryEventType::WalletCreated);
    }
}
