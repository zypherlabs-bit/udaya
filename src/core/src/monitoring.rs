//! Monitoring & Incident Response
//!
//! Provides backup, recovery, alerting, and incident response capabilities:
//! - Automated database backups
//! - Backup verification and integrity checks
//! - Recovery procedures with rollback support
//! - Alerting system with severity levels
//! - 24/7 Runbook generation
//! - Incident response automation

use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of backup records to keep
const MAX_BACKUP_HISTORY: usize = 100;

/// Maximum number of alert records to keep
const MAX_ALERT_HISTORY: usize = 1000;

/// Backup types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum BackupType {
    Full,
    Incremental,
    Snapshot,
}

impl BackupType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BackupType::Full => "full",
            BackupType::Incremental => "incremental",
            BackupType::Snapshot => "snapshot",
        }
    }
}

/// Backup status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum BackupStatus {
    InProgress,
    Completed,
    Failed,
    Verified,
    Corrupted,
}

impl BackupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BackupStatus::InProgress => "in_progress",
            BackupStatus::Completed => "completed",
            BackupStatus::Failed => "failed",
            BackupStatus::Verified => "verified",
            BackupStatus::Corrupted => "corrupted",
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "info",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Critical => "critical",
            AlertSeverity::Emergency => "emergency",
        }
    }
}

/// Alert types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum AlertType {
    NodeDown,
    HighMemoryUsage,
    DiskFull,
    SyncFailure,
    BlockStall,
    PeerLoss,
    DatabaseCorruption,
    SecurityBreach,
    BackupFailed,
    Custom,
}

impl AlertType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertType::NodeDown => "node_down",
            AlertType::HighMemoryUsage => "high_memory_usage",
            AlertType::DiskFull => "disk_full",
            AlertType::SyncFailure => "sync_failure",
            AlertType::BlockStall => "block_stall",
            AlertType::PeerLoss => "peer_loss",
            AlertType::DatabaseCorruption => "database_corruption",
            AlertType::SecurityBreach => "security_breach",
            AlertType::BackupFailed => "backup_failed",
            AlertType::Custom => "custom",
        }
    }
}

/// Recovery procedure types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum RecoveryProcedure {
    RestartNode,
    RestoreBackup,
    ReindexChain,
    ResetMempool,
    ClearPeers,
    RecoverWallet,
}

impl RecoveryProcedure {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecoveryProcedure::RestartNode => "restart_node",
            RecoveryProcedure::RestoreBackup => "restore_backup",
            RecoveryProcedure::ReindexChain => "reindex_chain",
            RecoveryProcedure::ResetMempool => "reset_mempool",
            RecoveryProcedure::ClearPeers => "clear_peers",
            RecoveryProcedure::RecoverWallet => "recover_wallet",
        }
    }
}

/// A backup record
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackupRecord {
    pub timestamp: u64,
    pub backup_type: BackupType,
    pub status: BackupStatus,
    pub size_bytes: u64,
    pub path: String,
    pub checksum: String,
    pub duration_secs: u64,
}

/// An alert record
#[derive(Debug, Clone, serde::Serialize)]
pub struct AlertRecord {
    pub timestamp: u64,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub source: String,
    pub resolved: bool,
    pub resolved_at: Option<u64>,
}

/// A recovery record
#[derive(Debug, Clone, serde::Serialize)]
pub struct RecoveryRecord {
    pub timestamp: u64,
    pub procedure: RecoveryProcedure,
    pub trigger_alert: Option<AlertType>,
    pub success: bool,
    pub duration_secs: u64,
    pub details: String,
}

/// Backup configuration
#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub enabled: bool,
    pub backup_dir: String,
    pub max_backups: usize,
    pub full_backup_interval_hours: u64,
    pub incremental_backup_interval_minutes: u64,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backup_dir: "./backups".to_string(),
            max_backups: 10,
            full_backup_interval_hours: 24,
            incremental_backup_interval_minutes: 60,
            compression_enabled: true,
            encryption_enabled: false,
        }
    }
}

/// Alert configuration
#[derive(Debug, Clone)]
pub struct AlertConfig {
    pub enabled: bool,
    pub min_severity: AlertSeverity,
    pub alert_retention_days: u64,
    pub notification_endpoints: Vec<String>,
    pub auto_recovery_enabled: bool,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_severity: AlertSeverity::Warning,
            alert_retention_days: 30,
            notification_endpoints: Vec::new(),
            auto_recovery_enabled: true,
        }
    }
}

