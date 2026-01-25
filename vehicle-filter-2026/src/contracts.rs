#[derive(Clone, Debug)]
pub struct CorridorBands {
    pub varid: String,
    pub units: String,
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub weight_w: f64,
    pub lyap_channel: u16,
}

#[derive(Clone, Debug)]
pub struct RiskCoord {
    pub value: f64,        // rx in [0, 1]
    pub sigma: f64,        // uncertainty
    pub bands: CorridorBands,
}

#[derive(Clone, Debug)]
pub struct Residual {
    pub vt: f64,
    pub coords: Vec<RiskCoord>,
}

#[derive(Clone, Debug)]
pub struct CorridorDecision {
    pub derate: bool,
    pub stop: bool,
    pub reason: String,
}

// Build-time invariant: no corridor, no build.
pub fn corridor_present(required_ids: &[&str], corridors: &[CorridorBands]) -> bool {
    for rid in required_ids {
        if !corridors.iter().any(|c| c.varid == *rid) {
            return false;
        }
    }
    true
}

// Runtime invariant: rx <= 1 and V_{t+1} <= V_t outside safe interior.
pub fn safestep(prev: &Residual, next: &Residual) -> CorridorDecision {
    for rc in &next.coords {
        if rc.value > 1.0 {
            return CorridorDecision {
                derate: true,
                stop: true,
                reason: format!("Hard corridor breach for {}", rc.bands.varid),
            };
        }
    }

    let all_safe = next.coords.iter().all(|rc| rc.value <= rc.bands.safe);
    if !all_safe && next.vt > prev.vt {
        return CorridorDecision {
            derate: true,
            stop: true,
            reason: "Lyapunov residual increased outside safe interior".into(),
        };
    }

    CorridorDecision {
        derate: false,
        stop: false,
        reason: "OK".into(),
    }
}
