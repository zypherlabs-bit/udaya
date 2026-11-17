//! Noise XX transport with Ed25519 identity authentication.
//!
//! Handshake sequence:
//!
//! 1. Noise XX establishes an encrypted channel using ephemeral and static
//!    X25519 keys. This provides confidentiality, forward secrecy and peer
//!    authentication *relative to the Noise static keys*.
//! 2. Each side sends one encrypted `Hello`. The Hello contains its Ed25519
//!    peer identity, chain ID, network magic, genesis hash, a random nonce, its
//!    advertised listen address, and a signature over all of those fields plus
//!    both Noise static keys.
//! 3. The receiver verifies the signature, the chain/genesis binding and the
//!    Noise-static binding. Only then is a `PeerSession` returned.
//!
//! This prevents an active man-in-the-middle from reusing an identity Hello on
//! another Noise channel, and prevents cross-chain connections even when two
//! networks happen to share a bootstrap address.

use crate::framing::{read_frame, write_frame};
use crate::identity::{validate_remote_peer, LocalIdentity, PeerId};
use crate::message::Message;
use crate::rate_limit::{RateLimitConfig, RateLimiter};
use crate::{
    Error, Result, HANDSHAKE_FRAME_MAX, MAX_FRAME_BYTES, MAX_LISTEN_ADDR_BYTES, PROTOCOL_VERSION,
};
use snow::{Builder, TransportState};
use std::fmt;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio::time::timeout;
use ug_core::codec::{Decoder, Encoder};
use ug_core::crypto::{self, Hash32, SignatureBytes};
use ug_core::ChainParams;

const NOISE_PARAMS: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";
const HELLO_VERSION: u8 = 1;
const HELLO_TAG: &str = "ug/p2p-hello";
const MAX_HELLO_BYTES: usize = 512;
const MAX_CLOCK_DRIFT_SECS: u64 = 600;
const AEAD_OVERHEAD: usize = 16;

/// Connection direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Inbound,
    Outbound,
}

/// Role inside the Noise handshake.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Initiator = 1,
    Responder = 2,
}

impl Role {
    fn opposite(self) -> Self {
        match self {
            Self::Initiator => Self::Responder,
            Self::Responder => Self::Initiator,
        }
    }

    fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Initiator),
            2 => Ok(Self::Responder),
            other => Err(Error::Handshake(format!("unknown hello role {other}"))),
        }
    }
}

/// Network configuration for one node.
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub chain: ChainParams,
    pub genesis_hash: Hash32,
    pub identity: LocalIdentity,
    pub listen_addr: SocketAddr,
    pub max_frame_bytes: usize,
    pub handshake_timeout: Duration,
    pub connect_timeout: Duration,
    pub rate_limit: RateLimitConfig,
}

impl NetworkConfig {
    pub fn new(
        chain: ChainParams,
        genesis_hash: Hash32,
        identity: LocalIdentity,
        listen_addr: SocketAddr,
    ) -> Self {
        Self {
            chain,
            genesis_hash,
            identity,
            listen_addr,
            max_frame_bytes: MAX_FRAME_BYTES,
            handshake_timeout: Duration::from_secs(10),
            connect_timeout: Duration::from_secs(10),
            rate_limit: RateLimitConfig::default(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.chain
            .validate()
            .map_err(|error| Error::Chain(error.to_string()))?;
        if self.genesis_hash == [0u8; 32] {
            return Err(Error::Chain("genesis hash must be pinned".into()));
        }
        if self.listen_addr.port() == 0 {
            return Err(Error::Chain("listen address must have a port".into()));
        }
        if self.max_frame_bytes < 1024
            || self.max_frame_bytes > MAX_FRAME_BYTES
            || self.max_frame_bytes < self.chain.max_block_bytes.saturating_add(512)
        {
            return Err(Error::Chain("invalid maximum frame size".into()));
        }
        if self.handshake_timeout.is_zero() || self.connect_timeout.is_zero() {
            return Err(Error::Chain("timeouts must be non-zero".into()));
        }
        self.rate_limit.validate()?;
        Ok(())
    }
}

/// Information about the authenticated remote peer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub remote_addr: SocketAddr,
    pub listen_addr: SocketAddr,
    pub noise_static: [u8; 32],
    pub direction: Direction,
    pub connected_at_unix: u64,
}

/// Encrypted Hello. Kept private to the handshake module; only the verified
/// peer information escapes.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Hello {
    version: u8,
    protocol_version: u32,
    role: Role,
    chain_id: String,
    network_magic: [u8; 4],
    genesis_hash: Hash32,
    peer_id: PeerId,
    noise_static: [u8; 32],
    nonce: [u8; 32],
    timestamp: u64,
    listen_addr: SocketAddr,
    signature: SignatureBytes,
}

