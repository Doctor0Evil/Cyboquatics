// air-globe-lyapunov-2026/src/normalize_air.rs
use crate::contracts::CorridorBands;
use crate::kernels::{to_rj, vt_from_coords};
use crate::contracts::RiskCoord;

#[derive(Clone, Debug)]
pub struct AirSensors {
    pub co2_tons_per_year: f64,  // Capture rate.
    pub pm_mg_m3: f64,
    pub nox_ppm: f64,
    pub energy_output_kw: f64,  // Tie-in efficiency.
    pub pollutant_residual: f64,  // Post-filter.
}

#[derive(Clone, Debug)]
pub struct AirGlobeBands {
    pub co2: CorridorBands,  // e.g., safe=200.0, gold=250.0, hard=300.0, w=0.4
    pub pm: CorridorBands,
    pub nox: CorridorBands,
    pub energy: CorridorBands,  // Inverted: higher better, normalize 1 - r.
    pub residual: CorridorBands,
}

#[derive(Clone, Debug)]
pub struct AirRisk {
    pub r_co2: RiskCoord,
    pub r_pm: RiskCoord,
    pub r_nox: RiskCoord,
    pub r_energy: RiskCoord,
    pub r_residual: RiskCoord,
    pub vt: Residual,
}

pub fn normalize_air(s: &AirSensors, bands: &AirGlobeBands) -> AirRisk {
    let r_co2 = to_rj(s.co2_tons_per_year, &bands.co2);
    let r_pm = to_rj(s.pm_mg_m3, &bands.pm);
    let r_nox = to_rj(s.nox_ppm, &bands.nox);
    let r_energy = to_rj(s.energy_output_kw, &bands.energy);  // Invert if needed: 1.0 - r.value for output metrics.
    let r_residual = to_rj(s.pollutant_residual, &bands.residual);
    let coords = [r_co2.clone(), r_pm.clone(), r_nox.clone(), r_energy.clone(), r_residual.clone()];
    let vt = vt_from_coords(&coords);
    AirRisk { r_co2, r_pm, r_nox, r_energy, r_residual, vt }
}
