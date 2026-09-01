//! Phase 15 — Monitoring & Incident Response
//!
//! Validates Udaya's monitoring and incident response capabilities:
//! - Automated backup procedures
//! - Backup verification and integrity
//! - Recovery procedures with rollback
//! - Alerting system with severity levels
//! - 24/7 Runbook generation
//! - Incident response automation
//!
//! Run with: cargo test --test phase15_monitoring_incident_response -- --nocapture

use udaya_core::monitoring::{
    AlertConfig, AlertRecord, AlertSeverity, AlertType, BackupConfig, BackupRecord,
    BackupStatus, BackupType, MonitoringManager, MonitoringReport, MonitoringStats,
    RecoveryProcedure, RecoveryRecord, Runbook, RunbookSection,
};

// ============================================================================
// PHASE 15 TEST 1: AUTOMATED BACKUP PROCEDURES
// ============================================================================
#[test]
fn test_phase15_automated_backup_procedures() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 15 — MONITORING & INCIDENT RESPONSE");
    println!("  Automated Backup Procedures");
    println!("══════════════════════════════════════════════════════\n");

    let manager = MonitoringManager::new(BackupConfig::default(), AlertConfig::default());

    // Test full backup
    manager.record_backup(
        BackupType::Full,
        BackupStatus::Completed,
        1_500_000_000,
        "/backups/udaya_full_001.dat",
        "sha256:abc123def456",
        450,
    );

    // Test incremental backup
    manager.record_backup(
        BackupType::Incremental,
        BackupStatus::Completed,
        150_000_000,
        "/backups/udaya_incr_001.dat",
        "sha256:789ghi012jkl",
        45,
    );

    // Test failed backup
    manager.record_backup(
        BackupType::Snapshot,
        BackupStatus::Failed,
        0,
        "/backups/udaya_snap_001.dat",
        "",
        0,
    );

    let stats = manager.get_stats();

    println!("\n  Backup Statistics:");
    println!("  ─────────────────────────────────────────────");
    println!("    Total Backups: {}", stats.total_backups);
    println!("    Failed Backups: {}", stats.total_backups_failed);
    println!("    Success Rate: {:.1}%", stats.backup_success_rate * 100.0);
    println!("    Last Full Backup: {}", stats.last_full_backup);
    println!("    Last Incremental Backup: {}", stats.last_incremental_backup);

    assert_eq!(stats.total_backups, 3);
    assert_eq!(stats.total_backups_failed, 1);
    assert!((stats.backup_success_rate - 0.666).abs() < 0.01);

    println!("\n  ✅ Automated backup procedures verified");
}

// ============================================================================
// PHASE 15 TEST 2: BACKUP VERIFICATION AND INTEGRITY
// ============================================================================
#[test]
fn test_phase15_backup_verification() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 15 — MONITORING & INCIDENT RESPONSE");
    println!("  Backup Verification and Integrity");
    println!("══════════════════════════════════════════════════════\n");

    let manager = MonitoringManager::new(BackupConfig::default(), AlertConfig::default());

    // Test verified backup
    manager.record_backup(
        BackupType::Full,
        BackupStatus::Verified,
        2_000_000_000,
        "/backups/udaya_full_verified.dat",
        "sha256:verified123",
        600,
    );

    // Test corrupted backup
    manager.record_backup(
        BackupType::Full,
        BackupStatus::Corrupted,
        2_000_000_000,
        "/backups/udaya_full_corrupt.dat",
        "sha256:corrupt456",
        600,
    );

    let stats = manager.get_stats();

    println!("\n  Backup Verification Results:");
    println!("  ─────────────────────────────────────────────");
    println!("    Total Backups: {}", stats.total_backups);
    println!("    Failed/Corrupted: {}", stats.total_backups_failed);

    assert_eq!(stats.total_backups, 2);
    assert_eq!(stats.total_backups_failed, 1);

    // Verify backup records
    let backups = manager.backups.read();
    assert_eq!(backups.len(), 2);
    assert_eq!(backups[0].status, BackupStatus::Verified);
    assert_eq!(backups[1].status, BackupStatus::Corrupted);

    println!("\n  ✅ Backup verification and integrity checks passed");
}

