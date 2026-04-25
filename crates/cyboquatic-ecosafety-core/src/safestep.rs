// Filename: crates/cyboquatic-ecosafety-core/src/safestep.rs
// Role: Safestep invariant; non‑increasing V_t with per‑plane hard bands.[file:12]

#![forbid(unsafe_code)]
#![no_std]

use serde::{Deserialize, Serialize};

use crate::riskvector::{Residual, RiskCoord, RiskVector};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SafeDecision {
    Accept,
    Derate,
    Stop,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SafeStepConfig {
    pub epsilon:        f64,
    pub max_risk:       RiskCoord,
}

impl Default for SafeStepConfig {
    fn default() -> Self {
        Self {
            epsilon: 1.0e-3,
            max_risk: RiskCoord::new_clamped(0.13),
        }
    }
}

pub fn safestep(prev: Residual,
                next: Residual,
                rv_next: RiskVector,
                cfg: SafeStepConfig) -> SafeDecision {
    // Hard gate: any plane (including rcarbon, rbiodiversity, rcalib) ≥ band → Stop.[file:13][file:12]
    if rv_next.max_coord().value() >= cfg.max_risk.value() {
        return SafeDecision::Stop;
    }

    let vt  = prev.value;
    let vt1 = next.value;

    if vt1 > vt + cfg.epsilon {
        SafeDecision::Stop
    } else if vt1 > vt {
        SafeDecision::Derate
    } else {
        SafeDecision::Accept
    }
}