impl Hello {
    fn new_signed(config: &NetworkConfig, role: Role, remote_noise_static: &[u8; 32]) -> Self {
        let mut hello = Self {
            version: HELLO_VERSION,
            protocol_version: PROTOCOL_VERSION,
            role,
            chain_id: config.chain.chain_id.clone(),
            network_magic: config.chain.network_magic,
            genesis_hash: config.genesis_hash,
            peer_id: config.identity.peer_id(),
            noise_static: config.identity.noise_static(),
            nonce: rand::random(),
            timestamp: unix_now(),
            listen_addr: config.listen_addr,
            signature: [0u8; 64],
        };
        hello.signature = config
            .identity
            .sign_digest(&hello.digest(remote_noise_static));
        hello
    }

    fn digest(&self, remote_noise_static: &[u8; 32]) -> Hash32 {
        let mut e = Encoder::with_capacity(256);
        e.u8(self.version);
        e.u32(self.protocol_version);
        e.u8(self.role as u8);
        e.u32(self.chain_id.len() as u32);
        e.fixed(self.chain_id.as_bytes());
        e.fixed(&self.network_magic);
        e.fixed(&self.genesis_hash);
        e.fixed(&self.peer_id.0);
        e.fixed(&self.noise_static);
        e.fixed(remote_noise_static);
        e.fixed(&self.nonce);
        e.u64(self.timestamp);
        let listen = self.listen_addr.to_string();
        e.fixed(listen.as_bytes());
        crypto::tagged_hash(HELLO_TAG, &[&e.finish()])
    }

    fn verify(
        &self,
        config: &NetworkConfig,
        expected_role: Role,
        actual_remote_noise_static: &[u8; 32],
    ) -> Result<()> {
        if self.version != HELLO_VERSION {
            return Err(Error::Handshake(format!(
                "unsupported hello version {}",
                self.version
            )));
        }
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(Error::Handshake(format!(
                "peer speaks protocol version {}, expected {}",
                self.protocol_version, PROTOCOL_VERSION
            )));
        }
        if self.role != expected_role {
            return Err(Error::Handshake(
                "hello role does not match the connection direction".into(),
            ));
        }
        if self.chain_id != config.chain.chain_id
            || self.network_magic != config.chain.network_magic
        {
            return Err(Error::Handshake(
                "peer belongs to a different chain (chain id/magic mismatch)".into(),
            ));
        }
        if self.genesis_hash != config.genesis_hash {
            return Err(Error::Handshake("peer has a different genesis hash".into()));
        }
        if self.noise_static != *actual_remote_noise_static {
            return Err(Error::Handshake(
                "hello does not match the Noise static key".into(),
            ));
        }
        if self.listen_addr.port() == 0 || self.listen_addr.ip().is_unspecified() {
            return Err(Error::Handshake(
                "peer advertised an unusable address".into(),
            ));
        }
        validate_remote_peer(&config.identity, &self.peer_id)?;
        let now = unix_now();
        let drift = now.abs_diff(self.timestamp);
        if drift > MAX_CLOCK_DRIFT_SECS {
            return Err(Error::Handshake(format!(
                "peer clock is {drift}s away from local time"
            )));
        }
        let digest = self.digest(&config.identity.noise_static());
        if !LocalIdentity::verify_digest(&self.peer_id, &self.signature, &digest) {
            return Err(Error::Handshake(
                "hello signature verification failed".into(),
            ));
        }
        Ok(())
    }

    fn encode(&self) -> Result<Vec<u8>> {
        if self.chain_id.len() > 64 {
            return Err(Error::Handshake("chain id too long in hello".into()));
        }
        let listen = self.listen_addr.to_string();
        if listen.len() > MAX_LISTEN_ADDR_BYTES {
            return Err(Error::Handshake("listen address too long in hello".into()));
        }
        let mut e = Encoder::with_capacity(MAX_HELLO_BYTES);
        e.u8(self.version);
        e.u32(self.protocol_version);
        e.u8(self.role as u8);
        e.bytes(self.chain_id.as_bytes());
        e.fixed(&self.network_magic);
        e.fixed(&self.genesis_hash);
        e.fixed(&self.peer_id.0);
        e.fixed(&self.noise_static);
        e.fixed(&self.nonce);
        e.u64(self.timestamp);
        e.bytes(listen.as_bytes());
        e.fixed(&self.signature);
        let bytes = e.finish();
        if bytes.len() > MAX_HELLO_BYTES {
            return Err(Error::Handshake("encoded hello is too large".into()));
        }
        Ok(bytes)
    }

    fn decode(data: &[u8]) -> Result<Self> {
        if data.len() > MAX_HELLO_BYTES {
            return Err(Error::Handshake("hello is too large".into()));
        }
        let mut d = Decoder::new(data);
        let version = d.u8()?;
        let protocol_version = d.u32()?;
        let role = Role::from_u8(d.u8()?)?;
        let chain_bytes = d.bytes(64)?;
        let chain_id = std::str::from_utf8(chain_bytes)
            .map_err(|_| Error::Handshake("chain id is not UTF-8".into()))?
            .to_string();
        let network_magic = d.fixed::<4>()?;
        let genesis_hash = d.fixed::<32>()?;
        let peer_id = PeerId(d.fixed::<32>()?);
        let noise_static = d.fixed::<32>()?;
        let nonce = d.fixed::<32>()?;
        let timestamp = d.u64()?;
        let listen_bytes = d.bytes(MAX_LISTEN_ADDR_BYTES)?;
        let listen_addr = std::str::from_utf8(listen_bytes)
            .map_err(|_| Error::Handshake("listen address is not UTF-8".into()))?
            .parse::<SocketAddr>()
            .map_err(|_| Error::Handshake("listen address is not a socket address".into()))?;
        let signature = d.fixed::<64>()?;
        d.finish()?;
        Ok(Self {
            version,
            protocol_version,
            role,
            chain_id,
            network_magic,
            genesis_hash,
            peer_id,
            noise_static,
            nonce,
            timestamp,
            listen_addr,
            signature,
        })
    }
}

