//! Persisted double-sign protection.
//!
//! A validator must never sign two different votes of the same type at the same
//! height and round. This module records the intent to vote in an append-only
//! journal, fsyncs it, and only then produces the Ed25519 signature. If the
//! process crashes after the journal write but before broadcasting, the vote is
//! still treated as cast on restart.
//!
//! The journal is deliberately fail-closed: a truncated, corrupt or
//! wrong-validator journal is rejected rather than silently repaired. There is
//! no rotation or pruning yet; see `docs/releases/RELEASE_READINESS.md`.

use crate::vote::{Proposal, Vote, VoteType, NIL_HASH};
use crate::{Error, Result, MAX_HEIGHT, MAX_ROUND};
use ed25519_dalek::SigningKey;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use ug_core::codec::{Decoder, Encoder};
use ug_core::crypto::{self, Hash32};
use ug_core::{ChainParams, Height, Round};

const JOURNAL_MAGIC: [u8; 4] = *b"UGVJ";
const JOURNAL_VERSION: u8 = 1;
const RECORD_VERSION: u8 = 1;
const RECORD_PAYLOAD_LEN: usize = 1 + 1 + 8 + 4 + 32;
const RECORD_CHECKSUM_LEN: usize = 8;
const MAX_CHAIN_ID_BYTES: usize = 64;
const MAX_JOURNAL_BYTES: u64 = 128 * 1024 * 1024;
const MAX_JOURNAL_RECORDS: usize = 2_000_000;

/// A local Ed25519 signing key. The key is never printed by `Debug`.
#[derive(Clone)]
pub struct LocalSigner {
    key: SigningKey,
}

impl LocalSigner {
    pub fn from_key(key: SigningKey) -> Self {
        Self { key }
    }

    pub fn from_seed(seed: &[u8; 32]) -> Self {
        Self {
            key: SigningKey::from_bytes(seed),
        }
    }

    pub fn generate() -> Self {
        Self {
            key: crypto::generate_key(),
        }
    }

    pub fn public_key(&self) -> Hash32 {
        crypto::public_key(&self.key)
    }
}

impl fmt::Debug for LocalSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LocalSigner(<redacted>)")
    }
}

/// One persisted vote intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoteRecord {
    pub vote_type: VoteType,
    pub height: Height,
    pub round: Round,
    pub block_hash: Hash32,
}

impl VoteRecord {
    fn encode_payload(&self) -> Vec<u8> {
        let mut e = Encoder::with_capacity(RECORD_PAYLOAD_LEN);
        e.u8(RECORD_VERSION);
        e.u8(self.vote_type.as_u8());
        e.u64(self.height);
        e.u32(self.round);
        e.fixed(&self.block_hash);
        let out = e.finish();
        debug_assert_eq!(out.len(), RECORD_PAYLOAD_LEN);
        out
    }

    fn decode_payload(data: &[u8]) -> Result<Self> {
        if data.len() != RECORD_PAYLOAD_LEN {
            return Err(Error::Journal(format!(
                "vote record must be {RECORD_PAYLOAD_LEN} bytes, got {}",
                data.len()
            )));
        }
        let mut d = Decoder::new(data);
        let version = d.u8()?;
        if version != RECORD_VERSION {
            return Err(Error::Journal(format!(
                "unsupported vote record version {version}"
            )));
        }
        let vote_type = VoteType::from_u8(d.u8()?)?;
        let height = d.u64()?;
        let round = d.u32()?;
        let block_hash = d.fixed::<32>()?;
        d.finish()?;
        if height == 0 || height > MAX_HEIGHT {
            return Err(Error::Journal(format!(
                "vote record height {height} out of range"
            )));
        }
        if round > MAX_ROUND {
            return Err(Error::Journal(format!(
                "vote record round {round} out of range"
            )));
        }
        Ok(Self {
            vote_type,
            height,
            round,
            block_hash,
        })
    }
}

/// Append-only, fsynced vote journal.
pub struct VoteJournal {
    file: File,
    path: PathBuf,
}

