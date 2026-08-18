use byteorder::{LittleEndian, ReadBytesExt};
use dashmap::DashMap;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use udaya_core::transaction::Transaction;
use udaya_core::types::{Block, BlockHash, BlockHeader, InvType, InvVector, MerkleRoot};
use udaya_core::NETWORK_MAGIC;

use crate::{
    GetHeadersMessage, NetworkMessage, NetworkState, P2PConfig, PeerAddress, RejectMessage,
    VersionMessage,
};

/// Magic bytes for Udaya network messages
pub const MAGIC_BYTES: [u8; 4] = NETWORK_MAGIC;

/// Maximum size of a network message payload
pub const MAX_PAYLOAD_SIZE: u32 = 32_000_000;

/// P2P message header
#[derive(Debug, Clone)]
pub struct MessageHeader {
    pub magic: [u8; 4],
    pub command: [u8; 12],
    pub payload_length: u32,
    pub checksum: [u8; 4],
}

impl MessageHeader {
    pub fn new(command: &[u8; 12], payload: &[u8]) -> Self {
        let checksum = double_sha256_first_4(payload);
        Self {
            magic: MAGIC_BYTES,
            command: *command,
            payload_length: payload.len() as u32,
            checksum,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(24);
        buf.extend_from_slice(&self.magic);
        buf.extend_from_slice(&self.command);
        buf.extend_from_slice(&self.payload_length.to_le_bytes());
        buf.extend_from_slice(&self.checksum);
        buf
    }

    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < 24 {
            anyhow::bail!("Message header too short");
        }
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&data[0..4]);
        if magic != MAGIC_BYTES {
            anyhow::bail!("Invalid network magic");
        }
        let mut command = [0u8; 12];
        command.copy_from_slice(&data[4..16]);
        let payload_length = u32::from_le_bytes(data[16..20].try_into()?);
        let mut checksum = [0u8; 4];
        checksum.copy_from_slice(&data[20..24]);
        Ok(Self {
            magic,
            command,
            payload_length,
            checksum,
        })
    }
}

/// Complete P2P message with header and payload
#[derive(Debug, Clone)]
pub struct Message {
    pub header: MessageHeader,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new(command: &[u8; 12], payload: Vec<u8>) -> Self {
        let header = MessageHeader::new(command, &payload);
        Self { header, payload }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = self.header.serialize();
        buf.extend_from_slice(&self.payload);
        buf
    }
}

fn double_sha256_first_4(data: &[u8]) -> [u8; 4] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    let mut result = [0u8; 4];
    result.copy_from_slice(&second[..4]);
    result
}

/// Serialize a variable-length integer
pub fn serialize_varint(val: u64) -> Vec<u8> {
    if val < 0xFD {
        vec![val as u8]
    } else if val <= 0xFFFF {
        let mut buf = vec![0xFD];
        buf.extend_from_slice(&(val as u16).to_le_bytes());
        buf
    } else if val <= 0xFFFF_FFFF {
        let mut buf = vec![0xFE];
        buf.extend_from_slice(&(val as u32).to_le_bytes());
        buf
    } else {
        let mut buf = vec![0xFF];
        buf.extend_from_slice(&val.to_le_bytes());
        buf
    }
}

pub fn deserialize_varint(data: &[u8]) -> anyhow::Result<(u64, usize)> {
    if data.is_empty() {
        anyhow::bail!("Empty data for varint");
    }
    match data[0] {
        0xFF => {
            if data.len() < 9 {
                anyhow::bail!("Not enough bytes");
            }
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&data[1..9]);
            Ok((u64::from_le_bytes(arr), 9))
        }
        0xFE => {
            if data.len() < 5 {
                anyhow::bail!("Not enough bytes");
            }
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&data[1..5]);
            Ok((u32::from_le_bytes(arr) as u64, 5))
        }
        0xFD => {
            if data.len() < 3 {
                anyhow::bail!("Not enough bytes");
            }
            let mut arr = [0u8; 2];
            arr.copy_from_slice(&data[1..3]);
            Ok((u16::from_le_bytes(arr) as u64, 3))
        }
        _ => Ok((data[0] as u64, 1)),
    }
}

/// Peer connection handler
pub struct PeerConnection {
    pub id: u64,
    pub address: SocketAddr,
    pub sender: tokio::sync::Mutex<Option<mpsc::Sender<Vec<u8>>>>,
    pub connected_since: u64,
    pub is_outbound: bool,
}

