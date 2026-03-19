// ============================================================================
// Cyboquatic Gateway Core Library
// ============================================================================
// Version: 1.0.0
// License: Apache-2.0 OR MIT
// Authors: Cyboquatic Research Collective
// 
// This library implements the safety mediation layer between legacy hardware
// control systems (HCS) and physical actuators. All actuation commands pass
// through the SafeStepGate which enforces Lyapunov stability (Vt non-increase)
// and corridor-bounded risk coordinates (rx ∈ [0,1]).
//
// Continuity Guarantee: Designed for 20-50 year operational lifespan with
// cryptographic audit trails, sensor drift compensation, and phased rollout
// support (monitoring → derate → full-gate modes).
// ============================================================================

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod gateway;
pub mod invariants;
pub mod calibration;
pub mod audit;

pub use gateway::{SafeStepGate, GatewayMode, GatewayConfig};
pub use invariants::{EcoSafetyKernel, RiskVector, RiskCoord, Residual, KerTriad};
pub use calibration::{SensorCalibration, DriftCompensator, CalibrationRecord};
pub use audit::{AuditLog, AuditEntry, HexStamp};

// ============================================================================
// Re-exports from cyboquatic-ecosafety-core for convenience
// ============================================================================

/// Dimensionless risk coordinate r ∈ [0,1].
/// 
/// All risk coordinates are clamped to this range to ensure bounded
/// Lyapunov residuals and predictable safety behavior.
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
}

impl Default for RiskCoord {
    fn default() -> Self { RiskCoord(0.0) }
}

/// Vector of risk coordinates representing multi-dimensional safety state.
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
    pub fn with_labels(coords: Vec<RiskCoord>, labels: Vec<String>) -> Self {
        assert_eq!(coords.len(), labels.len(), "coords and labels must have same length");
        RiskVector { coords, labels }
    }

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

    /// Returns the weighted sum of squared risks (Lyapunov residual component).
    pub fn weighted_squared_sum(&self, weights: &[f64]) -> f64 {
        let mut sum = 0.0;
        for (r, w) in self.coords.iter().zip(weights.iter()) {
            let v = r.value();
            sum += w.max(0.0) * v * v;
        }
        sum
    }

    /// Returns the number of risk coordinates in the "hard-band".
    pub fn hard_band_count(&self) -> usize {
        self.coords.iter().filter(|r| r.is_hard_band()).count()
    }

    /// Returns the number of risk coordinates in the "gold-band".
    pub fn gold_band_count(&self) -> usize {
        self.coords.iter().filter(|r| r.is_gold_band()).count()
    }
}

impl Default for RiskVector {
    fn default() -> Self { RiskVector::new(Vec::new()) }
}

/// Lyapunov residual V_t = Σ w_j r_j^2.
/// 
/// This residual must be non-increasing (or bounded increase ε) for system stability.
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

    /// Returns true if this residual satisfies the Lyapunov condition vs. previous.
    pub fn is_lyapunov_stable(&self, prev: &Residual, epsilon: f64) -> bool {
        self.vt <= prev.vt + epsilon
    }
}

/// K/E/R triad for ecological impact assessment.
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

    /// Returns true if this triad meets deployment thresholds.
    pub fn meets_deployment_criteria(&self) -> bool {
        self.k_knowledge >= 0.90 && self.e_ecoimpact >= 0.90 && self.r_risk_of_harm <= 0.13
    }

    /// Returns a composite safety score (higher is better).
    pub fn composite_score(&self) -> f64 {
        (self.k_knowledge + self.e_ecoimpact + (1.0 - self.r_risk_of_harm)) / 3.0
    }
}

impl std::fmt::Display for KerTriad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "KER(k={:.3}, e={:.3}, r={:.3}, score={:.3})",
            self.k_knowledge, self.e_ecoimpact, self.r_risk_of_harm, self.composite_score()
        )
    }
}

// ============================================================================
// Gateway Mode Enumeration for Phased Rollout
// ============================================================================

/// Operational mode for the safety gateway.
/// 
/// Supports phased rollout: monitoring → derate → full-gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GatewayMode {
    /// Passive monitoring only; no actuation mediation.
    Monitoring,
    /// Active mediation; can derate but not stop commands.
    DerateOnly,
    /// Full mediation; can accept, derate, or stop any command.
    FullGate,
}

impl GatewayMode {
    /// Returns true if this mode can modify actuation commands.
    pub fn can_mediate(&self) -> bool {
        matches!(self, GatewayMode::DerateOnly | GatewayMode::FullGate)
    }

