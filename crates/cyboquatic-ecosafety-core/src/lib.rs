//! Cyboquatic Ecosafety Core Library
//!
//! Provides the foundational Lyapunov-based risk framework for energy-efficient,
//! carbon-negative, and ecologically-restorative industrial machinery.
//!
//! # Architecture
//!
//! Each operational domain (energy, hydraulics, biology, carbon, materials, etc.)
//! is normalized to a risk coordinate r_x ∈ [0,1]. These coordinates are
//! aggregated into a quadratic Lyapunov residual V_t = Σ w_j r_j² that
//! governs all actuation decisions.
//!
//! # Safety Guarantees
//!
//! - No action without a risk estimate (enforced at type level)
//! - Lyapunov stability invariant: V_{t+1} ≤ V_t + ε
//! - Hard corridor gates prevent instantiation of unsafe configurations
//!
//! # Governance Metrics
//!
//! K/E/R triad provides rolling-window assessment:
//! - K (Knowledge-factor): Fraction of Lyapunov-safe steps
//! - E (Eco-impact): Complement of maximum risk coordinate
//! - R (Risk-of-harm): Maximum observed risk coordinate

#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

use alloc::string::String;
use core::fmt;

// ============================================================================
// CORE CONSTANTS
// ============================================================================

pub const MAX_RISK_PLANES: usize = 8;
pub const DEFAULT_LYAPUNOV_EPSILON: f64 = 0.001;
pub const K_THRESHOLD_DEPLOY: f64 = 0.90;
pub const E_THRESHOLD_DEPLOY: f64 = 0.90;
pub const R_THRESHOLD_DEPLOY: f64 = 0.13;
pub const KER_WINDOW_SIZE: usize = 100;

// ============================================================================
// INNER KERNEL: FIXED MULTI-PLANE RISK COORDS & LYAPUNOV (CORE, F32)
// ============================================================================

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RiskCoordCore(pub f32);

impl RiskCoordCore {
    #[inline]
    pub fn clamped(v: f32) -> Self {
        let v = if v < 0.0 {
            0.0
        } else if v > 1.0 {
            1.0
        } else {
            v
        };
        RiskCoordCore(v)
    }

