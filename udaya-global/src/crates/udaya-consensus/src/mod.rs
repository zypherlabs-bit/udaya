/// Udaya Consensus - Experimental U-ADC consensus engine
/// Hybrid Proof-of-Work + VRF-based validator selection

use crate::primitives::*;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::collections::BTreeMap;

/// Experimental chain parameters
pub const EXPERIMENTIAL_BITS: u32 = 0x207FFFFF;
pub const MAINNET_BITS: u32 = 0x1D00FFFF;

/// Block height
pub type BlockHeight = u64;

/// Committee member ID
pub type ValidatorId = u64;

/// Stake amount (UDYA)
pub type Stake = u64;

/// Finality depth
pub type FinalityDepth = usize;

/// Committee rotation interval (blocks)
pub const ROTATION_INTERVAL: u64 = 1000;

/// Finality depth for committee attestations
pub const FINALITY_DEPTH: usize = 50;

/// Threshold signature requirement (2f+1)
pub const THRESHOLD: usize = 14;

/// Committee size
pub const COMMITTEE_SIZE: usize = 21;

/// Minimum validator stake (UDYA)
pub const MIN_STAKE: u64 = 100_000;

/// Maximum weight percent per validator
pub const MAX_WEIGHT_PCT: u64 = 1;