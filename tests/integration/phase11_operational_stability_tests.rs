//! Phase 11 — Operational Stability Validation
//!
//! Validates Udaya's operational stability under continuous operation:
//! - Crash detection and recovery
//! - Memory leak detection
//! - Sync failure tracking and recovery
//! - Long-running stability simulation
//!
//! Run with: cargo test --test phase11_operational_stability -- --nocapture

use std::time::{Duration, Instant};
use udaya_core::stability::{
    StabilityEventType, StabilityMonitor, StabilityReport,
};

// ============================================================================
// HELPER: Simulate extended operation with periodic events
// ============================================================================
fn simulate_extended_operation(monitor: &StabilityMonitor, duration_secs: u64) {
    let start = Instant::now();
    let target = Duration::from_secs(duration_secs);

    // Simulate various events during operation
    for i in 0..duration_secs {
        if start.elapsed() > target {
            break;
        }

        // Simulate memory snapshots
        monitor.record_memory_snapshot(
            100_000_000 + (i * 100_000), // Gradual growth
            50_000_000,
        );

        // Simulate occasional peer disconnects
        if i % 60 == 0 && i > 0 {
            monitor.record_event(
                StabilityEventType::PeerDisconnect,
                format!("Peer 192.168.1.{} disconnected", i % 255),
                true,
            );
        }

        // Simulate occasional sync failures
        if i % 120 == 0 && i > 0 {
            monitor.record_sync_failure(
                format!("peer{}", i % 10),
                1000 + i,
                900 + i,
                "timeout",
            );
            // Simulate recovery after 30 seconds
            if i % 240 == 0 {
                monitor.mark_sync_recovered(&format!("peer{}", i % 10));
            }
        }

        // Simulate memory leak detection
        if i == duration_secs / 2 {
            monitor.detect_memory_leaks();
        }
    }
}

// ============================================================================
// PHASE 11 TEST 1: LONG-RUNNING STABILITY SIMULATION
// ============================================================================
#[test]
fn test_phase11_long_running_stability() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 11 — OPERATIONAL STABILITY");
    println!("  Long-Running Stability Simulation");
    println!("══════════════════════════════════════════════════════\n");

    let monitor = StabilityMonitor::new();

    println!("  Simulating 7200 seconds (2 hours) of continuous operation...");
    simulate_extended_operation(&monitor, 7200);

    let stats = monitor.get_stats();
    println!("\n  Stability Statistics:");
    println!("  ─────────────────────────────────────────────");
    println!("    Uptime: {} seconds ({:.1} hours)", stats.uptime_seconds, stats.uptime_seconds as f64 / 3600.0);
    println!("    Total Events: {}", stats.total_events);
    println!("    Crashes: {}", stats.crash_count);
    println!("    Panics: {}", stats.panic_count);
    println!("    Memory Leaks: {}", stats.memory_leak_count);
    println!("    Sync Failures: {}", stats.sync_failure_count);
    println!("    Peak Memory: {} MB", stats.peak_memory_bytes / 1_000_000);

    // Verify no crashes or panics during operation
    assert_eq!(stats.crash_count, 0, "Should have no crashes");
    assert_eq!(stats.panic_count, 0, "Should have no panics");

    // Verify events were recorded
    assert!(stats.total_events > 0, "Should have recorded events");

    // Verify memory snapshots were taken
    assert!(stats.peak_memory_bytes > 0, "Should have memory snapshots");

    println!("\n  ✅ Long-running stability simulation passed");
}

