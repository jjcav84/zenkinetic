//! The thermodynamic privacy gate — the core of ZenKinetic.
//!
//! Adapts orkid's `OrkidKineticHook` thermodynamic gate from MEV protection
//! to privacy protection. The gate evaluates each transaction's negentropy
//! extraction (privacy preservation) and outputs a fee decision:
//!
//! - **Aligned** (privacy-preserving) → lower fees
//! - **Misaligned** (potentially deanonymizing) → higher fees
//!
//! ## The Formula
//!
//! The gate combines three negentropy-powered scores:
//!
//! 1. **Route energy** (from `negentropy::RouteEnergy`) — the core
//!    thermodynamic score: `confidence × √(depth × timing) × latency × (1 − cost)`
//! 2. **Negentropy bits** (from `negentropy::Negentropy`) — information
//!    extracted by the ZK proof: `constraints × log₂(anonymity_set)`
//! 3. **Committor** (from `negentropy::Committor`) — probability the
//!    transaction is valid and privacy-preserving
//!
//! The final fee is: `base_fee × (1 − alignment × stake_discount)`
//!
//! Where `alignment` is the normalized energy score (0..1).

use serde::{Deserialize, Serialize};

use negentropy::{Committor, Negentropy, RouteEnergy};

use crate::profile::{TransactionKind, TransactionProfile};
use crate::staking::ZenStake;

/// The gate's decision for a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyGate {
    /// The gate decision: Allow (aligned) or Penalize (misaligned)
    pub decision: GateDecision,
    /// Alignment score (0.0 = fully misaligned, 1.0 = fully aligned)
    pub alignment: f64,
    /// Route energy score from negentropy (raw)
    pub energy: f64,
    /// Negentropy extracted by the ZK proof (bits)
    pub negentropy_bits: f64,
    /// Committor probability (likelihood tx is valid & privacy-preserving)
    pub committor: f64,
    /// Fee in basis points (0 = free, 1000 = 10%)
    pub fee_bps: u32,
    /// Fee after stake discount
    pub discounted_fee_bps: u32,
    /// Transaction kind classification
    pub kind: TransactionKind,
    /// ZEN stake tier
    pub stake_tier: String,
    /// Timing factor (recency decay)
    pub timing_factor: f64,
    /// Latency decay (proof speed)
    pub latency_decay: f64,
}

/// The gate's verdict on a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateDecision {
    /// Transaction is privacy-preserving (aligned) — low/zero fee
    Allow,
    /// Transaction is potentially deanonymizing (misaligned) — high fee
    Penalize,
    /// Transaction has no ZK proof — standard fee, no privacy features
    Standard,
}

/// Base fee for misaligned (privacy-leaking) transactions, in basis points.
const BASE_FEE_BPS: u32 = 1000; // 10%

/// Fee for aligned (privacy-preserving) transactions, in basis points.
const ALIGNED_FEE_BPS: u32 = 0; // 0% — privacy is free

/// Fee for standard (no proof) transactions, in basis points.
const STANDARD_FEE_BPS: u32 = 100; // 1%

/// Half-life for proof recency decay (1 hour, in seconds).
const HALF_LIFE_SECS: f64 = 3600.0;

/// Maximum energy for normalization (empirically tuned).
const MAX_ENERGY: f64 = 1000.0;

