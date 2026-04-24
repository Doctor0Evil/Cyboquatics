// crates/cyboquatic-ecosafety-core/src/controller.rs

use crate::spine::{CorridorBands, CorridorDecision, KerWindow, Residual, RiskVector, compute_residual, safestep};

/// Minimal state view required for ecosafety decisions.
/// Real implementations wrap qpudatashard rows. [file:21][file:23]
pub trait NodeState {
    fn current_risks(&self) -> RiskVector;
}

/// Abstract actuation command for industrial Cyboquatic nodes
/// (pumps, turbines, tray lines, filters, etc.).
pub trait Actuation {
    fn is_noop(&self) -> bool;
}

/// Controller proposal carrying both an actuation and a RiskVector.
pub struct Proposal<A: Actuation> {
    pub actuation: A,
    pub risk_vector_next: RiskVector,
    pub residual_prev: Residual,
    pub residual_next: Residual,
}

/// Controllers must implement this trait; without it,
/// code does not type‑check and cannot actuate. [file:23]
pub trait SafeController<A: Actuation, S: NodeState> {
    /// Propose a next step and its risk vector, given current state.
    fn propose(&self, state: &S) -> Proposal<A>;
}

/// Shared ecosafety gate applied before any hardware access.
pub fn evaluate_and_gate<A: Actuation>(
    proposal: &Proposal<A>,
    bands: &CorridorBands,
) -> CorridorDecision {
    // V_t and V_{t+1} are already carried in the proposal.
    safestep(
        &proposal.residual_prev,
        &proposal.residual_next,
        &proposal.risk_vector_next,
        bands,
    )
}

/// Helper for computing updated KER window.
/// In real deployments this is done over rolling logs. [file:23]
pub fn update_ker_window(
    window: &mut KerWindow,
    decision: CorridorDecision,
    rv_next: &RiskVector,
) {
    // K = fraction of steps where safestep returned Ok/Derate (Lyapunov‑safe). [file:23]
    let safe = matches!(decision, CorridorDecision::Ok | CorridorDecision::Derate);
    let alpha = 0.01; // EWMA smoothing.

    window.k_knowledge = (1.0 - alpha) * window.k_knowledge
        + alpha * if safe { 1.0 } else { 0.0 };

    let r_max = rv_next.max_coord();
    window.r_risk_of_harm = (1.0 - alpha) * window.r_risk_of_harm + alpha * r_max;
    window.normalize();
}

/// Example utility: compute residual from a state‑supplied RiskVector.
pub fn residual_from_state<S: NodeState>(
    state: &S,
    weights: &crate::spine::LyapunovWeights,
) -> Residual {
    let rv = state.current_risks();
    compute_residual(&rv, weights)
}
