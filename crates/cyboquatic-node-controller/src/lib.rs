// Filename: crates/cyboquatic-node-controller/src/lib.rs
// Role: Non‑actuating controller interface integrating all planes.[file:13]

#![forbid(unsafe_code)]
#![no_std]

use serde::{Deserialize, Serialize};

use cyboquatic_ecosafety_core::riskvector::{
    RiskCoord, RiskVector, LyapunovWeights, Residual,
};
use cyboquatic_ecosafety_core::safestep::{safestep, SafeDecision, SafeStepConfig};
use cyboquatic_carbon_kernel::{CarbonCorridor, CarbonRaw};
use cyboquatic_biodiversity_kernel::{BiodiversityCorridors, BiodiversityRaw};

/// Domain‑specific state metrics – replace with your existing CEIM structures.[file:12][file:13]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub renergy:       f64,
    pub rhydraulics:   f64,
    pub rbiology:      f64,
    pub rmaterials:    f64,
    pub rcalib:        f64,
    pub mass_c_kg:     f64,
    pub net_c_kg:      f64,
    pub energy_kwh:    f64,
    pub grid_kg_kwh:   f64,
    pub conn_idx:      f64,
    pub struct_complex: f64,
    pub colon_score:   f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NodeState {
    pub metrics: NodeMetrics,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NodeActuation {
    pub command_id: u32,
    // purely descriptive: the ecosafety kernel will decide whether this can apply.
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CyboquaticNodeController {
    pub carbon_corridor:   CarbonCorridor,
    pub biodiversity_corr: BiodiversityCorridors,
    pub weights:           LyapunovWeights,
}

impl CyboquaticNodeController {
    /// Propose a physical actuation and associated RiskVector (non‑actuating).[file:13]
    pub fn propose_step(
        &self,
        state: NodeState,
        prev_residual: Residual,
    ) -> (NodeActuation, RiskVector, Residual, SafeDecision) {
        let m = state.metrics;

        // 1. Base physical planes, filled from CEIM / hydraulics kernels.[file:12]
        let renergy     = RiskCoord::new_clamped(m.renergy);
        let rhydraulics = RiskCoord::new_clamped(m.rhydraulics);
        let rbiology    = RiskCoord::new_clamped(m.rbiology);
        let rmaterials  = RiskCoord::new_clamped(m.rmaterials);
        let rcalib      = RiskCoord::new_clamped(m.rcalib);

        // 2. Carbon plane.
        let raw_c = CarbonRaw {
            mass_processed_kg:  m.mass_c_kg,
            net_sequestered_kg: m.net_c_kg,
            energy_kwh:         m.energy_kwh,
        };
        let c_score = self.carbon_corridor.score(raw_c, m.grid_kg_kwh);

        // 3. Biodiversity plane.
        let raw_bio = BiodiversityRaw {
            connectivity_index:   m.conn_idx,
            structural_complexity: m.struct_complex,
            colonization_score:   m.colon_score,
        };
        let bio_score = self.biodiversity_corr.score(raw_bio);

        // 4. Compose full RiskVector.
        let rv = RiskVector {
            renergy,
            rhydraulics,
            rbiology,
            rcarbon:      c_score.rcarbon,
            rmaterials,
            rbiodiversity: bio_score.rbiodiversity,
            rcalib,
        };

        // 5. Compute next residual and apply safestep.[file:12]
        let next_res = rv.residual(self.weights);
        let decision = safestep(prev_residual, next_res, rv, SafeStepConfig::default());

        // 6. Physical actuation description (application fenced elsewhere).[file:12]
        let act = NodeActuation { command_id: 0 };

        (act, rv, next_res, decision)
    }
}
