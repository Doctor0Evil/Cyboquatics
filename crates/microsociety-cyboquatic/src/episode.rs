use crate::state::{CyboquaticLattice, CyboquaticSite};
use crate::governance::EpisodeJusticeSummary;
use serde::{Deserialize, Serialize};

/// Per-tick snapshot of selected metrics for replayable Episodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeTickRecord {
    pub tick: u64,
    pub global_v: f64,
    pub mean_rcec: f64,
    pub mean_rtox: f64,
    pub mean_rerg: f64,
    pub mean_p_h: f64,
    pub mean_p_th: f64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EpisodeLog {
    pub ticks: Vec<EpisodeTickRecord>,
    pub justice_summary: EpisodeJusticeSummary,
}

impl EpisodeLog {
    pub fn record_tick(
        &mut self,
        tick: u64,
        lattice: &CyboquaticLattice,
        global_v: f64,
    ) {
        let n = lattice.len().max(1) as f64;
        let mut sum_rcec = 0.0;
        let mut sum_rtox = 0.0;
        let mut sum_rerg = 0.0;
        let mut sum_ph = 0.0;
        let mut sum_pth = 0.0;

        for s in &lattice.sites {
            sum_rcec += s.risk.rcec;
            sum_rtox += s.risk.rtox;
            sum_rerg += s.risk.rerg;
            sum_ph += s.hydro.p_h;
            sum_pth += s.hydro.p_th;
        }

        self.ticks.push(EpisodeTickRecord {
            tick,
            global_v,
            mean_rcec: sum_rcec / n,
            mean_rtox: sum_rtox / n,
            mean_rerg: sum_rerg / n,
            mean_p_h: sum_ph / n,
            mean_p_th: sum_pth / n,
        });

        self.justice_summary.ticks = tick + 1;
    }

    /// Update ERG and collapse counts after an Episode run.
    pub fn finalize_from_lattice(&mut self, lattice: &CyboquaticLattice, collapses: u32) {
        let n = lattice.len().max(1) as f64;
        let mut sum_erg = 0.0;
        let mut max_erg = 0.0;

        for s in &lattice.sites {
            sum_erg += s.risk.rerg;
            if s.risk.rerg > max_erg {
                max_erg = s.risk.rerg;
            }
        }

        self.justice_summary.erg_mean = sum_erg / n;
        self.justice_summary.erg_max = max_erg;
        self.justice_summary.collapses = collapses;
    }
}
