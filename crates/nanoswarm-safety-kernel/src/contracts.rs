#![no_std]

use crate::types::Residual;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CorridorDecision {
    Ok,
    Derate,
    Stop,
}

/// Lyapunov / corridor gate used by all ecosafety kernels.
/// - Stop: any r_j >= 1.0
/// - Derate: no hard breach, but V_{t+1} > V_t outside safe interior
/// - Ok: otherwise
pub fn safestep(prev: &Residual, next: &Residual) -> CorridorDecision {
    let any_hard = next
        .coords
        .iter()
        .any(|c| c.r >= 1.0);

    if any_hard {
        return CorridorDecision::Stop;
    }

    if next.vt > prev.vt {
        return CorridorDecision::Derate;
    }

    CorridorDecision::Ok
}
/// Minimal view of the nanoswarm.corridor.v1 shard needed for CI.
/// In practice this should come from your ALN → Rust generator.
pub struct CorridorRow<'a> {
    pub varid: &'a str,
    pub mandatory: bool,
}

pub struct NanoswarmCorridorShard<'a> {
    pub moduletype: &'a str,
    pub region: &'a str,
    pub corridors: &'a [CorridorRow<'a>],
}

/// Returns Ok(()) if all mandatory nanoswarm corridors are present,
/// Err(msg) otherwise. Intended for use in build.rs / CI.
pub fn corridor_present(shard: &NanoswarmCorridorShard<'_>) -> Result<(), &'static str> {
    // Only enforce for nanoswarm‑class modules; other moduletype values
    // can be ignored or handled by domain‑specific kernels.
    if !shard.moduletype.starts_with("Nanoswarm-") {
        return Ok(());
    }

    // Required corridors for the current nanoswarm-safety-kernel.
    const REQUIRED: [&str; 4] = ["TDI", "MBI", "EcoImpactScore", "RadiationIndex"];

    for &var in REQUIRED.iter() {
        let mut found = false;
        for row in shard.corridors.iter() {
            if row.mandatory && row.varid == var {
                found = true;
                break;
            }
        }
        if !found {
            return Err("missing mandatory nanoswarm corridor");
        }
    }

    Ok(())
}
