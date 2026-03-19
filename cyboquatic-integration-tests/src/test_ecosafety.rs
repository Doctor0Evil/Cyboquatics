//! Cyboquatic Integration Test Suite
//! 
//! Validates the full-stack safety contracts across Rust core, material kinetics,
//! and adaptive calibration systems. These tests serve as executable documentation
//! of the ecosafety invariants required for carbon-negative industrial machinery.
//! 
//! # Test Categories
//! 
//! 1. **Lyapunov Stability:** Verifies V_t non-increase invariant.
//! 2. **Material Hard Gates:** Ensures PFAS and toxicity rejections.
//! 3. **KER Governance:** Validates deployment thresholds.
//! 4. **Calibration Scenarios:** Stress-tests sensor drift robustness.
//! 
//! # Safety Guarantees
//! 
//! - All tests must pass before deployment artifacts are generated.
//! - Failure indicates a potential ecological risk vector.
//! 
//! @file test_ecosafety.rs
//! @destination cyboquatic-integration-tests/src/test_ecosafety.rs

#![cfg(test)]
#![allow(dead_code)]
#![allow(unused_variables)]

// Simulated imports from core crates (in actual project, these are external crates)
use cyboquatic_ecosafety_core::{
    CorridorBands, EcosafetyEnforcer, KERMetrics, RiskPlane, RiskVector, 
    SystemState, OperatingMode, DEFAULT_LYAPUNOV_EPSILON, K_THRESHOLD_DEPLOY, 
    E_THRESHOLD_DEPLOY, R_THRESHOLD_DEPLOY, KER_WINDOW_SIZE
};
use cyboquatic_ecosafety_core::material_kinetics::{
    MaterialKinetics, MaterialError, AntSafeSubstrate, MaterialCalibrationState,
    CalibrationScenario, ParameterAdjustmentQueue, DEFAULT_T90_MAX_DAYS
};

use std::vec::Vec;
use std::string::String;
use std::assert;

// ============================================================================
// TEST UTILITIES
// ============================================================================

/// Helper to create a valid risk vector with specific coordinates
fn make_risk_vector(timestamp: u64, coords: &[f64]) -> RiskVector {
    let mut rv = RiskVector::new(timestamp);
    for (i, &val) in coords.iter().enumerate() {
        if i < 8 {
            rv.set_coordinate(unsafe { std::mem::transmute::<u8, RiskPlane>(i as u8) }, val);
        }
    }
    rv
}

/// Helper to create default corridor bands
fn default_corridors() -> [CorridorBands; 8] {
    [CorridorBands::default(); 8]
}

// ============================================================================
// LYAPUNOV STABILITY TESTS
// ============================================================================

#[test]
fn test_lyapunov_stability_invariant_holds() {
    // Goal: Verify that safe actuations do not increase V_t beyond epsilon
    let mut enforcer = EcosafetyEnforcer::new();
    let mut current_state = SystemState::new(1000);
    
    // Set initial low risk state
    let mut initial_risk = RiskVector::new(1000);
    initial_risk.set_coordinate(RiskPlane::Energy, 0.1);
    initial_risk.set_coordinate(RiskPlane::Hydraulic, 0.1);
    initial_risk.set_coordinate(RiskPlane::Biology, 0.1);
    // ... set others low
    
    let initial_vt = initial_risk.lyapunov_residual(&enforcer.weights);
    current_state.current_v_t = initial_vt;
    
    // Propose a safe actuation (lower or equal risk)
    let mut proposed_risk = RiskVector::new(1001);
    proposed_risk.set_coordinate(RiskPlane::Energy, 0.05); // Reduced energy risk
    proposed_risk.set_coordinate(RiskPlane::Hydraulic, 0.1);
    proposed_risk.set_coordinate(RiskPlane::Biology, 0.1);
    
    let result = enforcer.enforce("safe_actuation", proposed_risk);
    
    assert!(result.is_ok(), "Safe actuation should be enforced");
    assert!(enforcer.current_lyapunov_residual() <= initial_vt + DEFAULT_LYAPUNOV_EPSILON);
}

