// cyboquatic-eco-kernel/src/lib.rs

#![forbid(unsafe_code)]
#![deny(warnings)]

use cyboquatic_ecosafety_core::{CorridorBands, RiskCoord, RiskVector};

/// **Per-cycle carbon metrics** for a machine or line (kg CO₂e / cycle). [file:18]
#[derive(Clone, Copy, Debug)]
pub struct CarbonCycle {
    pub net_kg_co2e: f64,   // negative => net sequestration
}

/// **Biodegradation kinetics** for a substrate (ISO-14851 style t90 in days). [file:21][file:18]
#[derive(Clone, Copy, Debug)]
pub struct MaterialKinetics {
    pub t90_days: f64,
    pub residual_fraction: f64,
}

/// **Toxicity and micro-residue metrics** from LC-MS & ecotox labs. [file:21][file:18]
#[derive(Clone, Copy, Debug)]
pub struct MaterialToxicology {
    pub r_tox: f64,         // normalized 0–1 toxicity corridor r_tox. [file:21]
    pub r_micro: f64,       // normalized 0–1 micro-residue / microplastic risk. [file:21]
    pub r_cec: f64,         // normalized 0–1 leachate CEC / PFAS corridor. [file:18][file:21]
}

/// **Energy footprint** for one operating cycle. [file:21]
#[derive(Clone, Copy, Debug)]
pub struct EnergyCycle {
    pub grid_kwh: f64,
    pub hydro_kwh: f64, // canal / head-driven energy displacing grid power. [file:21]
}

impl EnergyCycle {
    pub fn net_grid_equiv(&self) -> f64 {
        (self.grid_kwh - self.hydro_kwh).max(0.0)
    }
}

/// Normalization kernels for **r_carbon** and **r_materials**. [file:18][file:21]
pub struct EcoKernelConfig {
    pub carbon_corridor: CorridorBands,
    pub t90_corridor: CorridorBands,
    pub energy_corridor: CorridorBands,
}

impl EcoKernelConfig {
    pub fn carbon_risk(&self, cc: &CarbonCycle) -> RiskCoord {
        self.carbon_corridor.normalize(cc.net_kg_co2e)
    }

    pub fn materials_risk(
        &self,
        kin: &MaterialKinetics,
        tox: &MaterialToxicology,
    ) -> RiskCoord {
        let r_t90 = self.t90_corridor.normalize(kin.t90_days);
        let w_t = 0.4_f64;
        let w_tox = 0.3_f64;
        let w_micro = 0.2_f64;
        let w_cec = 0.1_f64;

        let r = w_t * r_t90.value()
            + w_tox * tox.r_tox
            + w_micro * tox.r_micro
            + w_cec * tox.r_cec;
        RiskCoord::new_clamped(r)
    }

    pub fn energy_risk(&self, e: &EnergyCycle) -> RiskCoord {
        let net = self.energy_corridor.normalize(e.net_grid_equiv());
        net
    }
}

/// **Ant-safe substrate gate**: hard block bait-grade or slow/toxic recipes. [file:21][file:18]
pub fn ant_safe_substrate_ok(
    kin: &MaterialKinetics,
    tox: &MaterialToxicology,
    max_caloric_fraction: f64,
    caloric_fraction: f64,
) -> bool {
    let ant_safe_t90_max_days = 120.0; // gold band for Phoenix compost trays. [file:21]
    let r_tox_max = 0.10_f64;          // gold-band toxicity safety margin. [file:21]
    let r_micro_max = 0.05_f64;        // micro-residue bound. [file:21]

    kin.t90_days <= ant_safe_t90_max_days
        && tox.r_tox <= r_tox_max
        && tox.r_micro <= r_micro_max
        && caloric_fraction <= max_caloric_fraction
}

/// Build the **risk vector** slice for a tray line or Cyboquatic node. [file:18][file:21]
pub fn build_risk_vector(
    cfg: &EcoKernelConfig,
    carbon: &CarbonCycle,
    kin: &MaterialKinetics,
    tox: &MaterialToxicology,
    energy: &EnergyCycle,
    r_hydraulic: RiskCoord,
    r_bio: RiskCoord,
) -> RiskVector {
    let r_carbon = cfg.carbon_risk(carbon);
    let r_materials = cfg.materials_risk(kin, tox);
    let r_energy = cfg.energy_risk(energy);

    RiskVector {
        r_energy,
        r_hydraulic,
        r_bio,
        r_carbon,
        r_materials,
    }
}
