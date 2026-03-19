// ============================================================================
// Cyboquatic Safety Invariants Core
// ============================================================================
// Version: 1.0.0
// License: Apache-2.0 OR MIT
// Authors: Cyboquatic Research Collective
//
// This module implements the mathematical foundation of the Cyboquatic safety
// framework: Lyapunov stability (Vt non-increase), corridor-bounded risks
// (rx ∈ [0,1]), and K/E/R triad metrics for ecological impact assessment.
//
// Continuity Guarantee: All invariants are designed for formal verification
// and long-term stability analysis. No floating-point operations without
// explicit bounds checking or epsilon tolerance.
// ============================================================================

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt;

// ============================================================================
// Corridor Bands for Risk Coordinate Normalization
// ============================================================================

/// Defines the safety corridor bands for a single physical metric.
///
/// Corridors divide the operational space into three regions:
/// - Safe (0.0 to x_safe): No risk, optimal operation
/// - Gold (x_safe to x_gold): Acceptable risk, monitored operation
/// - Hard (x_gold to x_hard): Elevated risk, intervention required
/// - Violation (>= x_hard): Catastrophic risk, immediate stop
///
/// For 20-50 year continuity, corridor bounds should be derived from
/// empirical data and updated through the calibration pipeline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CorridorBands {
    /// Lower bound of safe region (risk = 0.0).
    pub x_safe: f64,
    /// Boundary between safe and gold regions (risk = 0.5).
    pub x_gold: f64,
    /// Upper bound of gold region (risk = 1.0).
    pub x_hard: f64,
}

impl CorridorBands {
    /// Creates new corridor bands with validation.
    ///
    /// # Errors
    /// Returns `CorridorError` if bounds are not properly ordered.
    pub fn new(x_safe: f64, x_gold: f64, x_hard: f64) -> Result<Self, CorridorError> {
        if x_safe > x_gold {
            return Err(CorridorError::InvalidOrder);
        }
        if x_gold > x_hard {
            return Err(CorridorError::InvalidOrder);
        }
        if (x_hard - x_safe).abs() < f64::EPSILON {
            return Err(CorridorError::ZeroWidth);
        }
        Ok(CorridorBands { x_safe, x_gold, x_hard })
    }

    /// Creates corridor bands without validation (use with caution).
    /// Only use when bounds are already verified.
    pub fn new_unchecked(x_safe: f64, x_gold: f64, x_hard: f64) -> Self {
        debug_assert!(x_safe <= x_gold && x_gold <= x_hard, "Corridor bounds must be ordered");
        CorridorBands { x_safe, x_gold, x_hard }
    }

    /// Normalizes a physical value to a risk coordinate in [0,1].
    ///
    /// # Algorithm
    /// - x <= x_safe → risk = 0.0
    /// - x_safe < x <= x_gold → risk ∈ (0.0, 0.5]
    /// - x_gold < x < x_hard → risk ∈ (0.5, 1.0)
    /// - x >= x_hard → risk = 1.0
    #[inline]
    pub fn normalize(&self, x: f64) -> RiskCoord {
        if x <= self.x_safe {
            return RiskCoord::new_clamped(0.0);
        }
        if x >= self.x_hard {
            return RiskCoord::new_clamped(1.0);
        }
        if x <= self.x_gold {
            let num = x - self.x_safe;
            let den = (self.x_gold - self.x_safe).max(f64::EPSILON);
            return RiskCoord::new_clamped(num / den * 0.5);
        }
        let num = x - self.x_gold;
        let den = (self.x_hard - self.x_gold).max(f64::EPSILON);
        RiskCoord::new_clamped(0.5 + num / den * 0.5)
    }

    /// Returns the width of the safe region.
    #[inline]
    pub fn safe_width(&self) -> f64 {
        self.x_gold - self.x_safe
    }

    /// Returns the width of the gold region.
    #[inline]
    pub fn gold_width(&self) -> f64 {
        self.x_hard - self.x_gold
    }

    /// Returns the total corridor width.
    #[inline]
    pub fn total_width(&self) -> f64 {
        self.x_hard - self.x_safe
    }

    /// Checks if a value is within the safe region.
    #[inline]
    pub fn is_safe(&self, x: f64) -> bool {
        x <= self.x_safe
    }

