// crates/cyboquatic-fog-router/src/eco_predicates.rs

use cyboquatic_ecosafety_core::spine::{CorridorDecision, CorridorBands, KerWindow, RiskVector};
use crate::node::NodeShard;      // wraps CyboquaticIndustrialNode2026v1 ALN row.
use crate::workload::Workload;   // typed workload descriptor (no "CyboVariant" symbol). [file:23]

/// Enforce carbon‑negative / biodegradable corridors when routing
/// high‑impact industrial workloads. [file:21][file:23]
pub fn eco_industrial_ok(node: &NodeShard, workload: &Workload) -> bool {
    // Only apply to workloads marked as industrial / heavy.
    if !workload.is_industrial {
        return true;
    }

    let r_c = node.r_carbon;
    let r_m = node.r_materials;

    // Require both planes in the gold band for actuation.
    r_c <= 0.30 && r_m <= 0.30
}

/// Composite decision that includes energy, hydraulics, biology,
/// and the eco‑industrial corridors.
pub fn decide_industrial_route(
    node: &NodeShard,
    workload: &Workload,
    bands: &CorridorBands,
    ker: &KerWindow,
) -> CorridorDecision {
    // First, respect existing energy/hydraulics/bio/Vt predicates
    // (tailwindvalid, hydraulicok, biosurfaceok, lyapunovok). [file:21]
    if !super::tailwind_valid(node, workload) {
        return CorridorDecision::Derate;
    }
    if !super::hydraulic_ok(node, workload) {
        return CorridorDecision::Derate;
    }
    if !super::biosurface_ok(node, workload) {
        return CorridorDecision::Derate;
    }
    if !super::lyapunov_ok(node, workload, bands) {
        return CorridorDecision::Stop;
    }

    // Then apply eco‑industrial corridor.
    if !eco_industrial_ok(node, workload) {
        return CorridorDecision::Stop;
    }

    // Optionally require production‑grade KER for dispatch. [file:23][file:24]
    if !ker.is_production_grade() {
        return CorridorDecision::Derate;
    }

    CorridorDecision::Ok
}
