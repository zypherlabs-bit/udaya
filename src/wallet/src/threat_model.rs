/// Udaya Formal Threat Model
///
/// Machine-readable threat model for CI/CD integration.
/// Covers 20 threats across 8 components with mitigations and verification status.
use serde::{Deserialize, Serialize};

/// Threat severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

/// Threat likelihood
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatLikelihood {
    VeryLikely,
    Likely,
    Possible,
    Unlikely,
    Rare,
}

/// Threat status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreatStatus {
    Mitigated,
    PartiallyMitigated,
    NotMitigated,
    NotApplicable,
}

/// A single threat entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: ThreatSeverity,
    pub likelihood: ThreatLikelihood,
    pub status: ThreatStatus,
    pub component: String,
    pub attack_vector: String,
    pub mitigations: Vec<String>,
    pub mitre_attack_id: Option<String>,
    pub verification_test: Option<String>,
}

/// Full threat model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatModel {
    pub version: String,
    pub date: String,
    pub system: String,
    pub threats: Vec<ThreatEntry>,
}

impl Default for ThreatModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreatModel {
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            date: "2026-06-11".to_string(),
            system: "Udaya Mainnet v1.0.0".to_string(),
            threats: generate_threats(),
        }
    }

    pub fn by_severity(&self, severity: ThreatSeverity) -> Vec<&ThreatEntry> {
        self.threats
            .iter()
            .filter(|t| t.severity == severity)
            .collect()
    }

    pub fn by_component(&self, component: &str) -> Vec<&ThreatEntry> {
        self.threats
            .iter()
            .filter(|t| t.component == component)
            .collect()
    }

    pub fn unmitigated(&self) -> Vec<&ThreatEntry> {
        self.threats
            .iter()
            .filter(|t| t.status == ThreatStatus::NotMitigated)
            .collect()
    }

    pub fn total(&self) -> usize {
        self.threats.len()
    }

    pub fn mitigated_count(&self) -> usize {
        self.threats
            .iter()
            .filter(|t| t.status == ThreatStatus::Mitigated)
            .count()
    }

    pub fn mitigation_pct(&self) -> f64 {
        if self.threats.is_empty() {
            return 100.0;
        }
        (self.mitigated_count() as f64 / self.threats.len() as f64) * 100.0
    }
}

