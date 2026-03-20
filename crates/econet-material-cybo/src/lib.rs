#![forbid(unsafe_code)]

use cyboquatic_ecosafety_core::{CorridorBands, RiskCoord, RiskVector};

/// Raw kinetics + ecotoxicology measured under Phoenix-class conditions. [file:11]
#[derive(Clone, Debug)]
pub struct MaterialKinetics {
    pub t90_days: f64,
    pub r_tox: f64,
    pub r_micro: f64,
    pub r_leach_cec: f64,
    pub r_pfas_resid: f64,
    pub caloric_density: f64, // dimensionless 0–1 (baiting risk surrogate)
}

/// Normalized material-plane risk coordinates feeding V_t. [file:11][file:3]
#[derive(Clone, Debug)]
pub struct MaterialRisks {
    pub r_t90: RiskCoord,
    pub r_tox: RiskCoord,
    pub r_micro: RiskCoord,
    pub r_leach_cec: RiskCoord,
    pub r_pfas_resid: RiskCoord,
    pub r_bait: RiskCoord,
}

impl MaterialRisks {
    pub fn from_kinetics(
        kin: &MaterialKinetics,
        corr_t90: &CorridorBands,
        corr_tox: &CorridorBands,
        corr_micro: &CorridorBands,
        corr_leach: &CorridorBands,
        corr_pfas: &CorridorBands,
        corr_bait: &CorridorBands,
    ) -> Self {
        Self {
            r_t90: corr_t90.normalize(kin.t90_days),
            r_tox: corr_tox.normalize(kin.r_tox),
            r_micro: corr_micro.normalize(kin.r_micro),
            r_leach_cec: corr_leach.normalize(kin.r_leach_cec),
            r_pfas_resid: corr_pfas.normalize(kin.r_pfas_resid),
            r_bait: corr_bait.normalize(kin.caloric_density),
        }
    }

    /// Composite r_materials for the Lyapunov plane. [file:3][file:11]
    pub fn r_materials(&self, weights: [f64; 6]) -> RiskCoord {
        let coords = [
            self.r_t90,
            self.r_tox,
            self.r_micro,
            self.r_leach_cec,
            self.r_pfas_resid,
            self.r_bait,
        ];
        let mut num = 0.0;
        let mut den = 0.0;
        for (w, r) in weights.iter().zip(coords.iter()) {
            num += w * r.value();
            den += w;
        }
        RiskCoord::new(if den > 0.0 { num / den } else { 0.0 })
    }
}

/// Hard gate for substrates (AntSafeSubstrate). [file:11]
pub trait AntSafeSubstrate {
    fn corridor_ok(&self) -> bool;
    fn risks(&self) -> &MaterialRisks;
}

/// Example Phoenix defaults: t90 ≤ 180 d hard, ≤ 120 d gold; r_tox ≤ 0.10; r_micro ≤ 0.05. [file:11]
#[derive(Clone, Debug)]
pub struct PhoenixMaterialCorridors {
    pub corr_t90: CorridorBands,
    pub corr_tox: CorridorBands,
    pub corr_micro: CorridorBands,
    pub corr_leach: CorridorBands,
    pub corr_pfas: CorridorBands,
    pub corr_bait: CorridorBands,
}

impl Default for PhoenixMaterialCorridors {
    fn default() -> Self {
        Self {
            corr_t90: CorridorBands {
                safe_min: 0.0,
                safe_max: 90.0,
                gold_min: 90.0,
                gold_max: 120.0,
                hard_min: 0.0,
                hard_max: 180.0,
            },
            corr_tox: CorridorBands {
                safe_min: 0.0,
                safe_max: 0.05,
                gold_min: 0.05,
                gold_max: 0.10,
                hard_min: 0.0,
                hard_max: 0.20,
            },
            corr_micro: CorridorBands {
                safe_min: 0.0,
                safe_max: 0.02,
                gold_min: 0.02,
                gold_max: 0.05,
                hard_min: 0.0,
                hard_max: 0.10,
            },
            corr_leach: CorridorBands {
                safe_min: 0.0,
                safe_max: 0.05,
                gold_min: 0.05,
                gold_max: 0.10,
                hard_min: 0.0,
                hard_max: 0.20,
            },
            corr_pfas: CorridorBands {
                safe_min: 0.0,
                safe_max: 0.01,
                gold_min: 0.01,
                gold_max: 0.03,
                hard_min: 0.0,
                hard_max: 0.05,
            },
            corr_bait: CorridorBands {
                safe_min: 0.0,
                safe_max: 0.10,
                gold_min: 0.10,
                gold_max: 0.30,
                hard_min: 0.0,
                hard_max: 0.50,
            },
        }
    }
}

/// Concrete biodegradable stack implementing AntSafeSubstrate. [file:11]
#[derive(Clone, Debug)]
pub struct SubstrateStack {
    pub name: String,
    pub kinetics: MaterialKinetics,
    pub corridors: PhoenixMaterialCorridors,
    pub risks: MaterialRisks,
}

impl SubstrateStack {
    pub fn new(name: impl Into<String>, kin: MaterialKinetics) -> Self {
        let corridors = PhoenixMaterialCorridors::default();
        let risks = MaterialRisks::from_kinetics(
            &kin,
            &corridors.corr_t90,
            &corridors.corr_tox,
            &corridors.corr_micro,
            &corridors.corr_leach,
            &corridors.corr_pfas,
            &corridors.corr_bait,
        );
        Self {
            name: name.into(),
            kinetics: kin,
            corridors,
            risks,
        }
    }
}

impl AntSafeSubstrate for SubstrateStack {
    fn corridor_ok(&self) -> bool {
        let k = &self.kinetics;
        let c = &self.corridors;

        // Hard gates reflecting ISO/OECD-style biodegradation and toxicity bands. [file:11]
        if !c.corr_t90.within_hard(k.t90_days) {
            return false;
        }
        if k.r_tox > c.corr_tox.gold_max {
            return false;
        }
        if k.r_micro > c.corr_micro.gold_max {
            return false;
        }
        if k.r_leach_cec > c.corr_leach.gold_max {
            return false;
        }
        if k.r_pfas_resid > c.corr_pfas.gold_max {
            return false;
        }
        if k.caloric_density > c.corr_bait.gold_max {
            return false;
        }
        true
    }

    fn risks(&self) -> &MaterialRisks {
        &self.risks
    }
}

/// Helper: lift material risks into the ecosafety RiskVector’s materials plane. [file:3][file:11]
pub fn material_plane_from_substrate(
    base: &mut RiskVector,
    substrate: &impl AntSafeSubstrate,
    weights: [f64; 6],
) {
    let rm = substrate.risks().r_materials(weights);
    base.r_materials = rm;
}
