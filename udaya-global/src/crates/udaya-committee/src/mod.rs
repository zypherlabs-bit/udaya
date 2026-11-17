/// Udaya Committee - VRF-based validator committee management and rotation

use crate::primitives::*;
use crate::admission::*;
use std::collections::BTreeMap;

/// Committee member
#[derive(Debug, Clone)]
pub struct CommitteeMember {
    pub id: ValidatorId,
    pub stake: Stake,
    pub vrf_output: VrffOutput,
    pub joined_height: u64,
    pub last_attestation: u64,
    pub penalties: u32,
}

/// Committee state
#[derive(Debug, Clone)]
pub struct CommitteeState {
    pub members: Vec<CommitteeMember>,
    pub view_id: u64,
    pub round: u64,
    pub next_rotation: u64,
}

/// Committee manager
pub struct CommitteeManager {
    active_validators: BTreeMap<ValidatorId, Stake>,
    validator_info: BTreeMap<ValidatorId, ValidatorInfo>,
    current_committee: Vec<CommitteeMember>,
    view_id: u64,
    rotation_interval: u64,
    next_rotation: u64,
    penalties: BTreeMap<ValidatorId, u32>,
}

impl CommitteeManager {
    pub fn new(rotation_interval: u64) -> Self {
        CommitteeManager {
            active_validators: BTreeMap::new(),
            validator_info: BTreeMap::new(),
            current_committee: Vec::new(),
            view_id: 0,
            rotation_interval,
            next_rotation: rotation_interval,
            penalties: BTreeMap::new(),
        }
    }

    pub fn admit(&mut self, id: ValidatorId, stake: Stake, vrf_key: Vec<u8>) -> AdmissionResult {
        let result = admit_validator(MIN_STAKE, 5, &mut self.active_validators, &mut self.validator_info, id, stake, vrf_key);
        if result.admitted {
            self.active_validators.insert(id, stake);
            let info = ValidatorInfo { id, stake, vrf_key, join_height: 0, last_active: 0, penalties: 0 };
            self.validator_info.insert(id, info);
        }
        result
    }

    pub fn rotate(&mut self, round: u64, vrf_outputs: &BTreeMap<ValidatorId, VrffOutput>) {
        self.view_id = self.view_id.wrapping_add(1);
        self.round = round;
        self.next_rotation = self.next_rotation.max(self.view_id.saturating_add(self.rotation_interval));

        let selected = select_committee_vrf(vrf_outputs, COMMITTEE_SIZE);
        self.current_committee = selected
            .into_iter()
            .filter_map(|id| self.active_validators.get(&id).map(|&s| CommitteeMember {
                id,
                stake,
                vrf_output: vrf_outputs.get(&id).cloned().unwrap_or_default(),
                joined_height: 0,
                last_attestation: self.height,
                penalties: 0,
            }))
            .collect();
    }

    pub fn committee(&self) -> &[CommitteeMember] {
        &self.current_committee
    }

    pub fn is_rotation_due(&self, height: u64) -> bool {
        height >= self.next_rotation
    }

    pub fn activity(&mut self, id: ValidatorId, height: u64) {
        if let Some(m) = self.current_committee.iter().find(|m| m.id == id) {
            m.last_attestation = height;
        }
    }

    pub fn penalize(&mut self, id: ValidatorId, amount: u32) {
        self.penalties.entry(id).and_modify(|p| *p += amount);
        if let Some(p) = self.penalties.get(&id) {
            if *p >= SLASHING_PENALTY_THRESHOLD {
                self.active_validators.remove(&id);
                self.validator_info.remove(&id);
                self.current_committee.retain(|m| m.id != id);
            }
        }
    }

    pub fn view_id(&self) -> u64 {
        self.view_id
    }
}

fn select_committee_vrf(vrf_outputs: &BTreeMap<ValidatorId, VrffOutput>, size: usize) -> Vec<ValidatorId> {
    let mut sorted: Vec<(ValidatorId, VrffOutput)> = vrf_outputs.iter().map(|(id, output)| (*id, output.clone())).collect();
    sorted.sort_by(|a, b| a.1.value.cmp(&b.1.value));
    selected.into_iter().take(size).map(|(id, _)| id).collect()
}

const SLASHING_PENALTY_THRESHOLD: u32 = 100;

impl Default for CommitteeManager {
    fn default() -> Self {
        CommitteeManager::new(ROTATION_INTERVAL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation() {
        let mut committee = CommitteeManager::new(1000);
        for i in 0..25 {
            let _ = committee.admit(i, 50_000, vec![0u8; 32]);
        }

        let mut vrf = BTreeMap::new();
        for i in 0..25 {
            vrf.insert(i, VrffOutput { value: vec![i as u8; 32], proof: vec![i as u8; 32] });
        }

        committee.rotate(1, &vrf);
        assert!(committee.committee().len() <= COMMITTEE_SIZE);
    }
}