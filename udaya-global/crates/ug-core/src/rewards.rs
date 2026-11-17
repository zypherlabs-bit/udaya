//! Community Infrastructure Incentive Protocol (UG-CIIP) reward accounting.
//!
//! Udaya Global pays infrastructure rewards from a fixed, explicitly
//! authorized reward reserve allocated at genesis. There is no unlimited
//! issuance, no automatic expansion of the reserve, and no borrowing from any
//! other chain. Unearned or unclaimed rewards remain within the reserve.
//!
//! This module is an off-chain accounting layer: it scores participants
//! deterministically, applies epoch budgets and per-participant caps, and
//! records what may be paid out of the reserve. It does not mint supply and
//! does not affect block validity. The node/dashboard reads these accounts to
//! settle rewards.

use crate::error::{Error, Result};
use std::collections::BTreeMap;

/// Allocation of the fixed reward reserve across incentive components, in
/// basis points (1 bp = 0.01%). The defaults match the initial economic model
/// proposed for simulation: validator 50%, storage 25%, uptime 15%,
/// network service 10%.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReserveAllocation {
    pub validator_bp: u16,
    pub storage_bp: u16,
    pub uptime_bp: u16,
    pub network_service_bp: u16,
}

const BASIS_POINTS: u16 = 10_000;

impl Default for ReserveAllocation {
    fn default() -> Self {
        Self {
            validator_bp: 5_000,
            storage_bp: 2_500,
            uptime_bp: 1_500,
            network_service_bp: 1_000,
        }
    }
}

impl ReserveAllocation {
    pub fn validate(&self) -> Result<()> {
        let total = self
            .validator_bp
            .checked_add(self.storage_bp)
            .and_then(|v| v.checked_add(self.uptime_bp))
            .and_then(|v| v.checked_add(self.network_service_bp))
            .ok_or_else(|| Error::Reward("reserve allocation overflow".into()))?;
        if total != BASIS_POINTS {
            return Err(Error::Reward(format!(
                "reserve allocation must total 10000 bp, got {total}"
            )));
        }
        Ok(())
    }
}

/// Reward tier for continuous uptime, measured over an evaluation window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UptimeTier {
    /// Below 90% availability: no uptime bonus.
    None = 0,
    /// 90% - 94.99%.
    Basic = 1,
    /// 95% - 98.99%.
    Higher = 2,
    /// 99% - 99.89%.
    Premium = 3,
    /// 99.9% and above: maximum.
    Maximum = 4,
}

impl UptimeTier {
    /// Classify an availability measurement expressed in parts per thousand
    /// (1000 = 100%).
    pub fn from_per_mille(availability_per_mille: u64) -> Self {
        match availability_per_mille {
            999..=1000 => UptimeTier::Maximum,
            990..=998 => UptimeTier::Premium,
            950..=989 => UptimeTier::Higher,
            900..=949 => UptimeTier::Basic,
            _ => UptimeTier::None,
        }
    }

    /// Weight used in deterministic scoring (higher tier = more weight).
    pub fn weight(self) -> u64 {
        match self {
            UptimeTier::None => 0,
            UptimeTier::Basic => 1,
            UptimeTier::Higher => 2,
            UptimeTier::Premium => 3,
            UptimeTier::Maximum => 4,
        }
    }
}

/// A registered storage contribution.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StorageContribution {
    /// The storage provider's public key.
    pub provider: [u8; 32],
    /// Bytes the provider advertised committing to store.
    pub committed_bytes: u64,
    /// Bytes verified as actually stored and retrievable (proof of
    /// retrievability challenges passed).
    pub verified_bytes: u64,
    /// Number of randomized storage challenges passed.
    pub challenges_passed: u64,
    /// Number of randomized storage challenges failed.
    pub challenges_failed: u64,
}

/// A network-service credit entry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NetworkServiceCredit {
    pub node: [u8; 32],
    /// Verified useful work credits (transaction/block relay, historical
    /// block serving, sync assistance, onboarding).
    pub credits: u64,
}

