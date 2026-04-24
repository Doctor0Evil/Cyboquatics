//! Type-level "no action without a risk estimate" contracts for
//! industrial Cyboquatic controllers.

use crate::{IndustrialRiskVector, NodeState, CommandEnvelope, StepVerdict};
use ecosafety_core::{Residual, CorridorDecision};

/// Abstract view of node state needed for ecosafety decisions.
pub trait NodeStateTrait {
    fn node_class(&self) -> crate::NodeClass;
    fn medium(&self) -> crate::MediumClass;
    fn current_risks(&self) -> IndustrialRiskVector;
}

/// Domain actuation command envelope (no risk embedded).
pub trait CommandEnvelopeTrait {
    fn is_noop(&self) -> bool;
}

impl CommandEnvelopeTrait for CommandEnvelope {
    fn is_noop(&self) -> bool {
        self.is_noop()
    }
}

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
        residual: &Residual,
        proposed: &CommandEnvelope,
    ) -> (IndustrialRiskVector, StepVerdict);
}

/// Industrial controller: can only actuate via SafeStepKernel.
///
/// The compiler enforces that no implementation can bypass the
/// ecosafety kernel, because it never sees a hardware handle.
pub trait IndustrialSafeController<S: NodeStateTrait, C: CommandEnvelopeTrait> {
    /// Propose a step given the current node state, returning a command plus risk estimate.
    fn propose_step(&self, state: &S) -> (C, IndustrialRiskVector);

    /// Apply a step that has already passed ecosafety validation.
    fn apply_step(&mut self, state: &mut S, cmd: C);
}
