//! Canonical consensus messages: votes, proposals and quorum certificates.
//!
//! Every signature is domain-separated and bound to the Udaya Global chain ID.
//! Nil votes use an all-zero block hash; a nil pre-vote or pre-commit is part of
//! the protocol and never counts toward a commit certificate.

use crate::validator::ValidatorSet;
use crate::{Error, Result, MAX_HEIGHT, MAX_ROUND, MAX_VALIDATORS};
use ed25519_dalek::{Signer, SigningKey};
use std::collections::BTreeSet;
use ug_core::block::Block;
use ug_core::codec::{Decoder, Encoder};
use ug_core::crypto::{self, Hash32, SignatureBytes};
use ug_core::{ChainParams, Height, Round};

/// Version of the canonical vote encoding.
pub const VOTE_VERSION: u8 = 1;
/// Version of the canonical proposal encoding.
pub const PROPOSAL_VERSION: u8 = 1;
/// Version of the canonical quorum certificate encoding.
pub const QC_VERSION: u8 = 1;

/// Encoded size of a vote, including the version byte.
pub const VOTE_ENCODED_LEN: usize = 1 + 1 + 8 + 4 + 32 + 32 + 64;

/// The nil block hash used by pre-votes and pre-commits.
pub const NIL_HASH: Hash32 = [0u8; 32];

const VOTE_TAG: &str = "ug/consensus-vote";
const PROPOSAL_TAG: &str = "ug/consensus-proposal";

/// Vote step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VoteType {
    PreVote = 1,
    PreCommit = 2,
}

impl VoteType {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::PreVote),
            2 => Ok(Self::PreCommit),
            other => Err(Error::InvalidVote(format!("unknown vote type {other}"))),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::PreVote => "prevote",
            Self::PreCommit => "precommit",
        }
    }
}

/// A signed consensus vote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vote {
    pub version: u8,
    pub vote_type: VoteType,
    pub height: Height,
    pub round: Round,
    pub block_hash: Hash32,
    pub validator: Hash32,
    pub signature: SignatureBytes,
}

impl Vote {
    /// Construct and sign a vote.
    ///
    /// This low-level constructor performs **no** double-sign protection. The
    /// node must use [`crate::DoubleSignGuard::sign_vote`] for its own votes.
    /// It exists for tests and for verifying adversarial messages.
    pub fn new_signed(
        chain: &ChainParams,
        vote_type: VoteType,
        height: Height,
        round: Round,
        block_hash: Hash32,
        key: &SigningKey,
    ) -> Result<Self> {
        check_height_round(height, round)?;
        let validator = crypto::public_key(key);
        let mut vote = Self {
            version: VOTE_VERSION,
            vote_type,
            height,
            round,
            block_hash,
            validator,
            signature: [0u8; 64],
        };
        vote.signature = key.sign(&vote.digest(chain)).to_bytes();
        Ok(vote)
    }

    pub fn is_nil(&self) -> bool {
        self.block_hash == NIL_HASH
    }

    /// Canonical body that is signed (everything except the signature).
    pub fn signing_body(&self) -> Vec<u8> {
        let mut e = Encoder::with_capacity(VOTE_ENCODED_LEN - 64);
        e.u8(self.version);
        e.u8(self.vote_type.as_u8());
        e.u64(self.height);
        e.u32(self.round);
        e.fixed(&self.block_hash);
        e.fixed(&self.validator);
        e.finish()
    }

    pub fn digest(&self, chain: &ChainParams) -> Hash32 {
        crypto::tagged_hash(VOTE_TAG, &[chain.chain_id.as_bytes(), &self.signing_body()])
    }

    /// Verify encoding bounds, committee membership and the signature.
    pub fn verify(&self, chain: &ChainParams, validators: &ValidatorSet) -> Result<()> {
        if self.version != VOTE_VERSION {
            return Err(Error::InvalidVote(format!(
                "unsupported vote version {}",
                self.version
            )));
        }
        check_height_round(self.height, self.round)?;
        if !validators.contains(&self.validator) {
            return Err(Error::InvalidVote(
                "vote was not signed by a committee member".into(),
            ));
        }
        if !crypto::verify(
            &self.validator,
            &self.signature,
            VOTE_TAG,
            &[chain.chain_id.as_bytes(), &self.signing_body()],
        ) {
            return Err(Error::InvalidVote(
                "vote signature verification failed".into(),
            ));
        }
        Ok(())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut e = Encoder::with_capacity(VOTE_ENCODED_LEN);
        e.u8(self.version);
        e.u8(self.vote_type.as_u8());
        e.u64(self.height);
        e.u32(self.round);
        e.fixed(&self.block_hash);
        e.fixed(&self.validator);
        e.fixed(&self.signature);
        let bytes = e.finish();
        debug_assert_eq!(bytes.len(), VOTE_ENCODED_LEN);
        bytes
    }