    /// Checks if a value is within the gold region.
    #[inline]
    pub fn is_gold(&self, x: f64) -> bool {
        x > self.x_safe && x <= self.x_gold
    }

    /// Checks if a value is within the hard region (but not violation).
    #[inline]
    pub fn is_hard(&self, x: f64) -> bool {
        x > self.x_gold && x < self.x_hard
    }

    /// Checks if a value is a violation (>= x_hard).
    #[inline]
    pub fn is_violation(&self, x: f64) -> bool {
        x >= self.x_hard
    }
}

/// Errors that can occur during corridor band construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorridorError {
    /// Bounds are not in proper order (safe <= gold <= hard).
    InvalidOrder,
    /// Corridor has zero or near-zero width.
    ZeroWidth,
}

impl fmt::Display for CorridorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CorridorError::InvalidOrder => write!(f, "corridor bounds must satisfy safe <= gold <= hard"),
            CorridorError::ZeroWidth => write!(f, "corridor width must be greater than zero"),
        }
    }
}

impl std::error::Error for CorridorError {}

// ============================================================================
// Risk Coordinates (rx)
// ============================================================================

/// Dimensionless risk coordinate r ∈ [0,1].
///
/// All risk coordinates are clamped to this range to ensure bounded
/// Lyapunov residuals and predictable safety behavior. This is the
/// fundamental unit of safety measurement in the Cyboquatic framework.
///
/// # Invariants
/// - 0.0 ≤ r ≤ 1.0 (always clamped)
/// - r = 0.0 → completely safe
/// - r = 0.5 → gold-band boundary
/// - r = 1.0 → hard-band violation (catastrophic)
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct RiskCoord(f64);

impl RiskCoord {
    /// Creates a new risk coordinate, clamping the value to [0,1].
    #[inline]
    pub fn new_clamped(v: f64) -> Self {
        let v = if v < 0.0 { 0.0 } else if v > 1.0 { 1.0 } else { v };
        RiskCoord(v)
    }

    /// Creates a new risk coordinate without clamping (use with caution).
    /// Only use when the value is already known to be in [0,1].
    #[inline]
    pub fn new_unchecked(v: f64) -> Self {
        debug_assert!(v >= 0.0 && v <= 1.0, "RiskCoord must be in [0,1]");
        RiskCoord(v)
    }

    /// Returns the underlying f64 value.
    #[inline]
    pub fn value(self) -> f64 { self.0 }

    /// Returns true if this risk coordinate is in the "hard-band" (catastrophic).
    #[inline]
    pub fn is_hard_band(self) -> bool { self.0 >= 1.0 }

    /// Returns true if this risk coordinate is in the "gold-band" (optimal).
    #[inline]
    pub fn is_gold_band(self) -> bool { self.0 <= 0.5 }

    /// Returns true if this risk coordinate is in the "safe-band" (minimal risk).
    #[inline]
    pub fn is_safe_band(self) -> bool { self.0 <= 0.25 }

    /// Squares the risk coordinate (for Lyapunov residual calculation).
    #[inline]
    pub fn squared(self) -> f64 { self.0 * self.0 }
}

impl Default for RiskCoord {
    fn default() -> Self { RiskCoord(0.0) }
}

impl fmt::Display for RiskCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r={:.4}", self.0)
    }
}

// ============================================================================
// Risk Vector (multi-dimensional safety state)
// ============================================================================

/// Vector of risk coordinates representing multi-dimensional safety state.
///
/// Each coordinate represents a different safety dimension (e.g., pressure,
/// temperature, chemical concentration, hydraulic load). The vector is
/// used to compute the Lyapunov residual and K/E/R triad metrics.
#[derive(Clone, Debug)]
pub struct RiskVector {
    /// Individual risk coordinates, each representing a different safety dimension.
    pub coords: Vec<RiskCoord>,
    /// Optional labels for each coordinate (for audit/logging purposes).
    pub labels: Vec<String>,
}

impl RiskVector {
    /// Creates a new risk vector from a slice of coordinates.
    pub fn new(coords: Vec<RiskCoord>) -> Self {
        let labels = (0..coords.len()).map(|i| format!("r_{}", i)).collect();
        RiskVector { coords, labels }
    }

