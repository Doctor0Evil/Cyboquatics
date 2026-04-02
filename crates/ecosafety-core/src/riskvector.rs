// Filename: crates/ecosafety-core/src/riskvector.rs

use serde::{Deserialize, Serialize};

/// Normalized 0–1 risk coordinate.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RiskCoord(pub f64);

impl RiskCoord {
    pub fn new_clamped(v: f64) -> Self {
        Self(v.max(0.0).min(1.0))
    }
    pub fn value(self) -> f64 {
        self.0
    }
}

/// Unified risk vector including carbon & biodiversity.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RiskVector {
    pub renergy:        RiskCoord,
    pub rhydraulics:    RiskCoord,
    pub rbiology:       RiskCoord,
    pub rcarbon:        RiskCoord,
    pub rmaterials:     RiskCoord,
    pub rbiodiversity:  RiskCoord,
    // existing planes like rspecies, rtox, rplume can remain here
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LyapunovWeights {
    pub w_energy:       f64,
    pub w_hydraulics:   f64,
    pub w_biology:      f64,
    pub w_carbon:       f64,
    pub w_materials:    f64,
    pub w_biodiversity: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Residual {
    pub value: f64,
}

impl Residual {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl RiskVector {
    /// V_t = Σ_j w_j r_j^2
    pub fn residual(&self, w: LyapunovWeights) -> f64 {
        let mut vt = 0.0;
        vt += w.w_energy       * self.renergy.value().powi(2);
        vt += w.w_hydraulics   * self.rhydraulics.value().powi(2);
        vt += w.w_biology      * self.rbiology.value().powi(2);
        vt += w.w_carbon       * self.rcarbon.value().powi(2);
        vt += w.w_materials    * self.rmaterials.value().powi(2);
        vt += w.w_biodiversity * self.rbiodiversity.value().powi(2);
        vt
    }

    /// Hard-band check: no coordinate at or above 1.0.
    pub fn any_hard_breach(&self) -> bool {
        let coords = [
            self.renergy,
            self.rhydraulics,
            self.rbiology,
            self.rcarbon,
            self.rmaterials,
            self.rbiodiversity,
        ];
        coords.iter().any(|c| c.value() >= 1.0 - 1e-9)
    }
}
