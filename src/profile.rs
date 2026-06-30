//! Transaction profiling — classifying Horizen transactions by privacy kind.

use serde::{Deserialize, Serialize};

/// The kind of transaction being evaluated by the privacy gate.
///
/// Each kind maps to a different negentropy extraction model, analogous to
/// how orkid maps different DEX route types to the FMD energy formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionKind {
    /// ZK proof transaction (age verification, attestation, vote) — highest privacy
    ZkProof,
    /// Confidential transfer (shielded pool) — high privacy
    ConfidentialTransfer,
    /// Anonymous vote (zk-ballot pattern) — high privacy, time-bounded
    AnonymousVote,
    /// Standard transparent transaction — no privacy, baseline
    Transparent,
}

impl TransactionKind {
    /// Base confidence weight for this transaction kind (0..100).
    ///
    /// Analogous to `AttestationType::base_depth()` in zk-attest —
    /// privacy-preserving kinds have higher base confidence because they
    /// extract more negentropy from the network's information state.
    pub fn base_confidence(&self) -> f64 {
        match self {
            Self::ZkProof => 100.0,
            Self::ConfidentialTransfer => 90.0,
            Self::AnonymousVote => 85.0,
            Self::Transparent => 10.0,
        }
    }

    /// Whether this kind carries a ZK proof (negentropy extraction).
    pub fn has_proof(&self) -> bool {
        matches!(self, Self::ZkProof | Self::ConfidentialTransfer | Self::AnonymousVote)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::ZkProof => "zk-proof",
            Self::ConfidentialTransfer => "confidential-transfer",
            Self::AnonymousVote => "anonymous-vote",
            Self::Transparent => "transparent",
        }
    }
}

/// Profile of a transaction being evaluated by the privacy gate.
///
/// This is the Horizen equivalent of orkid's `RoutePotential` —
/// the input configuration from which the thermodynamic gate computes
/// its fee decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionProfile {
    /// Whether the transaction includes a ZK proof
    pub has_zk_proof: bool,
    /// Number of constraints in the ZK circuit (if proof present)
    pub constraint_count: u64,
    /// Anonymity set size in bits (log₂ of possible senders)
    pub anonymity_set_bits: u64,
    /// Age of the proof in seconds (recency decay)
    pub proof_age_secs: f64,
    /// Proof generation latency in milliseconds
    pub proof_latency_ms: u64,
    /// Verification latency in milliseconds
    pub verify_latency_ms: u64,
    /// ZEN tokens staked by the sender (for fee tier discounts)
    pub zen_staked: f64,
}

impl TransactionProfile {
    /// Derive the transaction kind from the profile.
    pub fn kind(&self) -> TransactionKind {
        if !self.has_zk_proof {
            return TransactionKind::Transparent;
        }
        if self.anonymity_set_bits >= 1 && self.constraint_count >= 15 {
            TransactionKind::AnonymousVote
        } else if self.anonymity_set_bits > 0 {
            TransactionKind::ConfidentialTransfer
        } else {
            TransactionKind::ZkProof
        }
    }

    /// Total latency (proof gen + verify) in milliseconds.
    pub fn total_latency_ms(&self) -> u64 {
        self.proof_latency_ms + self.verify_latency_ms
    }
}

impl Default for TransactionProfile {
    fn default() -> Self {
        Self {
            has_zk_proof: true,
            constraint_count: 20,
            anonymity_set_bits: 4, // 2^4 = 16 possible senders
            proof_age_secs: 0.0,
            proof_latency_ms: 800,
            verify_latency_ms: 27,
            zen_staked: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_classification() {
        let mut p = TransactionProfile::default();
        assert_eq!(p.kind(), TransactionKind::AnonymousVote);

        p.anonymity_set_bits = 0;
        assert_eq!(p.kind(), TransactionKind::ZkProof);

        p.anonymity_set_bits = 4;
        p.constraint_count = 10;
        assert_eq!(p.kind(), TransactionKind::ConfidentialTransfer);

        p.has_zk_proof = false;
        assert_eq!(p.kind(), TransactionKind::Transparent);
    }

    #[test]
    fn test_base_confidence_ordering() {
        assert!(TransactionKind::ZkProof.base_confidence() > TransactionKind::Transparent.base_confidence());
        assert!(TransactionKind::ConfidentialTransfer.base_confidence() > TransactionKind::Transparent.base_confidence());
    }
}