// ============================================================================
// PHASE 11 TEST 2: CRASH DETECTION AND RECOVERY
// ============================================================================
#[test]
fn test_phase11_crash_detection_and_recovery() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 11 — OPERATIONAL STABILITY");
    println!("  Crash Detection and Recovery");
    println!("══════════════════════════════════════════════════════\n");

    let monitor = StabilityMonitor::new();

    // Simulate a crash event
    monitor.record_event(
        StabilityEventType::Crash,
        "Segmentation fault in block validation",
        true,
    );

    // Simulate a panic event
    monitor.record_event(
        StabilityEventType::Panic,
        "Index out of bounds in transaction parsing",
        true,
    );

    // Record crash details
    monitor.record_crash(
        "segfault",
        "stack backtrace: ...",
        true,
        5,
    );

    let stats = monitor.get_stats();
    println!("\n  Crash Statistics:");
    println!("  ─────────────────────────────────────────────");
    println!("    Total Crashes: {}", stats.crash_count);
    println!("    Total Panics: {}", stats.panic_count);
    println!("    Total Events: {}", stats.total_events);

    // Verify crashes were recorded
    assert_eq!(stats.crash_count, 1, "Should have 1 crash");
    assert_eq!(stats.panic_count, 1, "Should have 1 panic");

    // Generate stability report
    let report = StabilityReport::generate(&monitor);
    println!("\n  Stability Report:");
    println!("    Score: {:.1}%", report.stability_score);
    println!("    Grade: {}", report.grade);
    println!("    Recommendations: {}", report.recommendations.len());

    assert!(report.stability_score < 100.0, "Crashes should reduce score");

    println!("\n  ✅ Crash detection and recovery test passed");
}

// ============================================================================
// PHASE 11 TEST 3: MEMORY LEAK DETECTION
// ============================================================================
#[test]
fn test_phase11_memory_leak_detection() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 11 — OPERATIONAL STABILITY");
    println!("  Memory Leak Detection");
    println!("══════════════════════════════════════════════════════\n");

    let monitor = StabilityMonitor::new();

    // Scenario 1: Stable memory usage
    println!("  Scenario 1: Stable memory usage");
    for i in 0..20 {
        let timestamp = 1000000 + i; // Incrementing timestamps
        monitor.record_memory_snapshot_at(100_000_000, 50_000_000, timestamp);
    }
    let leak1 = monitor.detect_memory_leaks();
    println!("    Leak detected: {}", leak1.is_some());

    // Scenario 2: Memory leak
    println!("\n  Scenario 2: Memory leak detection");
    monitor.reset();
    for i in 0..30 {
        let timestamp = 2000000 + i;
        monitor.record_memory_snapshot_at(100_000_000 + i * 5_000_000, 50_000_000, timestamp);
    }
    let leak2 = monitor.detect_memory_leaks();
    println!("    Leak detected: {}", leak2.is_some());
    if let Some(ref leak) = leak2 {
        println!("    Leak details: {}", leak);
    }

    let stats = monitor.get_stats();
    println!("\n  Memory Statistics:");
    println!("    Peak Memory: {} MB", stats.peak_memory_bytes / 1_000_000);
    println!("    Memory Leak Events: {}", stats.memory_leak_count);

    assert!(leak2.is_some(), "Should detect memory leak");
    assert!(stats.memory_leak_count > 0, "Should record memory leak event");

    println!("\n  ✅ Memory leak detection test passed");
}

