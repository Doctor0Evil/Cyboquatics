use crate::actuation::{apply_energy_actuation, EnergyActuationProposal};
use crate::lyapunov::{all_risks_below_ceiling, global_lyapunov, lyapunov_is_non_increasing};
use crate::state::{CyboquaticLattice, CyboquaticParams};
use serde::{Deserialize, Serialize};

/// Episode-level justice metrics used to compute TECR.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EpisodeJusticeSummary {
    pub erg_mean: f64,
    pub erg_max: f64,
    pub collapses: u32,
    pub ticks: u64,
}

/// Governance configuration for TECR and ERG clamping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceParams {
    pub max_tecr: f64,
    pub max_erg_mean: f64,
}

impl Default for GovernanceParams {
    fn default() -> Self {
        Self {
            max_tecr: 0.05,   // e.g. 5% collapse rate threshold
            max_erg_mean: 0.5,
        }
    }
}

/// Decision returned by the governance shard for a single actuation proposal.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GovernanceDecision {
    Allow,
    RejectRisk,       // violates rx < 1
    RejectLyapunov,   // violates V_{t+1} <= V_t
    RejectJustice,    // violates ERG / TECR constraints
}

/// Evaluate a single actuation proposal under Lyapunov, rx, ERG and TECR constraints.
/// The function is pure with respect to the input lattice; it uses a cloned state.
pub fn evaluate_actuation_proposal(
    lattice: &CyboquaticLattice,
    cybo_params: &CyboquaticParams,
    gov_params: &GovernanceParams,
    prev_global_v: f64,
    episode_justice: &EpisodeJusticeSummary,
    proposal: &EnergyActuationProposal,
) -> GovernanceDecision {
    // Reject immediately if justice metrics already exceed thresholds.
    let tecr = if episode_justice.ticks > 0 {
        episode_justice.collapses as f64 / episode_justice.ticks as f64
    } else {
        0.0
    };

    if tecr > gov_params.max_tecr || episode_justice.erg_mean > gov_params.max_erg_mean {
        return GovernanceDecision::RejectJustice;
    }

    // Clone lattice so we can simulate the effect locally.
    let mut trial = lattice.clone();

    // Apply actuation to trial state.
    apply_energy_actuation(&mut trial, proposal);

    // Enforce ecological/energy ceilings.
    for site in &mut trial.sites {
        site.hydro.p_h = site.hydro.p_h.min(cybo_params.max_p_h);
        site.hydro.p_th = site.hydro.p_th.min(cybo_params.max_p_th);
    }

    // Check risk ceiling.
    if !all_risks_below_ceiling(&trial, cybo_params) {
        return GovernanceDecision::RejectRisk;
    }

    // Check Lyapunov non-increase.
    if !lyapunov_is_non_increasing(prev_global_v, &trial, cybo_params) {
        return GovernanceDecision::RejectLyapunov;
    }

    GovernanceDecision::Allow
}
