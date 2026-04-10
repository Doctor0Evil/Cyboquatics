use ecosafety_core::types::RiskVector;

use crate::MaterialRisks;

/// Marker trait enforced at compile/load time for safe substrates.
///
/// Implementations are typically derived or generated from MaterialRisks
/// and corridor checks in CI.
pub trait AntSafeSubstrateCorridorOk {
    fn material_risks(&self) -> &MaterialRisks;
}

/// Helper to inject r_materials into the global RiskVector.
pub fn with_materials_plane(
    mut rv: RiskVector,
    risks: &MaterialRisks,
) -> RiskVector {
    rv.r_materials = risks.r_materials;
    rv
}