    /// Strict fixed-size decode. Rejects truncation, trailing bytes, unknown
    /// versions and unknown vote types.
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() != VOTE_ENCODED_LEN {
            return Err(Error::InvalidVote(format!(
                "vote must be exactly {VOTE_ENCODED_LEN} bytes, got {}",
                data.len()
            )));
        }
        let mut d = Decoder::new(data);
        let version = d.u8()?;
        if version != VOTE_VERSION {
            return Err(Error::InvalidVote(format!(
                "unsupported vote version {version}"
            )));
        }
        let vote_type = VoteType::from_u8(d.u8()?)?;
        let height = d.u64()?;
        let round = d.u32()?;
        let block_hash = d.fixed::<32>()?;
        let validator = d.fixed::<32>()?;
        let signature = d.fixed::<64>()?;
        d.finish()?;
        check_height_round(height, round)?;
        Ok(Self {
            version,
            vote_type,
            height,
            round,
            block_hash,
            validator,
            signature,
        })
    }
}

/// A signed block proposal for one round.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    pub version: u8,
    pub height: Height,
    pub round: Round,
    pub block: Block,
    pub proposer: Hash32,
    pub signature: SignatureBytes,
}

impl Proposal {
    /// Build and sign a proposal. The block must already be structurally valid
    /// and must name `key` as proposer.
    pub fn new_signed(chain: &ChainParams, block: Block, key: &SigningKey) -> Result<Self> {
        let proposer = crypto::public_key(key);
        if block.header.height == 0 {
            return Err(Error::InvalidProposal(
                "genesis cannot be proposed by consensus".into(),
            ));
        }
        check_height_round(block.header.height, block.header.round)?;
        if block.header.proposer != proposer {
            return Err(Error::InvalidProposal(
                "block proposer field does not match signing key".into(),
            ));
        }
        block
            .validate_structure(chain)
            .map_err(|e| Error::InvalidProposal(e.to_string()))?;
        let mut proposal = Self {
            version: PROPOSAL_VERSION,
            height: block.header.height,
            round: block.header.round,
            block,
            proposer,
            signature: [0u8; 64],
        };
        proposal.signature = key.sign(&proposal.digest(chain)).to_bytes();
        Ok(proposal)
    }

    pub fn block_hash(&self, chain: &ChainParams) -> Hash32 {
        self.block.hash(chain)
    }

    fn signing_body_for(&self, chain: &ChainParams) -> Vec<u8> {
        let mut e = Encoder::with_capacity(1 + 8 + 4 + 32 + 32);
        e.u8(self.version);
        e.u64(self.height);
        e.u32(self.round);
        e.fixed(&self.block.hash(chain));
        e.fixed(&self.proposer);
        e.finish()
    }

    pub fn digest(&self, chain: &ChainParams) -> Hash32 {
        crypto::tagged_hash(
            PROPOSAL_TAG,
            &[chain.chain_id.as_bytes(), &self.signing_body_for(chain)],
        )
    }

    /// Verify proposer selection, block structure and signature.
    ///
    /// This does **not** execute the block or check the parent/state root. The
    /// node performs those checks through its [`crate::ProposalVerifier`]
    /// before voting.
    pub fn verify(&self, chain: &ChainParams, validators: &ValidatorSet) -> Result<()> {
        if self.version != PROPOSAL_VERSION {
            return Err(Error::InvalidProposal(format!(
                "unsupported proposal version {}",
                self.version
            )));
        }
        check_height_round(self.height, self.round)?;
        if self.height == 0 {
            return Err(Error::InvalidProposal(
                "genesis cannot be proposed by consensus".into(),
            ));
        }
        if self.block.header.height != self.height || self.block.header.round != self.round {
            return Err(Error::InvalidProposal(
                "proposal height/round does not match its block".into(),
            ));
        }
        if self.block.header.proposer != self.proposer {
            return Err(Error::InvalidProposal(
                "proposal proposer does not match block proposer".into(),
            ));
        }
        if validators.proposer(self.height, self.round) != self.proposer {
            return Err(Error::InvalidProposal(
                "proposal was signed by the wrong proposer for this round".into(),
            ));
        }
        if !validators.contains(&self.proposer) {
            return Err(Error::InvalidProposal(
                "proposer is not a committee member".into(),
            ));
        }
        self.block
            .validate_structure(chain)
            .map_err(|e| Error::InvalidProposal(e.to_string()))?;
        if !crypto::verify(
            &self.proposer,
            &self.signature,
            PROPOSAL_TAG,
            &[chain.chain_id.as_bytes(), &self.signing_body_for(chain)],
        ) {
            return Err(Error::InvalidProposal(
                "proposal signature verification failed".into(),
            ));
        }
        Ok(())
    }

