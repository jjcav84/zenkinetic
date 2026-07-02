//! ZEN token staking — fee tier discounts for privacy feature access.
//!
//! Staking ZEN grants access to privacy features and fee discounts.
//! Higher stake = lower fees, analogous to orkid's whitelisted bot
//! zero-fee flash loans.

use serde::{Deserialize, Serialize};

/// Staking tier — determines fee discount level.
///
/// | Tier | Min ZEN Staked | Fee Discount |
/// |------|---------------|-------------|
/// | None | 0             | 0%          |
/// | Basic| 100           | 25%         |
/// | Pro  | 1,000         | 50%         |
/// | Max  | 10,000        | 75%         |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StakeTier {
    None,
    Basic,
    Pro,
    Max,
}

impl StakeTier {
    /// Classify a staked amount into a tier.
    pub fn from_staked(zen_staked: f64) -> Self {
        if zen_staked >= 10_000.0 {
            Self::Max
        } else if zen_staked >= 1_000.0 {
            Self::Pro
        } else if zen_staked >= 100.0 {
            Self::Basic
        } else {
            Self::None
        }
    }

    /// Fee discount multiplier (0.0 = no discount, 0.75 = 75% off).
    pub fn fee_discount(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Basic => 0.25,
            Self::Pro => 0.50,
            Self::Max => 0.75,
        }
    }

    /// Whether this tier grants access to confidential transfers.
    pub fn grants_confidential_transfer(&self) -> bool {
        matches!(self, Self::Basic | Self::Pro | Self::Max)
    }

    /// Whether this tier grants access to anonymous voting.
    pub fn grants_anonymous_voting(&self) -> bool {
        matches!(self, Self::Pro | Self::Max)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Basic => "basic",
            Self::Pro => "pro",
            Self::Max => "max",
        }
    }
}

/// ZEN stake position.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ZenStake {
    /// Amount of ZEN staked
    pub amount: f64,
    /// Staking tier derived from amount
    pub tier: StakeTier,
}

impl ZenStake {
    /// Create a stake position from a staked amount.
    pub fn new(amount: f64) -> Self {
        assert!(amount.is_finite() && amount >= 0.0, "stake amount must be non-negative");
        Self {
            amount,
            tier: StakeTier::from_staked(amount),
        }
    }

    /// Fee discount for this stake (0.0 to 0.75).
    pub fn fee_discount(&self) -> f64 {
        self.tier.fee_discount()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_classification() {
        assert_eq!(StakeTier::from_staked(0.0), StakeTier::None);
        assert_eq!(StakeTier::from_staked(50.0), StakeTier::None);
        assert_eq!(StakeTier::from_staked(100.0), StakeTier::Basic);
        assert_eq!(StakeTier::from_staked(500.0), StakeTier::Basic);
        assert_eq!(StakeTier::from_staked(1000.0), StakeTier::Pro);
        assert_eq!(StakeTier::from_staked(5000.0), StakeTier::Pro);
        assert_eq!(StakeTier::from_staked(10000.0), StakeTier::Max);
    }

    #[test]
    fn test_fee_discounts() {
        assert_eq!(StakeTier::None.fee_discount(), 0.0);
        assert_eq!(StakeTier::Basic.fee_discount(), 0.25);
        assert_eq!(StakeTier::Pro.fee_discount(), 0.50);
        assert_eq!(StakeTier::Max.fee_discount(), 0.75);
    }

    #[test]
    fn test_feature_access() {
        assert!(!StakeTier::None.grants_confidential_transfer());
        assert!(StakeTier::Basic.grants_confidential_transfer());
        assert!(!StakeTier::Basic.grants_anonymous_voting());
        assert!(StakeTier::Pro.grants_anonymous_voting());
    }
}
