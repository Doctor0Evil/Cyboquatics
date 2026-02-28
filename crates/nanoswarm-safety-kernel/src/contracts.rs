#![no_std]

use crate::types::Residual;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CorridorDecision {
    Ok,
    Derate,
    Stop,
}

/// Lyapunov / corridor gate used by all ecosafety kernels.
/// - Stop: any r_j >= 1.0
/// - Derate: no hard breach, but V_{t+1} > V_t outside safe interior
/// - Ok: otherwise
pub fn safestep(prev: &Residual, next: &Residual) -> CorridorDecision {
    let any_hard = next
        .coords
        .iter()
        .any(|c| c.r >= 1.0);

    if any_hard {
        return CorridorDecision::Stop;
    }

    if next.vt > prev.vt {
        return CorridorDecision::Derate;
    }

    CorridorDecision::Ok
}
