//! Cyboquatic Material Kinetics Module
//! 
//! Models biodegradable casings, liners, trays, and filter media via MaterialKinetics.
//! Converts physical properties (t90, toxicity, micro-residue, leachate CEC, PFAS)
//! into normalized material risk coordinates (r_materials) for the Lyapunov residual.
//! 
//! # Safety Features
//! 
//! - AntSafeSubstrate trait enforces hard gates at instantiation
//! - Calibration error scenario generation for sensor drift robustness
//! - Automatic parameter adjustment queue for risk-of-harm reduction
//! 
//! # Integration
//! 
//! Feeds directly into RiskPlane::Materials within the core ecosafety spine.

#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use core::fmt;
use crate::{CorridorBands, CorridorStatus, RiskPlane, CorridorError};

// ============================================================================
// MATERIAL KINETICS CONSTANTS
// ============================================================================

/// Default maximum t90 (days) for acceptable biodegradation in restorative contexts
pub const DEFAULT_T90_MAX_DAYS: f64 = 180.0;

/// Default toxicity threshold (arbitrary units, normalized later)
pub const DEFAULT_TOXICITY_THRESHOLD: f64 = 0.1;

/// PFAS presence flag (true = immediate hard violation)
pub const PFAS_PRESENCE_VIOLATION: bool = true;

/// Calibration window size for material mass loss sensors
pub const MATERIAL_CALIBRATION_WINDOW: usize = 50;

// ============================================================================
// MATERIAL KINETICS STRUCT
// ============================================================================

/// Defines the kinetic properties of a substrate material
#[derive(Debug, Clone, Copy)]
pub struct MaterialKinetics {
    /// Time to 90% degradation (days)
    pub t90_days: f64,
    /// Toxicity index of breakdown products (0.0 = inert, 1.0 = lethal)
    pub toxicity_index: f64,
    /// Micro-residue formation rate (mass fraction per day)
    pub micro_residue_rate: f64,
    /// Leachate Cation Exchange Capacity (CEC) impact (mmol/kg)
    pub leachate_cec: f64,
    /// Presence of PFAS (permanent vs biodegradable)
    pub pfas_present: bool,
    /// Caloric density of material (kJ/kg) - relevant for energy recovery
    pub caloric_density: f64,
    /// Carbon sequestration potential (kg CO2e per kg material)
    pub carbon_sequestration_potential: f64,
}

impl MaterialKinetics {
    /// Creates a new MaterialKinetics profile with validation
    pub fn new(
        t90_days: f64,
        toxicity_index: f64,
        micro_residue_rate: f64,
        leachate_cec: f64,
        pfas_present: bool,
        caloric_density: f64,
        carbon_sequestration_potential: f64,
    ) -> Result<Self, MaterialError> {
        if t90_days < 0.0 || toxicity_index < 0.0 || micro_residue_rate < 0.0 {
            return Err(MaterialError::InvalidKineticValue);
        }
        if pfas_present {
            return Err(MaterialError::PfasDetected);
        }
        
        Ok(MaterialKinetics {
            t90_days,
            toxicity_index,
            micro_residue_rate,
            leachate_cec,
            pfas_present,
            caloric_density,
            carbon_sequestration_potential,
        })
    }
    
    /// Computes the normalized material risk coordinate r_materials
    pub fn compute_risk(&self, corridors: &CorridorBands) -> f64 {
        // Base risk from t90 (deviation from optimal degradation rate)
        // For restorative machinery, faster non-toxic breakdown is preferred
        let t90_normalized = if self.t90_days > DEFAULT_T90_MAX_DAYS {
            1.0
        } else {
            self.t90_days / DEFAULT_T90_MAX_DAYS
        };
        
        // Toxicity risk (direct mapping)
        let tox_risk = self.toxicity_index.clamp(0.0, 1.0);
        
        // Micro-residue risk (penalize persistent microplastics)
        let micro_risk = (self.micro_residue_rate * 100.0).clamp(0.0, 1.0);
        
        // Leachate risk (normalized against typical soil CEC)
        let leachate_risk = (self.leachate_cec / 50.0).clamp(0.0, 1.0);
        
        // Carbon benefit (negative risk if sequestration is high)
        let carbon_benefit = self.carbon_sequestration_potential.clamp(0.0, 1.0);
        
        // Aggregate risk (weighted sum)
        let mut raw_risk = (t90_normalized * 0.3)
            + (tox_risk * 0.4)
            + (micro_risk * 0.2)
            + (leachate_risk * 0.1)
            - (carbon_benefit * 0.2); // Benefit reduces risk
            
        raw_risk = raw_risk.clamp(0.0, 1.0);
        
        // Apply corridor normalization for final scaling
        corridors.normalize(raw_risk, 0.0, 1.0)
    }
}

/// Errors specific to material kinetics validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialError {
    InvalidKineticValue,
    PfasDetected,
    CorridorViolation,
    CalibrationDrift,
}