#[test]
fn test_lyapunov_violation_rejects_actuation() {
    // Goal: Verify that unsafe actuations are rejected
    let mut enforcer = EcosafetyEnforcer::new();
    
    // Set current state to moderate risk
    let mut current_risk = RiskVector::new(1000);
    current_risk.set_coordinate(RiskPlane::Energy, 0.5);
    let initial_vt = current_risk.lyapunov_residual(&enforcer.weights);
    enforcer.current_v_t = initial_vt; // Manually set for test
    
    // Propose a high risk actuation
    let mut proposed_risk = RiskVector::new(1001);
    proposed_risk.set_coordinate(RiskPlane::Energy, 0.9); // Spike in energy risk
    
    let result = enforcer.enforce("unsafe_actuation", proposed_risk);
    
    assert!(result.is_err(), "Unsafe actuation should be rejected");
    assert_eq!(enforcer.current_lyapunov_residual(), initial_vt, "V_t should not update on rejection");
}

#[test]
fn test_lyapunov_weights_affect_stability() {
    // Goal: Verify that changing weights changes V_t calculation
    let mut enforcer = EcosafetyEnforcer::new();
    let mut rv = RiskVector::new(1000);
    rv.set_coordinate(RiskPlane::Biology, 0.5);
    
    let vt_default = rv.lyapunov_residual(&enforcer.weights);
    
    // Increase weight on Biology
    enforcer.set_weight(RiskPlane::Biology, 5.0);
    let vt_weighted = rv.lyapunov_residual(&enforcer.weights);
    
    assert!(vt_weighted > vt_default, "Higher weight should increase V_t contribution");
}

// ============================================================================
// MATERIAL KINETICS & HARD GATE TESTS
// ============================================================================

#[test]
fn test_pfas_hard_gate_rejection() {
    // Goal: Ensure PFAS presence is a hard gate violation (Shift failure to CI)
    let result = MaterialKinetics::new(
        90.0,   // t90
        0.05,   // toxicity
        0.01,   // micro_residue
        10.0,   // leachate
        true,   // pfas_present (VIOLATION)
        15000.0,
        0.5
    );
    
    assert_eq!(result, Err(MaterialError::PfasDetected));
}

#[test]
fn test_t90_threshold_enforcement() {
    // Goal: Verify slow-degrading materials are penalized in risk calculation
    let corridors = CorridorBands::default();
    
    // Fast degrading (Safe)
    let fast_mat = MaterialKinetics {
        t90_days: 30.0, toxicity_index: 0.05, micro_residue_rate: 0.0,
        leachate_cec: 5.0, pfas_present: false, caloric_density: 0.0,
        carbon_sequestration_potential: 0.5,
    };
    
    // Slow degrading (Risk)
    let slow_mat = MaterialKinetics {
        t90_days: 300.0, toxicity_index: 0.05, micro_residue_rate: 0.0,
        leachate_cec: 5.0, pfas_present: false, caloric_density: 0.0,
        carbon_sequestration_potential: 0.5,
    };
    
    let risk_fast = fast_mat.compute_risk(&corridors);
    let risk_slow = slow_mat.compute_risk(&corridors);
    
    assert!(risk_fast < risk_slow, "Slow degradation should yield higher risk");
    assert!(risk_slow >= 1.0 || risk_slow > risk_fast, "Slow material should approach hard limit");
}

#[test]
fn test_material_calibration_drift_detection() {
    // Goal: Verify adaptive calibration detects sensor drift
    let mut cal_state = MaterialCalibrationState::new();
    
    // Simulate consistent positive drift (sensor reading higher than actual)
    for _ in 0..60 {
        cal_state.record_sample(0.1);
    }
    
    assert!(!cal_state.is_calibrated(), "Calibration should detect drift");
    assert!(cal_state.drift_coefficient() > 1.0, "Drift coefficient should adjust upward");
}

// ============================================================================
// KER GOVERNANCE TESTS
// ============================================================================

#[test]
fn test_ker_metrics_deployment_gate() {
    // Goal: Verify KER thresholds prevent deployment of unsafe systems
    let mut metrics = KERMetrics::new();
    
    // Simulate high performance window
    for _ in 0..KER_WINDOW_SIZE {
        metrics.record_step(true, 0.05); // Safe steps, low risk
    }
    
    assert!(metrics.meets_deployment_thresholds());
    assert!(metrics.knowledge_factor() >= K_THRESHOLD_DEPLOY);
    assert!(metrics.eco_impact() >= E_THRESHOLD_DEPLOY);
    assert!(metrics.risk_of_harm() <= R_THRESHOLD_DEPLOY);
}

