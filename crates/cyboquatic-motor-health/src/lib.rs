// crates/cyboquatic-motor-health/src/lib.rs
use serde::{Deserialize, Serialize};
use cyboquatic_ecosafety_core::risk::RiskCoord;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MotorRaw {
    pub voltage_v:      f64,
    pub current_a:      f64,
    pub torque_nm:      f64,
    pub speed_rpm:      f64,
    pub mech_power_kw:  f64,
    pub elec_power_kw:  f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MotorCorridors {
    pub eff_safe_min: f64,
    pub eff_hard_min: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MotorScore {
    pub r_degradation: RiskCoord,
    pub efficiency:    f64,
    pub corridor_ok:   bool,
}

impl MotorCorridors {
    pub fn score(&self, raw: MotorRaw) -> MotorScore {
        let efficiency = if raw.elec_power_kw > 0.0 {
            (raw.mech_power_kw / raw.elec_power_kw).max(0.0).min(1.2)
        } else {
            0.0
        };

        // Higher efficiency → lower risk.
        let mut r = if efficiency >= self.eff_safe_min {
            0.0
        } else if efficiency <= self.eff_hard_min {
            1.0
        } else {
            (self.eff_safe_min - efficiency) / (self.eff_safe_min - self.eff_hard_min)
        };
        r = r.max(0.0).min(1.0);

        let corridor_ok = efficiency >= self.eff_hard_min - 1e-9;

        MotorScore {
            r_degradation: RiskCoord::new_clamped(r),
            efficiency,
            corridor_ok,
        }
    }
}
