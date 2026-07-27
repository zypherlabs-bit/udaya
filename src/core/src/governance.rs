use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Udaya Governance Framework
/// On-chain governance for protocol upgrades, treasury management, and community voting

/// Proposal status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Rejected,
    Executed,
    Expired,
}

/// Proposal type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    ProtocolUpgrade {
        new_version: String,
        changes: Vec<String>,
        activation_height: u64,
    },
    TreasurySpending {
        recipient: String,
        amount: u64,
        purpose: String,
    },
    ParameterChange {
        parameter: String,
        old_value: String,
        new_value: String,
    },
    CommunityFund {
        project_name: String,
        description: String,
        requested_amount: u64,
    },
    General {
        title: String,
        description: String,
    },
}

/// A governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub proposal_type: ProposalType,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub votes_for: u64,
    pub votes_against: u64,
    pub quorum_threshold: u64,
    pub approval_threshold: f64,
    pub executed_at: Option<u64>,
    pub transaction_hash: Option<String>,
}

/// A vote on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: String,
    pub voter: String,
    pub vote_for: bool,
    pub voting_power: u64,
    pub timestamp: u64,
    pub signature: Option<Vec<u8>>,
}

/// Governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub min_proposal_fee: u64,
    pub voting_period_blocks: u64,
    pub quorum_percentage: f64,
    pub approval_percentage: f64,
    pub min_voting_power: u64,
    pub max_active_proposals: u32,
    pub treasury_address: String,
    pub community_fund_percent: f64,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            min_proposal_fee: 100_000_000, // 1 UDYA
            voting_period_blocks: 20_160,  // ~14 days
            quorum_percentage: 10.0,       // 10% of staked supply
            approval_percentage: 60.0,     // 60% approval required
            min_voting_power: 10_000_000,  // 0.1 UDYA min to vote
            max_active_proposals: 10,
            treasury_address: "UDYA_TREASURY_RESERVED".to_string(),
            community_fund_percent: 10.0,  // 10% of block rewards
        }
    }
}

/// Governance engine
pub struct GovernanceEngine {
    config: GovernanceConfig,
    proposals: Arc<RwLock<HashMap<String, Proposal>>>,
    votes: Arc<RwLock<HashMap<String, Vec<Vote>>>>,
    voter_power: Arc<RwLock<HashMap<String, u64>>>,
}

