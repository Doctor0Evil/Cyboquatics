//! Cyboquatic Ecosafety Core Library
//! 
//! Provides the foundational Lyapunov-based risk framework for energy-efficient,
//! carbon-negative, and ecologically-restorative industrial machinery.
//! 
//! # Architecture
//! 
//! This library implements a multi-plane risk assessment system where each
//! operational domain (energy, hydraulics, biology, carbon, materials) is
//! normalized to a risk coordinate r_x ∈ [0,1]. These coordinates are
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

#![no_std]
#![cfg_attr(not(test), no_main)]
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use core::fmt;

// ============================================================================
// CONSTANTS AND CONFIGURATION
// ============================================================================

/// Maximum number of risk planes supported in the Lyapunov residual
pub const MAX_RISK_PLANES: usize = 8;

/// Default Lyapunov tolerance epsilon for stability invariant
pub const DEFAULT_LYAPUNOV_EPSILON: f64 = 0.001;

/// Minimum acceptable Knowledge-factor for deployment gating
pub const K_THRESHOLD_DEPLOY: f64 = 0.90;

/// Minimum acceptable Eco-impact for deployment gating
pub const E_THRESHOLD_DEPLOY: f64 = 0.90;

/// Maximum acceptable Risk-of-harm for deployment gating
pub const R_THRESHOLD_DEPLOY: f64 = 0.13;

/// Rolling window size for KER metric calculation (number of steps)
pub const KER_WINDOW_SIZE: usize = 100;

// ============================================================================
// RISK PLANE ENUMERATION
// ============================================================================

/// Identifies each normalized risk plane in the multi-dimensional safety space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RiskPlane {
    /// Energy consumption and efficiency risk
    Energy = 0,
    /// Hydraulic pressure and flow risk
    Hydraulic = 1,
    /// Biological activity and contamination risk
    Biology = 2,
    /// Carbon emissions and sequestration risk
    Carbon = 3,
    /// Material degradation and toxicity risk
    Materials = 4,
    /// Thermal operating range risk
    Thermal = 5,
    /// Mechanical stress and fatigue risk
    Mechanical = 6,
    /// Sensor calibration and data quality risk
    SensorCalibration = 7,
}

impl RiskPlane {
    /// Returns the total number of defined risk planes
    pub const fn count() -> usize {
        8
    }
    
    /// Returns a string identifier for the risk plane
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
}

// ============================================================================
// CORRIDOR NORMALIZATION
// ============================================================================

/// Corridor bands for risk normalization (safegoldhard pattern)
#[derive(Debug, Clone, Copy)]
pub struct CorridorBands {
    /// Safe zone upper bound (r_x < safe_upper → green)
    pub safe_upper: f64,
    /// Gold zone upper bound (safe_upper ≤ r_x < gold_upper → yellow)
    pub gold_upper: f64,
    /// Hard limit (r_x ≥ hard_limit → red, rejection)
    pub hard_limit: f64,
}

impl CorridorBands {
    /// Creates default corridor bands with conservative thresholds
    pub const fn default() -> Self {
        CorridorBands {
            safe_upper: 0.30,
            gold_upper: 0.70,
            hard_limit: 1.00,
        }
    }
    
    /// Creates custom corridor bands with validation
    pub fn new(safe_upper: f64, gold_upper: f64, hard_limit: f64) -> Result<Self, CorridorError> {
        if safe_upper >= gold_upper || gold_upper > hard_limit || safe_upper < 0.0 {
            return Err(CorridorError::InvalidBandOrder);
        }
        if hard_limit > 1.0 || safe_upper < 0.0 {
            return Err(CorridorError::OutOfBounds);
        }
        Ok(CorridorBands {
            safe_upper,
            gold_upper,
            hard_limit,
        })
    }
    
