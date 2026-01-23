use ecosafety_spine::{RiskCoord, CorridorBands, Residual, CorridorDecision, safestep};

/// Pure, deterministic normalization kernel: ΔP → r_P in [0,1].
pub fn normalize_delta_p(dp_measured: f64, bands: &CorridorBands) -> RiskCoord {
    // piecewise-linear map 0 at safe, 1 at hard (clipped); gold used by controllers.
    let x = dp_measured;
    let safe = bands.safe;
    let hard = bands.hard;
    let value = if x <= safe {
        0.0
    } else if x >= hard {
        1.0
    } else {
        (x - safe) / (hard - safe)
    };
    RiskCoord {
        value,
        bands: bands.clone(),
        sigma: 0.0, // or sensor-uncertainty model
    }
}

/// Similar kernels for velocity, shear, fouling, energy per m³, etc.
pub fn normalize_velocity(v: f64, bands: &CorridorBands) -> RiskCoord { /* as above */ }
pub fn normalize_shear(tau: f64, bands: &CorridorBands) -> RiskCoord { /* as above */ }
pub fn normalize_fouling(rfoul_raw: f64, bands: &CorridorBands) -> RiskCoord { /* as above */ }

/// Compute Lyapunov-style residual V_t = Σ w_j * r_j for a flow-vac module.
pub fn compute_residual(coords: &[RiskCoord]) -> Residual {
    let vt = coords.iter()
        .map(|rc| rc.bands.weight * rc.value)
        .sum();
    Residual {
        vt,
        rx: coords.to_vec(),
        w: coords.iter().map(|rc| rc.bands.weight).collect(),
    }
}

/// Single-step corridor-enforced update.
/// Controllers must call this before actuating any hardware.
pub fn flowvac_safestep(prev: &Residual, next: &Residual) -> CorridorDecision {
    safestep(prev, next) // delegates to spine’s formally verified invariant
}
