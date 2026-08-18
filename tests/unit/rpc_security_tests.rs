use dashmap::DashMap;
use parking_lot::Mutex;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use udaya_core::config::UdayaConfig;

// ============================================================================
// RateLimiterState unit (mirrors the one in main.rs)
// ============================================================================

struct RateLimiterState {
    requests: DashMap<IpAddr, (AtomicU64, Mutex<Instant>)>,
}

impl RateLimiterState {
    fn new() -> Self {
        Self {
            requests: DashMap::new(),
        }
    }

    fn check_rate_limit(&self, ip: IpAddr, _max_rps: u32, burst: u32) -> bool {
        let now = Instant::now();
        let entry = self
            .requests
            .entry(ip)
            .or_insert_with(|| (AtomicU64::new(0), Mutex::new(now)));
        let (count, last_reset_mutex) = entry.value();

        let mut last_reset = last_reset_mutex.lock();
        let elapsed = now.duration_since(*last_reset).as_secs();
        if elapsed >= 1 {
            count.store(0, Ordering::SeqCst);
            *last_reset = now;
        }
        drop(last_reset);

        let current = count.fetch_add(1, Ordering::SeqCst) + 1;
        if current > burst as u64 {
            return false;
        }
        true
    }
}

// ============================================================================
// Rate limiter tests
// ============================================================================

#[test]
fn test_rate_limiter_allows_requests_under_burst() {
    let limiter = RateLimiterState::new();
    let ip = IpAddr::from([192, 168, 1, 1]);

    for _ in 0..5 {
        assert!(limiter.check_rate_limit(ip, 10, 10));
    }
}

#[test]
fn test_rate_limiter_blocks_over_burst() {
    let limiter = RateLimiterState::new();
    let ip = IpAddr::from([192, 168, 1, 2]);

    for _ in 0..5 {
        assert!(limiter.check_rate_limit(ip, 10, 5));
    }
    assert!(!limiter.check_rate_limit(ip, 10, 5));
}

#[test]
fn test_rate_limiter_resets_after_window() {
    let limiter = RateLimiterState::new();
    let ip = IpAddr::from([192, 168, 1, 3]);

    for _ in 0..3 {
        assert!(limiter.check_rate_limit(ip, 10, 3));
    }
    assert!(!limiter.check_rate_limit(ip, 10, 3));

    // Manually advance the last_reset time by 2 seconds
    {
        let entry = limiter.requests.entry(ip).or_insert_with(|| {
            let now = Instant::now();
            (AtomicU64::new(0), Mutex::new(now - Duration::from_secs(2)))
        });
        let (count, last_reset_mutex) = entry.value();
        let mut last_reset = last_reset_mutex.lock();
        *last_reset = Instant::now() - Duration::from_secs(2);
        count.store(0, Ordering::SeqCst);
    }

    assert!(limiter.check_rate_limit(ip, 10, 3));
}

#[test]
fn test_rate_limiter_per_ip_isolation() {
    let limiter = RateLimiterState::new();
    let ip1 = IpAddr::from([192, 168, 1, 4]);
    let ip2 = IpAddr::from([192, 168, 1, 5]);

    for _ in 0..5 {
        assert!(limiter.check_rate_limit(ip1, 10, 5));
    }
    assert!(!limiter.check_rate_limit(ip1, 10, 5));
    assert!(limiter.check_rate_limit(ip2, 10, 5));
}

// ============================================================================
// Basic auth parsing tests
// ============================================================================

fn parse_basic_auth(auth_header: &str) -> Option<(String, String)> {
    if !auth_header.starts_with("Basic ") {
        return None;
    }
    let decoded = match base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &auth_header[6..],
    ) {
        Ok(d) => d,
        Err(_) => return None,
    };
    let credentials = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => return None,
    };
    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    Some((parts[0].to_string(), parts[1].to_string()))
}

#[test]
fn test_parse_valid_basic_auth() {
    let encoded =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, "user:password");
    let result = parse_basic_auth(&format!("Basic {}", encoded));
    assert_eq!(result, Some(("user".to_string(), "password".to_string())));
}

#[test]
fn test_parse_invalid_basic_auth_no_basic_prefix() {
    let result = parse_basic_auth("Bearer token123");
    assert!(result.is_none());
}

#[test]
fn test_parse_invalid_basic_auth_bad_base64() {
    let result = parse_basic_auth("Basic not-valid-base64!!!");
    assert!(result.is_none());
}

#[test]
fn test_parse_invalid_basic_auth_no_colon() {
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, "nopassword");
    let result = parse_basic_auth(&format!("Basic {}", encoded));
    assert!(result.is_none());
}

// ============================================================================
// Config parsing tests for RPC security fields
// ============================================================================

#[test]
fn test_rpc_security_config_defaults() {
    let config = UdayaConfig::default();
    assert!(!config.rpc.enable_tls);
    assert!(!config.rpc.enable_auth);
    assert_eq!(config.rpc.rate_limit_rps, 100);
    assert_eq!(config.rpc.rate_limit_burst, 200);
    assert_eq!(config.rpc.max_request_size_mb, 10);
    assert_eq!(config.rpc.max_connections, 500);
    assert_eq!(
        config.rpc.restricted_methods,
        vec![
            "stop".to_string(),
            "generate".to_string(),
            "invalidateblock".to_string(),
            "reconsiderblock".to_string(),
        ]
    );
}

#[test]
fn test_restricted_methods_config() {
    let mut config = UdayaConfig::default();
    config.rpc.restricted_methods = vec![
        "stop".to_string(),
        "restart".to_string(),
        "upgrade".to_string(),
    ];
    assert!(config.rpc.restricted_methods.contains(&"stop".to_string()));
    assert!(config
        .rpc
        .restricted_methods
        .contains(&"restart".to_string()));
}

// ============================================================================
// Restricted method matching tests
// ============================================================================

fn is_restricted_method(method: &str, restricted: &[String]) -> bool {
    restricted.iter().any(|m| m == method)
}

#[test]
fn test_restricted_method_detection() {
    let restricted = vec!["stop".to_string(), "restart".to_string()];
    assert!(is_restricted_method("stop", &restricted));
    assert!(is_restricted_method("restart", &restricted));
    assert!(!is_restricted_method("getblockchaininfo", &restricted));
}

#[test]
fn test_restricted_method_case_sensitive() {
    let restricted = vec!["stop".to_string()];
    assert!(is_restricted_method("stop", &restricted));
    assert!(!is_restricted_method("STOP", &restricted));
    assert!(!is_restricted_method("Stop", &restricted));
}

// ============================================================================
// Request size limit tests
// ============================================================================

fn check_request_size(body_len: usize, max_mb: u64) -> bool {
    let max_bytes = (max_mb as usize) * 1024 * 1024;
    body_len <= max_bytes
}

#[test]
fn test_request_size_under_limit() {
    assert!(check_request_size(1024, 10));
    assert!(check_request_size(1024 * 1024, 1));
}

#[test]
fn test_request_size_over_limit() {
    assert!(!check_request_size((11 * 1024 * 1024), 10));
}

#[test]
fn test_request_size_exact_limit() {
    assert!(check_request_size(10 * 1024 * 1024, 10));
}