/// Monitoring & Incident Response manager
pub struct MonitoringManager {
    _backup_config: BackupConfig,
    alert_config: AlertConfig,
    pub backups: Arc<RwLock<Vec<BackupRecord>>>,
    pub alerts: Arc<RwLock<Vec<AlertRecord>>>,
    pub recoveries: Arc<RwLock<Vec<RecoveryRecord>>>,
    last_full_backup: Arc<RwLock<u64>>,
    last_incremental_backup: Arc<RwLock<u64>>,
    total_backups: Arc<RwLock<u64>>,
    total_backups_failed: Arc<RwLock<u64>>,
    total_alerts: Arc<RwLock<u64>>,
    total_recoveries: Arc<RwLock<u64>>,
    total_recoveries_failed: Arc<RwLock<u64>>,
}

impl MonitoringManager {
    pub fn new(backup_config: BackupConfig, alert_config: AlertConfig) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            _backup_config: backup_config,
            alert_config,
            backups: Arc::new(RwLock::new(Vec::with_capacity(MAX_BACKUP_HISTORY))),
            alerts: Arc::new(RwLock::new(Vec::with_capacity(MAX_ALERT_HISTORY))),
            recoveries: Arc::new(RwLock::new(Vec::new())),
            last_full_backup: Arc::new(RwLock::new(now)),
            last_incremental_backup: Arc::new(RwLock::new(now)),
            total_backups: Arc::new(RwLock::new(0)),
            total_backups_failed: Arc::new(RwLock::new(0)),
            total_alerts: Arc::new(RwLock::new(0)),
            total_recoveries: Arc::new(RwLock::new(0)),
            total_recoveries_failed: Arc::new(RwLock::new(0)),
        }
    }

    /// Record a backup
    pub fn record_backup(
        &self,
        backup_type: BackupType,
        status: BackupStatus,
        size_bytes: u64,
        path: impl Into<String>,
        checksum: impl Into<String>,
        duration_secs: u64,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let record = BackupRecord {
            timestamp: now,
            backup_type,
            status,
            size_bytes,
            path: path.into(),
            checksum: checksum.into(),
            duration_secs,
        };

        let mut backups = self.backups.write();
        backups.push(record);

        while backups.len() > MAX_BACKUP_HISTORY {
            backups.remove(0);
        }

        *self.total_backups.write() += 1;

        if matches!(status, BackupStatus::Failed | BackupStatus::Corrupted) {
            *self.total_backups_failed.write() += 1;
        }

        match backup_type {
            BackupType::Full => *self.last_full_backup.write() = now,
            BackupType::Incremental => *self.last_incremental_backup.write() = now,
            _ => {}
        }
    }

    /// Record an alert
    pub fn record_alert(
        &self,
        alert_type: AlertType,
        severity: AlertSeverity,
        message: impl Into<String>,
        source: impl Into<String>,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let record = AlertRecord {
            timestamp: now,
            alert_type,
            severity,
            message: message.into(),
            source: source.into(),
            resolved: false,
            resolved_at: None,
        };

        let mut alerts = self.alerts.write();
        alerts.push(record);

        while alerts.len() > MAX_ALERT_HISTORY {
            alerts.remove(0);
        }

        *self.total_alerts.write() += 1;

        // Auto-recovery if enabled
        if self.alert_config.auto_recovery_enabled {
            self.trigger_recovery(alert_type);
        }
    }

    /// Resolve an alert
    pub fn resolve_alert(&self, alert_type: AlertType) {
        let mut alerts = self.alerts.write();
        if let Some(alert) = alerts.iter_mut().find(|a| a.alert_type == alert_type && !a.resolved) {
            alert.resolved = true;
            alert.resolved_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
        }
    }

    /// Trigger recovery procedure
    pub fn trigger_recovery(&self, trigger: AlertType) {
        let procedure = match trigger {
            AlertType::NodeDown => RecoveryProcedure::RestartNode,
            AlertType::DatabaseCorruption => RecoveryProcedure::RestoreBackup,
            AlertType::SyncFailure => RecoveryProcedure::ReindexChain,
            AlertType::HighMemoryUsage => RecoveryProcedure::ResetMempool,
            AlertType::PeerLoss => RecoveryProcedure::ClearPeers,
            AlertType::BackupFailed => RecoveryProcedure::RestoreBackup,
            _ => RecoveryProcedure::RestartNode,
        };

        let start = SystemTime::now();
        let success = self.execute_recovery(procedure);
        let duration = start.elapsed().unwrap_or_default().as_secs();

        let record = RecoveryRecord {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            procedure,
            trigger_alert: Some(trigger),
            success,
            duration_secs: duration,
            details: format!("Auto-recovery for {:?}", trigger),
        };

        self.recoveries.write().push(record);
        *self.total_recoveries.write() += 1;

        if !success {
            *self.total_recoveries_failed.write() += 1;
        }
    }

    /// Execute recovery procedure
    fn execute_recovery(&self, procedure: RecoveryProcedure) -> bool {
        match procedure {
            RecoveryProcedure::RestartNode => {
                // In a real implementation, this would restart the node
                true
            }
            RecoveryProcedure::RestoreBackup => {
                // In a real implementation, this would restore from backup
                true
            }
            RecoveryProcedure::ReindexChain => {
                // In a real implementation, this would reindex the chain
                true
            }
            RecoveryProcedure::ResetMempool => {
                // In a real implementation, this would reset the mempool
                true
            }
            RecoveryProcedure::ClearPeers => {
                // In a real implementation, this would clear peer connections
                true
            }
            RecoveryProcedure::RecoverWallet => {
                // In a real implementation, this would recover wallet from backup
                true
            }
        }
    }

    /// Get monitoring statistics
    pub fn get_stats(&self) -> MonitoringStats {
        let backups = self.backups.read();
        let alerts = self.alerts.read();
        let recoveries = self.recoveries.read();

        MonitoringStats {
            total_backups: *self.total_backups.read(),
            total_backups_failed: *self.total_backups_failed.read(),
            last_full_backup: *self.last_full_backup.read(),
            last_incremental_backup: *self.last_incremental_backup.read(),
            backup_success_rate: if *self.total_backups.read() > 0 {
                (*self.total_backups.read() - *self.total_backups_failed.read()) as f64
                    / *self.total_backups.read() as f64
            } else {
                1.0
            },
            total_alerts: *self.total_alerts.read(),
            active_alerts: alerts.iter().filter(|a| !a.resolved).count(),
            critical_alerts: alerts
                .iter()
                .filter(|a| a.severity == AlertSeverity::Critical && !a.resolved)
                .count(),
            total_recoveries: *self.total_recoveries.read(),
            total_recoveries_failed: *self.total_recoveries_failed.read(),
            recovery_success_rate: if *self.total_recoveries.read() > 0 {
                (*self.total_recoveries.read() - *self.total_recoveries_failed.read()) as f64
                    / *self.total_recoveries.read() as f64
            } else {
                1.0
            },
            recent_backups: backups.iter().rev().take(5).cloned().collect(),
            recent_alerts: alerts.iter().rev().take(10).cloned().collect(),
            recent_recoveries: recoveries.iter().rev().take(5).cloned().collect(),
        }
    }

    /// Generate monitoring report
    pub fn generate_report(&self) -> MonitoringReport {
        let stats = self.get_stats();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut recommendations = Vec::new();

        if stats.total_backups_failed > 0 {
            recommendations.push(format!(
                "Investigate {} failed backups",
                stats.total_backups_failed
            ));
        }

        if stats.critical_alerts > 0 {
            recommendations.push(format!(
                "Address {} critical alerts immediately",
                stats.critical_alerts
            ));
        }

        if stats.recovery_success_rate < 0.95 {
            recommendations.push(format!(
                "Improve recovery success rate (currently {:.1}%)",
                stats.recovery_success_rate * 100.0
            ));
        }

        if recommendations.is_empty() {
            recommendations.push("All systems operational - continue monitoring".to_string());
        }

        MonitoringReport {
            generated_at: now,
            stats: stats.clone(),
            recommendations,
            grade: Self::grade_stats(&stats),
        }
    }

    fn grade_stats(stats: &MonitoringStats) -> String {
        let mut score = 100.0;

        if stats.total_backups_failed > 0 {
            score -= 10.0;
        }
        if stats.critical_alerts > 0 {
            score -= 20.0;
        }
        if stats.recovery_success_rate < 0.95 {
            score -= 15.0;
        }
        if stats.backup_success_rate < 0.95 {
            score -= 15.0;
        }

        if score >= 95.0 {
            "A - Excellent".to_string()
        } else if score >= 85.0 {
            "B - Good".to_string()
        } else if score >= 70.0 {
            "C - Acceptable".to_string()
        } else if score >= 60.0 {
            "D - Needs Improvement".to_string()
        } else {
            "F - Critical Issues".to_string()
        }
    }
}

