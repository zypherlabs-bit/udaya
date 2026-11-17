//! U-ADC consensus: a Tendermint-style Byzantine fault tolerant engine for
//! Udaya Global.
//!
//! Safety properties implemented by this crate:
//!
//! * A validator signs at most one pre-vote and one pre-commit for a given
//!   `(height, round)`. The rule is enforced in memory and persisted to an
//!   append-only, fsynced journal *before* a signature is produced. Restarting
//!   a node never permits a conflicting vote.
//! * Votes and proposals are bound to the chain ID and to the validator key.
//! * A commit requires a quorum certificate containing at least `2f+1`
//!   distinct pre-commits from the `3f+1` validator set.
//! * A node only pre-commits a block after it has seen a `2f+1` pre-vote
//!   quorum and has itself accepted the proposal (including full block
//!   execution when wired to the node's verifier).
//! * Locks are released only by a higher-round quorum, as in Tendermint.
//!
//! The engine is deterministic and transport-independent. It produces
//! [`Action`] values; the node is responsible for networking, persistence of
//! blocks, and executing transactions before calling
//! [`Consensus::on_proposal_verified`].
//!
//! Committee rotation, dynamic validator joining/leaving, VRF proposer
//! selection, and state sync are deliberately **not** implemented in this
//! version. The validator set is fixed at genesis; see `docs/releases`.

pub mod engine;
pub mod guard;
pub mod validator;
pub mod vote;

pub use engine::{
    Action, Consensus, ConsensusConfig, EquivocationEvidence, ProposalVerifier, Step,
};
pub use guard::{DoubleSignGuard, LocalSigner, VoteJournal, VoteRecord};
pub use validator::{ValidatorSet, MAX_VALIDATORS, MIN_VALIDATORS};
pub use vote::{
    Proposal, QuorumCert, Vote, VoteType, NIL_HASH, PROPOSAL_VERSION, QC_VERSION, VOTE_VERSION,
};

use ug_core::Height;
use ug_core::Round;

/// Consensus-layer errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid validator set: {0}")]
    ValidatorSet(String),
    #[error("invalid vote: {0}")]
    InvalidVote(String),
    #[error("invalid proposal: {0}")]
    InvalidProposal(String),
    #[error("invalid quorum certificate: {0}")]
    InvalidCertificate(String),
    #[error("double-sign protection refused: {0}")]
    DoubleSign(String),
    #[error("vote journal error: {0}")]
    Journal(String),
    #[error("consensus codec error: {0}")]
    Codec(String),
    #[error("consensus engine error: {0}")]
    Engine(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<ug_core::Error> for Error {
    fn from(value: ug_core::Error) -> Self {
        Error::Codec(value.to_string())
    }
}

/// Maximum consensus height. Kept below `i64::MAX` so heights remain
/// representable in persisted formats used by downstream tooling.
pub const MAX_HEIGHT: Height = i64::MAX as Height;

/// Maximum consensus round at one height.
pub const MAX_ROUND: Round = 10_000_000;

/// Consensus step within a round.
pub use engine::Step as RoundStep;
