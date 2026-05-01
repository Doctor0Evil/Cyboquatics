// File: crates/cyboquatic-energy-mass/src/lib.rs

#![forbid(unsafe_code)]

use cyboquatic_ecosafety_core::{LyapunovWeights, Residual, RiskCoord, RiskVector};

/// Telemetry for a time interval [t0, t1].
/// Units: Q_m3s in m^3/s, Cin/Cout in mg/L, energy_kwh in kWh.
#[derive(Clone, Copy, Debug)]
pub struct IntervalSample {
    pub duration_s: f64,
    pub q_m3s: f64,
    pub cin_mg_l: f64,
    pub cout_mg_l: f64,
    pub energy_kwh: f64,
}

/// Aggregated result for a pollutant over a time horizon.
#[derive(Clone, Copy, Debug)]
pub struct EnergyMassResult {
    /// Total mass removed [kg].
    pub mass_removed_kg: f64,
    /// Total energy consumed [kWh].
    pub energy_kwh: f64,
    /// Specific energy [kWh/kg] if mass_removed_kg > 0.
    pub kwh_per_kg: f64,
}

/// Integrate mass removed and energy used.
pub fn integrate_energy_mass(samples: &[IntervalSample]) -> EnergyMassResult {
    let mut total_mass_kg = 0.0;
    let mut total_energy_kwh = 0.0;
    for s in samples {
        if s.duration_s <= 0.0 || s.q_m3s <= 0.0 {
            continue;
        }
        // Volume [m^3] -> [L].
        let vol_l = s.q_m3s * s.duration_s * 1_000.0;
        let delta_c_mg_l = (s.cin_mg_l - s.cout_mg_l).max(0.0);
        // Mass removed [mg] -> [kg].
        let mass_removed_kg = delta_c_mg_l * vol_l / 1_000_000_000.0;
        total_mass_kg += mass_removed_kg;
        total_energy_kwh += s.energy_kwh.max(0.0);
    }
    let kwh_per_kg = if total_mass_kg > 0.0 {
        total_energy_kwh / total_mass_kg
    } else {
        f64::INFINITY
    };
    EnergyMassResult {
        mass_removed_kg: total_mass_kg,
        energy_kwh: total_energy_kwh,
        kwh_per_kg,
    }
}

/// Corridor for specific energy [kWh/kg] mapped to a normalized risk.
/// Example: safe <= 0.5, gold <= 1.0, hard limit 2.0.
#[derive(Clone, Copy, Debug)]
pub struct EnergyCorridor {
    pub safe_max_kwh_per_kg: f64,
    pub gold_max_kwh_per_kg: f64,
    pub hard_max_kwh_per_kg: f64,
}

impl EnergyCorridor {
    pub fn phoenix_pfbs_default() -> Self {
        Self {
            safe_max_kwh_per_kg: 0.5,
            gold_max_kwh_per_kg: 1.0,
            hard_max_kwh_per_kg: 2.0,
        }
    }

    /// Map specific energy to r_energy in [0, 1].
    pub fn normalize(&self, kwh_per_kg: f64) -> RiskCoord {
        if !kwh_per_kg.is_finite() || kwh_per_kg <= 0.0 {
            return RiskCoord::new(1.0);
        }
        let x = kwh_per_kg;
        let r = if x <= self.safe_max_kwh_per_kg {
            0.0
        } else if x <= self.gold_max_kwh_per_kg {
            (x - self.safe_max_kwh_per_kg)
                / (self.gold_max_kwh_per_kg - self.safe_max_kwh_per_kg)
                * 0.4
        } else if x <= self.hard_max_kwh_per_kg {
            0.4 + (x - self.gold_max_kwh_per_kg)
                / (self.hard_max_kwh_per_kg - self.gold_max_kwh_per_kg)
                * 0.5
        } else {
            0.9 + (x - self.hard_max_kwh_per_kg)
                / (self.hard_max_kwh_per_kg)
                * 0.1
        };
        RiskCoord::new(r)
    }
}

/// Lift energy kernel into a full RiskVector, reusing residual weights.
/// Callers fill in other planes (hydraulics, biology, carbon, materials, biodiversity, sigma).
pub fn update_risk_vector_with_energy(
    rv_current: &RiskVector,
    energy_risk: RiskCoord,
) -> RiskVector {
    RiskVector {
        r_energy: energy_risk,
        r_hydraulics: rv_current.r_hydraulics,
        r_biology: rv_current.r_biology,
        r_carbon: rv_current.r_carbon,
        r_materials: rv_current.r_materials,
        r_biodiversity: rv_current.r_biodiversity,
        r_sigma: rv_current.r_sigma,
    }
}

/// Example utility: evaluate residual before/after improved energy efficiency.
pub fn compare_modes(
    rv_base: &RiskVector,
    rv_improved: &RiskVector,
    weights: LyapunovWeights,
) -> (Residual, Residual) {
    let r0 = weights.evaluate(rv_base);
    let r1 = weights.evaluate(rv_improved);
    (r0, r1)
}
