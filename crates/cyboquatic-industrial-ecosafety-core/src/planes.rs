//! Risk planes specialized for Cyboquatic industrial machinery,
//! expressed as newtypes around the shared RiskCoord 0..=1.

use ecosafety_core::RiskCoord;

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
    pub energy: RiskCoord,
    pub hydraulics: RiskCoord,
    pub biology: RiskCoord,
    pub carbon: RiskCoord,
    pub materials: RiskCoord,
}

impl IndustrialRiskVector {
    /// Create a new industrial risk vector from raw coordinates.
    pub fn new(
        energy: f64,
        hydraulics: f64,
        biology: f64,
        carbon: f64,
        materials: f64,
    ) -> Self {
        Self {
            energy: RiskCoord::new_clamped(energy),
            hydraulics: RiskCoord::new_clamped(hydraulics),
            biology: RiskCoord::new_clamped(biology),
            carbon: RiskCoord::new_clamped(carbon),
            materials: RiskCoord::new_clamped(materials),
        }
    }
}
