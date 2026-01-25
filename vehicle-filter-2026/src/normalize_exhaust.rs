use crate::contracts::{CorridorBands, RiskCoord, Residual};
use crate::kernels::{residual_from_coords, to_rx};

#[derive(Clone, Debug)]
pub struct ExhaustSensors {
    pub pm_mass_mg_m3: f64,
    pub nox_ppm: f64,
    pub hc_ppm: f64,
    pub co_ppm: f64,
    pub backpressure_kpa: f64,
    pub substrate_temp_c: f64,
}

#[derive(Clone, Debug)]
pub struct VehicleFilterBands {
    pub pm: CorridorBands,
    pub nox: CorridorBands,
    pub hc: CorridorBands,
    pub co: CorridorBands,
    pub backpressure: CorridorBands,
    pub substrate_temp: CorridorBands,
}

#[derive(Clone, Debug)]
pub struct ExhaustRisk {
    pub r_pm: RiskCoord,
    pub r_nox: RiskCoord,
    pub r_hc: RiskCoord,
    pub r_co: RiskCoord,
    pub r_backpressure: RiskCoord,
    pub r_substrate_temp: RiskCoord,
    pub residual: Residual,
}

pub fn normalize_exhaust(s: &ExhaustSensors, bands: &VehicleFilterBands) -> ExhaustRisk {
    let r_pm = to_rx(s.pm_mass_mg_m3, &bands.pm);
    let r_nox = to_rx(s.nox_ppm, &bands.nox);
    let r_hc = to_rx(s.hc_ppm, &bands.hc);
    let r_co = to_rx(s.co_ppm, &bands.co);
    let r_backpressure = to_rx(s.backpressure_kpa, &bands.backpressure);
    let r_substrate_temp = to_rx(s.substrate_temp_c, &bands.substrate_temp);

    let residual = residual_from_coords(vec![
        r_pm.clone(),
        r_nox.clone(),
        r_hc.clone(),
        r_co.clone(),
        r_backpressure.clone(),
        r_substrate_temp.clone(),
    ]);

    ExhaustRisk {
        r_pm,
        r_nox,
        r_hc,
        r_co,
        r_backpressure,
        r_substrate_temp,
        residual,
    }
}