    /// Creates a new risk vector with explicit labels.
    ///
    /// # Panics
    /// Panics if coords and labels have different lengths.
    pub fn with_labels(coords: Vec<RiskCoord>, labels: Vec<String>) -> Self {
        assert_eq!(coords.len(), labels.len(), "coords and labels must have same length");
        RiskVector { coords, labels }
    }

    /// Creates an empty risk vector.
    pub fn empty() -> Self {
        RiskVector { coords: Vec::new(), labels: Vec::new() }
    }

    /// Returns the number of risk coordinates in this vector.
    pub fn len(&self) -> usize { self.coords.len() }

    /// Returns true if the vector is empty.
    pub fn is_empty(&self) -> bool { self.coords.is_empty() }

    /// Returns the maximum risk coordinate in this vector.
    pub fn max(&self) -> RiskCoord {
        self.coords
            .iter()
            .copied()
            .max_by(|a, b| a.value().partial_cmp(&b.value()).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(RiskCoord::default())
    }

    /// Returns the index of the maximum risk coordinate.
    pub fn max_index(&self) -> Option<usize> {
        if self.coords.is_empty() {
            return None;
        }
        let mut max_idx = 0;
        let mut max_val = self.coords[0].value();
        for (i, r) in self.coords.iter().enumerate() {
            if r.value() > max_val {
                max_val = r.value();
                max_idx = i;
            }
        }
        Some(max_idx)
    }

    /// Returns the minimum risk coordinate in this vector.
    pub fn min(&self) -> RiskCoord {
        self.coords
            .iter()
            .copied()
            .min_by(|a, b| a.value().partial_cmp(&b.value()).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(RiskCoord::default())
    }

    /// Returns the weighted sum of squared risks (Lyapunov residual component).
    ///
    /// # Formula
    /// V_t = Σ w_j * r_j^2
    ///
    /// Where w_j are the weights and r_j are the risk coordinates.
    pub fn weighted_squared_sum(&self, weights: &[f64]) -> f64 {
        let mut sum = 0.0;
        for (r, w) in self.coords.iter().zip(weights.iter()) {
            let v = r.value();
            sum += w.max(0.0) * v * v;
        }
        sum
    }

    /// Returns the arithmetic mean of all risk coordinates.
    pub fn mean(&self) -> f64 {
        if self.coords.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.coords.iter().map(|r| r.value()).sum();
        sum / self.coords.len() as f64
    }

    /// Returns the number of risk coordinates in the "hard-band".
    pub fn hard_band_count(&self) -> usize {
        self.coords.iter().filter(|r| r.is_hard_band()).count()
    }

    /// Returns the number of risk coordinates in the "gold-band".
    pub fn gold_band_count(&self) -> usize {
        self.coords.iter().filter(|r| r.is_gold_band()).count()
    }

    /// Returns the number of risk coordinates in the "safe-band".
    pub fn safe_band_count(&self) -> usize {
        self.coords.iter().filter(|r| r.is_safe_band()).count()
    }

    /// Returns true if all coordinates are in the safe or gold bands.
    pub fn is_acceptable(&self) -> bool {
        self.coords.iter().all(|r| !r.is_hard_band())
    }
}

impl Default for RiskVector {
    fn default() -> Self { RiskVector::empty() }
}

impl fmt::Display for RiskVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RiskVector[{} dims, max={:.4}, mean={:.4}]", 
               self.len(), self.max().value(), self.mean())
    }
}

// ============================================================================
// Lyapunov Residual (Vt)
// ============================================================================

/// Lyapunov residual V_t = Σ w_j r_j^2.
///
/// This residual must be non-increasing (or bounded increase ε) for system
/// stability. The Lyapunov condition V_{t+1} ≤ V_t + ε is the core invariant
/// that guarantees long-term operational safety.
///
/// # Continuity Note
/// For 20-50 year operation, the epsilon tolerance should be set based on
/// sensor precision and expected drift rates. Typical values: 0.001 to 0.01.
#[derive(Clone, Copy, Debug)]
pub struct Residual {
    /// The computed residual value.
    pub vt: f64,
    /// The timestep at which this residual was computed (for audit trails).
    pub timestep: u64,
}

