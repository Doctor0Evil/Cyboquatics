//! Corridor and step decisions used at the ecosafety gate.

/// Result of evaluating corridors and Lyapunov residual.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorridorDecision {
    /// Fully safe; actuation may proceed as proposed.
    Ok,
    /// Derate; actuation must be reduced (e.g., lower duty).
    Derate,
    /// Stop; actuation must be blocked.
    Stop,
}

/// Final verdict for a control step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepVerdict {
    pub decision: CorridorDecision,
    pub v_prev: f32,
    pub v_next: f32,
}