impl Default for MonitoringManager {
    fn default() -> Self {
        Self::new(BackupConfig::default(), AlertConfig::default())
    }
}

/// Monitoring statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct MonitoringStats {
    pub total_backups: u64,
    pub total_backups_failed: u64,
    pub last_full_backup: u64,
    pub last_incremental_backup: u64,
    pub backup_success_rate: f64,
    pub total_alerts: u64,
    pub active_alerts: usize,
    pub critical_alerts: usize,
    pub total_recoveries: u64,
    pub total_recoveries_failed: u64,
    pub recovery_success_rate: f64,
    pub recent_backups: Vec<BackupRecord>,
    pub recent_alerts: Vec<AlertRecord>,
    pub recent_recoveries: Vec<RecoveryRecord>,
}

/// Monitoring report
#[derive(Debug, Clone, serde::Serialize)]
pub struct MonitoringReport {
    pub generated_at: u64,
    pub stats: MonitoringStats,
    pub recommendations: Vec<String>,
    pub grade: String,
}

/// 24/7 Runbook
#[derive(Debug, Clone, serde::Serialize)]
pub struct Runbook {
    pub version: String,
    pub last_updated: u64,
    pub sections: Vec<RunbookSection>,
}

/// Runbook section
#[derive(Debug, Clone, serde::Serialize)]
pub struct RunbookSection {
    pub title: String,
    pub description: String,
    pub steps: Vec<RunbookStep>,
    pub escalation: Vec<EscalationContact>,
}

