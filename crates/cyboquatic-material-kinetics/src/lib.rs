// File: crates/cyboquatic-material-kinetics/src/lib.rs

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use cyboquatic_ecosafety_core::risk::{RiskCoord, RiskVector};

/// Lab-derived material kinetics and toxicity metrics as raw inputs.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MaterialKineticsRaw {
    /// Time to 90% mass loss [days].
    pub t90_days: f64,
    /// Micro-residue concentration at t90 [mg/L or µg/kg].
    pub micro_residue: f64,
    /// Leachate toxicity index (e.g., normalized LC50, 0 = non-toxic, 1 = highly toxic).
    pub leach_toxicity: f64,
}

/// Corridor bands for biodegradable substrates (single-plane version).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MaterialCorridors {
    /// Max safe t90 (fast breakdown), e.g. 180 days.
    pub t90_safe_max: f64,
    /// Hard max t90, e.g. 365 days.
    pub t90_hard_max: f64,

    /// Max safe micro-residue level.
    pub micro_safe_max: f64,
    /// Hard max micro-residue.
    pub micro_hard_max: f64,

    /// Max safe leachate toxicity index.
    pub tox_safe_max: f64,
    /// Hard max leachate toxicity.
    pub tox_hard_max: f64,

    /// Aggregation weights.
    pub w_t90:   f64,
    pub w_micro: f64,
    pub w_tox:   f64,
}

/// Risk score in material plane derived from MaterialKineticsRaw and MaterialCorridors.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MaterialRiskScore {
    pub r_t90:       RiskCoord,
    pub r_micro:     RiskCoord,
    pub r_tox:       RiskCoord,
    pub r_materials: RiskCoord,
    pub corridor_ok: bool,
}

impl MaterialCorridors {
    fn normalize_pos(value: f64, safe_max: f64, hard_max: f64) -> RiskCoord {
        let r = if value <= safe_max {
            0.0
        } else if value >= hard_max {
            1.0
        } else {
            (value - safe_max) / (hard_max - safe_max)
        };
        RiskCoord::new_clamped(r)
    }

    /// Score a raw material sample into per-metric risks and a composite r_materials.
    pub fn score(&self, raw: MaterialKineticsRaw) -> MaterialRiskScore {
        let r_t90   = Self::normalize_pos(raw.t90_days,       self.t90_safe_max,   self.t90_hard_max);
        let r_micro = Self::normalize_pos(raw.micro_residue,  self.micro_safe_max, self.micro_hard_max);
        let r_tox   = Self::normalize_pos(raw.leach_toxicity, self.tox_safe_max,   self.tox_hard_max);

        let wsum = self.w_t90 + self.w_micro + self.w_tox;
        let (wt, wm, wv) = if wsum > 0.0 {
            (self.w_t90 / wsum, self.w_micro / wsum, self.w_tox / wsum)
        } else {
            (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
        };

        let r_sq =
            wt * r_t90.value().powi(2) +
            wm * r_micro.value().powi(2) +
            wv * r_tox.value().powi(2);

        let r_materials = RiskCoord::new_clamped(r_sq.sqrt());

        let corridor_ok =
            raw.t90_days       <= self.t90_hard_max   + 1e-9 &&
            raw.micro_residue  <= self.micro_hard_max + 1e-9 &&
            raw.leach_toxicity <= self.tox_hard_max   + 1e-9;

        MaterialRiskScore {
            r_t90,
            r_micro,
            r_tox,
            r_materials,
            corridor_ok,
        }
    }
}

/// Lab-derived kinetics summary for a candidate substrate (extended view).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct KineticsSummary {
    /// Time to 90% mass loss [days].
    pub t90_days: f64,
    /// Leachate toxicity index (0 = non-toxic, 1 = highly toxic).
    pub tox_index: f64,
    /// Micro-residue index (0 = none, 1 = heavy persistent microfragments).
    pub micro_index: f64,
    /// Leachate CEC / PFAS index (0 = none, 1 = worst case).
    pub leach_cec_index: f64,
}

/// Corridor configuration for material risks (multi-metric, index-based view).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MaterialCorridorIndex {
    pub t90_safe_max_days: f64,
    pub t90_hard_max_days: f64,
    pub tox_safe_max: f64,
    pub tox_hard_max: f64,
    pub micro_safe_max: f64,
    pub micro_hard_max: f64,
    pub leach_safe_max: f64,
    pub leach_hard_max: f64,
}

impl MaterialCorridorIndex {
    /// Phoenix-class default corridors for biodegradable substrates.
    pub fn phoenix_default() -> Self {
        Self {
            t90_safe_max_days: 180.0,
            t90_hard_max_days: 365.0,
            tox_safe_max: 0.10,
            tox_hard_max: 0.30,
            micro_safe_max: 0.05,
            micro_hard_max: 0.20,
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

    /// Aggregate t90, tox, micro, leach into a single r_materials coordinate (max-aggregated).
    pub fn aggregate_r_materials(&self, k: &KineticsSummary) -> RiskCoord {
        let r_t90   = self.r_t90(k.t90_days);
        let r_tox   = self.r_tox(k.tox_index);
        let r_micro = self.r_micro(k.micro_index);
        let r_leach = self.r_leach(k.leach_cec_index);

        let r = r_t90
            .max(r_tox)
            .max(r_micro)
            .max(r_leach);

        RiskCoord::new_clamped(r)
    }

    /// Hard corridor gate: only true if all underlying coordinates are strictly below 1.
    pub fn corridor_ok(&self, k: &KineticsSummary) -> bool {
        self.r_t90(k.t90_days)        < 1.0 &&
        self.r_tox(k.tox_index)       < 1.0 &&
        self.r_micro(k.micro_index)   < 1.0 &&
        self.r_leach(k.leach_cec_index) < 1.0
    }
}

/// Lift material risk into the global RiskVector, keeping all other planes unchanged.
pub fn update_risk_vector_with_materials(
    rv_current: &RiskVector,
    r_materials: RiskCoord,
) -> RiskVector {
    RiskVector {
        r_energy:       rv_current.r_energy,
        r_hydraulics:   rv_current.r_hydraulics,
        r_biology:      rv_current.r_biology,
        r_carbon:       rv_current.r_carbon,
        r_materials:    r_materials,
        r_biodiversity: rv_current.r_biodiversity,
    }
}
