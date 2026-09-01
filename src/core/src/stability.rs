//! Operational Stability Monitoring
//!
//! Tracks crashes, memory leaks, sync failures, and generates stability reports.

use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of events to keep in history
const MAX_EVENT_HISTORY: usize = 10_000;

/// Maximum number of memory snapshots to keep
const MAX_MEMORY_SNAPSHOTS: usize = 1000;

/// Stability event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum StabilityEventType {
    Crash,
    Panic,
    MemoryLeak,
    SyncFailure,
    PeerDisconnect,
    BlockValidationError,
    DatabaseCorruption,
    DiskFull,
    CpuSpike,
    NetworkTimeout,
    StartupFailure,
    ShutdownFailure,
}

impl StabilityEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StabilityEventType::Crash => "crash",
            StabilityEventType::Panic => "panic",
            StabilityEventType::MemoryLeak => "memory_leak",
            StabilityEventType::SyncFailure => "sync_failure",
            StabilityEventType::PeerDisconnect => "peer_disconnect",
            StabilityEventType::BlockValidationError => "block_validation_error",
            StabilityEventType::DatabaseCorruption => "database_corruption",
            StabilityEventType::DiskFull => "disk_full",
            StabilityEventType::CpuSpike => "cpu_spike",
            StabilityEventType::NetworkTimeout => "network_timeout",
            StabilityEventType::StartupFailure => "startup_failure",
            StabilityEventType::ShutdownFailure => "shutdown_failure",
        }
    }

    pub fn severity(&self) -> Severity {
        match self {
            StabilityEventType::Crash
            | StabilityEventType::Panic
            | StabilityEventType::DatabaseCorruption => Severity::Critical,
            StabilityEventType::MemoryLeak
            | StabilityEventType::SyncFailure
            | StabilityEventType::DiskFull => Severity::High,
            StabilityEventType::BlockValidationError
            | StabilityEventType::StartupFailure
            | StabilityEventType::ShutdownFailure => Severity::Medium,
            _ => Severity::Low,
        }
    }
}

/// Event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }
}

/// A stability event record
#[derive(Debug, Clone, serde::Serialize)]
pub struct StabilityEvent {
    pub event_type: StabilityEventType,
    pub timestamp: u64,
    pub message: String,
    pub severity: Severity,
    pub recovered: bool,
}

/// Memory usage snapshot for leak detection
#[derive(Debug, Clone, serde::Serialize)]
pub struct MemorySnapshot {
    pub timestamp: u64,
    pub rss_bytes: u64,
    pub heap_bytes: u64,
}

/// Sync failure record
#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncFailure {
    pub timestamp: u64,
    pub peer: String,
    pub height_expected: u64,
    pub height_actual: u64,
    pub error: String,
    pub recovered: bool,
}

/// Crash recovery record
#[derive(Debug, Clone, serde::Serialize)]
pub struct CrashRecord {
    pub timestamp: u64,
    pub cause: String,
    pub stack_trace: String,
    pub auto_restarted: bool,
    pub recovery_time_secs: u64,
}

/// Operational stability monitor
pub struct StabilityMonitor {
    events: Arc<RwLock<VecDeque<StabilityEvent>>>,
    memory_snapshots: Arc<RwLock<VecDeque<MemorySnapshot>>>,
    sync_failures: Arc<RwLock<VecDeque<SyncFailure>>>,
    crash_records: Arc<RwLock<VecDeque<CrashRecord>>>,
    start_time: Arc<RwLock<u64>>,
    crash_count: Arc<RwLock<u64>>,
    panic_count: Arc<RwLock<u64>>,
    memory_leak_count: Arc<RwLock<u64>>,
    sync_failure_count: Arc<RwLock<u64>>,
    last_memory_check: Arc<RwLock<u64>>,
    peak_memory_bytes: Arc<RwLock<u64>>,
}

