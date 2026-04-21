// Filename: crates/ecosafety-core/src/safestep.rs

use serde::{Deserialize, Serialize};

use crate::residual::compute_residual;
use crate::riskvector::{LyapunovWeights, Residual, RiskVector};

/// Decision produced by the ecosafety gate for a proposed step.
///
/// Semantics:
/// - `Accept`: proposed step clearly reduces Lyapunov residual V_t.
/// - `Derate`: residual is flat or within a small tolerance band; caller should
///   apply a lower-power / safer actuation or hold state.
/// - `Stop`: residual increases or any hard-band breach is detected; caller must
///   block actuation and treat this as a safety incident.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SafeDecision {
    Accept,
    Derate,
    Stop,
}

/// Core safestep invariant with internal residual recomputation.
///
/// This function:
/// - Recomputes `V_{t+1}` from `rv_next` and `weights`.
/// - Enforces hard-band checks via `RiskVector::any_hard_breach`.
/// - Applies a caller-provided `epsilon` tolerance to distinguish
///   `Accept` / `Derate` / `Stop`.
///
/// Contract:
/// - If any coordinate is at or beyond its hard band (`any_hard_breach == true`),
///   the decision is always `Stop`.
/// - Outside the hard band, `V_{t+1}` must not grow more than `epsilon` above
///   `V_t`. Larger growth returns `Stop`, near-flat returns `Derate`, and
///   strictly decreasing returns `Accept`.
pub fn safestep(
    prev: Residual,
    rv_next: &RiskVector,
    weights: &LyapunovWeights,
    epsilon: f64,
) -> (SafeDecision, Residual) {
    // Enforce hard-band safety first: no actuation allowed if any coordinate
    // is at or beyond its hard corridor.
    if rv_next.any_hard_breach() {
        let vt1 = compute_residual(rv_next, weights);
        return (SafeDecision::Stop, vt1);
    }

    let vt = prev.value;
    let vt1_residual = compute_residual(rv_next, weights);
    let vt1 = vt1_residual.value;

    // Ensure epsilon is non-negative to avoid inverting semantics.
    let eps = epsilon.max(0.0);

    if vt1 > vt + eps {
        (SafeDecision::Stop, vt1_residual)
    } else if vt1 > vt - eps {
        (SafeDecision::Derate, vt1_residual)
    } else {
        (SafeDecision::Accept, vt1_residual)
    }
}

/// Safestep variant for callers that have already computed `next` residual.
///
/// This variant:
/// - Uses the precomputed `next` residual (`V_{t+1}`).
/// - Still enforces hard-band checks on `rv_next`.
/// - Uses a fixed small tolerance `eps` for gating.
///
/// It is appropriate when:
/// - A domain crate has already computed a residual (e.g. for logging or
///   subsystem-local checks) and wants the global ecosafety decision without
///   recomputing `V_{t+1}`.
/// - The same `LyapunovWeights` that produced `next` are in scope at the call
///   site (the `_weights` parameter is carried for API symmetry / future use).
pub fn safestep_with_residuals(
    prev: Residual,
    next: Residual,
    rv_next: &RiskVector,
    _weights: &LyapunovWeights,
) -> SafeDecision {
    // Hard-band breach is always a stop, regardless of residual trend.
    if rv_next.any_hard_breach() {
        return SafeDecision::Stop;
    }

    let vt = prev.value;
    let vt1 = next.value;
    let eps = 1e-3;

    // If we are already outside the small-safe interior (vt > eps),
    // enforce a very strict non-increase condition (numerical slack only).
    if vt > eps && vt1 > vt + 1e-9 {
        return SafeDecision::Stop;
    }

    // Inside the small-safe interior (vt <= eps), allow a bit more slack:
    // treat modest increases as a derate request, not an immediate stop.
    if vt <= eps && vt1 > vt + eps {
        SafeDecision::Derate
    } else {
        SafeDecision::Accept
    }
}
