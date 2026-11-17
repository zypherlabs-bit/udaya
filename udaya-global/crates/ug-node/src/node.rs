//! Node runtime: P2P sessions, consensus actions, commit persistence and
//! catch-up.
//!
//! The event loop is single-owner: every state mutation happens in one task.
//! Peer sessions are owned by small actor tasks that only do framing/encryption
//! and forward decoded messages to the loop. This avoids lock ordering issues
//! and guarantees that the double-sign guard is never used concurrently.

use crate::config::NodeConfig;
use crate::store::{
    append_equivocation, load_mempool, read_tip, save_mempool, BlockStore, StateSnapshot,
    StorePaths,
};
use crate::verifier::NodeVerifier;
use crate::{Error, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::JoinHandle;
use ug_consensus::{
    Action, Consensus, DoubleSignGuard, EquivocationEvidence, LocalSigner, QuorumCert, ValidatorSet,
};
use ug_core::block::{Block, BlockHeader, BLOCK_VERSION};
use ug_core::crypto::{self, Hash32};
use ug_core::genesis::Genesis;
use ug_core::mempool::Mempool;
use ug_core::state::UtxoSet;
use ug_core::tx::Transaction;
use ug_core::{ChainParams, Height};
use ug_net::peer::PeerStoreConfig;
use ug_net::{
    CommitData, LocalIdentity, Message, NetworkConfig, PeerAddress, PeerId, PeerInfo, PeerSession,
    PeerStore, MAX_PEERS_PER_MESSAGE,
};

/// Maximum number of cached peer blocks kept for commit assembly.
const MAX_CACHED_BLOCKS: usize = 128;
/// Maximum number of actions processed in one pump without progress.
const MAX_ACTION_PUMP: usize = 4096;
/// How long to wait between catch-up requests.
const SYNC_REQUEST_INTERVAL: Duration = Duration::from_millis(500);
/// Retry interval for a missing block referenced by a quorum certificate.
const BLOCK_RETRY_INTERVAL: Duration = Duration::from_secs(2);
/// Maximum missing-block retries before the node reports an error.
const MAX_BLOCK_RETRIES: u32 = 30;

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn hex_hash(hash: &Hash32) -> String {
    hex::encode(hash)
}

/// Live node status. Fields that are unavailable in read-only disk mode are
/// `None`.
#[derive(Clone, Debug, Serialize)]
pub struct NodeStatus {
    pub chain_id: String,
    pub genesis_hash: String,
    pub peer_id: Option<String>,
    pub listen_addr: Option<String>,
    pub height: Height,
    pub block_hash: String,
    pub state_root: String,
    pub supply: Option<u64>,
    pub utxos: Option<usize>,
    pub mempool_transactions: Option<usize>,
    pub mempool_bytes: Option<usize>,
    pub connected_peers: Option<usize>,
    pub validators: usize,
    pub round: Option<u32>,
    pub step: Option<String>,
    pub locked: Option<String>,
    pub timestamp: u64,
}

struct PeerLink {
    session_id: u64,
    info: PeerInfo,
    sender: mpsc::Sender<Message>,
    last_seen: Instant,
}

struct PeerConnection {
    session_id: u64,
    info: PeerInfo,
    sender: mpsc::Sender<Message>,
    dialed: Option<SocketAddr>,
}

struct PendingFinalize {
    certificate: QuorumCert,
    height: Height,
    attempts: u32,
    last_request: Instant,
}

enum Command {
    SubmitTransaction(
        Transaction,
        oneshot::Sender<std::result::Result<Hash32, String>>,
    ),
    Status(oneshot::Sender<NodeStatus>),
    Shutdown(oneshot::Sender<()>),
}

enum Event {
    PeerReady(PeerConnection),
    PeerClosed {
        session_id: u64,
    },
    Message(PeerId, Message),
    Timeout {
        height: Height,
        round: u32,
        step: ug_consensus::Step,
    },
    DialFailed(SocketAddr),
    Command(Command),
}

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// A running Udaya Global node.
pub struct Node {
    chain: ChainParams,
    genesis_hash: Hash32,
    validators: ValidatorSet,
    paths: StorePaths,
    config: NodeConfig,
    network: NetworkConfig,
    listener: Option<TcpListener>,
    store: BlockStore,
    state: UtxoSet,
    mempool: Mempool,
    peer_store: PeerStore,
    engine: Consensus,
    sessions: HashMap<PeerId, PeerLink>,
    peer_heights: HashMap<PeerId, Height>,
    known_addrs: BTreeSet<SocketAddr>,
    dialing: HashSet<SocketAddr>,
    last_dial: HashMap<SocketAddr, Instant>,
    pending_commits: BTreeMap<Height, CommitData>,
    pending_finalize: BTreeMap<Hash32, PendingFinalize>,
    cached_blocks: BTreeMap<Hash32, Block>,
    last_sync_request: Instant,
    last_ping: Instant,
    last_peer_save: Instant,
    ping_nonce: u64,
    events_tx: mpsc::Sender<Event>,
    events_rx: Option<mpsc::Receiver<Event>>,
    shutdown_tx: watch::Sender<bool>,
    tasks: Vec<JoinHandle<()>>,
}

impl Node {
    /// Open a node from disk. The key must be a genesis validator; observer
    /// mode is not implemented in this release.
    pub async fn open(
        data_dir: impl Into<PathBuf>,
        key_path: &Path,
        mut config: NodeConfig,
    ) -> Result<Self> {
        config.validate()?;
        let paths = StorePaths::new(data_dir);
        paths.ensure()?;

        let pinned = config.pinned_genesis()?;
        let genesis = Genesis::load(&paths.genesis, None, pinned)?;
        let chain = genesis.chain_params();
        chain.validate()?;
        let genesis_hash = genesis.hash();
        let validators = ValidatorSet::from_slice(genesis.validators())?;

        let seed = crate::load_seed(key_path)?;
        let signer = LocalSigner::from_seed(&seed);
        if !validators.contains(&signer.public_key()) {
            return Err(Error::Config(
                "key file is not a genesis validator; observer mode is not implemented".into(),
            ));
        }
        if config.listen.ip().is_unspecified() {
            return Err(Error::Config(
                "bind to a specific interface; advertising an unspecified address is not supported"
                    .into(),
            ));
        }
        let listener = TcpListener::bind(config.listen).await?;
        let listen_addr = listener.local_addr()?;
        config.listen = listen_addr;

        let identity = LocalIdentity::from_seed(&seed);
        let network = NetworkConfig::new(chain.clone(), genesis_hash, identity, listen_addr);
        network.validate()?;

        let store = BlockStore::open(&paths, chain.clone(), genesis.block(), &validators)?;

        let (mut state, snapshot_height) =
            match StateSnapshot::load(&paths, &chain, &genesis_hash, &store)? {
                Some(snapshot) => (snapshot.state, snapshot.height),
                None => {
                    let (state, _) = genesis.block().execute_genesis(&chain)?;
                    (state, 0)
                }
            };
        for height in snapshot_height.saturating_add(1)..=store.height() {
            let block = store.block(height)?;
            let parent = *store
                .header(height - 1)
                .ok_or_else(|| Error::Store("missing replay parent".into()))?;
            let now = unix_now().max(block.header.timestamp);
            let (next, _) = block.execute(&chain, &parent, &state, now)?;
            state = next;
        }
        let tip = store.tip_header();
        if state.state_root(&chain) != tip.state_root {
            return Err(Error::Store(
                "replayed state root does not match the tip header".into(),
            ));
        }
        state.audit_supply()?;

        let now = unix_now();
        let (mut mempool, dirty) = load_mempool(
            &paths,
            &chain,
            &state,
            store.height().saturating_add(1),
            now,
            config.mempool.to_core(),
        )?;
        let mempool_len = mempool.len();
        mempool.reconcile(&chain, &state, store.height().saturating_add(1), now);
        if dirty || !paths.mempool.exists() || mempool_len != mempool.len() {
            save_mempool(&paths, &mempool)?;
        }

        let peer_store = PeerStore::load(&paths.peers, PeerStoreConfig::default())?;
        let mut known_addrs: BTreeSet<SocketAddr> = config.bootstrap.iter().copied().collect();
        for record in peer_store.iter() {
            if !record.banned {
                if let Ok(address) = record.socket_addr() {
                    known_addrs.insert(address);
                }
            }
        }

        let guard = DoubleSignGuard::open(&chain, signer, &paths.vote_journal)?;
        let start_height = store.height().saturating_add(1);
        if guard.highest_height() > start_height {
            return Err(Error::Node(format!(
                "vote journal contains height {}, but the block tip is {}; refusing to start below a signed height",
                guard.highest_height(),
                store.height()
            )));
        }
        let start_round = guard
            .highest_round_at(start_height)
            .map(|round| round.saturating_add(1))
            .unwrap_or(0);
        let engine = Consensus::new_at_round(
            chain.clone(),
            validators.clone(),
            config.consensus.to_core(),
            start_height,
            start_round,
            guard,
        )?;

        let (events_tx, events_rx) = mpsc::channel(4096);
        let (shutdown_tx, _) = watch::channel(false);
        let now_instant = Instant::now();
        Ok(Self {
            chain,
            genesis_hash,
            validators,
            paths,
            config,
            network,
            listener: Some(listener),
            store,
            state,
            mempool,
            peer_store,
            engine,
            sessions: HashMap::new(),
            peer_heights: HashMap::new(),
            known_addrs,
            dialing: HashSet::new(),
            last_dial: HashMap::new(),
            pending_commits: BTreeMap::new(),
            pending_finalize: BTreeMap::new(),
            cached_blocks: BTreeMap::new(),
            last_sync_request: now_instant,
            last_ping: now_instant,
            last_peer_save: now_instant,
            ping_nonce: 1,
            events_tx,
            events_rx: Some(events_rx),
            shutdown_tx,
            tasks: Vec::new(),
        })
    }

    pub fn peer_id(&self) -> PeerId {
        self.network.identity.peer_id()
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.network.listen_addr
    }

    pub fn genesis_hash(&self) -> Hash32 {
        self.genesis_hash
    }

    /// Move the node into its runtime task and return a control handle.
    pub fn spawn(mut self) -> NodeHandle {
        let events_tx = self.events_tx.clone();
        let shutdown_tx = self.shutdown_tx.clone();
        let peer_id = self.peer_id();
        let listen_addr = self.listen_addr();
        let mut events_rx = self
            .events_rx
            .take()
            .expect("node can only be spawned once");
        let join = tokio::spawn(async move { self.run(&mut events_rx).await });
        NodeHandle {
            peer_id,
            listen_addr,
            events_tx,
            shutdown_tx,
            join: Some(join),
        }
    }

    fn status(&self) -> NodeStatus {
        let tip = self.store.tip_header();
        NodeStatus {
            chain_id: self.chain.chain_id.clone(),
            genesis_hash: hex_hash(&self.genesis_hash),
            peer_id: Some(self.peer_id().to_string()),
            listen_addr: Some(self.listen_addr().to_string()),
            height: self.store.height(),
            block_hash: hex_hash(&tip.hash(&self.chain)),
            state_root: hex_hash(&self.state.state_root(&self.chain)),
            supply: Some(self.state.total_value()),
            utxos: Some(self.state.len()),
            mempool_transactions: Some(self.mempool.len()),
            mempool_bytes: Some(self.mempool.total_bytes()),
            connected_peers: Some(self.sessions.len()),
            validators: self.validators.len(),
            round: Some(self.engine.round()),
            step: Some(self.engine.step().name().to_string()),
            locked: self.engine.locked().map(|(_, hash)| hex_hash(&hash)),
            timestamp: unix_now(),
        }
    }

    async fn run(&mut self, events_rx: &mut mpsc::Receiver<Event>) -> Result<()> {
        let mut shutdown = self.shutdown_tx.subscribe();
        self.start_network_tasks();
        self.dial_known();
        self.pump_actions()?;

        let mut ticker = tokio::time::interval(Duration::from_secs(1));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut shutdown_ack: Option<oneshot::Sender<()>> = None;

        loop {
            tokio::select! {
                _ = shutdown.changed() => break,
                _ = ticker.tick() => {
                    if let Err(error) = self.tick() {
                        eprintln!("ug-node: tick error: {error}");
                        return Err(error);
                    }
                }
                event = events_rx.recv() => {
                    let Some(event) = event else { break };
                    match self.handle_event(event) {
                        Ok(Some(ack)) => {
                            shutdown_ack = Some(ack);
                            break;
                        }
                        Ok(None) => {}
                        Err(error) => {
                            eprintln!("ug-node: fatal error: {error}");
                            return Err(error);
                        }
                    }
                }
            }
        }

        self.shutdown_cleanup();
        if let Some(ack) = shutdown_ack {
            let _ = ack.send(());
        }
        Ok(())
    }

    fn start_network_tasks(&mut self) {
        let Some(listener) = self.listener.take() else {
            return;
        };
        let network = self.network.clone();
        let events = self.events_tx.clone();
        let mut shutdown = self.shutdown_tx.subscribe();
        let accept_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.changed() => break,
                    accepted = listener.accept() => {
                        let Ok((stream, _remote)) = accepted else { continue };
                        let network = network.clone();
                        let events = events.clone();
                        let shutdown = shutdown.clone();
                        tokio::spawn(async move {
                            if let Ok(session) = ug_net::accept(stream, &network).await { run_peer(session, events, shutdown).await }
                        });
                    }
                }
            }
        });
        self.tasks.push(accept_task);
    }

    fn shutdown_cleanup(&mut self) {
        let _ = self.shutdown_tx.send(true);
        for task in self.tasks.drain(..) {
            task.abort();
        }
        self.sessions.clear();
        let _ = save_mempool(&self.paths, &self.mempool);
        let _ = self.peer_store.save(&self.paths.peers);
    }

    fn handle_event(&mut self, event: Event) -> Result<Option<oneshot::Sender<()>>> {
        match event {
            Event::PeerReady(connection) => {
                self.register_peer(connection);
            }
            Event::PeerClosed { session_id } => {
                let peer = self
                    .sessions
                    .iter()
                    .find(|(_, link)| link.session_id == session_id)
                    .map(|(peer, _)| *peer);
                if let Some(peer) = peer {
                    self.sessions.remove(&peer);
                    self.peer_heights.remove(&peer);
                }
            }
            Event::Message(peer, message) => {
                if let Some(link) = self.sessions.get_mut(&peer) {
                    link.last_seen = Instant::now();
                }
                self.handle_message(peer, message)?;
            }
            Event::Timeout {
                height,
                round,
                step,
            } => {
                self.engine.on_timeout(height, round, step)?;
                self.pump_actions()?;
            }
            Event::DialFailed(address) => {
                self.dialing.remove(&address);
            }
            Event::Command(command) => match command {
                Command::SubmitTransaction(transaction, ack) => {
                    let result = self.submit_transaction(transaction, true);
                    let _ = ack.send(result);
                }
                Command::Status(ack) => {
                    let _ = ack.send(self.status());
                }
                Command::Shutdown(ack) => return Ok(Some(ack)),
            },
        }
        Ok(None)
    }

    fn register_peer(&mut self, connection: PeerConnection) {
        if let Some(address) = connection.dialed {
            self.dialing.remove(&address);
        }
        let peer = connection.info.peer_id;
        if peer == self.peer_id() || self.sessions.contains_key(&peer) {
            return;
        }
        if self.sessions.len() >= self.config.max_peers {
            return;
        }
        self.known_addrs.insert(connection.info.listen_addr);
        let _ = self
            .peer_store
            .record_success(peer, connection.info.listen_addr, unix_now());
        let _ = self.peer_store.save(&self.paths.peers);
        self.sessions.insert(
            peer,
            PeerLink {
                session_id: connection.session_id,
                info: connection.info,
                sender: connection.sender,
                last_seen: Instant::now(),
            },
        );
        self.peer_heights.insert(peer, 0);
        self.send_to(peer, Message::GetPeers);
        self.request_commits(true);
    }

    fn send_to(&self, peer: PeerId, message: Message) {
        if let Some(link) = self.sessions.get(&peer) {
            let _ = link.sender.try_send(message);
        }
    }

    fn broadcast(&self, message: Message) {
        let peers: Vec<PeerId> = self.sessions.keys().copied().collect();
        for peer in peers {
            self.send_to(peer, message.clone());
        }
    }

    fn request_commits(&mut self, force: bool) {
        if !force && self.last_sync_request.elapsed() < SYNC_REQUEST_INTERVAL {
            return;
        }
        self.last_sync_request = Instant::now();
        self.broadcast(Message::GetCommits {
            from: self.store.height().saturating_add(1),
            limit: self.config.sync_limit,
        });
    }

    fn handle_message(&mut self, from: PeerId, message: Message) -> Result<()> {
        match message {
            Message::Ping { nonce } => self.send_to(from, Message::Pong { nonce }),
            Message::Pong { .. } => {}
            Message::GetPeers => {
                let mut peers = Vec::new();
                for record in self.peer_store.iter() {
                    if peers.len() >= MAX_PEERS_PER_MESSAGE {
                        break;
                    }
                    if record.peer_id() == self.peer_id() {
                        continue;
                    }
                    if let Ok(address) = record.socket_addr() {
                        peers.push(PeerAddress {
                            peer_id: record.peer_id(),
                            address,
                        });
                    }
                }
                self.send_to(from, Message::Peers { peers });
            }
            Message::Peers { peers } => {
                for peer in peers {
                    if peer.address != self.listen_addr() {
                        self.known_addrs.insert(peer.address);
                    }
                }
                if self.known_addrs.len() > 512 {
                    let excess: Vec<SocketAddr> =
                        self.known_addrs.iter().skip(512).copied().collect();
                    for address in excess {
                        self.known_addrs.remove(&address);
                    }
                }
                self.dial_known();
            }
            Message::NewTransaction { transaction } => {
                let txid = transaction.txid(&self.chain);
                if self.mempool.contains(&txid) {
                    return Ok(());
                }
                match self.submit_transaction(transaction.clone(), false) {
                    Ok(_) => self.broadcast(Message::NewTransaction { transaction }),
                    Err(_) => {
                        // Invalid, conflicting or duplicate transactions are
                        // dropped; they are never relayed.
                    }
                }
            }
            Message::NewProposal { proposal } => {
                self.observe_peer_height(from, proposal.height);
                if proposal.height < self.engine.height() {
                    return Ok(());
                }
                if proposal.height > self.engine.height() {
                    self.request_commits(false);
                    return Ok(());
                }
                let tip = *self.store.tip_header();
                let verifier = NodeVerifier {
                    chain: &self.chain,
                    parent: &tip,
                    parent_state: &self.state,
                    now_unix: unix_now(),
                };
                self.engine.on_proposal_verified(proposal, &verifier)?;
                self.pump_actions()?;
            }
            Message::NewVote { vote } => {
                self.observe_peer_height(from, vote.height);
                if vote.height > self.engine.height() {
                    self.request_commits(false);
                }
                if let Err(error) = self.engine.on_vote(vote) {
                    // An invalid vote is a peer/protocol fault, not a local
                    // fatal error. It is not relayed.
                    eprintln!("ug-node: rejected vote from {from}: {error}");
                    return Ok(());
                }
                self.pump_actions()?;
            }
            Message::NewCommit { commit } => {
                self.observe_peer_height(from, commit.block.header.height);
                self.accept_commit(commit, true)?;
                self.pump_actions()?;
            }
            Message::GetCommits { from: start, limit } => {
                let limit = (limit as usize).min(self.config.sync_limit as usize).max(1);
                let commits = self.store.commits_from(start, limit)?;
                self.send_to(from, Message::Commits { commits });
            }
            Message::Commits { commits } => {
                for commit in commits {
                    self.observe_peer_height(from, commit.block.header.height);
                    self.buffer_or_apply_commit(commit)?;
                }
                self.drain_pending_commits()?;
                self.pump_actions()?;
            }
            Message::GetBlock { block_hash } => {
                if let Some(block) = self.store.block_by_hash(&block_hash)? {
                    self.send_to(from, Message::Block { block });
                }
            }
            Message::Block { block } | Message::NewBlock { block } => {
                self.on_block(block)?;
            }
            Message::GetBlocks { locator, stop } => {
                let blocks =
                    self.store
                        .blocks_after(&locator, &stop, ug_net::MAX_BLOCKS_PER_MESSAGE)?;
                self.send_to(from, Message::Blocks { blocks });
            }
            Message::Blocks { blocks } => {
                for block in blocks {
                    self.on_block(block)?;
                }
            }
        }
        Ok(())
    }

    fn observe_peer_height(&mut self, peer: PeerId, height: Height) {
        let entry = self.peer_heights.entry(peer).or_insert(0);
        if height > *entry {
            *entry = height;
        }
    }

    fn submit_transaction(
        &mut self,
        transaction: Transaction,
        relay: bool,
    ) -> std::result::Result<Hash32, String> {
        let height = self.store.height().saturating_add(1);
        let now = unix_now();
        match self
            .mempool
            .submit(&self.chain, &self.state, transaction.clone(), height, now)
        {
            Ok(txid) => {
                save_mempool(&self.paths, &self.mempool).map_err(|error| error.to_string())?;
                if relay {
                    self.broadcast(Message::NewTransaction { transaction });
                }
                Ok(txid)
            }
            Err(error) => Err(error.to_string()),
        }
    }

    fn pump_actions(&mut self) -> Result<()> {
        for _ in 0..MAX_ACTION_PUMP {
            let actions = self.engine.drain_actions();
            if actions.is_empty() {
                return Ok(());
            }
            for action in actions {
                self.handle_action(action)?;
            }
        }
        Err(Error::Node(
            "consensus action pump did not converge; refusing to spin".into(),
        ))
    }

    fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Propose { height, round } => {
                if height == self.engine.height()
                    && round == self.engine.round()
                    && self.engine.is_proposer()
                {
                    self.propose_block()?;
                }
            }
            Action::ScheduleTimeout {
                height,
                round,
                step,
                delay_ms,
            } => {
                let events = self.events_tx.clone();
                let mut shutdown = self.shutdown_tx.subscribe();
                tokio::spawn(async move {
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_millis(delay_ms)) => {
                            let _ = events
                                .send(Event::Timeout { height, round, step })
                                .await;
                        }
                        _ = shutdown.changed() => {}
                    }
                });
            }
            Action::BroadcastProposal(proposal) => {
                self.broadcast(Message::NewProposal { proposal });
            }
            Action::BroadcastVote(vote) => {
                self.broadcast(Message::NewVote { vote });
            }
            Action::Commit {
                height,
                block_hash,
                block,
                certificate,
                ..
            } => {
                if let Some(block) = block {
                    self.accept_commit(CommitData { block, certificate }, true)?;
                } else {
                    self.request_final_block(height, block_hash, certificate);
                }
            }
            Action::RequestBlock { block_hash, .. } => {
                self.broadcast(Message::GetBlock { block_hash });
            }
            Action::RejectProposal { reason, .. } => {
                eprintln!("ug-node: rejected proposal: {reason}");
            }
            Action::Equivocation(evidence) => {
                self.record_equivocation(&evidence)?;
            }
        }
        Ok(())
    }

    fn propose_block(&mut self) -> Result<()> {
        let height = self.engine.height();
        let round = self.engine.round();
        let parent = *self.store.tip_header();
        if parent.height.saturating_add(1) != height {
            return Err(Error::Node(format!(
                "proposal height {height} does not follow tip {}",
                parent.height
            )));
        }
        let transactions =
            self.mempool
                .template(&self.chain, &self.state, height, self.chain.max_block_bytes);
        let mut working = self.state.clone();
        for transaction in &transactions {
            working.apply_transaction(&self.chain, transaction, height)?;
        }
        let txids: Vec<Hash32> = transactions
            .iter()
            .map(|transaction| transaction.txid(&self.chain))
            .collect();
        let timestamp = unix_now().max(parent.timestamp);
        let block = Block {
            header: BlockHeader {
                version: BLOCK_VERSION,
                height,
                previous_hash: parent.hash(&self.chain),
                timestamp,
                tx_root: crypto::merkle_root(&txids),
                state_root: working.state_root(&self.chain),
                proposer: self.engine.validator(),
                round,
            },
            transactions,
        };
        let verifier = NodeVerifier {
            chain: &self.chain,
            parent: &parent,
            parent_state: &self.state,
            now_unix: unix_now(),
        };
        if let Err(error) = self.engine.propose(block, &verifier) {
            eprintln!("ug-node: could not propose block: {error}");
        }
        Ok(())
    }

    fn request_final_block(&mut self, height: Height, block_hash: Hash32, certificate: QuorumCert) {
        if let Some(block) = self.cached_blocks.get(&block_hash).cloned() {
            if block.header.height == height {
                if let Err(error) = self.accept_commit(CommitData { block, certificate }, true) {
                    eprintln!("ug-node: failed to apply cached final block: {error}");
                }
                return;
            }
        }
        let pending = PendingFinalize {
            certificate,
            height,
            attempts: 0,
            last_request: Instant::now(),
        };
        self.pending_finalize.insert(block_hash, pending);
        self.broadcast(Message::GetBlock { block_hash });
    }

    fn on_block(&mut self, block: Block) -> Result<()> {
        let hash = block.hash(&self.chain);
        let height = block.header.height;
        if height <= self.store.height() {
            if self.store.hash_at(height) == Some(hash) {
                return Ok(());
            }
            return Err(Error::Node(format!(
                "received block {height} conflicts with the finalized chain"
            )));
        }
        if let Some(pending) = self.pending_finalize.remove(&hash) {
            if pending.height == height {
                self.accept_commit(
                    CommitData {
                        block: block.clone(),
                        certificate: pending.certificate,
                    },
                    true,
                )?;
                self.drain_pending_commits()?;
                return Ok(());
            }
            self.pending_finalize.insert(hash, pending);
        }
        if self.cached_blocks.len() >= MAX_CACHED_BLOCKS {
            if let Some(first) = self.cached_blocks.keys().next().copied() {
                self.cached_blocks.remove(&first);
            }
        }
        self.cached_blocks.insert(hash, block);
        if height > self.store.height().saturating_add(1) {
            self.request_commits(false);
        }
        Ok(())
    }

    fn verify_commit(&self, commit: &CommitData) -> std::result::Result<(), String> {
        let hash = commit.block.hash(&self.chain);
        commit
            .certificate
            .verify(&self.chain, &self.validators)
            .map_err(|error| error.to_string())?;
        if commit.certificate.height != commit.block.header.height
            || commit.certificate.round != commit.block.header.round
            || commit.certificate.block_hash != hash
        {
            return Err("certificate does not bind its block".into());
        }
        if commit.block.header.height == 0 {
            return Err("genesis cannot be delivered as a commit".into());
        }
        Ok(())
    }

    fn accept_commit(&mut self, commit: CommitData, relay: bool) -> Result<()> {
        if let Err(error) = self.verify_commit(&commit) {
            eprintln!("ug-node: ignored invalid commit certificate: {error}");
            return Ok(());
        }
        let height = commit.block.header.height;
        let hash = commit.block.hash(&self.chain);
        if height <= self.store.height() {
            if self.store.hash_at(height) == Some(hash) {
                return Ok(());
            }
            return Err(Error::Node(format!(
                "two conflicting final blocks at height {height}"
            )));
        }
        if height == self.store.height().saturating_add(1) {
            self.apply_contiguous(commit, relay)?;
            self.drain_pending_commits()?;
            return Ok(());
        }
        if let Some(existing) = self.pending_commits.get(&height) {
            if existing.block.hash(&self.chain) != hash {
                return Err(Error::Node(format!(
                    "two conflicting final blocks buffered at height {height}"
                )));
            }
            return Ok(());
        }
        if self.pending_commits.len() >= self.config.max_pending_commits {
            if let Some((&highest, _)) = self.pending_commits.iter().next_back() {
                if height > highest {
                    return Ok(());
                }
                self.pending_commits.remove(&highest);
            }
        }
        self.pending_commits.insert(height, commit);
        self.request_commits(false);
        Ok(())
    }

    fn buffer_or_apply_commit(&mut self, commit: CommitData) -> Result<()> {
        self.accept_commit(commit, false)
    }

    fn drain_pending_commits(&mut self) -> Result<()> {
        loop {
            let next = self.store.height().saturating_add(1);
            let Some(commit) = self.pending_commits.remove(&next) else {
                return Ok(());
            };
            self.apply_contiguous(commit, true)?;
        }
    }

    fn apply_contiguous(&mut self, commit: CommitData, relay: bool) -> Result<()> {
        let CommitData { block, certificate } = commit;
        let height = block.header.height;
        if height != self.store.height().saturating_add(1) {
            return Err(Error::Node(format!(
                "commit {height} is not contiguous with tip {}",
                self.store.height()
            )));
        }
        if let Err(error) = self.verify_commit(&CommitData {
            block: block.clone(),
            certificate: certificate.clone(),
        }) {
            return Err(Error::Node(format!(
                "refusing to persist an unverified commit: {error}"
            )));
        }
        let parent = *self.store.tip_header();
        let block_hash = block.hash(&self.chain);
        let now = unix_now().max(block.header.timestamp);
        let (next_state, _receipt) = block.execute(&self.chain, &parent, &self.state, now)?;

        // Durable order: block + QC, then state snapshot. The block is the
        // source of truth; startup replays anything the snapshot is missing.
        self.store.append(&block, &certificate)?;
        self.state = next_state;
        StateSnapshot::save(
            &self.paths,
            &self.chain,
            &self.genesis_hash,
            height,
            block_hash,
            &self.state,
        )?;
        self.mempool
            .reconcile(&self.chain, &self.state, height.saturating_add(1), now);
        save_mempool(&self.paths, &self.mempool)?;
        self.pending_finalize.remove(&block_hash);

        if self.engine.height() < height.saturating_add(1) {
            self.engine.fast_forward(height.saturating_add(1))?;
        } else if self.engine.height() == height.saturating_add(1) {
            // The engine already advanced after its own decision.
        } else {
            return Err(Error::Node(format!(
                "consensus engine at height {} is ahead of committed height {height}",
                self.engine.height()
            )));
        }

        if relay {
            self.broadcast(Message::NewCommit {
                commit: CommitData { block, certificate },
            });
        }
        Ok(())
    }

    fn record_equivocation(&mut self, evidence: &EquivocationEvidence) -> Result<()> {
        let line = serde_json::json!({
            "validator": hex_hash(&evidence.validator),
            "vote_type": evidence.vote_type.name(),
            "height": evidence.height,
            "round": evidence.round,
            "first_block": hex_hash(&evidence.first.block_hash),
            "second_block": hex_hash(&evidence.second.block_hash),
            "timestamp": unix_now(),
        });
        append_equivocation(&self.paths, &serde_json::to_string(&line)?)?;
        eprintln!(
            "ug-node: EQUIVOCATION by {} at height {} round {}",
            hex_hash(&evidence.validator),
            evidence.height,
            evidence.round
        );
        Ok(())
    }

    fn tick(&mut self) -> Result<()> {
        if self.last_ping.elapsed() >= Duration::from_secs(self.config.ping_interval_secs) {
            self.last_ping = Instant::now();
            self.ping_nonce = self.ping_nonce.wrapping_add(1);
            self.broadcast(Message::Ping {
                nonce: self.ping_nonce,
            });
        }

        let timeout = Duration::from_secs(self.config.peer_timeout_secs);
        let stale: Vec<PeerId> = self
            .sessions
            .iter()
            .filter(|(_, link)| link.last_seen.elapsed() > timeout)
            .map(|(peer, _)| *peer)
            .collect();
        for peer in stale {
            self.sessions.remove(&peer);
            self.peer_heights.remove(&peer);
        }

        let now = Instant::now();
        let mut retry = Vec::new();
        let mut failed = None;
        for (hash, pending) in &mut self.pending_finalize {
            if now.duration_since(pending.last_request) >= BLOCK_RETRY_INTERVAL {
                pending.attempts = pending.attempts.saturating_add(1);
                pending.last_request = now;
                if pending.attempts > MAX_BLOCK_RETRIES {
                    failed = Some(*hash);
                    break;
                }
                retry.push(*hash);
            }
        }
        if let Some(hash) = failed {
            self.pending_finalize.remove(&hash);
            return Err(Error::Node(format!(
                "could not fetch final block {} after {} attempts",
                hex_hash(&hash),
                MAX_BLOCK_RETRIES
            )));
        }
        for hash in retry {
            self.broadcast(Message::GetBlock { block_hash: hash });
        }

        if self
            .peer_heights
            .values()
            .any(|height| *height > self.store.height())
        {
            self.request_commits(false);
        }

        if !self.mempool.expire(unix_now()).is_empty() {
            save_mempool(&self.paths, &self.mempool)?;
        }

        if self.last_peer_save.elapsed() >= Duration::from_secs(30) {
            self.last_peer_save = Instant::now();
            let _ = self.peer_store.save(&self.paths.peers);
        }

        self.dial_known();
        Ok(())
    }

    fn dial_known(&mut self) {
        let connected: HashSet<SocketAddr> = self
            .sessions
            .values()
            .flat_map(|link| [link.info.listen_addr, link.info.remote_addr])
            .collect();
        let now = Instant::now();
        let candidates: Vec<SocketAddr> = self
            .known_addrs
            .iter()
            .copied()
            .filter(|address| {
                *address != self.listen_addr()
                    && !connected.contains(address)
                    && !self.dialing.contains(address)
                    && self
                        .last_dial
                        .get(address)
                        .map(|instant| {
                            now.duration_since(*instant)
                                >= Duration::from_secs(self.config.reconnect_interval_secs)
                        })
                        .unwrap_or(true)
            })
            .take(4)
            .collect();
        for address in candidates {
            self.dialing.insert(address);
            self.last_dial.insert(address, now);
            let network = self.network.clone();
            let events = self.events_tx.clone();
            let shutdown = self.shutdown_tx.subscribe();
            tokio::spawn(async move {
                dial_peer(address, network, events, shutdown).await;
            });
        }
    }
}