impl PrivacyGate {
    /// Evaluate a transaction through the thermodynamic privacy gate.
    ///
    /// This is the Horizen equivalent of orkid's `beforeSwap` + `getFee`
    /// logic, adapted to score privacy instead of MEV alignment.
    pub fn evaluate(profile: &TransactionProfile) -> Self {
        let kind = profile.kind();
        let stake = ZenStake::new(profile.zen_staked);

        // Transparent transactions: no privacy scoring, standard fee
        if kind == TransactionKind::Transparent {
            let fee = STANDARD_FEE_BPS;
            let discounted = (fee as f64 * (1.0 - stake.fee_discount())).round() as u32;
            return Self {
                decision: GateDecision::Standard,
                alignment: 0.0,
                energy: 0.0,
                negentropy_bits: 0.0,
                committor: 0.0,
                fee_bps: fee,
                discounted_fee_bps: discounted,
                kind,
                stake_tier: stake.tier.label().to_string(),
                timing_factor: 0.0,
                latency_decay: 0.0,
            };
        }

        // --- Privacy-preserving transaction: score via negentropy ---

        // Confidence: base weight from transaction kind
        let confidence = kind.base_confidence();

        // Depth ratio: anonymity set strength (more possible senders = more privacy)
        // For non-anonymous ZK proofs (age, attestations), use constraint count as depth proxy
        let anonymity_set = if profile.anonymity_set_bits >= 64 {
            u64::MAX
        } else {
            1u64 << profile.anonymity_set_bits
        };
        let depth_ratio = if profile.anonymity_set_bits > 0 {
            // Clamp to 64 to prevent f64 precision loss for very large values
            confidence * (profile.anonymity_set_bits.min(64)) as f64 / 4.0
        } else {
            // Non-anonymous proof: depth from constraint count (more constraints = more negentropy)
            confidence * (profile.constraint_count as f64 / 4.0).max(1.0)
        };

        // Timing factor: exponential decay based on proof age
        let timing_factor = (-profile.proof_age_secs / HALF_LIFE_SECS).exp();

        // Latency decay: proof generation + verification speed
        let latency_decay = 1.0 / (1.0 + profile.total_latency_ms() as f64 * 0.0001);

        // Cost penalty: ZEN staking reduces cost (more stake = less penalty)
        let cost_penalty = (1.0 - stake.fee_discount()).clamp(0.0, 0.5) * 0.1;

        // Core energy from negentropy RouteEnergy
        let energy = RouteEnergy::new(
            confidence,
            depth_ratio,
            timing_factor,
            latency_decay,
            cost_penalty,
        )
        .energy;

        // Negentropy extracted: N = constraints × log₂(anonymity_set)
        let negentropy_bits =
            Negentropy::from_constraints(profile.constraint_count, anonymity_set).bits();

        // Committor: probability tx is valid & privacy-preserving
        let committor = Committor::score(depth_ratio, timing_factor, cost_penalty);

        // Alignment: normalized energy (0..1)
        let alignment = (energy / MAX_ENERGY).clamp(0.0, 1.0);

        // Gate decision: aligned if energy is significant
        let decision = if alignment > 0.3 {
            GateDecision::Allow
        } else {
            GateDecision::Penalize
        };

        // Fee: aligned = 0 bps, misaligned = 1000 bps, discounted by stake
        let base_fee = match decision {
            GateDecision::Allow => ALIGNED_FEE_BPS,
            GateDecision::Penalize => BASE_FEE_BPS,
            GateDecision::Standard => STANDARD_FEE_BPS,
        };
        let discounted_fee = (base_fee as f64 * (1.0 - stake.fee_discount())).round() as u32;

        Self {
            decision,
            alignment,
            energy,
            negentropy_bits,
            committor,
            fee_bps: base_fee,
            discounted_fee_bps: discounted_fee,
            kind,
            stake_tier: stake.tier.label().to_string(),
            timing_factor,
            latency_decay,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zk_proof_transaction_aligned() {
        let profile = TransactionProfile::default();
        let gate = PrivacyGate::evaluate(&profile);

        assert_eq!(gate.decision, GateDecision::Allow);
        assert!(gate.alignment > 0.3);
        assert!(gate.negentropy_bits > 0.0);
        assert_eq!(gate.fee_bps, 0); // aligned = free
    }

    #[test]
    fn test_transparent_transaction_standard() {
        let profile = TransactionProfile {
            has_zk_proof: false,
            ..Default::default()
        };
        let gate = PrivacyGate::evaluate(&profile);

        assert_eq!(gate.decision, GateDecision::Standard);
        assert_eq!(gate.fee_bps, 100); // 1% standard fee
        assert_eq!(gate.negentropy_bits, 0.0);
    }

    #[test]
    fn test_stale_proof_penalized() {
        let profile = TransactionProfile {
            proof_age_secs: 14400.0, // 4 hours = 4 half-lives
            ..Default::default()
        };

        let fresh = PrivacyGate::evaluate(&TransactionProfile::default());
        let stale = PrivacyGate::evaluate(&profile);

        assert!(stale.energy < fresh.energy);
        assert!(stale.timing_factor < fresh.timing_factor * 0.1);
    }

    #[test]
    fn test_staking_reduces_fee() {
        let profile_unstaked = TransactionProfile {
            zen_staked: 0.0,
            has_zk_proof: false,
            ..Default::default()
        };
        let profile_max_staked = TransactionProfile {
            zen_staked: 10_000.0,
            has_zk_proof: false,
            ..Default::default()
        };

        let unstaked = PrivacyGate::evaluate(&profile_unstaked);
        let staked = PrivacyGate::evaluate(&profile_max_staked);

        assert!(staked.discounted_fee_bps < unstaked.discounted_fee_bps);
    }

    #[test]
    fn test_deeper_anonymity_more_negentropy() {
        let shallow = TransactionProfile {
            anonymity_set_bits: 4, // 16 senders
            ..Default::default()
        };
        let deep = TransactionProfile {
            anonymity_set_bits: 10, // 1024 senders
            ..Default::default()
        };

        let shallow_gate = PrivacyGate::evaluate(&shallow);
        let deep_gate = PrivacyGate::evaluate(&deep);

        assert!(deep_gate.negentropy_bits > shallow_gate.negentropy_bits);
    }

    #[test]
    fn test_committor_in_range() {
        let profile = TransactionProfile::default();
        let gate = PrivacyGate::evaluate(&profile);

        assert!(gate.committor > 0.0 && gate.committor <= 1.0);
    }

    #[test]
    fn test_anonymous_vote_classification() {
        let profile = TransactionProfile {
            has_zk_proof: true,
            constraint_count: 20,
            anonymity_set_bits: 4,
            ..Default::default()
        };
        let gate = PrivacyGate::evaluate(&profile);

        assert_eq!(gate.kind, TransactionKind::AnonymousVote);
        assert_eq!(gate.decision, GateDecision::Allow);
    }
}