impl Residual {
    /// Creates a new residual from a risk vector and weights.
    pub fn from_weights(risks: &RiskVector, weights: &[f64], timestep: u64) -> Self {
        let vt = risks.weighted_squared_sum(weights);
        Residual { vt, timestep }
    }

    /// Creates a residual with explicit value (for testing/manual override).
    pub fn new(vt: f64, timestep: u64) -> Self {
        Residual { vt, timestep }
    }

    /// Returns true if this residual satisfies the Lyapunov condition vs. previous.
    ///
    /// # Formula
    /// V_t ≤ V_{t-1} + ε
    ///
    /// Where ε is the allowed tolerance per timestep.
    pub fn is_lyapunov_stable(&self, prev: &Residual, epsilon: f64) -> bool {
        self.vt <= prev.vt + epsilon
    }

    /// Returns the change in residual from previous timestep.
    pub fn delta(&self, prev: &Residual) -> f64 {
        self.vt - prev.vt
    }

    /// Returns true if the residual is decreasing (ideal stability).
    pub fn is_decreasing(&self, prev: &Residual) -> bool {
        self.vt < prev.vt
    }

    /// Returns true if the residual is within acceptable bounds.
    pub fn is_within_bounds(&self, max_vt: f64) -> bool {
        self.vt <= max_vt
    }
}

impl Default for Residual {
    fn default() -> Self {
        Residual { vt: 0.0, timestep: 0 }
    }
}

impl fmt::Display for Residual {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vt={:.6}@t={}", self.vt, self.timestep)
    }
}

// ============================================================================
// K/E/R Triad (Ecological Impact Assessment)
// ============================================================================

/// K/E/R triad for ecological impact assessment.
///
/// - K (Knowledge): Fraction of steps with Lyapunov stability
/// - E (Eco-impact): 1 - average risk (higher is better)
/// - R (Risk of harm): Maximum risk coordinate observed
///
/// This triad provides a comprehensive view of system safety and ecological
/// performance over time. For deployment, all three metrics must meet
/// minimum thresholds.
#[derive(Clone, Copy, Debug)]
pub struct KerTriad {
    /// Knowledge factor: fraction of steps with Lyapunov stability.
    pub k_knowledge: f64,
    /// Eco-impact factor: 1 - average risk (higher is better).
    pub e_ecoimpact: f64,
    /// Risk of harm: maximum risk coordinate observed.
    pub r_risk_of_harm: f64,
}

impl KerTriad {
    /// Creates a new KER triad from component values.
    pub fn new(k: f64, e: f64, r: f64) -> Self {
        KerTriad {
            k_knowledge: k.clamp(0.0, 1.0),
            e_ecoimpact: e.clamp(0.0, 1.0),
            r_risk_of_harm: r.clamp(0.0, 1.0),
        }
    }

    /// Creates a KER triad from a KerWindow (historical data).
    pub fn from_window(window: &KerWindow) -> Self {
        window.finalize()
    }

    /// Returns true if this triad meets deployment thresholds.
    ///
    /// Default thresholds: K ≥ 0.90, E ≥ 0.90, R ≤ 0.13
    pub fn meets_deployment_criteria(&self) -> bool {
        self.k_knowledge >= 0.90 && self.e_ecoimpact >= 0.90 && self.r_risk_of_harm <= 0.13
    }

    /// Returns true if this triad meets custom thresholds.
    pub fn meets_custom_criteria(&self, k_min: f64, e_min: f64, r_max: f64) -> bool {
        self.k_knowledge >= k_min && self.e_ecoimpact >= e_min && self.r_risk_of_harm <= r_max
    }

    /// Returns a composite safety score (higher is better).
    ///
    /// # Formula
    /// score = (K + E + (1 - R)) / 3
    pub fn composite_score(&self) -> f64 {
        (self.k_knowledge + self.e_ecoimpact + (1.0 - self.r_risk_of_harm)) / 3.0
    }

    /// Returns the safety margin (distance from failure thresholds).
    pub fn safety_margin(&self) -> f64 {
        let k_margin = self.k_knowledge - 0.90;
        let e_margin = self.e_ecoimpact - 0.90;
        let r_margin = 0.13 - self.r_risk_of_harm;
        k_margin.min(e_margin).min(r_margin)
    }

