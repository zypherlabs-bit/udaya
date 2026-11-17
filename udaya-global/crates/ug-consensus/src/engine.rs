//! The U-ADC consensus state machine.
//!
//! The engine is transport-independent. It accepts verified proposals and
//! votes, updates its round state, and emits [`Action`] values for the node to
//! broadcast, persist or execute. It never touches the network, disk or clock
//! directly; timeouts are scheduled by the node and delivered back through
//! [`Consensus::on_timeout`].
//!
//! The implementation follows the Tendermint safety rules:
//!
//! 1. A validator pre-votes the locked block when it is locked, otherwise the
//!    proposal it accepted (or nil).
//! 2. A `2f+1` pre-vote quorum for a block lets a validator lock and pre-commit
//!    that block, even if it pre-voted something else in the same round.
//! 3. A `2f+1` pre-commit quorum produces a commit certificate.
//! 4. A higher-round `2f+1` pre-vote quorum supersedes an older lock.
//!
//! The node MUST pass a [`ProposalVerifier`] that fully executes the block
//! against the parent state before the engine votes. Consensus messages alone
//! are never sufficient to accept state.

use crate::guard::DoubleSignGuard;
use crate::validator::ValidatorSet;
use crate::vote::{Proposal, QuorumCert, Vote, VoteType, NIL_HASH};
use crate::{Error, Result, MAX_HEIGHT, MAX_ROUND};
use std::collections::{BTreeMap, VecDeque};
use ug_core::block::Block;
use ug_core::crypto::Hash32;
use ug_core::{ChainParams, Height, Round};

/// Maximum number of future-round votes retained.
pub const MAX_BUFFERED_VOTES: usize = 8_192;
/// Maximum number of rounds ahead for which votes are buffered.
pub const MAX_FUTURE_ROUNDS: Round = 1_024;

/// Consensus step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Step {
    Propose = 0,
    PreVote = 1,
    PreCommit = 2,
    Commit = 3,
}

impl Step {
    pub fn name(self) -> &'static str {
        match self {
            Self::Propose => "propose",
            Self::PreVote => "prevote",
            Self::PreCommit => "precommit",
            Self::Commit => "commit",
        }
    }
}

/// Timeout configuration. The node owns the clock.
#[derive(Clone, Debug)]
pub struct ConsensusConfig {
    pub propose_timeout_ms: u64,
    pub prevote_timeout_ms: u64,
    pub precommit_timeout_ms: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            propose_timeout_ms: 3_000,
            prevote_timeout_ms: 2_000,
            precommit_timeout_ms: 2_000,
        }
    }
}

impl ConsensusConfig {
    fn delay(&self, step: Step) -> u64 {
        match step {
            Step::Propose => self.propose_timeout_ms,
            Step::PreVote => self.prevote_timeout_ms,
            Step::PreCommit => self.precommit_timeout_ms,
            Step::Commit => 0,
        }
    }
}

/// Full block validation callback supplied by the node.
///
/// The verifier must check the parent hash, execute every transaction against
/// the parent state, and verify the resulting state root and supply. Returning
/// an error causes the engine to pre-vote nil and never pre-commit the block.
pub trait ProposalVerifier {
    fn verify(&self, chain: &ChainParams, block: &Block) -> std::result::Result<(), String>;
}

/// Evidence that a validator sent two different votes of the same type at the
/// same height and round. The node should log/persist this for operations.
#[derive(Clone, Debug)]
pub struct EquivocationEvidence {
    pub validator: Hash32,
    pub vote_type: VoteType,
    pub height: Height,
    pub round: Round,
    pub first: Vote,
    pub second: Vote,
}

/// Output of the engine.
#[derive(Clone, Debug)]
pub enum Action {
    /// The node should build and propose a block if it is the proposer.
    Propose {
        height: Height,
        round: Round,
    },
    /// The node should call `on_timeout` after the given delay.
    ScheduleTimeout {
        height: Height,
        round: Round,
        step: Step,
        delay_ms: u64,
    },
    BroadcastProposal(Proposal),
    BroadcastVote(Vote),
    /// A quorum has committed a block. `block` is present when the engine has
    /// the matching proposal; otherwise the node must fetch it by hash.
    Commit {
        height: Height,
        round: Round,
        block_hash: Hash32,
        block: Option<Block>,
        certificate: QuorumCert,
    },
    /// The node should fetch a block whose pre-vote quorum was observed.
    RequestBlock {
        height: Height,
        round: Round,
        block_hash: Hash32,
    },
    RejectProposal {
        height: Height,
        round: Round,
        reason: String,
    },
    Equivocation(EquivocationEvidence),
}

#[derive(Clone, Debug, Default)]
struct VoteSet {
    votes: BTreeMap<Hash32, Vote>,
}

// The equivocation variant intentionally carries two full votes; boxing would add
// an allocation on the common (non-equivocation) path and complicate matching.
#[allow(clippy::large_enum_variant)]
enum AddVote {
    Added,
    Duplicate,
    Equivocated { first: Vote, second: Vote },
}

impl VoteSet {
    fn clear(&mut self) {
        self.votes.clear();
    }

