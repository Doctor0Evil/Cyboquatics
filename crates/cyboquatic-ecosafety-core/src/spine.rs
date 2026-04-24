// crates/cyboquatic-ecosafety-core/src/spine.rs

#![forbid(unsafe_code)]

pub type RiskCoord = f64;

/// Shared risk coordinates for cyboquatic industrial machinery.
/// All are normalized to [0, 1] in corridor space.
#[derive(Clone, Copy, Debug, Default)]
pub struct RiskVector {
    /// Energy plane: intensity per unit useful work.
    pub r_energy: RiskCoord,
    /// Hydraulics plane: surcharge, HLR, SAT corridors.
    pub r_hydraulics: RiskCoord,
    /// Biology plane: pathogens, fouling, CEC biology.
    pub r_biology: RiskCoord,
    /// Carbon plane: net CO2‑e per cycle, corridor‑normalized.
    pub r_carbon: RiskCoord,
    /// Materials plane: t90, toxicity, micro‑residue, leachate.
    pub r_materials: RiskCoord,
}

impl RiskVector {
    pub fn max_coord(&self) -> RiskCoord {
        self.r_energy
            .max(self.r_hydraulics)
            .max(self.r_biology)
            .max(self.r_carbon)
            .max(self.r_materials)
    }
}

/// Lyapunov residual over the five planes.
/// This matches the existing rx–Vt–KER grammar used in Cyboquatic ecosafety docs. [file:23]
#[derive(Clone, Copy, Debug)]
pub struct Residual {
    pub v_t: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct LyapunovWeights {
    pub w_energy: f64,
    pub w_hydraulics: f64,
    pub w_biology: f64,
    pub w_carbon: f64,
    pub w_materials: f64,
}

impl LyapunovWeights {
    pub fn default_hazard_ordering() -> Self {
        // Up‑weight carbon and materials to force carbon‑negative,
        // fast‑degrading, non‑toxic substrates, consistent with your
        // carbon/material planes. [file:23]
        Self {
            w_energy: 1.0,
            w_hydraulics: 1.2,
            w_biology: 1.5,
            w_carbon: 1.5,
            w_materials: 1.5,
        }
    }
}

pub fn compute_residual(rv: &RiskVector, w: &LyapunovWeights) -> Residual {
    let v = w.w_energy * rv.r_energy.powi(2)
        + w.w_hydraulics * rv.r_hydraulics.powi(2)
        + w.w_biology * rv.r_biology.powi(2)
        + w.w_carbon * rv.r_carbon.powi(2)
        + w.w_materials * rv.r_materials.powi(2);
    Residual { v_t: v }
}

/// Decision for a proposed step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorridorDecision {
    Ok,
    Derate,
    Stop,
}

/// Static corridor bands shared across nodes; in practice these
/// are loaded from ALN / qpudatashard rows, not hard‑coded. [file:23]
#[derive(Clone, Copy, Debug)]
pub struct CorridorBands {
    /// Max allowed risk coordinate in normal operation (gold band).
    pub r_gold_max: f64,
    /// Hard stop threshold.
    pub r_hard_max: f64,
    /// Allowed Lyapunov non‑increase slack.
    pub dv_t_max: f64,
}

impl CorridorBands {
    pub fn eco_industrial_default() -> Self {
        // R_gold ~ 0.3, R_hard = 1.0, dv<=0 (no uncontrolled increase). [file:23]
        Self {
            r_gold_max: 0.3,
            r_hard_max: 1.0,
            dv_t_max: 0.0,
        }
    }
}

/// Enforce V_{t+1} <= V_t and per‑coordinate corridor bands.
/// This is the run‑time “safestep” gate used before any actuation. [file:23]
pub fn safestep(
    prev: &Residual,
    next: &Residual,
    rv_next: &RiskVector,
    bands: &CorridorBands,
) -> CorridorDecision {
    let dv = next.v_t - prev.v_t;

    let r_max = rv_next.max_coord();
    if r_max >= bands.r_hard_max || dv > bands.dv_t_max {
        CorridorDecision::Stop
    } else if r_max > bands.r_gold_max {
        CorridorDecision::Derate
    } else {
        CorridorDecision::Ok
    }
}

/// KER window metrics over a recent time window,
/// matching your governance triad. [file:23]
#[derive(Clone, Copy, Debug, Default)]
pub struct KerWindow {
    pub k_knowledge: f64,
    pub e_eco_impact: f64,
    pub r_risk_of_harm: f64,
}

impl KerWindow {
    /// Recompute E = 1 - R, ensure E, R in [0, 1].
    pub fn normalize(&mut self) {
        if self.r_risk_of_harm < 0.0 {
            self.r_risk_of_harm = 0.0;
        }
        if self.r_risk_of_harm > 1.0 {
            self.r_risk_of_harm = 1.0;
        }
        self.e_eco_impact = 1.0 - self.r_risk_of_harm;
        if self.k_knowledge > 1.0 {
            self.k_knowledge = 1.0;
        }
        if self.k_knowledge < 0.0 {
            self.k_knowledge = 0.0;
        }
    }

    /// Quick check for production‑lane eligibility using your 2026 band:
    /// K >= 0.90, E >= 0.90, R <= 0.13. [file:23][file:24]
    pub fn is_production_grade(&self) -> bool {
        self.k_knowledge >= 0.90 && self.e_eco_impact >= 0.90 && self.r_risk_of_harm <= 0.13
    }
}