/// Outcome of settling one epoch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpochSettlement {
    pub epoch: u64,
    /// Total paid out of the reserve this epoch (satoshis).
    pub total_paid: u64,
    /// Amount paid to validators.
    pub validator_paid: u64,
    /// Amount paid for storage contributions.
    pub storage_paid: u64,
    /// Amount paid for uptime.
    pub uptime_paid: u64,
    /// Amount paid for network service.
    pub network_service_paid: u64,
    /// Reserve remaining after this epoch (satoshis).
    pub reserve_remaining: u64,
}

/// Deterministic reward accounting over a fixed reserve.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RewardAccounting {
    /// Fixed reserve total (satoshis), authorized at genesis.
    pub reserve_total: u64,
    /// Total distributed from the reserve (satoshis).
    pub reserve_distributed: u64,
    /// Per-component distributed totals (satoshis).
    pub validator_distributed: u64,
    pub storage_distributed: u64,
    pub uptime_distributed: u64,
    pub network_service_distributed: u64,
    /// Current epoch number.
    pub epoch: u64,
    /// Storage contributions keyed by provider.
    pub storage: BTreeMap<[u8; 32], StorageContribution>,
    /// Latest uptime availability per node, in parts per thousand.
    pub uptime: BTreeMap<[u8; 32], u64>,
    /// Network service credits per node.
    pub network_service: BTreeMap<[u8; 32], NetworkServiceCredit>,
}

impl RewardAccounting {
    /// Create accounting for a fixed reward reserve.
    pub fn new(reserve_total: u64) -> Self {
        Self {
            reserve_total,
            ..Default::default()
        }
    }

    /// Reserve still available for distribution (satoshis).
    pub fn reserve_available(&self) -> u64 {
        self.reserve_total.saturating_sub(self.reserve_distributed)
    }

    /// Register or update a storage contribution. Only verified bytes count
    /// toward rewards; advertised-but-unverified storage earns nothing.
    pub fn record_storage_contribution(
        &mut self,
        provider: [u8; 32],
        committed_bytes: u64,
        verified_bytes: u64,
        challenges_passed: u64,
        challenges_failed: u64,
    ) -> Result<()> {
        if verified_bytes > committed_bytes {
            return Err(Error::Reward(format!(
                "provider {}: verified bytes {} exceed committed bytes {}",
                hex::encode(provider),
                verified_bytes,
                committed_bytes
            )));
        }
        let entry = self.storage.entry(provider).or_insert(StorageContribution {
            provider,
            ..Default::default()
        });
        entry.committed_bytes = committed_bytes;
        entry.verified_bytes = verified_bytes;
        entry.challenges_passed += challenges_passed;
        entry.challenges_failed += challenges_failed;
        Ok(())
    }

    /// Record an uptime measurement for a node (availability in parts per
    /// thousand, measured by multiple independent peers).
    pub fn record_uptime(&mut self, node: [u8; 32], availability_per_mille: u64) {
        self.uptime.insert(node, availability_per_mille.min(1000));
    }

    /// Record verified network-service credits for a node.
    pub fn record_network_service(&mut self, node: [u8; 32], credits: u64) {
        let entry = self
            .network_service
            .entry(node)
            .or_insert(NetworkServiceCredit {
                node,
                ..Default::default()
            });
        entry.credits = entry.credits.saturating_add(credits);
    }

    /// Deterministic storage score for a provider: verified bytes weighted by
    /// challenge reliability. A provider with no verified storage scores zero.
    fn storage_score(&self, c: &StorageContribution) -> u64 {
        if c.verified_bytes == 0 || c.challenges_passed + c.challenges_failed == 0 {
            return 0;
        }
        let reliability = c
            .challenges_passed
            .checked_div(c.challenges_passed + c.challenges_failed)
            .unwrap_or(0);
        // Score = verified bytes * reliability (per-mille), so fake data
        // requests that are never retrieved do not inflate the score.
        c.verified_bytes
            .checked_mul(reliability)
            .unwrap_or(0)
            .saturating_div(1000)
    }