    fn add(&mut self, vote: Vote) -> AddVote {
        match self.votes.get(&vote.validator) {
            Some(first) if first.block_hash == vote.block_hash => AddVote::Duplicate,
            Some(first) => AddVote::Equivocated {
                first: first.clone(),
                second: vote,
            },
            None => {
                self.votes.insert(vote.validator, vote);
                AddVote::Added
            }
        }
    }

    fn hashes_with_quorum(&self, quorum: usize) -> Vec<Hash32> {
        let mut counts: BTreeMap<Hash32, usize> = BTreeMap::new();
        for vote in self.votes.values() {
            *counts.entry(vote.block_hash).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .filter(|(_, count)| *count >= quorum)
            .map(|(hash, _)| hash)
            .collect()
    }

    fn votes_for(&self, hash: &Hash32) -> Vec<Vote> {
        let mut votes: Vec<Vote> = self
            .votes
            .values()
            .filter(|vote| vote.block_hash == *hash)
            .cloned()
            .collect();
        votes.sort_by_key(|a| a.validator);
        votes
    }
}

/// A single-node view of the U-ADC protocol.
pub struct Consensus {
    chain: ChainParams,
    validators: ValidatorSet,
    config: ConsensusConfig,
    guard: DoubleSignGuard,
    height: Height,
    round: Round,
    step: Step,
    proposal: Option<Proposal>,
    locked: Option<(Round, Hash32)>,
    prevotes: VoteSet,
    precommits: VoteSet,
    future_votes: VecDeque<Vote>,
    actions: VecDeque<Action>,
    decided: bool,
    last_commit: Option<(Height, Hash32, QuorumCert)>,
}

impl Consensus {
    /// Create an engine for `start_height` (genesis is height 0 and is not
    /// produced by consensus, so the first call is normally `1`).
    pub fn new(
        chain: ChainParams,
        validators: ValidatorSet,
        config: ConsensusConfig,
        start_height: Height,
        guard: DoubleSignGuard,
    ) -> Result<Self> {
        Self::new_at_round(chain, validators, config, start_height, 0, guard)
    }

    /// Like [`Consensus::new`], but starts at `start_round`.
    ///
    /// A restarted validator uses this when its double-sign journal already
    /// contains a vote at `start_height`. It must not sign a conflicting vote
    /// at the same round, so it resumes at a strictly higher round.
    pub fn new_at_round(
        chain: ChainParams,
        validators: ValidatorSet,
        config: ConsensusConfig,
        start_height: Height,
        start_round: Round,
        guard: DoubleSignGuard,
    ) -> Result<Self> {
        chain.validate()?;
        if start_height == 0 || start_height > MAX_HEIGHT {
            return Err(Error::Engine(format!(
                "cannot start consensus at height {start_height}"
            )));
        }
        if start_round > MAX_ROUND {
            return Err(Error::Engine(format!(
                "cannot start consensus at round {start_round}"
            )));
        }
        if !validators.contains(&guard.validator()) {
            return Err(Error::Engine(
                "the local signing key is not in the validator set".into(),
            ));
        }
        guard.check_chain(&chain)?;
        let mut engine = Self {
            chain,
            validators,
            config,
            guard,
            height: start_height,
            round: start_round,
            step: Step::Propose,
            proposal: None,
            locked: None,
            prevotes: VoteSet::default(),
            precommits: VoteSet::default(),
            future_votes: VecDeque::new(),
            actions: VecDeque::new(),
            decided: false,
            last_commit: None,
        };
        engine.begin_round();
        Ok(engine)
    }

    pub fn chain(&self) -> &ChainParams {
        &self.chain
    }

    pub fn validators(&self) -> &ValidatorSet {
        &self.validators
    }

    pub fn height(&self) -> Height {
        self.height
    }

    pub fn round(&self) -> Round {
        self.round
    }

    pub fn step(&self) -> Step {
        self.step
    }

    pub fn validator(&self) -> Hash32 {
        self.guard.validator()
    }

    pub fn is_proposer(&self) -> bool {
        self.validators
            .is_proposer(self.height, self.round, &self.guard.validator())
    }

    pub fn locked(&self) -> Option<(Round, Hash32)> {
        self.locked
    }

    pub fn last_commit(&self) -> Option<(Height, Hash32, &QuorumCert)> {
        self.last_commit
            .as_ref()
            .map(|(height, hash, qc)| (*height, *hash, qc))
    }

    pub fn drain_actions(&mut self) -> Vec<Action> {
        self.actions.drain(..).collect()
    }

    fn begin_round(&mut self) {
        self.step = Step::Propose;
        self.proposal = None;
        self.prevotes.clear();
        self.precommits.clear();
        self.schedule(Step::Propose);
        self.actions.push_back(Action::Propose {
            height: self.height,
            round: self.round,
        });
    }

    fn schedule(&mut self, step: Step) {
        self.actions.push_back(Action::ScheduleTimeout {
            height: self.height,
            round: self.round,
            step,
            delay_ms: self.config.delay(step),
        });
    }

