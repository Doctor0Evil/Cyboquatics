// biodeg_kinetics.rs: Rust model for biodegradable tray decomposition kinetics
// DID-signed: bostrom1qxljm5f632vgw0dh3y8cqqrv4jes6grxn9zqda9ncv9yddgtyv4qs3mrx4
// Hex-stamp: 0x1234567890ABCDEF

use std::f64;

#[derive(Clone, Debug)]
pub struct BiodegParams {
    pub k: f64,  // specific degradation rate (day^-1)
    pub y: f64,  // yield coefficient
    pub d: f64,  // death rate (day^-1)
    pub s0: f64, // initial substrate (kg)
    pub x0: f64, // initial biomass (kg)
}

pub fn simulate_decomposition(params: &BiodegParams, dt: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    let mut results = Vec::with_capacity(steps);
    let mut t = 0.0;
    let mut s = params.s0;
    let mut x = params.x0;
    results.push((t, s, x));

    for _ in 1..steps {
        let ds_dt = -params.k * s * x;
        let dx_dt = params.y * params.k * s * x - params.d * x;
        s += ds_dt * dt;
        x += dx_dt * dt;
        if s < 0.0 { s = 0.0; }
        if x < 0.0 { x = 0.0; }
        t += dt;
        results.push((t, s, x));
    }
    results
}

pub fn compute_t90(results: &[(f64, f64, f64)], s0: f64) -> Option<f64> {
    let threshold = 0.1 * s0;  // 90% degraded when s <= 0.1 s0
    results.iter().find(|&&(_, s, _)| s <= threshold).map(|&(t, _, _)| t)
}

// Invariant: eco_impact > 0.9 if t90 < 180 days
pub fn eco_impact(t90: Option<f64>) -> f64 {
    match t90 {
        Some(t) if t < 180.0 => 0.95 - (t / 180.0) * 0.05,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decomposition() {
        let params = BiodegParams { k: 0.05, y: 0.5, d: 0.01, s0: 1.0, x0: 0.1 };
        let results = simulate_decomposition(&params, 0.1, 2000);
        let t90 = compute_t90(&results, params.s0);
        assert!(t90.unwrap_or(f64::INFINITY) < 180.0);
        let impact = eco_impact(t90);
        assert!(impact > 0.9);
    }
}
