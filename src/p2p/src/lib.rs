pub mod discovery;
pub mod network;

use dashmap::DashMap;
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use udaya_core::transaction::Transaction;
use udaya_core::types::{Block, BlockHash, InvVector};

/// Node type for the network
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    FullNode,
    LightNode,
    ArchiveNode,
    MinerNode,
    BootstrapNode,
}

/// Peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: u64,
    pub address: SocketAddr,
    pub version: u32,
    pub node_type: NodeType,
    pub user_agent: String,
    pub height: u64,
    pub connected_since: u64,
    pub is_outbound: bool,
    pub score: i32,
    pub ban_score: i32,
    pub last_ping: u64,
    pub ping_time: u64,
    pub last_seen: u64,
    pub fail_count: u32,
}

/// Network configuration
#[derive(Clone)]
pub struct P2PConfig {
    pub listen_addr: String,
    pub listen_port: u16,
    pub max_peers: usize,
    pub min_peers: usize,
    pub target_outbound: usize,
    pub ban_threshold: i32,
    pub ping_interval_secs: u64,
    pub timeout_secs: u64,
    pub enable_dns_seed: bool,
    pub dns_seeds: Vec<String>,
    pub seed_nodes: Vec<String>,
    pub relay_transactions: bool,
    pub protocol_version: u32,
    pub services: u64,
    pub user_agent: String,
    pub enable_tls: bool,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub tls_ca_cert_path: Option<String>,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0".to_string(),
            listen_port: 9798,
            max_peers: 125,
            min_peers: 8,
            target_outbound: 8,
            ban_threshold: 100,
            ping_interval_secs: 120,
            timeout_secs: 600,
            enable_dns_seed: true,
            dns_seeds: vec![
                "seed-us.Udaya.org".to_string(),
                "seed-eu.Udaya.org".to_string(),
                "seed-ap.Udaya.org".to_string(),
            ],
            seed_nodes: vec![
                "seed-us.Udaya.org:9798".to_string(),
                "seed-eu.Udaya.org:9798".to_string(),
                "seed-ap.Udaya.org:9798".to_string(),
            ],
            relay_transactions: true,
            protocol_version: 70016,
            services: 1 | 2,
            user_agent: "/Udaya:1.0.0/".to_string(),
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            tls_ca_cert_path: None,
        }
    }
}

/// Network message types
#[derive(Debug, Clone)]
pub enum NetworkMessage {
    Version(VersionMessage),
    Verack,
    Ping(u64),
    Pong(u64),
    GetAddr,
    Addr(Vec<PeerAddress>),
    Inv(Vec<InvVector>),
    GetData(Vec<InvVector>),
    NotFound(Vec<InvVector>),
    GetBlocks(GetBlocksMessage),
    GetHeaders(GetHeadersMessage),
    Tx(Transaction),
    Block(Block),
    Headers(Vec<udaya_core::types::BlockHeader>),
    SendHeaders,
    SendCmpct,
    CmpctBlock,
    GetBlockTxn,
    BlockTransactions,
    FeeFilter(u64),
    Reject(RejectMessage),
    Alert(String),
}

/// Version message for handshake
#[derive(Debug, Clone)]
pub struct VersionMessage {
    pub version: u32,
    pub services: u64,
    pub timestamp: i64,
    pub addr_recv: PeerAddress,
    pub addr_from: PeerAddress,
    pub nonce: u64,
    pub user_agent: String,
    pub start_height: u64,
    pub relay: bool,
}

/// Peer address for addr messages
#[derive(Debug, Clone)]
pub struct PeerAddress {
    pub time: u32,
    pub services: u64,
    pub address: String,
    pub port: u16,
}

/// GetBlocks message
#[derive(Debug, Clone)]
pub struct GetBlocksMessage {
    pub version: u32,
    pub locator_hashes: Vec<BlockHash>,
    pub hash_stop: BlockHash,
}

/// GetHeaders message
#[derive(Debug, Clone)]
pub struct GetHeadersMessage {
    pub version: u32,
    pub locator_hashes: Vec<BlockHash>,
    pub hash_stop: BlockHash,
}

/// Reject message
#[derive(Debug, Clone)]
pub struct RejectMessage {
    pub message: String,
    pub ccode: u8,
    pub reason: String,
    pub data: Vec<u8>,
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub inbound_peers: usize,
    pub outbound_peers: usize,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub uptime_seconds: u64,
}

/// Shared network state
pub struct NetworkState {
    pub peers: DashMap<u64, PeerInfo>,
    pub banned_peers: DashMap<String, i64>,
    pub addr_cache: DashMap<String, PeerAddress>,
    pub config: P2PConfig,
    pub stats: RwLock<NetworkStats>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Current chain height advertised to peers in version messages
    pub start_height: AtomicU64,
}

impl NetworkState {
    pub fn new(config: P2PConfig) -> Self {
        Self {
            peers: DashMap::new(),
            banned_peers: DashMap::new(),
            addr_cache: DashMap::new(),
            config,
            stats: RwLock::new(NetworkStats {
                connected_peers: 0,
                inbound_peers: 0,
                outbound_peers: 0,
                total_bytes_sent: 0,
                total_bytes_received: 0,
                messages_sent: 0,
                messages_received: 0,
                uptime_seconds: 0,
            }),
            start_time: chrono::Utc::now(),
            start_height: AtomicU64::new(0),
        }
    }

    /// Set the chain height advertised in version messages
    pub fn set_start_height(&self, height: u64) {
        let old = self.start_height.swap(height, Ordering::Relaxed);
        if old != height {
            log::debug!("Advertised chain height updated: {} -> {}", old, height);
        }
    }

    /// Current advertised chain height
    pub fn advertised_height(&self) -> u64 {
        self.start_height.load(Ordering::Relaxed)
    }

    pub fn is_banned(&self, addr: &str) -> bool {
        if let Some(expiry) = self.banned_peers.get(addr) {
            chrono::Utc::now().timestamp() < *expiry
        } else {
            false
        }
    }

    pub fn ban_peer(&self, addr: &str, duration_secs: i64) {
        let expiry = chrono::Utc::now().timestamp() + duration_secs;
        self.banned_peers.insert(addr.to_string(), expiry);
    }

    pub fn get_connected_count(&self) -> usize {
        self.peers.len()
    }

    pub fn update_peer_height(&self, peer_id: u64, height: u64) {
        if let Some(mut peer) = self.peers.get_mut(&peer_id) {
            peer.height = height;
        }
    }

    pub fn increment_ban_score(&self, addr: &str, score: i32) -> bool {
        let mut ban = false;
        if let Some(mut peer) = self
            .peers
            .iter_mut()
            .find(|p| p.address.to_string().starts_with(addr))
        {
            peer.ban_score += score;
            if peer.ban_score >= self.config.ban_threshold {
                ban = true;
            }
        }
        if ban {
            self.ban_peer(addr, 3600); // Ban for 1 hour
        }
        ban
    }

    pub fn get_outbound_peers(&self) -> Vec<PeerInfo> {
        self.peers
            .iter()
            .filter(|p| p.is_outbound)
            .map(|p| p.clone())
            .collect()
    }

    pub fn get_best_peers(&self, count: usize) -> Vec<PeerInfo> {
        let mut peers: Vec<PeerInfo> = self
            .peers
            .iter()
            .map(|p| p.clone())
            .filter(|p| p.ban_score < self.config.ban_threshold)
            .collect();

        peers.sort_by(|a, b| b.score.cmp(&a.score));
        peers.truncate(count);
        peers
    }
}