/// Handle used by the CLI and tests to submit transactions, query status and
/// shut the node down cleanly.
pub struct NodeHandle {
    pub peer_id: PeerId,
    pub listen_addr: SocketAddr,
    events_tx: mpsc::Sender<Event>,
    shutdown_tx: watch::Sender<bool>,
    join: Option<JoinHandle<Result<()>>>,
}

impl NodeHandle {
    pub async fn submit_transaction(&self, transaction: Transaction) -> Result<Hash32> {
        let (ack_tx, ack_rx) = oneshot::channel();
        self.events_tx
            .send(Event::Command(Command::SubmitTransaction(
                transaction,
                ack_tx,
            )))
            .await
            .map_err(|_| Error::Node("node event loop is stopped".into()))?;
        match ack_rx.await {
            Ok(Ok(txid)) => Ok(txid),
            Ok(Err(error)) => Err(Error::Node(error)),
            Err(_) => Err(Error::Node("node dropped the transaction request".into())),
        }
    }

    pub async fn status(&self) -> Result<NodeStatus> {
        let (ack_tx, ack_rx) = oneshot::channel();
        self.events_tx
            .send(Event::Command(Command::Status(ack_tx)))
            .await
            .map_err(|_| Error::Node("node event loop is stopped".into()))?;
        ack_rx
            .await
            .map_err(|_| Error::Node("node dropped the status request".into()))
    }

