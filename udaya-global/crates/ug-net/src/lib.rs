//! Udaya Global peer-to-peer networking.
//!
//! Security properties of this layer:
//!
//! * Every connection is encrypted and forward-secret with Noise XX
//!   (`Noise_XX_25519_ChaChaPoly_BLAKE2s`).
//! * The Noise channel is authenticated by an Ed25519 identity exchanged in an
//!   encrypted Hello. The signature binds the peer ID, the chain ID, the
//!   genesis hash, both Noise static keys, a random nonce and the advertised
//!   listen address. A party without the Ed25519 key cannot impersonate a peer,
//!   and a man-in-the-middle cannot transplant the Hello onto another channel.
//! * Frames are length-prefixed and bounded before allocation. Consensus
//!   messages use the canonical codecs from `ug-core`/`ug-consensus`.
//! * Every peer has an inbound token bucket for messages and bytes. Overflow is
//!   an error, not a warning, so the caller disconnects the peer.
//! * The peer store supports atomic persistence, failure backoff and banning;
//!   the routing table is a bounded Kademlia-style bucket structure.
//!
//! This is a TCP transport with a Kademlia-style peer table. QUIC, NAT
//! traversal/hole punching, and a full distributed hash table are **not**
//! implemented. See `docs/architecture/P2P_NETWORK_ARCHITECTURE.md`.

pub mod framing;
pub mod handshake;
pub mod identity;
pub mod message;
pub mod peer;
pub mod rate_limit;

pub use handshake::{accept, connect, Direction, NetworkConfig, PeerInfo, PeerSession};
pub use identity::{LocalIdentity, PeerId};
pub use message::{CommitData, Message, PeerAddress};
pub use peer::{PeerRecord, PeerStore, PeerStoreConfig, RoutingTable, MAX_STORED_PEERS};
pub use rate_limit::{RateLimitConfig, RateLimiter};

/// Wire protocol version. Must match `ug_core::PROTOCOL_VERSION`.
pub const PROTOCOL_VERSION: u32 = ug_core::PROTOCOL_VERSION;

/// Maximum plaintext frame accepted from a peer (8 MiB).
pub const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;

/// Maximum size of one Noise handshake message.
pub const HANDSHAKE_FRAME_MAX: usize = 4096;

/// Maximum peer addresses in a `Peers` message.
pub const MAX_PEERS_PER_MESSAGE: usize = 64;

/// Maximum blocks in a `Blocks` message.
pub const MAX_BLOCKS_PER_MESSAGE: usize = 2;

/// Maximum commit certificates in a `Commits` message. Each commit contains a
/// full block, so this stays small enough for the frame limit.
pub const MAX_COMMITS_PER_MESSAGE: usize = 2;

/// Upper bound on one encoded quorum certificate: header plus the maximum
/// number of votes.
pub const MAX_QC_ENCODED_BYTES: usize =
    64 + ug_consensus::MAX_VALIDATORS * (ug_consensus::vote::VOTE_ENCODED_LEN + 4);

/// Maximum locator hashes in a `GetBlocks` message.
pub const MAX_LOCATOR_HASHES: usize = 64;

/// Maximum encoded listen address string.
pub const MAX_LISTEN_ADDR_BYTES: usize = 128;

/// Maximum on-disk peer store size.
pub const MAX_PEER_STORE_BYTES: u64 = 4 * 1024 * 1024;

/// Networking errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("noise error: {0}")]
    Noise(String),
    #[error("codec error: {0}")]
    Codec(String),
    #[error("handshake failed: {0}")]
    Handshake(String),
    #[error("peer sent an oversized frame: {0} bytes")]
    FrameTooLarge(usize),
    #[error("peer exceeded its rate limit")]
    RateLimited,
    #[error("peer error: {0}")]
    Peer(String),
    #[error("peer store error: {0}")]
    Store(String),
    #[error("network identity error: {0}")]
    Chain(String),
    #[error("operation timed out")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<ug_core::Error> for Error {
    fn from(value: ug_core::Error) -> Self {
        Error::Codec(value.to_string())
    }
}

impl From<ug_consensus::Error> for Error {
    fn from(value: ug_consensus::Error) -> Self {
        Error::Codec(value.to_string())
    }
}
