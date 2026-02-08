#![forbid(unsafe_code)]

use std::fmt;

/// Normalized risk coordinate in [0, 1].
#[derive(Clone, Copy, Debug)]
pub struct RiskCoord {
    /// Identifier, e.g. "rSAT", "rPFAS", "rfouling", "rsurcharge", "rtox".
    pub var_id: &'static str,
    /// Current normalized value in [0, 1]; 0 = ideal, 1 = corridor edge.
    pub value: f64,
    /// Inner "gold" target band (stricter than legal limits).
    pub gold: f64,
    /// Hard corridor edge; must never be exceeded in actuation.
    pub hard: f64,
    /// Weight used when aggregating into Lyapunov-like residual V_t.
    pub weight: f64,
}

/// Violation residual V_t aggregating multiple risk coordinates.
#[derive(Clone, Copy, Debug)]
pub struct Residual {
    /// Scalar Lyapunov-like residual (lower is safer).
    pub vt: f64,
}

/// Decision returned by corridor checks and Lyapunov guard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorridorDecision {
    /// True when controller must derate (still allowed to operate, but in a safer,
    /// less aggressive mode, e.g., reduced flow or recharge).
    pub derate: bool,
    /// True when controller must stop this action entirely.
    pub stop: bool,
    /// Human-readable reason for logs and qpudatashards.
    pub reason: &'static str,
}

impl CorridorDecision {
    pub const OK: Self = Self {
        derate: false,
        stop: false,
        reason: "ok",
    };

    pub const fn derate(reason: &'static str) -> Self {
        Self {
            derate: true,
            stop: false,
            reason,
        }
    }

    pub const fn stop(reason: &'static str) -> Self {
        Self {
            derate: true,
            stop: true,
            reason,
        }
    }
}

impl fmt::Display for CorridorDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CorridorDecision {{ derate: {}, stop: {}, reason: {} }}",
            self.derate, self.stop, self.reason
        )
    }
}

/// Pilot metrics for a single cyboquatic module (vault / turbine / SAT cell).
/// These map directly onto the ecosafety grammar:
/// - hydraulic/structural: rsurcharge, Q, HLR
/// - treatment/SAT: rSAT, rPFAS, rCEC
/// - fouling/OM: rfouling
/// - plus an existing Lyapunov residual V_t.
#[derive(Clone, Debug)]
pub struct PilotMetrics {
    pub r_sat: RiskCoord,
    pub r_pfas: RiskCoord,
    pub r_cec: RiskCoord,
    pub r_surcharge: RiskCoord,
    pub r_fouling: RiskCoord,
    /// Previous residual V_t (global or node-local view).
    pub vt_prev: Residual,
    /// Candidate residual V_t for the proposed control step.
    pub vt_next: Residual,
}

/// Hard corridor check: "no corridor, no build" and edge detect.
///
/// - Returns stop if any risk coordinate exceeds its hard edge.
/// - Returns derate if any coordinate resides between gold and hard band.
/// - Returns OK only when all coordinates are inside gold bands.
pub fn check_corridors(metrics: &PilotMetrics) -> CorridorDecision {
    let coords = [
        metrics.r_sat,
        metrics.r_pfas,
        metrics.r_cec,
        metrics.r_surcharge,
        metrics.r_fouling,
    ];

    let mut any_derate = false;

    for rc in &coords {
        if rc.value > rc.hard {
            return CorridorDecision::stop(match rc.var_id {
                "rSAT" => "SAT corridor breached: rSAT > hard",
                "rPFAS" => "PFAS corridor breached: rPFAS > hard",
                "rCEC" => "CEC corridor breached: rCEC > hard",
                "rsurcharge" => "Hydraulic corridor breached: rsurcharge > hard",
                "rfouling" => "Fouling corridor breached: rfouling > hard",
                _ => "Unknown corridor breached: value > hard",
            });
        }

        if rc.value > rc.gold {
            any_derate = true;
        }
    }

    if any_derate {
        CorridorDecision::derate("inside legal corridor but outside gold band")
    } else {
        CorridorDecision::OK
    }
}

/// Lyapunov-like guard enforcing V_{t+1} <= V_t ("violated corridor, derate/stop").
///
/// - If V_{t+1} > V_t + eps, returns stop (unsafe trajectory).
/// - If V_{t+1} is slightly higher but within eps, returns derate.
/// - Otherwise returns OK.
pub fn check_lyapunov(prev: Residual, next: Residual, eps: f64) -> CorridorDecision {
    if next.vt > prev.vt + eps {
        CorridorDecision::stop("Lyapunov residual increased beyond tolerance")
    } else if next.vt > prev.vt {
        CorridorDecision::derate("Lyapunov residual slightly increased; derate required")
    } else {
        CorridorDecision::OK
    }
}

