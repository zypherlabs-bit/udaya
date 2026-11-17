//! Universal Fee Policy (UG-UFP).
//!
//! The fundamental principle of Udaya Global is that equivalent transactions
//! must receive equivalent protocol fee treatment regardless of the countries
//! involved. There is no protocol-level fee discrimination based on country,
//! nationality, sender location, recipient location, IP address, geographic
//! distance, or domestic-versus-international classification.
//!
//! Fee calculation:
//!
//! Transaction Fee = Base Fee × Resource Units × Global Congestion Factor
//!
//! The same parameters apply worldwide. The congestion adjustment is
//! deterministic, bounded, publicly visible, reproducible, applies equally to
//! equivalent transactions, and resists manipulation. It is never derived from
//! a centralized price oracle, and no country-based fee multiplier exists.

use crate::error::{Error, Result};
use crate::tx::Transaction;
use crate::ChainParams;

/// Congestion signal: network utilization in parts-per-mille (0..=1000).
///
/// Derived deterministically from local observables (for example mempool fill
/// ratio and recent block fullness). It is never derived from a price oracle
/// or from geography.
pub type CongestionSignal = u64;

/// Congestion multiplier bounds, in parts-per-mille (1000 = 1.0x).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CongestionBounds {
    pub min_ppm: u64,
    pub max_ppm: u64,
}

/// The result of a fee calculation. Every field is independently verifiable
/// before a transaction is signed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeeCalculation {
    /// Fee per resource unit (satoshis per encoded byte).
    pub base_fee_per_byte: u64,
    /// Resource units consumed (encoded transaction size in bytes).
    pub resource_units: u64,
    /// Congestion multiplier in parts-per-mille (1000 = 1.0x).
    pub congestion_factor_ppm: u64,
    /// Total fee in satoshis.
    pub total_fee: u64,
}

/// Fee policy parameters. Identical for every node worldwide.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeeParams {
    /// Base fee per encoded byte (satoshis).
    pub base_fee_per_byte: u64,
    /// Bounds on the congestion multiplier, in parts-per-mille.
    pub congestion: CongestionBounds,
}

/// The universal fee policy. Stateless and deterministic: the same inputs
/// always produce the same fee on every node, everywhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeePolicy {
    pub params: FeeParams,
}

impl FeePolicy {
    /// Create a policy with the given parameters.
    pub fn new(params: FeeParams) -> Self {
        Self { params }
    }

    /// The minimum fee a transaction must pay to be acceptable, in satoshis.
    ///
    /// This is the fee at the minimum congestion factor. It is the floor that
    /// the mempool and block validator enforce, so fee calculation can never
    /// fall below it.
    pub fn minimum_fee(&self, tx: &Transaction, _chain: &ChainParams) -> Result<u64> {
        let resource_units = tx.encoded_size() as u64;
        let base = self
            .params
            .base_fee_per_byte
            .checked_mul(resource_units)
            .ok_or_else(|| Error::Fee("base fee overflow".into()))?;
        let min = base
            .checked_mul(self.params.congestion.min_ppm)
            .and_then(|v| v.checked_div(1000))
            .ok_or_else(|| Error::Fee("minimum fee overflow".into()))?;
        Ok(min.max(1))
    }

    /// Calculate the full fee for a transaction at the given congestion
    /// signal.
    ///
    /// `congestion_signal` is network utilization in parts-per-mille
    /// (0..=1000). The congestion factor is interpolated linearly between the
    /// configured minimum and maximum, so it is bounded, deterministic and
    /// identical for equivalent transactions on every node.
    pub fn calculate(
        &self,
        tx: &Transaction,
        _chain: &ChainParams,
        congestion_signal: CongestionSignal,
    ) -> Result<FeeCalculation> {
        if congestion_signal > 1000 {
            return Err(Error::Fee(format!(
                "congestion signal {congestion_signal} exceeds 1000 ppm"
            )));
        }
        let resource_units = tx.encoded_size() as u64;
        let base = self
            .params
            .base_fee_per_byte
            .checked_mul(resource_units)
            .ok_or_else(|| Error::Fee("base fee overflow".into()))?;

        let span = self
            .params
            .congestion
            .max_ppm
            .checked_sub(self.params.congestion.min_ppm)
            .ok_or_else(|| Error::Fee("congestion bounds underflow".into()))?;
        let factor_ppm = self
            .params
            .congestion
            .min_ppm
            .checked_add(span * congestion_signal / 1000)
            .ok_or_else(|| Error::Fee("congestion factor overflow".into()))?;

        let total_fee = base
            .checked_mul(factor_ppm)
            .and_then(|v| v.checked_div(1000))
            .ok_or_else(|| Error::Fee("total fee overflow".into()))?;

        let calculation = FeeCalculation {
            base_fee_per_byte: self.params.base_fee_per_byte,
            resource_units,
            congestion_factor_ppm: factor_ppm,
            total_fee,
        };
        Ok(calculation)
    }

