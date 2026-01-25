// formate-pha-gate-2026/src/normalize_pha.rs
use crate::contracts::CorridorBands;
use crate::kernels::{to_rj, vt_from_coords};
use crate::contracts::RiskCoord;

#[derive(Clone, Debug)]
pub struct PhaSensors {
    pub formate_intake_mol: f64,
    pub acetyl_rate_mol_h: f64,
    pub pha_yield_g: f64,
    pub degrad_time_days: f64,
    pub tox_out_ppm: f64,
}

#[derive(Clone, Debug)]
pub struct PhaBands {
    pub formate: CorridorBands,  // safe=0.5, w=0.25
    pub acetyl: CorridorBands,
    pub pha: CorridorBands,
    pub degrad: CorridorBands,
    pub tox: CorridorBands,
}

#[derive(Clone, Debug)]
pub struct PhaRisk {
    pub r_formate: RiskCoord,
    pub r_acetyl: RiskCoord,
    pub r_pha: RiskCoord,
    pub r_degrad: RiskCoord,
    pub r_tox: RiskCoord,
    pub vt: Residual,
}

pub fn normalize_pha(s: &PhaSensors, bands: &PhaBands) -> PhaRisk {
    let r_formate = to_rj(s.formate_intake_mol, &bands.formate);
    let r_acetyl = to_rj(s.acetyl_rate_mol_h, &bands.acetyl);
    let r_pha = to_rj(s.pha_yield_g, &bands.pha);
    let r_degrad = to_rj(s.degrad_time_days, &bands.degrad);
    let r_tox = to_rj(s.tox_out_ppm, &bands.tox);
    let coords = [r_formate.clone(), r_acetyl.clone(), r_pha.clone(), r_degrad.clone(), r_tox.clone()];
    let vt = vt_from_coords(&coords);
    PhaRisk { r_formate, r_acetyl, r_pha, r_degrad, r_tox, vt }
}
