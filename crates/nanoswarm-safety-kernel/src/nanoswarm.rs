#![no_std]

use crate::types::{CorridorBands, RiskCoord, Residual, to_r_linear};
use crate::contracts::CorridorDecision;

/// Minimal environmental inputs for canal‑adjacent nanoswarm ops.
/// These map directly to Lifeforce5DVoxel indices in your geo shard.
#[derive(Clone, Copy)]
pub struct NanoswarmEnvInputs {
    pub tdi: f32,             // Toxin Density Index
    pub mbi: f32,             // Macro‑biome Index
    pub eco_impact_score: f32,
    pub radiation_index: f32,
}

/// High‑level, non‑hardware control intent for a swarm + MAR corridor.
/// No direct motor / actuator outputs live here.
#[derive(Clone, Copy)]
pub struct NanoswarmControlIntent {
    /// 0–1 scalar for per‑voxel cleanup duty (fraction of maximum allowed dose)
    pub cleanup_duty: f32,
    /// 0–1 throttling of movement into higher‑risk voxels
    pub advance_fraction: f32,
}

/// Trait a nanoswarm ecosafety kernel must implement.
/// Implementations hide corridor math and Lyapunov residual details.
pub trait NanoswarmSafetyKernel {
    fn check_step(
        &self,
        env: NanoswarmEnvInputs,
        proposed: NanoswarmControlIntent,
        prev: Residual,
    ) -> (CorridorDecision, Residual);
}

/// Static corridor bands for Phoenix‑canal 2026 pilot.
/// These should be generated from qpudatashards/geo/NanoswarmSitePhoenixCanal2026v1.csv.
static TDI_BANDS: CorridorBands = CorridorBands {
    varid: "TDI",
    units: "dimensionless",
    safe: 0.3,
    gold: 0.5,
    hard: 1.0,
    weight: 0.35,
    lyap_channel: 0,
    mandatory: true,
};

static MBI_BANDS: CorridorBands = CorridorBands {
    varid: "MBI",
    units: "dimensionless",
    safe: 0.2,   // lower is worse: normalization handled upstream
    gold: 0.4,
    hard: 1.0,
    weight: 0.25,
    lyap_channel: 1,
    mandatory: true,
};

static EIS_BANDS: CorridorBands = CorridorBands {
    varid: "EcoImpactScore",
    units: "dimensionless",
    safe: 0.4,
    gold: 0.7,
    hard: 1.0,
    weight: 0.25,
    lyap_channel: 2,
    mandatory: true,
};

static RAD_BANDS: CorridorBands = CorridorBands {
    varid: "RadiationIndex",
    units: "dimensionless",
    safe: 0.1,
    gold: 0.3,
    hard: 1.0,
    weight: 0.15,
    lyap_channel: 3,
    mandatory: true,
};

/// Default kernel for Phoenix‑canal nanoswarm cleanup.
/// All it does is convert Lifeforce5DVoxel‑like indices into RiskCoord,
/// recompute V_t, and apply the global safestep gate.
pub struct DefaultNanoswarmSafetyKernel;

/// NOTE: this implementation is intentionally conservative:
/// higher TDI / RadiationIndex → higher risk; lower MBI / EcoImpactScore
/// can be normalized upstream before calling into this kernel.
impl NanoswarmSafetyKernel for DefaultNanoswarmSafetyKernel {
    fn check_step(
        &self,
        env: NanoswarmEnvInputs,
        proposed: NanoswarmControlIntent,
        mut prev: Residual,
    ) -> (CorridorDecision, Residual) {
        // Map raw indices to normalized risk coordinates.
        // Here we assume env.* are already scaled into [0, 1] corridors;
        // any additional scaling lives in your voxel → kernel adapter.
        static mut COORDS_BUF: [RiskCoord; 4] = [
            RiskCoord { r: 0.0, sigma: 0.0, bands: &TDI_BANDS },
            RiskCoord { r: 0.0, sigma: 0.0, bands: &MBI_BANDS },
            RiskCoord { r: 0.0, sigma: 0.0, bands: &EIS_BANDS },
            RiskCoord { r: 0.0, sigma: 0.0, bands: &RAD_BANDS },
        ];

        // SAFETY: single‑threaded, nostd embedded context.
        let coords = unsafe { &mut COORDS_BUF };

        coords[0] = to_r_linear(env.tdi as f64, &TDI_BANDS);
        coords[1] = to_r_linear(env.mbi as f64, &MBI_BANDS);
        coords[2] = to_r_linear(env.eco_impact_score as f64, &EIS_BANDS);
        coords[3] = to_r_linear(env.radiation_index as f64, &RAD_BANDS);

        let next = Residual {
            vt: 0.0,
            coords,
        };

        let mut next_mut = next;
        next_mut.recompute();

        let decision = crate::contracts::safestep(&prev, &next_mut);

        // The control intent is not applied here; controllers must treat
        // CorridorDecision::Derate / ::Stop as hard gates.
        (decision, next_mut)
    }
}
