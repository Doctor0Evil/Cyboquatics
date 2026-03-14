// Filename: cyboquatic-hydro-kernel/src/lib.rs

pub mod hydro;
pub mod slurry;
pub mod micro;
pub mod ecoscore;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydroSite {
    pub node_id: String,
    pub region: String,
    pub lat: f64,
    pub lon: f64,
    pub area_m2: f64,
    pub velocity_ms: f64,
    pub cp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydroResult {
    pub hydropower_kw: f64,
    pub energy_kwh_per_year: f64,
}

pub mod hydro {
    use super::{HydroSite, HydroResult};

    pub fn compute_hydropower(site: &HydroSite, hours_per_day: f64) -> HydroResult {
        let rho = 1000.0_f64;
        let p_kw = 0.5 * rho * site.area_m2 * site.velocity_ms.powi(3) * site.cp / 1000.0;
        let e_year = p_kw * hours_per_day * 365.0;
        HydroResult {
            hydropower_kw: p_kw,
            energy_kwh_per_year: e_year,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlurryConfig {
    pub velocity_ms: f64,
    pub diameter_m: f64,
}

pub mod slurry {
    use super::SlurryConfig;

    pub fn shear_rate_s_inv(cfg: &SlurryConfig) -> f64 {
        // γ_dot = 8 v / D
        8.0 * cfg.velocity_ms / cfg.diameter_m
    }
}

pub mod micro {
    use super::SlurryConfig;
    use crate::slurry::shear_rate_s_inv;

    #[derive(Debug, Clone)]
    pub struct MicroRisk {
        pub shear_s_inv: f64,
        pub r_micro: f64, // 0–1 normalized microplastic / fiber-break risk
    }

    pub fn micro_risk(cfg: &SlurryConfig) -> MicroRisk {
        let shear = shear_rate_s_inv(cfg);
        // Corridor: safe if shear <= 500 s^-1, linear penalty to 1 at 2000 s^-1
        let r = if shear <= 500.0 {
            0.0
        } else if shear >= 2000.0 {
            1.0
        } else {
            (shear - 500.0) / (2000.0 - 500.0)
        };
        MicroRisk {
            shear_s_inv: shear,
            r_micro: r,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoKernelInput {
    pub hydropower_kw: f64,
    pub grid_intensity_kgco2_per_kwh: f64,
    pub baseline_kwh_per_cycle: f64,
    pub cycles_per_year: f64,
    pub plastic_kg_avoided_per_cycle: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoKernelOutput {
    pub ecoimpact_score: f64,     // 0–1
    pub co2_tons_avoided_per_year: f64,
    pub plastic_tons_avoided_per_year: f64,
}

pub mod ecoscore {
    use super::{EcoKernelInput, EcoKernelOutput};

    pub fn eco_kernel(input: &EcoKernelInput) -> EcoKernelOutput {
        let energy_year = input.baseline_kwh_per_cycle * input.cycles_per_year;
        let co2_baseline = energy_year * input.grid_intensity_kgco2_per_kwh / 1000.0;
        let co2_with_hydro = 0.0_f64;
        let co2_avoided = (co2_baseline - co2_with_hydro).max(0.0);

        let plastic_tons = input.plastic_kg_avoided_per_cycle * input.cycles_per_year / 1000.0;

        // Simple normalized eco-impact: combine carbon and plastic avoided
        let co2_norm = (co2_avoided / 5.0).min(1.0);      // 5 tCO2/y → ~1
        let plastic_norm = (plastic_tons / 50.0).min(1.0); // 50 t/y → ~1
        let eco = 0.5 * co2_norm + 0.5 * plastic_norm;

        EcoKernelOutput {
            ecoimpact_score: eco,
            co2_tons_avoided_per_year: co2_avoided,
            plastic_tons_avoided_per_year: plastic_tons,
        }
    }
}

/// Hard eco-corridor gate: reject configs that are not carbon-negative & high-impact.
pub fn corridor_ok(output: &EcoKernelOutput) -> bool {
    output.co2_tons_avoided_per_year > 0.0 && output.ecoimpact_score >= 0.9
}
