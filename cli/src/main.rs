//! ZenKinetic CLI — score transaction privacy and gate fees on Horizen.
//!
//! Usage:
//!   zenkinetic gate --constraints 20 --anonymity 4 --age 60 --latency 800 --stake 1000
//!   zenkinetic theory

use clap::{Parser, Subcommand};
use serde_json::json;
use zenkinetic::{PrivacyGate, TransactionProfile};

#[derive(Parser)]
#[command(
    name = "zenkinetic",
    about = "Thermodynamic privacy gate for Horizen Base L3 — score transaction privacy via negentropy",
    long_about = "Adapts the orkid KineticHook thermodynamic gate from MEV protection to privacy protection.\n\nPrivacy-preserving transactions (ZK proofs) get lower fees.\nDeanonymizing transactions get higher fees.\n\nPowered by the negentropy physics engine: https://github.com/jjcav84/negentropy"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate a transaction through the privacy gate
    Gate {
        #[arg(long, help = "Number of ZK circuit constraints", default_value = "20")]
        constraints: u64,
        #[arg(long, help = "Anonymity set size in bits (log2 of possible senders)", default_value = "4")]
        anonymity: u64,
        #[arg(long, help = "Proof age in seconds", default_value = "0")]
        age: f64,
        #[arg(long, help = "Proof generation latency in ms", default_value = "800")]
        latency: u64,
        #[arg(long, help = "ZEN tokens staked (for fee discounts)", default_value = "0")]
        stake: f64,
        #[arg(long, help = "Whether tx has a ZK proof", default_value_t = true)]
        proof: bool,
    },
    /// Show the thermodynamic privacy gate theory
    Theory,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Gate { constraints, anonymity, age, latency, stake, proof } => {
            let profile = TransactionProfile {
                has_zk_proof: proof,
                constraint_count: constraints,
                anonymity_set_bits: anonymity,
                proof_age_secs: age,
                proof_latency_ms: latency,
                verify_latency_ms: 27,
                zen_staked: stake,
            };
            let gate = PrivacyGate::evaluate(&profile);
            let result = json!({
                "decision": format!("{:?}", gate.decision),
                "alignment": format!("{:.1}%", gate.alignment * 100.0),
                "energy": gate.energy,
                "negentropy_bits": gate.negentropy_bits,
                "committor": gate.committor,
                "fee_bps": gate.fee_bps,
                "discounted_fee_bps": gate.discounted_fee_bps,
                "kind": gate.kind.label(),
                "stake_tier": gate.stake_tier,
                "timing_factor": gate.timing_factor,
                "latency_decay": gate.latency_decay,
                "formula": "energy = confidence × √(depth × timing) × latency_decay × (1 − cost)",
                "origin": "orkid OrkidKineticHook → ZenKinetic privacy gate",
                "engine": "negentropy (https://github.com/jjcav84/negentropy)",
            };
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Commands::Theory => {
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║         ZENKINETIC — THERMODYNAMIC PRIVACY GATE          ║");
            println!("╠══════════════════════════════════════════════════════════╣");
            println!("║                                                          ║");
            println!("║  Adapted from orkid's OrkidKineticHook:                  ║");
            println!("║  MEV protection → Privacy protection                     ║");
            println!("║                                                          ║");
            println!("║  ┌────────┐   ┌────────┐   ┌────────────┐   ┌────────┐ ║");
            println!("║  │ENTROPY │──▶│  BURN  │──▶│ EXTRACTION │──▶│REBIRTH │ ║");
            println!("║  │private │   │ZK proof│   │negentropy  │   │confiden│ ║");
            println!("║  │tx data │   │compute │   │scored      │   │tial tx │ ║");
            println!("║  └────────┘   └────────┘   └────────────┘   └────────┘ ║");
            println!("║                                                          ║");
            println!("║  ALIGNED (privacy-preserving) → 0% fee                   ║");
            println!("║  MISALIGNED (deanonymizing)   → 10% fee                  ║");
            println!("║  STANDARD (no ZK proof)       → 1% fee                   ║");
            println!("║                                                          ║");
            println!("║  ZEN staking discounts:                                  ║");
            println!("║    100 ZEN  → 25% off  (Basic tier)                      ║");
            println!("║    1K ZEN   → 50% off  (Pro tier)                        ║");
            println!("║    10K ZEN  → 75% off  (Max tier)                        ║");
            println!("║                                                          ║");
            println!("║  Engine: negentropy                                       ║");
            println!("║  Origin: orkid OrkidKineticHook                           ║");
            println!("║  Target: Horizen Base L3                                  ║");
            println!("║                                                          ║");
            println!("╚══════════════════════════════════════════════════════════╝");
        }
    }
}
