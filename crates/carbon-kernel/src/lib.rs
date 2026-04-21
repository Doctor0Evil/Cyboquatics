// Filename: crates/carbon-kernel/src/lib.rs

use serde::{Deserialize, Serialize};
use ecosafety_core::riskvector::RiskCoord;

/// Raw carbon/energy accounting over a window (cycle, hour, etc.).
///
/// Typical sources:
/// - integrated mass flow (∫ ṁ dt) for CO₂e streams,
/// - net sequestered mass from CEIM accounting,
/// - integrated power draw (∫ P dt) converted to kWh.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonRaw {
    /// Total mass of carbon processed over the window (kg CO₂e).
    pub mass_processed_kg:  f64,
    /// Net sequestered mass (kg CO₂e), positive if removed from atmosphere.
    pub net_sequestered_kg: f64,
    /// Total energy used over the window (kWh).
    pub energy_kwh:         f64,
}

/// Corridor parameters for net carbon performance, per technology / site.
///
/// Semantics:
/// - `safe_kg_per_kwh`  : strongly negative target (deeply carbon‑negative).
/// - `gold_kg_per_kwh`  : near‑neutral “gold” band center.
/// - `hard_kg_per_kwh`  : worst acceptable net intensity; beyond this, the
///                        configuration is considered corridor‑violating.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonCorridor {
    /// Safe (strongly negative) target, e.g. -0.30 kgCO₂e/kWh.
    pub safe_kg_per_kwh: f64,
    /// Gold band (near carbon‑neutral), e.g. -0.05 kgCO₂e/kWh.
    pub gold_kg_per_kwh: f64,
    /// Hard limit (worst acceptable), e.g. +0.05 kgCO₂e/kWh.
    pub hard_kg_per_kwh: f64,
}

/// Score for the carbon plane over a window.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CarbonScore {
    /// Normalized carbon risk coordinate r_carbon ∈ [0,1].
    pub r_carbon:    RiskCoord,
    /// Effective emissions intensity (kg CO₂e / kWh).
    pub intensity:   f64,
    /// True if intensity is within the configured hard band.
    pub corridor_ok: bool,
}

impl CarbonCorridor {
    /// Normalize emissions intensity into r_carbon ∈ [0,1].
    ///
    /// Interpretation:
    /// - intensity ≤ safe → r_carbon ≈ 0 (strongly carbon‑negative).
    /// - safe < intensity < hard → r_carbon scales linearly (0 → 1).
    /// - intensity ≥ hard → r_carbon ≈ 1 (unacceptable carbon performance).
    ///
    /// `grid_emissions_kg_per_kwh` allows the same kernel to be reused for
    /// grid‑supplied, renewable, or on‑site generation mixes:
    ///   - grid mix → positive value (kgCO₂e/kWh),
    ///   - verified renewables → often ~0.0.
    pub fn score(&self, raw: CarbonRaw, grid_emissions_kg_per_kwh: f64) -> CarbonScore {
        // Effective net intensity, including energy’s own carbon.
        // If no energy is recorded, treat the point as unknown and pessimistic.
        let intensity = if raw.energy_kwh <= 0.0 {
            self.hard_kg_per_kwh
        } else {
            // Gross intensity from the process (negative means net sequestration).
            let gross_intensity = -raw.net_sequestered_kg / raw.energy_kwh;
            // Add grid / supply‑side emissions.
            gross_intensity + grid_emissions_kg_per_kwh
        };

        // Piecewise‑linear normalization over [safe, hard].
        let (lo, hi) = (self.safe_kg_per_kwh, self.hard_kg_per_kwh);
        let mut r = if intensity <= lo {
            0.0
        } else if intensity >= hi {
            1.0
        } else {
            (intensity - lo) / (hi - lo)
        };

        // Clamp for numerical safety before constructing the RiskCoord.
        r = r.max(0.0).min(1.0);

        // Corridor-ok: intensity is at or below the hard limit (within band).
        let corridor_ok = intensity <= self.hard_kg_per_kwh + 1e-9;

        CarbonScore {
            r_carbon: RiskCoord::new_clamped(r),
            intensity,
            corridor_ok,
        }
    }
}