struct EncryptedStream {
    stream: TcpStream,
    transport: TransportState,
    max_frame: usize,
}

impl EncryptedStream {
    async fn read_frame(&mut self) -> Result<Vec<u8>> {
        let ciphertext = read_frame(&mut self.stream, self.max_frame + AEAD_OVERHEAD).await?;
        let mut plaintext = vec![0u8; ciphertext.len().saturating_sub(AEAD_OVERHEAD).max(1)];
        let length = self
            .transport
            .read_message(&ciphertext, &mut plaintext)
            .map_err(|error| Error::Noise(format!("{error:?}")))?;
        plaintext.truncate(length);
        Ok(plaintext)
    }

    async fn write_frame(&mut self, plaintext: &[u8]) -> Result<()> {
        if plaintext.len() > self.max_frame {
            return Err(Error::FrameTooLarge(plaintext.len()));
        }
        let mut ciphertext = vec![0u8; plaintext.len() + AEAD_OVERHEAD];
        let length = self
            .transport
            .write_message(plaintext, &mut ciphertext)
            .map_err(|error| Error::Noise(format!("{error:?}")))?;
        write_frame(
            &mut self.stream,
            &ciphertext[..length],
            self.max_frame + AEAD_OVERHEAD,
        )
        .await
    }
}

/// An authenticated, encrypted peer connection.
pub struct PeerSession {
    stream: EncryptedStream,
    peer: PeerInfo,
    limiter: RateLimiter,
    chain: ChainParams,
}

impl PeerSession {
    pub fn peer(&self) -> &PeerInfo {
        &self.peer
    }

    pub fn chain(&self) -> &ChainParams {
        &self.chain
    }

    /// Send one canonical message.
    pub async fn send(&mut self, message: &Message) -> Result<()> {
        self.stream.write_frame(&message.encode()?).await
    }

    /// Receive one canonical message, enforcing the per-peer inbound limiter.
    pub async fn recv(&mut self) -> Result<Message> {
        let frame = self.stream.read_frame().await?;
        self.limiter.check_message(frame.len())?;
        Message::decode(&frame, &self.chain)
    }

