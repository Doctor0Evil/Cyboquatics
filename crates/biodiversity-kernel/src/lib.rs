// Filename: crates/biodiversity-kernel/src/lib.rs

use serde::{Deserialize, Serialize};
use ecosafety_core::riskvector::RiskCoord;

/// Raw, dimensionful habitat metrics from modeling / measurement.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityRaw {
    /// Dimensionless connectivity index (e.g. graph-based, 0–1).
    pub connectivity_index: f64,
    /// Structural complexity measure (e.g. fractal dimension, normalized 1–3).
    pub structural_complexity: f64,
    /// Colonization score (e.g. % cover of target taxa or lab-derived index).
    pub colonization_score: f64,
}

/// Corridors for each biodiversity dimension.
/// Higher is better: low risk near 0 when value ≥ gold, hard is a minimum.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityCorridors {
    pub conn_gold:  f64,
    pub conn_hard:  f64, // minimum acceptable connectivity

    pub comp_gold:  f64,
    pub comp_hard:  f64, // minimum acceptable complexity

    pub colon_gold: f64,
    pub colon_hard: f64, // minimum acceptable colonization potential

    /// Aggregation weights for composite r_biodiversity.
    pub w_conn:  f64,
    pub w_comp:  f64,
    pub w_colon: f64,
}

/// Normalized biodiversity risk score, ready to plug into RiskVector.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityScore {
    /// Composite biodiversity risk coordinate (0–1).
    pub r_biodiversity: RiskCoord,
    /// Per-dimension risk coordinates (0–1).
    pub r_conn:         RiskCoord,
    pub r_comp:         RiskCoord,
    pub r_colon:        RiskCoord,
    /// True iff all raw metrics meet or exceed the hard corridor minima.
    pub corridor_ok:    bool,
}

impl BiodiversityCorridors {
    /// For metrics where higher is better and `hard` < `gold` is a minimum:
    ///
    /// value ≥ gold  -> r ≈ 0 (low risk, good habitat)
    /// value ≤ hard  -> r ≈ 1 (high risk, corridor violated)
    /// in between    -> linear interpolation in [0,1].
    fn normalize_inverse_good(gold: f64, hard: f64, value: f64) -> RiskCoord {
        let (lo, hi) = (hard, gold);

        // Degenerate corridor: fall back to a pessimistic clamp.
        if hi <= lo {
            return RiskCoord::new_clamped(1.0);
        }

        let r = if value >= hi {
            0.0
        } else if value <= lo {
            1.0
        } else {
            (hi - value) / (hi - lo)
        };

        RiskCoord::new_clamped(r)
    }

    /// Map raw habitat metrics into per-dimension RiskCoords and a composite
    /// r_biodiversity coordinate compatible with the Lyapunov residual.
    pub fn score(&self, raw: BiodiversityRaw) -> BiodiversityScore {
        let r_conn  = Self::normalize_inverse_good(self.conn_gold,  self.conn_hard,  raw.connectivity_index);
        let r_comp  = Self::normalize_inverse_good(self.comp_gold,  self.comp_hard,  raw.structural_complexity);
        let r_colon = Self::normalize_inverse_good(self.colon_gold, self.colon_hard, raw.colonization_score);

        // Weighted quadratic aggregation for the biodiversity plane:
        // r_biodiversity = sqrt(Σ w_i * r_i^2), with Σ w_i = 1.
        let w_sum   = self.w_conn + self.w_comp + self.w_colon;
        let w_conn  = if w_sum > 0.0 { self.w_conn  / w_sum } else { 1.0 / 3.0 };
        let w_comp  = if w_sum > 0.0 { self.w_comp  / w_sum } else { 1.0 / 3.0 };
        let w_colon = if w_sum > 0.0 { self.w_colon / w_sum } else { 1.0 / 3.0 };

        let r_bio_sq =
            w_conn  * r_conn.value().powi(2) +
            w_comp  * r_comp.value().powi(2) +
            w_colon * r_colon.value().powi(2);

        // Composite coordinate is sqrt of weighted sum of squares, clamped to [0,1].
        let r_bio = RiskCoord::new_clamped(r_bio_sq.sqrt());

        // Hard corridor check: all raw metrics must meet or exceed hard minima.
        let corridor_ok =
            raw.connectivity_index    >= self.conn_hard &&
            raw.structural_complexity >= self.comp_hard &&
            raw.colonization_score    >= self.colon_hard;

        BiodiversityScore {
            r_biodiversity: r_bio,
            r_conn,
            r_comp,
            r_colon,
            corridor_ok,
        }
    }
}