    /// Returns true if this mode can stop actuation entirely.
    pub fn can_stop(&self) -> bool {
        matches!(self, GatewayMode::FullGate)
    }
}

/// Configuration for the safety gateway.
#[derive(Clone, Debug)]
pub struct GatewayConfig {
    /// Operational mode (for phased rollout).
    pub mode: GatewayMode,
    /// Lyapunov epsilon (allowed Vt increase per step).
    pub eps_vt: f64,
    /// Risk weights for residual calculation.
    pub risk_weights: Vec<f64>,
    /// Minimum KER scores for continued operation.
    pub ker_min_k: f64,
    pub ker_min_e: f64,
    pub ker_max_r: f64,
    /// Audit logging enabled.
    pub audit_enabled: bool,
    /// Sensor drift compensation enabled.
    pub drift_compensation_enabled: bool,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        GatewayConfig {
            mode: GatewayMode::Monitoring,
            eps_vt: 0.001,
            risk_weights: vec![1.0],
            ker_min_k: 0.90,
            ker_min_e: 0.90,
            ker_max_r: 0.13,
            audit_enabled: true,
            drift_compensation_enabled: true,
        }
    }
}

impl GatewayConfig {
    /// Validates the configuration parameters.
    pub fn validate(&self) -> Result<(), GatewayConfigError> {
        if self.eps_vt < 0.0 {
            return Err(GatewayConfigError::NegativeEpsilon);
        }
        if self.risk_weights.iter().any(|w| *w < 0.0) {
            return Err(GatewayConfigError::NegativeWeight);
        }
        if self.ker_min_k < 0.0 || self.ker_min_k > 1.0 {
            return Err(GatewayConfigError::InvalidKerThreshold);
        }
        if self.ker_min_e < 0.0 || self.ker_min_e > 1.0 {
            return Err(GatewayConfigError::InvalidKerThreshold);
        }
        if self.ker_max_r < 0.0 || self.ker_max_r > 1.0 {
            return Err(GatewayConfigError::InvalidKerThreshold);
        }
        Ok(())
    }
}

/// Errors that can occur during gateway configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GatewayConfigError {
    NegativeEpsilon,
    NegativeWeight,
    InvalidKerThreshold,
}

impl std::fmt::Display for GatewayConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GatewayConfigError::NegativeEpsilon => write!(f, "epsilon must be non-negative"),
            GatewayConfigError::NegativeWeight => write!(f, "risk weights must be non-negative"),
            GatewayConfigError::InvalidKerThreshold => write!(f, "KER thresholds must be in [0,1]"),
        }
    }
}

impl std::error::Error for GatewayConfigError {}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_coord_clamping() {
        assert_eq!(RiskCoord::new_clamped(-0.5).value(), 0.0);
        assert_eq!(RiskCoord::new_clamped(0.5).value(), 0.5);
        assert_eq!(RiskCoord::new_clamped(1.5).value(), 1.0);
    }

    #[test]
    fn test_risk_vector_max() {
        let coords = vec![
            RiskCoord::new_clamped(0.3),
            RiskCoord::new_clamped(0.7),
            RiskCoord::new_clamped(0.5),
        ];
        let rv = RiskVector::new(coords);
        assert_eq!(rv.max().value(), 0.7);
    }

    #[test]
    fn test_residual_lyapunov_stability() {
        let prev = Residual { vt: 1.0, timestep: 0 };
        let curr = Residual { vt: 1.0005, timestep: 1 };
        assert!(curr.is_lyapunov_stable(&prev, 0.001));
        assert!(!curr.is_lyapunov_stable(&prev, 0.0001));
    }

    #[test]
    fn test_ker_triad_deployment_criteria() {
        let ker = KerTriad::new(0.95, 0.92, 0.10);
        assert!(ker.meets_deployment_criteria());
        
        let ker_fail = KerTriad::new(0.85, 0.92, 0.10);
        assert!(!ker_fail.meets_deployment_criteria());
    }

    #[test]
    fn test_gateway_mode_capabilities() {
        assert!(!GatewayMode::Monitoring.can_mediate());
        assert!(GatewayMode::DerateOnly.can_mediate());
        assert!(!GatewayMode::DerateOnly.can_stop());
        assert!(GatewayMode::FullGate.can_stop());
    }

    #[test]
    fn test_gateway_config_validation() {
        let config = GatewayConfig::default();
        assert!(config.validate().is_ok());

        let mut bad_config = config.clone();
        bad_config.eps_vt = -0.001;
        assert!(bad_config.validate().is_err());
    }
}
