//! Helpers for writing qpudatashard rows for material stacks.
//! These structs are designed to serialize directly into
//! CyboquaticMaterialLinkPhoenix2026v1.aln-compatible CSV.

use crate::traits::{AntSafeSubstrate, CyboNodeCompatible, NodeTargets};

/// Governance-ready record for a single material stack.
///
/// This does not perform I/O itself; higher layers will turn this
/// into CSV/ALN rows and attach hex-stamped evidence.
#[derive(Clone, Debug)]
pub struct MaterialShardRecord {
    pub material_id: u64,
    pub label: [u8; 32],
    pub t90_days: f32,
    pub rtox: f32,
    pub rmicro: f32,
    pub caloric_density: f32,
    pub ecoimpact_score: f32,
    pub corridors_ok: bool,
    pub compatible_pfas: bool,
    pub compatible_pathogens: bool,
    pub compatible_nutrients: bool,
    pub compatible_salinity: bool,
}

impl MaterialShardRecord {
    pub fn from_substrate<S: AntSafeSubstrate + CyboNodeCompatible>(
        material_id: u64,
        label: [u8; 32],
        substrate: &S,
        targets: &NodeTargets,
    ) -> Self {
        let d = substrate.degradation();
        let t90 = d.t90_days();
        let tox = substrate.toxicity().normalized_rtox();
        let rm = substrate.micro_residue().normalized_rmicro();
        let c = substrate.caloric_density();

        // Simple eco-impact proxy; full version can reuse your
        // existing ecoimpactscore kernel.
        let ecoimpact = {
            let fast_decay = (T90_HARD_DAYS - t90).max(0.0) / T90_HARD_DAYS;
            let low_tox = 1.0 - tox;
            let low_micro = 1.0 - rm;
            let low_cal = 1.0 - c;
            0.25 * (fast_decay + low_tox + low_micro + low_cal)
        };

        let corridors_ok = substrate.corridors_ok();
        let compat = substrate.compatible_with_targets(targets);

        Self {
            material_id,
            label,
            t90_days: t90,
            rtox: tox,
            rmicro: rm,
            caloric_density: c,
            ecoimpact_score: ecoimpact,
            corridors_ok,
            compatible_pfas: compat && targets.pfas,
            compatible_pathogens: compat && targets.pathogens,
            compatible_nutrients: compat && targets.nutrients,
            compatible_salinity: compat && targets.salinity,
        }
    }
}
