//! Persistent peer store and bounded Kademlia-style routing table.
//!
//! The store is local metadata, not consensus data. It is still written
//! atomically (temporary file, fsync, rename) and validated strictly on load.
//! Peers that repeatedly fail are put into exponential backoff and eventually
//! banned. The routing table keeps at most `K` peers per XOR-distance bucket.

use crate::identity::PeerId;
use crate::{Error, Result, MAX_PEER_STORE_BYTES};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::Path;
use ug_core::crypto::Hash32;

/// Maximum peers per routing-table bucket.
pub const ROUTING_K: usize = 20;

pub const MAX_STORED_PEERS: usize = 4096;

/// One persisted peer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerRecord {
    #[serde(with = "hex32")]
    pub peer_id: Hash32,
    pub address: String,
    pub last_seen_unix: u64,
    pub failures: u32,
    pub banned: bool,
}

impl PeerRecord {
    pub fn peer_id(&self) -> PeerId {
        PeerId(self.peer_id)
    }

    pub fn socket_addr(&self) -> Result<SocketAddr> {
        self.address
            .parse()
            .map_err(|_| Error::Store(format!("invalid peer address {}", self.address)))
    }
}

/// Peer store policy.
#[derive(Clone, Debug)]
pub struct PeerStoreConfig {
    pub max_peers: usize,
    pub base_backoff_secs: u64,
    pub max_backoff_secs: u64,
    pub ban_after_failures: u32,
}

impl Default for PeerStoreConfig {
    fn default() -> Self {
        Self {
            max_peers: MAX_STORED_PEERS,
            base_backoff_secs: 2,
            max_backoff_secs: 300,
            ban_after_failures: 8,
        }
    }
}

impl PeerStoreConfig {
    fn validate(&self) -> Result<()> {
        if self.max_peers == 0 || self.max_peers > MAX_STORED_PEERS {
            return Err(Error::Store("invalid peer store capacity".into()));
        }
        if self.base_backoff_secs == 0
            || self.max_backoff_secs < self.base_backoff_secs
            || self.ban_after_failures == 0
        {
            return Err(Error::Store("invalid peer backoff policy".into()));
        }
        Ok(())
    }
}

/// Persistent peer metadata.
#[derive(Clone, Debug)]
pub struct PeerStore {
    config: PeerStoreConfig,
    peers: BTreeMap<Hash32, PeerRecord>,
}