impl VoteJournal {
    /// Open or create a journal bound to `chain_id` and `validator`.
    ///
    /// Returns the journal and all records that were already persisted.
    pub fn open(
        path: &Path,
        chain_id: &str,
        validator: &Hash32,
    ) -> Result<(Self, Vec<VoteRecord>)> {
        if chain_id.is_empty() || chain_id.len() > MAX_CHAIN_ID_BYTES {
            return Err(Error::Journal("chain id is too long".into()));
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let exists = match std::fs::metadata(path) {
            Ok(meta) => {
                if meta.len() > MAX_JOURNAL_BYTES {
                    return Err(Error::Journal(format!(
                        "journal is {} bytes, above the {} byte limit",
                        meta.len(),
                        MAX_JOURNAL_BYTES
                    )));
                }
                true
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        };

        let mut records = Vec::new();
        let empty = exists && std::fs::metadata(path)?.len() == 0;
        if !exists || empty {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?;
            file.write_all(&encode_header(chain_id, validator)?)?;
            file.sync_all()?;
        } else {
            let mut file = File::open(path)?;
            let mut data = Vec::with_capacity(std::fs::metadata(path)?.len() as usize);
            file.read_to_end(&mut data)?;
            records = parse_journal(&data, chain_id, validator)?;
        }

        let file = OpenOptions::new().append(true).open(path)?;
        Ok((
            Self {
                file,
                path: path.to_path_buf(),
            },
            records,
        ))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn append(&mut self, record: &VoteRecord) -> Result<()> {
        let payload = record.encode_payload();
        let mut e = Encoder::with_capacity(4 + RECORD_PAYLOAD_LEN + RECORD_CHECKSUM_LEN);
        e.bytes(&payload);
        e.fixed(&record_checksum(&payload));
        self.file.write_all(&e.finish())?;
        // The durability guarantee is the point of this journal.
        self.file.sync_all()?;
        Ok(())
    }
}

fn encode_header(chain_id: &str, validator: &Hash32) -> Result<Vec<u8>> {
    let mut e = Encoder::with_capacity(4 + 1 + 4 + chain_id.len() + 32);
    e.fixed(&JOURNAL_MAGIC);
    e.u8(JOURNAL_VERSION);
    e.bytes(chain_id.as_bytes());
    e.fixed(validator);
    Ok(e.finish())
}

fn parse_journal(data: &[u8], chain_id: &str, validator: &Hash32) -> Result<Vec<VoteRecord>> {
    let mut d = Decoder::new(data);
    let magic = d.fixed::<4>()?;
    if magic != JOURNAL_MAGIC {
        return Err(Error::Journal("bad journal magic".into()));
    }
    let version = d.u8()?;
    if version != JOURNAL_VERSION {
        return Err(Error::Journal(format!(
            "unsupported journal version {version}"
        )));
    }
    let persisted_chain = d.bytes(MAX_CHAIN_ID_BYTES)?;
    if persisted_chain != chain_id.as_bytes() {
        return Err(Error::Journal(
            "journal belongs to a different chain id".into(),
        ));
    }
    let persisted_validator = d.fixed::<32>()?;
    if persisted_validator != *validator {
        return Err(Error::Journal(
            "journal belongs to a different validator key".into(),
        ));
    }

    let mut records = Vec::new();
    while d.remaining() > 0 {
        if records.len() >= MAX_JOURNAL_RECORDS {
            return Err(Error::Journal("journal record limit reached".into()));
        }
        if d.remaining() < 4 + RECORD_PAYLOAD_LEN + RECORD_CHECKSUM_LEN {
            return Err(Error::Journal(
                "journal ends with a truncated record".into(),
            ));
        }
        let payload = d.bytes(RECORD_PAYLOAD_LEN)?;
        let checksum = d.fixed::<RECORD_CHECKSUM_LEN>()?;
        if checksum != record_checksum(payload) {
            return Err(Error::Journal("journal record checksum mismatch".into()));
        }
        records.push(VoteRecord::decode_payload(payload)?);
    }
    d.finish()?;
    Ok(records)
}

fn record_checksum(payload: &[u8]) -> [u8; RECORD_CHECKSUM_LEN] {
    let digest = crypto::tagged_hash("ug/vote-journal-record", &[payload]);
    let mut out = [0u8; RECORD_CHECKSUM_LEN];
    out.copy_from_slice(&digest[..RECORD_CHECKSUM_LEN]);
    out
}

/// Double-sign guard around a single validator key.
pub struct DoubleSignGuard {
    chain_id: String,
    validator: Hash32,
    signer: LocalSigner,
    journal: Option<VoteJournal>,
    records: BTreeMap<(Height, Round, VoteType), Hash32>,
    last: BTreeMap<VoteType, (Height, Round)>,
    highest_height: Height,
}

impl DoubleSignGuard {
    /// Open a guard backed by an append-only journal.
    ///
    /// Refuses journals bound to another chain, another validator, or with any
    /// corrupt/truncated record. There is no "repair" path: a damaged journal
    /// must be investigated by an operator.
    pub fn open(chain: &ChainParams, signer: LocalSigner, path: &Path) -> Result<Self> {
        chain.validate()?;
        let validator = signer.public_key();
        let (journal, records) = VoteJournal::open(path, &chain.chain_id, &validator)?;
        let mut guard = Self::empty(chain, signer, Some(journal));
        for record in records {
            guard.remember(record)?;
        }
        Ok(guard)
    }

    /// In-memory guard for tests and ephemeral networks. It enforces every
    /// in-process rule but does not survive a restart.
    pub fn in_memory(chain: &ChainParams, signer: LocalSigner) -> Result<Self> {
        chain.validate()?;
        Ok(Self::empty(chain, signer, None))
    }

    fn empty(chain: &ChainParams, signer: LocalSigner, journal: Option<VoteJournal>) -> Self {
        Self {
            chain_id: chain.chain_id.clone(),
            validator: signer.public_key(),
            signer,
            journal,
            records: BTreeMap::new(),
            last: BTreeMap::new(),
            highest_height: 0,
        }
    }

    pub fn validator(&self) -> Hash32 {
        self.validator
    }

    /// Highest round for which this validator has signed any vote at `height`.
    ///
    /// A restarted node uses this to resume at a strictly higher round, which
    /// avoids equivocating with a vote that was journaled immediately before a
    /// crash.
    pub fn highest_round_at(&self, height: Height) -> Option<Round> {
        self.records
            .keys()
            .filter(|(recorded_height, _, _)| *recorded_height == height)
            .map(|(_, round, _)| *round)
            .max()
    }

    /// Highest height for which any vote has been journaled.
    pub fn highest_height(&self) -> Height {
        self.highest_height
    }

    pub fn chain_id(&self) -> &str {
        &self.chain_id
    }

    pub fn check_chain(&self, chain: &ChainParams) -> Result<()> {
        if chain.chain_id != self.chain_id {
            return Err(Error::DoubleSign(format!(
                "guard is bound to chain {}, not {}",
                self.chain_id, chain.chain_id
            )));
        }
        Ok(())
    }

    fn remember(&mut self, record: VoteRecord) -> Result<()> {
        let key = (record.height, record.round, record.vote_type);
        if let Some(existing) = self.records.get(&key) {
            if *existing != record.block_hash {
                return Err(Error::DoubleSign(format!(
                    "journal contains conflicting {} votes at height {} round {}",
                    record.vote_type.name(),
                    record.height,
                    record.round
                )));
            }
            return Ok(());
        }
        self.records.insert(key, record.block_hash);
        self.highest_height = self.highest_height.max(record.height);
        let entry = self
            .last
            .entry(record.vote_type)
            .or_insert((record.height, record.round));
        if (record.height, record.round) > *entry {
            *entry = (record.height, record.round);
        }
        Ok(())
    }

    /// Durably record the intent, then sign. A conflicting vote is never
    /// signed, even after a crash and restart.
    pub fn sign_vote(
        &mut self,
        chain: &ChainParams,
        vote_type: VoteType,
        height: Height,
        round: Round,
        block_hash: Hash32,
    ) -> Result<Vote> {
        self.check_chain(chain)?;
        if height == 0 || height > MAX_HEIGHT {
            return Err(Error::DoubleSign(format!("height {height} out of range")));
        }
        if round > MAX_ROUND {
            return Err(Error::DoubleSign(format!("round {round} out of range")));
        }
        if height < self.highest_height {
            return Err(Error::DoubleSign(format!(
                "refusing to vote at height {height}, already voted at height {}",
                self.highest_height
            )));
        }
        if let Some((last_height, last_round)) = self.last.get(&vote_type) {
            if (height, round) < (*last_height, *last_round) {
                return Err(Error::DoubleSign(format!(
                    "refusing to regress {} vote from round {} to round {} at height {}",
                    vote_type.name(),
                    last_round,
                    round,
                    height
                )));
            }
        }

        let key = (height, round, vote_type);
        if let Some(existing) = self.records.get(&key) {
            if *existing == block_hash {
                // Idempotent re-signing is safe: Ed25519 is deterministic and
                // the vote is already durable.
                return Vote::new_signed(
                    chain,
                    vote_type,
                    height,
                    round,
                    block_hash,
                    &self.signer.key,
                );
            }
            return Err(Error::DoubleSign(format!(
                "refusing to sign a second {} vote at height {height} round {round}",
                vote_type.name()
            )));
        }

        let record = VoteRecord {
            vote_type,
            height,
            round,
            block_hash,
        };
        if let Some(journal) = self.journal.as_mut() {
            journal.append(&record)?;
        }
        let vote = Vote::new_signed(
            chain,
            vote_type,
            height,
            round,
            block_hash,
            &self.signer.key,
        )?;
        self.remember(record)?;
        Ok(vote)
    }

    /// Sign a proposal. Proposal equivocation is not a slashing condition, but
    /// the engine still produces at most one proposal per round.
    pub fn sign_proposal(
        &self,
        chain: &ChainParams,
        block: ug_core::block::Block,
    ) -> Result<Proposal> {
        self.check_chain(chain)?;
        Proposal::new_signed(chain, block, &self.signer.key)
    }

    /// True when this validator has already signed a vote of `vote_type` at
    /// `(height, round)`.
    pub fn has_voted(&self, vote_type: VoteType, height: Height, round: Round) -> bool {
        self.records.contains_key(&(height, round, vote_type))
    }

    /// The block hash this validator voted for, if any.
    pub fn vote_for(&self, vote_type: VoteType, height: Height, round: Round) -> Option<Hash32> {
        self.records.get(&(height, round, vote_type)).copied()
    }

    /// Nil-hash convenience used by tests and tooling.
    pub fn nil_hash() -> Hash32 {
        NIL_HASH
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ug_core::crypto::public_key;

    fn chain() -> ChainParams {
        ChainParams::devnet()
    }

    fn signer(seed: u8) -> LocalSigner {
        LocalSigner::from_seed(&[seed; 32])
    }

    #[test]
    fn guard_is_idempotent_and_rejects_conflict() {
        let chain = chain();
        let mut guard = DoubleSignGuard::in_memory(&chain, signer(1)).unwrap();
        let first = guard
            .sign_vote(&chain, VoteType::PreVote, 1, 0, [1u8; 32])
            .unwrap();
        let again = guard
            .sign_vote(&chain, VoteType::PreVote, 1, 0, [1u8; 32])
            .unwrap();
        assert_eq!(first, again);
        assert!(guard
            .sign_vote(&chain, VoteType::PreVote, 1, 0, [2u8; 32])
            .is_err());
        assert!(guard.has_voted(VoteType::PreVote, 1, 0));
        assert_eq!(guard.vote_for(VoteType::PreVote, 1, 0), Some([1u8; 32]));
    }

    #[test]
    fn guard_rejects_height_and_round_regression() {
        let chain = chain();
        let mut guard = DoubleSignGuard::in_memory(&chain, signer(1)).unwrap();
        guard
            .sign_vote(&chain, VoteType::PreVote, 5, 3, [1u8; 32])
            .unwrap();
        assert!(guard
            .sign_vote(&chain, VoteType::PreVote, 5, 2, [2u8; 32])
            .is_err());
        assert!(guard
            .sign_vote(&chain, VoteType::PreVote, 4, 9, [2u8; 32])
            .is_err());
        // A different vote type may advance independently.
        assert!(guard
            .sign_vote(&chain, VoteType::PreCommit, 5, 3, [1u8; 32])
            .is_ok());
    }

    #[test]
    fn guard_is_chain_bound() {
        let chain = chain();
        let mut guard = DoubleSignGuard::in_memory(&chain, signer(1)).unwrap();
        let mut other = chain.clone();
        other.chain_id = "udaya-global-devnet-2".into();
        assert!(guard
            .sign_vote(&other, VoteType::PreVote, 1, 0, [1u8; 32])
            .is_err());
        assert!(!guard.has_voted(VoteType::PreVote, 1, 0));
    }

    #[test]
    fn persisted_guard_survives_restart_and_refuses_equivocation() {
        let chain = chain();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("votes.journal");
        {
            let mut guard = DoubleSignGuard::open(&chain, signer(1), &path).unwrap();
            guard
                .sign_vote(&chain, VoteType::PreVote, 3, 2, [0xA1; 32])
                .unwrap();
        }
        {
            let mut guard = DoubleSignGuard::open(&chain, signer(1), &path).unwrap();
            assert!(guard
                .sign_vote(&chain, VoteType::PreVote, 3, 2, [0xA2; 32])
                .is_err());
            // Idempotent re-signing after restart is allowed.
            assert!(guard
                .sign_vote(&chain, VoteType::PreVote, 3, 2, [0xA1; 32])
                .is_ok());
            guard
                .sign_vote(&chain, VoteType::PreCommit, 3, 2, [0xA1; 32])
                .unwrap();
        }
        {
            let guard = DoubleSignGuard::open(&chain, signer(1), &path).unwrap();
            assert_eq!(guard.vote_for(VoteType::PreVote, 3, 2), Some([0xA1; 32]));
            assert_eq!(guard.vote_for(VoteType::PreCommit, 3, 2), Some([0xA1; 32]));
        }
    }

    #[test]
    fn journal_is_bound_to_validator_and_chain() {
        let chain = chain();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("votes.journal");
        {
            let mut guard = DoubleSignGuard::open(&chain, signer(1), &path).unwrap();
            guard
                .sign_vote(&chain, VoteType::PreVote, 1, 0, [1u8; 32])
                .unwrap();
        }
        assert!(DoubleSignGuard::open(&chain, signer(2), &path).is_err());
        let mut other = chain.clone();
        other.chain_id = "udaya-global-devnet-2".into();
        assert!(DoubleSignGuard::open(&other, signer(1), &path).is_err());
    }

    #[test]
    fn corrupted_and_truncated_journals_fail_closed() {
        let chain = chain();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("votes.journal");
        {
            let mut guard = DoubleSignGuard::open(&chain, signer(1), &path).unwrap();
            guard
                .sign_vote(&chain, VoteType::PreVote, 1, 0, [1u8; 32])
                .unwrap();
        }
        let original = std::fs::read(&path).unwrap();

        let mut truncated = original.clone();
        truncated.pop();
        std::fs::write(&path, &truncated).unwrap();
        assert!(DoubleSignGuard::open(&chain, signer(1), &path).is_err());

        let mut corrupted = original.clone();
        let last = corrupted.len() - 10;
        corrupted[last] ^= 1;
        std::fs::write(&path, &corrupted).unwrap();
        assert!(DoubleSignGuard::open(&chain, signer(1), &path).is_err());

        let mut bad_header = original;
        bad_header[0] ^= 1;
        std::fs::write(&path, &bad_header).unwrap();
        assert!(DoubleSignGuard::open(&chain, signer(1), &path).is_err());
    }

    #[test]
    fn journal_rejects_manual_conflicting_records() {
        let chain = chain();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("votes.journal");
        {
            let mut guard = DoubleSignGuard::open(&chain, signer(1), &path).unwrap();
            guard
                .sign_vote(&chain, VoteType::PreVote, 1, 0, [1u8; 32])
                .unwrap();
        }
        let validator = public_key(&SigningKey::from_bytes(&[1u8; 32]));
        let mut file = OpenOptions::new().append(true).open(&path).unwrap();
        let record = VoteRecord {
            vote_type: VoteType::PreVote,
            height: 1,
            round: 0,
            block_hash: [2u8; 32],
        };
        let payload = record.encode_payload();
        let mut e = Encoder::new();
        e.bytes(&payload);
        e.fixed(&record_checksum(&payload));
        file.write_all(&e.finish()).unwrap();
        file.sync_all().unwrap();
        let _ = validator;
        assert!(DoubleSignGuard::open(&chain, signer(1), &path).is_err());
    }

    #[test]
    fn debug_does_not_leak_the_key() {
        let text = format!("{:?}", signer(7));
        assert!(text.contains("redacted"));
        assert!(!text.contains("7, 7"));
    }
}
