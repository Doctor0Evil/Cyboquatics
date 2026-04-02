// Filename: crates/ecosafety-core/src/safestep.rs

use serde::{Deserialize, Serialize};
use crate::riskvector::{RiskVector, LyapunovWeights};
use crate::riskvector::Residual;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SafeDecision {
    Accept,
    Derate,
    Stop,
}

pub fn safestep(prev: Residual, next: Residual, rv_next: RiskVector, w: LyapunovWeights) 
    -> SafeDecision 
{
    if rv_next.any_hard_breach() {
        return SafeDecision::Stop;
    }

    let vt  = prev.value;
    let vt1 = next.value;
    let eps = 1e-3;

    if vt > eps && vt1 > vt + 1e-9 {
        return SafeDecision::Stop;
    }

    if vt <= eps && vt1 > vt + eps {
        SafeDecision::Derate
    } else {
        SafeDecision::Accept
    }
}