    pub fn encode(&self) -> Vec<u8> {
        let block = self.block.encode();
        let mut e = Encoder::with_capacity(block.len() + 256);
        e.u8(self.version);
        e.u64(self.height);
        e.u32(self.round);
        e.fixed(&self.proposer);
        e.bytes(&block);
        e.fixed(&self.signature);
        e.finish()
    }

    pub fn decode(data: &[u8], chain: &ChainParams) -> Result<Self> {
        if data.len() > chain.max_block_bytes.saturating_add(512) {
            return Err(Error::InvalidProposal(
                "encoded proposal exceeds the maximum block size".into(),
            ));
        }
        let mut d = Decoder::new(data);
        let version = d.u8()?;
        if version != PROPOSAL_VERSION {
            return Err(Error::InvalidProposal(format!(
                "unsupported proposal version {version}"
            )));
        }
        let height = d.u64()?;
        let round = d.u32()?;
        let proposer = d.fixed::<32>()?;
        let block_bytes = d.bytes(chain.max_block_bytes)?;
        let signature = d.fixed::<64>()?;
        d.finish()?;
        let block =
            Block::decode(block_bytes, chain).map_err(|e| Error::InvalidProposal(e.to_string()))?;
        let proposal = Self {
            version,
            height,
            round,
            block,
            proposer,
            signature,
        };
        check_height_round(proposal.height, proposal.round)?;
        Ok(proposal)
    }
}

/// A quorum certificate over pre-commits for one block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuorumCert {
    pub version: u8,
    pub height: Height,
    pub round: Round,
    pub block_hash: Hash32,
    pub votes: Vec<Vote>,
}

impl QuorumCert {
    /// Build a certificate from pre-commits, then verify it before returning.
    /// Votes are sorted by validator key so the encoding is canonical.
    pub fn from_votes(
        chain: &ChainParams,
        validators: &ValidatorSet,
        height: Height,
        round: Round,
        block_hash: Hash32,
        mut votes: Vec<Vote>,
    ) -> Result<Self> {
        votes.sort_by_key(|a| a.validator);
        let certificate = Self {
            version: QC_VERSION,
            height,
            round,
            block_hash,
            votes,
        };
        certificate.verify(chain, validators)?;
        Ok(certificate)
    }

    /// Verify that the certificate contains at least `2f+1` distinct, valid
    /// pre-commits from the committee for the same height, round and non-nil
    /// block hash.
    pub fn verify(&self, chain: &ChainParams, validators: &ValidatorSet) -> Result<()> {
        if self.version != QC_VERSION {
            return Err(Error::InvalidCertificate(format!(
                "unsupported certificate version {}",
                self.version
            )));
        }
        check_height_round(self.height, self.round)?;
        if self.block_hash == NIL_HASH {
            return Err(Error::InvalidCertificate(
                "a nil block cannot have a commit certificate".into(),
            ));
        }
        if self.votes.len() > validators.len() {
            return Err(Error::InvalidCertificate(
                "certificate contains more votes than validators".into(),
            ));
        }
        if self.votes.len() < validators.quorum() {
            return Err(Error::InvalidCertificate(format!(
                "certificate has {} votes, quorum is {}",
                self.votes.len(),
                validators.quorum()
            )));
        }
        let mut seen = BTreeSet::new();
        for vote in &self.votes {
            if vote.vote_type != VoteType::PreCommit {
                return Err(Error::InvalidCertificate(
                    "certificate contains a non-precommit vote".into(),
                ));
            }
            if vote.height != self.height
                || vote.round != self.round
                || vote.block_hash != self.block_hash
            {
                return Err(Error::InvalidCertificate(
                    "certificate vote does not match certificate header".into(),
                ));
            }
            if !seen.insert(vote.validator) {
                return Err(Error::InvalidCertificate(
                    "certificate contains a duplicate validator".into(),
                ));
            }
            vote.verify(chain, validators)?;
        }
        Ok(())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut e = Encoder::with_capacity(64 + self.votes.len() * (VOTE_ENCODED_LEN + 4));
        e.u8(self.version);
        e.u64(self.height);
        e.u32(self.round);
        e.fixed(&self.block_hash);
        e.u32(self.votes.len() as u32);
        for vote in &self.votes {
            e.bytes(&vote.encode());
        }
        e.finish()
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut d = Decoder::new(data);
        let version = d.u8()?;
        if version != QC_VERSION {
            return Err(Error::InvalidCertificate(format!(
                "unsupported certificate version {version}"
            )));
        }
        let height = d.u64()?;
        let round = d.u32()?;
        let block_hash = d.fixed::<32>()?;
        let count = d.len(MAX_VALIDATORS)?;
        let mut votes = Vec::with_capacity(count);
        for _ in 0..count {
            let bytes = d.bytes(VOTE_ENCODED_LEN)?;
            votes.push(Vote::decode(bytes)?);
        }
        d.finish()?;
        check_height_round(height, round)?;
        Ok(Self {
            version,
            height,
            round,
            block_hash,
            votes,
        })
    }
}