    pub async fn ping(&mut self, nonce: u64) -> Result<u64> {
        self.send(&Message::Ping { nonce }).await?;
        match self.recv().await? {
            Message::Pong { nonce: received } if received == nonce => Ok(received),
            other => Err(Error::Peer(format!("unexpected ping response: {other:?}"))),
        }
    }
}

/// Connect to a peer and perform the authenticated handshake.
pub async fn connect(config: &NetworkConfig, address: SocketAddr) -> Result<PeerSession> {
    config.validate()?;
    let stream = timeout(config.connect_timeout, TcpStream::connect(address))
        .await
        .map_err(|_| Error::Timeout)??;
    stream.set_nodelay(true)?;
    let remote_addr = stream.peer_addr()?;
    handshake(config, stream, Direction::Outbound, remote_addr).await
}

/// Accept an already-connected TCP stream and perform the authenticated
/// handshake.
pub async fn accept(stream: TcpStream, config: &NetworkConfig) -> Result<PeerSession> {
    config.validate()?;
    stream.set_nodelay(true)?;
    let remote_addr = stream.peer_addr()?;
    handshake(config, stream, Direction::Inbound, remote_addr).await
}

async fn handshake(
    config: &NetworkConfig,
    mut stream: TcpStream,
    direction: Direction,
    remote_addr: SocketAddr,
) -> Result<PeerSession> {
    let params = NOISE_PARAMS
        .parse()
        .map_err(|error| Error::Noise(format!("{error:?}")))?;
    let local_private = config.identity.noise_secret_bytes();
    let builder = Builder::new(params)
        .local_private_key(&local_private)
        .map_err(|error| Error::Noise(format!("{error:?}")))?;
    let local_role = match direction {
        Direction::Outbound => Role::Initiator,
        Direction::Inbound => Role::Responder,
    };
    let remote_role = local_role.opposite();
    let handshake_timeout = config.handshake_timeout;

    let (transport, remote_noise_static) = if direction == Direction::Outbound {
        let mut handshake = builder
            .build_initiator()
            .map_err(|error| Error::Noise(format!("{error:?}")))?;
        let mut buffer = vec![0u8; HANDSHAKE_FRAME_MAX];

        let length = handshake
            .write_message(&[], &mut buffer)
            .map_err(|error| Error::Noise(format!("{error:?}")))?;
        timeout(
            handshake_timeout,
            write_frame(&mut stream, &buffer[..length], HANDSHAKE_FRAME_MAX),
        )
        .await
        .map_err(|_| Error::Timeout)??;

        let message = timeout(
            handshake_timeout,
            read_frame(&mut stream, HANDSHAKE_FRAME_MAX),
        )
        .await
        .map_err(|_| Error::Timeout)??;
        handshake
            .read_message(&message, &mut buffer)
            .map_err(|error| Error::Noise(format!("{error:?}")))?;

        let length = handshake
            .write_message(&[], &mut buffer)
            .map_err(|error| Error::Noise(format!("{error:?}")))?;
        timeout(
            handshake_timeout,
            write_frame(&mut stream, &buffer[..length], HANDSHAKE_FRAME_MAX),
        )
        .await
        .map_err(|_| Error::Timeout)??;

        let remote = remote_static(&handshake)?;
        let transport = handshake
            .into_transport_mode()
            .map_err(|error| Error::Noise(format!("{error:?}")))?;
        (transport, remote)
    } else {
        let mut handshake = builder
            .build_responder()
            .map_err(|error| Error::Noise(format!("{error:?}")))?;
        let mut buffer = vec![0u8; HANDSHAKE_FRAME_MAX];

        let message = timeout(
            handshake_timeout,
            read_frame(&mut stream, HANDSHAKE_FRAME_MAX),
        )
        .await
        .map_err(|_| Error::Timeout)??;
        handshake
            .read_message(&message, &mut buffer)
            .map_err(|error| Error::Noise(format!("{error:?}")))?;

        let length = handshake
            .write_message(&[], &mut buffer)
            .map_err(|error| Error::Noise(format!("{error:?}")))?;
        timeout(
            handshake_timeout,
            write_frame(&mut stream, &buffer[..length], HANDSHAKE_FRAME_MAX),
        )
        .await
        .map_err(|_| Error::Timeout)??;

        let message = timeout(
            handshake_timeout,
            read_frame(&mut stream, HANDSHAKE_FRAME_MAX),
        )
        .await
        .map_err(|_| Error::Timeout)??;
        handshake
            .read_message(&message, &mut buffer)
            .map_err(|error| Error::Noise(format!("{error:?}")))?;

        let remote = remote_static(&handshake)?;
        let transport = handshake
            .into_transport_mode()
            .map_err(|error| Error::Noise(format!("{error:?}")))?;
        (transport, remote)
    };

    let mut encrypted = EncryptedStream {
        stream,
        transport,
        max_frame: config.max_frame_bytes,
    };

    let local_hello = Hello::new_signed(config, local_role, &remote_noise_static);
    encrypted.write_frame(&local_hello.encode()?).await?;

    let remote_hello_bytes = encrypted.read_frame().await?;
    let remote_hello = Hello::decode(&remote_hello_bytes)?;
    remote_hello.verify(config, remote_role, &remote_noise_static)?;

    let peer = PeerInfo {
        peer_id: remote_hello.peer_id,
        remote_addr,
        listen_addr: remote_hello.listen_addr,
        noise_static: remote_noise_static,
        direction,
        connected_at_unix: unix_now(),
    };
    let limiter = RateLimiter::new(&config.rate_limit)?;
    Ok(PeerSession {
        stream: encrypted,
        peer,
        limiter,
        chain: config.chain.clone(),
    })
}