#[test]
fn test_ker_metrics_failure_detection() {
    // Goal: Verify KER metrics detect unsafe operation
    let mut metrics = KERMetrics::new();
    
    // Simulate risky window
    for _ in 0..KER_WINDOW_SIZE {
        metrics.record_step(false, 0.50); // Unsafe steps, high risk
    }
    
    assert!(!metrics.meets_deployment_thresholds());
    assert!(metrics.risk_of_harm() > R_THRESHOLD_DEPLOY);
}

#[test]
fn test_ker_rolling_window_behavior() {
    // Goal: Verify rolling window updates correctly
    let mut metrics = KERMetrics::new();
    
    // Fill window with safe steps
    for _ in 0..KER_WINDOW_SIZE {
        metrics.record_step(true, 0.1);
    }
    assert_eq!(metrics.knowledge_factor(), 1.0);
    
    // Add one unsafe step (should drop K slightly)
    metrics.record_step(false, 0.1);
    let expected_k = (KER_WINDOW_SIZE - 1) as f64 / KER_WINDOW_SIZE as f64;
    assert!(metrics.knowledge_factor() < 1.0);
    assert!((metrics.knowledge_factor() - expected_k).abs() < 0.001);
}

// ============================================================================
// CALIBRATION ERROR SCENARIO TESTS
// ============================================================================

#[test]
fn test_calibration_scenario_sensor_drift() {
    // Goal: Validate error scenario generation for stress testing
    let cal_state = MaterialCalibrationState::new();
    let scenarios = cal_state.generate_error_scenario(CalibrationScenario::SensorDrift);
    
    assert_eq!(scenarios.len(), 10);
    assert!(scenarios[0] < scenarios[9], "Drift should increase over time");
}

#[test]
fn test_calibration_scenario_noise_spike() {
    // Goal: Validate noise spike scenario generation
    let cal_state = MaterialCalibrationState::new();
    let scenarios = cal_state.generate_error_scenario(CalibrationScenario::NoiseSpike);
    
    assert_eq!(scenarios.len(), 10);
    assert_eq!(scenarios[5], 0.5); // Spike at index 5
    assert_eq!(scenarios[0], 0.0);
}

#[test]
fn test_parameter_adjustment_queue_safety() {
    // Goal: Verify adjustment queue respects bounds
    let mut queue = ParameterAdjustmentQueue::new(10);
    
    queue.push("corridor_bands.energy.safe_upper".to_string(), 0.4);
    queue.push("lyapunov_weights.default.energy".to_string(), 2.0);
    
    assert_eq!(queue.depth(), 2);
    
    let adjustments = queue.process_adjustments();
    assert_eq!(adjustments.len(), 2);
    assert_eq!(queue.depth(), 0); // Queue cleared after processing
}

// ============================================================================
// CROSS-LANGUAGE FIDELITY TESTS (Rust vs C++ Logic)
// ============================================================================

#[test]
fn test_corridor_normalization_consistency() {
    // Goal: Ensure Rust normalization matches C++ kernel logic exactly
    // This test documents the expected values for C++ verification
    let corridors = CorridorBands::new(0.3, 0.7, 1.0).unwrap();
    
    // Test Safe Zone
    let safe_val = corridors.normalize(0.15, 0.0, 1.0);
    assert!(safe_val < 0.3);
    
    // Test Gold Zone
    let gold_val = corridors.normalize(0.5, 0.0, 1.0);
    assert!(gold_val >= 0.3 && gold_val < 0.7);
    
    // Test Hard Zone
    let hard_val = corridors.normalize(0.9, 0.0, 1.0);
    assert!(hard_val >= 0.7);
    
    // Documented Expected Values for C++ Assertion
    // normalize(0.5) should be 0.15 + (0.5 - 0.3) * 1.25 = 0.15 + 0.25 = 0.40
    assert!((gold_val - 0.40).abs() < 0.001);
}

#[test]
fn test_risk_vector_aggregation() {
    // Goal: Verify risk vector aggregation matches Lyapunov formula
    let mut rv = RiskVector::new(1000);
    rv.set_coordinate(RiskPlane::Energy, 0.5);
    rv.set_coordinate(RiskPlane::Hydraulic, 0.5);
    
    let mut weights = [0.0; 8];
    weights[0] = 2.0; // Energy
    weights[1] = 2.0; // Hydraulic
    
    let vt = rv.lyapunov_residual(&weights);
    // Expected: 2.0 * 0.5^2 + 2.0 * 0.5^2 = 2.0 * 0.25 + 2.0 * 0.25 = 0.5 + 0.5 = 1.0
    assert!((vt - 1.0).abs() < 0.001);
}

