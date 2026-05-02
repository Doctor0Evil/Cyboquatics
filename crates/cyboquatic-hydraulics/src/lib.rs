// crates/cyboquatic-hydraulics/src/lib.rs
use serde::{Deserialize, Serialize};
use cyboquatic_ecosafety_core::risk::{RiskCoord, RiskVector, LyapunovWeights, Residual};

/// Raw hydraulic state for a reach or node.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HydraulicRaw {
    pub pressure_bar: f64,
    pub flow_m3_s:    f64,
    pub temp_c:       f64,
    pub head_m:       f64,
}

/// Corridors for hydraulic safety and efficiency.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HydraulicCorridors {
    pub pressure_safe_max: f64,
    pub pressure_hard_max: f64,
    pub head_safe_max:     f64,
    pub head_hard_max:     f64,
    pub w_pressure:        f64,
    pub w_head:            f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HydraulicScore {
    pub r_hydraulics: RiskCoord,
    pub corridor_ok:  bool,
}

impl HydraulicCorridors {
    fn normalize_pos(value: f64, safe_max: f64, hard_max: f64) -> RiskCoord {
        let mut r = if value <= safe_max {
            0.0
        } else if value >= hard_max {
            1.0
        } else {
            (value - safe_max) / (hard_max - safe_max)
        };
        r = r.max(0.0).min(1.0);
        RiskCoord::new_clamped(r)
    }

    pub fn score(&self, raw: HydraulicRaw) -> HydraulicScore {
        let r_p = Self::normalize_pos(raw.pressure_bar, self.pressure_safe_max, self.pressure_hard_max);
        let r_h = Self::normalize_pos(raw.head_m,       self.head_safe_max,     self.head_hard_max);

        let wsum = self.w_pressure + self.w_head;
        let (wp, wh) = if wsum > 0.0 {
            (self.w_pressure / wsum, self.w_head / wsum)
        } else {
            (0.5, 0.5)
        };

        let r_sq = wp * r_p.value().powi(2) + wh * r_h.value().powi(2);
        let r_hyd = RiskCoord::new_clamped(r_sq.sqrt());

        let corridor_ok =
            raw.pressure_bar <= self.pressure_hard_max + 1e-9 &&
            raw.head_m       <= self.head_hard_max     + 1e-9;

        HydraulicScore {
            r_hydraulics: r_hyd,
            corridor_ok,
        }
    }
}

/// Example helper to fold subsystem scores into a RiskVector.
pub fn update_risk_vector(
    base: &RiskVector,
    hyd_score: HydraulicScore,
) -> RiskVector {
    RiskVector {
        r_hydraulics: hyd_score.r_hydraulics,
        ..*base
    }
}
