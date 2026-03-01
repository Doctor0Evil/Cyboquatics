use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KerScore {
    pub k: f64,
    pub e: f64,
    pub r: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KerInputs {
    pub num_external_studies: usize,
    pub num_pilots: usize,
    pub corridor_coverage: f64, // 0–1 fraction of required metrics with calibrated bands
    pub impact_deforestation: f64, // 0–1 normalized benefit
    pub impact_pollutants: f64,    // 0–1 normalized benefit
    pub impact_resilience: f64,    // 0–1 normalized benefit
    pub residual_uncertainty: f64, // 0–1 normalized model/sensor uncertainty
}

impl KerScore {
    pub fn from_inputs(inp: &KerInputs) -> Self {
        let study_term = (inp.num_external_studies as f64).min(20.0) / 20.0;
        let pilot_term = (inp.num_pilots as f64).min(10.0) / 10.0;
        let k = 0.4 * study_term + 0.3 * pilot_term + 0.3 * inp.corridor_coverage;

        let e = 0.34 * inp.impact_deforestation
            + 0.33 * inp.impact_pollutants
            + 0.33 * inp.impact_resilience;

        let r_raw = 0.6 * inp.residual_uncertainty
            + 0.4 * (1.0 - e).max(0.0);
        let r = r_raw.clamp(0.0, 1.0);

        Self { k: k.clamp(0.0, 1.0), e: e.clamp(0.0, 1.0), r }
    }
}