impl GovernanceEngine {
    pub fn new(config: GovernanceConfig) -> Self {
        Self {
            config,
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            voter_power: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new proposal
    pub fn create_proposal(
        &self,
        proposal_type: ProposalType,
        proposer: &str,
        title: &str,
        description: &str,
        start_height: u64,
    ) -> anyhow::Result<String> {
        let active_count = self.get_active_proposals().len() as u32;
        if active_count >= self.config.max_active_proposals {
            anyhow::bail!("Maximum active proposals reached: {}", self.config.max_active_proposals);
        }

        let id = uuid::Uuid::new_v4().to_string();
        let proposal = Proposal {
            id: id.clone(),
            proposal_type,
            proposer: proposer.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: ProposalStatus::Active,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            start_height,
            end_height: start_height + self.config.voting_period_blocks,
            votes_for: 0,
            votes_against: 0,
            quorum_threshold: 0,
            approval_threshold: self.config.approval_percentage,
            executed_at: None,
            transaction_hash: None,
        };

        let mut proposals = self.proposals.write();
        proposals.insert(id.clone(), proposal);
        log::info!("New governance proposal created: {} - {}", id, title);
        
        Ok(id)
    }

    /// Cast a vote on a proposal
    pub fn cast_vote(
        &self,
        proposal_id: &str,
        voter: &str,
        vote_for: bool,
        voting_power: u64,
    ) -> anyhow::Result<()> {
        let mut proposals = self.proposals.write();
        let proposal = proposals.get_mut(proposal_id)
            .ok_or_else(|| anyhow::anyhow!("Proposal not found: {}", proposal_id))?;

        if proposal.status != ProposalStatus::Active {
            anyhow::bail!("Proposal is not active");
        }

        if voting_power < self.config.min_voting_power {
            anyhow::bail!("Insufficient voting power");
        }

        // Record vote
        let vote = Vote {
            proposal_id: proposal_id.to_string(),
            voter: voter.to_string(),
            vote_for,
            voting_power,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: None,
        };

        let mut votes = self.votes.write();
        votes.entry(proposal_id.to_string())
            .or_insert_with(Vec::new)
            .push(vote);

        // Update counts
        if vote_for {
            proposal.votes_for += voting_power;
        } else {
            proposal.votes_against += voting_power;
        }

        // Update voter power tracking
        let mut powers = self.voter_power.write();
        *powers.entry(voter.to_string()).or_insert(0) += voting_power;

        log::info!("Vote cast on {} by {}: {}", proposal_id, voter, if vote_for { "FOR" } else { "AGAINST" });
        Ok(())
    }

    /// Finalize a proposal (called at end_height)
    pub fn finalize_proposal(&self, proposal_id: &str) -> anyhow::Result<ProposalStatus> {
        let mut proposals = self.proposals.write();
        let proposal = proposals.get_mut(proposal_id)
            .ok_or_else(|| anyhow::anyhow!("Proposal not found: {}", proposal_id))?;

        if proposal.status != ProposalStatus::Active {
            anyhow::bail!("Proposal is not in active state");
        }

        // Calculate quorum
        let total_votes = proposal.votes_for + proposal.votes_against;
        let total_supply = 21_000_000 * 100_000_000u64; // Max supply in satoshis
        let quorum_met = (total_votes as f64 / total_supply as f64 * 100.0) >= self.config.quorum_percentage;

        if !quorum_met {
            proposal.status = ProposalStatus::Expired;
            log::info!("Proposal {} expired - quorum not met", proposal_id);
            return Ok(ProposalStatus::Expired);
        }

        // Calculate approval
        let total_votes_f = total_votes as f64;
        let approval_percent = if total_votes_f > 0.0 {
            (proposal.votes_for as f64 / total_votes_f) * 100.0
        } else {
            0.0
        };

        if approval_percent >= self.config.approval_percentage {
            proposal.status = ProposalStatus::Passed;
            proposal.executed_at = Some(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs());
            log::info!("Proposal {} PASSED with {:.2}% approval", proposal_id, approval_percent);
            Ok(ProposalStatus::Passed)
        } else {
            proposal.status = ProposalStatus::Rejected;
            log::info!("Proposal {} REJECTED with {:.2}% approval", proposal_id, approval_percent);
            Ok(ProposalStatus::Rejected)
        }
    }

    /// Get all proposals
    pub fn get_proposals(&self) -> Vec<Proposal> {
        self.proposals.read().values().cloned().collect()
    }

    /// Get active proposals
    pub fn get_active_proposals(&self) -> Vec<Proposal> {
        self.proposals.read().values()
            .filter(|p| p.status == ProposalStatus::Active)
            .cloned()
            .collect()
    }

    /// Get votes for a proposal
    pub fn get_votes(&self, proposal_id: &str) -> Vec<Vote> {
        self.votes.read()
            .get(proposal_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get governance analytics
    pub fn get_analytics(&self) -> GovernanceAnalytics {
        let proposals = self.proposals.read();
        let votes = self.votes.read();
        let voter_power = self.voter_power.read();

        let total_proposals = proposals.len() as u64;
        let active_proposals = proposals.values().filter(|p| p.status == ProposalStatus::Active).count() as u64;
        let passed_proposals = proposals.values().filter(|p| p.status == ProposalStatus::Passed).count() as u64;
        
        let total_votes: u64 = votes.values().map(|v| v.len() as u64).sum();
        let unique_voters = voter_power.len() as u64;
        
        GovernanceAnalytics {
            total_proposals,
            active_proposals,
            passed_proposals,
            total_votes,
            unique_voters,
            participation_rate: 0.0, // Calculated externally
        }
    }
}

/// Governance analytics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceAnalytics {
    pub total_proposals: u64,
    pub active_proposals: u64,
    pub passed_proposals: u64,
    pub total_votes: u64,
    pub unique_voters: u64,
    pub participation_rate: f64,
}

/// Treasury management
pub struct Treasury {
    balance: Arc<RwLock<u64>>,
    transactions: Arc<RwLock<Vec<TreasuryTx>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryTx {
    pub txid: String,
    pub amount: u64,
    pub recipient: String,
    pub purpose: String,
    pub timestamp: u64,
    pub approved_by: String,
}

impl Treasury {
    pub fn new(initial_balance: u64) -> Self {
        Self {
            balance: Arc::new(RwLock::new(initial_balance)),
            transactions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_balance(&self) -> u64 {
        *self.balance.read()
    }

    pub fn spend(&self, amount: u64, recipient: &str, purpose: &str, approver: &str) -> anyhow::Result<()> {
        let mut balance = self.balance.write();
        if *balance < amount {
            anyhow::bail!("Insufficient treasury balance");
        }
        *balance -= amount;
        
        let mut txs = self.transactions.write();
        txs.push(TreasuryTx {
            txid: uuid::Uuid::new_v4().to_string(),
            amount,
            recipient: recipient.to_string(),
            purpose: purpose.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            approved_by: approver.to_string(),
        });
        
        log::info!("Treasury spend: {} -> {} for {}", amount, recipient, purpose);
        Ok(())
    }

    pub fn get_history(&self) -> Vec<TreasuryTx> {
        self.transactions.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proposal() {
        let config = GovernanceConfig::default();
        let engine = GovernanceEngine::new(config);
        
        let id = engine.create_proposal(
            ProposalType::General {
                title: "Test Proposal".to_string(),
                description: "Testing governance".to_string(),
            },
            "proposer1",
            "Test Proposal",
            "Testing the governance system",
            100,
        ).unwrap();
        
        assert!(!id.is_empty());
        assert_eq!(engine.get_proposals().len(), 1);
    }

    #[test]
    fn test_voting() {
        let config = GovernanceConfig::default();
        let engine = GovernanceEngine::new(config);
        
        let id = engine.create_proposal(
            ProposalType::General {
                title: "Vote Test".to_string(),
                description: "Testing voting".to_string(),
            },
            "proposer1",
            "Vote Test",
            "Testing voting mechanism",
            100,
        ).unwrap();
        
        assert!(engine.cast_vote(&id, "voter1", true, 100_000_000).is_ok());
        assert!(engine.cast_vote(&id, "voter2", false, 50_000_000).is_ok());
        
        let proposal = engine.get_proposals().into_iter()
            .find(|p| p.id == id).unwrap();
        assert_eq!(proposal.votes_for, 100_000_000);
        assert_eq!(proposal.votes_against, 50_000_000);
    }

    #[test]
    fn test_treasury() {
        let treasury = Treasury::new(1_000_000_000);
        assert_eq!(treasury.get_balance(), 1_000_000_000);
        
        assert!(treasury.spend(100_000_000, "recipient1", "Development grant", "governance").is_ok());
        assert_eq!(treasury.get_balance(), 900_000_000);
        
        assert!(treasury.spend(1_000_000_000, "recipient2", "Too much", "governance").is_err());
        assert_eq!(treasury.get_history().len(), 1);
    }
}