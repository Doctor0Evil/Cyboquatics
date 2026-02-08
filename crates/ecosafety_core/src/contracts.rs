use crate::types::{Residual, RiskCoord, CorridorBands, CorridorDecision};

pub fn corridor_present(corridors: &[CorridorBands]) -> bool {
    // CI/ALN layer enforces the required set; here just "non-empty and all mandatory present".
    corridors.iter().any(|c| c.mandatory)
        && corridors.iter().all(|c| {
            !c.var_id.is_empty()
                && c.hard >= c.gold
                && c.gold >= c.safe
        })
}

// Enforce per-coordinate r_x ≤ 1 and V_{t+1} ≤ V_t outside safe interior.
pub fn safe_step(prev: &Residual, next: &Residual, safe_interior_eps: f64) -> CorridorDecision {
    // 1. Coordinate-wise check
    for rc in &next.coords {
        if rc.value > 1.0 + 1e-9 {
            return CorridorDecision {
                derate: true,
                stop:   true,
                reason: format!("hard-limit breach in {}", rc.bands.var_id),
            };
        }
    }

    // 2. Lyapunov monotonicity (allow slack inside safe interior)
    let all_inside_safe = next.coords.iter().all(|rc| rc.value <= rc.bands.safe + safe_interior_eps);

    if !all_inside_safe && next.vt > prev.vt + 1e-9 {
        return CorridorDecision {
            derate: true,
            stop:   true,
            reason: "Lyapunov residual increased outside safe interior".to_string(),
        };
    }

    CorridorDecision {
        derate: false,
        stop:   false,
        reason: "ok".to_string(),
    }
}
