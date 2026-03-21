
use crate::{RiskVector, ResidualState, CorridorBand, StepDecision, lyapunov_residual, safestep};

pub trait SafeController<State, Act> {
    fn propose_step(&self, state: &State, prev_residual: &ResidualState)
        -> (Act, RiskVector);
}

pub fn apply_with_ecosafety<State, Act, C: SafeController<State, Act>>(
    controller: &C,
    state: &State,
    prev_residual: &ResidualState,
    bands: &[CorridorBand],
    eps: f32,
    apply_actuation: &mut dyn FnMut(&Act),
) -> (ResidualState, StepDecision, RiskVector) {
    let (act, rv) = controller.propose_step(state, prev_residual);
    let next_residual = lyapunov_residual(&rv, bands);
    let decision = safestep(prev_residual, &next_residual, &rv, bands, eps);

    match decision {
        StepDecision::Accept => apply_actuation(&act),
        StepDecision::Derate | StepDecision::Reject => { /* safe derate/stop */ }
    }

    (next_residual, decision, rv)
}
