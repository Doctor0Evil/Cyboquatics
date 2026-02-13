use crate::types::{Residual, RiskCoord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CorridorDecision {
    Allow,
    Derate,          // allowed only under reduced operating envelope
    Forbid,          // block actuation; diagnostics/logging only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafestepParams {
    pub lyapunov_tol: f64,   // numerical tolerance for V_{t+1} <= V_t
    pub derate_margin: f64,  // margin below hard threshold for derating
}

impl Default for SafestepParams {
    fn default() -> Self {
        Self {
            lyapunov_tol: 1e-9,
            derate_margin: 0.8, // e.g., derate when rx in [0.8, 1)
        }
    }
}

/// Compute Lyapunov residual V_t from coordinates, if not pre-filled.
pub fn compute_residual_v(coords: &[RiskCoord]) -> f64 {
    coords.iter().map(|c| c.weighted_harm()).sum()
}

/// Final safety gate.
/// - Enforces V_{t+1} <= V_t (within tolerance).
/// - Enforces rx < 1 for all risk coordinates.
/// - Returns Allow, Derate, or Forbid.
pub fn safestep(prev: &Residual, next: &Residual, params: &SafestepParams) -> CorridorDecision {
    let v_prev = prev.vt;
    let v_next = if next.vt.is_finite() { next.vt } else { compute_residual_v(&next.coords) };

    // Lyapunov non-increase check.
    if v_next > v_prev + params.lyapunov_tol {
        return CorridorDecision::Forbid;
    }

    // Hard corridor check and derate band.
    let mut any_derate = false;
    for coord in &next.coords {
        if coord.violates_hard() {
            return CorridorDecision::Forbid;
        }
        if coord.rx() >= params.derate_margin {
            any_derate = true;
        }
    }

    if any_derate {
        CorridorDecision::Derate
    } else {
        CorridorDecision::Allow
    }
}
