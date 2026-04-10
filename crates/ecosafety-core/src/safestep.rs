// Filename: crates/ecosafety-core/src/safestep.rs

use serde::{Deserialize, Serialize};

use crate::residual::compute_residual;
use crate::types::{LyapunovWeights, Residual, RiskVector};

/// Decision produced by the ecosafety gate for a proposed step.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SafeDecision {
    Accept,
    Derate,
    Stop,
}

/// Safestep invariant with internal residual recomputation.
///
/// - Computes V_{t+1} from `rv_next` and `weights`.
/// - Enforces hard-band checks via `any_hard_breach`.
/// - Applies a tolerance `epsilon` to distinguish Accept / Derate / Stop.
pub fn safestep(
    prev: Residual,
    rv_next: &RiskVector,
    weights: &LyapunovWeights,
    epsilon: f64,
) -> (SafeDecision, Residual) {
    if rv_next.any_hard_breach() {
        let vt1 = compute_residual(rv_next, weights);
        return (SafeDecision::Stop, vt1);
    }

    let vt = prev.value;
    let vt1 = compute_residual(rv_next, weights).value;
    let eps = epsilon.max(0.0);

    if vt1 > vt + eps {
        (SafeDecision::Stop, Residual::new(vt1))
    } else if vt1 > vt - eps {
        (SafeDecision::Derate, Residual::new(vt1))
    } else {
        (SafeDecision::Accept, Residual::new(vt1))
    }
}

/// Safestep invariant when the caller has already computed `next` residual.
///
/// This variant:
/// - Uses the precomputed `next` residual.
/// - Still enforces hard-band checks on `rv_next`.
/// - Uses a fixed small tolerance `eps` for gating.
pub fn safestep_with_residuals(
    prev: Residual,
    next: Residual,
    rv_next: &RiskVector,
    _weights: &LyapunovWeights,
) -> SafeDecision {
    if rv_next.any_hard_breach() {
        return SafeDecision::Stop;
    }

    let vt = prev.value;
    let vt1 = next.value;
    let eps = 1e-3;

    if vt > eps && vt1 > vt + 1e-9 {
        return SafeDecision::Stop;
    }

    if vt <= eps && vt1 > vt + eps {
        SafeDecision::Derate
    } else {
        SafeDecision::Accept
    }
}