    #[inline]
    pub fn value(self) -> f32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RiskVectorCore {
    pub r_energy: RiskCoordCore,
    pub r_hydraulic: RiskCoordCore,
    pub r_biology: RiskCoordCore,
    pub r_carbon: RiskCoordCore,
    pub r_materials: RiskCoordCore,
}

impl RiskVectorCore {
    #[inline]
    pub fn max_coord(&self) -> RiskCoordCore {
        let mut m = self.r_energy.value();
        m = m.max(self.r_hydraulic.value());
        m = m.max(self.r_biology.value());
        m = m.max(self.r_carbon.value());
        m = m.max(self.r_materials.value());
        RiskCoordCore(m)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LyapunovResidualCore {
    pub value: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct ResidualWeights {
    pub w_energy: f32,
    pub w_hydraulic: f32,
    pub w_biology: f32,
    pub w_carbon: f32,
    pub w_materials: f32,
}

impl ResidualWeights {
    pub const fn default() -> Self {
        Self {
            w_energy: 1.0,
            w_hydraulic: 1.2,
            w_biology: 1.5,
            w_carbon: 1.3,
            w_materials: 1.4,
        }
    }
}

#[inline]
pub fn compute_residual_core(r: &RiskVectorCore, w: &ResidualWeights) -> LyapunovResidualCore {
    let e = r.r_energy.value();
    let h = r.r_hydraulic.value();
    let b = r.r_biology.value();
    let c = r.r_carbon.value();
    let m = r.r_materials.value();
    let v = w.w_energy * e * e
        + w.w_hydraulic * h * h
        + w.w_biology * b * b
        + w.w_carbon * c * c
        + w.w_materials * m * m;
    LyapunovResidualCore { value: v }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SafeStepDecisionCore {
    Accept,
    Derate,
    Stop,
}

pub trait SafeControllerCore<State, Command> {
    fn propose_step(&self, state: &State) -> (Command, RiskVectorCore);
}

pub struct EcoSafetyKernelCore {
    pub eps_vt: f32,
    pub weights: ResidualWeights,
}

impl EcoSafetyKernelCore {
    pub const fn new(eps_vt: f32, weights: ResidualWeights) -> Self {
        Self { eps_vt, weights }
    }

    pub fn check_step(
        &self,
        v_prev: LyapunovResidualCore,
        r_next: &RiskVectorCore,
    ) -> (LyapunovResidualCore, SafeStepDecisionCore) {
        let r_max = r_next.max_coord();
        if r_max.value() >= 1.0 {
            let v = compute_residual_core(r_next, &self.weights);
            return (v, SafeStepDecisionCore::Stop);
        }

        let v_next = compute_residual_core(r_next, &self.weights);

        if v_next.value <= v_prev.value {
            (v_next, SafeStepDecisionCore::Accept)
        } else if v_next.value <= v_prev.value + self.eps_vt {
            (v_next, SafeStepDecisionCore::Derate)
        } else {
            (v_next, SafeStepDecisionCore::Stop)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct KerTriadCore {
    pub k_knowledge: f32,
    pub e_ecoimpact: f32,
    pub r_risk_of_harm: f32,
}

pub struct KerWindowCore<const N: usize> {
    steps_total: u32,
    steps_safe: u32,
    max_r: f32,
    buf_v: [f32; N],
    idx: usize,
}

impl<const N: usize> KerWindowCore<N> {
    pub const fn new() -> Self {
        Self {
            steps_total: 0,
            steps_safe: 0,
            max_r: 0.0,
            buf_v: [0.0; N],
            idx: 0,
        }
    }

    pub fn record_step(
        &mut self,
        v_prev: LyapunovResidualCore,
        v_next: LyapunovResidualCore,
        r_max: RiskCoordCore,
    ) {
        self.steps_total += 1;
        if v_next.value <= v_prev.value {
            self.steps_safe += 1;
        }
        if r_max.value() > self.max_r {
            self.max_r = r_max.value();
        }
        self.buf_v[self.idx] = v_next.value;
        self.idx = (self.idx + 1) % N;
    }

    pub fn triad(&self) -> KerTriadCore {
        let k = if self.steps_total == 0 {
            1.0
        } else {
            (self.steps_safe as f32) / (self.steps_total as f32)
        };
        let r = self.max_r;
        let e = 1.0 - r;
        KerTriadCore {
            k_knowledge: k,
            e_ecoimpact: e,
            r_risk_of_harm: r,
        }
    }
}

// ============================================================================
// GOVERNANCE-LAYER RISK PLANES & CORRIDORS (F64)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RiskPlane {
    Energy = 0,
    Hydraulic = 1,
    Biology = 2,
    Carbon = 3,
    Materials = 4,
    Thermal = 5,
    Mechanical = 6,
    SensorCalibration = 7,
}

impl RiskPlane {
    pub const fn count() -> usize {
        MAX_RISK_PLANES
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RiskPlane::Energy => "energy",
            RiskPlane::Hydraulic => "hydraulic",
            RiskPlane::Biology => "biology",
            RiskPlane::Carbon => "carbon",
            RiskPlane::Materials => "materials",
            RiskPlane::Thermal => "thermal",
            RiskPlane::Mechanical => "mechanical",
            RiskPlane::SensorCalibration => "sensor_calibration",
        }
    }

    pub fn from(idx: usize) -> Self {
        match idx {
            0 => RiskPlane::Energy,
            1 => RiskPlane::Hydraulic,
            2 => RiskPlane::Biology,
            3 => RiskPlane::Carbon,
            4 => RiskPlane::Materials,
            5 => RiskPlane::Thermal,
            6 => RiskPlane::Mechanical,
            _ => RiskPlane::SensorCalibration,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CorridorBands {
    pub var_id: &'static str,
    pub units: &'static str,
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub weight: f64,
    pub lyap_chan: u8,
}

impl CorridorBands {
    pub const fn default_for(var_id: &'static str, units: &'static str) -> Self {
        Self {
            var_id,
            units,
            safe: 0.30,
            gold: 0.70,
            hard: 1.00,
            weight: 1.0,
            lyap_chan: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorridorStatus {
    Safe,
    Gold,
    Hard,
    Violation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorridorError {
    InvalidBandOrder,
    OutOfBounds,
    PlaneNotFound,
}

impl fmt::Display for CorridorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CorridorError::InvalidBandOrder => {
                write!(f, "Corridor bands must satisfy safe ≤ gold ≤ hard")
            }
            CorridorError::OutOfBounds => {
                write!(f, "Corridor values must be in range [0.0, 1.0]")
            }
            CorridorError::PlaneNotFound => {
                write!(f, "Requested risk plane not found in configuration")
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RiskCoordGov {
    pub r: f64,
    pub sigma: f64,
    pub bands: CorridorBands,
}

impl RiskCoordGov {
    pub fn status(&self) -> CorridorStatus {
        if self.r < self.bands.safe {
            CorridorStatus::Safe
        } else if self.r < self.bands.gold {
            CorridorStatus::Gold
        } else if self.r < self.bands.hard {
            CorridorStatus::Hard
        } else {
            CorridorStatus::Violation
        }
    }

    pub fn corridor_ok(&self) -> bool {
        self.status() != CorridorStatus::Violation
    }
}

pub fn normalize_to_r(raw: f64, min: f64, max: f64, bands: CorridorBands) -> RiskCoordGov {
    if max <= min {
        return RiskCoordGov {
            r: 1.0,
            sigma: 0.0,
            bands,
        };
    }
    let normalized = (raw - min) / (max - min);
    let clamped = normalized.clamp(0.0, 1.0);

    let r = if clamped < bands.safe {
        clamped * 0.5
    } else if clamped < bands.gold {
        0.15 + (clamped - bands.safe) * 1.25
    } else {
        0.65 + (clamped - bands.gold) * 1.75
    }
    .clamp(0.0, 1.0);

    RiskCoordGov {
        r,
        sigma: 0.0,
        bands,
    }
}

// ============================================================================
// RESIDUAL STATE & KER (GOVERNANCE VIEW, F64)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct ResidualState {
    pub vt: f64,
    pub r_e: RiskCoordGov,
    pub r_h: RiskCoordGov,
    pub r_b: RiskCoordGov,
    pub r_c: RiskCoordGov,
    pub r_m: RiskCoordGov,
    pub r_t: RiskCoordGov,
    pub r_mech: RiskCoordGov,
    pub r_sens: RiskCoordGov,
}

impl ResidualState {
    pub fn coords(&self) -> [RiskCoordGov; MAX_RISK_PLANES] {
        [
            self.r_e,
            self.r_h,
            self.r_b,
            self.r_c,
            self.r_m,
            self.r_t,
            self.r_mech,
            self.r_sens,
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KerTriad {
    pub k_knowledge: f64,
    pub e_ecoimpact: f64,
    pub r_risk: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct KerWindowGov {
    pub steps_total: u64,
    pub steps_lyap_safe: u64,
    pub max_r_seen: f64,
}

impl KerWindowGov {
    pub const fn new() -> Self {
        Self {
            steps_total: 0,
            steps_lyap_safe: 0,
            max_r_seen: 0.0,
        }
    }

    pub fn update(&mut self, vt_ok: bool, coords: &[RiskCoordGov; MAX_RISK_PLANES]) {
        self.steps_total += 1;
        if vt_ok {
            self.steps_lyap_safe += 1;
        }
        for c in coords {
            if c.r > self.max_r_seen {
                self.max_r_seen = c.r;
            }
        }
    }

    pub fn triad(&self) -> KerTriad {
        if self.steps_total == 0 {
            return KerTriad {
                k_knowledge: 0.0,
                e_ecoimpact: 0.0,
                r_risk: 0.0,
            };
        }
        let k = (self.steps_lyap_safe as f64) / (self.steps_total as f64);
        let r = self.max_r_seen;
        let e = (1.0 - r).clamp(0.0, 1.0);
        KerTriad {
            k_knowledge: k,
            e_ecoimpact: e,
            r_risk: r,
        }
    }
}

pub fn recompute_vt(state: &mut ResidualState) {
    let coords = state.coords();
    let mut vt = 0.0;
    for c in coords {
        vt += c.bands.weight * c.r * c.r;
    }
    state.vt = vt;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorridorDecision {
    Ok,
    Derate,
    Stop,
}

pub fn safe_step(prev: &ResidualState, next: &ResidualState, eps_vt: f64) -> CorridorDecision {
    let any_hard = next.coords().iter().any(|c| c.r >= 1.0);
    if any_hard {
        return CorridorDecision::Stop;
    }
    if next.vt <= prev.vt + eps_vt {
        CorridorDecision::Ok
    } else {
        CorridorDecision::Derate
    }
}

// ============================================================================
// HIGH-LEVEL RISK VECTOR & SYSTEM STATE (PLANE-INDEXED, F64)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct RiskCoord {
    pub value: f64,
}

impl RiskCoord {
    pub fn new(raw: f64) -> Self {
        Self {
            value: raw.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskVector {
    coordinates: [f64; MAX_RISK_PLANES],
    timestamp: u64,
    validated: bool,
}

impl RiskVector {
    pub fn new(timestamp: u64) -> Self {
        RiskVector {
            coordinates: [0.0; MAX_RISK_PLANES],
            timestamp,
            validated: true,
        }
    }

    pub fn set_coordinate(&mut self, plane: RiskPlane, value: f64) {
        let clamped = value.clamp(0.0, 1.0);
        self.coordinates[plane as usize] = clamped;
    }

    pub fn get_coordinate(&self, plane: RiskPlane) -> f64 {
        self.coordinates[plane as usize]
    }

    pub fn max_coordinate(&self) -> f64 {
        self.coordinates.iter().copied().fold(0.0_f64, f64::max)
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn is_valid(&self) -> bool {
        self.validated
            && self
                .coordinates
                .iter()
                .all(|&c| (0.0..=1.0).contains(&c))
    }

    pub fn lyapunov_residual(&self, weights: &[f64; MAX_RISK_PLANES]) -> f64 {
        let mut v_t = 0.0;
        for i in 0..MAX_RISK_PLANES {
            v_t += weights[i].max(0.0) * self.coordinates[i].powi(2);
        }
        v_t
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingMode {
    Idle,
    Normal,
    EcoRestorative,
    CarbonNegative,
    Maintenance,
    Emergency,
}

#[derive(Debug, Clone)]
pub struct SystemState {
    pub current_v_t: f64,
    pub previous_v_t: f64,
    pub current_risk: RiskVector,
    pub energy_surplus: f64,
    pub mode: OperatingMode,
    pub timestamp: u64,
}

impl SystemState {
    pub fn new(timestamp: u64) -> Self {
        SystemState {
            current_v_t: 0.0,
            previous_v_t: 0.0,
            current_risk: RiskVector::new(timestamp),
            energy_surplus: 0.0,
            mode: OperatingMode::Idle,
            timestamp,
        }
    }

    pub fn would_violate_invariant(&self, proposed_v_t: f64, epsilon: f64) -> bool {
        proposed_v_t > self.current_v_t + epsilon
    }
}

// ============================================================================
// TYPE-LEVEL "NO ACTION WITHOUT RISK" CONTROLLER TRAIT (HIGH-LEVEL)
// ============================================================================

pub trait LyapunovController {
    type Actuation;

    fn propose_actuation(
        &self,
        current_state: &SystemState,
        timestamp: u64,
    ) -> Option<(Self::Actuation, RiskVector)>;

    fn controller_id(&self) -> u32;

    fn affected_planes(&self) -> &'static [RiskPlane];
}

// ============================================================================
// KER GOVERNANCE METRICS (ROLLING WINDOW, HIGH-LEVEL)
// ============================================================================

#[derive(Debug, Clone)]
pub struct KERMetrics {
    safe_steps: [bool; KER_WINDOW_SIZE],
    max_risks: [f64; KER_WINDOW_SIZE],
    window_index: usize,
    total_steps: usize,
}

impl KERMetrics {
    pub fn new() -> Self {
        KERMetrics {
            safe_steps: [false; KER_WINDOW_SIZE],
            max_risks: [0.0; KER_WINDOW_SIZE],
            window_index: 0,
            total_steps: 0,
        }
    }

    fn effective_window_size(&self) -> usize {
        self.total_steps.min(KER_WINDOW_SIZE)
    }

    pub fn record_step(&mut self, is_lyapunov_safe: bool, max_risk: f64) {
        self.safe_steps[self.window_index] = is_lyapunov_safe;
        self.max_risks[self.window_index] = max_risk.clamp(0.0, 1.0);
        self.window_index = (self.window_index + 1) % KER_WINDOW_SIZE;
        self.total_steps = self.total_steps.saturating_add(1);
    }

    pub fn knowledge_factor(&self) -> f64 {
        let n = self.effective_window_size();
        if n == 0 {
            return 1.0;
        }
        let safe_count = self.safe_steps[..n].iter().filter(|&&s| s).count();
        safe_count as f64 / n as f64
    }

    pub fn eco_impact(&self) -> f64 {
        let n = self.effective_window_size();
        if n == 0 {
            return 1.0;
        }
        let max_observed = self.max_risks[..n]
            .iter()
            .copied()
            .fold(0.0_f64, f64::max);
        (1.0 - max_observed).clamp(0.0, 1.0)
    }

    pub fn risk_of_harm(&self) -> f64 {
        let n = self.effective_window_size();
        if n == 0 {
            return 0.0;
        }
        self.max_risks[..n]
            .iter()
            .copied()
            .fold(0.0_f64, f64::max)
    }

    pub fn meets_deployment_thresholds(&self) -> bool {
        self.knowledge_factor() >= K_THRESHOLD_DEPLOY
            && self.eco_impact() >= E_THRESHOLD_DEPLOY
            && self.risk_of_harm() <= R_THRESHOLD_DEPLOY
    }

    pub fn summary(&self) -> String {
        alloc::format!(
            "KER: K={:.3}, E={:.3}, R={:.3} | Deployable: {}",
            self.knowledge_factor(),
            self.eco_impact(),
            self.risk_of_harm(),
            self.meets_deployment_thresholds()
        )
    }
}

impl Default for KERMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ECOSAFETY ENFORCER (NON-ACTUATING GATE)
// ============================================================================

pub struct EcosafetyEnforcer {
    pub weights: [f64; MAX_RISK_PLANES],
    pub epsilon: f64,
    pub corridors: [CorridorBands; MAX_RISK_PLANES],
    metrics: KERMetrics,
    current_v_t: f64,
}

impl EcosafetyEnforcer {
    pub fn new() -> Self {
        let mut corridors = [CorridorBands::default_for("r0", "-"); MAX_RISK_PLANES];
        corridors[RiskPlane::Energy as usize] = CorridorBands::default_for("r_energy", "-");
        EcosafetyEnforcer {
            weights: [1.0; MAX_RISK_PLANES],
            epsilon: DEFAULT_LYAPUNOV_EPSILON,
            corridors,
            metrics: KERMetrics::new(),
            current_v_t: 0.0,
        }
    }

    pub fn set_weight(&mut self, plane: RiskPlane, weight: f64) {
        self.weights[plane as usize] = weight.max(0.0);
    }

    pub fn set_corridor(&mut self, plane: RiskPlane, corridor: CorridorBands) {
        self.corridors[plane as usize] = corridor;
    }

    pub fn enforce<A>(
        &mut self,
        actuation: A,
        risk_vector: RiskVector,
    ) -> Result<A, EnforcementError> {
        if !risk_vector.is_valid() {
            return Err(EnforcementError::InvalidRiskVector);
        }

        for plane in [
            RiskPlane::Energy,
            RiskPlane::Hydraulic,
            RiskPlane::Biology,
            RiskPlane::Carbon,
            RiskPlane::Materials,
            RiskPlane::Thermal,
            RiskPlane::Mechanical,
            RiskPlane::SensorCalibration,
        ] {
            let coord = risk_vector.get_coordinate(plane);
            let bands = self.corridors[plane as usize];
            let rc = RiskCoordGov {
                r: coord,
                sigma: 0.0,
                bands,
            };
            if !rc.corridor_ok() {
                self.metrics
                    .record_step(false, risk_vector.max_coordinate());
                return Err(EnforcementError::CorridorViolation(plane));
            }
        }

        let proposed_v_t = risk_vector.lyapunov_residual(&self.weights);

        if proposed_v_t > self.current_v_t + self.epsilon {
            self.metrics
                .record_step(false, risk_vector.max_coordinate());
            return Err(EnforcementError::LyapunovViolation {
                current: self.current_v_t,
                proposed: proposed_v_t,
            });
        }

        self.metrics
            .record_step(true, risk_vector.max_coordinate());
        self.current_v_t = proposed_v_t;
        Ok(actuation)
    }

    pub fn metrics(&self) -> &KERMetrics {
        &self.metrics
    }

    pub fn current_lyapunov_residual(&self) -> f64 {
        self.current_v_t
    }

    pub fn reset(&mut self) {
        self.current_v_t = 0.0;
        self.metrics = KERMetrics::new();
    }
}

impl Default for EcosafetyEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnforcementError {
    InvalidRiskVector,
    CorridorViolation(RiskPlane),
    LyapunovViolation { current: f64, proposed: f64 },
}

impl fmt::Display for EnforcementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnforcementError::InvalidRiskVector => {
                write!(f, "Risk vector failed validation")
            }
            EnforcementError::CorridorViolation(plane) => {
                write!(f, "Corridor violation in {} plane", plane.as_str())
            }
            EnforcementError::LyapunovViolation { current, proposed } => {
                write!(
                    f,
                    "Lyapunov violation: V_t would increase from {} to {}",
                    current, proposed
                )
            }
        }
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

pub fn aggregate_risk(risk_vector: &RiskVector, weights: &[f64; MAX_RISK_PLANES]) -> f64 {
    let mut aggregate = 0.0;
    let mut w_sum = 0.0;
    for i in 0..MAX_RISK_PLANES {
        let w = weights[i].max(0.0);
        aggregate += w * risk_vector.get_coordinate(RiskPlane::from(i));
        w_sum += w;
    }
    if w_sum > 0.0 {
        aggregate / w_sum
    } else {
        0.0
    }
}

pub fn generate_diagnostics(enforcer: &EcosafetyEnforcer) -> String {
    alloc::format!(
        "=== Cyboquatic Ecosafety Diagnostics ===\n\
         Lyapunov V_t: {:.6}\n\
         {}\n\
         Weights: {:?}\n",
        enforcer.current_lyapunov_residual(),
        enforcer.metrics().summary(),
        enforcer.weights
    )
}