    /// Returns true if the system is in a degraded state (margins < 0.1).
    pub fn is_degraded(&self) -> bool {
        self.safety_margin() < 0.1
    }

    /// Returns true if the system is in a critical state (any margin < 0).
    pub fn is_critical(&self) -> bool {
        self.safety_margin() < 0.0
    }
}

impl Default for KerTriad {
    fn default() -> Self {
        KerTriad::new(1.0, 1.0, 0.0)
    }
}

impl fmt::Display for KerTriad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "KER(k={:.3}, e={:.3}, r={:.3}, score={:.3}, margin={:.3})",
            self.k_knowledge, self.e_ecoimpact, self.r_risk_of_harm, 
            self.composite_score(), self.safety_margin()
        )
    }
}

// ============================================================================
// KerWindow (Rolling K/E/R Calculation)
// ============================================================================

/// Rolling window for K/E/R triad calculation over time.
///
/// Tracks Lyapunov stability history and risk observations to compute
/// the K (knowledge) and R (risk of harm) components of the KER triad.
/// The E (eco-impact) component is computed separately from risk means.
///
/// # Continuity Note
/// For 20-50 year operation, this window should be periodically flushed
/// and archived to prevent unbounded memory growth. Recommended: flush
/// every 1M steps or 30 days, whichever comes first.
#[derive(Clone, Debug)]
pub struct KerWindow {
    /// Total number of steps observed.
    total_steps: u64,
    /// Number of steps with Lyapunov stability.
    lyapunov_safe_steps: u64,
    /// Maximum risk coordinate observed.
    max_risk: f64,
    /// Sum of all risk coordinates (for mean calculation).
    risk_sum: f64,
    /// Start timestep for this window.
    start_timestep: u64,
    /// End timestep for this window.
    end_timestep: u64,
}

impl KerWindow {
    /// Creates a new empty KerWindow.
    pub fn new() -> Self {
        KerWindow {
            total_steps: 0,
            lyapunov_safe_steps: 0,
            max_risk: 0.0,
            risk_sum: 0.0,
            start_timestep: 0,
            end_timestep: 0,
        }
    }

    /// Creates a KerWindow starting at a specific timestep.
    pub fn with_start_timestep(start_timestep: u64) -> Self {
        KerWindow {
            total_steps: 0,
            lyapunov_safe_steps: 0,
            max_risk: 0.0,
            risk_sum: 0.0,
            start_timestep,
            end_timestep: start_timestep,
        }
    }

    /// Updates the window with a new step observation.
    ///
    /// # Arguments
    /// * `lyapunov_safe` - Whether this step satisfied Lyapunov stability
    /// * `risks` - Risk vector for this step
    pub fn update_step(&mut self, lyapunov_safe: bool, risks: &RiskVector) {
        self.total_steps += 1;
        self.end_timestep += 1;
        
        if lyapunov_safe {
            self.lyapunov_safe_steps += 1;
        }
        
        let m = risks.max().value();
        if m > self.max_risk {
            self.max_risk = m;
        }
        
        self.risk_sum += risks.mean();
    }

    /// Updates the window with explicit risk value (simplified path).
    pub fn update_risk(&mut self, risk: f64, lyapunov_safe: bool) {
        self.total_steps += 1;
        self.end_timestep += 1;
        
        if lyapunov_safe {
            self.lyapunov_safe_steps += 1;
        }
        
        if risk > self.max_risk {
            self.max_risk = risk;
        }
        
        self.risk_sum += risk;
    }

    /// Finalizes the window and returns the KER triad.
    pub fn finalize(&self) -> KerTriad {
        let k = if self.total_steps == 0 {
            1.0
        } else {
            self.lyapunov_safe_steps as f64 / self.total_steps as f64
        };
        
        let r = self.max_risk;
        let avg_risk = if self.total_steps == 0 {
            0.0
        } else {
            self.risk_sum / self.total_steps as f64
        };
        let e = (1.0 - avg_risk).max(0.0);
        
        KerTriad {
            k_knowledge: k,
            e_ecoimpact: e,
            r_risk_of_harm: r,
        }
    }

