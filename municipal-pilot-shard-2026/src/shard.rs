// MunicipalPilotShard2026v1 – governance-grade shard for MAR / AirGlobe / biochar pilots.

use serde::{Deserialize, Serialize};

/// Normalized corridor bands for a single variable (safe/gold/hard plus weight).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorridorBands {
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub weight_w: f64,
    /// Optional: small tolerance around safe interior (0.0–0.1).
    pub safe_rx: f64,
}

/// Risk coordinate entry: normalized rx plus bands and raw measurement.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskCoordEntry {
    pub var_id: String,        // e.g. "pfas", "hlr", "wbgt", "grid_co2"
    pub measured: f64,         // raw physical value (units documented in annex)
    pub rx: f64,               // normalized 0–1 risk coordinate
    pub bands: CorridorBands,
}

/// Lyapunov-style residual for the pilot node.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Residual {
    pub vt: f64,               // aggregate residual V(t) >= 0
    pub coords: Vec<RiskCoordEntry>,
}

/// K/E/R scores for the pilot configuration/window.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KerScores {
    pub k_knowledge: f64,      // 0–1 fraction of config bound to typed corridors/shards
    pub e_eco_impact: f64,     // 0–1 normalized benefit (mass removed, WBGT delta, recharge)
    pub r_risk_of_harm: f64,   // 0–1 residual risk based on residual trends / violations
}

/// Explicit cost fields for transparency (no license cost for ecosafety layer).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostFields {
    // Capital and O&M are scoped to hardware/integration, NOT the ecosafety grammar.
    pub capex_hardware_usd: f64,        // MAR vaults, AirGlobe housings, biochar units
    pub capex_sensors_usd: f64,         // field sensors, telemetry
    pub capex_integration_usd: f64,     // installation, SCADA/IT integration

    pub opex_energy_usd_per_year: f64,  // electricity, pumps, fans, etc.
    pub opex_maintenance_usd_per_year: f64,
    pub opex_lab_monitor_usd_per_year: f64,

    // Always zero for the safety kernel; kept for explicitness and audit.
    pub ecosafety_license_fee_usd: f64, // MUST be 0.0 for Cyboquatics ecosafety layer
}

/// DID and provenance for audit and chain anchoring.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PilotHeader {
    pub shard_id: String,          // unique ID (UUID/tx-hash style)
    pub node_id: String,           // e.g. "PHX-MAR-01", "PHX-AIRGLOBE-05"
    pub medium: String,            // "water", "air", "soil"
    pub region: String,            // e.g. "Phoenix-AZ"
    pub twindow_start_utc: String, // ISO 8601
    pub twindow_end_utc: String,   // ISO 8601

    pub did: String,               // Bostrom / municipal DID
    pub did_signature: String,     // hex-encoded signature
    pub evidence_hex: String,      // hex-stamp for external evidence bundle
}

/// MunicipalPilotShard2026v1 – single artifact for pilots and annexes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MunicipalPilotShard2026v1 {
    pub header: PilotHeader,

    // Safety grammar core
    pub corridors: Vec<RiskCoordEntry>, // required vars: pfas, cec, hlr, temp, wbgt, grid_co2, etc.
    pub residual: Residual,             // V(t) and coords normalized

    // Governance metrics
    pub ker: KerScores,                 // K/E/R triad
    pub dt_trust: f64,                  // multonry trust scalar 0–1 (optional)

    // Costs (explicit separation of safety layer vs hardware)
    pub costs: CostFields,

    // Lane and pilot-gate state
    pub lane: String,                   // "PRODUCTION" | "EXPERIMENTAL"
    pub pilot_gate_decision: String,    // "APPROVE" | "DERATE" | "STOP" | "PENDING"
}

/// Normalize a raw metric into rx 0–1 using corridor bands (safe=0, hard=1).
pub fn normalize_metric(measured: f64, bands: &CorridorBands) -> f64 {
    if measured <= bands.safe {
        0.0
    } else if measured >= bands.hard {
        1.0
    } else {
        (measured - bands.safe) / (bands.hard - bands.safe)
    }
}

/// Recompute V(t) as weighted sum of rx from shard corridors.
pub fn compute_residual(coords: &[RiskCoordEntry]) -> Residual {
    let vt = coords
        .iter()
        .map(|c| c.bands.weight_w * c.rx)
        .sum::<f64>();

    Residual {
        vt,
        coords: coords.to_vec(),
    }
}

/// CI/annex invariant: no corridor, no deployment (structure only).
pub fn corridor_present(shard: &MunicipalPilotShard2026v1, required_vars: &[&str]) -> bool {
    if shard.corridors.is_empty() {
        return false;
    }
    for var in required_vars {
        let found = shard.corridors.iter().any(|c| c.var_id == *var);
        if !found {
            return false;
        }
    }
    true
}

/// Runtime invariant: violated corridor = derate/stop (Lyapunov-style sketch).
pub fn safe_step(prev: &Residual, next: &Residual) -> bool {
    // Hard-limit check: no rx >= 1.0 allowed.
    let hard_ok = next.coords.iter().all(|c| c.rx < 1.0);
    if !hard_ok {
        return false; // force derate/stop
    }

    // Safe interior: all rx <= safe_rx; allow V(t) to float.
    let all_safe = next
        .coords
        .iter()
        .all(|c| c.rx <= c.bands.safe_rx);

    if all_safe {
        return true;
    }

    // Outside safe interior: require V_next <= V_prev (non-increasing residual).
    next.vt <= prev.vt
}
