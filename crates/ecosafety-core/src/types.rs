#![forbid(unsafe_code)]

use std::collections::HashMap;

/// Canonical corridor bands for one variable, aligned with your
/// varid, units, safe, gold, hard, weight, lyapchannel, mandatory schema.[file:88][file:89]
#[derive(Clone, Debug)]
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

/// Normalized risk coordinate r in [0,1] with optional sigma for uncertainty.[file:89][file:91]
#[derive(Clone, Debug)]
pub struct RiskCoord {
    pub var_id: String,
    pub r: f64,
    pub sigma: f64,
    pub bands: CorridorBands,
}

/// Lyapunov-style residual V_t over a set of risk coordinates.[file:88][file:91]
#[derive(Clone, Debug)]
pub struct Residual {
    pub vt: f64,
    pub coords: Vec<RiskCoord>,
}

/// Ecosafety decision used across controllers and portals.[file:84][file:88]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorridorDecision {
    Ok,
    Derate { reason: String },
    Stop { reason: String },
}
