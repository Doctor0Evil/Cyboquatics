// crates/cyboquatic-ecosafety-core/src/risk.rs
use serde::{Deserialize, Serialize};

/// Normalized [0,1] risk coordinate.
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

/// Unified risk vector, including energy, carbon, materials, and biodiversity.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RiskVector {
    pub r_energy:        RiskCoord,
    pub r_hydraulics:    RiskCoord,
    pub r_biology:       RiskCoord,
    pub r_carbon:        RiskCoord,
    pub r_materials:     RiskCoord,
    pub r_biodiversity:  RiskCoord,
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
    /// Quadratic Lyapunov residual V_t = Σ_j w_j * r_j^2.
    pub fn residual(&self, w: LyapunovWeights) -> f64 {
        let mut vt = 0.0;
        vt += w.w_energy       * self.r_energy.value().powi(2);
        vt += w.w_hydraulics   * self.r_hydraulics.value().powi(2);
        vt += w.w_biology      * self.r_biology.value().powi(2);
        vt += w.w_carbon       * self.r_carbon.value().powi(2);
        vt += w.w_materials    * self.r_materials.value().powi(2);
        vt += w.w_biodiversity * self.r_biodiversity.value().powi(2);
        vt
    }

    /// Hard band: no coordinate at or above 1.0.
    pub fn any_hard_breach(&self) -> bool {
        let coords = [
            self.r_energy,
            self.r_hydraulics,
            self.r_biology,
            self.r_carbon,
            self.r_materials,
            self.r_biodiversity,
        ];
        coords.iter().any(|c| c.value() >= 1.0 - 1e-9)
    }
}

/// Safe decision channel for controllers and FOG router.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SafeDecision {
    Accept,
    Derate,
    Stop,
}

/// Global safestep invariant: V_{t+1} ≤ V_t, plus hard-band checks.
pub fn safestep(prev: Residual, next: Residual, rv_next: RiskVector, w: LyapunovWeights) -> SafeDecision {
    if rv_next.any_hard_breach() {
        return SafeDecision::Stop;
    }

    let vt  = prev.value;
    let vt1 = next.value;
    let eps = 1e-3;

    if vt + eps < vt1 && vt > 1e-9 {
        SafeDecision::Stop
    } else if vt + eps < vt1 && vt <= 1e-9 {
        SafeDecision::Derate
    } else {
        SafeDecision::Accept
    }
}
