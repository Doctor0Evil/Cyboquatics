use serde::{Deserialize, Serialize};

pub type SiteIndex = usize;
pub type Tick = u64;

/// Pollution stocks and microbial activity at a marine cell.
/// All fields are normalized to [0, 1] or have explicit, documented units.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct PollutantState {
    pub pfas: f64,          // normalized PFAS stock (rCEC,PFAS channel)
    pub plastics: f64,      // normalized microplastic stock
    pub hydrocarbons: f64,  // normalized PAH / hydrocarbon stock
    pub metals: f64,        // normalized metal complex stock
}

/// Microbial and hydraulic state relevant for cyboquatic reactors.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct CyboBioState {
    pub pfna_defluor_rate: f64,   // PFAS defluorination (fraction per unit time)
    pub fluoride_release: f64,    // normalized fluoride release
    pub plastic_erosion: f64,     // fraction of plastic surface area lost
    pub pah_mineralization: f64,  // fraction of PAH removed
    pub metal_complex_dissoc: f64,// fraction of complexes dissociated
    pub eps_binding: f64,         // normalized EPS/metal binding
}

/// Hydraulic and energy variables; energy is strictly derived.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct HydroEnergyState {
    pub flow_rate: f64,      // Q(t), e.g. m^3/s, normalized by design max
    pub head: f64,           // ΔH(t), normalized by design max
    pub temp_gradient: f64,  // ΔT(t), normalized by design max
    pub eta_h: f64,          // microhydro efficiency
    pub eta_th: f64,         // thermal recovery efficiency
    pub p_h: f64,            // computed microhydro power (normalized)
    pub p_th: f64,           // computed thermal power (normalized)
}

/// Risk coordinates rx for a site, all dimensionless and typically [0, 1].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct RiskVector {
    pub rcec: f64,          // chemical exposure risk from PFAS etc.
    pub rtox: f64,          // aggregate ecotoxicological risk
    pub rpathogen: f64,     // pathogen / HGT risk
    pub rmicroplastics: f64,// microplastic risk
    pub rmetal: f64,        // metal-related risk
    pub rerg: f64,          // exposure–responsibility gap
}

/// Justice metrics aggregated per-site over an episode window.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct JusticeMetrics {
    pub hpcc: f64,          // habit–pollution / pollution–cleanup coupling
    pub erg: f64,           // exposure–responsibility gap (per site)
    pub tecr: f64,          // token-enforced collapse contribution
}

/// Cyboquatic extension for one marine / shoreline site.
/// This is intended to be embedded in the existing SiteState.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CyboquaticSite {
    pub pollutants: PollutantState,
    pub bio: CyboBioState,
    pub hydro: HydroEnergyState,
    pub risk: RiskVector,
    pub justice: JusticeMetrics,
    /// Local Lyapunov value at this site (harm functional).
    pub lyapunov_v: f64,
}

/// Global constraints and scaling factors for the shard.
/// These constants are part of the Tree-of-Life bark for cyboquatic reactors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyboquaticParams {
    pub max_pfas: f64,
    pub max_plastics: f64,
    pub max_hydrocarbons: f64,
    pub max_metals: f64,
    pub safe_risk_ceiling: f64,  // rx < 1 requirement
    pub max_p_h: f64,            // max allowed hydro power (normalized)
    pub max_p_th: f64,           // max allowed thermal power (normalized)
    pub lyapunov_tolerance: f64, // allowed numerical epsilon for V_{t+1} <= V_t
}

impl Default for CyboquaticParams {
    fn default() -> Self {
        Self {
            max_pfas: 1.0,
            max_plastics: 1.0,
            max_hydrocarbons: 1.0,
            max_metals: 1.0,
            safe_risk_ceiling: 1.0,
            max_p_h: 1.0,
            max_p_th: 1.0,
            lyapunov_tolerance: 1e-9,
        }
    }
}

/// Snapshot of all cyboquatic sites for a given world.
/// This hooks into the Jetson-Line lattice by index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyboquaticLattice {
    pub sites: Vec<CyboquaticSite>,
    pub params: CyboquaticParams,
}

impl CyboquaticLattice {
    pub fn new(len: usize, params: CyboquaticParams) -> Self {
        Self {
            sites: vec![CyboquaticSite::default(); len],
            params,
        }
    }

    pub fn len(&self) -> usize {
        self.sites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sites.is_empty()
    }
}