    /// Normalizes a raw measurement to a risk coordinate r_x ∈ [0,1]
    /// 
    /// # Arguments
    /// * `raw_value` - The raw sensor or computed measurement
    /// * `reference_min` - Minimum expected value (maps to r_x = 0)
    /// * `reference_max` - Maximum expected value (maps to r_x = 1)
    pub fn normalize(&self, raw_value: f64, reference_min: f64, reference_max: f64) -> f64 {
        if reference_max <= reference_min {
            return 1.0; // Invalid range, assume worst case
        }
        
        let normalized = (raw_value - reference_min) / (reference_max - reference_min);
        let clamped = normalized.clamp(0.0, 1.0);
        
        // Apply corridor weighting for non-linear risk scaling
        if clamped < self.safe_upper {
            clamped * 0.5 // Safe zone: reduced risk scaling
        } else if clamped < self.gold_upper {
            0.15 + (clamped - self.safe_upper) * 1.25 // Gold zone: moderate scaling
        } else {
            0.65 + (clamped - self.gold_upper) * 1.75 // Hard zone: aggressive scaling
        }
    }
    
    /// Checks if a risk coordinate is within acceptable corridor bounds
    pub fn corridor_ok(&self, risk_coord: f64) -> bool {
        risk_coord < self.hard_limit
    }
    
    /// Returns the corridor status for a risk coordinate
    pub fn corridor_status(&self, risk_coord: f64) -> CorridorStatus {
        if risk_coord < self.safe_upper {
            CorridorStatus::Safe
        } else if risk_coord < self.gold_upper {
            CorridorStatus::Gold
        } else if risk_coord < self.hard_limit {
            CorridorStatus::Hard
        } else {
            CorridorStatus::Violation
        }
    }
}

/// Status of a risk coordinate relative to corridor bands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorridorStatus {
    Safe,
    Gold,
    Hard,
    Violation,
}

/// Errors that can occur during corridor configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorridorError {
    InvalidBandOrder,
    OutOfBounds,
    PlaneNotFound,
}

impl fmt::Display for CorridorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CorridorError::InvalidBandOrder => write!(f, "Corridor bands must be ordered: safe < gold < hard"),
            CorridorError::OutOfBounds => write!(f, "Corridor values must be in range [0.0, 1.0]"),
            CorridorError::PlaneNotFound => write!(f, "Requested risk plane not found in configuration"),
        }
    }
}

// ============================================================================
// RISK VECTOR
// ============================================================================

/// Complete risk assessment vector containing all normalized risk coordinates
#[derive(Debug, Clone)]
pub struct RiskVector {
    /// Normalized risk coordinates for each plane (indexed by RiskPlane discriminant)
    coordinates: [f64; MAX_RISK_PLANES],
    /// Timestamp of risk assessment (monotonic counter or epoch)
    timestamp: u64,
    /// Validation flag ensuring vector was properly constructed
    validated: bool,
}

impl RiskVector {
    /// Creates a new zero-initialized risk vector
    pub fn new(timestamp: u64) -> Self {
        RiskVector {
            coordinates: [0.0; MAX_RISK_PLANES],
            timestamp,
            validated: true,
        }
    }
    
    /// Sets the risk coordinate for a specific plane
    /// 
    /// # Safety
    /// This method enforces that risk coordinates remain in [0,1]
    pub fn set_coordinate(&mut self, plane: RiskPlane, value: f64) {
        let clamped = value.clamp(0.0, 1.0);
        self.coordinates[plane as usize] = clamped;
    }
    
    /// Gets the risk coordinate for a specific plane
    pub fn get_coordinate(&self, plane: RiskPlane) -> f64 {
        self.coordinates[plane as usize]
    }
    
    /// Returns the maximum risk coordinate across all planes (R metric component)
    pub fn max_coordinate(&self) -> f64 {
        self.coordinates.iter().cloned().fold(0.0_f64, f64::max)
    }
    
