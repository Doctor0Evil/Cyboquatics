// File: crates/econet_material_cybo/src/lib.rs

#![no_std]

use cyboquatic_ecosafety_core::RiskCoord;

/// Raw kinetic and ecotox data for a biodegradable material candidate.
#[derive(Clone, Copy)]
pub struct MaterialKinetics {
    pub t90_days: f32,        // time to 90% mass loss under Phoenix conditions
    pub r_tox: RiskCoord,     // normalized ecotoxicity (0 = benign, 1 = severe)
    pub r_micro: RiskCoord,   // normalized micro-residue risk
    pub r_leach_cec: RiskCoord,   // normalized leachate CEC risk
    pub r_pfas_resid: RiskCoord,  // normalized PFAS-like residual risk
    pub caloric_density: f32,     // dimensionless [0,1], baiting risk proxy
}

/// Trait: only substrates that satisfy all safety corridors may be instantiated.
pub trait AntSafeSubstrate {
    fn kinetics(&self) -> &MaterialKinetics;

    /// Hard-coded Phoenix corridors; implementations may be more strict, but not looser.
    fn corridors_ok(&self) -> bool {
        let k = self.kinetics();

        let t90_hard_max = 180.0_f32;
        let t90_gold_max = 120.0_f32;

        let t90_ok = k.t90_days <= t90_hard_max;
        let t90_gold = k.t90_days <= t90_gold_max;

        let rtox_gold_max: RiskCoord = 0.10;
        let rmicro_max: RiskCoord = 0.05;
        let r_leach_cec_max: RiskCoord = 0.10;
        let r_pfas_resid_max: RiskCoord = 0.10;
        let caloric_max: f32 = 0.30;

        let tox_ok = k.r_tox <= rtox_gold_max;
        let micro_ok = k.r_micro <= rmicro_max;
        let leach_ok = k.r_leach_cec <= r_leach_cec_max;
        let pfas_ok = k.r_pfas_resid <= r_pfas_resid_max;
        let caloric_ok = k.caloric_density <= caloric_max;

        t90_ok && tox_ok && micro_ok && leach_ok && pfas_ok && caloric_ok && t90_gold
    }
}

/// Trait: substrate must not undermine node’s ecological function (no back-leach of targeted pollutants).
pub trait CyboNodeCompatible {
    fn is_compatible_with_node(&self, node_class: NodeClass) -> bool;
}

/// Example classification for Cyboquatic nodes.
#[derive(Clone, Copy)]
pub enum NodeClass {
    MARVault,
    CanalPurifier,
    FlowVacFOG,
    Wetland,
}

/// Composite trait: only materials that are safe and node-compatible may appear in a machine.
pub trait CyboMaterial: AntSafeSubstrate + CyboNodeCompatible {}

impl<T> CyboMaterial for T where T: AntSafeSubstrate + CyboNodeCompatible {}

/// Compute a scalar eco-impact score for ranking materials (0 = poor, 1 = excellent).
pub fn eco_impact_score(k: &MaterialKinetics) -> f32 {
    let t90_score = (180.0_f32 - k.t90_days).max(0.0) / 180.0;
    let tox_score = 1.0 - k.r_tox;
    let micro_score = 1.0 - k.r_micro;
    let leach_score = 1.0 - k.r_leach_cec;
    let pfas_score = 1.0 - k.r_pfas_resid;
    let caloric_score = 1.0 - k.caloric_density;

    let sum = t90_score + tox_score + micro_score + leach_score + pfas_score + caloric_score;
    sum / 6.0
}