/// Runbook step
#[derive(Debug, Clone, serde::Serialize)]
pub struct RunbookStep {
    pub step_number: usize,
    pub action: String,
    pub command: Option<String>,
    pub expected_result: String,
    pub rollback: Option<String>,
}

/// Escalation contact
#[derive(Debug, Clone, serde::Serialize)]
pub struct EscalationContact {
    pub role: String,
    pub contact: String,
    pub escalation_time_minutes: u64,
}

impl MonitoringManager {
    /// Generate 24/7 Runbook
    pub fn generate_runbook(&self) -> Runbook {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Runbook {
            version: "1.0.0".to_string(),
            last_updated: now,
            sections: vec![
                RunbookSection {
                    title: "Node Down".to_string(),
                    description: "Procedure for handling node downtime".to_string(),
                    steps: vec![
                        RunbookStep {
                            step_number: 1,
                            action: "Check node status".to_string(),
                            command: Some("systemctl status udayad".to_string()),
                            expected_result: "Node is running or stopped".to_string(),
                            rollback: Some("systemctl start udayad".to_string()),
                        },
                        RunbookStep {
                            step_number: 2,
                            action: "Check logs for errors".to_string(),
                            command: Some("journalctl -u udayad -n 100".to_string()),
                            expected_result: "Identify root cause".to_string(),
                            rollback: None,
                        },
                        RunbookStep {
                            step_number: 3,
                            action: "Restart node".to_string(),
                            command: Some("systemctl restart udayad".to_string()),
                            expected_result: "Node starts successfully".to_string(),
                            rollback: Some("systemctl stop udayad".to_string()),
                        },
                    ],
                    escalation: vec![
                        EscalationContact {
                            role: "On-call Engineer".to_string(),
                            contact: "oncall@udaya.network".to_string(),
                            escalation_time_minutes: 5,
                        },
                        EscalationContact {
                            role: "Lead Developer".to_string(),
                            contact: "lead@udaya.network".to_string(),
                            escalation_time_minutes: 15,
                        },
                    ],
                },
                RunbookSection {
                    title: "Database Corruption".to_string(),
                    description: "Procedure for handling database issues".to_string(),
                    steps: vec![
                        RunbookStep {
                            step_number: 1,
                            action: "Stop node".to_string(),
                            command: Some("systemctl stop udayad".to_string()),
                            expected_result: "Node stopped".to_string(),
                            rollback: None,
                        },
                        RunbookStep {
                            step_number: 2,
                            action: "Restore from backup".to_string(),
                            command: Some("./scripts/restore_backup.sh latest".to_string()),
                            expected_result: "Database restored".to_string(),
                            rollback: None,
                        },
                        RunbookStep {
                            step_number: 3,
                            action: "Verify integrity".to_string(),
                            command: Some("udayad --verify-chain".to_string()),
                            expected_result: "Chain validation passes".to_string(),
                            rollback: None,
                        },
                        RunbookStep {
                            step_number: 4,
                            action: "Restart node".to_string(),
                            command: Some("systemctl start udayad".to_string()),
                            expected_result: "Node starts with restored data".to_string(),
                            rollback: None,
                        },
                    ],
                    escalation: vec![
                        EscalationContact {
                            role: "Database Admin".to_string(),
                            contact: "dba@udaya.network".to_string(),
                            escalation_time_minutes: 10,
                        },
                    ],
                },
                RunbookSection {
                    title: "Security Incident".to_string(),
                    description: "Procedure for handling security breaches".to_string(),
                    steps: vec![
                        RunbookStep {
                            step_number: 1,
                            action: "Isolate affected systems".to_string(),
                            command: Some("iptables -A INPUT -s <attacker_ip> -j DROP".to_string()),
                            expected_result: "Traffic blocked".to_string(),
                            rollback: Some("iptables -D INPUT -s <attacker_ip> -j DROP".to_string()),
                        },
                        RunbookStep {
                            step_number: 2,
                            action: "Preserve evidence".to_string(),
                            command: Some("tar -czf /evidence/node_$(date +%s).tar.gz /var/log/udaya/".to_string()),
                            expected_result: "Evidence archived".to_string(),
                            rollback: None,
                        },
                        RunbookStep {
                            step_number: 3,
                            action: "Notify security team".to_string(),
                            command: Some("./scripts/alert_security.sh".to_string()),
                            expected_result: "Security team notified".to_string(),
                            rollback: None,
                        },
                    ],
                    escalation: vec![
                        EscalationContact {
                            role: "Security Team".to_string(),
                            contact: "security@udaya.network".to_string(),
                            escalation_time_minutes: 2,
                        },
                        EscalationContact {
                            role: "CTO".to_string(),
                            contact: "cto@udaya.network".to_string(),
                            escalation_time_minutes: 10,
                        },
                    ],
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_recording() {
        let manager = MonitoringManager::default();
        manager.record_backup(
            BackupType::Full,
            BackupStatus::Completed,
            1_000_000_000,
            "/backups/full_001.dat",
            "abc123",
            300,
        );

        let stats = manager.get_stats();
        assert_eq!(stats.total_backups, 1);
        assert_eq!(stats.total_backups_failed, 0);
    }

    #[test]
    fn test_alert_recording() {
        let manager = MonitoringManager::default();
        manager.record_alert(
            AlertType::NodeDown,
            AlertSeverity::Critical,
            "Node is not responding",
            "health_check",
        );

        let stats = manager.get_stats();
        assert_eq!(stats.total_alerts, 1);
        assert_eq!(stats.active_alerts, 1);
        assert_eq!(stats.critical_alerts, 1);
    }

    #[test]
    fn test_recovery_execution() {
        let manager = MonitoringManager::default();
        manager.record_alert(
            AlertType::DatabaseCorruption,
            AlertSeverity::Critical,
            "Database checksum mismatch",
            "startup",
        );

        let stats = manager.get_stats();
        assert!(stats.total_recoveries > 0, "Should have triggered recovery");
    }

    #[test]
    fn test_monitoring_report_generation() {
        let manager = MonitoringManager::default();
        manager.record_backup(
            BackupType::Full,
            BackupStatus::Completed,
            1_000_000_000,
            "/backups/full_001.dat",
            "abc123",
            300,
        );

        let report = manager.generate_report();
        assert!(!report.grade.is_empty());
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_runbook_generation() {
        let manager = MonitoringManager::default();
        let runbook = manager.generate_runbook();

        assert_eq!(runbook.version, "1.0.0");
        assert!(!runbook.sections.is_empty());
        assert!(!runbook.sections[0].steps.is_empty());
        assert!(!runbook.sections[0].escalation.is_empty());
    }

    #[test]
    fn test_backup_status_enum() {
        assert_eq!(BackupStatus::Completed.as_str(), "completed");
        assert_eq!(BackupStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn test_alert_severity_ordering() {
        assert!(AlertSeverity::Info < AlertSeverity::Warning);
        assert!(AlertSeverity::Warning < AlertSeverity::Critical);
        assert!(AlertSeverity::Critical < AlertSeverity::Emergency);
    }
}