    /// Returns the current (partial) KER triad without finalizing.
    pub fn current_ker(&self) -> KerTriad {
        self.finalize()
    }

    /// Returns the total number of steps in this window.
    pub fn total_steps(&self) -> u64 {
        self.total_steps
    }

    /// Returns the Lyapunov stability rate (K component).
    pub fn stability_rate(&self) -> f64 {
        if self.total_steps == 0 {
            return 1.0;
        }
        self.lyapunov_safe_steps as f64 / self.total_steps as f64
    }

    /// Returns the maximum risk observed in this window.
    pub fn max_risk(&self) -> f64 {
        self.max_risk
    }

    /// Returns the average risk observed in this window.
    pub fn average_risk(&self) -> f64 {
        if self.total_steps == 0 {
            return 0.0;
        }
        self.risk_sum / self.total_steps as f64
    }

    /// Resets the window to initial state.
    pub fn reset(&mut self) {
        self.total_steps = 0;
        self.lyapunov_safe_steps = 0;
        self.max_risk = 0.0;
        self.risk_sum = 0.0;
        self.end_timestep = self.start_timestep;
    }

    /// Merges another window into this one (for distributed systems).
    pub fn merge(&mut self, other: &KerWindow) {
        self.total_steps += other.total_steps;
        self.lyapunov_safe_steps += other.lyapunov_safe_steps;
        self.max_risk = self.max_risk.max(other.max_risk);
        self.risk_sum += other.risk_sum;
        self.end_timestep = self.end_timestep.max(other.end_timestep);
    }
}

impl Default for KerWindow {
    fn default() -> Self {
        KerWindow::new()
    }
}

impl fmt::Display for KerWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ker = self.finalize();
        write!(f, "KerWindow[steps={}, {}]", self.total_steps, ker)
    }
}

// ============================================================================
// EcoSafety Kernel (Core Safety Enforcement)
// ============================================================================

/// Ecosafety kernel enforcing V_{t+1} ≤ V_t + ε and no hard-band violations.
///
/// This is the core safety enforcement engine. Every control step must pass
/// through this kernel before actuation. The kernel maintains state across
/// steps to track Lyapunov residual history.
///
/// # Invariants
/// - V_{t+1} ≤ V_t + ε (Lyapunov stability with tolerance)
/// - max(r_x) < 1.0 (no hard-band violations)
/// - All risk coordinates clamped to [0,1]
#[derive(Clone, Debug)]
pub struct EcoSafetyKernel {
    /// Previous Lyapunov residual value.
    pub vt_prev: f64,
    /// Allowed epsilon increase per timestep.
    pub eps_vt: f64,
    /// Timestep counter.
    pub timestep: u64,
    /// Rolling window for K/E/R calculation.
    pub ker_window: KerWindow,
}

/// Decision returned by the safety kernel for each step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SafeStepDecision {
    /// Command is safe and can proceed unchanged.
    Accept,
    /// Command is risky; should be derated (reduced aggressiveness).
    Derate,
    /// Command is unsafe; must be blocked (emergency stop).
    Stop,
}

impl SafeStepDecision {
    /// Returns true if this decision allows actuation.
    pub fn allows_actuation(&self) -> bool {
        matches!(self, SafeStepDecision::Accept | SafeStepDecision::Derate)
    }

    /// Returns true if this decision requires command modification.
    pub fn requires_modification(&self) -> bool {
        matches!(self, SafeStepDecision::Derate | SafeStepDecision::Stop)
    }
}

impl fmt::Display for SafeStepDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SafeStepDecision::Accept => write!(f, "Accept"),
            SafeStepDecision::Derate => write!(f, "Derate"),
            SafeStepDecision::Stop => write!(f, "Stop"),
        }
    }
}

impl EcoSafetyKernel {
    /// Creates a new safety kernel with the given epsilon tolerance.
    pub fn new(eps_vt: f64) -> Self {
        EcoSafetyKernel {
            vt_prev: 0.0,
            eps_vt,
            timestep: 0,
            ker_window: KerWindow::new(),
        }
    }

