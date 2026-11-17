//! Durable block, state and mempool storage.
//!
//! Layout under the data directory:
//!
//! ```text
//! genesis.json            pinned genesis configuration
//! node.json               node configuration
//! vote-journal.bin        append-only double-sign journal
//! peers.json              peer metadata
//! mempool.bin             pending transaction snapshot
//! state.bin               UTXO snapshot at a committed height
//! equivocations.jsonl     append-only equivocation evidence
//! blocks/00000000000000000001.blk
//! blocks/00000000000000000001.qc
//! ```
//!
//! Crash ordering is deliberate: a block and its quorum certificate are
//! fsynced before the state snapshot that includes it. On startup the state
//! snapshot is checked against the block header, then all later blocks are
//! re-executed. A snapshot that fails its state-root or supply audit is
//! reported as corruption, never silently ignored.

use crate::{Error, Result};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use ug_consensus::{QuorumCert, ValidatorSet};
use ug_core::block::{Block, BlockHeader};
use ug_core::codec::{Decoder, Encoder};
use ug_core::mempool::{Mempool, MempoolConfig};
use ug_core::state::UtxoSet;
use ug_core::tx::Transaction;
use ug_core::{ChainParams, Hash32, Height};
use ug_net::CommitData;

/// Maximum encoded block file size (one block plus framing slack).
pub const MAX_BLOCK_FILE_BYTES: u64 = 8 * 1024 * 1024;
/// Maximum quorum certificate file size.
pub const MAX_QC_FILE_BYTES: u64 = 256 * 1024;
/// Maximum state snapshot file size.
pub const MAX_STATE_SNAPSHOT_BYTES: u64 = 512 * 1024 * 1024;
/// Maximum UTXO entries accepted from a snapshot.
pub const MAX_STATE_UTXOS: usize = 16_000_000;
/// Maximum mempool snapshot file size.
pub const MAX_MEMPOOL_FILE_BYTES: u64 = 64 * 1024 * 1024;
/// Maximum equivocation evidence log size.
pub const MAX_EQUIVOCATION_LOG_BYTES: u64 = 16 * 1024 * 1024;

const STATE_MAGIC: [u8; 4] = *b"UGST";
const STATE_VERSION: u8 = 1;
const BLOCK_EXTENSION: &str = "blk";
const QC_EXTENSION: &str = "qc";

/// Canonical paths inside a node data directory.
#[derive(Clone, Debug)]
pub struct StorePaths {
    pub data_dir: PathBuf,
    pub genesis: PathBuf,
    pub node_config: PathBuf,
    pub blocks: PathBuf,
    pub state: PathBuf,
    pub mempool: PathBuf,
    pub peers: PathBuf,
    pub vote_journal: PathBuf,
    pub equivocations: PathBuf,
}

impl StorePaths {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        Self {
            genesis: data_dir.join("genesis.json"),
            node_config: data_dir.join("node.json"),
            blocks: data_dir.join("blocks"),
            state: data_dir.join("state.bin"),
            mempool: data_dir.join("mempool.bin"),
            peers: data_dir.join("peers.json"),
            vote_journal: data_dir.join("vote-journal.bin"),
            equivocations: data_dir.join("equivocations.jsonl"),
            data_dir,
        }
    }

    pub fn ensure(&self) -> Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.blocks)?;
        Ok(())
    }

    pub fn block_path(&self, height: Height) -> PathBuf {
        self.blocks.join(format!("{height:020}.{BLOCK_EXTENSION}"))
    }

    pub fn qc_path(&self, height: Height) -> PathBuf {
        self.blocks.join(format!("{height:020}.{QC_EXTENSION}"))
    }
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{nanos}-{counter}", std::process::id())
}

/// Write a file durably: create a new temporary file, fsync, rename, fsync the
/// directory (best effort on platforms without directory fsync).
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| Error::Store(format!("path {} has no parent", path.display())))?;
    std::fs::create_dir_all(parent)?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("tmp");
    let tmp = path.with_extension(format!("{extension}.tmp-{}", temp_suffix()));
    {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        file.write_all(data)?;
        file.sync_all()?;
    }
    if let Err(error) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(error.into());
    }
    sync_directory(parent);
    Ok(())
}

