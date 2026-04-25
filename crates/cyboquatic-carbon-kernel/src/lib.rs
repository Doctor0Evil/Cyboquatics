// Filename: crates/cyboquatic-carbon-kernel/src/lib.rs
// Role: Non‑actuating rcarbon kernel for carbon‑negative performance.[file:13]

#![forbid(unsafe_code)]
#![no_std]

use serde::{Deserialize, Serialize};
use cyboquatic_ecosafety_core::riskvector::RiskCoord;

/// Raw carbon/energy accounting over a window.[file:13]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonRaw {
    /// Total mass of carbon processed (kg CO2‑equivalent).
    pub mass_processed_kg:  f64,
    /// Net sequestered mass (kg CO2‑equivalent, positive means removed).
    pub net_sequestered_kg: f64,
    /// Electrical/mechanical energy used (kWh).
    pub energy_kwh:         f64,
}

/// Corridor parameters for net carbon intensity (kg CO2e/kWh).[file:13]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonCorridor {
    /// Strongly negative (best) e.g. ‑0.30 kgCO2e/kWh.
    pub safe_kg_per_kwh: f64,
    /// Near‑neutral gold band e.g. ‑0.05 kgCO2e/kWh.
    pub gold_kg_per_kwh: f64,
    /// Worst acceptable intensity e.g. 0.05 kgCO2e/kWh.
    pub hard_kg_per_kwh: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonScore {
    pub rcarbon:      RiskCoord,
    pub intensity:    f64,   // kgCO2e/kWh
    pub corridor_ok:  bool,  // inside hard band
}

impl CarbonCorridor {
    /// Map net intensity into rcarbon ∈ [0,1] using corridor bands.[file:13]
    pub fn score(&self, raw: CarbonRaw, grid_emissions_kg_per_kwh: f64) -> CarbonScore {
        let intensity = if raw.energy_kwh <= 0.0 {
            // Pessimistic: unknown energy use maps near hard bound.[file:13]
            self.hard_kg_per_kwh
        } else {
            let gross = -raw.net_sequestered_kg / raw.energy_kwh;
            gross + grid_emissions_kg_per_kwh
        };

        let lo = self.safe_kg_per_kwh;
        let hi = self.hard_kg_per_kwh;

        let r = if intensity <= lo {
            0.0
        } else if intensity >= hi {
            1.0
        } else {
            (intensity - lo) / (hi - lo).max(1.0e-9)
        };

        let rcarbon = RiskCoord::new_clamped(r);
        let corridor_ok = intensity <= self.hard_kg_per_kwh + 1.0e-9;

        CarbonScore { rcarbon, intensity, corridor_ok }
    }
}
