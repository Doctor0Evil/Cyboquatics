// co2-substrate-conv-2026/src/kernels.rs
// vt residuals for CO2-substrate stability.
use crate::contracts::{CorridorBands, RiskCoord, Residual};

pub fn to_rj(measured: f64, bands: &CorridorBands) -> RiskCoord {
    let r = if measured <= bands.safe {
        0.0
    } else if measured >= bands.hard {
        1.0
    } else {
        (measured - bands.safe) / (bands.hard - bands.safe)
    };
    RiskCoord {
        value: r,
        sigma: 0.0,
        bands: bands.clone(),
    }
}

pub fn vt_from_coords(coords: &[RiskCoord]) -> Residual {
    let vt = coords.iter().map(|rc| rc.bands.weight_w * rc.value).sum::<f64>();
    Residual { vt, coords: coords.to_vec() }
}

pub fn lyapunov_decrease(prev: &Residual, next: &Residual) -> bool {
    let all_safe = next.coords.iter().all(|rc| rc.value <= rc.bands.safe);
    all_safe || (next.vt <= prev.vt)
}