fn sync_directory(path: &Path) {
    #[cfg(unix)]
    {
        if let Ok(directory) = File::open(path) {
            let _ = directory.sync_all();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

fn read_bounded(path: &Path, max: u64) -> Result<Vec<u8>> {
    let meta = std::fs::metadata(path)?;
    if meta.len() > max {
        return Err(Error::Store(format!(
            "{} is {} bytes, above the {} byte limit",
            path.display(),
            meta.len(),
            max
        )));
    }
    let mut data = Vec::with_capacity(meta.len() as usize);
    let mut file = File::open(path)?;
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn read_block_file(path: &Path, chain: &ChainParams) -> Result<Block> {
    let data = read_bounded(path, MAX_BLOCK_FILE_BYTES)?;
    let block = Block::decode(&data, chain)?;
    block.validate_structure(chain)?;
    Ok(block)
}

fn read_qc_file(path: &Path) -> Result<QuorumCert> {
    let data = read_bounded(path, MAX_QC_FILE_BYTES)?;
    Ok(QuorumCert::decode(&data)?)
}

fn verify_qc_binding(
    chain: &ChainParams,
    validators: &ValidatorSet,
    block: &Block,
    certificate: &QuorumCert,
) -> Result<()> {
    certificate.verify(chain, validators)?;
    if certificate.height != block.header.height
        || certificate.round != block.header.round
        || certificate.block_hash != block.hash(chain)
    {
        return Err(Error::Store(
            "quorum certificate does not match its block".into(),
        ));
    }
    if certificate.height == 0 {
        return Err(Error::Store(
            "genesis cannot have a quorum certificate".into(),
        ));
    }
    Ok(())
}

/// Append-only-by-height block store with verified quorum certificates.
///
/// Blocks are immutable once written. There is no delete or rewrite API.
#[derive(Clone, Debug)]
pub struct BlockStore {
    paths: StorePaths,
    chain: ChainParams,
    headers: BTreeMap<Height, BlockHeader>,
    hashes: BTreeMap<Hash32, Height>,
    tip: Height,
}

impl BlockStore {
    /// Open the store, creating the genesis block file if the directory is
    /// new. Every existing block is decoded, structurally validated, linked to
    /// its parent, and paired with a verified quorum certificate.
    pub fn open(
        paths: &StorePaths,
        chain: ChainParams,
        genesis: &Block,
        validators: &ValidatorSet,
    ) -> Result<Self> {
        paths.ensure()?;
        if genesis.header.height != 0 {
            return Err(Error::Store("genesis block has non-zero height".into()));
        }
        genesis.validate_structure(&chain)?;

        let mut heights = Vec::new();
        for entry in std::fs::read_dir(&paths.blocks)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some(BLOCK_EXTENSION) {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| Error::Store("non-UTF-8 block filename".into()))?;
            let height: Height = stem
                .parse()
                .map_err(|_| Error::Store(format!("invalid block filename {}", path.display())))?;
            if format!("{height:020}") != stem {
                return Err(Error::Store(format!(
                    "block filename {} is not canonical",
                    path.display()
                )));
            }
            heights.push(height);
        }
        heights.sort_unstable();

        if heights.is_empty() {
            atomic_write(&paths.block_path(0), &genesis.encode())?;
            heights.push(0);
        }

        let mut headers: BTreeMap<Height, BlockHeader> = BTreeMap::new();
        let mut hashes = BTreeMap::new();
        let mut tip = 0u64;
        for (index, height) in heights.iter().copied().enumerate() {
            if height != index as Height {
                return Err(Error::Store(format!(
                    "block log is not contiguous: expected height {} but found {}",
                    index, height
                )));
            }
            let block = read_block_file(&paths.block_path(height), &chain)?;
            if block.header.height != height {
                return Err(Error::Store(format!(
                    "block file {height} contains height {}",
                    block.header.height
                )));
            }
            if height == 0 {
                if block != *genesis {
                    return Err(Error::Store(
                        "stored genesis block does not match genesis.json".into(),
                    ));
                }
            } else {
                let parent = headers
                    .get(&(height - 1))
                    .ok_or_else(|| Error::Store("missing parent header".into()))?;
                if block.header.previous_hash != parent.hash(&chain) {
                    return Err(Error::Store(format!(
                        "block {height} does not link to height {}",
                        height - 1
                    )));
                }
                let certificate = read_qc_file(&paths.qc_path(height))?;
                verify_qc_binding(&chain, validators, &block, &certificate)?;
            }
            let hash = block.hash(&chain);
            if hashes.insert(hash, height).is_some() {
                return Err(Error::Store("duplicate block hash in block log".into()));
            }
            headers.insert(height, block.header);
            tip = height;
        }

        let store = Self {
            paths: paths.clone(),
            chain,
            headers,
            hashes,
            tip,
        };
        Ok(store)
    }

    pub fn paths(&self) -> &StorePaths {
        &self.paths
    }

    pub fn height(&self) -> Height {
        self.tip
    }

    pub fn tip_header(&self) -> &BlockHeader {
        self.headers
            .get(&self.tip)
            .expect("tip header always exists")
    }

    pub fn header(&self, height: Height) -> Option<&BlockHeader> {
        self.headers.get(&height)
    }

    pub fn hash_at(&self, height: Height) -> Option<Hash32> {
        self.headers
            .get(&height)
            .map(|header| header.hash(&self.chain))
    }

    pub fn contains_hash(&self, hash: &Hash32) -> bool {
        self.hashes.contains_key(hash)
    }

    pub fn block(&self, height: Height) -> Result<Block> {
        if height > self.tip {
            return Err(Error::Store(format!(
                "block {height} is above tip {}",
                self.tip
            )));
        }
        let block = read_block_file(&self.paths.block_path(height), &self.chain)?;
        let expected = self
            .hash_at(height)
            .ok_or_else(|| Error::Store("missing header".into()))?;
        if block.hash(&self.chain) != expected {
            return Err(Error::Store(format!(
                "block file {height} was modified after startup"
            )));
        }
        Ok(block)
    }

    pub fn block_by_hash(&self, hash: &Hash32) -> Result<Option<Block>> {
        let Some(height) = self.hashes.get(hash) else {
            return Ok(None);
        };
        Ok(Some(self.block(*height)?))
    }

    /// Load a finalized block together with its verified certificate.
    pub fn commit(&self, height: Height) -> Result<CommitData> {
        if height == 0 || height > self.tip {
            return Err(Error::Store(format!(
                "commit {height} is outside 1..={}",
                self.tip
            )));
        }
        let block = self.block(height)?;
        let certificate = read_qc_file(&self.paths.qc_path(height))?;
        Ok(CommitData { block, certificate })
    }

    /// Persist a finalized block and its certificate. The caller must already
    /// have verified the certificate and executed the block.
    pub fn append(&mut self, block: &Block, certificate: &QuorumCert) -> Result<()> {
        let height = block.header.height;
        if height != self.tip.saturating_add(1) {
            return Err(Error::Store(format!(
                "cannot append height {height} above tip {}",
                self.tip
            )));
        }
        block.validate_structure(&self.chain)?;
        if block.header.previous_hash != self.tip_header().hash(&self.chain) {
            return Err(Error::Store("block does not extend the local tip".into()));
        }
        if self.hashes.contains_key(&block.hash(&self.chain)) {
            return Err(Error::Store("duplicate block hash".into()));
        }
        // Block first, certificate second, state snapshot later. A crash after
        // the block write leaves a block without a QC, which startup reports
        // as corruption instead of guessing.
        atomic_write(&self.paths.block_path(height), &block.encode())?;
        atomic_write(&self.paths.qc_path(height), &certificate.encode())?;
        self.headers.insert(height, block.header);
        self.hashes.insert(block.hash(&self.chain), height);
        self.tip = height;
        Ok(())
    }

    /// Blocks after the first locator hash that this node knows. Used to serve
    /// `GetBlocks`; the request is capped by the caller.
    pub fn blocks_after(
        &self,
        locator: &[Hash32],
        stop: &Hash32,
        limit: usize,
    ) -> Result<Vec<Block>> {
        let mut start = 0u64;
        for hash in locator {
            if let Some(height) = self.hashes.get(hash) {
                start = height.saturating_add(1);
                break;
            }
        }
        let mut blocks = Vec::new();
        for height in start..=self.tip {
            if blocks.len() >= limit {
                break;
            }
            let block = self.block(height)?;
            let hash = block.hash(&self.chain);
            blocks.push(block);
            if hash == *stop {
                break;
            }
        }
        Ok(blocks)
    }

    /// A bounded batch of finalized commits for catch-up.
    pub fn commits_from(&self, from: Height, limit: usize) -> Result<Vec<CommitData>> {
        let mut commits = Vec::new();
        let mut height = from.max(1);
        while height <= self.tip && commits.len() < limit {
            commits.push(self.commit(height)?);
            height += 1;
        }
        Ok(commits)
    }
}

/// Decoded state snapshot.
#[derive(Clone, Debug)]
pub struct StateSnapshot {
    pub height: Height,
    pub block_hash: Hash32,
    pub state: UtxoSet,
}

impl StateSnapshot {
    fn encode(&self, chain: &ChainParams, genesis_hash: &Hash32) -> Vec<u8> {
        let state = self.state.snapshot_encode();
        let mut e = Encoder::with_capacity(4 + 1 + 64 + 32 + 8 + 32 + 32 + 4 + state.len());
        e.fixed(&STATE_MAGIC);
        e.u8(STATE_VERSION);
        e.bytes(chain.chain_id.as_bytes());
        e.fixed(genesis_hash);
        e.u64(self.height);
        e.fixed(&self.block_hash);
        e.fixed(&self.state.state_root(chain));
        e.bytes(&state);
        e.finish()
    }

    /// Persist the state at `height`. The block must already be durable.
    pub fn save(
        paths: &StorePaths,
        chain: &ChainParams,
        genesis_hash: &Hash32,
        height: Height,
        block_hash: Hash32,
        state: &UtxoSet,
    ) -> Result<()> {
        if height == 0 {
            return Err(Error::Store(
                "refusing to save a genesis state snapshot".into(),
            ));
        }
        let snapshot = Self {
            height,
            block_hash,
            state: state.clone(),
        };
        atomic_write(&paths.state, &snapshot.encode(chain, genesis_hash))?;
        Ok(())
    }

    /// Load and cryptographically verify a snapshot. Returns `None` only when
    /// no snapshot file exists.
    pub fn load(
        paths: &StorePaths,
        chain: &ChainParams,
        genesis_hash: &Hash32,
        store: &BlockStore,
    ) -> Result<Option<Self>> {
        if !paths.state.exists() {
            return Ok(None);
        }
        let data = read_bounded(&paths.state, MAX_STATE_SNAPSHOT_BYTES)?;
        let mut d = Decoder::new(&data);
        let magic = d.fixed::<4>()?;
        if magic != STATE_MAGIC {
            return Err(Error::Store("bad state snapshot magic".into()));
        }
        let version = d.u8()?;
        if version != STATE_VERSION {
            return Err(Error::Store(format!(
                "unsupported state snapshot version {version}"
            )));
        }
        let chain_id = d.bytes(64)?;
        if chain_id != chain.chain_id.as_bytes() {
            return Err(Error::Store(
                "state snapshot belongs to a different chain".into(),
            ));
        }
        let snapshot_genesis = d.fixed::<32>()?;
        if snapshot_genesis != *genesis_hash {
            return Err(Error::Store(
                "state snapshot belongs to a different genesis".into(),
            ));
        }
        let height = d.u64()?;
        let block_hash = d.fixed::<32>()?;
        let expected_root = d.fixed::<32>()?;
        let state_bytes = d.bytes(MAX_STATE_SNAPSHOT_BYTES as usize)?;
        d.finish()?;

        if height == 0 {
            return Err(Error::Store("state snapshot has height zero".into()));
        }
        if height > store.height() {
            return Err(Error::Store(format!(
                "state snapshot height {height} is above block tip {}",
                store.height()
            )));
        }
        let header = store
            .header(height)
            .ok_or_else(|| Error::Store("state snapshot block is missing".into()))?;
        if header.hash(chain) != block_hash {
            return Err(Error::Store(
                "state snapshot block hash does not match the block log".into(),
            ));
        }
        let state = UtxoSet::snapshot_decode(chain, state_bytes, MAX_STATE_UTXOS)?;
        let root = state.state_root(chain);
        if root != expected_root || root != header.state_root {
            return Err(Error::Store("state snapshot root mismatch".into()));
        }
        state.audit_supply()?;
        Ok(Some(Self {
            height,
            block_hash,
            state,
        }))
    }
}

/// Load and validate the mempool snapshot. Invalid entries are dropped and
/// reported via the returned `dirty` flag so the caller can rewrite the file.
pub fn load_mempool(
    paths: &StorePaths,
    chain: &ChainParams,
    base: &UtxoSet,
    height: Height,
    now: u64,
    config: MempoolConfig,
) -> Result<(Mempool, bool)> {
    let max_entries = config.max_transactions;
    let mut mempool = Mempool::new(config);
    if !paths.mempool.exists() {
        return Ok((mempool, false));
    }
    let data = read_bounded(&paths.mempool, MAX_MEMPOOL_FILE_BYTES)?;
    let entries = Mempool::snapshot_decode(chain, &data, max_entries)?;
    let mut dirty = false;
    for (tx, received_at) in entries {
        if mempool
            .submit(chain, base, tx, height, received_at)
            .is_err()
        {
            dirty = true;
        }
    }
    if !mempool.expire(now).is_empty() {
        dirty = true;
    }
    Ok((mempool, dirty))
}

pub fn save_mempool(paths: &StorePaths, mempool: &Mempool) -> Result<()> {
    atomic_write(&paths.mempool, &mempool.snapshot_encode())?;
    Ok(())
}

/// Append one JSON line of equivocation evidence, bounded in size.
pub fn append_equivocation(paths: &StorePaths, line: &str) -> Result<()> {
    if let Ok(meta) = std::fs::metadata(&paths.equivocations) {
        if meta.len() > MAX_EQUIVOCATION_LOG_BYTES {
            return Err(Error::Store(
                "equivocation evidence log is full; operator review required".into(),
            ));
        }
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.equivocations)?;
    file.write_all(line.as_bytes())?;
    if !line.ends_with('\n') {
        file.write_all(b"\n")?;
    }
    file.sync_all()?;
    Ok(())
}

/// Read only the block tip without loading or executing state. Used by the
/// read-only `status` command.
pub fn read_tip(paths: &StorePaths, chain: &ChainParams) -> Result<Option<(Height, Hash32)>> {
    let mut best: Option<(Height, BlockHeader)> = None;
    if !paths.blocks.exists() {
        return Ok(None);
    }
    for entry in std::fs::read_dir(&paths.blocks)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some(BLOCK_EXTENSION) {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        let Ok(height) = stem.parse::<Height>() else {
            continue;
        };
        if best
            .as_ref()
            .map(|(best, _)| height > *best)
            .unwrap_or(true)
        {
            let block = read_block_file(&path, chain)?;
            best = Some((height, block.header));
        }
    }
    Ok(best.map(|(height, header)| (height, header.hash(chain))))
}

/// Decode a transaction from a hex string for tooling and tests.
pub fn transaction_from_hex(chain: &ChainParams, hex_text: &str) -> Result<Transaction> {
    let bytes = hex::decode(hex_text.trim())
        .map_err(|_| Error::Config("transaction is not hexadecimal".into()))?;
    Ok(Transaction::decode(&bytes, chain)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use ug_consensus::{Vote, VoteType};
    use ug_core::block::BLOCK_VERSION;
    use ug_core::crypto::{self, public_key};
    use ug_core::genesis::{create_config, Allocation, Genesis};
    use ug_core::tx::TxOut;

    fn keys() -> Vec<SigningKey> {
        (1..=4)
            .map(|seed| SigningKey::from_bytes(&[seed; 32]))
            .collect()
    }

    fn genesis() -> Genesis {
        let keys = keys();
        Genesis::build(create_config(
            1_700_000_000,
            vec![Allocation {
                recipient: public_key(&keys[0]),
                value: 1_000_000,
            }],
            keys.iter().map(public_key).collect(),
        ))
        .unwrap()
    }

    fn child(chain: &ChainParams, parent: &Block, timestamp: u64) -> Block {
        let header = BlockHeader {
            version: BLOCK_VERSION,
            height: 1,
            previous_hash: parent.hash(chain),
            timestamp,
            tx_root: crypto::merkle_root(&[]),
            state_root: parent.header.state_root,
            proposer: public_key(&keys()[1]),
            round: 0,
        };
        Block {
            header,
            transactions: vec![],
        }
    }

    fn qc(chain: &ChainParams, block: &Block) -> QuorumCert {
        let validators: Vec<[u8; 32]> = keys().iter().map(public_key).collect();
        let validator_set = ValidatorSet::from_slice(&validators).unwrap();
        let votes: Vec<Vote> = keys()
            .iter()
            .take(3)
            .map(|key| {
                Vote::new_signed(
                    chain,
                    VoteType::PreCommit,
                    block.header.height,
                    block.header.round,
                    block.hash(chain),
                    key,
                )
                .unwrap()
            })
            .collect();
        QuorumCert::from_votes(
            chain,
            &validator_set,
            block.header.height,
            block.header.round,
            block.hash(chain),
            votes,
        )
        .unwrap()
    }

    #[test]
    fn store_roundtrip_reopen_and_snapshot_verification() {
        let dir = tempfile::tempdir().unwrap();
        let paths = StorePaths::new(dir.path());
        let genesis = genesis();
        let chain = genesis.chain_params();
        let validators = ValidatorSet::from_slice(genesis.validators()).unwrap();
        let mut store =
            BlockStore::open(&paths, chain.clone(), genesis.block(), &validators).unwrap();
        assert_eq!(store.height(), 0);

        let block = child(&chain, genesis.block(), 1_700_000_001);
        let certificate = qc(&chain, &block);
        store.append(&block, &certificate).unwrap();
        assert_eq!(store.height(), 1);
        assert_eq!(store.block(1).unwrap(), block);
        assert_eq!(store.commit(1).unwrap().certificate, certificate);

        StateSnapshot::save(
            &paths,
            &chain,
            &genesis.hash(),
            1,
            block.hash(&chain),
            genesis.state(),
        )
        .unwrap();
        let loaded = StateSnapshot::load(&paths, &chain, &genesis.hash(), &store)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.height, 1);
        assert_eq!(loaded.state, *genesis.state());

        let reopened =
            BlockStore::open(&paths, chain.clone(), genesis.block(), &validators).unwrap();
        assert_eq!(reopened.height(), 1);
        assert_eq!(reopened.block(1).unwrap(), block);
    }

    #[test]
    fn modified_block_and_missing_qc_are_detected() {
        let dir = tempfile::tempdir().unwrap();
        let paths = StorePaths::new(dir.path());
        let genesis = genesis();
        let chain = genesis.chain_params();
        let validators = ValidatorSet::from_slice(genesis.validators()).unwrap();
        let mut store =
            BlockStore::open(&paths, chain.clone(), genesis.block(), &validators).unwrap();
        let block = child(&chain, genesis.block(), 1_700_000_001);
        let certificate = qc(&chain, &block);
        store.append(&block, &certificate).unwrap();

        std::fs::write(paths.qc_path(1), b"garbage").unwrap();
        assert!(BlockStore::open(&paths, chain.clone(), genesis.block(), &validators).is_err());

        std::fs::write(paths.qc_path(1), certificate.encode()).unwrap();
        std::fs::write(paths.block_path(1), b"garbage").unwrap();
        assert!(BlockStore::open(&paths, chain, genesis.block(), &validators).is_err());
    }

    #[test]
    fn state_snapshot_rejects_tampering() {
        let dir = tempfile::tempdir().unwrap();
        let paths = StorePaths::new(dir.path());
        let genesis = genesis();
        let chain = genesis.chain_params();
        let validators = ValidatorSet::from_slice(genesis.validators()).unwrap();
        let mut store =
            BlockStore::open(&paths, chain.clone(), genesis.block(), &validators).unwrap();
        let block = child(&chain, genesis.block(), 1_700_000_001);
        store.append(&block, &qc(&chain, &block)).unwrap();
        StateSnapshot::save(
            &paths,
            &chain,
            &genesis.hash(),
            1,
            block.hash(&chain),
            genesis.state(),
        )
        .unwrap();
        let mut data = std::fs::read(&paths.state).unwrap();
        let last = data.len() - 1;
        data[last] ^= 1;
        std::fs::write(&paths.state, data).unwrap();
        assert!(StateSnapshot::load(&paths, &chain, &genesis.hash(), &store).is_err());
    }

    #[test]
    fn genesis_allocation_is_not_accepted_as_a_transaction() {
        let genesis = genesis();
        let chain = genesis.chain_params();
        let tx = Transaction::new(vec![], vec![TxOut::new(1_000, public_key(&keys()[0]))], 0);
        let encoded = tx.encode();
        assert!(Transaction::decode(&encoded, &chain).is_err());
    }
}
