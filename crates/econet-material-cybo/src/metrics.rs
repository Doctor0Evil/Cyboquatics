//! Minimal metric structs and kernels for material corridors.

/// First-order or Monod kinetics parameters used to derive t90.
#[derive(Clone, Copy, Debug)]
pub struct DegradationKinetics {
    pub k_day_inv: f32, // effective rate constant in 1/day
}

impl DegradationKinetics {
    pub fn t90_days(&self) -> f32 {
        // t90 = ln(10)/k
        const LN_10: f32 = 2.302585093;
        if self.k_day_inv <= 0.0 {
            f32::INFINITY
        } else {
            LN_10 / self.k_day_inv
        }
    }
}

/// Toxicity profile derived from LC-MS and bioassays,
/// normalized to a dimensionless rtox 0..1.
#[derive(Clone, Copy, Debug)]
pub struct ToxicityProfile {
    pub rtox: f32,
}

impl ToxicityProfile {
    pub fn normalized_rtox(&self) -> f32 {
        self.rtox.clamp(0.0, 1.0)
    }
}

/// Micro-residue risk profile: probability of micro-fragment
/// formation under corridor shear and environmental conditions.
#[derive(Clone, Copy, Debug)]
pub struct MicroResidueProfile {
    pub rmicro: f32,
}

impl MicroResidueProfile {
    pub fn normalized_rmicro(&self) -> f32 {
        self.rmicro.clamp(0.0, 1.0)
    }
}
