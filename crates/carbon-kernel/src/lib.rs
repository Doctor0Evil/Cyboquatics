// Filename: crates/carbon-kernel/src/lib.rs

use serde::{Deserialize, Serialize};
use ecosafety_core::riskvector::RiskCoord;

/// Raw carbon/energy accounting over a window (cycle, hour, etc.).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonRaw {
    pub mass_processed_kg: f64,   // ∫ ṁ dt
    pub net_sequestered_kg: f64,  // positive = removed from atmosphere
    pub energy_kwh:        f64,   // ∫ P dt
}

/// Corridor parameters for net carbon performance, per technology / site.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonCorridor {
    /// Safe (strongly negative) target, e.g. -0.3 kgCO2e/kWh
    pub safe_kg_per_kwh: f64,
    /// Gold band (near carbon-neutral), e.g. -0.05 kgCO2e/kWh
    pub gold_kg_per_kwh: f64,
    /// Hard limit (worst acceptable), e.g. +0.05 kgCO2e/kWh
    pub hard_kg_per_kwh: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonScore {
    pub r_carbon:   RiskCoord,
    pub intensity:  f64,    // kgCO2e/kWh
    pub corridor_ok: bool,
}

impl CarbonCorridor {
    /// Normalize emissions intensity into r_carbon ∈ [0,1].
    ///
    /// safe … gold … hard forms a corridor where negative (sequestering)
    /// intensity maps near 0, near-neutral maps near ~0.5, and high positive
    /// intensity trends toward 1.
    pub fn score(&self, raw: CarbonRaw, grid_emissions_kg_per_kwh: f64) -> CarbonScore {
        // Effective net intensity, including energy’s own carbon.
        // If energy is from grid, add grid CO2e; if from verified renewables, grid term can be 0.
        let intensity = if raw.energy_kwh <= 0.0 {
            // No energy recorded: treat as unknown, pessimistically near hard.
            self.hard_kg_per_kwh
        } else {
            let gross_intensity = -raw.net_sequestered_kg / raw.energy_kwh;
            gross_intensity + grid_emissions_kg_per_kwh
        };

        // Piecewise-linear normalization over [safe, hard].
        let (lo, hi) = (self.safe_kg_per_kwh, self.hard_kg_per_kwh);
        let mut r = if intensity <= lo {
            0.0
        } else if intensity >= hi {
            1.0
        } else {
            (intensity - lo) / (hi - lo)
        };

        // Clamp for numerical safety.
        r = r.max(0.0).min(1.0);

        // Corridor-ok: within hard band.
        let corridor_ok = intensity <= self.hard_kg_per_kwh + 1e-9;

        CarbonScore {
            r_carbon: RiskCoord::new_clamped(r),
            intensity,
            corridor_ok,
        }
    }
}
