// File: crates/cyboquatic-material-kinetics/src/lib.rs

#![forbid(unsafe_code)]

use cyboquatic_ecosafety_core::{RiskCoord, RiskVector};

/// Lab-derived kinetics for a candidate substrate.
#[derive(Clone, Copy, Debug)]
pub struct KineticsSummary {
    /// Time to 90 % mass loss [days].
    pub t90_days: f64,
    /// Leachate toxicity index (0 = non-toxic, 1 = highly toxic).
    pub tox_index: f64,
    /// Micro-residue index (0 = none, 1 = heavy persistent microfragments).
    pub micro_index: f64,
    /// Leachate CEC / PFAS index (0 = none, 1 = worst case).
    pub leach_cec_index: f64,
}

/// Corridor configuration for material risks.
#[derive(Clone, Copy, Debug)]
pub struct MaterialCorridor {
    pub t90_safe_max_days: f64,
    pub t90_hard_max_days: f64,
    pub tox_safe_max: f64,
    pub tox_hard_max: f64,
    pub micro_safe_max: f64,
    pub micro_hard_max: f64,
    pub leach_safe_max: f64,
    pub leach_hard_max: f64,
}

impl MaterialCorridor {
    pub fn phoenix_default() -> Self {
        Self {
            // Example: require 90 % degradation within 180 days safe, hard stop at 365.
            t90_safe_max_days: 180.0,
            t90_hard_max_days: 365.0,
            // Toxicity indices: safe < 0.1, hard stop at 0.3.
            tox_safe_max: 0.10,
            tox_hard_max: 0.30,
            // Micro-residue: safe < 0.05, hard stop at 0.20.
            micro_safe_max: 0.05,
            micro_hard_max: 0.20,
            // Leachate CEC / PFAS: safe < 0.05, hard stop at 0.20.
            leach_safe_max: 0.05,
            leach_hard_max: 0.20,
        }
    }

    fn normalize_linear(&self, value: f64, safe_max: f64, hard_max: f64) -> f64 {
        if value <= safe_max {
            0.0
        } else if value >= hard_max {
            1.0
        } else {
            (value - safe_max) / (hard_max - safe_max)
        }
    }

    fn r_t90(&self, t90_days: f64) -> f64 {
        self.normalize_linear(t90_days, self.t90_safe_max_days, self.t90_hard_max_days)
    }

    fn r_tox(&self, tox_index: f64) -> f64 {
        self.normalize_linear(tox_index, self.tox_safe_max, self.tox_hard_max)
    }

    fn r_micro(&self, micro_index: f64) -> f64 {
        self.normalize_linear(micro_index, self.micro_safe_max, self.micro_hard_max)
    }

    fn r_leach(&self, leach_index: f64) -> f64 {
        self.normalize_linear(leach_index, self.leach_safe_max, self.leach_hard_max)
    }

    /// Aggregate into a single r_materials coordinate.
    pub fn aggregate_r_materials(&self, k: &KineticsSummary) -> RiskCoord {
        let r_t90 = self.r_t90(k.t90_days);
        let r_tox = self.r_tox(k.tox_index);
        let r_micro = self.r_micro(k.micro_index);
        let r_leach = self.r_leach(k.leach_cec_index);
        // Use max as conservative aggregator.
        let r = r_t90.max(r_tox).max(r_micro).max(r_leach);
        RiskCoord::new(r)
    }

    /// Hard corridor gate: only true if all underlying coordinates are < 1.
    pub fn corridor_ok(&self, k: &KineticsSummary) -> bool {
        self.r_t90(k.t90_days) < 1.0
            && self.r_tox(k.tox_index) < 1.0
            && self.r_micro(k.micro_index) < 1.0
            && self.r_leach(k.leach_cec_index) < 1.0
    }
}

/// Lift material risk into a RiskVector.
pub fn update_risk_vector_with_materials(
    rv_current: &RiskVector,
    r_materials: RiskCoord,
) -> RiskVector {
    RiskVector {
        r_energy: rv_current.r_energy,
        r_hydraulics: rv_current.r_hydraulics,
        r_biology: rv_current.r_biology,
        r_carbon: rv_current.r_carbon,
        r_materials,
        r_biodiversity: rv_current.r_biodiversity,
        r_sigma: rv_current.r_sigma,
    }
}