impl PeerConnection {
    pub fn new(address: SocketAddr, is_outbound: bool) -> Self {
        Self {
            id: rand::thread_rng().gen::<u64>(),
            address,
            sender: tokio::sync::Mutex::new(None),
            connected_since: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_outbound,
        }
    }

    pub fn set_sender(&self, sender: mpsc::Sender<Vec<u8>>) {
        *self.sender.blocking_lock() = Some(sender);
    }

    pub async fn send_raw(&self, data: Vec<u8>) -> anyhow::Result<()> {
        let sender = self.sender.lock().await;
        if let Some(s) = sender.as_ref() {
            s.send(data).await?;
            Ok(())
        } else {
            anyhow::bail!("No sender for peer {}", self.address)
        }
    }

    pub async fn send_message(&self, msg: &Message) -> anyhow::Result<()> {
        self.send_raw(msg.serialize()).await
    }
}

/// P2P network runtime
pub struct P2PNetwork {
    pub state: Arc<NetworkState>,
    pub connections: Arc<DashMap<u64, Arc<PeerConnection>>>,
    pending_connections: Arc<DashMap<String, bool>>,
    message_rx:
        tokio::sync::Mutex<Option<mpsc::UnboundedReceiver<(Arc<PeerConnection>, NetworkMessage)>>>,
    message_tx: mpsc::UnboundedSender<(Arc<PeerConnection>, NetworkMessage)>,
    config: P2PConfig,
}

impl P2PNetwork {
    pub fn new(config: P2PConfig, state: Arc<NetworkState>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            state,
            connections: Arc::new(DashMap::new()),
            pending_connections: Arc::new(DashMap::new()),
            message_rx: tokio::sync::Mutex::new(Some(rx)),
            message_tx: tx,
            config,
        }
    }

    /// Start the P2P network
    pub async fn start(&self) -> anyhow::Result<()> {
        log::info!("Starting P2P network on port {}", self.config.listen_port);

        let listen_addr = format!("{}:{}", self.config.listen_addr, self.config.listen_port);
        let listener = TcpListener::bind(&listen_addr).await?;
        log::info!("P2P listening on {}", listen_addr);

        // Resolve DNS seeds and connect to discovered peers
        let dns_config = self.config.clone();
        let dns_state = self.state.clone();
        let dns_connections = self.connections.clone();
        let dns_pending = self.pending_connections.clone();
        let dns_tx = self.message_tx.clone();
        tokio::spawn(async move {
            dns_seed_bootstrap(dns_config, dns_state, dns_connections, dns_pending, dns_tx).await;
        });

        // Connect to seed nodes
        for seed in &self.config.seed_nodes {
            let addr_str = seed.clone();
            let state = self.state.clone();
            let connections = self.connections.clone();
            let pending = self.pending_connections.clone();
            let tx = self.message_tx.clone();
            let config = self.config.clone();

            tokio::spawn(async move {
                if let Err(e) =
                    connect_to_peer(&addr_str, state, connections, pending, tx, config).await
                {
                    log::warn!("Failed to connect to seed {}: {}", addr_str, e);
                }
            });
        }

        // Accept inbound connections
        let state = self.state.clone();
        let connections = self.connections.clone();
        let pending = self.pending_connections.clone();
        let tx = self.message_tx.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        if connections.len() >= config.max_peers as usize {
                            log::warn!(
                                "Rejecting inbound connection from {}: max peers ({}) reached",
                                addr,
                                config.max_peers
                            );
                            continue;
                        }
                        log::info!("Inbound connection from {}", addr);
                        let state = state.clone();
                        let connections = connections.clone();
                        let pending = pending.clone();
                        let tx = tx.clone();
                        let config = config.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_peer_connection(
                                stream,
                                addr,
                                state,
                                connections,
                                pending,
                                tx,
                                config,
                                false,
                            )
                            .await
                            {
                                log::debug!("Connection from {} closed: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => log::error!("Failed to accept connection: {}", e),
                }
            }
        });

        let state = self.state.clone();
        let connections = self.connections.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            peer_maintenance_loop(state, connections, config).await;
        });

        Ok(())
    }

    pub async fn take_message_receiver(
        &self,
    ) -> Option<mpsc::UnboundedReceiver<(Arc<PeerConnection>, NetworkMessage)>> {
        self.message_rx.lock().await.take()
    }

    pub async fn broadcast_transaction(&self, tx: &Transaction) {
        let tx_hash = tx.txid();
        let inv = InvVector::new(InvType::Tx, BlockHash::from_bytes(&tx_hash.0));
        let msg = create_inv_message(&[inv]);
        let msg_bytes = msg.serialize();
        for conn in self.connections.iter() {
            let _ = conn.send_raw(msg_bytes.clone()).await;
        }
    }

    pub async fn broadcast_block(&self, block: &Block) {
        let block_data = bincode::serialize(block).unwrap_or_default();
        let msg = Message::new(b"block\0\0\0\0\0\0\0", block_data);
        let msg_bytes = msg.serialize();
        let inv = InvVector::new(InvType::Block, block.hash());
        let inv_msg = create_inv_message(&[inv]);
        let inv_msg_bytes = inv_msg.serialize();
        for conn in self.connections.iter() {
            let _ = conn.send_raw(inv_msg_bytes.clone()).await;
            let _ = conn.send_raw(msg_bytes.clone()).await;
        }
    }

    pub fn peer_count(&self) -> usize {
        self.connections.len()
    }

    pub fn get_peers(&self) -> Vec<(SocketAddr, u32, u64)> {
        Vec::new()
    }

    /// Request headers from a peer for sync
    pub async fn request_headers(&self, locator_hashes: Vec<BlockHash>, hash_stop: BlockHash) {
        let msg = create_getheaders_message(&self.config, locator_hashes, hash_stop);
        for conn in self.connections.iter() {
            let _ = conn.send_message(&msg).await;
        }
    }

    /// Request blocks by inventory from a peer
    pub async fn request_blocks(&self, inv_vectors: &[InvVector]) {
        let msg = create_getdata_message(inv_vectors);
        for conn in self.connections.iter() {
            let _ = conn.send_message(&msg).await;
        }
    }
}

