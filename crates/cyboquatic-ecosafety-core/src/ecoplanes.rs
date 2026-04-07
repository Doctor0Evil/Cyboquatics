// File: crates/cyboquatic-ecosafety-core/src/ecoplanes.rs

use crate::{RiskCoord, RiskVector, Scalar};

/// Raw carbon metrics for one control interval.
pub struct CarbonMetrics {
    pub net_co2e_kg:      Scalar, // emissions – sequestration
    pub service_output:   Scalar, // e.g. m^3 treated, kWh delivered
    pub corridor_neg:     Scalar, // strongly carbon-negative bound (kg / unit)
    pub corridor_neutral: Scalar, // carbon-neutral band center (kg / unit)
    pub corridor_pos:     Scalar, // unacceptable net emissions (kg / unit)
}

/// Map carbon performance into r_carbon ∈ [0, 1].
pub fn normalize_carbon(m: &CarbonMetrics) -> RiskCoord {
    if m.service_output <= 0.0 {
        return RiskCoord::clamped(1.0);
    }
    let intensity = m.net_co2e_kg / m.service_output;

    if intensity <= m.corridor_neg {
        return RiskCoord::clamped(0.0);
    }
    if intensity <= m.corridor_neutral {
        let t = (intensity - m.corridor_neg) / (m.corridor_neutral - m.corridor_neg);
        return RiskCoord::clamped(0.2 * t);
    }
    if intensity >= m.corridor_pos {
        return RiskCoord::clamped(1.0);
    }
    let t = (intensity - m.corridor_neutral) / (m.corridor_pos - m.corridor_neutral);
    RiskCoord::clamped(0.2 + 0.8 * t)
}

/// Material degradation and toxicity metrics for substrates / casings.
pub struct MaterialKinetics {
    pub t90_days:         Scalar, // time to 90% degradation
    pub t90_gold_days:    Scalar, // desired fast-degrade band
    pub t90_hard_days:    Scalar, // maximum allowed
    pub r_tox_raw:        Scalar, // 0–1 normalized toxicity from LC/MS
    pub r_micro_raw:      Scalar, // 0–1 normalized micro-residue risk
    pub r_leach_raw:      Scalar, // 0–1 normalized CEC / PFAS leach risk
    pub r_pf_resid_raw:   Scalar, // 0–1 PFAS residual risk
}

/// Normalize t90 into a risk coordinate.
fn risk_from_t90(k: &MaterialKinetics) -> RiskCoord {
    if k.t90_days <= k.t90_gold_days {
        return RiskCoord::clamped(0.0);
    }
    if k.t90_days >= k.t90_hard_days {
        return RiskCoord::clamped(1.0);
    }
    let t = (k.t90_days - k.t90_gold_days) / (k.t90_hard_days - k.t90_gold_days);
    RiskCoord::clamped(t)
}

/// Aggregate material sub‑risks into r_materials.
pub fn aggregate_material_risk(k: &MaterialKinetics) -> RiskCoord {
    let r_t90   = risk_from_t90(k).value();
    let r_tox   = RiskCoord::clamped(k.r_tox_raw).value();
    let r_micro = RiskCoord::clamped(k.r_micro_raw).value();
    let r_leach = RiskCoord::clamped(k.r_leach_raw).value();
    let r_pf    = RiskCoord::clamped(k.r_pf_resid_raw).value();

    // Weights reflect current governance priorities; tune via shards, not code.
    let w_t90   = 0.25;
    let w_tox   = 0.25;
    let w_micro = 0.20;
    let w_leach = 0.15;
    let w_pf    = 0.15;

    let r = w_t90 * r_t90
          + w_tox * r_tox
          + w_micro * r_micro
          + w_leach * r_leach
          + w_pf * r_pf;

    RiskCoord::clamped(r)
}

/// Helper to build a RiskVector from per-plane metrics
/// (energy, hydraulics, biology are supplied by domain‑specific kernels).
pub struct PlaneInputs {
    pub r_energy:    RiskCoord,
    pub r_hydraulic: RiskCoord,
    pub r_biology:   RiskCoord,
    pub carbon:      CarbonMetrics,
    pub materials:   MaterialKinetics,
}

pub fn build_risk_vector(p: &PlaneInputs) -> RiskVector {
    RiskVector {
        r_energy:    p.r_energy,
        r_hydraulic: p.r_hydraulic,
        r_biology:   p.r_biology,
        r_carbon:    normalize_carbon(&p.carbon),
        r_materials: aggregate_material_risk(&p.materials),
    }
}
