// File: crates/cyboquatic-energy-mass/src/lib.rs
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use cyboquatic_ecosafety_core::risk::{LyapunovWeights, Residual, RiskCoord, RiskVector};

/// Telemetry for a time interval [t0, t1].
/// Units: q_m3_s in m^3/s, cin/cout in mg/L, energy_kwh in kWh.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct IntervalSample {
    pub duration_s: f64,
    pub q_m3_s: f64,
    pub cin_mg_l: f64,
    pub cout_mg_l: f64,
    pub energy_kwh: f64,
}

/// Aggregated result for a pollutant over a time horizon.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EnergyMassResult {
    /// Total mass removed [kg].
    pub mass_removed_kg: f64,
    /// Total energy consumed [kWh].
    pub energy_kwh: f64,
    /// Specific energy [kWh/kg] (∞ if mass_removed_kg == 0).
    pub kwh_per_kg: f64,
}

/// Raw CEIM-style inputs over a window: Cin, Cout, Q, and energy.
/// This is a single-window equivalent of many IntervalSample entries.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EnergyMassRaw {
    /// Influent concentration [mg/L].
    pub c_in_mg_l: f64,
    /// Effluent concentration [mg/L].
    pub c_out_mg_l: f64,
    /// Flow rate [m^3/s].
    pub q_m3_s: f64,
    /// Duration [s].
    pub dt_s: f64,
    /// Electrical / mechanical energy consumed [kWh].
    pub energy_kwh: f64,
}

/// Corridor parameters for specific energy E_x [kWh/kg_removed].
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EnergyMassCorridor {
    /// Best / "safe" specific energy, e.g. 0.1 kWh/kg.
    pub safe_kwh_per_kg: f64,
    /// Gold band upper limit, e.g. 0.5 kWh/kg.
    pub gold_kwh_per_kg: f64,
    /// Hard upper limit, worst acceptable, e.g. 2.0 kWh/kg.
    pub hard_kwh_per_kg: f64,
}

/// Normalized score for a given window or interval bundle.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EnergyMassScore {
    /// Normalized risk coordinate r_energy,x in [0,1].
    pub r_energy: RiskCoord,
    /// Specific energy consumption [kWh/kg_removed].
    pub e_spec_kwh_per_kg: f64,
    /// True if within hard corridor.
    pub corridor_ok: bool,
    /// Mass removed [kg] over the window.
    pub mass_removed_kg: f64,
}

/// Integrate mass removed and energy used over a sequence of interval samples.
pub fn integrate_energy_mass(samples: &[IntervalSample]) -> EnergyMassResult {
    let mut total_mass_kg = 0.0;
    let mut total_energy_kwh = 0.0;

    for s in samples {
        if s.duration_s <= 0.0 || s.q_m3_s <= 0.0 {
            continue;
        }

        let vol_l = s.q_m3_s * s.duration_s * 1_000.0;
        let delta_c_mg_l = (s.cin_mg_l - s.cout_mg_l).max(0.0);
        let mass_removed_kg = delta_c_mg_l * vol_l * 1.0e-6;

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

/// Simple corridor for specific energy [kWh/kg] mapped to a normalized risk.
/// Example: safe <= 0.5, gold <= 1.0, hard limit 2.0.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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

    /// Map specific energy to r_energy in [0, 1] using a piecewise corridor.
    pub fn normalize(&self, kwh_per_kg: f64) -> RiskCoord {
        if !kwh_per_kg.is_finite() || kwh_per_kg <= 0.0 {
            return RiskCoord::new_clamped(1.0);
        }

        let x = kwh_per_kg;
        let r = if x <= self.safe_max_kwh_per_kg {
            0.0
        } else if x <= self.gold_max_kwh_per_kg {
            (x - self.safe_max_kwh_per_kg)
                / (self.gold_max_kwh_per_kg - self.safe_max_kwh_per_kg)
                * 0.4
        } else if x <= self.hard_max_kwh_per_kg {
            0.4
                + (x - self.gold_max_kwh_per_kg)
                    / (self.hard_max_kwh_per_kg - self.gold_max_kwh_per_kg)
                    * 0.5
        } else {
            0.9 + (x - self.hard_max_kwh_per_kg) / self.hard_max_kwh_per_kg * 0.1
        };

        RiskCoord::new_clamped(r)
    }
}

impl EnergyMassCorridor {
    /// Compute mass removed using CEIM-style approximation over dt:
    /// M_x ≈ (C_in - C_out) * Q * dt, converted to kg.
    fn mass_removed(raw: EnergyMassRaw) -> f64 {
        let delta_c_mg_l = (raw.c_in_mg_l - raw.c_out_mg_l).max(0.0);
        let volume_l = raw.q_m3_s * raw.dt_s * 1_000.0;
        let mass_mg = delta_c_mg_l * volume_l;
        mass_mg * 1.0e-6
    }

    /// Map E_spec into r_energy in [0,1] via a linear safe–hard corridor.
    pub fn score(&self, raw: EnergyMassRaw) -> EnergyMassScore {
        let m_removed = Self::mass_removed(raw);
        let e_spec = if m_removed > 0.0 {
            raw.energy_kwh / m_removed
        } else {
            self.hard_kwh_per_kg
        };

        let lo = self.safe_kwh_per_kg;
        let hi = self.hard_kwh_per_kg;

        let mut r = if e_spec <= lo {
            0.0
        } else if e_spec >= hi {
            1.0
        } else {
            (e_spec - lo) / (hi - lo)
        };

        r = r.max(0.0).min(1.0);
        let corridor_ok = e_spec <= self.hard_kwh_per_kg + 1e-9;

        EnergyMassScore {
            r_energy: RiskCoord::new_clamped(r),
            e_spec_kwh_per_kg: e_spec,
            corridor_ok,
            mass_removed_kg: m_removed,
        }
    }
}

/// Lift energy kernel into a full RiskVector, reusing other planes as-is.
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
    }
}

/// Evaluate residuals before/after an energy-efficiency change.
pub fn compare_modes(
    rv_base: &RiskVector,
    rv_improved: &RiskVector,
    weights: LyapunovWeights,
) -> (Residual, Residual) {
    let v0 = rv_base.residual(weights);
    let v1 = rv_improved.residual(weights);
    (Residual::new(v0), Residual::new(v1))
}
