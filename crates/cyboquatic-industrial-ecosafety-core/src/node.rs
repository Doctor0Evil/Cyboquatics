//! Node identity, media class, and minimal state/command envelopes
//! for industrial Cyboquatic machinery.

#[derive(Clone, Copy, Debug)]
pub enum NodeClass {
    MarModule,
    FogDesiccator,
    CanalPurifier,
    AirGlobe,
    Cain,
    Other,
}

#[derive(Clone, Copy, Debug)]
pub enum MediumClass {
    Water,
    Air,
    Fog,
    Mixed,
}

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