    /// Build (or relay) the proposal for the current round.
    ///
    /// The node supplies a fully executed block. The verifier is called again
    /// here as a defence in depth; a block that fails verification is never
    /// signed or broadcast.
    pub fn propose(&mut self, block: Block, verifier: &dyn ProposalVerifier) -> Result<()> {
        if self.decided || self.step != Step::Propose || self.proposal.is_some() {
            return Err(Error::Engine(
                "not in a state where a proposal can be made".into(),
            ));
        }
        if !self.is_proposer() {
            return Err(Error::Engine("not the proposer for this round".into()));
        }
        if block.header.height != self.height || block.header.round != self.round {
            return Err(Error::Engine(
                "block height/round does not match the current round".into(),
            ));
        }
        if block.header.proposer != self.guard.validator() {
            return Err(Error::Engine(
                "block proposer does not match the local validator".into(),
            ));
        }
        verifier
            .verify(&self.chain, &block)
            .map_err(Error::InvalidProposal)?;
        let proposal = self.guard.sign_proposal(&self.chain, block)?;
        proposal.verify(&self.chain, &self.validators)?;
        let proposal_hash = proposal.block_hash(&self.chain);

        // Persist the vote intent before exposing the proposal, so a crash
        // cannot produce a proposal the node refuses to vote for.
        let vote = self.guard.sign_vote(
            &self.chain,
            VoteType::PreVote,
            self.height,
            self.round,
            self.prevote_hash(proposal_hash),
        )?;
        self.proposal = Some(proposal.clone());
        self.actions.push_back(Action::BroadcastProposal(proposal));
        self.add_local_vote(vote);
        self.step = Step::PreVote;
        self.schedule(Step::PreVote);
        self.process_votes()
    }

    /// Accept a proposal from the network and perform the node's full
    /// verification before voting.
    pub fn on_proposal_verified(
        &mut self,
        proposal: Proposal,
        verifier: &dyn ProposalVerifier,
    ) -> Result<()> {
        proposal.verify(&self.chain, &self.validators)?;
        if proposal.height != self.height || proposal.round != self.round {
            // Future/stale proposals are dropped. The node will re-deliver the
            // current one after a round change.
            return Ok(());
        }
        if self.decided {
            return Ok(());
        }
        if let Some(existing) = &self.proposal {
            if existing.block_hash(&self.chain) == proposal.block_hash(&self.chain) {
                return Ok(());
            }
            return Err(Error::InvalidProposal(
                "the proposer sent two different blocks in one round".into(),
            ));
        }

        if let Err(reason) = verifier.verify(&self.chain, &proposal.block) {
            self.actions.push_back(Action::RejectProposal {
                height: self.height,
                round: self.round,
                reason: reason.clone(),
            });
            if self.step == Step::Propose {
                self.prevote(NIL_HASH)?;
            }
            return Ok(());
        }

        let proposal_hash = proposal.block_hash(&self.chain);
        self.proposal = Some(proposal);
        if self.step == Step::Propose {
            self.prevote(self.prevote_hash(proposal_hash))?;
        } else if self.step == Step::PreVote {
            // We may have pre-voted nil before seeing a valid proposal. The
            // engine never changes that vote; `process_votes` still uses the
            // proposal if a pre-vote quorum arrives.
            self.process_votes()?;
        }
        Ok(())
    }

    fn prevote_hash(&self, proposal_hash: Hash32) -> Hash32 {
        match self.locked {
            Some((_, locked_hash)) if locked_hash != proposal_hash => locked_hash,
            _ => proposal_hash,
        }
    }

    fn prevote(&mut self, block_hash: Hash32) -> Result<()> {
        if self.step != Step::Propose {
            return Ok(());
        }
        let vote = self.guard.sign_vote(
            &self.chain,
            VoteType::PreVote,
            self.height,
            self.round,
            block_hash,
        )?;
        self.add_local_vote(vote);
        self.step = Step::PreVote;
        self.schedule(Step::PreVote);
        self.process_votes()
    }

    fn precommit(&mut self, block_hash: Hash32) -> Result<()> {
        if self.step >= Step::PreCommit {
            return Ok(());
        }
        let vote = self.guard.sign_vote(
            &self.chain,
            VoteType::PreCommit,
            self.height,
            self.round,
            block_hash,
        )?;
        self.add_local_vote(vote);
        if block_hash != NIL_HASH {
            self.locked = Some((self.round, block_hash));
        }
        self.step = Step::PreCommit;
        self.schedule(Step::PreCommit);
        self.process_votes()
    }

    fn add_local_vote(&mut self, vote: Vote) {
        let set = match vote.vote_type {
            VoteType::PreVote => &mut self.prevotes,
            VoteType::PreCommit => &mut self.precommits,
        };
        match set.add(vote.clone()) {
            AddVote::Added => self.actions.push_back(Action::BroadcastVote(vote)),
            AddVote::Duplicate => {}
            AddVote::Equivocated { first, second } => {
                self.actions
                    .push_back(Action::Equivocation(EquivocationEvidence {
                        validator: second.validator,
                        vote_type: second.vote_type,
                        height: second.height,
                        round: second.round,
                        first,
                        second,
                    }));
            }
        }
    }

