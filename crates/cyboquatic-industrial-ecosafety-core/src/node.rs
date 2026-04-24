//! Node identity, media class, and minimal state/command envelopes
//! for industrial Cyboquatic machinery.

use crate::IndustrialRiskVector;

/// Industrial node class for Cyboquatic workloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeClass {
    MarModule,
    FogDesiccator,
    AirGlobe,
    Cain,
    CanalPurifier,
    Other,
}

/// Medium in which the node operates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediumClass {
    Water,
    Air,
    Fog,
    Mixed,
}

/// Lane governance level for deployment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    Research,
    Experimental,
    Production,
}

/// Minimal state snapshot seen by controllers.
///
/// All rich telemetry (pressures, flows, detailed chemistry) is folded
/// into risk coordinates upstream and carried separately.
#[derive(Clone, Copy, Debug)]
pub struct NodeState {
    pub node_id: u64,
    pub class: NodeClass,
    pub medium: MediumClass,
    pub lane: Lane,
}

impl NodeState {
    /// Create a new node state snapshot.
    pub fn new(node_id: u64, class: NodeClass, medium: MediumClass, lane: Lane) -> Self {
        Self { node_id, class, medium, lane }
    }
}

/// Abstract view of node state needed for ecosafety decisions.
///
/// This trait is implemented by concrete node types to expose
/// the minimal state required by the ecosafety kernel.
pub trait NodeStateView {
    fn node_class(&self) -> NodeClass;
    fn medium(&self) -> MediumClass;
    fn lane(&self) -> Lane;
    fn current_risks(&self) -> IndustrialRiskVector;
}

/// Minimal command envelope that a legacy PLC or drive would see.
///
/// This is deliberately thin and mechanical; all ecological semantics
/// live in the ecosafety kernel and corridors, not in this type.
#[derive(Clone, Copy, Debug)]
pub struct CommandEnvelope {
    pub target_pump_rpm: f32,
    pub valve_open_fraction: f32,
    pub fan_duty_cycle: f32,
    pub mode_flags: u32,
}

impl CommandEnvelope {
    /// Check if this command is a no-op (all zeros).
    pub fn is_noop(&self) -> bool {
        self.target_pump_rpm == 0.0
            && self.valve_open_fraction == 0.0
            && self.fan_duty_cycle == 0.0
            && self.mode_flags == 0
    }

    /// Create a noop command envelope.
    pub fn noop() -> Self {
        Self {
            target_pump_rpm: 0.0,
            valve_open_fraction: 0.0,
            fan_duty_cycle: 0.0,
            mode_flags: 0,
        }
    }
}
