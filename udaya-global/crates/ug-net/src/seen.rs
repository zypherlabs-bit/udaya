//! Bounded duplicate-suppression cache.
//!
//! Transaction and block propagation must not relay the same object forever.
//! A bounded, time-limited cache records recently seen object hashes. It is an
//! overlay only: the mempool and block store remain the authoritative
//! deduplicators, and a cache miss never causes an object to be accepted that
//! the node would otherwise reject.

use ug_core::crypto::Hash32;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Default number of recently seen hashes retained.
pub const DEFAULT_SEEN_CAPACITY: usize = 100_000;

/// Default time a seen hash is retained.
pub const DEFAULT_SEEN_TTL: Duration = Duration::from_secs(3_600);

struct Entry {
    hash: Hash32,
    expires: Instant,
}

/// A bounded seen-set with expiry.
#[derive(Clone, Debug)]
pub struct SeenCache {
    capacity: usize,
    ttl: Duration,
    entries: VecDeque<Entry>,
    index: HashMap<Hash32, usize>,
}

impl SeenCache {
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            capacity: capacity.max(1),
            ttl,
            entries: VecDeque::new(),
            index: HashMap::new(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Record a hash at a point in time. Returns `true` when the hash was not
    /// already present (or had expired).
    pub fn mark(&mut self, hash: &Hash32) -> bool {
        self.mark_at(hash, Instant::now())
    }

    /// Deterministic-time variant used by tests.
    pub fn mark_at(&mut self, hash: &Hash32, now: Instant) -> bool {
        self.evict_expired(now);
        if self.index.contains_key(hash) {
            return false;
        }
        self.entries.push_back(Entry {
            hash: *hash,
            expires: now + self.ttl,
        });
        self.index.insert(*hash, self.entries.len() - 1);
        while self.entries.len() > self.capacity {
            self.pop_oldest();
        }
        true
    }

    /// Whether the hash is currently recorded.
    pub fn contains(&mut self, hash: &Hash32) -> bool {
        self.contains_at(hash, Instant::now())
    }

    /// Deterministic-time variant used by tests.
    pub fn contains_at(&mut self, hash: &Hash32, now: Instant) -> bool {
        self.evict_expired(now);
        self.index.contains_key(hash)
    }

    fn evict_expired(&mut self, now: Instant) {
        while let Some(front) = self.entries.front() {
            if front.expires <= now {
                let hash = front.hash;
                self.entries.pop_front();
                self.index.remove(&hash);
                self.reindex();
            } else {
                break;
            }
        }
    }

    fn pop_oldest(&mut self) {
        if let Some(entry) = self.entries.pop_front() {
            self.index.remove(&entry.hash);
            self.reindex();
        }
    }

    /// The deque order is the source of truth for eviction. Positions are
    /// rebuilt lazily only after a removal from the front.
    fn reindex(&mut self) {
        self.index.clear();
        for (position, entry) in self.entries.iter().enumerate() {
            self.index.insert(entry.hash, position);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn hash(seed: u8) -> Hash32 {
        [seed; 32]
    }

    #[test]
    fn first_sighting_is_new_and_repeats_are_duplicates() {
        let mut cache = SeenCache::new(8, Duration::from_secs(60));
        assert!(cache.mark(&hash(1)));
        assert!(!cache.mark(&hash(1)));
        assert!(cache.contains(&hash(1)));
        assert!(cache.mark(&hash(2)));
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn entries_expire() {
        let mut cache = SeenCache::new(8, Duration::from_secs(10));
        let start = Instant::now();
        assert!(cache.mark_at(&hash(1), start));
        assert!(cache.contains_at(&hash(1), start + Duration::from_secs(5)));
        assert!(!cache.contains_at(&hash(1), start + Duration::from_secs(11)));
        assert!(cache.mark_at(&hash(1), start + Duration::from_secs(11)));
    }

    #[test]
    fn capacity_is_bounded() {
        let mut cache = SeenCache::new(4, Duration::from_secs(60));
        for seed in 0..10 {
            cache.mark(&hash(seed));
        }
        assert_eq!(cache.len(), 4);
        assert!(!cache.contains(&hash(0)));
        assert!(!cache.contains(&hash(1)));
        assert!(cache.contains(&hash(9)));
    }

    #[test]
    fn zero_capacity_is_clamped() {
        let cache = SeenCache::new(0, Duration::from_secs(60));
        assert_eq!(cache.capacity(), 1);
    }
}
