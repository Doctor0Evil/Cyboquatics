//! Type-level "no action without a risk estimate" contracts for
//! industrial Cyboquatic controllers.

use crate::{IndustrialRiskVector, NodeState, CommandEnvelope, StepVerdict};
use ecosafety_grammar::{ResidualState, CorridorBands, corridor_present, safestep};

/// Non-actuating ecosafety kernel: computes risk, V(t+1),
/// and enforces V(t+1) <= V(t) and corridor bands.
///
/// This trait is implemented once per node/plant type and then
/// reused across all controllers.
pub trait SafeStepKernel {
    /// Compute risk vector and Lyapunov verdict for a proposed command.
    fn evaluate_step(
        &self,
        state: &NodeState,
        corridors: &CorridorBands,
        residual: &ResidualState,
        proposed: &CommandEnvelope,
    ) -> (IndustrialRiskVector, StepVerdict);
}

/// Industrial controller: can only actuate via SafeStepKernel.
///
/// The compiler enforces that no implementation can bypass the
/// ecosafety kernel, because it never sees a hardware handle.
pub trait IndustrialSafeController {
    /// Stateless proposal: given node state, propose a command and
    /// its accompanying risk vector (must be returned).
    fn propose_step(
        &self,
        state: &NodeState,
    ) -> (CommandEnvelope, IndustrialRiskVector);

    /// Hook called by the gateway once a StepVerdict has been
    /// accepted and translated to a physical actuation. The
    /// controller itself never talks to actuators.
    fn on_step_applied(
        &mut self,
        state: &NodeState,
        verdict: &StepVerdict,
    );
}