fn generate_threats() -> Vec<ThreatEntry> {
    vec![
        ThreatEntry {
            id: "CONS-001".into(), name: "51% Attack".into(),
            description: "Mining entity controls >50% of network hashrate, enabling chain reorganization and double-spends.".into(),
            severity: ThreatSeverity::Critical, likelihood: ThreatLikelihood::Possible, status: ThreatStatus::PartiallyMitigated,
            component: "consensus".into(), attack_vector: "Mining centralization".into(),
            mitigations: vec![
                "6+ confirmations required for high-value transactions".into(),
                "Mining pool diversification encouraged (<30% per pool)".into(),
                "Checkpoint system for chain finality".into(),
            ],
            mitre_attack_id: Some("T1529".into()), verification_test: None,
        },
        ThreatEntry {
            id: "CONS-002".into(), name: "Double Spend".into(),
            description: "Attacker broadcasts conflicting transactions to reverse payments.".into(),
            severity: ThreatSeverity::High, likelihood: ThreatLikelihood::Unlikely, status: ThreatStatus::Mitigated,
            component: "consensus".into(), attack_vector: "Race condition in mempool propagation".into(),
            mitigations: vec![
                "UTXO model prevents inherent double-spend".into(),
                "Mempool validation rejects conflicting spends".into(),
                "First-seen rule enforces consistency".into(),
                "Merchant confirmation recommendations (6+ confirmations)".into(),
            ],
            mitre_attack_id: None, verification_test: None,
        },
        ThreatEntry {
            id: "CONS-003".into(), name: "Difficulty Adjustment Attack".into(),
            description: "Manipulating timestamps to cause incorrect difficulty adjustment.".into(),
            severity: ThreatSeverity::High, likelihood: ThreatLikelihood::Unlikely, status: ThreatStatus::Mitigated,
            component: "consensus".into(), attack_vector: "Timestamp manipulation".into(),
            mitigations: vec!["Median time past (MTP) enforcement".into(), "Strict timestamp validation rules".into(), "Difficulty adjustment every 2016 blocks".into()],
            mitre_attack_id: None, verification_test: None,
        },
        ThreatEntry {
            id: "NET-001".into(), name: "Sybil Attack".into(),
            description: "Attacker creates multiple fake peers to isolate or influence a node.".into(),
            severity: ThreatSeverity::Medium, likelihood: ThreatLikelihood::Possible, status: ThreatStatus::Mitigated,
            component: "networking".into(), attack_vector: "Creating fake node identities".into(),
            mitigations: vec!["DNS seed diversity (3 geographic regions)".into(), "Random peer selection algorithm".into(), "Maximum peer connection limits".into(), "Ban score system for misbehaving peers".into()],
            mitre_attack_id: Some("T1483".into()), verification_test: None,
        },
        ThreatEntry {
            id: "NET-002".into(), name: "Eclipse Attack".into(),
            description: "Attacker monopolizes all connections to/from a target node.".into(),
            severity: ThreatSeverity::Medium, likelihood: ThreatLikelihood::Possible, status: ThreatStatus::Mitigated,
            component: "networking".into(), attack_vector: "Connection table exhaustion".into(),
            mitigations: vec!["Multiple seed nodes (3+ geographic regions)".into(), "Diverse peer connection sources (DNS + PEX + manual)".into(), "Peer rotation for fresh connections".into(), "Connection limits per IP address".into()],
            mitre_attack_id: Some("T1483".into()), verification_test: None,
        },
        ThreatEntry {
            id: "NET-003".into(), name: "DNS Spoofing".into(),
            description: "Attacker intercepts DNS seed queries and returns malicious peer addresses.".into(),
            severity: ThreatSeverity::Medium, likelihood: ThreatLikelihood::Unlikely, status: ThreatStatus::Mitigated,
            component: "networking".into(), attack_vector: "DNS cache poisoning / man-in-the-middle".into(),
            mitigations: vec!["Multiple DNS seed domains across jurisdictions".into(), "Hardcoded fallback seed nodes".into(), "Peer address validation on connection".into()],
            mitre_attack_id: Some("T1558".into()), verification_test: None,
        },
        ThreatEntry {
            id: "NET-004".into(), name: "DoS - Message Flood".into(),
            description: "Overwhelming a node with network messages to cause resource exhaustion.".into(),
            severity: ThreatSeverity::Medium, likelihood: ThreatLikelihood::Likely, status: ThreatStatus::Mitigated,
            component: "networking".into(), attack_vector: "High-volume message injection".into(),
            mitigations: vec!["Message rate limiting".into(), "Maximum payload size enforcement (32MB)".into(), "Resource limits on message processing".into(), "Ban threshold for excessive traffic".into()],
            mitre_attack_id: Some("T1499".into()), verification_test: None,
        },
        ThreatEntry {
            id: "NET-005".into(), name: "Replay Attack".into(),
            description: "Transactions from other chains replayed on Udaya network.".into(),
            severity: ThreatSeverity::Low, likelihood: ThreatLikelihood::Unlikely, status: ThreatStatus::Mitigated,
            component: "networking".into(), attack_vector: "Broadcasting fork transactions".into(),
            mitigations: vec!["Unique network magic bytes (0xBF591AE7)".into(), "Udaya-specific coin type (257')".into(), "Distinct address format (btf1 prefix)".into()],
            mitre_attack_id: None, verification_test: None,
        },
        ThreatEntry {
            id: "WAL-001".into(), name: "Private Key Theft".into(),
            description: "Attacker gains access to private keys and steals funds.".into(),
            severity: ThreatSeverity::Critical, likelihood: ThreatLikelihood::Likely, status: ThreatStatus::Mitigated,
            component: "wallet".into(), attack_vector: "Malware, phishing, memory scraping".into(),
            mitigations: vec!["Hardware wallet support (Ledger, Trezor, Coldcard, Keystone)".into(), "PSBT multi-device signing workflow".into(), "Memory zeroization after key use".into(), "BIP39 passphrase support".into(), "Encrypted key storage on disk".into()],
            mitre_attack_id: Some("T1555".into()), verification_test: Some("tests/security/wallet.rs".into()),
        },
        ThreatEntry {
            id: "WAL-002".into(), name: "Mnemonic Phrase Exposure".into(),
            description: "12/24-word seed phrase leaked via backup, screenshot, or cloud sync.".into(),
            severity: ThreatSeverity::Critical, likelihood: ThreatLikelihood::Likely, status: ThreatStatus::Mitigated,
            component: "wallet".into(), attack_vector: "Backup compromise, cloud sync, social engineering".into(),
            mitigations: vec!["BIP39 passphrase adds last-word protection".into(), "User education on secure backup practices".into(), "No cloud sync for seed phrases".into(), "Encrypted wallet backups".into()],
            mitre_attack_id: None, verification_test: None,
        },
        ThreatEntry {
            id: "WAL-003".into(), name: "Weak Entropy Generation".into(),
            description: "Predictable random number generation leads to compromised keys.".into(),
            severity: ThreatSeverity::Critical, likelihood: ThreatLikelihood::Rare, status: ThreatStatus::Mitigated,
            component: "wallet".into(), attack_vector: "RNG weakness / OS entropy exhaustion".into(),
            mitigations: vec!["Cryptographically secure random number generator (OsRng)".into(), "128+ bits of entropy for key generation".into(), "rand crate with system entropy source".into()],
            mitre_attack_id: None, verification_test: Some("src/wallet/src/crypto.rs".into()),
        },
        ThreatEntry {
            id: "WAL-004".into(), name: "Memory Scraping of Keys".into(),
            description: "Attacker reads process memory to extract private keys.".into(),
            severity: ThreatSeverity::Medium, likelihood: ThreatLikelihood::Possible, status: ThreatStatus::Mitigated,
            component: "wallet".into(), attack_vector: "Process memory dump, cold boot attack".into(),
            mitigations: vec!["Zeroize crate integration for secure memory erasure".into(), "SensitiveData/SensitiveString wrappers auto-zeroize on Drop".into(), "ExtendedKey implements Drop for private key zeroization".into(), "Stack allocation for temporary key material".into()],
            mitre_attack_id: Some("T1056".into()), verification_test: Some("src/wallet/src/zeroize.rs".into()),
        },
        ThreatEntry {
            id: "WAL-005".into(), name: "Rogue Hardware Wallet".into(),
            description: "Malicious hardware wallet exfiltrates private keys during signing.".into(),
            severity: ThreatSeverity::High, likelihood: ThreatLikelihood::Rare, status: ThreatStatus::Mitigated,
            component: "wallet".into(), attack_vector: "Compromised supply chain / fake device".into(),
            mitigations: vec!["PSBT allows transaction verification before signing".into(), "Address verification on device screen".into(), "Purchase from official sources only".into()],
            mitre_attack_id: None, verification_test: None,
        },
        ThreatEntry {
            id: "STO-001".into(), name: "Database Corruption".into(),
            description: "UTXO set or block database becomes corrupted leading to inconsistent state.".into(),
            severity: ThreatSeverity::High, likelihood: ThreatLikelihood::Unlikely, status: ThreatStatus::Mitigated,
            component: "storage".into(), attack_vector: "Disk failure, software bug, malicious data injection".into(),
            mitigations: vec!["RocksDB transactional writes with WAL".into(), "Checksum verification on all stored data".into(), "Chainstate validation on startup".into(), "Database backup and recovery procedures".into()],
            mitre_attack_id: None, verification_test: None,
        },
        ThreatEntry {
            id: "MIN-001".into(), name: "Block Withholding Attack".into(),
            description: "Pool miner withholds found blocks to disrupt pool reward distribution.".into(),
            severity: ThreatSeverity::Medium, likelihood: ThreatLikelihood::Possible, status: ThreatStatus::Mitigated,
            component: "mining".into(), attack_vector: "Modified mining client".into(),
            mitigations: vec!["Pay-Per-Last-N-Shares (PPLNS) payout system".into(), "Share validation before acceptance".into(), "Pool monitoring for unusual patterns".into()],
            mitre_attack_id: None, verification_test: None,
        },
        ThreatEntry {
            id: "MIN-002".into(), name: "Selfish Mining".into(),
            description: "Miner privately mines blocks and selectively reveals them to waste competitor hashrate.".into(),
            severity: ThreatSeverity::Medium, likelihood: ThreatLikelihood::Unlikely, status: ThreatStatus::Mitigated,
            component: "mining".into(), attack_vector: "Block timestamp manipulation".into(),
            mitigations: vec!["Network propagation monitoring".into(), "Orphan block detection".into(), "Decentralized mining pools (Stratum V2)".into()],
            mitre_attack_id: None, verification_test: None,
        },
        ThreatEntry {
            id: "GOV-001".into(), name: "Treasury Theft via Governance".into(),
            description: "Malicious proposal drains founder treasury.".into(),
            severity: ThreatSeverity::Critical, likelihood: ThreatLikelihood::Rare, status: ThreatStatus::Mitigated,
            component: "governance".into(), attack_vector: "Compromised signer keys, social engineering".into(),
            mitigations: vec!["3-of-5 multisig for cold storage".into(), "PSBT multi-device signing workflow".into(), "48-hour time lock on large withdrawals".into(), "Daily withdrawal limits".into(), "Full audit trail for all transactions".into()],
            mitre_attack_id: None, verification_test: Some("src/wallet/src/treasury.rs".into()),
        },
        ThreatEntry {
            id: "GOV-002".into(), name: "Governance Takeover".into(),
            description: "Attacker gains voting majority to pass malicious proposals.".into(),
            severity: ThreatSeverity::High, likelihood: ThreatLikelihood::Rare, status: ThreatStatus::Mitigated,
            component: "governance".into(), attack_vector: "Vote buying, sybil attack on governance".into(),
            mitigations: vec!["Time-locked voting power".into(), "Vote delegation system".into(), "Quorum requirements for proposals".into(), "Transparent on-chain voting".into()],
            mitre_attack_id: None, verification_test: None,
        },
        ThreatEntry {
            id: "API-001".into(), name: "RPC Authentication Bypass".into(),
            description: "Unauthorized access to RPC endpoints enables fund theft.".into(),
            severity: ThreatSeverity::Critical, likelihood: ThreatLikelihood::Possible, status: ThreatStatus::Mitigated,
            component: "api".into(), attack_vector: "Weak credentials, network exposure".into(),
            mitigations: vec!["RPC authentication required (username/password)".into(), "IP whitelisting for RPC access".into(), "Localhost-only binding by default".into(), "HTTPS/TLS support for remote RPC".into()],
            mitre_attack_id: Some("T1078".into()), verification_test: None,
        },
        ThreatEntry {
            id: "EXP-001".into(), name: "XSS in Block Explorer".into(),
            description: "Malicious script execution via transaction data displayed on explorer.".into(),
            severity: ThreatSeverity::Medium, likelihood: ThreatLikelihood::Possible, status: ThreatStatus::Mitigated,
            component: "explorer".into(), attack_vector: "Transaction metadata injection".into(),
            mitigations: vec!["Input sanitization for all user-facing data".into(), "Content Security Policy (CSP) headers".into(), "Rate limiting on API endpoints".into()],
            mitre_attack_id: Some("T1059".into()), verification_test: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_model_creation() {
        let model = ThreatModel::new();
        assert!(model.total() > 0);
        assert_eq!(model.version, "1.0.0");
    }

    #[test]
    fn test_eight_components_covered() {
        let model = ThreatModel::new();
        let components: std::collections::HashSet<&str> =
            model.threats.iter().map(|t| t.component.as_str()).collect();
        for c in &[
            "consensus",
            "networking",
            "wallet",
            "storage",
            "mining",
            "governance",
            "api",
            "explorer",
        ] {
            assert!(
                components.contains(c),
                "Missing threats for component: {}",
                c
            );
        }
    }

    #[test]
    fn test_20_threats_defined() {
        let model = ThreatModel::new();
        assert_eq!(
            model.total(),
            20,
            "Threat model should contain exactly 20 threats"
        );
    }

    #[test]
    fn test_all_threats_have_mitigations() {
        let model = ThreatModel::new();
        for threat in &model.threats {
            assert!(
                !threat.mitigations.is_empty(),
                "Threat {} has no mitigations",
                threat.id
            );
        }
    }

    #[test]
    fn test_mitigation_percentage_above_90() {
        let model = ThreatModel::new();
        let pct = model.mitigation_pct();
        assert!(
            pct >= 95.0,
            "Mitigation coverage should be >=95%, got {}%",
            pct
        );
    }

    #[test]
    fn test_critical_threats_have_mitigations() {
        let model = ThreatModel::new();
        let critical = model.by_severity(ThreatSeverity::Critical);
        for threat in &critical {
            assert!(
                threat.mitigations.len() >= 3,
                "Critical threat {} needs 3+ mitigations, has {}",
                threat.id,
                threat.mitigations.len()
            );
        }
    }

    #[test]
    fn test_threat_model_serialization() {
        let model = ThreatModel::new();
        let json = serde_json::to_string(&model).expect("Should serialize to JSON");
        assert!(json.contains("CONS-001"));
        assert!(json.contains("51% Attack"));
        assert!(json.contains("\"Critical\""));
    }

    #[test]
    fn test_all_threats_have_ids() {
        let model = ThreatModel::new();
        for threat in &model.threats {
            assert!(!threat.id.is_empty(), "Threat {} missing ID", threat.name);
            assert!(threat.id.len() >= 7, "Threat ID too short: {}", threat.id);
        }
    }
}
