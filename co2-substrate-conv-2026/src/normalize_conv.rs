// co2-substrate-conv-2026/src/normalize_conv.rs
use crate::contracts::CorridorBands;
use crate::kernels::{to_rj, vt_from_coords};
use crate::contracts::RiskCoord;

#[derive(Clone, Debug)]
pub struct ConvSensors {
    pub co2_intake_tons: f64,
    pub formate_yield_mg: f64,
    pub acetyl_output_mol: f64,
    pub substrate_degrad_days: f64,
    pub tox_residual: f64,
}

#[derive(Clone, Debug)]
pub struct ConvBands {
    pub co2: CorridorBands,  // safe=200.0, w=0.3
    pub formate: CorridorBands,
    pub acetyl: CorridorBands,
    pub degrad: CorridorBands,
    pub tox: CorridorBands,
}

#[derive(Clone, Debug)]
pub struct ConvRisk {
    pub r_co2: RiskCoord,
    pub r_formate: RiskCoord,
    pub r_acetyl: RiskCoord,
    pub r_degrad: RiskCoord,
    pub r_tox: RiskCoord,
    pub vt: Residual,
}

pub fn normalize_conv(s: &ConvSensors, bands: &ConvBands) -> ConvRisk {
    let r_co2 = to_rj(s.co2_intake_tons, &bands.co2);
    let r_formate = to_rj(s.formate_yield_mg, &bands.formate);
    let r_acetyl = to_rj(s.acetyl_output_mol, &bands.acetyl);
    let r_degrad = to_rj(s.substrate_degrad_days, &bands.degrad);
    let r_tox = to_rj(s.tox_residual, &bands.tox);
    let coords = [r_co2.clone(), r_formate.clone(), r_acetyl.clone(), r_degrad.clone(), r_tox.clone()];
    let vt = vt_from_coords(&coords);
    ConvRisk { r_co2, r_formate, r_acetyl, r_degrad, r_tox, vt }
}