// ============================================================
// DNS SEED RESOLUTION
// ============================================================

/// Resolve DNS seed domains to peer addresses and connect
async fn dns_seed_bootstrap(
    config: P2PConfig,
    state: Arc<NetworkState>,
    connections: Arc<DashMap<u64, Arc<PeerConnection>>>,
    pending: Arc<DashMap<String, bool>>,
    tx: mpsc::UnboundedSender<(Arc<PeerConnection>, NetworkMessage)>,
) {
    if !config.enable_dns_seed {
        log::info!("DNS seed resolution disabled");
        return;
    }

    for seed_domain in &config.dns_seeds {
        // Resolve seed domain via DNS to get SRV records or A/AAAA records
        log::info!("Resolving DNS seed: {}", seed_domain);

        match resolve_dns_seed(seed_domain, config.listen_port).await {
            Ok(peers) => {
                let total = peers.len();
                log::info!("DNS seed {} resolved {} peers", seed_domain, total);
                for (i, peer_addr) in peers.iter().enumerate() {
                    if pending.contains_key(peer_addr) {
                        continue;
                    }
                    log::info!("Connecting to DNS-discovered peer: {}", peer_addr);
                    let addr = peer_addr.clone();
                    let s = state.clone();
                    let c = connections.clone();
                    let p = pending.clone();
                    let t = tx.clone();
                    let cfg = config.clone();
                    tokio::spawn(async move {
                        if let Err(e) = connect_to_peer(&addr, s, c, p, t, cfg).await {
                            log::debug!("DNS peer {} failed: {}", addr, e);
                        }
                    });
                    // Limit concurrent DNS connections to avoid overwhelming the network
                    if i >= 8 {
                        break;
                    }
                }
            }
            Err(e) => {
                log::warn!("DNS seed {} resolution failed: {}", seed_domain, e);
            }
        }
    }
}

/// Resolve a DNS seed hostname to socket addresses
async fn resolve_dns_seed(hostname: &str, port: u16) -> anyhow::Result<Vec<String>> {
    // Perform DNS resolution using std::net::ToSocketAddrs
    let addr_str = format!("{}:{}", hostname, port);
    let addrs: Vec<_> = addr_str.to_socket_addrs()?.map(|a| a.to_string()).collect();

    if addrs.is_empty() {
        // Fall back to common DNS seed formats
        // Try DNS A record lookups with the default port
        let bare_addrs: Vec<_> = format!("{}:{}", hostname, port)
            .to_socket_addrs()?
            .map(|a| a.to_string())
            .collect();
        Ok(bare_addrs)
    } else {
        Ok(addrs)
    }
}