    pub async fn shutdown(mut self) -> Result<()> {
        let (ack_tx, ack_rx) = oneshot::channel();
        let _ = self
            .events_tx
            .send(Event::Command(Command::Shutdown(ack_tx)))
            .await;
        let _ = ack_rx.await;
        let _ = self.shutdown_tx.send(true);
        let Some(join) = self.join.take() else {
            return Ok(());
        };
        match join.await {
            Ok(result) => result,
            Err(error) => Err(Error::Join(error)),
        }
    }
}

impl Drop for NodeHandle {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(true);
    }
}

async fn run_peer(
    mut session: PeerSession,
    events: mpsc::Sender<Event>,
    mut shutdown: watch::Receiver<bool>,
) {
    let session_id = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let info = session.peer().clone();
    let peer = info.peer_id;
    let (sender, mut outbound) = mpsc::channel::<Message>(256);
    if events
        .send(Event::PeerReady(PeerConnection {
            session_id,
            info,
            sender,
            dialed: None,
        }))
        .await
        .is_err()
    {
        return;
    }
    loop {
        tokio::select! {
            _ = shutdown.changed() => break,
            incoming = session.recv() => {
                match incoming {
                    Ok(message) => {
                        if events.send(Event::Message(peer, message)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            outgoing = outbound.recv() => {
                match outgoing {
                    Some(message) => {
                        if session.send(&message).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }
    let _ = events.send(Event::PeerClosed { session_id }).await;
}

async fn dial_peer(
    address: SocketAddr,
    network: NetworkConfig,
    events: mpsc::Sender<Event>,
    mut shutdown: watch::Receiver<bool>,
) {
    if *shutdown.borrow() {
        return;
    }
    let result = tokio::select! {
        _ = shutdown.changed() => return,
        result = ug_net::connect(&network, address) => result,
    };
    let session = match result {
        Ok(session) => session,
        Err(_) => {
            let _ = events.send(Event::DialFailed(address)).await;
            return;
        }
    };
    let session_id = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let info = session.peer().clone();
    let peer = info.peer_id;
    let (sender, mut outbound) = mpsc::channel::<Message>(256);
    if events
        .send(Event::PeerReady(PeerConnection {
            session_id,
            info,
            sender,
            dialed: Some(address),
        }))
        .await
        .is_err()
    {
        return;
    }
    let mut session = session;
    loop {
        tokio::select! {
            _ = shutdown.changed() => break,
            incoming = session.recv() => {
                match incoming {
                    Ok(message) => {
                        if events.send(Event::Message(peer, message)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            outgoing = outbound.recv() => {
                match outgoing {
                    Some(message) => {
                        if session.send(&message).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }
    let _ = events.send(Event::PeerClosed { session_id }).await;
}

/// Read-only status from disk. Never opens the network or the vote journal.
pub fn read_disk_status(data_dir: &Path) -> Result<NodeStatus> {
    let paths = StorePaths::new(data_dir);
    let genesis = Genesis::load(&paths.genesis, None, None)?;
    let chain = genesis.chain_params();
    let genesis_hash = genesis.hash();
    let validators = ValidatorSet::from_slice(genesis.validators())?;
    let store = BlockStore::open(&paths, chain.clone(), genesis.block(), &validators)?;
    let tip = *store.tip_header();
    let mut supply = None;
    let mut utxos = None;
    if paths.state.exists() {
        if let Some(snapshot) = StateSnapshot::load(&paths, &chain, &genesis_hash, &store)? {
            if snapshot.height == store.height() {
                supply = Some(snapshot.state.total_value());
                utxos = Some(snapshot.state.len());
            }
        }
    }
    let mempool = if paths.mempool.exists() {
        let meta = std::fs::metadata(&paths.mempool)?;
        if meta.len() > crate::store::MAX_MEMPOOL_FILE_BYTES {
            return Err(Error::Store("mempool snapshot is too large".into()));
        }
        let data = std::fs::read(&paths.mempool)?;
        let entries = Mempool::snapshot_decode(&chain, &data, 1_000_000)?;
        Some(entries.len())
    } else {
        None
    };
    let block_hash = hex_hash(&tip.hash(&chain));
    let state_root = hex_hash(&tip.state_root);
    Ok(NodeStatus {
        chain_id: chain.chain_id.clone(),
        genesis_hash: hex_hash(&genesis_hash),
        peer_id: None,
        listen_addr: None,
        height: store.height(),
        block_hash,
        state_root,
        supply,
        utxos,
        mempool_transactions: mempool,
        mempool_bytes: None,
        connected_peers: None,
        validators: validators.len(),
        round: None,
        step: None,
        locked: None,
        timestamp: unix_now(),
    })
}

/// Fully replay and verify a data directory without mutating it.
pub fn verify_data_dir(data_dir: &Path) -> Result<NodeStatus> {
    let paths = StorePaths::new(data_dir);
    let genesis = Genesis::load(&paths.genesis, None, None)?;
    let chain = genesis.chain_params();
    let genesis_hash = genesis.hash();
    let validators = ValidatorSet::from_slice(genesis.validators())?;
    let store = BlockStore::open(&paths, chain.clone(), genesis.block(), &validators)?;

    // Validate any existing snapshot first.
    let _ = StateSnapshot::load(&paths, &chain, &genesis_hash, &store)?;

    let (mut state, _) = genesis.block().execute_genesis(&chain)?;
    for height in 1..=store.height() {
        let block = store.block(height)?;
        let parent = *store
            .header(height - 1)
            .ok_or_else(|| Error::Store("missing verification parent".into()))?;
        let now = unix_now().max(block.header.timestamp);
        let (next, _) = block.execute(&chain, &parent, &state, now)?;
        state = next;
    }
    let tip = store.tip_header();
    if state.state_root(&chain) != tip.state_root {
        return Err(Error::Store("full replay state root mismatch".into()));
    }
    let supply = state.audit_supply()?;
    Ok(NodeStatus {
        chain_id: chain.chain_id.clone(),
        genesis_hash: hex_hash(&genesis_hash),
        peer_id: None,
        listen_addr: None,
        height: store.height(),
        block_hash: hex_hash(&tip.hash(&chain)),
        state_root: hex_hash(&tip.state_root),
        supply: Some(supply),
        utxos: Some(state.len()),
        mempool_transactions: None,
        mempool_bytes: None,
        connected_peers: None,
        validators: validators.len(),
        round: None,
        step: None,
        locked: None,
        timestamp: unix_now(),
    })
}

/// Read only the tip of a data directory. Kept for tooling that should not
/// load the full UTXO set.
pub fn tip_of(data_dir: &Path) -> Result<Option<(Height, Hash32)>> {
    let paths = StorePaths::new(data_dir);
    let genesis = Genesis::load(&paths.genesis, None, None)?;
    read_tip(&paths, &genesis.chain_params())
}

// Re-export commonly used consensus constants for CLI/tooling.
pub use ug_consensus::Step as ConsensusStep;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_time_is_plausible() {
        assert!(unix_now() > 1_700_000_000);
    }

    #[test]
    fn hex_hash_is_stable() {
        assert_eq!(hex_hash(&[0xAB; 32]), "ab".repeat(32));
    }
}
