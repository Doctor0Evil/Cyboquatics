use crate::metrics::{DegradationKinetics, ToxicityProfile, MicroResidueProfile};

/// Phoenix-anchored corridors for biodegradable substrates.
///
/// Hard and gold bands are measured under Phoenix matrix conditions
/// (canal, compost, soil) and must be met before deployment.
pub const T90_HARD_DAYS: f32 = 180.0;
pub const T90_GOLD_DAYS: f32 = 120.0;
pub const RTOX_GOLD_MAX: f32 = 0.10;
pub const RMICRO_MAX: f32 = 0.05;
pub const CALORIC_DENSITY_MAX: f32 = 0.30;

/// Trait marking a substrate stack as "Ant-safe" and biodegradable.
///
/// Used for casings, liners, filter media, and internal structures.
pub trait AntSafeSubstrate {
    fn degradation(&self) -> DegradationKinetics;
    fn toxicity(&self) -> ToxicityProfile;
    fn micro_residue(&self) -> MicroResidueProfile;
    fn caloric_density(&self) -> f32;

    /// Check corridors; returns true only if all bands are satisfied.
    fn corridors_ok(&self) -> bool {
        let d = self.degradation();
        let t90 = d.t90_days();
        if t90 > T90_HARD_DAYS {
            return false;
        }
        let tox = self.toxicity().normalized_rtox();
        if tox > RTOX_GOLD_MAX {
            return false;
        }
        let rmicro = self.micro_residue().normalized_rmicro();
        if rmicro > RMICRO_MAX {
            return false;
        }
        let c = self.caloric_density();
        if c > CALORIC_DENSITY_MAX {
            return false;
        }
        true
    }
}

/// Trait enforcing that a substrate does not undermine pollutant removal.
///
/// For example, media used in PFAS removal must not leach PFAS.
pub trait CyboNodeCompatible {
    /// Returns true if the substrate is compatible with this node's
    /// pollutant targets (PFAS, pathogens, nutrients, etc.).
    fn compatible_with_targets(&self, targets: &NodeTargets) -> bool;
}

/// Pollutant targets for a Cyboquatic node.
#[derive(Clone, Copy, Debug)]
pub struct NodeTargets {
    pub pfas: bool,
    pub pathogens: bool,
    pub nutrients: bool,
    pub salinity: bool,
}