fn check_height_round(height: Height, round: Round) -> Result<()> {
    if height == 0 || height > MAX_HEIGHT {
        return Err(Error::InvalidVote(format!("height {height} out of range")));
    }
    if round > MAX_ROUND {
        return Err(Error::InvalidVote(format!("round {round} out of range")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ug_core::block::{BlockHeader, BLOCK_VERSION};
    use ug_core::crypto::public_key;

    fn chain() -> ChainParams {
        ChainParams::devnet()
    }

    fn other_chain() -> ChainParams {
        let mut other = chain();
        other.chain_id = "udaya-global-devnet-2".into();
        other
    }

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn set() -> ValidatorSet {
        ValidatorSet::new((1..=4u8).map(|seed| public_key(&key(seed)))).unwrap()
    }

    fn block(_chain: &ChainParams, height: Height, round: Round, proposer: Hash32) -> Block {
        let header = BlockHeader {
            version: BLOCK_VERSION,
            height,
            previous_hash: [height as u8; 32],
            timestamp: 1_700_000_000 + height,
            tx_root: crypto::merkle_root(&[]),
            state_root: [0xAB; 32],
            proposer,
            round,
        };
        Block {
            header,
            transactions: vec![],
        }
    }

    fn proposal(
        chain: &ChainParams,
        set: &ValidatorSet,
        height: Height,
        round: Round,
    ) -> (Proposal, SigningKey) {
        let proposer = set.proposer(height, round);
        let signing_key = (1..=4u8)
            .map(key)
            .find(|k| public_key(k) == proposer)
            .unwrap();
        let block = block(chain, height, round, proposer);
        (
            Proposal::new_signed(chain, block, &signing_key).unwrap(),
            signing_key,
        )
    }

    fn vote(
        chain: &ChainParams,
        vote_type: VoteType,
        height: Height,
        round: Round,
        hash: Hash32,
        seed: u8,
    ) -> Vote {
        Vote::new_signed(chain, vote_type, height, round, hash, &key(seed)).unwrap()
    }

    #[test]
    fn vote_roundtrip_verification_and_chain_binding() {
        let chain = chain();
        let set = set();
        let vote = vote(&chain, VoteType::PreVote, 7, 3, [9u8; 32], 1);
        assert_eq!(vote.encode().len(), VOTE_ENCODED_LEN);
        let decoded = Vote::decode(&vote.encode()).unwrap();
        assert_eq!(vote, decoded);
        decoded.verify(&chain, &set).unwrap();
        assert!(decoded.verify(&other_chain(), &set).is_err());

        let mut trailing = vote.encode();
        trailing.push(0);
        assert!(Vote::decode(&trailing).is_err());
        assert!(Vote::decode(&vote.encode()[..VOTE_ENCODED_LEN - 1]).is_err());
    }

    #[test]
    fn vote_rejects_tampering_unknown_members_and_bad_types() {
        let chain = chain();
        let set = set();
        let mut vote = vote(&chain, VoteType::PreVote, 7, 3, [9u8; 32], 1);
        vote.block_hash[0] ^= 1;
        assert!(vote.verify(&chain, &set).is_err());

        let outsider =
            Vote::new_signed(&chain, VoteType::PreVote, 7, 3, [9u8; 32], &key(9)).unwrap();
        assert!(outsider.verify(&chain, &set).is_err());

        let mut bytes = vote.encode();
        bytes[1] = 9;
        assert!(Vote::decode(&bytes).is_err());
    }

    #[test]
    fn proposal_roundtrip_and_signature_binding() {
        let chain = chain();
        let set = set();
        let (proposal, _) = proposal(&chain, &set, 5, 2);
        let bytes = proposal.encode();
        let decoded = Proposal::decode(&bytes, &chain).unwrap();
        assert_eq!(proposal, decoded);
        decoded.verify(&chain, &set).unwrap();
        assert!(decoded.verify(&other_chain(), &set).is_err());

        let mut tampered = decoded;
        tampered.block.header.timestamp += 1;
        assert!(tampered.verify(&chain, &set).is_err());
    }

    #[test]
    fn proposal_rejects_wrong_proposer() {
        let chain = chain();
        let set = set();
        let height = 5;
        let round = 2;
        let expected = set.proposer(height, round);
        let wrong = (1..=4u8)
            .map(key)
            .find(|k| public_key(k) != expected)
            .unwrap();
        let block = block(&chain, height, round, public_key(&wrong));
        let proposal = Proposal::new_signed(&chain, block, &wrong).unwrap();
        assert!(proposal.verify(&chain, &set).is_err());
    }

    #[test]
    fn qc_requires_exact_quorum_and_unique_signers() {
        let chain = chain();
        let set = set();
        let hash = [3u8; 32];
        let votes = (1..=3u8)
            .map(|seed| vote(&chain, VoteType::PreCommit, 4, 1, hash, seed))
            .collect::<Vec<_>>();
        QuorumCert::from_votes(&chain, &set, 4, 1, hash, votes).unwrap();

        let two = (1..=2u8)
            .map(|seed| vote(&chain, VoteType::PreCommit, 4, 1, hash, seed))
            .collect::<Vec<_>>();
        assert!(QuorumCert::from_votes(&chain, &set, 4, 1, hash, two).is_err());

        let mut duplicated = (1..=3u8)
            .map(|seed| vote(&chain, VoteType::PreCommit, 4, 1, hash, seed))
            .collect::<Vec<_>>();
        duplicated[2] = duplicated[0].clone();
        assert!(QuorumCert::from_votes(&chain, &set, 4, 1, hash, duplicated).is_err());
    }

    #[test]
    fn qc_rejects_wrong_type_height_or_nil_block() {
        let chain = chain();
        let set = set();
        let hash = [4u8; 32];
        let prevotes = (1..=3u8)
            .map(|seed| vote(&chain, VoteType::PreVote, 4, 1, hash, seed))
            .collect::<Vec<_>>();
        assert!(QuorumCert::from_votes(&chain, &set, 4, 1, hash, prevotes).is_err());

        let mixed_height = vec![
            vote(&chain, VoteType::PreCommit, 4, 1, hash, 1),
            vote(&chain, VoteType::PreCommit, 5, 1, hash, 2),
            vote(&chain, VoteType::PreCommit, 4, 1, hash, 3),
        ];
        assert!(QuorumCert::from_votes(&chain, &set, 4, 1, hash, mixed_height).is_err());

        let votes = (1..=3u8)
            .map(|seed| vote(&chain, VoteType::PreCommit, 4, 1, hash, seed))
            .collect::<Vec<_>>();
        assert!(QuorumCert::from_votes(&chain, &set, 4, 1, NIL_HASH, votes).is_err());
    }

    #[test]
    fn qc_encoding_is_canonical_and_strict() {
        let chain = chain();
        let set = set();
        let hash = [7u8; 32];
        let votes = (1..=3u8)
            .map(|seed| vote(&chain, VoteType::PreCommit, 4, 1, hash, seed))
            .collect::<Vec<_>>();
        let qc = QuorumCert::from_votes(&chain, &set, 4, 1, hash, votes).unwrap();
        let bytes = qc.encode();
        let decoded = QuorumCert::decode(&bytes).unwrap();
        assert_eq!(qc, decoded);
        decoded.verify(&chain, &set).unwrap();

        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(QuorumCert::decode(&trailing).is_err());
        assert!(QuorumCert::decode(&bytes[..bytes.len() - 1]).is_err());
    }

    #[test]
    fn qc_is_chain_bound() {
        let chain = chain();
        let set = set();
        let hash = [7u8; 32];
        let votes = (1..=3u8)
            .map(|seed| vote(&chain, VoteType::PreCommit, 4, 1, hash, seed))
            .collect::<Vec<_>>();
        let qc = QuorumCert::from_votes(&chain, &set, 4, 1, hash, votes).unwrap();
        assert!(qc.verify(&other_chain(), &set).is_err());
    }

    #[test]
    fn height_and_round_bounds_are_enforced() {
        assert!(Vote::new_signed(&chain(), VoteType::PreVote, 0, 0, [1u8; 32], &key(1)).is_err());
        assert!(Vote::new_signed(
            &chain(),
            VoteType::PreVote,
            1,
            MAX_ROUND + 1,
            [1u8; 32],
            &key(1)
        )
        .is_err());
    }
}
