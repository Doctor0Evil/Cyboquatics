use crate::{CyboquaticError, PowerNode};

/// Safety envelope parameters derived from cyboquatic.marine.safety.envelope.v1.[file:5][web:13]
#[derive(Debug, Clone)]
pub struct MarineSafetyEnvelope {
    pub max_flow_velocity_ms: f64,
    pub max_intake_gradient_ms: f64,
    pub min_screen_mesh_mm: f64,
    pub exclusion_radius_m: f64,
}

impl Default for MarineSafetyEnvelope {
    fn default() -> Self {
        Self {
            max_flow_velocity_ms: 0.3,
            max_intake_gradient_ms: 0.05,
            min_screen_mesh_mm: 5.0,
            exclusion_radius_m: 10.0,
        }
    }
}

/// Check whether a candidate intake flow speed and gradient is compliant with the envelope.
/// Returns Ok(()) when safe, or an error describing the violation.[web:13]
pub fn check_intake_compliance(
    flow_ms: f64,
    gradient_ms_per_cm: f64,
    envelope: &MarineSafetyEnvelope,
) -> Result<(), CyboquaticError> {
    if flow_ms > envelope.max_flow_velocity_ms {
        return Err(CyboquaticError::InvalidParameter(format!(
            "intake flow {} m/s exceeds marine envelope {} m/s",
            flow_ms, envelope.max_flow_velocity_ms
        )));
    }
    if gradient_ms_per_cm > envelope.max_intake_gradient_ms {
        return Err(CyboquaticError::InvalidParameter(format!(
            "intake gradient {} m/s/cm exceeds marine envelope {} m/s/cm",
            gradient_ms_per_cm, envelope.max_intake_gradient_ms
        )));
    }
    Ok(())
}

/// Derive a recommended maximum intake flow speed for a node by taking
/// the minimum of node.max_intake_flow_ms and the envelope cap.[file:5]
pub fn recommended_intake_flow_ms(
    node: &PowerNode,
    envelope: &MarineSafetyEnvelope,
) -> f64 {
    node.max_intake_flow_ms
        .min(envelope.max_flow_velocity_ms)
}