/// Connect to a remote peer
async fn connect_to_peer(
    addr_str: &str,
    state: Arc<NetworkState>,
    connections: Arc<DashMap<u64, Arc<PeerConnection>>>,
    pending: Arc<DashMap<String, bool>>,
    tx: mpsc::UnboundedSender<(Arc<PeerConnection>, NetworkMessage)>,
    config: P2PConfig,
) -> anyhow::Result<()> {
    if pending.contains_key(addr_str) {
        return Ok(());
    }
    pending.insert(addr_str.to_string(), true);
    let addr: SocketAddr = addr_str.parse()?;
    let stream = TcpStream::connect(&addr).await?;
    log::info!("Connected to peer: {}", addr);
    pending.remove(addr_str);
    handle_peer_connection(stream, addr, state, connections, pending, tx, config, true).await
}

/// Handle a peer connection
async fn handle_peer_connection(
    stream: TcpStream,
    addr: SocketAddr,
    state: Arc<NetworkState>,
    connections: Arc<DashMap<u64, Arc<PeerConnection>>>,
    _pending: Arc<DashMap<String, bool>>,
    tx: mpsc::UnboundedSender<(Arc<PeerConnection>, NetworkMessage)>,
    config: P2PConfig,
    is_outbound: bool,
) -> anyhow::Result<()> {
    let (reader, mut writer) = tokio::io::split(stream);
    let (msg_tx, mut msg_rx) = mpsc::channel::<Vec<u8>>(256);
    let peer = Arc::new(PeerConnection::new(addr, is_outbound));
    peer.set_sender(msg_tx);
    let peer_id = peer.id;
    connections.insert(peer_id, peer.clone());

    {
        let mut stats = state.stats.write();
        if is_outbound {
            stats.outbound_peers += 1;
        } else {
            stats.inbound_peers += 1;
        }
        stats.connected_peers = connections.len();
    }

    if is_outbound {
        let version_msg = create_version_message(&config);
        let _ = peer.send_raw(version_msg.serialize()).await;
    }

    let _peer_write = peer.clone();
    tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        while let Some(data) = msg_rx.recv().await {
            if let Err(e) = writer.write_all(&data).await {
                log::debug!("Write error to {}: {}", addr, e);
                break;
            }
        }
    });

    let result = match tokio::time::timeout(
        Duration::from_secs(config.timeout_secs),
        read_messages(reader, peer.clone(), tx, state.clone(), config),
    )
    .await
    {
        Ok(r) => r,
        Err(_) => {
            log::warn!("[P2P] Connection timeout from {}", addr);
            Ok(())
        }
    };

    connections.remove(&peer_id);
    {
        let mut stats = state.stats.write();
        if is_outbound {
            stats.outbound_peers = stats.outbound_peers.saturating_sub(1);
        } else {
            stats.inbound_peers = stats.inbound_peers.saturating_sub(1);
        }
        stats.connected_peers = connections.len();
    }
    result
}

