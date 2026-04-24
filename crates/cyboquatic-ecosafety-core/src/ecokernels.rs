// crates/cyboquatic-ecosafety-core/src/ecokernels.rs

use crate::spine::{RiskCoord, RiskVector};

/// Carbon metrics per operational cycle of a machine.
#[derive(Clone, Copy, Debug)]
pub struct CarbonMetrics {
    /// Net kg CO2‑e per cycle (positive = emissions, negative = sequestration).
    pub kg_co2e_per_cycle: f64,
    /// Reference corridor bounds in kg CO2‑e per cycle.
    pub corridor_neg_best: f64, // strongly negative (best)
    pub corridor_neutral: f64,  // around zero
    pub corridor_pos_worst: f64,
}

/// Materials metrics for biodegradable industrial substrates.
#[derive(Clone, Copy, Debug)]
pub struct MaterialMetrics {
    /// Time to 90 % degradation in days (ASTM/ISO style). [file:22][file:23]
    pub t90_days: f64,
    /// Hard corridor maximum for t90 (e.g. 180 days). [file:22][file:23]
    pub t90_max_days: f64,
    /// Normalized toxicity risk r_tox in [0,1] from leachate / LCMS. [file:22][file:23]
    pub r_tox: f64,
    /// Micro‑residue / micro‑plastic risk r_micro in [0,1]. [file:22][file:23]
    pub r_micro: f64,
    /// Leachate / CEC risk r_cec in [0,1]. [file:23]
    pub r_cec: f64,
}

/// Map carbon metrics into a normalized risk coordinate r_carbon in [0,1].
/// Sequestration (negative emissions) -> near 0; strongly positive -> near 1. [file:23]
pub fn carbon_to_risk(m: &CarbonMetrics) -> RiskCoord {
    let x = m.kg_co2e_per_cycle;
    if x <= m.corridor_neg_best {
        0.0
    } else if x >= m.corridor_pos_worst {
        1.0
    } else if x <= m.corridor_neutral {
        // map [neg_best, neutral] to [0, 0.3] (gold band).
        let frac = (x - m.corridor_neg_best) / (m.corridor_neutral - m.corridor_neg_best);
        0.3 * frac
    } else {
        // map (neutral, pos_worst] to (0.3, 1].
        let frac = (x - m.corridor_neutral) / (m.corridor_pos_worst - m.corridor_neutral);
        0.3 + 0.7 * frac
    }
}

/// Map material metrics into r_materials in [0,1],
/// rewarding fast, non‑toxic, low‑micro‑residue substrates. [file:22][file:23]
pub fn materials_to_risk(m: &MaterialMetrics) -> RiskCoord {
    // Degradation component: t90 in [0, t90_max] -> r_deg in [0,1].
    let t = m.t90_days.max(0.0);
    let r_deg = (t / m.t90_max_days).min(1.0);

    // Composite risk: weighted mean of degradation, toxicity, micro‑residue, and CEC.
    // These weights align with your materials plane treating toxicity and micro‑risk
    // as critical. [file:22][file:23]
    let w_deg = 0.25;
    let w_tox = 0.30;
    let w_micro = 0.25;
    let w_cec = 0.20;

    let r = w_deg * r_deg
        + w_tox * m.r_tox
        + w_micro * m.r_micro
        + w_cec * m.r_cec;

    r.min(1.0).max(0.0)
}

/// Helper for updating the carbon & materials planes in a RiskVector.
pub fn update_carbon_and_materials(
    rv: &mut RiskVector,
    carbon: &CarbonMetrics,
    materials: &MaterialMetrics,
) {
    rv.r_carbon = carbon_to_risk(carbon);
    rv.r_materials = materials_to_risk(materials);
}