impl fmt::Display for MaterialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaterialError::InvalidKineticValue => write!(f, "Kinetic values must be non-negative"),
            MaterialError::PfasDetected => write!(f, "PFAS presence is a hard gate violation"),
            MaterialError::CorridorViolation => write!(f, "Material risk exceeds corridor bounds"),
            MaterialError::CalibrationDrift => write!(f, "Sensor calibration drift detected"),
        }
    }
}

// ============================================================================
// SAFE SUBSTRATE TRAIT (HARD GATE)
// ============================================================================

/// Hard gate trait forbidding unsafe substrates from instantiation
/// 
/// Implementors must prove their kinetics fall within safe corridor bands
/// before being allowed in Cyboquatic nodes. This shifts failure from the
/// field into tests/CI.
pub trait AntSafeSubstrate {
    /// Returns the material kinetics profile
    fn kinetics(&self) -> MaterialKinetics;
    
    /// Validates against corridor bands
    fn corridor_ok(&self, bands: &CorridorBands) -> bool {
        let risk = self.kinetics().compute_risk(bands);
        bands.corridor_ok(risk)
    }
    
    /// Returns the substrate identifier
    fn substrate_id(&self) -> u32;
}

/// Example implementation for a biodegradable filter media
pub struct BioFilterMedia {
    pub id: u32,
    pub kinetics: MaterialKinetics,
}

impl AntSafeSubstrate for BioFilterMedia {
    fn kinetics(&self) -> MaterialKinetics {
        self.kinetics
    }
    
    fn substrate_id(&self) -> u32 {
        self.id
    }
}

// ============================================================================
// CALIBRATION AND ADAPTIVE ADJUSTMENT
// ============================================================================

/// Tracks sensor calibration state for material mass loss monitoring
#[derive(Debug, Clone)]
pub struct MaterialCalibrationState {
    /// Rolling window of observed vs predicted mass loss
    error_history: [f64; MATERIAL_CALIBRATION_WINDOW],
    /// Current index in rolling window
    index: usize,
    /// Count of samples processed
    samples: usize,
    /// Drift coefficient (automatic adjustment factor)
    drift_coefficient: f64,
}

impl MaterialCalibrationState {
    pub fn new() -> Self {
        MaterialCalibrationState {
            error_history: [0.0; MATERIAL_CALIBRATION_WINDOW],
            index: 0,
            samples: 0,
            drift_coefficient: 1.0,
        }
    }
    
    /// Records a calibration sample (observed - predicted)
    pub fn record_sample(&mut self, error: f64) {
        self.error_history[self.index] = error;
        self.index = (self.index + 1) % MATERIAL_CALIBRATION_WINDOW;
        self.samples = self.samples.saturating_add(1);
        
        // Automatic adjustment: update drift coefficient if error trend detected
        if self.samples >= MATERIAL_CALIBRATION_WINDOW {
            self.update_drift_coefficient();
        }
    }
    
    /// Updates the drift coefficient based on rolling average error
    fn update_drift_coefficient(&mut self) {
        let sum: f64 = self.error_history.iter().sum();
        let avg_error = sum / MATERIAL_CALIBRATION_WINDOW as f64;
        
        // Simple proportional adjustment
        self.drift_coefficient = 1.0 + (avg_error * 0.1);
        self.drift_coefficient = self.drift_coefficient.clamp(0.8, 1.2);
    }
    
    /// Returns the current drift coefficient for parameter adjustment
    pub fn drift_coefficient(&self) -> f64 {
        self.drift_coefficient
    }
    
    /// Generates a calibration error scenario for stress testing
    pub fn generate_error_scenario(&self, scenario_type: CalibrationScenario) -> Vec<f64> {
        let mut scenarios = Vec::new();
        match scenario_type {
            CalibrationScenario::SensorDrift => {
                for i in 0..10 {
                    scenarios.push(0.05 * (i as f64)); // Linear drift
                }
            },
            CalibrationScenario::NoiseSpike => {
                for i in 0..10 {
                    if i == 5 {
                        scenarios.push(0.5); // Spike
                    } else {
                        scenarios.push(0.0);
                    }
                }
            },
            CalibrationScenario::BiasOffset => {
                for i in 0..10 {
                    scenarios.push(0.1); // Constant bias
                }
            },
        }
        scenarios
    }
    
    /// Checks if calibration drift exceeds safe thresholds
    pub fn is_calibrated(&self) -> bool {
        self.drift_coefficient >= 0.9 && self.drift_coefficient <= 1.1
    }
}

impl Default for MaterialCalibrationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of calibration error scenarios for robustness testing
#[derive(Debug, Clone, Copy)]
pub enum CalibrationScenario {
    SensorDrift,
    NoiseSpike,
    BiasOffset,
}

// ============================================================================
// PARAMETER ADJUSTMENT QUEUE
// ============================================================================

/// Queue for pending parameter adjustments to reduce residual risk
#[derive(Debug, Clone)]
pub struct ParameterAdjustmentQueue {
    /// Pending adjustments (key, value)
    queue: Vec<(String, f64)>,
    /// Maximum queue size
    max_size: usize,
}

impl ParameterAdjustmentQueue {
    pub fn new(max_size: usize) -> Self {
        ParameterAdjustmentQueue {
            queue: Vec::new(),
            max_size,
        }
    }
    
