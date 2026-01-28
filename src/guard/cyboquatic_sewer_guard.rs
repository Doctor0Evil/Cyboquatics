use crate::core::sewer_kernel; // Rust port of SewerDigestionKernel
use crate::shards::SewerNodeShard;

pub struct CorridorBands {
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub weight: f64,
}

// Normalization helper: x -> r_x in [0, 1] inside [safe, hard], >1 means violation.
fn normalize_coord(x: f64, bands: &CorridorBands) -> f64 {
    if x <= bands.safe {
        0.0
    } else if x >= bands.hard {
        1.0
    } else {
        (x - bands.safe) / (bands.hard - bands.safe)
    }
}

pub struct SewerCorridors {
    pub vel: CorridorBands,
    pub tres: CorridorBands,
    pub tss_out: CorridorBands,
    pub fog_out: CorridorBands,
    pub nox: CorridorBands,
    pub pm: CorridorBands,
}

pub struct Residual {
    pub vt: f64,
    pub coords: Vec<f64>,
}

pub struct CorridorDecision {
    pub derate: bool,
    pub stop: bool,
    pub reason: String,
}

// ALN-style invariant: no rx >= 1, and V_{t+1} <= V_t outside the safe interior.
pub fn safestep(prev: &Residual, next: &Residual) -> CorridorDecision {
    let mut max_r = 0.0;
    for &r in &next.coords {
        if r > max_r {
            max_r = r;
        }
    }

    if max_r >= 1.0 {
        return CorridorDecision {
            derate: true,
            stop: true,
            reason: "corridor hard limit violated".to_string(),
        };
    }

    if next.vt > prev.vt {
        return CorridorDecision {
            derate: true,
            stop: false,
            reason: "Lyapunov residual increased".to_string(),
        };
    }

    CorridorDecision {
        derate: false,
        stop: false,
        reason: "ok".to_string(),
    }
}

// Main guard: compute impact, risk coords, and decision for a control tick.
pub fn evaluate_sewer_node(
    shard: &SewerNodeShard,
    corridors: &SewerCorridors,
    prev_residual: &Residual,
) -> (sewer_kernel::SewageNodeResult, Residual, CorridorDecision) {
    // 1. Compute mass removal and node impact.
    let result = sewer_kernel::accumulate_sewage_impact(&shard.samples, &shard.node_cfg);

    // 2. Build risk coordinates from current telemetry.
    let r_vel = normalize_coord(shard.metrics.velocity_ms, &corridors.vel);
    let r_tres = normalize_coord(shard.metrics.t_res_s, &corridors.tres);
    let r_tss = normalize_coord(shard.metrics.cout_tss_mgL, &corridors.tss_out);
    let r_fog = normalize_coord(shard.metrics.cout_fog_mgL, &corridors.fog_out);
    let r_nox = normalize_coord(shard.metrics.nox_mgNm3, &corridors.nox);
    let r_pm  = normalize_coord(shard.metrics.pm_mgNm3, &corridors.pm);

    let coords = vec![r_vel, r_tres, r_tss, r_fog, r_nox, r_pm];

    // 3. Compute updated Lyapunov residual V_{t+1}.
    let mut vt_next = 0.0;
    let weights = [
        corridors.vel.weight,
        corridors.tres.weight,
        corridors.tss_out.weight,
        corridors.fog_out.weight,
        corridors.nox.weight,
        corridors.pm.weight,
    ];
    for (w, r) in weights.iter().zip(coords.iter()) {
        vt_next += w * r;
    }

    let next_residual = Residual {
        vt: vt_next,
        coords,
    };

    // 4. Apply invariants to decide derate/stop.
    let decision = safestep(prev_residual, &next_residual);

    (result, next_residual, decision)
}
