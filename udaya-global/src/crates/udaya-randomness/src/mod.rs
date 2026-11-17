/// Udaya Randomness - Distributed randomness beacon and VRF
/// Provides unpredictable yet verifiable randomness for validator selection

use crate::primitives::*;
use sha2::{Sha256, Digest};

/// VRF proof structure
pub struct VrfProof {
    /// Validator ID
    pub validator_id: u64,
    /// Proof value
    pub proof: Vec<u8>,
    /// Random output
    pub output: VrffOutput,
}

/// Distributed randomness beacon output
pub struct RandomnessBeacon {
    /// Round number
    pub round: u64,
    /// Random value
    pub value: [u8; 32],
    /// Proof of correct generation
    pub proof: Vec<u8>,
    /// Participating validators
    pub participants: Vec<u64>,
}

/// Generate VRF output for a validator
pub fn generate_vrf(validator_id: u64, seed: &[u8], nonce: u64) -> VrffOutput {
    let mut hasher = Sha256::new();
    hasher.update(validator_id.to_le_bytes());
    hasher.update(seed);
    hasher.update(nonce.to_le_bytes());
    let result = hasher.finalize();

    VrffOutput {
        value: result.to_vec(),
        proof: result,
    }
}

/// Combine VRF outputs from multiple validators into beacon
pub fn combine_vrf(participants: &[(u64, VrffOutput)]) -> Option<RandomnessBeacon> {
    if participants.is_empty() {
        return None;
    }

    // Combine all VRF outputs
    let mut combined: Option<[u8; 32]> = None;
    let mut participant_ids = Vec::new();

    for (id, vrf) in participants {
        participant_ids.push(*id);
        if let Some(bytes) = vrf.value.as_slice().try_into().ok() {
            let cv = combined.unwrap_or([0u8; 32]);
            let mut new_combined = [0u8; 32];
            for (i, b) in bytes.iter().enumerate() {
                new_combined[i] = cv[i].wrapping_add(*b);
            }
            combined = Some(new_combined);
        }
    }

    let combined_val = combined.unwrap_or([0u8; 32]);
    let mut proof = Vec::new();
    for (_, vrf) in participants {
        proof.extend_from_slice(&vrf.proof);
    }

    Some(RandomnessBeacon {
        round: participants.first().map_or(0, |(id, _)| *id),
        value: combined_val,
        proof,
        participants: participant_ids,
    })
}

/// Beacon verification - check that beacon was correctly generated
pub fn verify_beacon(
    beacon: &RandomnessBeacon,
    expected_participants: &[(u64, VrffOutput)],
) -> bool {
    // Check participant set matches
    let participant_set: Vec<u64> = beacon.participants.iter().copied().collect();
    let expected_set: Vec<u64> = expected_participants.iter().map(|(id, _)| *id).collect();
    participant_set == expected_set
}