    /// Settle one epoch. Distributes the epoch budget from the reserve
    /// according to `allocation`, capped per component and per participant,
    /// and never paying more than the reserve holds.
    pub fn settle_epoch(
        &mut self,
        allocation: &ReserveAllocation,
        max_epoch_payout: u64,
        max_per_participant: u64,
    ) -> Result<EpochSettlement> {
        allocation.validate()?;
        let available = self.reserve_available();
        let budget = available.min(max_epoch_payout);

        let total_bp = allocation.validate_bp_sum();
        let component_budget = |bp: u16| -> u64 {
            budget
                .checked_mul(bp as u64)
                .and_then(|v| v.checked_div(total_bp))
                .unwrap_or(0)
        };

        let storage_budget = component_budget(allocation.storage_bp);
        let uptime_budget = component_budget(allocation.uptime_bp);
        let network_budget = component_budget(allocation.network_service_bp);

        // Validator rewards are settled by the consensus layer based on
        // verified validator performance (Section 6.3). This module accounts
        // for the validator budget line but does not auto-pay it: rewards
        // depend on useful and correct participation, and unearned rewards
        // remain in the reserve.
        let validator_paid = 0u64;

        let storage_paid = self.settle_storage(storage_budget, max_per_participant)?;
        let uptime_paid = self.settle_uptime(uptime_budget, max_per_participant)?;
        let network_paid = self.settle_network_service(network_budget, max_per_participant)?;

        let total_paid = validator_paid
            .checked_add(storage_paid)
            .and_then(|v| v.checked_add(uptime_paid))
            .and_then(|v| v.checked_add(network_paid))
            .ok_or_else(|| Error::Reward("settlement overflow".into()))?;

        if total_paid > available {
            return Err(Error::Reward("settlement exceeds reserve".into()));
        }

        self.validator_distributed = self
            .validator_distributed
            .checked_add(validator_paid)
            .ok_or_else(|| Error::Reward("overflow".into()))?;
        self.storage_distributed = self
            .storage_distributed
            .checked_add(storage_paid)
            .ok_or_else(|| Error::Reward("overflow".into()))?;
        self.uptime_distributed = self
            .uptime_distributed
            .checked_add(uptime_paid)
            .ok_or_else(|| Error::Reward("overflow".into()))?;
        self.network_service_distributed = self
            .network_service_distributed
            .checked_add(network_paid)
            .ok_or_else(|| Error::Reward("overflow".into()))?;
        self.reserve_distributed = self
            .reserve_distributed
            .checked_add(total_paid)
            .ok_or_else(|| Error::Reward("overflow".into()))?;
        self.epoch += 1;

        Ok(EpochSettlement {
            epoch: self.epoch,
            total_paid,
            validator_paid,
            storage_paid,
            uptime_paid,
            network_service_paid: network_paid,
            reserve_remaining: self.reserve_available(),
        })
    }

    /// Distribute the storage budget proportionally to deterministic scores,
    /// capped per provider.
    fn settle_storage(&mut self, budget: u64, max_per_participant: u64) -> Result<u64> {
        let mut scores: Vec<([u8; 32], u64)> = self
            .storage
            .iter()
            .map(|(provider, c)| (*provider, self.storage_score(c)))
            .filter(|(_, score)| *score > 0)
            .collect();
        scores.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let total_score: u64 = scores.iter().map(|(_, s)| *s).sum();
        if total_score == 0 || budget == 0 {
            return Ok(0);
        }

        let mut paid = 0u64;
        let mut remaining_budget = budget;
        for (_, score) in scores {
            let share = budget
                .checked_mul(score)
                .and_then(|v| v.checked_div(total_score))
                .unwrap_or(0);
            let capped = share.min(max_per_participant).min(remaining_budget);
            paid = paid
                .checked_add(capped)
                .ok_or_else(|| Error::Reward("overflow".into()))?;
            remaining_budget = remaining_budget.saturating_sub(capped);
            if remaining_budget == 0 {
                break;
            }
        }
        Ok(paid)
    }

