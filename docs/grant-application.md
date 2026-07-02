# ZenKinetic — Thrive Horizen Genesis Program (#38) Application

**By [Orkid Labs](https://www.orkidlabs.com)** — privacy-first crypto engineering

## Project Summary

**ZenKinetic** is a thermodynamic privacy gate for Horizen Base L3 that
uses negentropy scoring to dynamically adjust transaction fees based on
privacy preservation. Privacy-preserving transactions (ZK proofs,
confidential transfers, anonymous votes) receive zero or discounted fees;
potentially deanonymizing transactions pay a premium. ZEN token staking
grants fee discounts and access to privacy features.

The system adapts the thermodynamic gate concept from orkid's
`OrkidKineticHook` — a production MEV protection hook on Uniswap V4 —
converting it from MEV alignment scoring to privacy alignment scoring.

**Track**: Anonymous Infrastructure + Confidential DeFi
**Funding request**: $75K in ZEN tokens
**Timeline**: 150 days

---

## Technical Architecture

### Overview

ZenKinetic consists of three layers:

1. **On-chain gate** (`ZenKineticGate.sol`) — Solidity contract deployed
   on Horizen Base L3 that evaluates transactions and determines fees
2. **Privacy scoring oracle** (`IPrivacyValidator`) — External contract
   or off-chain service powered by the negentropy Rust engine
3. **Rust library + CLI** (`zenkinetic`) — Off-chain scoring, simulation,
   and integration tooling

### Privacy Technology

ZenKinetic uses **zero-knowledge proofs** as the primary privacy
mechanism, scored by the **negentropy thermodynamic engine**:

- **ZK-SNARKs** (Halo2 / Groth16): Proof generation for confidential
  transactions, anonymous voting, and identity verification
- **Negentropy scoring**: Each ZK proof extracts negentropy (information)
  from private data, quantified as `N = constraints × log₂(anonymity_set)`.
  This score determines the transaction's privacy alignment.
- **Thermodynamic gate**: Dynamic fees based on the Phoenix cycle:
  - **Entropy** (private data) → **Burn** (ZK proof compute) →
    **Extraction** (negentropy scored) → **Rebirth** (confidential tx)

### The Formula

Adapted from orkid's FMD route energy:

```
energy = confidence × √(depth_ratio × timing_factor) × latency_decay × (1 − cost_penalty)
```

Where for Horizen privacy:
- `confidence` = transaction kind base weight (ZK proof = 100, confidential = 90, vote = 85, transparent = 10)
- `depth_ratio` = `confidence × anonymity_set_bits / 4` (more possible senders = more privacy)
- `timing_factor` = `exp(-proof_age / 3600)` (recency decay, 1-hour half-life)
- `latency_decay` = `1 / (1 + total_latency_ms × 0.0001)` (proof speed)
- `cost_penalty` = reduced by ZEN staking (more stake = less penalty)

### Smart Contract Architecture

```
ZenKineticGate.sol
├── evaluateGate()           → returns fee + alignment decision
├── applyStakeDiscount()     → ZEN staking fee discounts
├── _scorePrivacy()          → delegates to IPrivacyValidator
├── _heuristicScore()        → on-chain fallback scoring
└── admin functions          → validator/token management
```

### ZEN Token Utility

| Stake Tier | Min ZEN | Fee Discount | Features |
|-----------|---------|-------------|----------|
| Basic | 100 ZEN | 25% off | Confidential transfers |
| Pro | 1,000 ZEN | 50% off | + Anonymous voting |
| Max | 10,000 ZEN | 75% off | + Governance weight |

ZEN is used for:
1. **Staking** — Lock ZEN to access privacy features and fee discounts
2. **Fee payment** — Transaction fees denominated in ZEN
3. **Governance** — Anonymous voting on gate parameters (thresholds, fees)

### Integration with Existing Ecosystem

ZenKinetic is part of a proven ecosystem of privacy-preserving ZK projects:

- [negentropy](https://github.com/jjcav84/negentropy) — The shared physics
  engine (47 tests, production-ready)
- [orkid](https://github.com/jjcav84/orkid) — Origin of the thermodynamic
  gate concept (195+ live fills on Base/Ethereum). **Private repo — access
  available for Thrive Protocol reviewers and other appropriate cases on
  request.** Theoretical foundation published as preprint: ["Negative EV
  per Unit Time as Blockchain Inefficiency"](https://www.researchgate.net/publication/399474539_Negative_EV_per_Unit_Time_as_Blockchain_Inefficiency)
  — [Jacob Cavazos, ResearchGate](https://www.researchgate.net/profile/Jacob-Cavazos).
- [zk-age](https://github.com/jjcav84/zk-age) — Privacy-preserving age
  verification (negentropy-powered)
- [zk-attest](https://github.com/jjcav84/zk-attest) — ZK attestations on
  Hedera (negentropy-powered)
- [zk-ballot](https://github.com/jjcav84/zk-ballot) — Anonymous on-chain
  voting with Halo2 (negentropy-powered)

All sibling projects depend on `negentropy` as a shared library, creating
a unified privacy infrastructure stack.

---

## Privacy Excellence

### Cryptographic Implementation

- **ZK proofs**: Halo2-based proof system for transaction privacy
- **Negentropy scoring**: Information-theoretic privacy quantification
  (Shannon entropy, KL divergence, Brillouin negentropy principle)
- **Committor function**: Transition Path Sampling for privacy validity
  prediction
- **Thermodynamic gate**: Dynamic fee adjustment based on negentropy
  extraction

### Privacy Preservation Methodology

1. **Proof generation**: User generates ZK proof proving transaction
   validity without revealing sender, amount, or recipient
2. **Negentropy scoring**: Gate scores the proof's privacy preservation:
   `N = constraints × log₂(anonymity_set)` bits of negentropy extracted
3. **Fee determination**: Aligned proofs (high negentropy) → 0% fee;
   misaligned (low negentropy) → 10% fee
4. **Settlement**: Transaction settles confidentially with verifiable
   privacy guarantees

---

## Team Background

**Jacob Cavazos** — Solo developer with a proven track record in ZK
privacy infrastructure:

- Built and deployed orkid (MEV protection with 195+ live fills on
  Base/Ethereum, audited TVMExecutor smart contract)
- Created the negentropy physics engine (generalized from orkid's FMD
  model, 47 tests, used across 4 projects)
- Published the theoretical foundation as a preprint:
  ["Negative EV per Unit Time as Blockchain Inefficiency"](https://www.researchgate.net/publication/399474539_Negative_EV_per_Unit_Time_as_Blockchain_Inefficiency)
  — [ResearchGate profile](https://www.researchgate.net/profile/Jacob-Cavazos)
- Built zk-age, zk-attest, zk-ballot — three privacy-preserving ZK
  applications, all powered by negentropy
- Deep expertise in: Rust, Solidity, Halo2, ZK circuits, thermodynamic
  modeling, DeFi MEV protection

---

## Milestone Plan (150 days)

### Application (10% unlocked)

- ✅ Technical architecture documented (this document)
- ✅ ZK proof integration designed (negentropy scoring model)
- ✅ Privacy UX design (fee gating + staking tiers)
- ✅ ZEN token utility defined (staking, fees, governance)
- ✅ Codebase initialized with working Rust library + Solidity contract
- ✅ 13 tests passing, clippy-clean

### Milestone 1 (20% unlocked) — 45 days

**Smart contract deployed on Horizen testnet with privacy features functional**

- [ ] Deploy `ZenKineticGate.sol` on Horizen Base L3 testnet
- [ ] Implement `IPrivacyValidator` oracle (negentropy-powered scoring)
- [ ] Integrate ZEN token staking contract
- [ ] Core privacy mechanisms: ZK proof verification, negentropy scoring,
  dynamic fee gating
- [ ] Security review of smart contracts
- [ ] Technical documentation published with privacy proofs
- [ ] Beta testing with 10+ users providing feedback

### Milestone 2 (30% unlocked) — 90 days

**Mainnet deployment with full privacy feature set**

- [ ] Deploy `ZenKineticGate.sol` on Horizen Base L3 mainnet
- [ ] Full privacy feature set: confidential transfers, anonymous voting,
  identity verification
- [ ] Privacy compliance documentation
- [ ] Integration with Horizen ecosystem tools
- [ ] Early traction (target: 250+ unique wallets utilizing privacy features)
- [ ] ZEN staking live with 3 tiers (Basic, Pro, Max)

### Milestone 3 (40% unlocked) — 150 days

**Scale metrics**

- [ ] 500+ unique wallets utilizing privacy features
- [ ] 20,000+ privacy-preserving transactions
- [ ] Anonymous governance voting live (zk-ballot integration)
- [ ] Cross-ecosystem integration: zk-age identity proofs accepted as
  gate inputs
- [ ] Open-source SDK for third-party privacy app development on Horizen

---

## Why ZenKinetic?

1. **Proven concept**: The thermodynamic gate is battle-tested in orkid's
   MEV protection system (195+ live fills). ZenKinetic adapts it to privacy.
2. **Shared infrastructure**: Built on negentropy, used across 4 production
   ZK projects. Not a one-off — it's part of a cohesive privacy stack.
3. **Real ZEN utility**: Staking for privacy access, fee payment, and
   governance — not just speculation.
4. **Composable**: Integrates with zk-age (identity), zk-ballot (voting),
  and zk-attest (attestations) — all already negentropy-powered.
5. **Privacy-first by design**: The thermodynamic gate makes privacy the
   economically aligned choice, not just an option.

---

## References

- [negentropy](https://github.com/jjcav84/negentropy) — The physics of
  information extraction
- orkid OrkidKineticHook.sol — Origin of the thermodynamic gate (private repo)
- [orkid blog: Blockchain Thermodynamics](https://www.orkidlabs.com/blog/blockchain-thermodynamics-negentropy-mev-physics/)
- [orkid blog: Negentropy = Information](https://www.orkidlabs.com/blog/negentropy-information-generalized-framework/)
- Cavazos, J. — ["Negative EV per Unit Time as Blockchain Inefficiency"](https://www.researchgate.net/publication/399474539_Negative_EV_per_Unit_Time_as_Blockchain_Inefficiency) (preprint)
- Shannon (1948) — Information entropy
- Landauer (1961) — Thermodynamic cost of information erasure
- Brillouin (1953) — Negentropy principle
- Bolhuis et al. (2002) — Transition Path Sampling
