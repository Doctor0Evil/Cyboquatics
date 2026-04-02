// Filename: crates/biodiversity-kernel/src/lib.rs

use serde::{Deserialize, Serialize};
use ecosafety_core::riskvector::RiskCoord;

/// Raw, dimensionful habitat metrics from modeling / measurement.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityRaw {
    /// Dimensionless connectivity index (e.g. graph-based, 0–1).
    pub connectivity_index:  f64,
    /// Structural complexity measure (e.g. fractal dimension, normalized 1–3).
    pub structural_complexity: f64,
    /// Colonization score (e.g. % cover of target taxa or lab-derived index).
    pub colonization_score:  f64,
}

/// Corridors for each biodiversity dimension.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityCorridors {
    // Higher is better => low risk near 0 when above gold.
    pub conn_gold:  f64,
    pub conn_hard:  f64,    // minimum acceptable connectivity

    pub comp_gold:  f64,
    pub comp_hard:  f64,    // minimum acceptable complexity

    pub colon_gold: f64,
    pub colon_hard: f64,    // minimum acceptable colonization potential

    /// Aggregation weights for composite r_biodiversity.
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
        // For metrics where higher is better and `hard` < `gold` is a minimum:
        // value >= gold  -> r ≈ 0
        // value <= hard  -> r ≈ 1
        // linear in between.
        let (lo, hi) = (hard, gold);
        let r = if value >= hi {
            0.0
        } else if value <= lo {
            1.0
        } else {
            (hi - value) / (hi - lo)
        };
        RiskCoord::new_clamped(r)
    }

    pub fn score(&self, raw: BiodiversityRaw) -> BiodiversityScore {
        let r_conn  = Self::normalize_inverse_good(self.conn_gold,  self.conn_hard,  raw.connectivity_index);
        let r_comp  = Self::normalize_inverse_good(self.comp_gold,  self.comp_hard,  raw.structural_complexity);
        let r_colon = Self::normalize_inverse_good(self.colon_gold, self.colon_hard, raw.colonization_score);

        // Weighted quadratic aggregation for the plane.
        let w_sum = self.w_conn + self.w_comp + self.w_colon;
        let w_conn = if w_sum > 0.0 { self.w_conn / w_sum } else { 1.0/3.0 };
        let w_comp = if w_sum > 0.0 { self.w_comp / w_sum } else { 1.0/3.0 };
        let w_col  = if w_sum > 0.0 { self.w_colon / w_sum } else { 1.0/3.0 };

        let r_bio_sq =
              w_conn * r_conn.value().powi(2)
            + w_comp * r_comp.value().powi(2)
            + w_col  * r_colon.value().powi(2);

        // Composite coordinate is sqrt of weighted sum of squares.
        let r_bio = RiskCoord::new_clamped(r_bio_sq.sqrt());

        let corridor_ok =
            raw.connectivity_index  >= self.conn_hard  &&
            raw.structural_complexity >= self.comp_hard &&
            raw.colonization_score >= self.colon_hard;

        BiodiversityScore {
            r_biodiversity: r_bio,
            r_conn,
            r_comp,
            r_colon,
            corridor_ok,
        }
    }
}