// ============================================================================
// PHASE 11 TEST 4: SYNC FAILURE TRACKING AND RECOVERY
// ============================================================================
#[test]
fn test_phase11_sync_failure_tracking() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 11 — OPERATIONAL STABILITY");
    println!("  Sync Failure Tracking and Recovery");
    println!("══════════════════════════════════════════════════════\n");

    let monitor = StabilityMonitor::new();

    // Simulate sync failures
    monitor.record_sync_failure("peer1", 1000, 900, "timeout");
    monitor.record_sync_failure("peer2", 1000, 800, "connection refused");
    monitor.record_sync_failure("peer3", 1000, 950, "invalid block");

    // Simulate recovery for some peers
    monitor.mark_sync_recovered("peer1");
    monitor.mark_sync_recovered("peer3");

    let sync_stats = monitor.get_sync_stats();
    println!("\n  Sync Statistics:");
    println!("  ─────────────────────────────────────────────");
    println!("    Total Failures: {}", sync_stats.total_failures);
    println!("    Recovered: {}", sync_stats.recovered);
    println!("    Unrecovered: {}", sync_stats.unrecovered);
    println!("    Recovery Rate: {:.1}%", sync_stats.recovery_rate * 100.0);

    assert_eq!(sync_stats.total_failures, 3, "Should have 3 sync failures");
    assert_eq!(sync_stats.recovered, 2, "Should have 2 recovered");
    assert_eq!(sync_stats.unrecovered, 1, "Should have 1 unrecovered");
    assert!(
        sync_stats.recovery_rate > 0.6,
        "Recovery rate should be > 60%"
    );

    // Record sync failure events
    monitor.record_event(
        StabilityEventType::SyncFailure,
        "Sync failure with peer2",
        false,
    );

    let stats = monitor.get_stats();
    assert_eq!(stats.sync_failure_count, 1, "Should have 1 sync failure event");

    println!("\n  ✅ Sync failure tracking test passed");
}

// ============================================================================
// PHASE 11 TEST 5: STABILITY REPORT GENERATION
// ============================================================================
#[test]
fn test_phase11_stability_report_generation() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 11 — OPERATIONAL STABILITY");
    println!("  Stability Report Generation");
    println!("══════════════════════════════════════════════════════\n");

    let monitor = StabilityMonitor::new();

    // Simulate some events
    monitor.record_event(StabilityEventType::PeerDisconnect, "Peer disconnected", true);
    monitor.record_event(StabilityEventType::SyncFailure, "Sync timeout", true);
    monitor.record_sync_failure("peer1", 100, 90, "timeout");
    monitor.record_memory_snapshot(150_000_000, 75_000_000);

    // Generate report
    let report = StabilityReport::generate(&monitor);

    println!("\n  Stability Report:");
    println!("  ─────────────────────────────────────────────");
    println!("    Generated At: {}", report.generated_at);
    println!("    Uptime: {} seconds", report.uptime_seconds);
    println!("    Stability Score: {:.1}%", report.stability_score);
    println!("    Grade: {}", report.grade);
    println!("    Crashes: {}", report.crash_count);
    println!("    Panics: {}", report.panic_count);
    println!("    Memory Leaks: {}", report.memory_leak_count);
    println!("    Sync Failures: {}", report.sync_failure_count);
    println!("    Peak Memory: {} MB", report.peak_memory_bytes / 1_000_000);
    println!("    Memory Trend: {}", report.memory_trend);
    println!("    Sync Recovery Rate: {:.1}%", report.sync_recovery_rate * 100.0);
    println!("    Critical Events: {}", report.critical_events.len());
    println!("    Recommendations: {}", report.recommendations.len());

    // Verify report structure
    assert!(report.stability_score >= 0.0);
    assert!(report.stability_score <= 100.0);
    assert!(!report.grade.is_empty());
    assert!(!report.recommendations.is_empty());

    // Verify JSON serialization
    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("stability_score"));
    assert!(json.contains("grade"));
    assert!(json.contains("recommendations"));

    // Write report to file
    std::fs::write("phase11_stability_report.json", json)
        .expect("Failed to write stability report");

    println!("\n  📄 Report saved to: phase11_stability_report.json");

    // Verify file was created
    assert!(
        std::path::Path::new("phase11_stability_report.json").exists(),
        "Stability report should be generated"
    );

    // Clean up
    let _ = std::fs::remove_file("phase11_stability_report.json");

    println!("\n  ✅ Stability report generation test passed");
}

