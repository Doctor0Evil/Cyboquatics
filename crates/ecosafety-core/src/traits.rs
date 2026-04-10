use crate::safestep::{safestep, SafeDecision};
use crate::types::{LyapunovWeights, Residual, RiskVector};

/// Trait implemented by all controllers subject to the ecosafety spine.
///
/// Implementors must propose a step by returning both an actuation object and
/// the corresponding RiskVector and candidate Residual V_{t+1}.
pub trait SafeController {
    type State;
    type Actuation;

    fn propose_step(
        &mut self,
        state: &Self::State,
        prev_residual: Residual,
        weights: &LyapunovWeights,
    ) -> (Self::Actuation, RiskVector);
}

/// Wrapper that applies the safestep invariant before actuating.
pub fn route_and_actuate<C>(
    controller: &mut C,
    state: &C::State,
    prev_residual: Residual,
    weights: &LyapunovWeights,
    epsilon: f64,
    apply_actuation: impl Fn(&C::Actuation),
) -> (SafeDecision, Residual)
where
    C: SafeController,
{
    let (act, rv_next) = controller.propose_step(state, prev_residual, weights);
    let (decision, next_resid) = safestep(prev_residual, &rv_next, weights, epsilon);

    match decision {
        SafeDecision::Accept => {
            apply_actuation(&act);
        }
        SafeDecision::Derate => {
            // Domain-specific derating can be applied by the caller based on rv_next.
        }
        SafeDecision::Stop => {
            // No actuation is applied.
        }
    }

    (decision, next_resid)
}
