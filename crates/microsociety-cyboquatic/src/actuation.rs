use crate::state::{CyboquaticLattice, CyboquaticSite};
use serde::{Deserialize, Serialize};

/// Local actuation proposal for microhydro / thermal modules.
/// This does not directly mutate state; governance must approve first.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnergyActuationProposal {
    pub site: usize,
    pub new_flow_rate: f64,
    pub new_head: f64,
    pub new_temp_gradient: f64,
}

/// Apply an already-approved actuation proposal to the cyboquatic lattice.
pub fn apply_energy_actuation(
    lattice: &mut CyboquaticLattice,
    proposal: &EnergyActuationProposal,
) {
    if proposal.site >= lattice.len() {
        return;
    }
    let site = &mut lattice.sites[proposal.site];

    site.hydro.flow_rate = proposal.new_flow_rate.clamp(0.0, 1.0);
    site.hydro.head = proposal.new_head.clamp(0.0, 1.0);
    site.hydro.temp_gradient = proposal.new_temp_gradient.clamp(0.0, 1.0);

    // Recompute derived powers under ecological constraints.
    let q = site.hydro.flow_rate;
    let h = site.hydro.head;
    let dt = site.hydro.temp_gradient;
    let eta_h = site.hydro.eta_h;
    let eta_th = site.hydro.eta_th;

    site.hydro.p_h = eta_h * q * h;
    site.hydro.p_th = eta_th * dt;
}

/// Convenience function to recompute P_h and P_th for all sites
/// without changing flow/head/gradient (e.g., after parameter updates).
pub fn recompute_energy_outputs(lattice: &mut CyboquaticLattice) {
    for site in &mut lattice.sites {
        let q = site.hydro.flow_rate;
        let h = site.hydro.head;
        let dt = site.hydro.temp_gradient;
        let eta_h = site.hydro.eta_h;
        let eta_th = site.hydro.eta_th;

        site.hydro.p_h = eta_h * q * h;
        site.hydro.p_th = eta_th * dt;
    }
}