// ============================================================================
// PHASE 11 TEST 6: CONTINUOUS OPERATION SIMULATION
// ============================================================================
#[test]
fn test_phase11_continuous_operation() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 11 — OPERATIONAL STABILITY");
    println!("  Continuous Operation Simulation");
    println!("══════════════════════════════════════════════════════\n");

    let monitor = StabilityMonitor::new();

    // Simulate 300 seconds of operation with various events
    let simulation_duration = 300u64;

    for i in 0..simulation_duration {
        let timestamp = 1000000 + i; // Incrementing timestamps
        // Memory snapshots every second
        monitor.record_memory_snapshot_at(100_000_000 + (i * 10_000), 50_000_000, timestamp);

        // Peer events every 30 seconds
        if i % 30 == 0 {
            monitor.record_event(
                StabilityEventType::PeerDisconnect,
                format!("Peer disconnect at second {}", i),
                true,
            );
        }

        // Sync failures every 60 seconds
        if i % 60 == 0 && i > 0 {
            monitor.record_sync_failure(
                format!("peer{}", i % 5),
                1000 + i,
                900 + i,
                "timeout",
            );
        }
    }

    let stats = monitor.get_stats();
    println!("\n  Operation Statistics:");
    println!("  ─────────────────────────────────────────────");
    println!("    Uptime: {} seconds", stats.uptime_seconds);
    println!("    Total Events: {}", stats.total_events);
    println!("    Memory Snapshots: {}", stats.peak_memory_bytes);
    println!("    Sync Failures: {}", stats.sync_failure_count);

    // Verify simulation ran
    assert!(stats.total_events > 0, "Should have events");
    assert!(stats.peak_memory_bytes > 0, "Should have memory snapshots");

    // Generate stability report
    let report = StabilityReport::generate(&monitor);
    println!("\n  Stability Grade: {}", report.grade);
    println!("  Stability Score: {:.1}%", report.stability_score);

    assert!(report.stability_score >= 70.0, "Should maintain stable operation");

    println!("\n  ✅ Continuous operation simulation passed");
}

// ============================================================================
// PHASE 11 TEST 7: RECOVERY SCENARIOS
// ============================================================================
#[test]
fn test_phase11_recovery_scenarios() {
    println!("\n══════════════════════════════════════════════════════");
    println!("  PHASE 11 — OPERATIONAL STABILITY");
    println!("  Recovery Scenarios");
    println!("══════════════════════════════════════════════════════\n");

    let monitor = StabilityMonitor::new();

    // Scenario 1: Peer disconnect and recovery
    println!("  Scenario 1: Peer disconnect and recovery");
    monitor.record_event(
        StabilityEventType::PeerDisconnect,
        "Peer 192.168.1.100 disconnected",
        false,
    );
    monitor.record_event(
        StabilityEventType::PeerDisconnect,
        "Peer 192.168.1.100 reconnected",
        true,
    );

    // Scenario 2: Sync failure and recovery
    println!("  Scenario 2: Sync failure and recovery");
    monitor.record_sync_failure("peer1", 1000, 900, "timeout");
    monitor.mark_sync_recovered("peer1");

    // Scenario 3: Database corruption detection
    println!("  Scenario 3: Database corruption detection");
    monitor.record_event(
        StabilityEventType::DatabaseCorruption,
        "UTXO set checksum mismatch",
        true,
    );

    // Scenario 4: Disk space warning
    println!("  Scenario 4: Disk space warning");
    monitor.record_event(
        StabilityEventType::DiskFull,
        "Disk usage at 95%",
        false,
    );

    let sync_stats = monitor.get_sync_stats();
    println!("\n  Recovery Statistics:");
    println!("  ─────────────────────────────────────────────");
    println!("    Sync Recovery Rate: {:.1}%", sync_stats.recovery_rate * 100.0);
    println!("    Total Sync Failures: {}", sync_stats.total_failures);
    println!("    Recovered: {}", sync_stats.recovered);

    assert_eq!(sync_stats.total_failures, 1);
    assert_eq!(sync_stats.recovered, 1);
    assert_eq!(sync_stats.recovery_rate, 1.0);

    let stats = monitor.get_stats();
    assert_eq!(stats.total_events, 4, "Should have 4 events");

    println!("\n  ✅ Recovery scenarios test passed");
}