impl StabilityMonitor {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            events: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_EVENT_HISTORY))),
            memory_snapshots: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_MEMORY_SNAPSHOTS))),
            sync_failures: Arc::new(RwLock::new(VecDeque::new())),
            crash_records: Arc::new(RwLock::new(VecDeque::new())),
            start_time: Arc::new(RwLock::new(now)),
            crash_count: Arc::new(RwLock::new(0)),
            panic_count: Arc::new(RwLock::new(0)),
            memory_leak_count: Arc::new(RwLock::new(0)),
            sync_failure_count: Arc::new(RwLock::new(0)),
            last_memory_check: Arc::new(RwLock::new(now)),
            peak_memory_bytes: Arc::new(RwLock::new(0)),
        }
    }

    /// Record a stability event
    pub fn record_event(
        &self,
        event_type: StabilityEventType,
        message: impl Into<String>,
        recovered: bool,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let event = StabilityEvent {
            event_type,
            timestamp: now,
            message: message.into(),
            severity: event_type.severity(),
            recovered,
        };

        let mut events = self.events.write();
        events.push_back(event);

        while events.len() > MAX_EVENT_HISTORY {
            events.pop_front();
        }

        match event_type {
            StabilityEventType::Crash => {
                *self.crash_count.write() += 1;
            }
            StabilityEventType::Panic => {
                *self.panic_count.write() += 1;
            }
            StabilityEventType::MemoryLeak => {
                *self.memory_leak_count.write() += 1;
            }
            StabilityEventType::SyncFailure => {
                *self.sync_failure_count.write() += 1;
            }
            _ => {}
        }
    }

    /// Record a sync failure
    pub fn record_sync_failure(
        &self,
        peer: impl Into<String>,
        height_expected: u64,
        height_actual: u64,
        error: impl Into<String>,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let failure = SyncFailure {
            timestamp: now,
            peer: peer.into(),
            height_expected,
            height_actual,
            error: error.into(),
            recovered: false,
        };

        self.sync_failures.write().push_back(failure);
    }

    /// Mark a sync failure as recovered
    pub fn mark_sync_recovered(&self, peer: &str) {
        let mut failures = self.sync_failures.write();
        if let Some(failure) = failures.iter_mut().find(|f| f.peer == peer && !f.recovered) {
            failure.recovered = true;
        }
    }

    /// Record a crash
    pub fn record_crash(
        &self,
        cause: impl Into<String>,
        stack_trace: impl Into<String>,
        auto_restarted: bool,
        recovery_time_secs: u64,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let record = CrashRecord {
            timestamp: now,
            cause: cause.into(),
            stack_trace: stack_trace.into(),
            auto_restarted,
            recovery_time_secs,
        };

        self.crash_records.write().push_back(record);
    }

    /// Record a memory snapshot for leak detection
    pub fn record_memory_snapshot(&self, rss_bytes: u64, heap_bytes: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.record_memory_snapshot_at(rss_bytes, heap_bytes, now);
    }

    /// Record a memory snapshot with a specific timestamp (for testing)
    pub fn record_memory_snapshot_at(&self, rss_bytes: u64, heap_bytes: u64, timestamp: u64) {
        let snapshot = MemorySnapshot {
            timestamp,
            rss_bytes,
            heap_bytes,
        };

        let mut snapshots = self.memory_snapshots.write();
        snapshots.push_back(snapshot);

        while snapshots.len() > MAX_MEMORY_SNAPSHOTS {
            snapshots.pop_front();
        }

        if rss_bytes > *self.peak_memory_bytes.read() {
            *self.peak_memory_bytes.write() = rss_bytes;
        }

        *self.last_memory_check.write() = timestamp;
    }

    /// Detect memory leaks by analyzing memory growth trend
    pub fn detect_memory_leaks(&self) -> Option<String> {
        let snapshots = self.memory_snapshots.read();
        if snapshots.len() < 10 {
            return None;
        }

        let first = snapshots.front()?;
        let last = snapshots.back()?;

        let duration_secs = (last.timestamp - first.timestamp) as f64;
        let num_snapshots = snapshots.len() as f64;

        // Use actual time duration if available, otherwise estimate from snapshot count
        let effective_duration = if duration_secs > 0.0 {
            duration_secs
        } else {
            num_snapshots // Assume 1 snapshot per second
        };

        let growth_rate = (last.rss_bytes as f64 - first.rss_bytes as f64) / effective_duration;

        // If memory grows more than 1MB per minute continuously, flag as leak
        let threshold = 1_000_000.0 / 60.0;
        if growth_rate > threshold {
            let msg = format!(
                "Potential memory leak detected: {:.2} MB/min growth rate",
                growth_rate / 1_000_000.0 * 60.0
            );
            self.record_event(StabilityEventType::MemoryLeak, &msg, false);
            *self.memory_leak_count.write() += 1;
            return Some(msg);
        }

        None
    }

    /// Get stability statistics
    pub fn get_stats(&self) -> StabilityStats {
        let events = self.events.read();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let start = *self.start_time.read();
        let uptime = now.saturating_sub(start);

        StabilityStats {
            uptime_seconds: uptime,
            total_events: events.len(),
            crash_count: *self.crash_count.read(),
            panic_count: *self.panic_count.read(),
            memory_leak_count: *self.memory_leak_count.read(),
            sync_failure_count: *self.sync_failure_count.read(),
            peak_memory_bytes: *self.peak_memory_bytes.read(),
            events_by_type: self.count_events_by_type(&events),
            recent_events: events.iter().rev().take(10).cloned().collect(),
        }
    }

    /// Get sync failure statistics
    pub fn get_sync_stats(&self) -> SyncStats {
        let failures = self.sync_failures.read();
        let total = failures.len();
        let recovered = failures.iter().filter(|f| f.recovered).count();
        let unrecovered = total - recovered;

        SyncStats {
            total_failures: total,
            recovered,
            unrecovered,
            recovery_rate: if total > 0 {
                recovered as f64 / total as f64
            } else {
                1.0
            },
        }
    }

    /// Count events by type
    fn count_events_by_type(&self, events: &VecDeque<StabilityEvent>) -> std::collections::HashMap<String, u64> {
        let mut counts = std::collections::HashMap::new();
        for event in events {
            *counts.entry(event.event_type.as_str().to_string()).or_insert(0) += 1;
        }
        counts
    }

    /// Reset all stability counters (for testing)
    pub fn reset(&self) {
        self.events.write().clear();
        self.memory_snapshots.write().clear();
        self.sync_failures.write().clear();
        self.crash_records.write().clear();
        *self.crash_count.write() = 0;
        *self.panic_count.write() = 0;
        *self.memory_leak_count.write() = 0;
        *self.sync_failure_count.write() = 0;
        *self.peak_memory_bytes.write() = 0;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        *self.start_time.write() = now;
    }
}