// ============================================================================
// ECOLOGICAL IMPACT TESTS
// ============================================================================

#[test]
fn test_carbon_negative_incentive() {
    // Goal: Verify carbon sequestration reduces material risk
    let corridors = CorridorBands::default();
    
    let mut mat_low_seq = MaterialKinetics {
        t90_days: 90.0, toxicity_index: 0.05, micro_residue_rate: 0.0,
        leachate_cec: 5.0, pfas_present: false, caloric_density: 0.0,
        carbon_sequestration_potential: 0.1, // Low sequestration
    };
    
    let mut mat_high_seq = MaterialKinetics {
        t90_days: 90.0, toxicity_index: 0.05, micro_residue_rate: 0.0,
        leachate_cec: 5.0, pfas_present: false, caloric_density: 0.0,
        carbon_sequestration_potential: 0.9, // High sequestration
    };
    
    let risk_low = mat_low_seq.compute_risk(&corridors);
    let risk_high = mat_high_seq.compute_risk(&corridors);
    
    assert!(risk_high < risk_low, "Higher sequestration should reduce risk");
}

#[test]
fn test_composite_material_aggregation() {
    // Goal: Verify composite materials aggregate risks correctly
    use cyboquatic_ecosafety_core::material_kinetics::CompositeMaterial;
    
    let mut comp = CompositeMaterial::new(1.0);
    
    // Safe component
    let safe_kinetics = MaterialKinetics {
        t90_days: 30.0, toxicity_index: 0.0, micro_residue_rate: 0.0,
        leachate_cec: 0.0, pfas_present: false, caloric_density: 0.0,
        carbon_sequestration_potential: 1.0,
    };
    
    // Risky component
    let risky_kinetics = MaterialKinetics {
        t90_days: 180.0, toxicity_index: 0.5, micro_residue_rate: 0.1,
        leachate_cec: 20.0, pfas_present: false, caloric_density: 0.0,
        carbon_sequestration_potential: 0.0,
    };
    
    comp.add_component(safe_kinetics, 0.5).unwrap();
    comp.add_component(risky_kinetics, 0.5).unwrap();
    
    let risk = comp.aggregate_risk(&CorridorBands::default());
    
    // Risk should be between the two components
    assert!(risk > 0.0 && risk < 1.0);
}

// ============================================================================
// STRESS TESTS
// ============================================================================

#[test]
fn test_enforcer_under_high_load() {
    // Goal: Verify enforcer stability under rapid successive calls
    let mut enforcer = EcosafetyEnforcer::new();
    let mut risk = RiskVector::new(0);
    risk.set_coordinate(RiskPlane::Energy, 0.1);
    
    for i in 0..10000 {
        risk.timestamp = i;
        let result = enforcer.enforce("actuation", risk.clone());
        assert!(result.is_ok(), "Enforcer failed at iteration {}", i);
    }
}

#[test]
fn test_ker_metrics_warmup_behavior() {
    // Goal: Verify KER metrics handle warmup period correctly
    let mut metrics = KERMetrics::new();
    
    // Less than window size
    for _ in 0..10 {
        metrics.record_step(true, 0.1);
    }
    
    // Should not panic, should calculate based on available data
    let k = metrics.knowledge_factor();
    assert!(k >= 0.0 && k <= 1.0);
}

// ============================================================================
// DOCUMENTATION TESTS
// ============================================================================

#[test]
fn test_documentation_thresholds_match_code() {
    // Goal: Ensure documented thresholds match constants
    // This prevents documentation drift
    assert!(K_THRESHOLD_DEPLOY >= 0.90);
    assert!(E_THRESHOLD_DEPLOY >= 0.90);
    assert!(R_THRESHOLD_DEPLOY <= 0.13);
    
    // Log for verification in CI output
    println!("Deployment Thresholds: K>={}, E>={}, R<={}", 
             K_THRESHOLD_DEPLOY, E_THRESHOLD_DEPLOY, R_THRESHOLD_DEPLOY);
}
