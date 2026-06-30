//! # ZenKinetic — Thermodynamic Privacy Gate for Horizen Base L3
//!
//! Adapts the [orkid `OrkidKineticHook`](https://github.com/jjcav84/orkid/blob/main/contracts/OrkidKineticHook.sol)
//! thermodynamic gate from **MEV protection** to **privacy protection** on
//! Horizen's privacy-first appchain.
//!
//! ## The Idea
//!
//! In orkid, the kinetic hook uses physics scoring to dynamically adjust DEX
//! fees: transactions that "collapse entropy" (aligned flow) get 0% fee,
//! transactions that "create entropy" (MEV extraction) get 10% fee.
//!
//! ZenKinetic applies the same thermodynamic principle to **privacy**:
//!
//! - **Aligned** (privacy-preserving) transactions — ZK proofs, confidential
//!   transfers, anonymous voting — *collapse entropy* by converting private
//!   chaos into verifiable order. These get **lower fees**.
//! - **Misaligned** (potentially deanonymizing) transactions — transparent
//!   broadcasts that leak ordering information — *create entropy* in the
//!   network's information state. These get **higher fees**.
//!
//! The negentropy extracted by a privacy-preserving transaction quantifies
//! its alignment. More negentropy = more privacy = lower fees.
//!
//! ## The Phoenix Cycle on Horizen
//!
//! ```text
//! ┌──────────┐    ┌──────────┐    ┌──────────────┐    ┌──────────┐
//! │  ENTROPY │───▶│   BURN   │───▶│  EXTRACTION  │───▶│ REBIRTH  │
//! │ private  │    │ ZK proof │    │ negentropy   │    │confiden- │
//! │ tx data  │    │ compute  │    │ scored       │    │ tial tx  │
//! └──────────┘    └──────────┘    └──────────────┘    └──────────┘
//! ```
//!
//! ## ZEN Token Utility
//!
//! - **Staking**: Stake ZEN to access privacy features (confidential transfer,
//!   anonymous voting, identity proofs)
//! - **Fee gating**: Transaction fees denominated in ZEN, discounted by
//!   negentropy score
//! - **Governance**: Anonymous voting on protocol parameters using zk-ballot
//!   patterns
//!
//! ## Quick Start
//!
//! ```rust
//! use zenkinetic::{PrivacyGate, TransactionProfile};
//!
//! let profile = TransactionProfile {
//!     has_zk_proof: true,
//!     constraint_count: 20,
//!     anonymity_set_bits: 16, // 2^4 = 16 possible senders
//!     proof_age_secs: 60.0,
//!     proof_latency_ms: 800,
//!     verify_latency_ms: 27,
//!     zen_staked: 1000.0,
//! };
//!
//! let gate = PrivacyGate::evaluate(&profile);
//! println!("Alignment: {:.1}%", gate.alignment * 100.0);
//! println!("Fee tier:  {} bps", gate.fee_bps);
//! println!("Negentropy: {:.1} bits", gate.negentropy_bits);
//! ```

pub mod gate;
pub mod profile;
pub mod staking;

pub use gate::{PrivacyGate, GateDecision};
pub use profile::{TransactionProfile, TransactionKind};
pub use staking::{ZenStake, StakeTier};

/// Re-export core negentropy types for convenience.
pub use negentropy::{Negentropy, RouteEnergy, Committor, Burn};
