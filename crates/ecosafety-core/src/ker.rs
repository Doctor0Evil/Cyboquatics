use serde::{Deserialize, Serialize};

use crate::types::RiskVector;

/// KER metrics over a rolling window.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct KerTriad {
    pub k: f64,
    pub e: f64,
    pub r: f64,
}

impl KerTriad {
    pub fn new(k: f64, e: f64, r: f64) -> Self {
        Self {
            k: k.clamp(0.0, 1.0),
            e: e.clamp(0.0, 1.0),
            r: r.clamp(0.0, 1.0),
        }
    }

    /// Simple default: E = 1 - R.
    pub fn from_window<RV: AsRef<[RiskVector]>, D: AsRef<[bool]>>(
        risk_series: RV,
        decisions_accept_or_derate: D,
    ) -> Self {
        let rvs = risk_series.as_ref();
        let flags = decisions_accept_or_derate.as_ref();

        let n = rvs.len().min(flags.len());
        if n == 0 {
            return Self::new(0.0, 0.0, 1.0);
        }

        let mut accept_count = 0usize;
        let mut r_max = 0.0f64;

        for i in 0..n {
            if flags[i] {
                accept_count += 1;
            }

            let rv = &rvs[i];
            let coords = [
                rv.r_energy,
                rv.r_hydraulics,
                rv.r_biology,
                rv.r_carbon,
                rv.r_materials,
                rv.r_biodiversity,
                rv.r_sigma,
            ];
            for c in coords {
                r_max = r_max.max(c.value());
            }
        }

        let k = (accept_count as f64) / (n as f64);
        let r = r_max.clamp(0.0, 1.0);
        let e = (1.0 - r).clamp(0.0, 1.0);

        Self::new(k, e, r)
    }

    /// Production gate: K ≥ 0.90, E ≥ 0.90, R ≤ 0.13.
    pub fn is_production_grade(&self) -> bool {
        self.k >= 0.90 && self.e >= 0.90 && self.r <= 0.13
    }
}