impl PeerStore {
    pub fn new(config: PeerStoreConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config,
            peers: BTreeMap::new(),
        })
    }

    pub fn config(&self) -> &PeerStoreConfig {
        &self.config
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    pub fn get(&self, peer: &PeerId) -> Option<&PeerRecord> {
        self.peers.get(&peer.0)
    }

    pub fn iter(&self) -> impl Iterator<Item = &PeerRecord> {
        self.peers.values()
    }

    /// Record a successful connection or handshake.
    pub fn record_success(
        &mut self,
        peer: PeerId,
        address: SocketAddr,
        now_unix: u64,
    ) -> Result<()> {
        if peer.0 == [0u8; 32] {
            return Err(Error::Store("zero peer id".into()));
        }
        if !self.peers.contains_key(&peer.0) && self.peers.len() >= self.config.max_peers {
            self.evict_one();
        }
        let record = self.peers.entry(peer.0).or_insert_with(|| PeerRecord {
            peer_id: peer.0,
            address: address.to_string(),
            last_seen_unix: now_unix,
            failures: 0,
            banned: false,
        });
        record.address = address.to_string();
        record.last_seen_unix = now_unix;
        record.failures = 0;
        Ok(())
    }

    /// Record a failed attempt. Repeated failures trigger exponential backoff
    /// and, after the configured threshold, a ban.
    pub fn record_failure(&mut self, peer: &PeerId, now_unix: u64) -> Result<()> {
        let Some(record) = self.peers.get_mut(&peer.0) else {
            return Err(Error::Store("unknown peer".into()));
        };
        record.failures = record.failures.saturating_add(1);
        record.last_seen_unix = now_unix;
        if record.failures >= self.config.ban_after_failures {
            record.banned = true;
        }
        Ok(())
    }

    pub fn ban(&mut self, peer: &PeerId) -> Result<()> {
        let Some(record) = self.peers.get_mut(&peer.0) else {
            return Err(Error::Store("unknown peer".into()));
        };
        record.banned = true;
        Ok(())
    }

    pub fn unban(&mut self, peer: &PeerId) -> Result<()> {
        let Some(record) = self.peers.get_mut(&peer.0) else {
            return Err(Error::Store("unknown peer".into()));
        };
        record.banned = false;
        record.failures = 0;
        Ok(())
    }

    pub fn remove(&mut self, peer: &PeerId) -> Option<PeerRecord> {
        self.peers.remove(&peer.0)
    }

    /// Seconds until this peer may be retried. Banned peers are never due.
    pub fn retry_delay_secs(&self, peer: &PeerId) -> Option<u64> {
        let record = self.peers.get(&peer.0)?;
        if record.banned {
            return None;
        }
        if record.failures == 0 {
            return Some(0);
        }
        let exponent = record.failures.saturating_sub(1).min(20);
        let delay = self
            .config
            .base_backoff_secs
            .saturating_mul(1u64 << exponent)
            .min(self.config.max_backoff_secs);
        Some(delay)
    }

    /// Whether a peer is due for an attempt at `now_unix`.
    pub fn is_due(&self, peer: &PeerId, now_unix: u64) -> bool {
        let Some(record) = self.peers.get(&peer.0) else {
            return false;
        };
        if record.banned {
            return false;
        }
        let Some(delay) = self.retry_delay_secs(peer) else {
            return false;
        };
        now_unix >= record.last_seen_unix.saturating_add(delay)
    }

    fn evict_one(&mut self) {
        let victim = self
            .peers
            .values()
            .filter(|record| !record.banned)
            .min_by_key(|record| record.last_seen_unix)
            .map(|record| record.peer_id)
            .or_else(|| self.peers.values().next().map(|record| record.peer_id));
        if let Some(victim) = victim {
            self.peers.remove(&victim);
        }
    }

    /// Atomic persistent save: temp file, fsync, rename.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let peers: Vec<&PeerRecord> = self.peers.values().collect();
        let json = serde_json::to_vec_pretty(&peers)
            .map_err(|error| Error::Store(format!("serialize peer store: {error}")))?;
        if json.len() as u64 > MAX_PEER_STORE_BYTES {
            return Err(Error::Store("peer store is too large".into()));
        }
        let tmp = path.with_extension("json.tmp");
        {
            use std::io::Write;
            let mut file = std::fs::File::create(&tmp)?;
            file.write_all(&json)?;
            file.sync_all()?;
        }
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Load a peer store, validating size, uniqueness, addresses and capacity.
    pub fn load(path: &Path, config: PeerStoreConfig) -> Result<Self> {
        let mut store = Self::new(config)?;
        if !path.exists() {
            return Ok(store);
        }
        let meta = std::fs::metadata(path)?;
        if meta.len() > MAX_PEER_STORE_BYTES {
            return Err(Error::Store("peer store file is too large".into()));
        }
        let data = std::fs::read(path)?;
        let records: Vec<PeerRecord> = serde_json::from_slice(&data)
            .map_err(|error| Error::Store(format!("parse peer store: {error}")))?;
        if records.len() > store.config.max_peers {
            return Err(Error::Store("peer store exceeds its capacity".into()));
        }
        for record in records {
            if record.peer_id == [0u8; 32] {
                return Err(Error::Store("zero peer id in store".into()));
            }
            record.socket_addr()?;
            if store.peers.contains_key(&record.peer_id) {
                return Err(Error::Store("duplicate peer id in store".into()));
            }
            store.peers.insert(record.peer_id, record);
        }
        Ok(store)
    }
}

/// A bounded Kademlia-style routing table.
#[derive(Clone, Debug)]
pub struct RoutingTable {
    local: PeerId,
    k: usize,
    buckets: Vec<Vec<PeerId>>,
}

impl RoutingTable {
    pub fn new(local: PeerId, k: usize) -> Result<Self> {
        if k == 0 || k > 100 {
            return Err(Error::Peer("invalid routing table k".into()));
        }
        Ok(Self {
            local,
            k,
            buckets: vec![Vec::new(); 256],
        })
    }

    pub fn local(&self) -> PeerId {
        self.local
    }

    /// XOR-distance bucket index in `0..=255`.
    pub fn bucket_index(a: &PeerId, b: &PeerId) -> usize {
        let mut xor = [0u8; 32];
        for index in 0..32 {
            xor[index] = a.0[index] ^ b.0[index];
        }
        let mut leading = 0usize;
        for byte in xor {
            if byte == 0 {
                leading += 8;
            } else {
                leading += byte.leading_zeros() as usize;
                break;
            }
        }
        leading.min(255)
    }

    /// Insert a peer. Returns true when the peer is present after the call.
    pub fn add(&mut self, peer: PeerId) -> bool {
        if peer == self.local || peer.0 == [0u8; 32] {
            return false;
        }
        let bucket = Self::bucket_index(&self.local, &peer);
        let entries = &mut self.buckets[bucket];
        if entries.contains(&peer) {
            // Move to the most-recently-seen position.
            entries.retain(|entry| *entry != peer);
            entries.push(peer);
            return true;
        }
        if entries.len() >= self.k {
            entries.remove(0);
        }
        entries.push(peer);
        true
    }

    pub fn remove(&mut self, peer: &PeerId) -> bool {
        let bucket = Self::bucket_index(&self.local, peer);
        let entries = &mut self.buckets[bucket];
        let before = entries.len();
        entries.retain(|entry| entry != peer);
        entries.len() != before
    }