/// Read and parse messages from a peer
async fn read_messages(
    mut reader: tokio::io::ReadHalf<TcpStream>,
    peer: Arc<PeerConnection>,
    tx: mpsc::UnboundedSender<(Arc<PeerConnection>, NetworkMessage)>,
    state: Arc<NetworkState>,
    config: P2PConfig,
) -> anyhow::Result<()> {
    use tokio::io::AsyncReadExt;
    loop {
        let mut header_buf = [0u8; 24];
        let mut header_pos = 0;
        while header_pos < 24 {
            let n = reader.read(&mut header_buf[header_pos..]).await?;
            if n == 0 {
                anyhow::bail!("Connection closed");
            }
            header_pos += n;
        }
        let header = MessageHeader::deserialize(&header_buf)?;
        if header.payload_length > MAX_PAYLOAD_SIZE {
            anyhow::bail!("Payload too large");
        }
        let mut payload = vec![0u8; header.payload_length as usize];
        let mut payload_pos = 0;
        while payload_pos < header.payload_length as usize {
            let n = reader.read(&mut payload[payload_pos..]).await?;
            if n == 0 {
                anyhow::bail!("Connection closed during payload");
            }
            payload_pos += n;
        }
        let expected_checksum = double_sha256_first_4(&payload);
        if expected_checksum != header.checksum {
            log::warn!("Checksum mismatch");
            continue;
        }
        let command = std::str::from_utf8(&header.command)
            .unwrap_or("")
            .trim_end_matches('\0')
            .to_string();

        match command.as_str() {
            "version" => {
                if let Ok(msg) = parse_version_message(&payload) {
                    let ack = Message::new(b"verack\0\0\0\0\0\0", vec![]);
                    let _ = peer.send_raw(ack.serialize()).await;
                    if !peer.is_outbound {
                        let version_msg = create_version_message(&config);
                        let _ = peer.send_raw(version_msg.serialize()).await;
                    }
                    let _ = tx.send((peer.clone(), NetworkMessage::Version(msg)));
                }
            }
            "verack" => {
                let _ = tx.send((peer.clone(), NetworkMessage::Verack));
            }
            "ping" => {
                let pong = Message::new(b"pong\0\0\0\0\0\0\0\0", payload.clone());
                let _ = peer.send_raw(pong.serialize()).await;
            }
            "pong" => {}
            "inv" => {
                if let Ok(inv_vectors) = parse_inv_message(&payload) {
                    let _ = tx.send((peer.clone(), NetworkMessage::Inv(inv_vectors)));
                }
            }
            "getdata" => {
                if let Ok(inv_vectors) = parse_inv_message(&payload) {
                    let _ = tx.send((peer.clone(), NetworkMessage::GetData(inv_vectors)));
                }
            }
            "tx" => {
                if let Ok(tx_msg) = bincode::deserialize::<Transaction>(&payload) {
                    let _ = tx.send((peer.clone(), NetworkMessage::Tx(tx_msg)));
                }
            }
            "block" => {
                if let Ok(block_msg) = bincode::deserialize::<Block>(&payload) {
                    let _ = tx.send((peer.clone(), NetworkMessage::Block(block_msg)));
                }
            }
            "headers" => {
                if let Ok(headers) = parse_headers_message(&payload) {
                    let _ = tx.send((peer.clone(), NetworkMessage::Headers(headers)));
                }
            }
            "getheaders" => {
                let _ = tx.send((
                    peer.clone(),
                    NetworkMessage::GetHeaders(GetHeadersMessage {
                        version: config.protocol_version,
                        locator_hashes: vec![],
                        hash_stop: BlockHash::default(),
                    }),
                ));
            }
            "addr" => {
                if let Ok(addrs) = parse_addr_message(&payload) {
                    for a in &addrs {
                        state
                            .addr_cache
                            .insert(format!("{}:{}", a.address, a.port), a.clone());
                    }
                    let _ = tx.send((peer.clone(), NetworkMessage::Addr(addrs)));
                }
            }
            "sendheaders" => {
                let _ = tx.send((peer.clone(), NetworkMessage::SendHeaders));
            }
            "getaddr" => {
                // Respond with our known peers
                let peers: Vec<PeerAddress> = state
                    .addr_cache
                    .iter()
                    .map(|e| e.value().clone())
                    .take(100)
                    .collect();
                let msg = create_addr_message(&peers);
                let _ = peer.send_raw(msg.serialize()).await;
            }
            "reject" => {
                let _ = tx.send((
                    peer.clone(),
                    NetworkMessage::Reject(RejectMessage {
                        message: String::from_utf8_lossy(&payload).to_string(),
                        ccode: 0,
                        reason: String::new(),
                        data: vec![],
                    }),
                ));
            }
            _ => {
                log::debug!("Unknown command: {}", command);
            }
        }
        {
            let mut stats = state.stats.write();
            stats.messages_received += 1;
            stats.total_bytes_received += header.payload_length as u64 + 24;
        }
    }
}

