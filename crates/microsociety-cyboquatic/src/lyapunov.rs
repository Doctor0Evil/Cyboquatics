use crate::state::{CyboquaticLattice, CyboquaticParams};
use crate::diagnostics::lyapunov_for_site;

/// Compute global Lyapunov value (e.g., sum over lattice).
pub fn global_lyapunov(lattice: &CyboquaticLattice) -> f64 {
    lattice
        .sites
        .iter()
        .map(|s| lyapunov_for_site(s))
        .sum()
}

/// Check whether a proposed lattice state respects V_{t+1} <= V_t.
pub fn lyapunov_is_non_increasing(
    prev_global_v: f64,
    lattice: &CyboquaticLattice,
    params: &CyboquaticParams,
) -> bool {
    let new_v = global_lyapunov(lattice);
    new_v <= prev_global_v + params.lyapunov_tolerance
}

/// Check whether all rx components remain below the configured ceiling.
pub fn all_risks_below_ceiling(
    lattice: &CyboquaticLattice,
    params: &CyboquaticParams,
) -> bool {
    let ceiling = params.safe_risk_ceiling;
    lattice.sites.iter().all(|s| {
        s.risk.rcec < ceiling
            && s.risk.rtox < ceiling
            && s.risk.rpathogen < ceiling
            && s.risk.rmicroplastics < ceiling
            && s.risk.rmetal < ceiling
            && s.risk.rerg < ceiling
    })
}
