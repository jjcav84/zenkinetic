// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {ReentrancyGuardTransient} from "@openzeppelin/contracts/utils/ReentrancyGuardTransient.sol";
import {TransientSlot} from "@openzeppelin/contracts/utils/TransientSlot.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title IPrivacyValidator
/// @notice External interface for negentropy-based privacy scoring.
/// @dev Adapted from orkid's IPhysicsValidator — scores privacy alignment
///      instead of MEV alignment.
interface IPrivacyValidator {
    /// @return alignment Privacy alignment score (0 = deanonymizing, 10000 = fully private)
    /// @return negentropyBits Information extracted by the ZK proof (in bits)
    function scorePrivacy(
        bool hasZkProof,
        uint256 constraintCount,
        uint256 anonymitySetBits,
        uint256 proofAgeSecs,
        uint256 totalLatencyMs
    ) external view returns (uint256 alignment, uint256 negentropyBits);
}

/// @title IZenToken
/// @notice Minimal interface for the ZEN token on Horizen Base L3.
interface IZenToken is IERC20 {
    function stake(uint256 amount) external;
    function unstake(uint256 amount) external;
    function stakedBalanceOf(address account) external view returns (uint256);
}

/// @title ZenKineticGate
/// @notice Thermodynamic privacy gate for Horizen Base L3.
///
/// Adapted from orkid's `OrkidKineticHook` — converts the MEV protection
/// thermodynamic gate to a privacy protection gate:
///
/// - **Aligned** (privacy-preserving) transactions: 0% fee
/// - **Misaligned** (potentially deanonymizing) transactions: 10% fee
/// - **Standard** (no ZK proof) transactions: 1% fee
///
/// ZEN token staking grants fee discounts and access to privacy features.
///
/// @dev The gate evaluates transactions before execution and determines
///      the fee tier based on negentropy extraction (privacy preservation).
contract ZenKineticGate is Ownable, ReentrancyGuardTransient {
    using TransientSlot for *;
    using SafeERC20 for IZenToken;

    // ═══════════════════════════════════════════════════════════
    //                        EVENTS
    // ═══════════════════════════════════════════════════════════

    event TransactionScored(
        address indexed sender,
        uint256 alignment,
        uint256 negentropyBits,
        bool aligned,
        uint24 fee
    );
    event PrivacyValidatorUpdated(address indexed oldValidator, address indexed newValidator);
    event ZenTokenUpdated(address indexed oldToken, address indexed newToken);
    event FeeTierUpdated(string tier, uint24 oldFee, uint24 newFee);
    event StakeTierUpdated(string tier, uint256 minStake, uint256 discountBps);

    // ═══════════════════════════════════════════════════════════
    //                        STATE
    // ═══════════════════════════════════════════════════════════

    /// @notice External privacy validator contract (negentropy-powered)
    address public privacyValidator;

    /// @notice ZEN token contract on Horizen Base L3
    address public zenToken;

    // Fee tiers (in basis points)
    uint24 public constant ALIGNED_FEE = 0;      // 0% for privacy-preserving
    uint24 public constant MISALIGNED_FEE = 1000; // 10% for deanonymizing
    uint24 public constant STANDARD_FEE = 100;    // 1% for no-proof

    // Alignment threshold (in basis points, 10000 = 100%)
    uint256 public constant ALIGNMENT_THRESHOLD = 3000; // 30% minimum to be "aligned"

    // ZEN staking tiers (in token units, assuming 18 decimals)
    uint256 public constant BASIC_STAKE = 100e18;   // 100 ZEN
    uint256 public constant PRO_STAKE = 1_000e18;    // 1,000 ZEN
    uint256 public constant MAX_STAKE = 10_000e18;   // 10,000 ZEN

    // Fee discounts per tier (in basis points)
    uint256 public constant BASIC_DISCOUNT = 2500; // 25%
    uint256 public constant PRO_DISCOUNT = 5000;   // 50%
    uint256 public constant MAX_DISCOUNT = 7500;   // 75%

    // Proof recency half-life (seconds)
    uint256 public constant PROOF_HALF_LIFE = 3600; // 1 hour

    constructor(address _privacyValidator, address _zenToken)
        Ownable(msg.sender)
    {
        require(
            _privacyValidator != address(0) && _zenToken != address(0),
            "ZenKineticGate: invalid_ctor_args"
        );
        privacyValidator = _privacyValidator;
        zenToken = _zenToken;
    }

    // ═══════════════════════════════════════════════════════════
    //                    GATE EVALUATION
    // ═══════════════════════════════════════════════════════════

    /// @notice Evaluate a transaction through the privacy gate.
    /// @param hasZkProof Whether the transaction includes a ZK proof
    /// @param constraintCount Number of constraints in the ZK circuit
    /// @param anonymitySetBits Anonymity set size in bits (log2 of senders)
    /// @param proofAgeSecs Age of the proof in seconds
    /// @param totalLatencyMs Total proof gen + verify latency in ms
    /// @return fee The fee in basis points (after stake discount)
    /// @return aligned Whether the transaction is privacy-preserving
    function evaluateGate(
        bool hasZkProof,
        uint256 constraintCount,
        uint256 anonymitySetBits,
        uint256 proofAgeSecs,
        uint256 totalLatencyMs
    ) external view returns (uint24 fee, bool aligned) {
        // No ZK proof → standard fee
        if (!hasZkProof) {
            uint24 baseFee = STANDARD_FEE;
            return (applyStakeDiscount(baseFee), false);
        }

        // Score privacy via external validator (negentropy-powered)
        (uint256 alignment, uint256 negentropyBits) = _scorePrivacy(
            hasZkProof,
            constraintCount,
            anonymitySetBits,
            proofAgeSecs,
            totalLatencyMs
        );

        // Gate decision: aligned if alignment > threshold
        aligned = alignment >= ALIGNMENT_THRESHOLD;
        uint24 baseFee = aligned ? ALIGNED_FEE : MISALIGNED_FEE;

        return (applyStakeDiscount(baseFee), aligned);
    }

    /// @notice Internal privacy scoring — delegates to external validator
    /// or falls back to on-chain heuristic.
    function _scorePrivacy(
        bool hasZkProof,
        uint256 constraintCount,
        uint256 anonymitySetBits,
        uint256 proofAgeSecs,
        uint256 totalLatencyMs
    ) internal view returns (uint256 alignment, uint256 negentropyBits) {
        if (privacyValidator != address(0)) {
            try IPrivacyValidator(privacyValidator).scorePrivacy(
                hasZkProof,
                constraintCount,
                anonymitySetBits,
                proofAgeSecs,
                totalLatencyMs
            ) returns (uint256 a, uint256 n) {
                return (a, n);
            } catch {
                // Fall back to on-chain heuristic
                return _heuristicScore(constraintCount, anonymitySetBits, proofAgeSecs, totalLatencyMs);
            }
        }
        return _heuristicScore(constraintCount, anonymitySetBits, proofAgeSecs, totalLatencyMs);
    }

    /// @dev On-chain fallback scoring heuristic.
    /// Approximates the negentropy formula: N = constraints × log₂(anonymity_set)
    /// Alignment = normalized energy with timing and latency decay.
    function _heuristicScore(
        uint256 constraintCount,
        uint256 anonymitySetBits,
        uint256 proofAgeSecs,
        uint256 totalLatencyMs
    ) internal pure returns (uint256 alignment, uint256 negentropyBits) {
        // Negentropy: N = constraints × anonymity_set_bits
        // (since log₂(2^bits) = bits)
        // Cap to prevent overflow in downstream multiplication
        negentropyBits = constraintCount * anonymitySetBits;
        if (negentropyBits > 10000) negentropyBits = 10000;

        // Timing decay: exp(-age / half_life) approximated linearly
        // For on-chain efficiency, use: max(0, 10000 - (age * 10000 / half_life))
        uint256 timingFactor = proofAgeSecs >= PROOF_HALF_LIFE
            ? 0
            : (PROOF_HALF_LIFE - proofAgeSecs) * 10000 / PROOF_HALF_LIFE;

        // Latency decay: 1 / (1 + latency * 0.0001) approximated
        // 10000 / (1 + latency / 10000)
        uint256 latencyFactor = 10000 * 10000 / (10000 + totalLatencyMs);

        // Alignment: product of factors, normalized to 10000
        // All three factors are now capped at 10000, so max product = 10000^3 = 1e12
        // Divided by 10000^2 = 1e8, giving max result = 10000 — no overflow
        alignment = (negentropyBits * timingFactor * latencyFactor) / (10000 * 10000);

        // Cap at 10000 (100%)
        if (alignment > 10000) alignment = 10000;
    }

    // ═══════════════════════════════════════════════════════════
    //                    STAKE DISCOUNTS
    // ═══════════════════════════════════════════════════════════

    /// @notice Apply ZEN stake discount to a fee.
    /// @param baseFee The fee before discount (in bps)
    /// @return The discounted fee (in bps)
    function applyStakeDiscount(uint24 baseFee) public view returns (uint24) {
        if (baseFee == 0) return 0; // already free

        uint256 staked = IZenToken(zenToken).stakedBalanceOf(msg.sender);
        uint256 discountBps;

        if (staked >= MAX_STAKE) {
            discountBps = MAX_DISCOUNT;
        } else if (staked >= PRO_STAKE) {
            discountBps = PRO_DISCOUNT;
        } else if (staked >= BASIC_STAKE) {
            discountBps = BASIC_DISCOUNT;
        } else {
            discountBps = 0;
        }

        uint256 discounted = uint256(baseFee) * (10000 - discountBps) / 10000;
        return uint24(discounted);
    }

    /// @notice Get the stake tier for an address.
    function getStakeTier(address account) external view returns (string memory tier, uint256 staked) {
        staked = IZenToken(zenToken).stakedBalanceOf(account);
        if (staked >= MAX_STAKE) {
            tier = "max";
        } else if (staked >= PRO_STAKE) {
            tier = "pro";
        } else if (staked >= BASIC_STAKE) {
            tier = "basic";
        } else {
            tier = "none";
        }
    }

    // ═══════════════════════════════════════════════════════════
    //                    ADMIN
    // ═══════════════════════════════════════════════════════════

    function setPrivacyValidator(address newValidator) external onlyOwner {
        require(newValidator != address(0), "ZenKineticGate: validator_zero");
        address old = privacyValidator;
        privacyValidator = newValidator;
        emit PrivacyValidatorUpdated(old, newValidator);
    }

    function setZenToken(address newToken) external onlyOwner {
        require(newToken != address(0), "ZenKineticGate: token_zero");
        address old = zenToken;
        zenToken = newToken;
        emit ZenTokenUpdated(old, newToken);
    }
}
