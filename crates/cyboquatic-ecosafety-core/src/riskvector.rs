// Filename: crates/cyboquatic-ecosafety-core/src/riskvector.rs
// Role: Core ecosafety grammar for Cyboquatic machinery (non‑actuating).
// K≈0.95, E≈0.93, R≈0.11 band as extension of existing core.[file:13][file:12]

#![forbid(unsafe_code)]
#![no_std]

use serde::{Deserialize, Serialize};

/// Normalized 0–1 risk coordinate (lower is better).
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

/// Unified risk vector, now including carbon and biodiversity planes.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RiskVector {
    pub renergy:       RiskCoord,
    pub rhydraulics:   RiskCoord,
    pub rbiology:      RiskCoord,
    pub rcarbon:       RiskCoord,      // NEW: carbon‑negative plane
    pub rmaterials:    RiskCoord,
    pub rbiodiversity: RiskCoord,      // NEW: habitat plane
    pub rcalib:        RiskCoord,      // data/ingest quality plane from rcalib work
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LyapunovWeights {
    pub wenergy:       f64,
    pub whydraulics:   f64,
    pub wbiology:      f64,
    pub wcarbon:       f64,
    pub wmaterials:    f64,
    pub wbiodiversity: f64,
    pub wcalib:        f64,
}

impl Default for LyapunovWeights {
    fn default() -> Self {
        Self {
            wenergy:       1.0,
            whydraulics:   1.0,
            wbiology:      1.2,
            wcarbon:       1.3,
            wmaterials:    1.1,
            wbiodiversity: 1.1,
            wcalib:        0.8,
        }
    }
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
    /// Quadratic Lyapunov residual V_t = Σ w_j r_j^2 including carbon/biodiversity.[file:13][file:12]
    pub fn residual(self, w: LyapunovWeights) -> Residual {
        let v =
            w.wenergy       * self.renergy.value().powi(2) +
            w.whydraulics   * self.rhydraulics.value().powi(2) +
            w.wbiology      * self.rbiology.value().powi(2) +
            w.wcarbon       * self.rcarbon.value().powi(2) +
            w.wmaterials    * self.rmaterials.value().powi(2) +
            w.wbiodiversity * self.rbiodiversity.value().powi(2) +
            w.wcalib        * self.rcalib.value().powi(2);
        Residual::new(v)
    }

    /// Max coordinate across all planes (hard gate, including carbon/biodiversity).[file:13]
    pub fn max_coord(self) -> RiskCoord {
        let mut m = self.renergy.value();
        for r in [
            self.rhydraulics,
            self.rbiology,
            self.rcarbon,
            self.rmaterials,
            self.rbiodiversity,
            self.rcalib,
        ] {
            if r.value() > m {
                m = r.value();
            }
        }
        RiskCoord::new_clamped(m)
    }
}