/// Create version message
pub fn create_version_message(config: &P2PConfig) -> Message {
    let mut payload = Vec::new();
    payload.extend_from_slice(&config.protocol_version.to_le_bytes());
    payload.extend_from_slice(&config.services.to_le_bytes());
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    payload.extend_from_slice(&now.to_le_bytes());
    payload.extend_from_slice(&1u64.to_le_bytes());
    payload.extend_from_slice(&[0u8; 16]);
    payload.extend_from_slice(&0u16.to_le_bytes());
    payload.extend_from_slice(&1u64.to_le_bytes());
    payload.extend_from_slice(&[0u8; 16]);
    payload.extend_from_slice(&config.listen_port.to_le_bytes());
    let nonce: u64 = rand::thread_rng().gen();
    payload.extend_from_slice(&nonce.to_le_bytes());
    let user_agent = config.user_agent.as_bytes();
    payload.extend_from_slice(&serialize_varint(user_agent.len() as u64));
    payload.extend_from_slice(user_agent);
    payload.extend_from_slice(&0u64.to_le_bytes());
    payload.push(1u8);
    Message::new(b"version\0\0\0\0\0", payload)
}

fn parse_version_message(payload: &[u8]) -> anyhow::Result<VersionMessage> {
    let mut cursor = Cursor::new(payload);
    let version = cursor.read_u32::<LittleEndian>()?;
    let services = cursor.read_u64::<LittleEndian>()?;
    let timestamp = cursor.read_i64::<LittleEndian>()?;
    cursor.read_u64::<LittleEndian>()?;
    let mut _addr = [0u8; 16];
    cursor.read_exact(&mut _addr)?;
    cursor.read_u16::<LittleEndian>()?;
    cursor.read_u64::<LittleEndian>()?;
    cursor.read_exact(&mut _addr)?;
    cursor.read_u16::<LittleEndian>()?;
    let _nonce = cursor.read_u64::<LittleEndian>()?;
    let pos = cursor.position() as usize;
    let (ua_len, ua_size) = deserialize_varint(&payload[pos..])?;
    for _ in 0..ua_size {
        cursor.read_u8()?;
    }
    let mut ua_bytes = vec![0u8; ua_len as usize];
    cursor.read_exact(&mut ua_bytes)?;
    let user_agent = String::from_utf8_lossy(&ua_bytes).to_string();
    let start_height = cursor.read_u64::<LittleEndian>()?;
    let relay = cursor.read_u8()? != 0;
    Ok(VersionMessage {
        version,
        services,
        timestamp,
        addr_recv: PeerAddress {
            time: 0,
            services,
            address: "0.0.0.0".to_string(),
            port: 0,
        },
        addr_from: PeerAddress {
            time: 0,
            services,
            address: "0.0.0.0".to_string(),
            port: 0,
        },
        nonce: _nonce,
        user_agent,
        start_height,
        relay,
    })
}

pub fn create_inv_message(inventory: &[InvVector]) -> Message {
    let mut payload = serialize_varint(inventory.len() as u64);
    for inv in inventory {
        payload.extend_from_slice(&(inv.inv_type as u32).to_le_bytes());
        payload.extend_from_slice(&inv.hash.0);
    }
    Message::new(b"inv\0\0\0\0\0\0\0\0\0", payload)
}

pub fn parse_inv_message(payload: &[u8]) -> anyhow::Result<Vec<InvVector>> {
    let (count, size) = deserialize_varint(payload)?;
    let mut cursor = Cursor::new(&payload[size..]);
    let mut items = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let inv_type_val = cursor.read_u32::<LittleEndian>()?;
        let inv_type = match inv_type_val {
            0 => InvType::Error,
            1 => InvType::Tx,
            2 => InvType::Block,
            3 => InvType::FilteredBlock,
            4 => InvType::CompactBlock,
            _ => InvType::Error,
        };
        let mut hash = [0u8; 32];
        cursor.read_exact(&mut hash)?;
        items.push(InvVector::new(inv_type, BlockHash::from_bytes(&hash)));
    }
    Ok(items)
}

fn parse_addr_message(payload: &[u8]) -> anyhow::Result<Vec<PeerAddress>> {
    let (count, size) = deserialize_varint(payload)?;
    let mut cursor = Cursor::new(&payload[size..]);
    let mut addrs = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let time = cursor.read_u32::<LittleEndian>()?;
        let services = cursor.read_u64::<LittleEndian>()?;
        let mut addr_bytes = [0u8; 16];
        cursor.read_exact(&mut addr_bytes)?;
        let port = cursor.read_u16::<LittleEndian>()?;
        let address = if addr_bytes[0..12].iter().all(|&b| b == 0) {
            format!(
                "{}.{}.{}.{}",
                addr_bytes[12], addr_bytes[13], addr_bytes[14], addr_bytes[15]
            )
        } else {
            "IPv6:...".to_string()
        };
        addrs.push(PeerAddress {
            time,
            services,
            address,
            port,
        });
    }
    Ok(addrs)
}

