use crate::CyboquaticError;

/// Simple PFBS mass-balance model for a cyboquatic unit treating a control volume.[web:13]
#[derive(Debug, Clone)]
pub struct PfbsRemediationConfig {
    pub volume_m3: f64,
    pub inflow_m3_per_h: f64,
    pub removal_efficiency: f64, // fraction [0,1] of PFBS removed per pass
}

impl PfbsRemediationConfig {
    pub fn validate(&self) -> Result<(), CyboquaticError> {
        if self.volume_m3 <= 0.0 {
            return Err(CyboquaticError::InvalidParameter(
                "volume_m3 must be > 0".into(),
            ));
        }
        if self.inflow_m3_per_h <= 0.0 {
            return Err(CyboquaticError::InvalidParameter(
                "inflow_m3_per_h must be > 0".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.removal_efficiency) {
            return Err(CyboquaticError::InvalidParameter(
                "removal_efficiency must be in [0,1]".into(),
            ));
        }
        Ok(())
    }
}

/// Compute steady-state PFBS outlet concentration (µg/L) given inlet concentration (µg/L),
/// assuming a continuous stirred-tank reactor model with fractional removal per pass.[web:13]
pub fn steady_state_pfbs_outlet_ug_l(
    cfg: &PfbsRemediationConfig,
    inlet_ug_l: f64,
) -> Result<f64, CyboquaticError> {
    cfg.validate()?;
    if inlet_ug_l < 0.0 {
        return Err(CyboquaticError::InvalidParameter(
            "inlet_ug_l must be >= 0".into(),
        ));
    }
    let fraction_remaining = 1.0 - cfg.removal_efficiency;
    Ok(inlet_ug_l * fraction_remaining)
}

/// Compute hourly PFBS mass removed [kg/h] from inlet and outlet concentrations.[web:13]
pub fn pfbs_mass_removed_kg_per_h(
    cfg: &PfbsRemediationConfig,
    inlet_ug_l: f64,
    outlet_ug_l: f64,
) -> Result<f64, CyboquaticError> {
    cfg.validate()?;
    if outlet_ug_l > inlet_ug_l {
        return Err(CyboquaticError::InvalidParameter(
            "outlet_ug_l cannot exceed inlet_ug_l".into(),
        ));
    }
    // Convert µg/L to kg/m^3 (1 µg/L = 1e-6 kg/m^3).
    let delta_kg_per_m3 = (inlet_ug_l - outlet_ug_l) * 1e-6;
    let mass_flow_kg_per_h = delta_kg_per_m3 * cfg.inflow_m3_per_h;
    Ok(mass_flow_kg_per_h)
}
