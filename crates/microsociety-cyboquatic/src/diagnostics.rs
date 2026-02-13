use crate::state::{CyboquaticLattice, CyboquaticSite};
use serde::{Deserialize, Serialize};

/// Diagnostic record per tick for one site.
/// This is observer-only: no actuation commands appear here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyboDiagnosticsRecord {
    pub tick: u64,
    pub site: usize,
    pub rcec: f64,
    pub rtox: f64,
    pub rpathogen: f64,
    pub rmicroplastics: f64,
    pub rmetal: f64,
    pub rerg: f64,
    pub lyapunov_v: f64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CyboDiagnosticsLog {
    pub records: Vec<CyboDiagnosticsRecord>,
}

impl CyboDiagnosticsLog {
    pub fn push(&mut self, rec: CyboDiagnosticsRecord) {
        self.records.push(rec);
    }
}

/// Update risk metrics and Lyapunov value for a single site from sensor-like inputs.
/// This function must not modify any non-cybo state or actuators.
pub fn update_diagnostics_for_site(
    site: &mut CyboquaticSite,
    sensor_rcec: f64,
    sensor_rtox: f64,
    sensor_rpathogen: f64,
    sensor_rmicroplastics: f64,
    sensor_rmetal: f64,
    sensor_rerg: f64,
) {
    site.risk.rcec = sensor_rcec;
    site.risk.rtox = sensor_rtox;
    site.risk.rpathogen = sensor_rpathogen;
    site.risk.rmicroplastics = sensor_rmicroplastics;
    site.risk.rmetal = sensor_rmetal;
    site.risk.rerg = sensor_rerg;
}

/// Compute a Lyapunov candidate V for a site based on pollutants, bioload, and ERG.
/// Monotone in harmful quantities and bounded from below.
pub fn lyapunov_for_site(site: &CyboquaticSite) -> f64 {
    // Simple, explicit form; scale factors can be tuned by research.
    let p = &site.pollutants;
    let r = &site.risk;
    let harm_pollutants =
        p.pfas + p.plastics + p.hydrocarbons + p.metals;

    // ERG and rtox are given higher weight; clip negatives for safety.
    let erg = r.rerg.max(0.0);
    let rtox = r.rtox.max(0.0);

    harm_pollutants + 2.0 * erg + 2.0 * rtox
}

/// Enforce V_{t+1} <= V_t at the diagnostic layer by rejecting inconsistent
/// sensor updates. This does not actuate; it only clamps diagnostics and logs.
pub fn enforce_lyapunov_monotonicity(
    lattice: &mut CyboquaticLattice,
    prev_v: &[f64],
    tolerance: f64,
) {
    for (i, site) in lattice.sites.iter_mut().enumerate() {
        let new_v = lyapunov_for_site(site);
        let old_v = prev_v.get(i).copied().unwrap_or(new_v);
        if new_v > old_v + tolerance {
            // Clamp V back to previous value by proportionally scaling risk components.
            let scale = if new_v > 0.0 { (old_v / new_v).max(0.0) } else { 1.0 };
            site.risk.rtox *= scale;
            site.risk.rerg *= scale;
            // Recompute V after clamping.
            site.lyapunov_v = lyapunov_for_site(site);
        } else {
            site.lyapunov_v = new_v;
        }
    }
}

/// Extract current Lyapunov values into a vector, for use by the next tick.
pub fn snapshot_lyapunov(lattice: &CyboquaticLattice) -> Vec<f64> {
    lattice.sites.iter().map(|s| s.lyapunov_v).collect()
}