    pub fn contains(&self, peer: &PeerId) -> bool {
        let bucket = Self::bucket_index(&self.local, peer);
        self.buckets[bucket].contains(peer)
    }

    pub fn len(&self) -> usize {
        self.buckets.iter().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Closest peers by XOR distance, ascending.
    pub fn closest(&self, target: &PeerId, limit: usize) -> Vec<PeerId> {
        let mut peers: Vec<PeerId> = self.buckets.iter().flatten().copied().collect();
        peers.sort_by_key(|a| xor_distance(a, target));
        peers.truncate(limit);
        peers
    }
}

fn xor_distance(a: &PeerId, b: &PeerId) -> [u8; 32] {
    let mut out = [0u8; 32];
    for index in 0..32 {
        out[index] = a.0[index] ^ b.0[index];
    }
    out
}

mod hex32 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(value))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let text = String::deserialize(d)?;
        let bytes = hex::decode(&text).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("expected 32 bytes"));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peer(seed: u8) -> PeerId {
        PeerId([seed; 32])
    }

    #[test]
    fn peer_store_persists_and_applies_backoff() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("peers.json");
        let mut store = PeerStore::new(PeerStoreConfig::default()).unwrap();
        let address: SocketAddr = "127.0.0.1:29798".parse().unwrap();
        store.record_success(peer(1), address, 1_000).unwrap();
        assert_eq!(store.len(), 1);
        for _ in 0..3 {
            store.record_failure(&peer(1), 1_000).unwrap();
        }
        let delay = store.retry_delay_secs(&peer(1)).unwrap();
        assert!(delay > 0);
        assert!(!store.is_due(&peer(1), 1_000));
        assert!(store.is_due(&peer(1), 1_000 + delay));
        store.save(&path).unwrap();

        let loaded = PeerStore::load(&path, PeerStoreConfig::default()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded.get(&peer(1)).unwrap().failures, 3);
        assert_eq!(loaded.get(&peer(1)).unwrap().address, address.to_string());
    }

    #[test]
    fn repeated_failures_ban_and_unban() {
        let mut store = PeerStore::new(PeerStoreConfig {
            ban_after_failures: 2,
            ..PeerStoreConfig::default()
        })
        .unwrap();
        let address: SocketAddr = "127.0.0.1:29798".parse().unwrap();
        store.record_success(peer(2), address, 10).unwrap();
        store.record_failure(&peer(2), 10).unwrap();
        assert!(!store.get(&peer(2)).unwrap().banned);
        store.record_failure(&peer(2), 10).unwrap();
        assert!(store.get(&peer(2)).unwrap().banned);
        assert!(store.retry_delay_secs(&peer(2)).is_none());
        store.unban(&peer(2)).unwrap();
        assert_eq!(store.retry_delay_secs(&peer(2)), Some(0));
    }

    #[test]
    fn store_rejects_corrupt_files_and_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("peers.json");
        std::fs::write(&path, b"not json").unwrap();
        assert!(PeerStore::load(&path, PeerStoreConfig::default()).is_err());

        let duplicate = r#"[
            {"peer_id":"0101010101010101010101010101010101010101010101010101010101010101","address":"127.0.0.1:29798","last_seen_unix":1,"failures":0,"banned":false},
            {"peer_id":"0101010101010101010101010101010101010101010101010101010101010101","address":"127.0.0.1:29799","last_seen_unix":1,"failures":0,"banned":false}
        ]"#;
        std::fs::write(&path, duplicate).unwrap();
        assert!(PeerStore::load(&path, PeerStoreConfig::default()).is_err());
    }

    #[test]
    fn routing_table_is_bounded_and_finds_closest() {
        let local = PeerId([0u8; 32]);
        let mut table = RoutingTable::new(local, 2).unwrap();
        assert_eq!(RoutingTable::bucket_index(&local, &PeerId([1u8; 32])), 7);
        // 0x80, 0x81 and 0x82 all share bucket 0 relative to the all-zero
        // local id, so they compete for the same two slots.
        let first = PeerId([0x80; 32]);
        let second = PeerId([0x81; 32]);
        let third = PeerId([0x82; 32]);
        assert!(table.add(first));
        assert!(table.add(second));
        assert!(table.add(third));
        assert_eq!(table.len(), 2);
        assert!(!table.contains(&first), "oldest entry is evicted");
        assert!(!table.add(local));

        let closest = table.closest(&second, 10);
        assert!(closest.contains(&second));
        assert!(closest.contains(&third));
        assert_eq!(closest.len(), 2);
    }

    #[test]
    fn routing_table_rejects_self_and_zero() {
        let local = peer(1);
        let mut table = RoutingTable::new(local, 20).unwrap();
        assert!(!table.add(local));
        assert!(!table.add(PeerId([0u8; 32])));
        assert!(table.is_empty());
    }
}