// ============================================================================
// PHASE 15 TEST 3: ALERTING SYSTEM
// ============================================================================
#[test]
fn test_phase15_alerting_system() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 15 — MONITORING & INCIDENT RESPONSE");
    println!("  Alerting System");
    println!("══════════════════════════════════════════════════════\n");

    let manager = MonitoringManager::new(BackupConfig::default(), AlertConfig::default());

    // Test different alert types and severities
    manager.record_alert(
        AlertType::NodeDown,
        AlertSeverity::Emergency,
        "Node unresponsive for 5 minutes",
        "health_check",
    );

    manager.record_alert(
        AlertType::HighMemoryUsage,
        AlertSeverity::Warning,
        "Memory usage at 85%",
        "system_monitor",
    );

    manager.record_alert(
        AlertType::SyncFailure,
        AlertSeverity::Critical,
        "Failed to sync with 3 peers",
        "p2p_network",
    );

    manager.record_alert(
        AlertType::BlockStall,
        AlertSeverity::Critical,
        "No new blocks for 30 minutes",
        "block_monitor",
    );

    manager.record_alert(
        AlertType::PeerLoss,
        AlertSeverity::Info,
        "Peer disconnected",
        "p2p_network",
    );

    let stats = manager.get_stats();

    println!("\n  Alert Statistics:");
    println!("  ─────────────────────────────────────────────");
    println!("    Total Alerts: {}", stats.total_alerts);
    println!("    Active Alerts: {}", stats.active_alerts);
    println!("    Critical Alerts: {}", stats.critical_alerts);

    assert_eq!(stats.total_alerts, 5);
    assert_eq!(stats.active_alerts, 5);
    assert_eq!(stats.critical_alerts, 2);

    // Test alert resolution
    manager.resolve_alert(AlertType::HighMemoryUsage);
    let stats_after = manager.get_stats();
    assert_eq!(stats_after.active_alerts, 4);

    println!("\n  ✅ Alerting system verified");
}

// ============================================================================
// PHASE 15 TEST 4: RECOVERY PROCEDURES
// ============================================================================
#[test]
fn test_phase15_recovery_procedures() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 15 — MONITORING & INCIDENT RESPONSE");
    println!("  Recovery Procedures");
    println!("══════════════════════════════════════════════════════\n");

    let manager = MonitoringManager::new(BackupConfig::default(), AlertConfig::default());

    // Test automatic recovery triggers
    manager.record_alert(
        AlertType::NodeDown,
        AlertSeverity::Emergency,
        "Node unresponsive",
        "health_check",
    );

    manager.record_alert(
        AlertType::DatabaseCorruption,
        AlertSeverity::Critical,
        "Database corruption detected",
        "startup",
    );

    manager.record_alert(
        AlertType::SyncFailure,
        AlertSeverity::Critical,
        "Sync failure with peers",
        "p2p_network",
    );

    let stats = manager.get_stats();

    println!("\n  Recovery Statistics:");
    println!("  ─────────────────────────────────────────────");
    println!("    Total Recoveries: {}", stats.total_recoveries);
    println!("    Failed Recoveries: {}", stats.total_recoveries_failed);
    println!("    Success Rate: {:.1}%", stats.recovery_success_rate * 100.0);

    // Auto-recovery should have triggered for all alerts
    assert!(stats.total_recoveries >= 3, "Should have triggered recoveries");

    // All recoveries should succeed in this simulation
    assert_eq!(stats.recovery_success_rate, 1.0);

    println!("\n  ✅ Recovery procedures verified");
}

// ============================================================================
// PHASE 15 TEST 5: 24/7 RUNBOOK GENERATION
// ============================================================================
#[test]
fn test_phase15_runbook_generation() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 15 — MONITORING & INCIDENT RESPONSE");
    println!("  24/7 Runbook Generation");
    println!("══════════════════════════════════════════════════════\n");

    let manager = MonitoringManager::new(BackupConfig::default(), AlertConfig::default());
    let runbook = manager.generate_runbook();

    println!("\n  Runbook Details:");
    println!("  ─────────────────────────────────────────────");
    println!("    Version: {}", runbook.version);
    println!("    Sections: {}", runbook.sections.len());
    println!("    Last Updated: {}", runbook.last_updated);

    // Verify runbook structure
    assert_eq!(runbook.version, "1.0.0");
    assert!(!runbook.sections.is_empty());

    // Verify each section has required components
    for section in &runbook.sections {
        println!("\n  Section: {}", section.title);
        println!("    Description: {}", section.description);
        println!("    Steps: {}", section.steps.len());
        println!("    Escalation Contacts: {}", section.escalation.len());

        assert!(!section.title.is_empty());
        assert!(!section.steps.is_empty());
        assert!(!section.escalation.is_empty());

        // Verify steps have required fields
        for step in &section.steps {
            assert!(step.step_number > 0);
            assert!(!step.action.is_empty());
            assert!(!step.expected_result.is_empty());
        }

        // Verify escalation contacts
        for contact in &section.escalation {
            assert!(!contact.role.is_empty());
            assert!(!contact.contact.is_empty());
            assert!(contact.escalation_time_minutes > 0);
        }
    }

    // Verify JSON serialization
    let json = serde_json::to_string_pretty(&runbook).unwrap();
    assert!(json.contains("version"));
    assert!(json.contains("sections"));
    assert!(json.contains("escalation"));

    // Write runbook to file
    std::fs::write("runbook.json", json).expect("Failed to write runbook");
    assert!(std::path::Path::new("runbook.json").exists());
    let _ = std::fs::remove_file("runbook.json");

    println!("\n  ✅ 24/7 Runbook generated and verified");
}