    /// Creates a kernel with explicit initial residual value.
    pub fn with_initial_vt(vt_initial: f64, eps_vt: f64) -> Self {
        EcoSafetyKernel {
            vt_prev: vt_initial,
            eps_vt,
            timestep: 0,
            ker_window: KerWindow::new(),
        }
    }

    /// Evaluates a proposed step and returns a safety decision.
    ///
    /// # Algorithm
    /// 1. Check for hard-band violations (any r_x >= 1.0) → Stop
    /// 2. Check Lyapunov condition (V_t ≤ V_{t-1} + ε) → Accept or Derate
    /// 3. Update internal state for next iteration
    pub fn evaluate_step(&mut self, residual: Residual, risks: &RiskVector) -> SafeStepDecision {
        self.timestep += 1;
        let vt = residual.vt;
        let max_r = risks.max().value();

        // Check for hard-band violation (catastrophic risk)
        if max_r >= 1.0 {
            self.vt_prev = vt;
            self.ker_window.update_step(false, risks);
            return SafeStepDecision::Stop;
        }

        // Check Lyapunov stability condition
        let lyapunov_safe = vt <= self.vt_prev + self.eps_vt;
        
        self.vt_prev = vt;
        self.ker_window.update_step(lyapunov_safe, risks);

        if lyapunov_safe {
            SafeStepDecision::Accept
        } else {
            SafeStepDecision::Derate
        }
    }

    /// Evaluates a step with explicit decision thresholds.
    ///
    /// # Arguments
    /// * `derate_threshold` - Vt increase ratio that triggers derate (default: 0.5)
    /// * `stop_threshold` - Vt increase ratio that triggers stop (default: 1.0)
    pub fn evaluate_step_thresholded(
        &mut self, 
        residual: Residual, 
        risks: &RiskVector,
        derate_threshold: f64,
        stop_threshold: f64,
    ) -> SafeStepDecision {
        self.timestep += 1;
        let vt = residual.vt;
        let max_r = risks.max().value();
        let vt_delta = vt - self.vt_prev;
        let vt_ratio = if self.vt_prev > f64::EPSILON {
            vt_delta / self.vt_prev
        } else {
            vt_delta
        };

        // Hard-band violation always stops
        if max_r >= 1.0 {
            self.vt_prev = vt;
            self.ker_window.update_step(false, risks);
            return SafeStepDecision::Stop;
        }

        // Threshold-based decision
        let decision = if vt_ratio >= stop_threshold {
            SafeStepDecision::Stop
        } else if vt_ratio >= derate_threshold {
            SafeStepDecision::Derate
        } else {
            SafeStepDecision::Accept
        };

        let lyapunov_safe = matches!(decision, SafeStepDecision::Accept);
        self.vt_prev = vt;
        self.ker_window.update_step(lyapunov_safe, risks);

        decision
    }

    /// Returns the current KER triad based on history.
    pub fn current_ker(&self) -> KerTriad {
        self.ker_window.current_ker()
    }

    /// Returns the current Lyapunov residual value.
    pub fn current_vt(&self) -> f64 {
        self.vt_prev
    }

    /// Returns the current timestep.
    pub fn current_timestep(&self) -> u64 {
        self.timestep
    }

    /// Returns the Lyapunov stability rate (K component).
    pub fn stability_rate(&self) -> f64 {
        self.ker_window.stability_rate()
    }

    /// Resets the kernel state (use with caution).
    pub fn reset(&mut self, initial_vt: f64) {
        self.vt_prev = initial_vt;
        self.timestep = 0;
        self.ker_window.reset();
    }

    /// Sets a new epsilon tolerance (for adaptive systems).
    pub fn set_epsilon(&mut self, eps_vt: f64) {
        self.eps_vt = eps_vt;
    }
}

impl Default for EcoSafetyKernel {
    fn default() -> Self {
        EcoSafetyKernel::new(0.001)
    }
}

