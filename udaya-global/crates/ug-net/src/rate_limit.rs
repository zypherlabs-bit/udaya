//! Per-peer inbound token buckets.
//!
//! Two buckets are enforced: one for messages and one for bytes. A message is
//! admitted only if both buckets have enough tokens, and both are debited
//! atomically. Refill is monotonic and capped at the configured burst.

use crate::{Error, Result};
use std::time::Instant;

/// Rate-limit configuration.
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub bytes_per_second: u64,
    pub bytes_burst: u64,
    pub messages_per_second: u64,
    pub messages_burst: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            bytes_per_second: 8 * 1024 * 1024,
            bytes_burst: 16 * 1024 * 1024,
            messages_per_second: 400,
            messages_burst: 800,
        }
    }
}

impl RateLimitConfig {
    pub fn validate(&self) -> Result<()> {
        if self.bytes_per_second == 0
            || self.bytes_burst == 0
            || self.messages_per_second == 0
            || self.messages_burst == 0
        {
            return Err(Error::Peer("rate limits must be non-zero".into()));
        }
        Ok(())
    }
}

struct Bucket {
    tokens: f64,
    capacity: f64,
    rate_per_second: f64,
    last: Instant,
}

impl Bucket {
    fn new(capacity: u64, rate_per_second: u64, now: Instant) -> Self {
        Self {
            tokens: capacity as f64,
            capacity: capacity as f64,
            rate_per_second: rate_per_second as f64,
            last: now,
        }
    }

    fn refill(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.rate_per_second).min(self.capacity);
            self.last = now;
        }
    }
}

/// Token-bucket limiter for one peer.
pub struct RateLimiter {
    messages: Bucket,
    bytes: Bucket,
}

impl RateLimiter {
    pub fn new(config: &RateLimitConfig) -> Result<Self> {
        config.validate()?;
        let now = Instant::now();
        Ok(Self {
            messages: Bucket::new(config.messages_burst, config.messages_per_second, now),
            bytes: Bucket::new(config.bytes_burst, config.bytes_per_second, now),
        })
    }

    /// Check and debit one inbound message.
    pub fn check_message(&mut self, bytes: usize) -> Result<()> {
        self.check_message_at(bytes, Instant::now())
    }

    /// Deterministic-time variant used by tests.
    pub fn check_message_at(&mut self, bytes: usize, now: Instant) -> Result<()> {
        self.messages.refill(now);
        self.bytes.refill(now);
        if self.messages.tokens < 1.0 || self.bytes.tokens < bytes as f64 {
            return Err(Error::RateLimited);
        }
        self.messages.tokens -= 1.0;
        self.bytes.tokens -= bytes as f64;
        Ok(())
    }

    /// Remaining message tokens, for diagnostics.
    pub fn message_tokens(&self) -> f64 {
        self.messages.tokens
    }

    /// Remaining byte tokens, for diagnostics.
    pub fn byte_tokens(&self) -> f64 {
        self.bytes.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn config() -> RateLimitConfig {
        RateLimitConfig {
            bytes_per_second: 100,
            bytes_burst: 100,
            messages_per_second: 2,
            messages_burst: 2,
        }
    }

    #[test]
    fn bursts_are_bounded_and_refill_over_time() {
        let mut limiter = RateLimiter::new(&config()).unwrap();
        let start = Instant::now();
        limiter.check_message_at(10, start).unwrap();
        limiter.check_message_at(10, start).unwrap();
        assert!(matches!(
            limiter.check_message_at(10, start).unwrap_err(),
            Error::RateLimited
        ));

        limiter
            .check_message_at(10, start + Duration::from_secs(1))
            .unwrap();
        limiter
            .check_message_at(10, start + Duration::from_secs(1))
            .unwrap();
        assert!(limiter
            .check_message_at(10, start + Duration::from_secs(1))
            .is_err());
    }

    #[test]
    fn byte_bucket_is_independent_and_capped() {
        let mut limiter = RateLimiter::new(&config()).unwrap();
        let start = Instant::now();
        assert!(limiter.check_message_at(101, start).is_err());
        assert_eq!(limiter.message_tokens(), 2.0);
        limiter.check_message_at(60, start).unwrap();
        assert!(limiter.check_message_at(50, start).is_err());

        // Refill is capped at the burst size, never above it.
        limiter
            .check_message_at(1, start + Duration::from_secs(3600))
            .unwrap();
        assert!(limiter.byte_tokens() <= 100.0);
    }

    #[test]
    fn zero_limits_are_rejected() {
        let mut config = config();
        config.messages_per_second = 0;
        assert!(RateLimiter::new(&config).is_err());
    }
}
