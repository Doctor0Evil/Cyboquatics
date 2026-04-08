//! crates/cyboquatic_ecosafety_core/src/lib.rs  (v2 — rcalib extension)
//!
//! Extended ecosafety spine with r_calib as a 6th risk plane.
//! All original rx/Vt/KER semantics preserved; rcalib is additive.
//! Belonging-to: Cyboquatic / ecosafety
//! Non-actuating. #![forbid(unsafe_code)]

#![forbid(unsafe_code)]
#![no_std]

pub type RiskCoord = f32;
pub type Residual = f32;

#[derive(Clone, Copy, Debug)]
pub struct CorridorBands {
    pub safe_max: RiskCoord,
    pub gold_max: RiskCoord,
    pub hard_max: RiskCoord,
}

impl CorridorBands {
    pub fn normalize(&self, raw: f32, raw_safe_max: f32, raw_hard_max: f32) -> RiskCoord {
        if raw_hard_max <= raw_safe_max {
            return 1.0;
        }
        let span = raw_hard_max - raw_safe_max;
        let clamped = if raw <= raw_safe_max {
            0.0
        } else if raw >= raw_hard_max {
            1.0
        } else {
            (raw - raw_safe_max) / span
        };
        clamped.min(1.0).max(0.0)
    }

    pub fn in_hard_violation(&self, r: RiskCoord) -> bool {
        r > self.hard_max
    }
}

/// v2: RiskVector now includes r_calib as the 6th plane.
#[derive(Clone, Copy, Debug, Default)]
pub struct RiskVector {
    pub r_energy: RiskCoord,
    pub r_hydraulic: RiskCoord,
    pub r_biology: RiskCoord,
    pub r_carbon: RiskCoord,
    pub r_materials: RiskCoord,
    pub r_calib: RiskCoord,   // NEW: ingest/schema quality risk
}

impl RiskVector {
    /// Max across all 6 planes including r_calib.
    pub fn max_coord(&self) -> RiskCoord {
        self.r_energy
            .max(self.r_hydraulic)
            .max(self.r_biology)
            .max(self.r_carbon)
            .max(self.r_materials)
            .max(self.r_calib)
    }

    /// Max across original 5 physical planes (excluding r_calib).
    pub fn max_physical(&self) -> RiskCoord {
        self.r_energy
            .max(self.r_hydraulic)
            .max(self.r_biology)
            .max(self.r_carbon)
            .max(self.r_materials)
    }

    /// True if r_calib alone would block deployment.
    pub fn calib_blocks_deploy(&self, hard_band: RiskCoord) -> bool {
        self.r_calib > hard_band
    }
}

/// v2: LyapunovWeights now includes w_calib.
#[derive(Clone, Copy, Debug)]
pub struct LyapunovWeights {
    pub w_energy: f32,
    pub w_hydraulic: f32,
    pub w_biology: f32,
    pub w_carbon: f32,
    pub w_materials: f32,
    pub w_calib: f32,   // NEW: weight for ingest/schema risk
}

impl LyapunovWeights {
    pub fn normalized() -> Self {
        Self {
            w_energy: 1.0,
            w_hydraulic: 1.0,
            w_biology: 1.2,
            w_carbon: 1.3,
            w_materials: 1.1,
            w_calib: 0.8,   // lower than physical planes, but non-zero
        }
    }
}