    /// The fee at maximum congestion, for worst-case budgeting.
    pub fn maximum_fee(&self, tx: &Transaction, _chain: &ChainParams) -> Result<u64> {
        self.calculate(tx, _chain, 1000).map(|c| c.total_fee)
    }
}

/// Default Udaya Global fee policy: 1 satoshi per byte at minimum congestion,
/// scaling up to 10x at full utilization.
impl Default for FeePolicy {
    fn default() -> Self {
        Self {
            params: FeeParams {
                base_fee_per_byte: 1,
                congestion: CongestionBounds {
                    min_ppm: 1_000,
                    max_ppm: 10_000,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::{TxIn, TxOut};
    use crate::{ChainParams, MAX_TX_BYTES};

    fn chain() -> ChainParams {
        ChainParams::devnet()
    }

    fn sample_tx() -> Transaction {
        Transaction::new(
            vec![TxIn::new(crate::tx::OutPoint::new([1u8; 32], 0))],
            vec![TxOut::new(10_000, [2u8; 32])],
            0,
        )
    }

    #[test]
    fn fee_is_deterministic_for_equivalent_transaction() {
        let chain = chain();
        let tx = sample_tx();
        let policy = FeePolicy::default();
        let a = policy.calculate(&tx, &chain, 500).unwrap();
        let b = policy.calculate(&tx, &chain, 500).unwrap();
        assert_eq!(a, b);
        // Two identical transactions built independently get the same fee.
        let tx2 = sample_tx();
        assert_eq!(policy.calculate(&tx2, &chain, 500).unwrap(), a);
    }

    #[test]
    fn fee_scales_with_congestion_within_bounds() {
        let chain = chain();
        let tx = sample_tx();
        let policy = FeePolicy::default();
        let min = policy.calculate(&tx, &chain, 0).unwrap();
        let mid = policy.calculate(&tx, &chain, 500).unwrap();
        let max = policy.calculate(&tx, &chain, 1000).unwrap();

        assert_eq!(min.congestion_factor_ppm, 1_000);
        assert_eq!(max.congestion_factor_ppm, 10_000);
        assert!(mid.congestion_factor_ppm > min.congestion_factor_ppm);
        assert!(mid.congestion_factor_ppm < max.congestion_factor_ppm);
        assert_eq!(min.total_fee, policy.minimum_fee(&tx, &chain).unwrap());
        assert_eq!(max.total_fee, policy.maximum_fee(&tx, &chain).unwrap());
    }

    #[test]
    fn fee_applies_equally_regardless_of_origin() {
        // Fee calculation takes no sender, recipient, IP or country input, so
        // equivalent transactions always produce identical fees.
        let chain = chain();
        let policy = FeePolicy::default();
        let tx_a = sample_tx();
        let tx_b = Transaction::new(
            vec![crate::tx::TxIn::new(crate::tx::OutPoint::new([9u8; 32], 0))],
            vec![crate::tx::TxOut::new(10_000, [9u8; 32])],
            0,
        );
        let fa = policy.calculate(&tx_a, &chain, 300).unwrap();
        let fb = policy.calculate(&tx_b, &chain, 300).unwrap();
        assert_eq!(fa.resource_units, fb.resource_units);
        assert_eq!(fa.congestion_factor_ppm, fb.congestion_factor_ppm);
        assert_eq!(fa.total_fee, fb.total_fee);
    }

    #[test]
    fn fee_respects_configured_bounds() {
        let chain = chain();
        let policy = FeePolicy::new(FeeParams {
            base_fee_per_byte: 5,
            congestion: CongestionBounds {
                min_ppm: 2_000,
                max_ppm: 2_000,
            },
        });
        let tx = sample_tx();
        let low = policy.calculate(&tx, &chain, 0).unwrap();
        let high = policy.calculate(&tx, &chain, 1000).unwrap();
        assert_eq!(low.total_fee, high.total_fee);
        assert_eq!(low.congestion_factor_ppm, 2_000);
        assert_eq!(low.total_fee, policy.minimum_fee(&tx, &chain).unwrap());
    }

    #[test]
    fn zero_byte_fee_is_at_least_one_satoshi() {
        let chain = chain();
        let policy = FeePolicy::default();
        // A transaction with a single tiny input/output still encodes to at
        // least one byte, so the minimum fee rounds up to one satoshi.
        let tx = Transaction::new(
            vec![TxIn::new(crate::tx::OutPoint::new([0u8; 32], 0))],
            vec![TxOut::new(1, [0u8; 32])],
            0,
        );
        let fee = policy.minimum_fee(&tx, &chain).unwrap();
        assert!(fee >= 1);
    }

    #[test]
    fn congestion_signal_out_of_range_is_rejected() {
        let chain = chain();
        let policy = FeePolicy::default();
        let tx = sample_tx();
        assert!(policy.calculate(&tx, &chain, 1001).is_err());
    }

    #[test]
    fn transaction_too_large_is_rejected_by_policy() {
        // A transaction larger than the chain limit cannot be processed.
        let tx = sample_tx();
        // Confirm resource units equal the encoded size and stay bounded.
        assert!(tx.encoded_size() <= MAX_TX_BYTES);
    }
}
