// Filename: crates/cyboquatic-biodiversity-kernel/src/lib.rs
// Role: Non‑actuating rbiodiversity kernel for habitat‑positive machinery.[file:13]

#![forbid(unsafe_code)]
#![no_std]

use serde::{Deserialize, Serialize};
use cyboquatic_ecosafety_core::riskvector::RiskCoord;

/// Raw, dimensionless biodiversity metrics.[file:13]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityRaw {
    /// Connectivity index (0–1) from graph / flow models.
    pub connectivity_index:   f64,
    /// Structural complexity (e.g. normalized fractal dimension 1–3).
    pub structural_complexity: f64,
    /// Colonization score (0–1) from trays, settlement or LC–MS assays.
    pub colonization_score:   f64,
}

/// Corridors for biodiversity dimensions (higher is better).[file:13]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityCorridors {
    pub conn_gold:  f64,
    pub conn_hard:  f64, // minimum acceptable connectivity
    pub comp_gold:  f64,
    pub comp_hard:  f64, // minimum acceptable complexity
    pub colon_gold: f64,
    pub colon_hard: f64, // minimum acceptable colonization

    pub w_conn:     f64,
    pub w_comp:     f64,
    pub w_colon:    f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiodiversityScore {
    pub rbiodiversity: RiskCoord,
    pub r_conn:        RiskCoord,
    pub r_comp:        RiskCoord,
    pub r_colon:       RiskCoord,
    pub corridor_ok:   bool,
}

impl BiodiversityCorridors {
    fn normalize_inverse_good(gold: f64, hard: f64, value: f64) -> RiskCoord {
        // For metrics where higher is better; values ≥ gold map near 0 risk.[file:13]
        let lo = hard;
        let hi = gold;
        let r = if value >= hi {
            0.0
        } else if value <= lo {
            1.0
        } else {
            (hi - value) / (hi - lo).max(1.0e-9)
        };
        RiskCoord::new_clamped(r)
    }

    pub fn score(&self, raw: BiodiversityRaw) -> BiodiversityScore {
        let r_conn  = Self::normalize_inverse_good(self.conn_gold,  self.conn_hard,  raw.connectivity_index);
        let r_comp  = Self::normalize_inverse_good(self.comp_gold,  self.comp_hard,  raw.structural_complexity);
        let r_colon = Self::normalize_inverse_good(self.colon_gold, self.colon_hard, raw.colonization_score);

        let w_sum = (self.w_conn + self.w_comp + self.w_colon).max(1.0e-9);
        let wc = self.w_conn  / w_sum;
        let wx = self.w_comp  / w_sum;
        let wz = self.w_colon / w_sum;

        let rbio_sq =
            wc * r_conn.value().powi(2) +
            wx * r_comp.value().powi(2) +
            wz * r_colon.value().powi(2);

        let rbiodiversity = RiskCoord::new_clamped(rbio_sq.sqrt());

        let corridor_ok = raw.connectivity_index   >= self.conn_hard  &&
                          raw.structural_complexity >= self.comp_hard  &&
                          raw.colonization_score   >= self.colon_hard;

        BiodiversityScore {
            rbiodiversity,
            r_conn,
            r_comp,
            r_colon,
            corridor_ok,
        }
    }
}