    /// Pushes a new adjustment request
    pub fn push(&mut self, key: String, value: f64) {
        if self.queue.len() >= self.max_size {
            self.queue.remove(0); // Drop oldest
        }
        self.queue.push((key, value));
    }
    
    /// Processes adjustments (called by ecosafety enforcer)
    pub fn process_adjustments(&mut self) -> Vec<(String, f64)> {
        let pending = self.queue.clone();
        self.queue.clear();
        pending
    }
    
    /// Returns queue depth
    pub fn depth(&self) -> usize {
        self.queue.len()
    }
}

// ============================================================================
// COMPOSITE MATERIAL AGGREGATION
// ============================================================================

/// Aggregates sub-risks of composite materials into a single r_materials
#[derive(Debug, Clone)]
pub struct CompositeMaterial {
    /// Component materials and their mass fractions
    components: Vec<(MaterialKinetics, f64)>,
    /// Total mass (kg)
    total_mass: f64,
}

impl CompositeMaterial {
    pub fn new(total_mass: f64) -> Self {
        CompositeMaterial {
            components: Vec::new(),
            total_mass,
        }
    }
    
    /// Adds a component material with mass fraction
    pub fn add_component(&mut self, kinetics: MaterialKinetics, mass_fraction: f64) -> Result<(), MaterialError> {
        if mass_fraction <= 0.0 || mass_fraction > 1.0 {
            return Err(MaterialError::InvalidKineticValue);
        }
        self.components.push((kinetics, mass_fraction));
        Ok(())
    }
    
    /// Computes aggregate risk coordinate
    pub fn aggregate_risk(&self, corridors: &CorridorBands) -> f64 {
        let mut weighted_risk = 0.0;
        let mut total_weight = 0.0;
        
        for (kinetics, weight) in &self.components {
            let risk = kinetics.compute_risk(corridors);
            weighted_risk += risk * weight;
            total_weight += weight;
        }
        
        if total_weight > 0.0 {
            (weighted_risk / total_weight).clamp(0.0, 1.0)
        } else {
            1.0 // Penalty for undefined composition
        }
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Validates a substrate against safety gates before instantiation
pub fn validate_substrate<S: AntSafeSubstrate>(substrate: &S, bands: &CorridorBands) -> Result<(), MaterialError> {
    if !substrate.corridor_ok(bands) {
        return Err(MaterialError::CorridorViolation);
    }
    if substrate.kinetics().pfas_present {
        return Err(MaterialError::PfasDetected);
    }
    Ok(())
}

/// Generates a diagnostic report for material kinetics
pub fn generate_material_diagnostics(kinetics: &MaterialKinetics, calibration: &MaterialCalibrationState) -> String {
    alloc::format!(
        "=== Material Kinetics Diagnostics ===\n\
         t90: {:.2} days\n\
         Toxicity: {:.3}\n\
         PFAS: {}\n\
         Calibration Drift: {:.3}\n\
         Calibrated: {}\n",
        kinetics.t90_days,
        kinetics.toxicity_index,
        kinetics.pfas_present,
        calibration.drift_coefficient(),
        calibration.is_calibrated()
    )
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_material_kinetics_risk() {
        let kinetics = MaterialKinetics {
            t90_days: 90.0,
            toxicity_index: 0.05,
            micro_residue_rate: 0.01,
            leachate_cec: 10.0,
            pfas_present: false,
            caloric_density: 15000.0,
            carbon_sequestration_potential: 0.5,
        };
        let corridors = CorridorBands::default();
        let risk = kinetics.compute_risk(&corridors);
        assert!(risk >= 0.0 && risk <= 1.0);
    }
    
    #[test]
    fn test_pfas_hard_gate() {
        let result = MaterialKinetics::new(
            90.0, 0.05, 0.01, 10.0, true, 15000.0, 0.5
        );
        assert_eq!(result, Err(MaterialError::PfasDetected));
    }
    
    #[test]
    fn test_calibration_drift() {
        let mut cal = MaterialCalibrationState::new();
        for i in 0..MATERIAL_CALIBRATION_WINDOW {
            cal.record_sample(0.1); // Consistent bias
        }
        assert!(!cal.is_calibrated()); // Should detect drift
    }
    
    #[test]
    fn test_composite_aggregation() {
        let mut comp = CompositeMaterial::new(1.0);
        let k1 = MaterialKinetics {
            t90_days: 50.0, toxicity_index: 0.1, micro_residue_rate: 0.0,
            leachate_cec: 5.0, pfas_present: false, caloric_density: 0.0,
            carbon_sequestration_potential: 0.2,
        };
        let k2 = MaterialKinetics {
            t90_days: 200.0, toxicity_index: 0.5, micro_residue_rate: 0.1,
            leachate_cec: 20.0, pfas_present: false, caloric_density: 0.0,
            carbon_sequestration_potential: 0.0,
        };
        comp.add_component(k1, 0.5).unwrap();
        comp.add_component(k2, 0.5).unwrap();
        
        let risk = comp.aggregate_risk(&CorridorBands::default());
        assert!(risk >= 0.0 && risk <= 1.0);
    }
}