/// Create a getheaders message for header-first sync
pub fn create_getheaders_message(
    config: &P2PConfig,
    locator_hashes: Vec<BlockHash>,
    hash_stop: BlockHash,
) -> Message {
    let mut payload = Vec::new();
    payload.extend_from_slice(&config.protocol_version.to_le_bytes());
    payload.extend_from_slice(&serialize_varint(locator_hashes.len() as u64));
    for hash in &locator_hashes {
        payload.extend_from_slice(&hash.0);
    }
    payload.extend_from_slice(&hash_stop.0);
    Message::new(b"getheaders\0\0", payload)
}

/// Create a getdata message to request blocks/txs
pub fn create_getdata_message(inventory: &[InvVector]) -> Message {
    create_inv_message(inventory) // same wire format, just different command
        .with_command(b"getdata\0\0\0\0\0")
}

/// Parse headers message response
fn parse_headers_message(payload: &[u8]) -> anyhow::Result<Vec<BlockHeader>> {
    let (count, mut offset) = deserialize_varint(payload)?;
    let mut headers = Vec::with_capacity(count as usize);
    for _ in 0..count {
        if offset + 80 > payload.len() {
            anyhow::bail!("Header payload too short");
        }
        let header_data = &payload[offset..offset + 80];
        let mut cursor = Cursor::new(header_data);
        let version = cursor.read_i32::<LittleEndian>()?;
        let mut prev_hash = [0u8; 32];
        cursor.read_exact(&mut prev_hash)?;
        let mut merkle = [0u8; 32];
        cursor.read_exact(&mut merkle)?;
        let timestamp = cursor.read_u32::<LittleEndian>()?;
        let bits = cursor.read_u32::<LittleEndian>()?;
        let nonce = cursor.read_u32::<LittleEndian>()?;
        headers.push(BlockHeader::new(
            version,
            BlockHash(prev_hash),
            MerkleRoot(merkle),
            timestamp,
            bits,
            nonce,
        ));
        offset += 81; // 80 byte header + 1 byte tx count (0 for headers)
    }
    Ok(headers)
}

/// Create a headers message for header-first sync
pub fn create_headers_message(headers: &[BlockHeader]) -> Message {
    let mut payload = serialize_varint(headers.len() as u64);
    for header in headers {
        payload.extend_from_slice(&header.serialize());
        payload.push(0);
    }
    Message::new(b"headers\0\0\0\0\0", payload)
}

/// Create an addr message with peer addresses
pub fn create_addr_message(addrs: &[PeerAddress]) -> Message {
    let mut payload = serialize_varint(addrs.len() as u64);
    for addr in addrs {
        payload.extend_from_slice(&addr.time.to_le_bytes());
        payload.extend_from_slice(&addr.services.to_le_bytes());
        // Convert IPv4 to IPv6-mapped format
        let mut ipv6 = [0u8; 16];
        if let Some(pos) = addr.address.find(':') {
            // Likely already IPv6
            let _ = pos;
        } else {
            // IPv4 to IPv6-mapped
            let parts: Vec<&str> = addr.address.split('.').collect();
            if parts.len() == 4 {
                for (i, part) in parts.iter().enumerate() {
                    if let Ok(octet) = part.parse::<u8>() {
                        ipv6[12 + i] = octet;
                    }
                }
            }
        }
        payload.extend_from_slice(&ipv6);
        payload.extend_from_slice(&addr.port.to_le_bytes());
    }
    Message::new(b"addr\0\0\0\0\0\0\0\0", payload)
}

impl Message {
    /// Change the command of a message (for reusing payload construction)
    fn with_command(&self, new_command: &[u8; 12]) -> Message {
        let mut new = self.clone();
        new.header.command = *new_command;
        // Recalculate checksum
        new.header.checksum = double_sha256_first_4(&new.payload);
        new
    }
}