    /// Feed a vote received from the network.
    ///
    /// Invalid signatures and non-committee members are hard errors (the peer
    /// should be scored/dropped). Stale votes are ignored; near-future votes are
    /// buffered up to a strict bound.
    pub fn on_vote(&mut self, vote: Vote) -> Result<()> {
        vote.verify(&self.chain, &self.validators)?;
        if vote.height < self.height {
            return Ok(());
        }
        if vote.height == self.height {
            if vote.round < self.round {
                return Ok(());
            }
            if vote.round == self.round {
                if self.decided {
                    return Ok(());
                }
                return self.accept_current_vote(vote);
            }
            if vote.round.saturating_sub(self.round) <= MAX_FUTURE_ROUNDS {
                self.buffer_future(vote);
            }
            return Ok(());
        }
        if vote.height == self.height.saturating_add(1) && vote.round <= MAX_FUTURE_ROUNDS {
            self.buffer_future(vote);
        }
        Ok(())
    }

    fn accept_current_vote(&mut self, vote: Vote) -> Result<()> {
        let set = match vote.vote_type {
            VoteType::PreVote => &mut self.prevotes,
            VoteType::PreCommit => &mut self.precommits,
        };
        match set.add(vote) {
            AddVote::Added => self.process_votes(),
            AddVote::Duplicate => Ok(()),
            AddVote::Equivocated { first, second } => {
                self.actions
                    .push_back(Action::Equivocation(EquivocationEvidence {
                        validator: second.validator,
                        vote_type: second.vote_type,
                        height: second.height,
                        round: second.round,
                        first,
                        second,
                    }));
                Ok(())
            }
        }
    }

    fn buffer_future(&mut self, vote: Vote) {
        if self.future_votes.len() >= MAX_BUFFERED_VOTES {
            // Drop the oldest buffered vote, not the new one: the newest round
            // is the most likely to be relevant.
            self.future_votes.pop_front();
        }
        self.future_votes.push_back(vote);
    }

    fn replay_future(&mut self) -> Result<()> {
        let height = self.height;
        let round = self.round;
        let mut ready = Vec::new();
        let mut retained = VecDeque::new();
        while let Some(vote) = self.future_votes.pop_front() {
            if vote.height == height && vote.round == round {
                ready.push(vote);
            } else if vote.height < height || (vote.height == height && vote.round < round) {
                // stale; drop
            } else if vote.height == height && vote.round.saturating_sub(round) > MAX_FUTURE_ROUNDS
                || vote.height > height.saturating_add(1)
            {
                // too far ahead; drop
            } else {
                retained.push_back(vote);
            }
        }
        self.future_votes = retained;
        for vote in ready {
            if self.decided {
                break;
            }
            self.accept_current_vote(vote)?;
        }
        Ok(())
    }

    /// Deliver a local timeout. Stale timeouts are ignored.
    pub fn on_timeout(&mut self, height: Height, round: Round, step: Step) -> Result<()> {
        if height != self.height || round != self.round || step != self.step || self.decided {
            return Ok(());
        }
        match step {
            Step::Propose => self.advance_round(),
            Step::PreVote => {
                // A missing/insufficient pre-vote quorum means this round
                // cannot commit; pre-commit nil and move on.
                if self.step == Step::PreVote {
                    self.precommit(NIL_HASH)?;
                }
                self.advance_round()
            }
            Step::PreCommit => self.advance_round(),
            Step::Commit => Ok(()),
        }
    }

    fn advance_round(&mut self) -> Result<()> {
        if self.decided {
            return Ok(());
        }
        if self.round >= MAX_ROUND {
            return Err(Error::Engine("round limit reached".into()));
        }
        self.round += 1;
        self.begin_round();
        self.replay_future()
    }

    /// Force the engine to `height` after an externally verified commit.
    ///
    /// Commit catch-up may deliver a quorum certificate for the block at the
    /// engine's current height while the local engine has not decided it. The
    /// certificate proves finality, so the node persists the block and asks the
    /// engine to start the next height. The double-sign guard remains
    /// authoritative: it refuses any vote that would regress or equivocate.
    pub fn reset_height(&mut self, height: Height) -> Result<()> {
        if height != self.height.saturating_add(1) {
            return Err(Error::Engine(format!(
                "reset height must be {}, got {height}",
                self.height.saturating_add(1)
            )));
        }
        self.fast_forward(height)
    }

