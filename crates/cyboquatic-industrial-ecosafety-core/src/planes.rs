//! Risk planes specialized for Cyboquatic industrial machinery,
//! expressed as newtypes around the shared RiskCoord 0..=1.

use ecosafety_grammar::{RiskCoord, RiskVector};

/// Energy plane (kWh/kg removed, pump duty, grid intensity).
#[derive(Clone, Copy, Debug)]
pub struct EnergyRisk(pub RiskCoord);

/// Hydraulics plane (HLR, surcharge, overflow, FOG blockage).
#[derive(Clone, Copy, Debug)]
pub struct HydraulicsRisk(pub RiskCoord);

/// Biological/chemical plane (PFAS, E. coli, nutrients, SAT).
#[derive(Clone, Copy, Debug)]
pub struct BiologyRisk(pub RiskCoord);

/// Carbon plane (net CO2-eq per cycle; sequestration → 0).
#[derive(Clone, Copy, Debug)]
pub struct CarbonRisk(pub RiskCoord);

/// Materials plane (t90, leachate toxicity, micro-residue).
#[derive(Clone, Copy, Debug)]
pub struct MaterialsRisk(pub RiskCoord);

/// Canonical 5-plane risk vector for industrial Cyboquatics.
///
/// Order is fixed and must align with corridor and weight tables.
#[derive(Clone, Copy, Debug)]
pub struct IndustrialRiskVector {
    pub energy: EnergyRisk,
    pub hydraulics: HydraulicsRisk,
    pub biology: BiologyRisk,
    pub carbon: CarbonRisk,
    pub materials: MaterialsRisk,
}

impl IndustrialRiskVector {
    /// Convert to the shared RiskVector used by the Lyapunov kernel.
    pub fn as_universal(&self) -> RiskVector {
        RiskVector::from_array(&[
            (self.energy.0).into_inner(),
            (self.hydraulics.0).into_inner(),
            (self.biology.0).into_inner(),
            (self.carbon.0).into_inner(),
            (self.materials.0).into_inner(),
        ])
    }
}
