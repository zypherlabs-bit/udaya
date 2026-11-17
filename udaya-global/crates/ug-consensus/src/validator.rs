//! The U-ADC validator set.
//!
//! The committee is fixed at genesis and consists of exactly `3f+1` unique
//! Ed25519 public keys. With `n = 3f+1`, the fault bound is `f` and the quorum
//! threshold is `2f+1`. For example, `n = 31` gives `f = 10` and a quorum of
//! `21`.
//!
//! Validators are sorted lexicographically by public key. This makes proposer
//! selection independent of the order in which keys appear in the genesis
//! file, so two nodes with the same set always agree on who proposes.

use crate::{Error, Result};
use std::collections::BTreeMap;
use ug_core::crypto::Hash32;
use ug_core::{Height, Round};

/// Minimum number of validators accepted by U-ADC.
pub const MIN_VALIDATORS: usize = 4;
/// Maximum number of validators accepted by U-ADC.
pub const MAX_VALIDATORS: usize = 100;

/// A deterministic, validated validator committee.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatorSet {
    ordered: Vec<Hash32>,
    index: BTreeMap<Hash32, usize>,
}

impl ValidatorSet {
    /// Build a validator set from public keys.
    ///
    /// Rejects duplicate keys, zero keys, counts outside `4..=100`, and any
    /// count that is not of the form `3f+1`.
    pub fn new(keys: impl IntoIterator<Item = Hash32>) -> Result<Self> {
        let mut ordered: Vec<Hash32> = keys.into_iter().collect();
        if ordered.len() < MIN_VALIDATORS {
            return Err(Error::ValidatorSet(format!(
                "at least {MIN_VALIDATORS} validators are required, got {}",
                ordered.len()
            )));
        }
        if ordered.len() > MAX_VALIDATORS {
            return Err(Error::ValidatorSet(format!(
                "at most {MAX_VALIDATORS} validators are supported, got {}",
                ordered.len()
            )));
        }
        let n = ordered.len();
        let f = (n - 1) / 3;
        if n != 3 * f + 1 {
            return Err(Error::ValidatorSet(format!(
                "validator count {n} is not of the form 3f+1"
            )));
        }
        ordered.sort_unstable();
        if ordered[0] == [0u8; 32] {
            return Err(Error::ValidatorSet("zero validator key".into()));
        }
        for pair in ordered.windows(2) {
            if pair[0] == pair[1] {
                return Err(Error::ValidatorSet(
                    "duplicate validator key in committee".into(),
                ));
            }
        }
        let index = ordered
            .iter()
            .enumerate()
            .map(|(position, key)| (*key, position))
            .collect();
        Ok(Self { ordered, index })
    }

    pub fn from_slice(keys: &[Hash32]) -> Result<Self> {
        Self::new(keys.iter().copied())
    }

    /// Number of validators.
    pub fn len(&self) -> usize {
        self.ordered.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ordered.is_empty()
    }

    /// Maximum number of Byzantine validators tolerated: `f = (n-1)/3`.
    pub fn fault_tolerance(&self) -> usize {
        (self.ordered.len() - 1) / 3
    }

    /// Quorum size: `2f+1`.
    pub fn quorum(&self) -> usize {
        2 * self.fault_tolerance() + 1
    }

    pub fn contains(&self, key: &Hash32) -> bool {
        self.index.contains_key(key)
    }

    pub fn position(&self, key: &Hash32) -> Option<usize> {
        self.index.get(key).copied()
    }

    pub fn validators(&self) -> &[Hash32] {
        &self.ordered
    }

    /// Deterministic round-robin proposer for `(height, round)`.
    ///
    /// The arithmetic is performed in `u128` so it cannot overflow for any
    /// accepted height and round.
    pub fn proposer(&self, height: Height, round: Round) -> Hash32 {
        let position = ((height as u128 + round as u128) % self.ordered.len() as u128) as usize;
        self.ordered[position]
    }

    /// True when `key` is the proposer for `(height, round)`.
    pub fn is_proposer(&self, height: Height, round: Round, key: &Hash32) -> bool {
        self.proposer(height, round) == *key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(seed: u8) -> Hash32 {
        [seed; 32]
    }

    fn keys(count: u8) -> Vec<Hash32> {
        (1..=count).map(key).collect()
    }

    #[test]
    fn accepts_3f_plus_1_and_computes_quorum() {
        let set = ValidatorSet::new(keys(4)).unwrap();
        assert_eq!(set.fault_tolerance(), 1);
        assert_eq!(set.quorum(), 3);

        let set = ValidatorSet::new(keys(7)).unwrap();
        assert_eq!(set.fault_tolerance(), 2);
        assert_eq!(set.quorum(), 5);

        let set = ValidatorSet::new(keys(31)).unwrap();
        assert_eq!(set.len(), 31);
        assert_eq!(set.fault_tolerance(), 10);
        assert_eq!(set.quorum(), 21);
    }

    #[test]
    fn rejects_wrong_sizes_and_duplicates() {
        for count in [0usize, 1, 2, 3, 5, 6, 8, 9] {
            let keys: Vec<Hash32> = (0..count).map(|i| [(i + 1) as u8; 32]).collect();
            assert!(
                ValidatorSet::new(keys).is_err(),
                "accepted invalid count {count}"
            );
        }

        let mut duplicated = keys(4);
        duplicated[3] = duplicated[0];
        assert!(ValidatorSet::new(duplicated).is_err());

        let mut zero = keys(4);
        zero[0] = [0u8; 32];
        assert!(ValidatorSet::new(zero).is_err());
    }

    #[test]
    fn sorting_makes_the_set_order_independent() {
        let mut a = keys(7);
        let mut b = a.clone();
        a.reverse();
        b.swap(1, 5);
        assert_eq!(ValidatorSet::new(a).unwrap(), ValidatorSet::new(b).unwrap());
    }

    #[test]
    fn proposer_selection_is_deterministic_and_round_robin() {
        let set = ValidatorSet::new(keys(4)).unwrap();
        assert_eq!(set.proposer(0, 0), set.validators()[0]);
        assert_eq!(set.proposer(0, 1), set.validators()[1]);
        assert_eq!(set.proposer(1, 0), set.validators()[1]);
        assert_eq!(set.proposer(4, 0), set.validators()[0]);
        assert_eq!(set.proposer(u64::MAX, 10_000_000), {
            let position = ((u64::MAX as u128 + 10_000_000u128) % 4) as usize;
            set.validators()[position]
        });
    }

    #[test]
    fn membership_checks_are_exact() {
        let set = ValidatorSet::new(keys(4)).unwrap();
        assert!(set.contains(&key(1)));
        assert!(!set.contains(&key(9)));
        assert_eq!(set.position(&key(1)), Some(0));
        assert_eq!(set.position(&key(9)), None);
    }
}
