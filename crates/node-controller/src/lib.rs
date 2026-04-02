// Filename: crates/node-controller/src/lib.rs

use ecosafety_core::riskvector::{RiskVector, RiskCoord, LyapunovWeights, Residual};
use ecosafety_core::safestep::{safestep, SafeDecision};
use carbon_kernel::{CarbonRaw, CarbonCorridor};
use biodiversity_kernel::{BiodiversityRaw, BiodiversityCorridors};
use ecosafety_core::traits::SafeController; // existing trait

pub struct CyboquaticNodeController {
    pub carbon_corridor:      CarbonCorridor,
    pub biodiversity_corr:    BiodiversityCorridors,
    pub lyap_weights:         LyapunovWeights,
}

impl SafeController for CyboquaticNodeController {
    type State     = NodeState;     // your existing plant state
    type Actuation = NodeActuation; // pumps/valves/etc.

    fn propose_step(
        &mut self,
        state: Self::State,
        prev_residual: Residual,
        weights: LyapunovWeights,
    ) -> (Self::Actuation, RiskVector, Residual) {
        // 1) Physical proposal (from existing CEIM-style logic).
        let (act, phys_rv) = self.propose_physical_step(state);

        // 2) Carbon plane.
        let carbon_raw = CarbonRaw {
            mass_processed_kg: state.metrics.mass_carbon_processed_kg,
            net_sequestered_kg: state.metrics.net_sequestered_carbon_kg,
            energy_kwh: state.metrics.energy_used_kwh,
        };
        let carbon_score = self.carbon_corridor.score(carbon_raw, state.metrics.grid_intensity_kg_per_kwh);

        // 3) Biodiversity plane (from geometry/material + habitat models).
        let bio_raw = BiodiversityRaw {
            connectivity_index:   state.habitat.connectivity_index,
            structural_complexity: state.habitat.structural_complexity,
            colonization_score:   state.habitat.colonization_score,
        };
        let bio_score = self.biodiversity_corr.score(bio_raw);

        // 4) Compose full RiskVector.
        let full_rv = RiskVector {
            renergy:       phys_rv.renergy,
            rhydraulics:   phys_rv.rhydraulics,
            rbiology:      phys_rv.rbiology,
            rcarbon:       carbon_score.r_carbon,
            rmaterials:    phys_rv.rmaterials,
            rbiodiversity: bio_score.r_biodiversity,
        };

        let vt_next = full_rv.residual(weights);
        let next_resid = Residual::new(vt_next);

        (act, full_rv, next_resid)
    }
}

// Routing / actuation shell:
pub fn route_and_actuate(
    ctrl: &mut CyboquaticNodeController,
    state: NodeState,
    prev_resid: Residual,
    weights: LyapunovWeights,
) -> (SafeDecision, Residual) {
    let (act, rv_next, resid_next) = ctrl.propose_step(state, prev_resid, weights);
    let decision = safestep(prev_resid, resid_next, rv_next, weights);

    match decision {
        SafeDecision::Accept => {
            // Apply actuation only if carbon & biodiversity also passed corridors.
            // If either corridor_ok was false, your ker_deployable / lane logic
            // should have already blocked deployment at CI.
            apply_actuation(act);
        }
        SafeDecision::Derate | SafeDecision::Stop => {
            // No actuation; optionally log and adjust planning.
        }
    }

    (decision, resid_next)
}
