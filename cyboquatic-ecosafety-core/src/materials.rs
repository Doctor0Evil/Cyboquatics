// filename: cyboquatic-ecosafety-core/src/materials.rs

use crate::{RiskCoord, RiskVector, CorridorBand, CorridorClass};

#[derive(Clone, Copy)]
pub struct MaterialKinetics {
    pub t90_days: f32,
    pub r_tox:    RiskCoord,
    pub r_micro:  RiskCoord,
    pub r_leach:  RiskCoord,
}

#[derive(Clone, Copy)]
pub struct MaterialCorridors {
    pub t90_max_days: f32,
    pub tox_band:     CorridorBand,
    pub micro_band:   CorridorBand,
    pub leach_band:   CorridorBand,
}

pub fn material_risk(
    kin: &MaterialKinetics,
    c: &MaterialCorridors,
    target_t90_days: f32,
) -> RiskVector {
    let mut rv = RiskVector::zero();

    let r_t90 = (kin.t90_days / c.t90_max_days).clamp(0.0, 1.0);
    rv.push(r_t90);
    rv.push(kin.r_tox);
    rv.push(kin.r_micro);
    rv.push(kin.r_leach);

    rv
}

/// Hard gate: only substrates whose kinetics and toxicity fit corridors may be used.
pub trait SafeSubstrate {
    fn kinetics(&self) -> MaterialKinetics;
    fn corridors(&self) -> MaterialCorridors;

    fn corridor_ok(&self, target_t90_days: f32) -> bool {
        let kin = self.kinetics();
        let cor = self.corridors();
        if kin.t90_days > cor.t90_max_days {
            return false;
        }
        let rv = material_risk(&kin, &cor, target_t90_days);
        let bands = [cor.tox_band, cor.micro_band, cor.leach_band];
        for i in 0..rv.len {
            if i >= bands.len() { break; }
            let band = &bands[i];
            if band.mandatory && matches!(band.classify(rv.coords[i]), CorridorClass::Breach) {
                return false;
            }
        }
        true
    }
}