/// Composite guard: combines corridor checks with Lyapunov residual.
///
/// Enforcement policy:
/// - If either component decides stop, overall result is stop.
/// - Else if any component decides derate, overall result is derate.
/// - Else OK.
///
/// This is the core "ecosafety spine" entry point cyboquatic controllers should
/// call before any actuation: "no safe_step, no actuation".
pub fn safe_step(metrics: &PilotMetrics, eps_vt: f64) -> CorridorDecision {
    let cd_corr = check_corridors(metrics);
    if cd_corr.stop {
        return cd_corr;
    }

    let cd_v = check_lyapunov(metrics.vt_prev, metrics.vt_next, eps_vt);
    if cd_v.stop {
        return cd_v;
    }

    if cd_corr.derate || cd_v.derate {
        return CorridorDecision::derate("derate required by corridor and/or Lyapunov guard");
    }

    CorridorDecision::OK
}

/// Example wiring stub for GitHub integration: this is the function that
/// higher-level crates (controllers, schedulers) should depend on. In a full
/// system it would also:
/// - emit qpudatashards rows with K/E/R and violation_residual fields,
/// - attach DID/hex provenance, and
/// - be called from all cyboquatic actuation code paths.
pub fn evaluate_control_step(metrics: &PilotMetrics) -> CorridorDecision {
    // Typical epsilon for V_t non-increase can be tuned from Phoenix pilots;
    // here we use a small illustrative value.
    let eps_vt = 1e-6;
    safe_step(metrics, eps_vt)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_coord(id: &'static str, value: f64) -> RiskCoord {
        RiskCoord {
            var_id: id,
            value,
            gold: 0.5,
            hard: 1.0,
            weight: 1.0,
        }
    }

    #[test]
    fn all_inside_gold_is_ok() {
        let metrics = PilotMetrics {
            r_sat: mk_coord("rSAT", 0.2),
            r_pfas: mk_coord("rPFAS", 0.3),
            r_cec: mk_coord("rCEC", 0.1),
            r_surcharge: mk_coord("rsurcharge", 0.4),
            r_fouling: mk_coord("rfouling", 0.3),
            vt_prev: Residual { vt: 0.9 },
            vt_next: Residual { vt: 0.88 },
        };

        let d = evaluate_control_step(&metrics);
        assert_eq!(d, CorridorDecision::OK);
    }

    #[test]
    fn gold_to_hard_band_derates() {
        let metrics = PilotMetrics {
            r_sat: mk_coord("rSAT", 0.6),
            r_pfas: mk_coord("rPFAS", 0.3),
            r_cec: mk_coord("rCEC", 0.1),
            r_surcharge: mk_coord("rsurcharge", 0.4),
            r_fouling: mk_coord("rfouling", 0.3),
            vt_prev: Residual { vt: 0.9 },
            vt_next: Residual { vt: 0.89 },
        };

        let d = evaluate_control_step(&metrics);
        assert!(d.derate && !d.stop);
    }

    #[test]
    fn hard_breach_stops() {
        let metrics = PilotMetrics {
            r_sat: mk_coord("rSAT", 1.05),
            r_pfas: mk_coord("rPFAS", 0.3),
            r_cec: mk_coord("rCEC", 0.1),
            r_surcharge: mk_coord("rsurcharge", 0.4),
            r_fouling: mk_coord("rfouling", 0.3),
            vt_prev: Residual { vt: 0.9 },
            vt_next: Residual { vt: 0.88 },
        };

        let d = evaluate_control_step(&metrics);
        assert!(d.stop);
    }

    #[test]
    fn lyapunov_increase_stops() {
        let metrics = PilotMetrics {
            r_sat: mk_coord("rSAT", 0.2),
            r_pfas: mk_coord("rPFAS", 0.3),
            r_cec: mk_coord("rCEC", 0.1),
            r_surcharge: mk_coord("rsurcharge", 0.4),
            r_fouling: mk_coord("rfouling", 0.3),
            vt_prev: Residual { vt: 0.9 },
            vt_next: Residual { vt: 0.91 },
        };

        let d = evaluate_control_step(&metrics);
        assert!(d.stop);
    }
}