impl Default for StabilityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Stability statistics summary
#[derive(Debug, Clone, serde::Serialize)]
pub struct StabilityStats {
    pub uptime_seconds: u64,
    pub total_events: usize,
    pub crash_count: u64,
    pub panic_count: u64,
    pub memory_leak_count: u64,
    pub sync_failure_count: u64,
    pub peak_memory_bytes: u64,
    pub events_by_type: std::collections::HashMap<String, u64>,
    pub recent_events: Vec<StabilityEvent>,
}

/// Sync failure statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncStats {
    pub total_failures: usize,
    pub recovered: usize,
    pub unrecovered: usize,
    pub recovery_rate: f64,
}

/// Operational stability report
#[derive(Debug, Clone, serde::Serialize)]
pub struct StabilityReport {
    pub generated_at: u64,
    pub uptime_seconds: u64,
    pub stability_score: f64,
    pub crash_count: u64,
    pub panic_count: u64,
    pub memory_leak_count: u64,
    pub sync_failure_count: u64,
    pub peak_memory_bytes: u64,
    pub memory_trend: String,
    pub sync_recovery_rate: f64,
    pub critical_events: Vec<StabilityEvent>,
    pub recommendations: Vec<String>,
    pub grade: String,
}

impl StabilityReport {
    pub fn generate(monitor: &StabilityMonitor) -> Self {
        let stats = monitor.get_stats();
        let sync_stats = monitor.get_sync_stats();

        let mut critical_events: Vec<StabilityEvent> = stats
            .recent_events
            .iter()
            .filter(|e| e.severity == Severity::Critical)
            .cloned()
            .collect();

        let mut recommendations = Vec::new();

        if stats.crash_count > 0 {
            recommendations.push(format!(
                "Investigate {} crashes - review panic logs and core dumps",
                stats.crash_count
            ));
        }

        if stats.panic_count > 0 {
            recommendations.push(format!(
                "Address {} panics - review panic hooks and error handling",
                stats.panic_count
            ));
        }

        if stats.memory_leak_count > 0 {
            recommendations.push("Memory leaks detected - run valgrind/ASan analysis".to_string());
        }

        if sync_stats.recovery_rate < 0.95 {
            recommendations.push(format!(
                "Sync recovery rate is {:.1}% - improve peer reconnection logic",
                sync_stats.recovery_rate * 100.0
            ));
        }

        if critical_events.is_empty() && stats.crash_count == 0 && stats.panic_count == 0 {
            recommendations.push("No critical issues detected - continue monitoring".to_string());
        }

        let memory_trend = if let Some(leak) = monitor.detect_memory_leaks() {
            format!("Leak detected: {}", leak)
        } else {
            "Stable".to_string()
        };

        let stability_score = Self::calculate_score(&stats, &sync_stats);
        let grade = Self::grade_score(stability_score);

        if critical_events.is_empty() {
            critical_events = stats
                .recent_events
                .iter()
                .filter(|e| e.severity == Severity::High)
                .cloned()
                .collect();
        }

        Self {
            generated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            uptime_seconds: stats.uptime_seconds,
            stability_score,
            crash_count: stats.crash_count,
            panic_count: stats.panic_count,
            memory_leak_count: stats.memory_leak_count,
            sync_failure_count: stats.sync_failure_count,
            peak_memory_bytes: stats.peak_memory_bytes,
            memory_trend,
            sync_recovery_rate: sync_stats.recovery_rate,
            critical_events,
            recommendations,
            grade,
        }
    }