fn remote_static(handshake: &snow::HandshakeState) -> Result<[u8; 32]> {
    let key = handshake
        .get_remote_static()
        .ok_or_else(|| Error::Handshake("Noise handshake did not reveal a static key".into()))?;
    if key.len() != 32 {
        return Err(Error::Handshake("invalid Noise static key length".into()));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(key);
    Ok(out)
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl fmt::Debug for PeerSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PeerSession")
            .field("peer", &self.peer)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PeerStore, PeerStoreConfig};
    use tokio::net::TcpListener;

    fn config(seed: u8, address: SocketAddr) -> NetworkConfig {
        NetworkConfig::new(
            ChainParams::devnet(),
            [0xAA; 32],
            LocalIdentity::from_seed(&[seed; 32]),
            address,
        )
    }

    async fn pair(config_a: NetworkConfig, config_b: NetworkConfig) -> (PeerSession, PeerSession) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let mut config_a = config_a;
        config_a.listen_addr = address;
        let server_config = config_a.clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            accept(stream, &server_config).await
        });
        let client = connect(&config_b, address).await.unwrap();
        let server = server.await.unwrap().unwrap();
        (client, server)
    }

    #[tokio::test]
    async fn encrypted_handshake_authenticates_both_peers() {
        let config_a = config(1, "127.0.0.1:29798".parse().unwrap());
        let config_b = config(2, "127.0.0.1:29799".parse().unwrap());
        let expected_a = config_a.identity.peer_id();
        let expected_b = config_b.identity.peer_id();
        let (mut client, mut server) = pair(config_a, config_b).await;

        assert_eq!(client.peer().peer_id, expected_a);
        assert_eq!(server.peer().peer_id, expected_b);
        assert_eq!(client.peer().direction, Direction::Outbound);
        assert_eq!(server.peer().direction, Direction::Inbound);
        assert_ne!(client.peer().noise_static, [0u8; 32]);

        client.send(&Message::Ping { nonce: 7 }).await.unwrap();
        assert_eq!(server.recv().await.unwrap(), Message::Ping { nonce: 7 });
        server.send(&Message::Pong { nonce: 7 }).await.unwrap();
        assert_eq!(client.recv().await.unwrap(), Message::Pong { nonce: 7 });
    }

    #[tokio::test]
    async fn chain_and_genesis_mismatch_fail_the_handshake() {
        let config_a = config(1, "127.0.0.1:29798".parse().unwrap());
        let mut config_b = config(2, "127.0.0.1:29799".parse().unwrap());
        config_b.genesis_hash = [0xBB; 32];
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let mut config_a = config_a;
        config_a.listen_addr = address;
        let server_config = config_a.clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            accept(stream, &server_config).await
        });
        let client = connect(&config_b, address).await;
        let server = server.await.unwrap();
        assert!(client.is_err());
        assert!(server.is_err());
    }

    #[tokio::test]
    async fn per_peer_rate_limit_disconnects_a_flooding_peer() {
        let mut config_a = config(1, "127.0.0.1:29798".parse().unwrap());
        config_a.rate_limit = RateLimitConfig {
            bytes_per_second: 1024,
            bytes_burst: 4096,
            messages_per_second: 1,
            messages_burst: 1,
        };
        let config_b = config(2, "127.0.0.1:29799".parse().unwrap());
        let (mut client, mut server) = pair(config_a, config_b).await;
        client.send(&Message::Ping { nonce: 1 }).await.unwrap();
        assert!(server.recv().await.is_ok());
        client.send(&Message::Ping { nonce: 2 }).await.unwrap();
        assert!(matches!(server.recv().await, Err(Error::RateLimited)));
    }

    #[test]
    fn hello_is_strictly_encoded_and_chain_bound() {
        let config_a = config(1, "127.0.0.1:29798".parse().unwrap());
        let config_b = config(2, "127.0.0.1:29799".parse().unwrap());
        let remote_noise = config_b.identity.noise_static();
        let hello = Hello::new_signed(&config_a, Role::Initiator, &remote_noise);
        let bytes = hello.encode().unwrap();
        let decoded = Hello::decode(&bytes).unwrap();
        assert_eq!(hello, decoded);
        decoded
            .verify(
                &config_b,
                Role::Initiator,
                &config_a.identity.noise_static(),
            )
            .unwrap();

        let mut tampered = decoded.clone();
        tampered.signature[0] ^= 1;
        assert!(tampered
            .verify(
                &config_b,
                Role::Initiator,
                &config_a.identity.noise_static()
            )
            .is_err());

        assert!(decoded
            .verify(
                &config_b,
                Role::Responder,
                &config_a.identity.noise_static()
            )
            .is_err());
        assert!(decoded
            .verify(&config_b, Role::Initiator, &[9u8; 32])
            .is_err());

        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Hello::decode(&trailing).is_err());
        assert!(Hello::decode(&bytes[..bytes.len() - 1]).is_err());
    }

    #[test]
    fn hello_rejects_a_stale_timestamp_and_wrong_chain() {
        let config_a = config(1, "127.0.0.1:29798".parse().unwrap());
        let config_b = config(2, "127.0.0.1:29799".parse().unwrap());
        let remote_noise = config_b.identity.noise_static();
        let mut hello = Hello::new_signed(&config_a, Role::Initiator, &remote_noise);
        hello.timestamp = unix_now().saturating_sub(MAX_CLOCK_DRIFT_SECS + 10);
        assert!(hello
            .verify(
                &config_b,
                Role::Initiator,
                &config_a.identity.noise_static()
            )
            .is_err());

        let mut other = config_b.clone();
        other.chain.chain_id = "udaya-global-devnet-2".into();
        let hello = Hello::new_signed(&config_a, Role::Initiator, &remote_noise);
        assert!(hello
            .verify(&other, Role::Initiator, &config_a.identity.noise_static())
            .is_err());
    }

    #[tokio::test]
    async fn handshake_sessions_can_be_persisted_and_routed() {
        let config_a = config(1, "127.0.0.1:29798".parse().unwrap());
        let config_b = config(2, "127.0.0.1:29799".parse().unwrap());
        let (client, server) = pair(config_a.clone(), config_b.clone()).await;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("peers.json");
        let mut store = PeerStore::new(PeerStoreConfig::default()).unwrap();
        store
            .record_success(client.peer().peer_id, client.peer().listen_addr, unix_now())
            .unwrap();
        store.save(&path).unwrap();
        let loaded = PeerStore::load(&path, PeerStoreConfig::default()).unwrap();
        assert!(loaded.get(&client.peer().peer_id).is_some());
        assert_eq!(server.peer().peer_id, config_b.identity.peer_id());
    }

    #[test]
    fn frame_size_configuration_is_validated() {
        let mut config = config(1, "127.0.0.1:29798".parse().unwrap());
        config.max_frame_bytes = 32;
        assert!(config.validate().is_err());
        config.max_frame_bytes = MAX_FRAME_BYTES + 1;
        assert!(config.validate().is_err());
        config.max_frame_bytes = MAX_FRAME_BYTES;
        assert!(config.validate().is_ok());
    }
}
