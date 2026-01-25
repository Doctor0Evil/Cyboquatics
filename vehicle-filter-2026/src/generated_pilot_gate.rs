// Transpiled from VehicleFilterPilotGate2026v1.aln on 2026-01-25.
// Pilot-Gate bindings for VehicleFilter2026v1 shards.
// Internal hex-stamps for bostrom DID anchoring, aligned with ecosafety spine.

use crate::contracts::{CorridorBands, RiskCoord, Residual};
use crate::shard::{VehicleFilterShard, CorridorRow};
use crate::normalize_exhaust::VehicleFilterBands;

// Enum for gate decisions, grounded in dual-threshold corridors.
#[derive(Clone, Debug, PartialEq)]
pub enum PilotGateDecision {
    Approve,
    Derate,
    Stop,
    Fail(String), // For invariant or completeness violations.
}

// Const hex-stamps for predicate authorship (bostrom-anchored).
const HEX_STAMP_HAS_ALL_CORRIDORS: &str =
    "0x4a2b1c3d5e6f7890a1b2c3d4e5f67890abcdef1234567890fedcba9876543210";
const HEX_STAMP_COMPUTE_RESIDUAL: &str =
    "0xabcdef1234567890fedcba98765432104a2b1c3d5e6f7890a1b2c3d4e5f67890";
const HEX_STAMP_SAFESTEP: &str =
    "0xfedcba98765432104a2b1c3d5e6f7890abcdef12345678901b2c3d4e5f67890a";
const HEX_STAMP_PILOT_GATE: &str =
    "0x1b2c3d4e5f67890a4a2b1c3d5e6f7890fedcba9876543210abcdef1234567890";
const HEX_STAMP_GOVERN: &str =
    "0xa1b2c3d4e5f678904a2b1c3d5e6f7890fedcba9876543210abcdef1234567890";

// Canonical required corridor variable IDs for VehicleFilter2026v1.
// Must match qpudatashard / ALN IDs.
pub const REQUIRED_VARS: &[&str] = &[
    "pm",
    "nox",
    "hc",
    "co",
    "backpressure",
    "substrate_temp",
];

// Predicate 1: has_all_corridors (ensures "no corridor, no deployment").
pub fn has_all_corridors(shard: &VehicleFilterShard) -> bool {
    let _stamp = HEX_STAMP_HAS_ALL_CORRIDORS; // Bostrom anchor.

    // Simulated DID validity (wire to real verifier in production).
    if shard.header.did_signature.is_empty() {
        return false;
    }

    for var in REQUIRED_VARS {
        if !shard.corridors.iter().any(|c| c.varid == *var) {
            return false;
        }
    }

    true
}

// Predicate 2: compute_residual (normalize risk and Vt).
// Uses corridor bands plus shard risk_state to reconstruct rx and Vt.
pub fn compute_residual(shard: &VehicleFilterShard) -> Option<Residual> {
    let _stamp = HEX_STAMP_COMPUTE_RESIDUAL; // Bostrom anchor.

    // Collect bands per corridor row.
    let bands: Vec<CorridorBands> = shard.corridors_to_bands();
    if bands.is_empty() {
        return None;
    }

    let mut coords: Vec<RiskCoord> = Vec::with_capacity(bands.len());

    for band in bands {
        // Measured value from shard risk_state (expand as needed).
        let measured = shard
            .risk_state
            .rx
            .get(&band.varid)
            .cloned()
            .unwrap_or(0.0);

        let rx = if measured <= band.safe {
            0.0
        } else if measured >= band.hard {
            1.0
        } else {
            (measured - band.safe) / (band.hard - band.safe)
        };

        // Hard bound: rx must not exceed 1.0.
        if rx > 1.0 {
            return None;
        }

        coords.push(RiskCoord {
            value: rx,
            sigma: 0.0,
            bands: band,
        });
    }

    let vt: f64 = coords.iter().map(|rc| rc.bands.weight_w * rc.value).sum();
    Some(Residual { vt, coords })
}

// Predicate 3: safestep (Lyapunov invariant for time-series).
// Enforces strictly increasing timestamps and V_{t+1} <= V_t outside safe interiors.
pub fn safestep(prev_shard: &VehicleFilterShard, next_shard: &VehicleFilterShard) -> bool {
    let _stamp = HEX_STAMP_SAFESTEP; // Bostrom anchor.

    // Time-order invariant.
    if prev_shard.header.timestamp >= next_shard.header.timestamp {
        return false;
    }

    let prev_residual = compute_residual(prev_shard);
    let next_residual = compute_residual(next_shard);
    if prev_residual.is_none() || next_residual.is_none() {
        return false;
    }

    let prev_res = prev_residual.as_ref().unwrap();
    let next_res = next_residual.as_ref().unwrap();

    let prev_vt = prev_res.vt;
    let next_vt = next_res.vt;

    // Safe interior: rx <= bands.safe_rx (or bands.safe interpreted as rx-safe).
    let all_safe = next_res
        .coords
        .iter()
        .all(|rc| rc.value <= rc.bands.safe_rx);

    all_safe || (next_vt <= prev_vt)
}

// Predicate 4: pilot_gate_decision (approve/derate/stop).
pub fn pilot_gate_decision(shard: &VehicleFilterShard) -> PilotGateDecision {
    let _stamp = HEX_STAMP_PILOT_GATE; // Bostrom anchor.

    if !has_all_corridors(shard) {
        return PilotGateDecision::Fail("Missing corridors or DID".into());
    }

    let residual = match compute_residual(shard) {
        Some(r) => r,
        None => return PilotGateDecision::Fail("Residual computation failed".into()),
    };

    let vt = residual.vt;

    // K/E/R gate: eco_impact and risk-of-harm bands.
    if shard.ker.eco_impact_value <= 0.9 || shard.ker.risk_of_harm >= 0.15 {
        return PilotGateDecision::Fail("KER gate violation".into());
    }

    // Dual-threshold on Vt: approve <0.5, derate [0.5,1.0), stop >=1.0.
    if vt < 0.5 {
        PilotGateDecision::Approve
    } else if vt < 1.0 {
        PilotGateDecision::Derate
    } else {
        PilotGateDecision::Stop
    }
}

// Predicate 5: govern_vehicle_filter (chain governance).
// Enforces safestep across the shard chain and then applies Pilot-Gate to the last shard.
pub fn govern_vehicle_filter(shard_chain: &[VehicleFilterShard]) -> PilotGateDecision {
    let _stamp = HEX_STAMP_GOVERN; // Bostrom anchor.

    if shard_chain.is_empty() {
        return PilotGateDecision::Fail("Empty chain".into());
    }

    // Time-series invariants over the chain.
    for i in 0..shard_chain.len().saturating_sub(1) {
        let prev = &shard_chain[i];
        let next = &shard_chain[i + 1];

        if !safestep(prev, next) {
            return PilotGateDecision::Fail(format!("Safestep failed at index {}", i));
        }

        if prev.header.did_signature.is_empty() {
            return PilotGateDecision::Fail("DID signature missing".into());
        }
    }

    // Final decision from last shard.
    pilot_gate_decision(&shard_chain[shard_chain.len() - 1])
}

// Integration hook: call from daemon for runtime gating.
pub fn gate_on_shard_chain(shard_chain: &[VehicleFilterShard]) -> bool {
    matches!(
        govern_vehicle_filter(shard_chain),
        PilotGateDecision::Approve
    )
}