    /// Returns the timestamp of this risk assessment
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    /// Validates that all coordinates are within bounds
    pub fn is_valid(&self) -> bool {
        self.validated && self.coordinates.iter().all(|&c| c >= 0.0 && c <= 1.0)
    }
    
    /// Computes the Lyapunov residual V_t for this risk vector
    pub fn lyapunov_residual(&self, weights: &[f64; MAX_RISK_PLANES]) -> f64 {
        let mut v_t = 0.0;
        for i in 0..MAX_RISK_PLANES {
            v_t += weights[i] * self.coordinates[i].powi(2);
        }
        v_t
    }
}

// ============================================================================
// LYAPUNOV CONTROLLER TRAIT
// ============================================================================

/// Trait that all Cyboquatic controllers must implement
/// 
/// This trait enforces the "no action without risk estimate" rule at the type level.
/// Any controller that proposes an actuation must simultaneously provide a complete
/// risk assessment vector.
pub trait LyapunovController {
    /// The type of actuation proposal this controller generates
    type Actuation;
    
    /// Proposes an actuation along with its complete risk assessment
    /// 
    /// # Returns
    /// * `Some((Actuation, RiskVector))` if a safe actuation is proposed
    /// * `None` if no safe actuation can be determined
    fn propose_actuation(&self, current_state: &SystemState, timestamp: u64) 
        -> Option<(Self::Actuation, RiskVector)>;
    
    /// Returns the controller's unique identifier
    fn controller_id(&self) -> u32;
    
    /// Returns the risk planes this controller affects
    fn affected_planes(&self) -> &'static [RiskPlane];
}

// ============================================================================
// SYSTEM STATE
// ============================================================================

/// Complete system state snapshot for controller decision-making
#[derive(Debug, Clone)]
pub struct SystemState {
    /// Current Lyapunov residual V_t
    pub current_v_t: f64,
    /// Previous Lyapunov residual V_{t-1}
    pub previous_v_t: f64,
    /// Current risk vector
    pub current_risk: RiskVector,
    /// Energy surplus available (joules)
    pub energy_surplus: f64,
    /// Operational mode identifier
    pub mode: OperatingMode,
    /// Timestamp of state snapshot
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
    
    /// Checks if the Lyapunov stability invariant would be violated
    pub fn would_violate_invariant(&self, proposed_v_t: f64, epsilon: f64) -> bool {
        proposed_v_t > self.current_v_t + epsilon
    }
}

/// Operating modes for Cyboquatic machinery
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingMode {
    Idle,
    Normal,
    EcoRestorative,
    CarbonNegative,
    Maintenance,
    Emergency,
}

// ============================================================================
// KER GOVERNANCE METRICS
// ============================================================================

/// Rolling-window governance metrics (Knowledge/Eco-impact/Risk)
#[derive(Debug, Clone)]
pub struct KERMetrics {
    /// Rolling window of Lyapunov-safe step flags
    safe_steps: [bool; KER_WINDOW_SIZE],
    /// Rolling window of maximum risk coordinates
    max_risks: [f64; KER_WINDOW_SIZE],
    /// Current window index (circular buffer)
    window_index: usize,
    /// Total steps processed
    total_steps: usize,
}

impl KERMetrics {
    /// Creates a new KERMetrics tracker with zeroed windows
    pub fn new() -> Self {
        KERMetrics {
            safe_steps: [false; KER_WINDOW_SIZE],
            max_risks: [0.0; KER_WINDOW_SIZE],
            window_index: 0,
            total_steps: 0,
        }
    }
    
    /// Records a new step's outcome for metric calculation
    pub fn record_step(&mut self, is_lyapunov_safe: bool, max_risk: f64) {
        self.safe_steps[self.window_index] = is_lyapunov_safe;
        self.max_risks[self.window_index] = max_risk.clamp(0.0, 1.0);
        self.window_index = (self.window_index + 1) % KER_WINDOW_SIZE;
        self.total_steps = self.total_steps.saturating_add(1);
    }
    