    /// Move directly to a higher height after externally verified commits.
    ///
    /// Used by commit catch-up when several heights arrive at once. The engine
    /// refuses to move backwards. If the double-sign journal already contains
    /// a vote at the target height, the engine resumes at the next round.
    pub fn fast_forward(&mut self, height: Height) -> Result<()> {
        if height <= self.height {
            return Err(Error::Engine(format!(
                "cannot fast-forward from height {} to {height}",
                self.height
            )));
        }
        if height > MAX_HEIGHT {
            return Err(Error::Engine("height limit reached".into()));
        }
        if self.guard.highest_height() > height {
            return Err(Error::Engine(format!(
                "vote journal already reaches height {}, above {height}",
                self.guard.highest_height()
            )));
        }
        let start_round = self
            .guard
            .highest_round_at(height)
            .map(|round| round.saturating_add(1))
            .unwrap_or(0);
        if start_round > MAX_ROUND {
            return Err(Error::Engine(
                "round limit reached during fast-forward".into(),
            ));
        }
        self.height = height;
        self.round = start_round;
        self.step = Step::Propose;
        self.proposal = None;
        self.locked = None;
        self.prevotes.clear();
        self.precommits.clear();
        self.decided = false;
        self.last_commit = None;
        self.begin_round();
        self.replay_future()
    }

    /// Advance to the next height after a commit.
    pub fn start_height(&mut self, height: Height) -> Result<()> {
        if !self.decided {
            return Err(Error::Engine(
                "cannot start a new height before committing the current one".into(),
            ));
        }
        if height != self.height.saturating_add(1) {
            return Err(Error::Engine(format!(
                "next height must be {}, got {height}",
                self.height.saturating_add(1)
            )));
        }
        if height > MAX_HEIGHT {
            return Err(Error::Engine("height limit reached".into()));
        }
        self.height = height;
        self.round = 0;
        self.step = Step::Propose;
        self.proposal = None;
        self.locked = None;
        self.prevotes.clear();
        self.precommits.clear();
        self.decided = false;
        self.begin_round();
        self.replay_future()
    }