impl fmt::Display for EcoSafetyKernel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EcoSafetyKernel[Vt={:.6}, ε={:.6}, t={}, {}]", 
               self.vt_prev, self.eps_vt, self.timestep, self.current_ker())
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corridor_bands_normalization() {
        let corridor = CorridorBands::new(0.0, 50.0, 100.0).unwrap();
        
        assert_eq!(corridor.normalize(0.0).value(), 0.0);
        assert_eq!(corridor.normalize(25.0).value(), 0.25);
        assert_eq!(corridor.normalize(50.0).value(), 0.5);
        assert_eq!(corridor.normalize(75.0).value(), 0.75);
        assert_eq!(corridor.normalize(100.0).value(), 1.0);
        assert_eq!(corridor.normalize(150.0).value(), 1.0);
    }

    #[test]
    fn test_corridor_bands_validation() {
        assert!(CorridorBands::new(0.0, 50.0, 100.0).is_ok());
        assert!(CorridorBands::new(50.0, 0.0, 100.0).is_err());
        assert!(CorridorBands::new(0.0, 100.0, 50.0).is_err());
    }

    #[test]
    fn test_risk_coord_properties() {
        let r = RiskCoord::new_clamped(0.7);
        assert!(!r.is_safe_band());
        assert!(!r.is_gold_band());
        assert!(!r.is_hard_band());
        assert_eq!(r.squared(), 0.49);
    }

    #[test]
    fn test_risk_vector_statistics() {
        let coords = vec![
            RiskCoord::new_clamped(0.3),
            RiskCoord::new_clamped(0.7),
            RiskCoord::new_clamped(0.5),
        ];
        let rv = RiskVector::new(coords);
        
        assert_eq!(rv.max().value(), 0.7);
        assert_eq!(rv.min().value(), 0.3);
        assert!((rv.mean() - 0.5).abs() < 0.001);
        assert_eq!(rv.len(), 3);
    }

    #[test]
    fn test_residual_lyapunov_stability() {
        let prev = Residual::new(1.0, 0);
        let curr = Residual::new(1.0005, 1);
        
        assert!(curr.is_lyapunov_stable(&prev, 0.001));
        assert!(!curr.is_lyapunov_stable(&prev, 0.0001));
        assert!(curr.is_decreasing(&Residual::new(1.1, 0)));
    }

    #[test]
    fn test_ker_triad_criteria() {
        let ker = KerTriad::new(0.95, 0.92, 0.10);
        assert!(ker.meets_deployment_criteria());
        assert!((ker.composite_score() - 0.923).abs() < 0.001);
        
        let ker_fail = KerTriad::new(0.85, 0.92, 0.10);
        assert!(!ker_fail.meets_deployment_criteria());
        assert!(ker_fail.is_critical());
    }

    #[test]
    fn test_ker_window_accumulation() {
        let mut window = KerWindow::new();
        
        for i in 0..100 {
            let risk = if i < 90 { 0.1 } else { 0.5 };
            let lyapunov_safe = i < 95;
            let coords = vec![RiskCoord::new_clamped(risk)];
            let rv = RiskVector::new(coords);
            window.update_step(lyapunov_safe, &rv);
        }
        
        let ker = window.finalize();
        assert!((ker.k_knowledge - 0.95).abs() < 0.01);
        assert_eq!(window.total_steps(), 100);
    }

    #[test]
    fn test_eco_safety_kernel_decisions() {
        let mut kernel = EcoSafetyKernel::new(0.001);
        
        // Safe step
        let coords = vec![RiskCoord::new_clamped(0.3)];
        let rv = RiskVector::new(coords);
        let res = Residual::new(0.09, 1);
        assert_eq!(kernel.evaluate_step(res, &rv), SafeStepDecision::Accept);
        
        // Hard-band violation
        let coords = vec![RiskCoord::new_clamped(1.0)];
        let rv = RiskVector::new(coords);
        let res = Residual::new(1.0, 2);
        assert_eq!(kernel.evaluate_step(res, &rv), SafeStepDecision::Stop);
    }

    #[test]
    fn test_kernel_ker_tracking() {
        let mut kernel = EcoSafetyKernel::new(0.001);
        
        for i in 0..50 {
            let risk = 0.2;
            let coords = vec![RiskCoord::new_clamped(risk)];
            let rv = RiskVector::new(coords);
            let res = Residual::new(0.04 + (i as f64 * 0.0001), i as u64);
            kernel.evaluate_step(res, &rv);
        }
        
        let ker = kernel.current_ker();
        assert!(ker.k_knowledge > 0.9);
        assert!(ker.e_ecoimpact > 0.7);
    }
}