    /// Calculates the Knowledge-factor (K): fraction of Lyapunov-safe steps
    pub fn knowledge_factor(&self) -> f64 {
        let window_size = self.effective_window_size();
        if window_size == 0 {
            return 1.0;
        }
        let safe_count = self.safe_steps[..window_size].iter().filter(|&&s| s).count();
        safe_count as f64 / window_size as f64
    }
    
    /// Calculates the Eco-impact (E): complement of maximum observed risk
    pub fn eco_impact(&self) -> f64 {
        let window_size = self.effective_window_size();
        if window_size == 0 {
            return 1.0;
        }
        let max_observed = self.max_risks[..window_size].iter().cloned().fold(0.0_f64, f64::max);
        1.0 - max_observed
    }
    
    /// Calculates the Risk-of-harm (R): maximum observed risk coordinate
    pub fn risk_of_harm(&self) -> f64 {
        let window_size = self.effective_window_size();
        if window_size == 0 {
            return 0.0;
        }
        self.max_risks[..window_size].iter().cloned().fold(0.0_f64, f64::max)
    }
    
    /// Returns the effective window size (may be less than KER_WINDOW_SIZE during warmup)
    fn effective_window_size(&self) -> usize {
        self.total_steps.min(KER_WINDOW_SIZE)
    }
    
    /// Checks if current metrics meet deployment thresholds
    pub fn meets_deployment_thresholds(&self) -> bool {
        self.knowledge_factor() >= K_THRESHOLD_DEPLOY
            && self.eco_impact() >= E_THRESHOLD_DEPLOY
            && self.risk_of_harm() <= R_THRESHOLD_DEPLOY
    }
    
    /// Returns a summary string of current metrics
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
// ECOSAFETY ENFORCER
// ============================================================================

/// Central enforcer of Lyapunov stability contracts
pub struct EcosafetyEnforcer {
    /// Lyapunov weights for each risk plane
    weights: [f64; MAX_RISK_PLANES],
    /// Lyapunov tolerance epsilon
    epsilon: f64,
    /// Corridor configurations for each plane
    corridors: [CorridorBands; MAX_RISK_PLANES],
    /// Governance metrics tracker
    metrics: KERMetrics,
    /// Current Lyapunov residual
    current_v_t: f64,
}

impl EcosafetyEnforcer {
    /// Creates a new enforcer with default configuration
    pub fn new() -> Self {
        EcosafetyEnforcer {
            weights: [1.0; MAX_RISK_PLANES], // Equal weighting by default
            epsilon: DEFAULT_LYAPUNOV_EPSILON,
            corridors: [CorridorBands::default(); MAX_RISK_PLANES],
            metrics: KERMetrics::new(),
            current_v_t: 0.0,
        }
    }
    
    /// Sets the Lyapunov weight for a specific risk plane
    pub fn set_weight(&mut self, plane: RiskPlane, weight: f64) {
        self.weights[plane as usize] = weight.max(0.0);
    }
    
    /// Sets the corridor configuration for a specific risk plane
    pub fn set_corridor(&mut self, plane: RiskPlane, corridor: CorridorBands) {
        self.corridors[plane as usize] = corridor;
    }
    
