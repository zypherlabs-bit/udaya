pub mod discovery;
pub mod network;
pub mod tls_utils;

use crate::isolation;
use std::net::SocketAddr;

// Re-export isolation/identity helpers from primitives for P2P handshake/config use.
pub mod isolation {
    pub const EXPERIMENTAL_NETWORK_MAGIC: [u8; 4] = [0x55, 0x41, 0x44, 0x43]; // "UADC"
    pub const PROTECTED_MAINNET_NETWORK_MAGIC: [u8; 4] = [0xBF, 0x59, 0x1A, 0xE7];
    pub const EXPERIMENTAL_DEFAULT_P2P_PORT: u16 = 19799;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum IsolationError {
        ProtectedMainnetRefused,
        ProtectedMainnetMagicRefused,
        MagicMismatch { expected: [u8; 4], got: [u8; 4] },
    }

    impl std::fmt::Display for IsolationError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                IsolationError::ProtectedMainnetRefused => write!(f, "protected existing Udaya mainnet refused"),
                IsolationError::ProtectedMainnetMagicRefused => write!(f, "protected existing Udaya mainnet magic refused"),
                IsolationError::MagicMismatch { expected, got } => write!(f, "network magic mismatch: expected {:02X?}, got {:02X?}", expected, got),
            }
        }
    }
    impl std::error::Error for IsolationError {}

    pub fn ensure_not_protected_magic(magic: [u8; 4]) -> Result<(), IsolationError> {
        if magic == PROTECTED_MAINNET_NETWORK_MAGIC { Err(IsolationError::ProtectedMainnetMagicRefused) } else { Ok(()) }
    }
    pub fn validate_peer_magic(expected: [u8; 4], got: [u8; 4]) -> Result<(), IsolationError> {
        ensure_not_protected_magic(got)?;
        if expected != got { Err(IsolationError::MagicMismatch { expected, got }) } else { Ok(()) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType { FullNode, LightNode, ArchiveNode, ValidatorNode, BootstrapNode }

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
}

#[derive(Clone, Debug)]
pub struct P2PConfig {
    pub listen_addr: String,
    pub listen_port: u16,
    pub network_magic: [u8; 4],
    pub chain_id: String,
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
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1".to_string(), // isolated/local by default
            listen_port: isolation::EXPERIMENTAL_DEFAULT_P2P_PORT,
            network_magic: isolation::EXPERIMENTAL_NETWORK_MAGIC,
            chain_id: "uadc-experimental-0001".to_string(),
            max_peers: 32,
            min_peers: 1,
            target_outbound: 4,
            ban_threshold: 50,
            ping_interval_secs: 120,
            timeout_secs: 300,
            enable_dns_seed: false, // no external dependency by default
            dns_seeds: vec![],
            seed_nodes: vec![], // explicit opt-in only
            relay_transactions: true,
            protocol_version: 1,
            services: 1,
            user_agent: "/Udaya-Global-Experimental:0.1.0/".to_string(),
            enable_tls: false,
        }
    }
}

impl P2PConfig {
    pub fn validate_isolation(&self) -> Result<(), isolation::IsolationError> {
        isolation::ensure_not_protected_magic(self.network_magic)
    }
}