    fn calculate_score(stats: &StabilityStats, sync_stats: &SyncStats) -> f64 {
        let mut score = 100.0;

        score -= stats.crash_count as f64 * 10.0;
        score -= stats.panic_count as f64 * 15.0;
        score -= stats.memory_leak_count as f64 * 5.0;
        score -= (1.0 - sync_stats.recovery_rate) * 20.0;

        score.clamp(0.0, 100.0)
    }

    fn grade_score(score: f64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stability_monitor_creation() {
        let monitor = StabilityMonitor::new();
        let stats = monitor.get_stats();
        assert_eq!(stats.crash_count, 0);
        assert_eq!(stats.panic_count, 0);
        assert_eq!(stats.total_events, 0);
    }

    #[test]
    fn test_record_event() {
        let monitor = StabilityMonitor::new();
        monitor.record_event(StabilityEventType::Crash, "Test crash", true);
        let stats = monitor.get_stats();
        assert_eq!(stats.crash_count, 1);
        assert_eq!(stats.total_events, 1);
    }

    #[test]
    fn test_record_sync_failure() {
        let monitor = StabilityMonitor::new();
        monitor.record_sync_failure("peer1", 100, 50, "timeout");
        let sync_stats = monitor.get_sync_stats();
        assert_eq!(sync_stats.total_failures, 1);
        assert_eq!(sync_stats.recovered, 0);
    }

    #[test]
    fn test_memory_leak_detection() {
        let monitor = StabilityMonitor::new();
        for i in 0..20 {
            monitor.record_memory_snapshot(100_000_000 + i * 2_000_000, 50_000_000);
        }
        let leak = monitor.detect_memory_leaks();
        // With 2MB growth per snapshot, we should detect a leak
        assert!(leak.is_some());
    }

    #[test]
    fn test_stability_report_generation() {
        let monitor = StabilityMonitor::new();
        monitor.record_event(StabilityEventType::Crash, "Test", true);
        let report = StabilityReport::generate(&monitor);
        assert!(report.stability_score < 100.0);
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_event_history_limit() {
        let monitor = StabilityMonitor::new();
        for i in 0..MAX_EVENT_HISTORY + 100 {
            monitor.record_event(StabilityEventType::PeerDisconnect, format!("Event {}", i), true);
        }
        let stats = monitor.get_stats();
        assert_eq!(stats.total_events, MAX_EVENT_HISTORY);
    }

    #[test]
    fn test_reset() {
        let monitor = StabilityMonitor::new();
        monitor.record_event(StabilityEventType::Crash, "Test", true);
        monitor.record_sync_failure("peer1", 100, 50, "timeout");
        monitor.reset();
        let stats = monitor.get_stats();
        assert_eq!(stats.crash_count, 0);
        assert_eq!(stats.total_events, 0);
    }
}