    /// Distribute the uptime budget proportionally to tier weight, capped per
    /// node. Nodes below the 90% threshold receive nothing.
    fn settle_uptime(&mut self, budget: u64, max_per_participant: u64) -> Result<u64> {
        let mut weighted: Vec<([u8; 32], u64)> = self
            .uptime
            .iter()
            .map(|(node, availability)| (*node, UptimeTier::from_per_mille(*availability).weight()))
            .filter(|(_, weight)| *weight > 0)
            .collect();
        weighted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let total_weight: u64 = weighted.iter().map(|(_, w)| *w).sum();
        if total_weight == 0 || budget == 0 {
            return Ok(0);
        }

        let mut paid = 0u64;
        let mut remaining_budget = budget;
        for (_, weight) in weighted {
            let share = budget
                .checked_mul(weight)
                .and_then(|v| v.checked_div(total_weight))
                .unwrap_or(0);
            let capped = share.min(max_per_participant).min(remaining_budget);
            paid = paid
                .checked_add(capped)
                .ok_or_else(|| Error::Reward("overflow".into()))?;
            remaining_budget = remaining_budget.saturating_sub(capped);
            if remaining_budget == 0 {
                break;
            }
        }
        Ok(paid)
    }

    /// Distribute the network-service budget proportionally to credits,
    /// capped per node. Artificial traffic that does not correspond to
    /// verified useful work earns no credits.
    fn settle_network_service(&mut self, budget: u64, max_per_participant: u64) -> Result<u64> {
        let mut credits: Vec<([u8; 32], u64)> = self
            .network_service
            .values()
            .map(|c| (c.node, c.credits))
            .filter(|(_, c)| *c > 0)
            .collect();
        credits.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let total_credits: u64 = credits.iter().map(|(_, c)| *c).sum();
        if total_credits == 0 || budget == 0 {
            return Ok(0);
        }

        let mut paid = 0u64;
        let mut remaining_budget = budget;
        for (_, credit) in credits {
            let share = budget
                .checked_mul(credit)
                .and_then(|v| v.checked_div(total_credits))
                .unwrap_or(0);
            let capped = share.min(max_per_participant).min(remaining_budget);
            paid = paid
                .checked_add(capped)
                .ok_or_else(|| Error::Reward("overflow".into()))?;
            remaining_budget = remaining_budget.saturating_sub(capped);
            if remaining_budget == 0 {
                break;
            }
        }
        Ok(paid)
    }
}

