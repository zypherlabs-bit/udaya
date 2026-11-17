/// Udaya P2P - Peer-to-peer networking layer
/// Cloud-independent peer discovery and communication

use crate::primitives::*;
use dashmap::DashMap;
use std::net::{SocketAddr, Shutdown};
use std::collections::HashMap;

/// Peer information
#[derive(Debug, Clone)]
pub struct Peer {
    /// Peer address
    pub address: SocketAddr,
    /// Services bitfield
    pub services: u64,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Connection score
    pub score: i32,
    /// Whether we're connecting outbound
    pub outbound: bool,
}

/// Peer manager for cloud-independent P2P
pub struct PeerManager {
    /// Connected peers
    peers: DashMap<SocketAddr, Peer>,
    /// banned addresses
    banned: DashMap<String, u64>,
    /// Our address
    our_address: SocketAddr,
    /// Our services
    our_services: u64,
}

impl PeerManager {
    /// Create new peer manager
    pub fn new(our_address: SocketAddr, our_services: u64) -> Self {
        PeerManager {
            peers: DashMap::new(),
            banned: DashMap::new(),
            our_address,
            our_services,
        }
    }

    /// Add a connected peer
    pub fn add_peer(&mut self, peer: Peer) {
        self.peers.insert(peer.address, peer);
    }

    /// Ban an address
    pub fn ban(&mut self, address: &str, duration: u64) {
        self.banned.insert(address.to_string(), std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + duration);
    }

    /// Check if address is banned
    pub fn is_banned(&self, address: &str) -> bool {
        self.banned.contains_key(address)
    }

    /// Get connected peer count
    pub fn count(&self) -> usize {
        self.peers.len()
    }

    /// Get outbound peer count
    pub fn outbound_count(&self) -> usize {
        self.peers.values().filter(|p| p.outbound).count()
    }

    /// Get all peers
    pub fn peers(&self) -> Vec<Peer> {
        self.peers.values().cloned().collect()
    }
}

/// Message types for P2P protocol
pub mod message {
    /// Version message
    pub struct Version {
        pub version: u32,
        pub services: u64,
        pub timestamp: u64,
        pub addr_recv: SocketAddr,
        pub addr_from: SocketAddr,
        pub nonce: u64,
        pub user_agent: String,
        pub start_height: u64,
        pub relay: bool,
    }

    /// Block message
    pub struct Block {
        pub height: u64,
        pub hash: Vec<u8>,
        pub transactions: Vec<Vec<u8>>,
    }

    /// Inventory message
    pub struct Inv {
        /// Type of item
        pub ty: u8,
        /// Item hash
        pub hash: Vec<u8>,
    }
}

/// Network configuration
pub mod config {
    /// Default listen port
    pub const DEFAULT_PORT: u16 = 9799;
    /// Maximum outbound peers
    pub const MAX_OUTBOUND: usize = 8;
    /// Minimum peers to maintain
    pub const MIN_PEERS: usize = 4;
    /// Ban duration (seconds)
    pub const BAN_DURATION: u64 = 3600;
    /// Ping interval (seconds)
    pub const PING_INTERVAL: u64 = 120;
    /// Timeout (seconds)
    pub const TIMEOUT: u64 = 300;
}