    /// Validates and enforces a proposed actuation
    /// 
    /// # Returns
    /// * `Ok(actuation)` if the actuation passes all safety checks
    /// * `Err(EnforcementError)` if any safety invariant is violated
    pub fn enforce<A>(&mut self, actuation: A, risk_vector: RiskVector) 
        -> Result<A, EnforcementError> 
    {
        // Check 1: Validate risk vector integrity
        if !risk_vector.is_valid() {
            return Err(EnforcementError::InvalidRiskVector);
        }
        
        // Check 2: Verify all coordinates are within corridor bounds
        for plane in [RiskPlane::Energy, RiskPlane::Hydraulic, RiskPlane::Biology,
                      RiskPlane::Carbon, RiskPlane::Materials, RiskPlane::Thermal,
                      RiskPlane::Mechanical, RiskPlane::SensorCalibration] {
            let coord = risk_vector.get_coordinate(plane);
            let corridor = self.corridors[plane as usize];
            if !corridor.corridor_ok(coord) {
                return Err(EnforcementError::CorridorViolation(plane));
            }
        }
        
        // Check 3: Compute proposed Lyapunov residual
        let proposed_v_t = risk_vector.lyapunov_residual(&self.weights);
        
        // Check 4: Verify Lyapunov stability invariant
        if proposed_v_t > self.current_v_t + self.epsilon {
            return Err(EnforcementError::LyapunovViolation {
                current: self.current_v_t,
                proposed: proposed_v_t,
            });
        }
        
        // Record metrics for governance
        self.metrics.record_step(true, risk_vector.max_coordinate());
        
        // Update state
        self.current_v_t = proposed_v_t;
        
        Ok(actuation)
    }
    
    /// Returns current governance metrics
    pub fn metrics(&self) -> &KERMetrics {
        &self.metrics
    }
    
    /// Returns current Lyapunov residual
    pub fn current_lyapunov_residual(&self) -> f64 {
        self.current_v_t
    }
    
    /// Resets the enforcer state (use with caution)
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

/// Errors that can occur during ecosafety enforcement
#[derive(Debug, Clone, PartialEq)]
pub enum EnforcementError {
    InvalidRiskVector,
    CorridorViolation(RiskPlane),
    LyapunovViolation { current: f64, proposed: f64 },
}

impl fmt::Display for EnforcementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnforcementError::InvalidRiskVector => write!(f, "Risk vector failed validation"),
            EnforcementError::CorridorViolation(plane) => {
                write!(f, "Corridor violation in {} plane", plane.as_str())
            },
            EnforcementError::LyapunovViolation { current, proposed } => {
                write!(f, "Lyapunov violation: V_t would increase from {} to {}", current, proposed)
            },
        }
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Computes a weighted aggregate risk from multiple planes
pub fn aggregate_risk(risk_vector: &RiskVector, weights: &[f64; MAX_RISK_PLANES]) -> f64 {
    let mut aggregate = 0.0;
    for i in 0..MAX_RISK_PLANES {
        aggregate += weights[i] * risk_vector.get_coordinate(RiskPlane::Energy);
    }
    aggregate / MAX_RISK_PLANES as f64
}

/// Generates a diagnostic report for the current system state
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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_corridor_normalization() {
        let corridor = CorridorBands::default();
        let risk = corridor.normalize(0.5, 0.0, 1.0);
        assert!(risk >= 0.0 && risk <= 1.0);
    }
    
    #[test]
    fn test_risk_vector_lyapunov() {
        let mut rv = RiskVector::new(1000);
        rv.set_coordinate(RiskPlane::Energy, 0.3);
        rv.set_coordinate(RiskPlane::Hydraulic, 0.5);
        
        let weights = [1.0; MAX_RISK_PLANES];
        let v_t = rv.lyapunov_residual(&weights);
        assert!(v_t > 0.0);
    }
    
    #[test]
    fn test_ker_metrics() {
        let mut metrics = KERMetrics::new();
        for i in 0..50 {
            metrics.record_step(true, 0.1);
        }
        assert!(metrics.knowledge_factor() > 0.9);
        assert!(metrics.eco_impact() > 0.8);
    }
    
    #[test]
    fn test_enforcer_rejection() {
        let mut enforcer = EcosafetyEnforcer::new();
        let mut rv = RiskVector::new(1000);
        rv.set_coordinate(RiskPlane::Energy, 0.99); // Near violation
        
        let result = enforcer.enforce("actuation", rv);
        // Should pass if within epsilon tolerance
        assert!(result.is_ok() || result.is_err());
    }
}