    fn process_votes(&mut self) -> Result<()> {
        if self.decided {
            return Ok(());
        }
        let quorum = self.validators.quorum();

        for hash in self.prevotes.hashes_with_quorum(quorum) {
            if hash == NIL_HASH {
                if self.step < Step::PreCommit {
                    self.precommit(NIL_HASH)?;
                }
                continue;
            }
            let own_prevote = self
                .guard
                .vote_for(VoteType::PreVote, self.height, self.round)
                == Some(hash);
            let known = self
                .proposal
                .as_ref()
                .map(|proposal| proposal.block_hash(&self.chain) == hash)
                .unwrap_or(false);
            if known || own_prevote {
                // A quorum at this round supersedes any older lock.
                self.locked = Some((self.round, hash));
                if self.step < Step::PreCommit {
                    self.precommit(hash)?;
                }
            } else {
                self.actions.push_back(Action::RequestBlock {
                    height: self.height,
                    round: self.round,
                    block_hash: hash,
                });
            }
        }

        if self.decided {
            return Ok(());
        }
        for hash in self.precommits.hashes_with_quorum(quorum) {
            if hash == NIL_HASH {
                continue;
            }
            let votes = self.precommits.votes_for(&hash);
            let certificate = QuorumCert::from_votes(
                &self.chain,
                &self.validators,
                self.height,
                self.round,
                hash,
                votes,
            )?;
            let block = self
                .proposal
                .as_ref()
                .filter(|proposal| proposal.block_hash(&self.chain) == hash)
                .map(|proposal| proposal.block.clone());
            self.actions.push_back(Action::Commit {
                height: self.height,
                round: self.round,
                block_hash: hash,
                block,
                certificate: certificate.clone(),
            });
            self.last_commit = Some((self.height, hash, certificate));
            self.decided = true;
            self.step = Step::Commit;
            break;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guard::LocalSigner;
    use crate::vote::{Proposal, VoteType};
    use ed25519_dalek::SigningKey;
    use ug_core::block::{BlockHeader, BLOCK_VERSION};
    use ug_core::crypto::{self, public_key};

    struct Accept;
    impl ProposalVerifier for Accept {
        fn verify(&self, _chain: &ChainParams, _block: &Block) -> std::result::Result<(), String> {
            Ok(())
        }
    }

    struct Reject;
    impl ProposalVerifier for Reject {
        fn verify(&self, _chain: &ChainParams, _block: &Block) -> std::result::Result<(), String> {
            Err("test rejection".into())
        }
    }

    fn chain() -> ChainParams {
        ChainParams::devnet()
    }

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn keys(count: u8) -> Vec<SigningKey> {
        (1..=count).map(key).collect()
    }

    fn validator_set(keys: &[SigningKey]) -> ValidatorSet {
        ValidatorSet::new(keys.iter().map(public_key)).unwrap()
    }

    fn config() -> ConsensusConfig {
        ConsensusConfig {
            propose_timeout_ms: 10,
            prevote_timeout_ms: 10,
            precommit_timeout_ms: 10,
        }
    }

    fn block(_chain: &ChainParams, height: Height, round: Round, proposer: Hash32) -> Block {
        let header = BlockHeader {
            version: BLOCK_VERSION,
            height,
            previous_hash: [height as u8; 32],
            timestamp: 1_700_000_000 + height,
            tx_root: crypto::merkle_root(&[]),
            state_root: [0xCD; 32],
            proposer,
            round,
        };
        Block {
            header,
            transactions: vec![],
        }
    }

    fn proposal_for(
        chain: &ChainParams,
        set: &ValidatorSet,
        keys: &[SigningKey],
        height: Height,
        round: Round,
    ) -> Proposal {
        let proposer = set.proposer(height, round);
        let key = keys
            .iter()
            .find(|key| public_key(key) == proposer)
            .unwrap()
            .clone();
        Proposal::new_signed(chain, block(chain, height, round, proposer), &key).unwrap()
    }

    fn vote(
        chain: &ChainParams,
        vote_type: VoteType,
        height: Height,
        round: Round,
        hash: Hash32,
        key: &SigningKey,
    ) -> Vote {
        Vote::new_signed(chain, vote_type, height, round, hash, key).unwrap()
    }

    fn engine_for(
        chain: &ChainParams,
        set: &ValidatorSet,
        keys: &[SigningKey],
        index: usize,
        height: Height,
    ) -> Consensus {
        Consensus::new(
            chain.clone(),
            set.clone(),
            config(),
            height,
            DoubleSignGuard::in_memory(chain, LocalSigner::from_key(keys[index].clone())).unwrap(),
        )
        .unwrap()
    }

    fn commits(actions: &[Action]) -> Vec<(Hash32, QuorumCert)> {
        actions
            .iter()
            .filter_map(|action| match action {
                Action::Commit {
                    block_hash,
                    certificate,
                    ..
                } => Some((*block_hash, certificate.clone())),
                _ => None,
            })
            .collect()
    }

    fn broadcast_votes(actions: &[Action]) -> Vec<Vote> {
        actions
            .iter()
            .filter_map(|action| match action {
                Action::BroadcastVote(vote) => Some(vote.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn happy_path_commits_with_exact_quorum() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let proposer_key = keys
            .iter()
            .find(|key| public_key(key) == set.proposer(1, 0))
            .unwrap()
            .clone();
        let proposer_index = keys.iter().position(|key| *key == proposer_key).unwrap();
        let mut engine = engine_for(&chain, &set, &keys, proposer_index, 1);

        let proposal = proposal_for(&chain, &set, &keys, 1, 0);
        let hash = proposal.block_hash(&chain);
        engine.propose(proposal.block.clone(), &Accept).unwrap();

        let actions = engine.drain_actions();
        assert!(actions
            .iter()
            .any(|action| matches!(action, Action::BroadcastProposal(_))));
        let prevotes = broadcast_votes(&actions);
        assert_eq!(prevotes.len(), 1);
        assert_eq!(prevotes[0].vote_type, VoteType::PreVote);
        assert_eq!(prevotes[0].block_hash, hash);

        for key in &keys {
            if public_key(key) == set.proposer(1, 0) {
                continue;
            }
            engine
                .on_vote(vote(&chain, VoteType::PreVote, 1, 0, hash, key))
                .unwrap();
        }
        let actions = engine.drain_actions();
        let precommits: Vec<Vote> = broadcast_votes(&actions)
            .into_iter()
            .filter(|vote| vote.vote_type == VoteType::PreCommit)
            .collect();
        assert_eq!(precommits.len(), 1);
        assert_eq!(precommits[0].block_hash, hash);
        assert!(commits(&actions).is_empty());

        for key in &keys {
            if public_key(key) == set.proposer(1, 0) {
                continue;
            }
            engine
                .on_vote(vote(&chain, VoteType::PreCommit, 1, 0, hash, key))
                .unwrap();
        }
        let actions = engine.drain_actions();
        let commits = commits(&actions);
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].0, hash);
        commits[0].1.verify(&chain, &set).unwrap();
        assert_eq!(engine.step(), Step::Commit);
        assert!(engine.last_commit().is_some());
    }

    #[test]
    fn commit_needs_three_precommits_not_two() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let proposer = set.proposer(1, 0);
        let index = keys
            .iter()
            .position(|key| public_key(key) == proposer)
            .unwrap();
        let mut engine = engine_for(&chain, &set, &keys, index, 1);
        let proposal = proposal_for(&chain, &set, &keys, 1, 0);
        let hash = proposal.block_hash(&chain);
        engine.propose(proposal.block.clone(), &Accept).unwrap();
        let _ = engine.drain_actions();

        for key in keys
            .iter()
            .filter(|key| public_key(key) != proposer)
            .take(2)
        {
            engine
                .on_vote(vote(&chain, VoteType::PreCommit, 1, 0, hash, key))
                .unwrap();
        }
        // Two external precommits plus the local one is only three, which is
        // already quorum; use a fresh engine without a local vote instead.
        let mut observer_index = 0;
        while public_key(&keys[observer_index]) == proposer {
            observer_index += 1;
        }
        let mut observer = engine_for(&chain, &set, &keys, observer_index, 1);
        observer
            .on_proposal_verified(proposal.clone(), &Accept)
            .unwrap();
        let _ = observer.drain_actions();
        let others: Vec<&SigningKey> = keys
            .iter()
            .filter(|key| public_key(key) != public_key(&keys[observer_index]))
            .collect();
        observer
            .on_vote(vote(&chain, VoteType::PreCommit, 1, 0, hash, others[0]))
            .unwrap();
        observer
            .on_vote(vote(&chain, VoteType::PreCommit, 1, 0, hash, others[1]))
            .unwrap();
        assert!(commits(&observer.drain_actions()).is_empty());
        observer
            .on_vote(vote(&chain, VoteType::PreCommit, 1, 0, hash, others[2]))
            .unwrap();
        assert_eq!(commits(&observer.drain_actions()).len(), 1);
    }

    #[test]
    fn invalid_proposal_makes_the_engine_prevote_nil() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let engine_index = keys
            .iter()
            .position(|key| public_key(key) != set.proposer(1, 0))
            .unwrap();
        let mut engine = engine_for(&chain, &set, &keys, engine_index, 1);
        let proposal = proposal_for(&chain, &set, &keys, 1, 0);
        engine.on_proposal_verified(proposal, &Reject).unwrap();
        let actions = engine.drain_actions();
        assert!(actions
            .iter()
            .any(|action| matches!(action, Action::RejectProposal { .. })));
        let votes = broadcast_votes(&actions);
        assert_eq!(votes.len(), 1);
        assert_eq!(votes[0].vote_type, VoteType::PreVote);
        assert!(votes[0].is_nil());
    }

    #[test]
    fn wrong_proposer_proposal_is_rejected_before_voting() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let mut engine = engine_for(&chain, &set, &keys, 0, 1);
        let expected = set.proposer(1, 0);
        let wrong = keys.iter().find(|key| public_key(key) != expected).unwrap();
        let proposal =
            Proposal::new_signed(&chain, block(&chain, 1, 0, public_key(wrong)), wrong).unwrap();
        assert!(engine.on_proposal_verified(proposal, &Accept).is_err());
        assert!(broadcast_votes(&engine.drain_actions()).is_empty());
    }