impl ReserveAllocation {
    fn validate_bp_sum(&self) -> u64 {
        self.validator_bp as u64
            + self.storage_bp as u64
            + self.uptime_bp as u64
            + self.network_service_bp as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider(seed: u8) -> [u8; 32] {
        [seed; 32]
    }

    #[test]
    fn default_allocation_totals_ten_thousand_bp() {
        let a = ReserveAllocation::default();
        assert!(a.validate().is_ok());
        assert_eq!(
            a.validator_bp + a.storage_bp + a.uptime_bp + a.network_service_bp,
            10_000
        );
    }

    #[test]
    fn non_ten_thousand_bp_allocation_is_rejected() {
        let a = ReserveAllocation {
            validator_bp: 6000,
            ..ReserveAllocation::default()
        };
        assert!(a.validate().is_err());
    }

    #[test]
    fn verified_bytes_only_count_toward_storage_score() {
        let mut acct = RewardAccounting::new(1_000_000_000);
        let p = provider(1);
        // Advertises 1 TB but verifies nothing: scores zero.
        acct.record_storage_contribution(p, 1_000_000_000_000, 0, 0, 0)
            .unwrap();
        assert_eq!(acct.storage_score(acct.storage.get(&p).unwrap()), 0);

        // Verifies 100 GB with perfect reliability: scores proportional.
        acct.record_storage_contribution(p, 100_000_000_000, 100_000_000_000, 100, 0)
            .unwrap();
        assert!(acct.storage_score(acct.storage.get(&p).unwrap()) > 0);
    }

    #[test]
    fn verified_bytes_cannot_exceed_committed_bytes() {
        let mut acct = RewardAccounting::new(1_000_000_000);
        let err = acct
            .record_storage_contribution(provider(1), 100, 200, 0, 0)
            .unwrap_err();
        assert!(err.to_string().contains("exceed"));
    }

    #[test]
    fn uptime_below_ninety_percent_receives_nothing() {
        let mut acct = RewardAccounting::new(1_000_000_000);
        acct.record_uptime(provider(1), 899); // 89.9%
        assert_eq!(UptimeTier::from_per_mille(899), UptimeTier::None);
        assert_eq!(UptimeTier::from_per_mille(899).weight(), 0);
        assert_eq!(UptimeTier::from_per_mille(999), UptimeTier::Maximum);
        assert_eq!(UptimeTier::from_per_mille(999).weight(), 4);
    }

    #[test]
    fn settle_epoch_never_exceeds_reserve() {
        let mut acct = RewardAccounting::new(1_000_000);
        // Give storage a verified contribution so it can earn.
        acct.record_storage_contribution(provider(1), 10_000_000_000, 10_000_000_000, 100, 0)
            .unwrap();

        let allocation = ReserveAllocation::default();
        let settlement = acct.settle_epoch(&allocation, 500_000, 100_000).unwrap();
        assert!(settlement.total_paid <= 1_000_000);
        assert!(settlement.reserve_remaining <= 1_000_000);
        assert_eq!(
            settlement.validator_paid
                + settlement.storage_paid
                + settlement.uptime_paid
                + settlement.network_service_paid,
            settlement.total_paid
        );
        // Validator budget is not auto-paid by this module; it stays in the
        // reserve until settled by the consensus layer.
        assert_eq!(settlement.validator_paid, 0);
    }

    #[test]
    fn settle_epoch_respects_per_participant_cap() {
        let mut acct = RewardAccounting::new(1_000_000_000);
        acct.record_storage_contribution(provider(1), 10_000_000_000, 10_000_000_000, 100, 0)
            .unwrap();
        acct.record_storage_contribution(provider(2), 10_000_000_000, 10_000_000_000, 100, 0)
            .unwrap();

        let allocation = ReserveAllocation::default();
        // Small per-participant cap: each provider gets at most the cap.
        let settlement = acct.settle_epoch(&allocation, 10_000_000, 1_000).unwrap();
        assert!(settlement.storage_paid <= 2_000);
    }

    #[test]
    fn reserve_exhaustion_stops_payouts() {
        let mut acct = RewardAccounting::new(100);
        acct.record_storage_contribution(provider(1), 10_000_000_000, 10_000_000_000, 100, 0)
            .unwrap();
        let allocation = ReserveAllocation::default();
        let first = acct
            .settle_epoch(&allocation, 10_000_000, 1_000_000)
            .unwrap();
        assert!(first.total_paid <= 100);
        // Reserve is exhausted or near-exhausted; next settlement pays little
        // or nothing.
        let second = acct
            .settle_epoch(&allocation, 10_000_000, 1_000_000)
            .unwrap();
        assert!(second.total_paid <= first.reserve_remaining);
        assert!(second.total_paid <= 100);
    }

    #[test]
    fn scoring_is_deterministic_across_independent_accounts() {
        let mut a = RewardAccounting::new(1_000_000_000);
        let mut b = RewardAccounting::new(1_000_000_000);
        for seed in 1..=10u8 {
            let p = provider(seed);
            a.record_storage_contribution(p, 100_000_000_000, 100_000_000_000, seed as u64 * 10, 0)
                .unwrap();
            a.record_uptime(p, 990);
            a.record_network_service(p, seed as u64 * 100);

            b.record_storage_contribution(p, 100_000_000_000, 100_000_000_000, seed as u64 * 10, 0)
                .unwrap();
            b.record_uptime(p, 990);
            b.record_network_service(p, seed as u64 * 100);
        }
        let alloc = ReserveAllocation::default();
        let sa = a.settle_epoch(&alloc, 5_000_000, 1_000_000).unwrap();
        let sb = b.settle_epoch(&alloc, 5_000_000, 1_000_000).unwrap();
        assert_eq!(sa, sb);
        assert_eq!(a.reserve_distributed, b.reserve_distributed);
    }

    #[test]
    fn unclaimed_rewards_remain_in_reserve() {
        let mut acct = RewardAccounting::new(1_000_000_000);
        // No contributions at all: nothing is paid, reserve untouched.
        let alloc = ReserveAllocation::default();
        let settlement = acct.settle_epoch(&alloc, 10_000_000, 1_000_000).unwrap();
        assert_eq!(settlement.total_paid, 0);
        assert_eq!(acct.reserve_available(), 1_000_000_000);
        assert_eq!(acct.reserve_distributed, 0);
    }
}
