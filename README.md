# ZenKinetic — Thermodynamic Privacy Gate for Horizen Base L3

> *Privacy is not free — it costs negentropy. ZenKinetic makes privacy the
> aligned flow, and transparency the penalty.*
>
> **By [Orkid Labs](https://www.orkidlabs.com)** — privacy-first crypto engineering

A privacy-first transaction gating system for Horizen's Base L3 appchain,
powered by the [negentropy](https://github.com/jjcav84/negentropy) physics
engine. ZenKinetic adapts the thermodynamic gate concept from orkid's
[`OrkidKineticHook`](https://github.com/jjcav84/orkid/blob/main/contracts/OrkidKineticHook.sol)
— converting it from **MEV protection** to **privacy protection**.

[![License: MIT](https://img.shields.io/badge/License-MIT-a78bfa.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-a78bfa.svg)](https://www.rust-lang.org/)
[![Horizen](https://img.shields.io/badge/Horizen-Base%20L3-ff6b35.svg)](https://horizen.org)
[![negentropy](https://img.shields.io/badge/powered%20by-negentropy-a78bfa.svg)](https://github.com/jjcav84/negentropy)

---

## The Idea

In orkid, the kinetic hook uses physics scoring to dynamically adjust DEX
fees: transactions that "collapse entropy" (aligned flow) get 0% fee,
transactions that "create entropy" (MEV extraction) get 10% fee.

ZenKinetic applies the same thermodynamic principle to **privacy**:

| Flow Type | Description | Entropy Effect | Fee |
|-----------|-------------|----------------|-----|
| **Aligned** | ZK proofs, confidential transfers, anonymous votes | Collapses entropy (privacy preserved) | 0% |
| **Misaligned** | Transparent broadcasts that leak ordering info | Creates entropy (deanonymizing) | 10% |
| **Standard** | No ZK proof | Baseline | 1% |

The negentropy extracted by a privacy-preserving transaction quantifies its
alignment. More negentropy = more privacy = lower fees.

## The Phoenix Cycle on Horizen

```text
┌──────────┐    ┌──────────┐    ┌──────────────┐    ┌──────────┐
│  ENTROPY │───▶│   BURN   │───▶│  EXTRACTION  │───▶│ REBIRTH  │
│ private  │    │ ZK proof │    │ negentropy   │    │confiden- │
│ tx data  │    │ compute  │    │ scored       │    │ tial tx  │
└──────────┘    └──────────┘    └──────────────┘    └──────────┘
```

1. **Entropy** — Private transaction data is high-entropy chaos
2. **Burn** — ZK proof generation pays the Landauer compute cost
3. **Extraction** — Negentropy scored: `N = constraints × log₂(anonymity_set)`
4. **Rebirth** — Confidential transaction settles with verifiable privacy

## ZEN Token Utility

| Stake Tier | Min ZEN Staked | Fee Discount | Confidential Transfers | Anonymous Voting |
|-----------|---------------|-------------|----------------------|-----------------|
| None | 0 | 0% | — | — |
| Basic | 100 | 25% off | ✓ | — |
| Pro | 1,000 | 50% off | ✓ | ✓ |
| Max | 10,000 | 75% off | ✓ | ✓ |

ZEN staking grants:
- **Fee discounts** on all transactions
- **Access** to confidential transfer features
- **Voting rights** in anonymous governance (Pro+ tier)

## Architecture

```text
                    ┌─────────────────────┐
                    │   ZenKineticGate    │
                    │   (Solidity)        │
                    │   Horizen Base L3   │
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │  PrivacyValidator   │
                    │  (negentropy-powered)│
                    └─────────┬───────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
   ┌──────▼──────┐   ┌───────▼───────┐   ┌──────▼──────┐
   │  RouteEnergy │   │  Negentropy   │   │  Committor  │
   │  (energy)    │   │  (bits)       │   │  (probability)│
   └──────────────┘   └───────────────┘   └─────────────┘
          │
   ┌──────▼──────────────────────────────────────┐
   │           negentropy crate                   │
   │   https://github.com/jjcav84/negentropy      │
   └─────────────────────────────────────────────┘
```

### Components

| Component | Language | Description |
|-----------|----------|-------------|
| `ZenKineticGate.sol` | Solidity | On-chain privacy gate with dynamic fees and ZEN staking |
| `zenkinetic` (Rust lib) | Rust | Off-chain privacy scoring via negentropy, CLI tool |
| `IPrivacyValidator` | Solidity | Interface for negentropy-powered scoring oracle |

## Quick Start

### Rust library

```rust
use zenkinetic::{PrivacyGate, TransactionProfile};

let profile = TransactionProfile {
    has_zk_proof: true,
    constraint_count: 20,
    anonymity_set_bits: 4,  // 2^4 = 16 possible senders
    proof_age_secs: 60.0,
    proof_latency_ms: 800,
    verify_latency_ms: 27,
    zen_staked: 1000.0,     // Pro tier
};

let gate = PrivacyGate::evaluate(&profile);
println!("Decision:    {:?}", gate.decision);      // Allow
println!("Alignment:   {:.1}%", gate.alignment * 100.0);
println!("Fee:         {} bps", gate.discounted_fee_bps);
println!("Negentropy:  {:.1} bits", gate.negentropy_bits);
```

### CLI

```bash
cargo run --bin zenkinetic -- gate --constraints 20 --anonymity 4 --age 60 --latency 800 --stake 1000
cargo run --bin zenkinetic -- theory
```

Example output:

```json
{
  "decision": "Allow",
  "alignment": "85.2%",
  "energy": 852.3,
  "negentropy_bits": 80.0,
  "committor": 0.95,
  "fee_bps": 0,
  "discounted_fee_bps": 0,
  "kind": "anonymous-vote",
  "stake_tier": "pro",
  "formula": "energy = confidence × √(depth × timing) × latency_decay × (1 − cost)",
  "engine": "negentropy (https://github.com/jjcav84/negentropy)"
}
```

## The Ecosystem

ZenKinetic is part of a family of privacy-preserving projects powered by
the negentropy physics engine:

```text
web3-defi/
├── negentropy/          ← thermodynamic engine (shared library)
├── orkid/               ← origin: FMD MEV detection (OrkidKineticHook)
├── zenkinetic/          ← THIS: privacy gate for Horizen Base L3
├── horizen-age/         ← age verification on Horizen (gated by ZenKinetic)
├── horizen-attest/      ← ZK attestations on Horizen (gated by ZenKinetic)
├── horizen-ballot/      ← anonymous voting on Horizen (gated by ZenKinetic)
├── zk-age/              ← age verification (original, zkVerify)
├── zk-attest/           ← ZK attestations (original, Hedera)
└── zk-ballot/           ← anonymous voting (original, on-chain)
```

| Project | Domain | negentropy Usage |
|---------|--------|-----------------|
| **orkid** | MEV detection | Route energy for arbitrage scoring (origin) |
| **zenkinetic** | Privacy gating | Route energy for privacy alignment scoring |
| **horizen-age** | Age verification (Horizen) | Negentropy for proof scoring + ZenKinetic gate |
| **horizen-attest** | Attestations (Horizen) | Negentropy for attestation scoring + ZenKinetic gate |
| **horizen-ballot** | Anonymous voting (Horizen) | Negentropy for ballot scoring + ZenKinetic gate |
| **zk-age** | Age verification (original) | Negentropy for proof quality scoring |
| **zk-attest** | Attestations (original) | Negentropy for attestation ranking |
| **zk-ballot** | Anonymous voting (original) | Negentropy for ballot quality scoring |

### Origin: orkid → ZenKinetic

ZenKinetic directly adapts orkid's `OrkidKineticHook` thermodynamic gate:

| orkid (MEV) | ZenKinetic (Privacy) |
|-------------|----------------------|
| `IPhysicsValidator.scoreRoute()` | `IPrivacyValidator.scorePrivacy()` |
| Aligned = collapses MEV entropy | Aligned = preserves privacy (negentropy) |
| Misaligned = MEV extraction | Misaligned = deanonymizing |
| 0% fee for aligned swaps | 0% fee for privacy-preserving txs |
| 10% fee for MEV | 10% fee for deanonymizing |
| Whitelisted bots → zero fee | ZEN stakers → fee discount |
| Euler vault profit sweep | ZEN staking for feature access |

## Tests

```bash
cargo test
```

13 tests covering: transaction classification, privacy gate decisions,
stale proof decay, stake tier discounts, anonymity set scaling, and
committor probability bounds. Clippy-clean.

## Grant: Thrive Horizen Genesis Program

ZenKinetic is built for the [Thrive Horizen Genesis Program](https://horizen.org),
targeting the **Infrastructure + DeFi** track:

- **Anonymous Infrastructure**: Privacy gate for confidential transactions,
  anonymous voting integration (zk-ballot patterns), confidential identity
  (zk-age patterns)
- **Confidential DeFi**: Negentropy-scored transaction fees, ZEN token
  staking for privacy access, thermodynamic fee gating

See [`docs/grant-application.md`](docs/grant-application.md) for the full
application aligned with the program's milestone structure.

## About

Built by [Orkid Labs](https://www.orkidlabs.com) — a privacy-first crypto
engineering lab building thermodynamic infrastructure for decentralized
systems. See our other work at [orkidlabs.com](https://www.orkidlabs.com).

## License

MIT — see [LICENSE](LICENSE).