    #[test]
    fn vote_for_another_chain_and_non_member_are_hard_errors() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let mut engine = engine_for(&chain, &set, &keys, 0, 1);
        let mut other = chain.clone();
        other.chain_id = "udaya-global-devnet-2".into();
        let bad = Vote::new_signed(&other, VoteType::PreVote, 1, 0, [1u8; 32], &keys[1]).unwrap();
        assert!(engine.on_vote(bad).is_err());

        let outsider =
            Vote::new_signed(&chain, VoteType::PreVote, 1, 0, [1u8; 32], &key(9)).unwrap();
        assert!(engine.on_vote(outsider).is_err());
    }

    #[test]
    fn equivocation_is_reported_and_not_double_counted() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let mut engine = engine_for(&chain, &set, &keys, 0, 1);
        let first = vote(&chain, VoteType::PreVote, 1, 0, [1u8; 32], &keys[1]);
        let second = vote(&chain, VoteType::PreVote, 1, 0, [2u8; 32], &keys[1]);
        engine.on_vote(first.clone()).unwrap();
        engine.on_vote(second).unwrap();
        let actions = engine.drain_actions();
        assert!(actions.iter().any(|action| matches!(
            action,
            Action::Equivocation(evidence)
                if evidence.first.block_hash == [1u8; 32]
                    && evidence.second.block_hash == [2u8; 32]
        )));
    }

    #[test]
    fn timeout_advances_round_and_schedules_new_work() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let mut engine = engine_for(&chain, &set, &keys, 0, 1);
        let _ = engine.drain_actions();
        engine.on_timeout(1, 0, Step::Propose).unwrap();
        assert_eq!(engine.round(), 1);
        assert_eq!(engine.step(), Step::Propose);
        let actions = engine.drain_actions();
        assert!(actions.iter().any(|action| matches!(
            action,
            Action::Propose {
                height: 1,
                round: 1
            }
        )));
        // A stale timeout does nothing.
        engine.on_timeout(1, 0, Step::Propose).unwrap();
        assert_eq!(engine.round(), 1);
    }

    #[test]
    fn prevote_timeout_precommits_nil_and_advances() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let mut engine = engine_for(&chain, &set, &keys, 0, 1);
        let proposal = proposal_for(&chain, &set, &keys, 1, 0);
        engine.on_proposal_verified(proposal, &Accept).unwrap();
        let _ = engine.drain_actions();
        engine.on_timeout(1, 0, Step::PreVote).unwrap();
        assert_eq!(engine.round(), 1);
        let actions = engine.drain_actions();
        let votes = broadcast_votes(&actions);
        let nil_precommits: Vec<&Vote> = votes
            .iter()
            .filter(|vote| vote.vote_type == VoteType::PreCommit && vote.is_nil())
            .collect();
        assert_eq!(nil_precommits.len(), 1);
    }

    #[test]
    fn future_round_votes_are_buffered_and_replayed() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let mut engine = engine_for(&chain, &set, &keys, 0, 1);
        let _ = engine.drain_actions();
        for key in keys
            .iter()
            .filter(|key| public_key(key) != public_key(&keys[0]))
            .take(3)
        {
            engine
                .on_vote(vote(&chain, VoteType::PreVote, 1, 1, NIL_HASH, key))
                .unwrap();
        }
        engine.on_timeout(1, 0, Step::Propose).unwrap();
        let actions = engine.drain_actions();
        let votes = broadcast_votes(&actions);
        assert!(votes
            .iter()
            .any(|vote| vote.vote_type == VoteType::PreCommit && vote.is_nil()));
    }

    #[test]
    fn lock_is_kept_across_rounds_and_released_by_higher_round_polka() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let proposer = set.proposer(1, 0);
        let index = keys
            .iter()
            .position(|key| public_key(key) == proposer)
            .unwrap();
        let mut engine = engine_for(&chain, &set, &keys, index, 1);
        let first = proposal_for(&chain, &set, &keys, 1, 0);
        let first_hash = first.block_hash(&chain);
        engine.propose(first.block.clone(), &Accept).unwrap();
        let _ = engine.drain_actions();
        let others: Vec<&SigningKey> = keys
            .iter()
            .filter(|key| public_key(key) != proposer)
            .collect();
        engine
            .on_vote(vote(&chain, VoteType::PreVote, 1, 0, first_hash, others[0]))
            .unwrap();
        engine
            .on_vote(vote(&chain, VoteType::PreVote, 1, 0, first_hash, others[1]))
            .unwrap();
        let _ = engine.drain_actions();
        assert_eq!(engine.locked(), Some((0, first_hash)));

        // Time out without committing; the lock must survive the round change.
        engine.on_timeout(1, 0, Step::PreCommit).unwrap();
        assert_eq!(engine.round(), 1);
        assert_eq!(engine.locked(), Some((0, first_hash)));

        let second = proposal_for(&chain, &set, &keys, 1, 1);
        let second_hash = second.block_hash(&chain);
        engine
            .on_proposal_verified(second.clone(), &Accept)
            .unwrap();
        let actions = engine.drain_actions();
        let votes = broadcast_votes(&actions);
        let prevote = votes
            .iter()
            .find(|vote| vote.vote_type == VoteType::PreVote)
            .unwrap();
        assert_eq!(
            prevote.block_hash, first_hash,
            "locked block must be prevoted"
        );

        // A 2f+1 prevote quorum for the new block at the higher round unlocks
        // and pre-commits it.
        for key in &others {
            engine
                .on_vote(vote(&chain, VoteType::PreVote, 1, 1, second_hash, key))
                .unwrap();
        }
        let actions = engine.drain_actions();
        let precommit = broadcast_votes(&actions)
            .into_iter()
            .find(|vote| vote.vote_type == VoteType::PreCommit)
            .unwrap();
        assert_eq!(precommit.block_hash, second_hash);
        assert_eq!(engine.locked(), Some((1, second_hash)));
    }

    #[test]
    fn start_height_resets_state_and_requires_a_commit() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let mut engine = engine_for(&chain, &set, &keys, 0, 1);
        assert!(engine.start_height(2).is_err());

        // Drive a commit quickly without proposal validation.
        let proposal = proposal_for(&chain, &set, &keys, 1, 0);
        let hash = proposal.block_hash(&chain);
        engine.on_proposal_verified(proposal, &Accept).unwrap();
        let _ = engine.drain_actions();
        for key in &keys {
            if public_key(key) == public_key(&keys[0]) {
                continue;
            }
            engine
                .on_vote(vote(&chain, VoteType::PreVote, 1, 0, hash, key))
                .unwrap();
        }
        let _ = engine.drain_actions();
        for key in &keys {
            if public_key(key) == public_key(&keys[0]) {
                continue;
            }
            engine
                .on_vote(vote(&chain, VoteType::PreCommit, 1, 0, hash, key))
                .unwrap();
        }
        let _ = engine.drain_actions();
        assert_eq!(engine.step(), Step::Commit);
        engine.start_height(2).unwrap();
        assert_eq!(engine.height(), 2);
        assert_eq!(engine.round(), 0);
        assert_eq!(engine.step(), Step::Propose);
        assert!(engine.locked().is_none());
    }

    #[test]
    fn commit_action_carries_the_block_when_known() {
        let chain = chain();
        let keys = keys(4);
        let set = validator_set(&keys);
        let mut engine = engine_for(&chain, &set, &keys, 0, 1);
        let proposal = proposal_for(&chain, &set, &keys, 1, 0);
        let block = proposal.block.clone();
        engine.on_proposal_verified(proposal, &Accept).unwrap();
        let _ = engine.drain_actions();
        let hash = block.hash(&chain);
        for key in &keys {
            if public_key(key) == public_key(&keys[0]) {
                continue;
            }
            engine
                .on_vote(vote(&chain, VoteType::PreVote, 1, 0, hash, key))
                .unwrap();
        }
        let _ = engine.drain_actions();
        for key in &keys {
            if public_key(key) == public_key(&keys[0]) {
                continue;
            }
            engine
                .on_vote(vote(&chain, VoteType::PreCommit, 1, 0, hash, key))
                .unwrap();
        }
        let actions = engine.drain_actions();
        let commit = actions
            .iter()
            .find_map(|action| match action {
                Action::Commit { block, .. } => Some(block.clone()),
                _ => None,
            })
            .unwrap();
        assert_eq!(commit, Some(block));
    }
}