// ============================================================================
// PHASE 15 TEST 6: MONITORING REPORT GENERATION
// ============================================================================
#[test]
fn test_phase15_monitoring_report_generation() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 15 — MONITORING & INCIDENT RESPONSE");
    println!("  Monitoring Report Generation");
    println!("══════════════════════════════════════════════════════\n");

    let manager = MonitoringManager::new(BackupConfig::default(), AlertConfig::default());

    // Simulate some activity
    manager.record_backup(
        BackupType::Full,
        BackupStatus::Completed,
        1_500_000_000,
        "/backups/udaya_full_001.dat",
        "sha256:abc123",
        300,
    );

    manager.record_alert(
        AlertType::HighMemoryUsage,
        AlertSeverity::Warning,
        "Memory usage at 80%",
        "system_monitor",
    );

    manager.record_alert(
        AlertType::PeerLoss,
        AlertSeverity::Info,
        "Peer disconnected",
        "p2p_network",
    );

    // Generate report
    let report = manager.generate_report();

    println!("\n  Monitoring Report:");
    println!("  ─────────────────────────────────────────────");
    println!("    Generated At: {}", report.generated_at);
    println!("    Grade: {}", report.grade);
    println!("    Total Backups: {}", report.stats.total_backups);
    println!("    Total Alerts: {}", report.stats.total_alerts);
    println!("    Active Alerts: {}", report.stats.active_alerts);
    println!("    Total Recoveries: {}", report.stats.total_recoveries);
    println!("    Recommendations: {}", report.recommendations.len());

    // Verify report structure
    assert!(report.grade.contains("A") || report.grade.contains("B") || report.grade.contains("C"));
    assert!(!report.recommendations.is_empty());

    // Verify JSON serialization
    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("grade"));
    assert!(json.contains("recommendations"));
    assert!(json.contains("stats"));

    // Write report to file
    std::fs::write("phase15_monitoring_report.json", json)
        .expect("Failed to write monitoring report");

    println!("\n  📄 Report saved to: phase15_monitoring_report.json");

    assert!(std::path::Path::new("phase15_monitoring_report.json").exists());
    let _ = std::fs::remove_file("phase15_monitoring_report.json");

    println!("\n  ✅ Monitoring report generated");
}

// ============================================================================
// PHASE 15 TEST 7: INCIDENT RESPONSE AUTOMATION
// ============================================================================
#[test]
fn test_phase15_incident_response_automation() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 15 — MONITORING & INCIDENT RESPONSE");
    println!("  Incident Response Automation");
    println!("══════════════════════════════════════════════════════\n");

    let backup_config = BackupConfig {
        enabled: true,
        backup_dir: "./backups".to_string(),
        max_backups: 10,
        full_backup_interval_hours: 24,
        incremental_backup_interval_minutes: 60,
        compression_enabled: true,
        encryption_enabled: false,
    };

    let alert_config = AlertConfig {
        enabled: true,
        min_severity: AlertSeverity::Warning,
        alert_retention_days: 30,
        notification_endpoints: vec!["mailto:alerts@udaya.network".to_string()],
        auto_recovery_enabled: true,
    };

    let manager = MonitoringManager::new(backup_config, alert_config);

    // Test auto-recovery for different alert types
    let test_cases = vec![
        (AlertType::NodeDown, RecoveryProcedure::RestartNode),
        (AlertType::DatabaseCorruption, RecoveryProcedure::RestoreBackup),
        (AlertType::SyncFailure, RecoveryProcedure::ReindexChain),
        (AlertType::HighMemoryUsage, RecoveryProcedure::ResetMempool),
        (AlertType::PeerLoss, RecoveryProcedure::ClearPeers),
    ];

    println!("\n  Auto-Recovery Test Cases:");
    println!("  ─────────────────────────────────────────────");

    for (alert_type, expected_procedure) in test_cases {
        manager.record_alert(
            alert_type,
            AlertSeverity::Critical,
            format!("Test alert for {:?}", alert_type),
            "test",
        );

        let recoveries = manager.recoveries.read();
        if let Some(last_recovery) = recoveries.last() {
            assert_eq!(last_recovery.procedure, expected_procedure);
            println!(
                "    {:?} -> {:?}: {}",
                alert_type, last_recovery.procedure, last_recovery.success
            );
        }
    }

    let stats = manager.get_stats();
    assert!(stats.total_recoveries >= 5, "Should have triggered 5 recoveries");
    assert_eq!(stats.recovery_success_rate, 1.0, "All recoveries should succeed");

    println!("\n  ✅ Incident response automation verified");
}
