#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Canonical corridor bands for one variable, aligned with
/// var_id, units, safe, gold, hard, weight, lyap_channel, mandatory schema.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorridorBands {
    pub var_id: String,
    pub units: String,
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub weight: f64,
    pub lyap_channel: u16,
    pub mandatory: bool,
}

/// Normalized [0,1] risk coordinate (scalar form used in the residual).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct RiskCoord(pub f64);

impl RiskCoord {
    pub fn new_clamped(v: f64) -> Self {
        Self(v.max(0.0).min(1.0))
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

/// Extended risk coordinate with var_id, sigma, and attached corridor bands.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskCoordExt {
    pub var_id: String,
    pub r: f64,
    pub sigma: f64,
    pub bands: CorridorBands,
}

impl RiskCoordExt {
    pub fn as_scalar(&self) -> RiskCoord {
        RiskCoord::new_clamped(self.r)
    }
}

/// Canonical multi-plane risk vector for Spine v1.
///
/// Additional coordinates may be added in downstream crates via wrappers or
/// parallel structs, but the scalar residual is always computed from a set of
/// RiskCoord values normalized to [0,1].
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RiskVector {
    pub r_energy: RiskCoord,
    pub r_hydraulics: RiskCoord,
    pub r_biology: RiskCoord,
    pub r_carbon: RiskCoord,
    pub r_materials: RiskCoord,
    pub r_biodiversity: RiskCoord,
    pub r_sigma: RiskCoord,
}

impl RiskVector {
    pub fn any_hard_breach(&self) -> bool {
        let coords = [
            self.r_energy,
            self.r_hydraulics,
            self.r_biology,
            self.r_carbon,
            self.r_materials,
            self.r_biodiversity,
            self.r_sigma,
        ];
        coords
            .iter()
            .any(|c| c.value() >= 1.0 - 1e-9)
    }
}

/// Lyapunov weights for each coordinate in the canonical RiskVector.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LyapunovWeights {
    pub w_energy: f64,
    pub w_hydraulics: f64,
    pub w_biology: f64,
    pub w_carbon: f64,
    pub w_materials: f64,
    pub w_biodiversity: f64,
    pub w_sigma: f64,
}

/// Scalar Lyapunov residual.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Residual {
    pub value: f64,
}

impl Residual {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

/// Lyapunov-style residual V_t over a set of extended risk coordinates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResidualExt {
    pub vt: f64,
    pub coords: Vec<RiskCoordExt>,
}

impl ResidualExt {
    pub fn from_coords(coords: Vec<RiskCoordExt>) -> Self {
        let mut vt = 0.0;
        for c in &coords {
            let r = c.r.max(0.0).min(1.0);
            vt += c.bands.weight * r * r;
        }
        Self { vt, coords }
    }
}

/// Ecosafety decision used across controllers and portals.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorridorDecision {
    Ok,
    Derate { reason: String },
    Stop { reason: String },
}

/// Optional lookup table for corridor bands indexed by var_id.
pub type CorridorTable = HashMap<String, CorridorBands>;
