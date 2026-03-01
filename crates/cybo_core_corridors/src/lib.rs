// Purpose: Core Rust kernel for drain ecosafety corridors and residuals
// Note: No external deps beyond std to keep verification surface small.

#![forbid(unsafe_code)]

pub mod types {
    /// Corridor bands for a single metric.
    #[derive(Clone, Debug)]
    pub struct CorridorBands {
        pub id: String,        // e.g. "pfas_sum"
        pub unit: String,      // e.g. "µg/L"
        pub safe: f64,
        pub gold: f64,
        pub hard: f64,
        pub weight_w: f64,
        pub lyap_channel: String,
    }

    /// Normalized risk coordinate r_x ∈ [0,1] with uncertainty.
    #[derive(Clone, Debug)]
    pub struct RiskCoord {
        pub id: String,
        pub rx: f64,
        pub sigma: f64,
        pub bands: CorridorBands,
    }

    /// Lyapunov-style residual V_t.
    #[derive(Clone, Debug)]
    pub struct DrainResidual {
        pub vt: f64,
        pub coords: Vec<RiskCoord>,
    }

    /// Uncertainty residual U_t.
    #[derive(Clone, Debug)]
    pub struct UncertaintyResidual {
        pub ut: f64,
        pub coords: Vec<RiskCoord>,
    }

    /// Complete corridor table for a drain node.
    #[derive(Clone, Debug)]
    pub struct DrainCorridorTable {
        pub metrics: Vec<CorridorBands>,
    }

    /// Decision returned after evaluating a control step.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct SafeStepDecision {
        pub derate: bool,
        pub stop: bool,
    }
}

pub mod norm {
    use crate::types::{CorridorBands, RiskCoord};

    /// Normalize a raw measurement x into r_x ∈ [0,1] using corridor bands.
    /// Assumes safe ≤ gold ≤ hard and x ≥ 0.
    pub fn to_risk_coord(id: &str, x: f64, sigma: f64, bands: &CorridorBands) -> RiskCoord {
        let rx = if x <= bands.safe {
            0.0
        } else if x >= bands.hard {
            1.0
        } else {
            // Linear mapping from safe..hard → 0..1
            (x - bands.safe) / (bands.hard - bands.safe)
        };

        RiskCoord {
            id: id.to_string(),
            rx,
            sigma,
            bands: bands.clone(),
        }
    }
}

pub mod residual {
    use crate::types::{DrainResidual, RiskCoord, UncertaintyResidual};

    /// Compute Lyapunov-style residual V_t = Σ w_j * r_j.
    pub fn compute_residual(coords: &[RiskCoord]) -> DrainResidual {
        let mut vt = 0.0;
        for c in coords {
            vt += c.bands.weight_w * c.rx;
        }
        DrainResidual {
            vt,
            coords: coords.to_vec(),
        }
    }

    /// Compute uncertainty residual U_t = Σ w_j * σ_j.
    pub fn compute_uncertainty(coords: &[RiskCoord]) -> UncertaintyResidual {
        let mut ut = 0.0;
        for c in coords {
            ut += c.bands.weight_w * c.sigma;
        }
        UncertaintyResidual {
            ut,
            coords: coords.to_vec(),
        }
    }

    /// Lyapunov monotonicity outside a small safe interior.
    pub fn lyap_non_increasing(prev: &DrainResidual, next: &DrainResidual) -> bool {
        let eps = 0.01_f64;
        if prev.vt <= eps {
            next.vt <= prev.vt + eps
        } else {
            next.vt <= prev.vt
        }
    }

    /// Uncertainty monotonicity outside a small safe interior.
    pub fn unc_non_increasing(prev: &UncertaintyResidual, next: &UncertaintyResidual) -> bool {
        let eps = 0.01_f64;
        if prev.ut <= eps {
            next.ut <= prev.ut + eps
        } else {
            next.ut <= prev.ut
        }
    }
}

pub mod contract {
    use crate::residual::{lyap_non_increasing, unc_non_increasing};
    use crate::types::{
        CorridorBands, DrainCorridorTable, DrainResidual, SafeStepDecision, UncertaintyResidual,
    };

