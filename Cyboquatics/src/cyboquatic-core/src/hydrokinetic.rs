use crate::{CyboquaticError, PowerNode};

/// Constant water density at standard seawater conditions [kg/m^3].[web:13]
const WATER_DENSITY_KG_M3: f64 = 1025.0;

/// Compute ideal hydrokinetic power for a given flow speed and capture area:
/// P = 0.5 * rho * A * v^3 * eta,
/// where eta is an overall efficiency factor in [0,1].[file:5]
pub fn compute_hydro_power_kw(
    flow_ms: f64,
    capture_area_m2: f64,
    efficiency: f64,
) -> Result<f64, CyboquaticError> {
    if flow_ms <= 0.0 {
        return Err(CyboquaticError::InvalidParameter(
            "flow_ms must be > 0".into(),
        ));
    }
    if capture_area_m2 <= 0.0 {
        return Err(CyboquaticError::InvalidParameter(
            "capture_area_m2 must be > 0".into(),
        ));
    }
    if !(0.0..=1.0).contains(&efficiency) {
        return Err(CyboquaticError::InvalidParameter(
            "efficiency must be in [0,1]".into(),
        ));
    }

    let power_w = 0.5 * WATER_DENSITY_KG_M3 * capture_area_m2 * flow_ms.powi(3) * efficiency;
    Ok(power_w / 1000.0)
}

/// Estimate a conservative capture area for a node from its rated power and mean flow.
/// This inverts the hydrokinetic equation using an assumed efficiency.[file:5]
pub fn estimate_capture_area_m2(
    node: &PowerNode,
    assumed_efficiency: f64,
) -> Result<f64, CyboquaticError> {
    if node.rated_power_kw <= 0.0 {
        return Err(CyboquaticError::InvalidParameter(
            "rated_power_kw must be > 0".into(),
        ));
    }
    let efficiency = if assumed_efficiency > 0.0 && assumed_efficiency <= 1.0 {
        assumed_efficiency
    } else {
        0.35
    };
    let power_w = node.rated_power_kw * 1000.0;
    let area = (2.0 * power_w) / (WATER_DENSITY_KG_M3 * node.mean_flow_ms.powi(3) * efficiency);
    Ok(area)
}