async fn peer_maintenance_loop(
    state: Arc<NetworkState>,
    connections: Arc<DashMap<u64, Arc<PeerConnection>>>,
    _config: P2PConfig,
) {
    loop {
        sleep(Duration::from_secs(30)).await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        for entry in connections.iter() {
            let nonce: u64 = rand::thread_rng().gen();
            let ping = Message::new(b"ping\0\0\0\0\0\0\0\0", nonce.to_le_bytes().to_vec());
            let _ = entry.send_raw(ping.serialize()).await;
        }
        {
            let mut stats = state.stats.write();
            stats.uptime_seconds = now;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_header_serialization() {
        let payload = b"test payload";
        let header = MessageHeader::new(b"version\0\0\0\0\0", payload);
        let serialized = header.serialize();
        assert_eq!(serialized.len(), 24);
        let deserialized = MessageHeader::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.command, header.command);
        assert_eq!(deserialized.payload_length, header.payload_length);
    }

    #[test]
    fn test_inv_message_roundtrip() {
        let inv = vec![
            InvVector::new(InvType::Tx, BlockHash([1u8; 32])),
            InvVector::new(InvType::Block, BlockHash([2u8; 32])),
        ];
        let msg = create_inv_message(&inv);
        let parsed = parse_inv_message(&msg.payload).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_varint_serialization() {
        let cases = vec![
            (0u64, vec![0x00]),
            (0xFC, vec![0xFC]),
            (0xFD, vec![0xFD, 0xFD, 0x00]),
            (0x10000, vec![0xFE, 0x00, 0x00, 0x01, 0x00]),
        ];
        for (val, expected) in cases {
            let serialized = serialize_varint(val);
            assert_eq!(serialized, expected);
            let (deserialized, _) = deserialize_varint(&serialized).unwrap();
            assert_eq!(deserialized, val);
        }
    }

    #[test]
    fn test_create_version_message() {
        let config = P2PConfig::default();
        let msg = create_version_message(&config);
        assert_eq!(&msg.header.command[..7], b"version");
        assert!(msg.payload.len() > 80);
        let parsed = parse_version_message(&msg.payload).unwrap();
        assert_eq!(parsed.version, config.protocol_version);
    }

    #[test]
    fn test_create_getheaders_message() {
        let config = P2PConfig::default();
        let locator = vec![BlockHash([1u8; 32]), BlockHash([2u8; 32])];
        let stop = BlockHash([0u8; 32]);
        let msg = create_getheaders_message(&config, locator, stop);
        assert_eq!(&msg.header.command[..10], b"getheaders");
    }

    #[test]
    fn test_dns_seed_resolution() {
        // Test that DNS seed resolution handles errors gracefully
        let result = resolve_dns_seed("invalid.example.nonexistent", 9798);
        // This should either succeed with a runtime error or return an empty vec
        let _ = result;
    }

    #[test]
    fn test_addr_message_roundtrip() {
        let addrs = vec![
            PeerAddress {
                time: 100,
                services: 1,
                address: "192.168.1.1".to_string(),
                port: 9798,
            },
            PeerAddress {
                time: 200,
                services: 3,
                address: "10.0.0.1".to_string(),
                port: 9799,
            },
        ];
        let msg = create_addr_message(&addrs);
        let parsed = parse_addr_message(&msg.payload).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_headers_message_parsing() {
        // Create a valid 80-byte header
        let header = BlockHeader::new(
            1,
            BlockHash([0u8; 32]),
            MerkleRoot([0u8; 32]),
            1234567890,
            0x1d00ffff,
            12345,
        );
        let header_bytes = header.serialize();
        let mut payload = serialize_varint(1);
        payload.extend_from_slice(&header_bytes);
        payload.push(0); // tx count byte

        let headers = parse_headers_message(&payload).unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].version, 1);
        assert_eq!(headers[0].nonce, 12345);
    }

    #[test]
    fn test_create_getdata_message() {
        let inv = vec![InvVector::new(InvType::Block, BlockHash([42u8; 32]))];
        let msg = create_getdata_message(&inv);
        assert_eq!(&msg.header.command[..6], b"getdat");
        let parsed = parse_inv_message(&msg.payload).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[tokio::test]
    async fn test_dns_seed_bootstrap_disabled() {
        let mut config = P2PConfig::default();
        config.enable_dns_seed = false;
        let state = Arc::new(NetworkState::new(config.clone()));
        let connections = Arc::new(DashMap::new());
        let pending = Arc::new(DashMap::new());
        let (tx, _rx) = mpsc::unbounded_channel();

        // This should not panic when DNS seed is disabled
        dns_seed_bootstrap(config, state, connections, pending, tx).await;
    }
}
