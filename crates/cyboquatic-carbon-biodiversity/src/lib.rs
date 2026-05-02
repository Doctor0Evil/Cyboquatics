// crates/cyboquatic-carbon-biodiversity/src/lib.rs
use serde::{Deserialize, Serialize};
use cyboquatic_ecosafety_core::risk::RiskCoord;

/// Raw carbon accounting over a window.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonRaw {
    /// Net sequestered carbon [kg CO2e], positive = removed from atmosphere.
    pub net_sequestered_kg: f64,
    /// Energy consumed [kWh].
    pub energy_kwh: f64,
}

/// Corridor for net carbon intensity [kg CO2e / kWh].
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonCorridor {
    /// Strongly negative (best) intensity, e.g. -0.3 kg/kWh.
    pub safe_kg_per_kwh: f64,
    /// Near-neutral band centre, e.g. -0.05 kg/kWh.
    pub gold_kg_per_kwh: f64,
    /// Hard limit worst acceptable, e.g. 0.05 kg/kWh.
    pub hard_kg_per_kwh: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonScore {
    pub r_carbon:   RiskCoord,
    pub intensity:  f64,
    pub corridor_ok: bool,
}

impl CarbonCorridor {
    pub fn score(&self, raw: CarbonRaw, grid_emissions_kg_per_kwh: f64) -> CarbonScore {
        // Negative intensity is better; include grid emissions.
        let intensity = if raw.energy_kwh > 0.0 {
            let gross = -raw.net_sequestered_kg / raw.energy_kwh;
            gross + grid_emissions_kg_per_kwh
        } else {
            self.hard_kg_per_kwh
        };

        // Safe (more negative) → r ≈ 0, hard (more positive) → r ≈ 1.
        let lo = self.safe_kg_per_kwh;
        let hi = self.hard_kg_per_kwh;
        let mut r = if intensity <= lo {
            0.0
        } else if intensity >= hi {
            1.0
        } else {
            (intensity - lo) / (hi - lo)
        };
        r = r.max(0.0).min(1.0);

        let corridor_ok = intensity <= self.hard_kg_per_kwh + 1e-9;

        CarbonScore {
            r_carbon: RiskCoord::new_clamped(r),
            intensity,
            corridor_ok,
        }
    }
}

/// Raw biodiversity metrics.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityRaw {
    /// Dimensionless connectivity index [0,1], higher is better.
    pub connectivity_index: f64,
    /// Structural complexity, e.g. fractal dimension [1,3], higher is better.
    pub structural_complexity: f64,
    /// Colonization score [0,1], higher is better.
    pub colonization_score: f64,
}

/// Corridors for biodiversity dimensions.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityCorridors {
    pub conn_gold:  f64,
    pub conn_hard:  f64,
    pub comp_gold:  f64,
    pub comp_hard:  f64,
    pub colon_gold: f64,
    pub colon_hard: f64,
    pub w_conn:     f64,
    pub w_comp:     f64,
    pub w_colon:    f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityScore {
    pub r_biodiversity: RiskCoord,
    pub r_conn:         RiskCoord,
    pub r_comp:         RiskCoord,
    pub r_colon:        RiskCoord,
    pub corridor_ok:    bool,
}

impl BiodiversityCorridors {
    fn normalize_inverse_good(gold: f64, hard: f64, value: f64) -> RiskCoord {
        // For metrics where higher is better: value >= gold → r≈0, value <= hard → r≈1.
        let lo = hard;
        let hi = gold;
        let mut r = if value >= hi {
            0.0
        } else if value <= lo {
            1.0
        } else {
            (hi - value) / (hi - lo)
        };
        r = r.max(0.0).min(1.0);
        RiskCoord::new_clamped(r)
    }

    pub fn score(&self, raw: BiodiversityRaw) -> BiodiversityScore {
        let r_conn  = Self::normalize_inverse_good(self.conn_gold,  self.conn_hard,  raw.connectivity_index);
        let r_comp  = Self::normalize_inverse_good(self.comp_gold,  self.comp_hard,  raw.structural_complexity);
        let r_colon = Self::normalize_inverse_good(self.colon_gold, self.colon_hard, raw.colonization_score);

        let wsum = self.w_conn + self.w_comp + self.w_colon;
        let (wc, wq, wl) = if wsum > 0.0 {
            (self.w_conn / wsum, self.w_comp / wsum, self.w_colon / wsum)
        } else {
            (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
        };

        let r_sq =
            wc * r_conn.value().powi(2) +
            wq * r_comp.value().powi(2) +
            wl * r_colon.value().powi(2);

        let r_bio = RiskCoord::new_clamped(r_sq.sqrt());

        let corridor_ok =
            raw.connectivity_index   >= self.conn_hard  - 1e-9 &&
            raw.structural_complexity >= self.comp_hard - 1e-9 &&
            raw.colonization_score   >= self.colon_hard - 1e-9;

        BiodiversityScore {
            r_biodiversity: r_bio,
            r_conn,
            r_comp,
            r_colon,
            corridor_ok,
        }
    }
}
