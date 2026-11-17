/// Udaya Admission - Validator admission and Sybil resistance
/// Bonded participation with VRF-based selection and stake weighting

use crate::primitives::*;
use crate::crypto::ThresholdSignature;
use std::collections::BTreeMap;

/// Validator admission status
#[derive(Debug, Clone, PartialEq)]
pub enum AdmissionStatus {
    /// Admitted with valid stake
    Admitted,
    /// Stake below minimum threshold
    InsufficientStake,
    /// Validator already active
    AlreadyActive,
    /// Slashing condition violated
    Slashed,
}

/// Validator information
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    /// Validator ID (public key hash)
    pub id: ValidatorId,
    /// Staked UDYA amount
    pub stake: Stake,
    /// VRF public key
    pub vrf_key: Vec<u8>,
    /// Join height
    pub join_height: u64,
    /// Last activity height
    pub last_active: u64,
    /// Penalties accumulated
    pub penalties: u32,
}

/// Admission result
#[derive(Debug, Clone)]
pub struct AdmissionResult {
    /// Whether admission was successful
    pub admitted: bool,
    /// Reason string
    pub reason: String,
    /// Validator info if admitted
    pub validator: Option<ValidatorInfo>,
}

/// Admission manager for bonded participation
pub struct AdmissionManager {
    /// Active validators (id -> stake)
    active_validators: BTreeMap<ValidatorId, Stake>,
    /// Validator info lookup
    validator_info: BTreeMap<ValidatorId, ValidatorInfo>,
    /// Minimum stake requirement
    min_stake: Stake,
    /// Slashing penalty percent
    slashing_pct: u32,
}

impl AdmissionManager {
    /// Create new admission manager
    pub fn new(min_stake: Stake, slashing_pct: u32) -> Self {
        AdmissionManager {
            active_validators: BTreeMap::new(),
            validator_info: BTreeMap::new(),
            min_stake,
            slashing_pct,
        }
    }

    /// Attempt to admit a new validator
    pub fn attempt_admission(
        &mut self,
        id: ValidatorId,
        stake: Stake,
        vrf_key: Vec<u8>,
    ) -> AdmissionResult {
        // Check minimum stake
        if stake < self.min_stake {
            return AdmissionResult {
                admitted: false,
                reason: format!("Stake {} below minimum {}", stake, self.min_stake),
                validator: None,
            };
        }

        // Check if already active
        if self.active_validators.contains_key(&id) {
            return AdmissionResult {
                admitted: false,
                reason: "Validator already active".to_string(),
                validator: None,
            };
        }

        // Admit validator
        let info = ValidatorInfo {
            id,
            stake,
            vrf_key,
            join_height: 0, // Would be set at actual join
            last_active: 0,
            penalties: 0,
        };

        self.active_validators.insert(id, stake);
        self.validator_info.insert(id, info.clone());

        AdmissionResult {
            admitted: true,
            reason: "Admitted".to_string(),
            validator: Some(info),
        }
    }

    /// Slash a validator for violations
    pub fn slash(&mut self, id: ValidatorId, violation: &str) -> Option<Stake> {
        let stake = self.active_validators.get(&id)?;
        let penalty = (*stake as u64 * self.slashing_pct as u64 / 100) as u64;

        // Remove from active set
        self.active_validators.remove(&id);
        self.validator_info.remove(&id);

        Some(stake)
    }

    /// Record validator activity at given height
    pub fn record_activity(&mut self, id: ValidatorId, height: u64) {
        if let Some(validator) = self.validator_info.get_mut(&id) {
            validator.last_active = height;
            validator.penalties = validator.penalties.saturating_sub(1); // Penalty decay
        }
    }

    /// Increase penalties for misbehavior
    pub fn penalize(&mut self, id: ValidatorId, amount: u32) {
        if let Some(validator) = self.validator_info.get_mut(&id) {
            validator.penalties = validator.penalties.saturating_add(amount);
        }
    }

    /// Get active validator count
    pub fn active_count(&self) -> usize {
        self.active_validators.len()
    }

    /// Get stake for validator
    pub fn get_stake(&self, id: ValidatorId) -> Option<Stake> {
        self.active_validators.get(&id).copied()
    }
}

/// Bonded participation admission logic
pub fn admit_validator(
    min_stake: Stake,
    slashing_pct: u32,
    active: &mut BTreeMap<ValidatorId, Stake>,
    info: &mut BTreeMap<ValidatorId, ValidatorInfo>,
    id: ValidatorId,
    stake: Stake,
    vrf_key: Vec<u8>,
) -> AdmissionResult {
    // Check minimum stake
    if stake < min_stake {
        return AdmissionResult {
            admitted: false,
            reason: format!("Stake {} below minimum {}", stake, min_stake),
            validator: None,
        };
    }

    // Check if already active
    if active.contains_key(&id) {
        return AdmissionResult {
            admitted: false,
            reason: "Validator already active".to_string(),
            validator: None,
        };
    }

    // Admit validator
    let validator_info = ValidatorInfo {
        id,
        stake,
        vrf_key,
        join_height: 0,
        last_active: 0,
        penalties: 0,
    };

    active.insert(id, stake);
    info.insert(id, validator_info);

    AdmissionResult {
        admitted: true,
        reason: "Admitted".to_string(),
        validator: Some(validator_info),
    }
}

/// Slash a validator for violations
pub fn slash_validator(
    active: &mut BTreeMap<ValidatorId, Stake>,
    info: &mut BTreeMap<ValidatorId, ValidatorInfo>,
    id: ValidatorId,
    violation: &str,
) -> Option<Stake> {
    let stake = active.get(&id)?;
    let penalty = (*stake as u64 * SLASHING_PCT as u64 / 100) as u64;

    active.remove(&id);
    info.remove(&id);

    Some(stake)
}