/// v2: Vt now includes w_calib * r_calib^2.
pub fn compute_residual(r: &RiskVector, w: &LyapunovWeights) -> Residual {
    w.w_energy * r.r_energy * r.r_energy
        + w.w_hydraulic * r.r_hydraulic * r.r_hydraulic
        + w.w_biology * r.r_biology * r.r_biology
        + w.w_carbon * r.r_carbon * r.r_carbon
        + w.w_materials * r.r_materials * r.r_materials
        + w.w_calib * r.r_calib * r.r_calib   // NEW term
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafeStepDecision {
    Accept,
    Derate,
    Stop,
}

#[derive(Clone, Copy, Debug)]
pub struct SafeStepGateConfig {
    pub epsilon: f32,
    pub max_risk_allowed: RiskCoord,
}

impl SafeStepGateConfig {
    pub fn default_research_band() -> Self {
        Self {
            epsilon: 1.0e-4,
            max_risk_allowed: 0.13,
        }
    }
}

/// v2: safestep now considers r_calib via max_coord().
pub fn eval_safestep(
    previous_v: Residual,
    next_v: Residual,
    risk: &RiskVector,
    config: &SafeStepGateConfig,
) -> SafeStepDecision {
    let r_max = risk.max_coord(); // includes r_calib

    if r_max > config.max_risk_allowed {
        return SafeStepDecision::Stop;
    }

    if next_v <= previous_v + config.epsilon {
        SafeStepDecision::Accept
    } else {
        SafeStepDecision::Derate
    }
}

/// v2: KerWindow tracks rcalib-aware risk.
#[derive(Clone, Copy, Debug, Default)]
pub struct KerWindow {
    pub steps: u32,
    pub lyapunov_safe_steps: u32,
    pub max_risk_observed: RiskCoord,
    pub max_rcalib_observed: RiskCoord,  // NEW: track rcalib peak separately
}

impl KerWindow {
    pub fn update(&mut self, risk: &RiskVector, lyapunov_safe: bool) {
        self.steps = self.steps.saturating_add(1);
        if lyapunov_safe {
            self.lyapunov_safe_steps = self.lyapunov_safe_steps.saturating_add(1);
        }
        let r_max = risk.max_coord();
        if r_max > self.max_risk_observed {
            self.max_risk_observed = r_max;
        }
        if risk.r_calib > self.max_rcalib_observed {
            self.max_rcalib_observed = risk.r_calib;
        }
    }

    pub fn knowledge_factor(&self) -> f32 {
        if self.steps == 0 {
            0.0
        } else {
            self.lyapunov_safe_steps as f32 / self.steps as f32
        }
    }

    pub fn eco_impact(&self) -> f32 {
        1.0 - self.max_risk_observed
    }

    pub fn risk_of_harm(&self) -> f32 {
        self.max_risk_observed
    }

    /// NEW: Can rcalib alone block kerdeployable?
    pub fn calib_risk_of_harm(&self) -> f32 {
        self.max_rcalib_observed
    }
}

// ─── kerdeployable gate (v2) ───────────────────────────────────

/// v2: KerDeployable configuration with rcalib thresholds.
#[derive(Clone, Copy, Debug)]
pub struct KerDeployableConfig {
    pub k_min: f32,
    pub e_min: f32,
    pub r_max: f32,
    pub rcalib_hard: f32,  // NEW: independent rcalib hard gate
}

impl KerDeployableConfig {
    pub fn production() -> Self {
        Self {
            k_min: 0.90,
            e_min: 0.90,
            r_max: 0.13,
            rcalib_hard: 0.13,
        }
    }
}

/// v2: Deployment decision with explicit rcalib blocking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeployDecision {
    Deploy,
    ResearchOnly,
    BlockedByCalib,   // NEW: rcalib alone caused rejection
    BlockedByRisk,
    BlockedByKER,
}

/// v2: Evaluate kerdeployable with rcalib participation.
pub fn eval_kerdeployable(
    window: &KerWindow,
    config: &KerDeployableConfig,
    dt_data: f32,
) -> DeployDecision {
    // rcalib can independently block deployment
    if window.calib_risk_of_harm() > config.rcalib_hard {
        return DeployDecision::BlockedByCalib;
    }

    // Apply Dt_data scaling to K and E
    let k_adj = window.knowledge_factor() * dt_data;
    let e_adj = window.eco_impact() * dt_data;
    let r = window.risk_of_harm(); // R is NOT scaled down by Dt

    if r > config.r_max {
        return DeployDecision::BlockedByRisk;
    }

    if k_adj < config.k_min || e_adj < config.e_min {
        return DeployDecision::BlockedByKER;
    }

    DeployDecision::Deploy
}