    /// Check that all required metrics are present with ordered bands.
    /// This is the Rust-side mirror of corridor_present().
    pub fn corridor_present(table: &DrainCorridorTable) -> bool {
        const REQUIRED: [&str; 8] = [
            "fog_load",
            "cod_effluent",
            "nutrients_total",
            "pfas_sum",
            "microplastics_total",
            "tss_effluent",
            "blockage_rate",
            "deforestation_risk",
        ];

        for r in REQUIRED.iter() {
            let mut found = false;
            for b in &table.metrics {
                if b.id == *r {
                    if !(b.safe <= b.gold && b.gold <= b.hard) {
                        return false;
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        true
    }

    /// Evaluate a control step for safety; returns derate/stop flags.
    pub fn safe_step(
        prev_res: &DrainResidual,
        prev_unc: &UncertaintyResidual,
        next_res: &DrainResidual,
        next_unc: &UncertaintyResidual,
    ) -> SafeStepDecision {
        // Any coordinate with rx > 1.0 → stop
        let any_hard_violation = next_res
            .coords
            .iter()
            .any(|c| c.rx > 1.0_f64);

        if any_hard_violation {
            return SafeStepDecision {
                derate: false,
                stop: true,
            };
        }

        let lyap_ok = lyap_non_increasing(prev_res, next_res);
        let unc_ok = unc_non_increasing(prev_unc, next_unc);

        if !lyap_ok || !unc_ok {
            return SafeStepDecision {
                derate: true,
                stop: false,
            };
        }

        SafeStepDecision {
            derate: false,
            stop: false,
        }
    }

    /// Helper to construct a DrainCorridorTable from bands.
    pub fn make_table(bands: Vec<CorridorBands>) -> DrainCorridorTable {
        DrainCorridorTable { metrics: bands }
    }
}

#[cfg(test)]
mod tests {
    use super::contract::{corridor_present, safe_step};
    use super::norm::to_risk_coord;
    use super::residual::{compute_residual, compute_uncertainty};
    use super::types::{CorridorBands, SafeStepDecision};

    fn bands(id: &str, safe: f64, gold: f64, hard: f64, w: f64) -> CorridorBands {
        CorridorBands {
            id: id.to_string(),
            unit: String::from("unit"),
            safe,
            gold,
            hard,
            weight_w: w,
            lyap_channel: String::from("chem"),
        }
    }

    #[test]
    fn test_corridor_present_ok() {
        let metrics = vec![
            bands("fog_load", 0.0, 0.5, 1.0, 1.0),
            bands("cod_effluent", 0.0, 0.5, 1.0, 1.0),
            bands("nutrients_total", 0.0, 0.5, 1.0, 1.0),
            bands("pfas_sum", 0.0, 0.5, 1.0, 2.0),
            bands("microplastics_total", 0.0, 0.5, 1.0, 2.0),
            bands("tss_effluent", 0.0, 0.5, 1.0, 1.0),
            bands("blockage_rate", 0.0, 0.5, 1.0, 1.0),
            bands("deforestation_risk", 0.0, 0.5, 1.0, 3.0),
        ];
        let table = crate::contract::make_table(metrics);
        assert!(corridor_present(&table));
    }

    #[test]
    fn test_safe_step_basic() {
        // Simple single-metric case
        let b = bands("pfas_sum", 0.0, 0.5, 1.0, 1.0);

        let c_prev = to_risk_coord("pfas_sum", 0.2, 0.01, &b);
        let c_next_better = to_risk_coord("pfas_sum", 0.1, 0.01, &b);

        let prev_res = compute_residual(&[c_prev.clone()]);
        let prev_unc = compute_uncertainty(&[c_prev.clone()]);

        let next_res = compute_residual(&[c_next_better.clone()]);
        let next_unc = compute_uncertainty(&[c_next_better]);

        let decision = safe_step(&prev_res, &prev_unc, &next_res, &next_unc);
        assert_eq!(
            decision,
            SafeStepDecision {
                derate: false,
                stop: false
            }
        );
    }
}
