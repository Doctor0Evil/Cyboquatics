#![cfg(test)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use cyboquatic_ecosafety_core::{RiskVector, Residual, LyapunovWeights, KerWindow, SafeStepDecision};
use cyboquatic_brain_identity_core::{
    BrainIdentityShard, BiRiskVector, BiResidual, BiLyapunovWeights, 
    BiSafeStepDecision, BiKerWindow, NeurorightsStatus, EvidenceMode,
    bi_safestep, BiSafeStepConfig
};
use cyboquatic_bi_integration::{
    IntegratedController, IntegratedDecision, IntegratedKerTracker, 
    BiAuditLogEntry, BiAuditLogger
};
use cyboquatic_bi_simulation::{
    SimulationConfig, BiSimulationRunner, SimulationReport, NullBiAuditLogger
};

const EPSILON: f32 = 0.001;
const KARMA_INITIAL: f32 = 100.0;
const STEPS_STANDARD: u32 = 100;

fn generate_test_identity_id() -> [u8; 32] {
    let mut id = [0u8; 32];
    for (i, byte) in id.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    id
}

fn generate_test_hexstamp() -> [u8; 32] {
    let mut stamp = [0u8; 32];
    for (i, byte) in stamp.iter_mut().enumerate() {
        *byte = ((i + 100) % 256) as u8;
    }
    stamp
}

fn default_physical_weights() -> LyapunovWeights {
    LyapunovWeights {
        w_energy: 0.20,
        w_hydraulic: 0.15,
        w_biology: 0.15,
        w_carbon: 0.25,
        w_materials: 0.25,
    }
}

fn default_bi_weights() -> BiLyapunovWeights {
    BiLyapunovWeights::default()
}

fn create_test_shard(karma_floor: f32) -> BrainIdentityShard {
    BrainIdentityShard::new(
        generate_test_identity_id(),
        generate_test_hexstamp(),
        karma_floor,
    )
}

fn create_physical_risk_vector(energy: f32, hydraulic: f32, biology: f32, carbon: f32, materials: f32) -> RiskVector {
    RiskVector {
        r_energy: cyboquatic_ecosafety_core::RiskCoord::new_clamped(energy),
        r_hydraulic: cyboquatic_ecosafety_core::RiskCoord::new_clamped(hydraulic),
        r_biology: cyboquatic_ecosafety_core::RiskCoord::new_clamped(biology),
        r_carbon: cyboquatic_ecosafety_core::RiskCoord::new_clamped(carbon),
        r_materials: cyboquatic_ecosafety_core::RiskCoord::new_clamped(materials),
    }
}

#[test]
fn test_brain_identity_shard_creation() {
    let shard = create_test_shard(KARMA_INITIAL);
    
    assert_eq!(shard.karma_floor, KARMA_INITIAL);
    assert_eq!(shard.neurorights_status, NeurorightsStatus::Active);
    assert_eq!(shard.evidence_mode, EvidenceMode::Redacted);
    assert_eq!(shard.data_sensitivity_level, 1);
    assert_eq!(shard.ecoimpactscore, 0.0);
    assert_eq!(shard.rsoul_residual, 0.0);
    assert_eq!(shard.social_exposure_coord, 0.0);
}

#[test]
fn test_karma_nonslash_invariant() {
    let mut shard = create_test_shard(KARMA_INITIAL);
    
    assert!(shard.try_set_karma_floor(100.0));
    assert_eq!(shard.karma_floor, 100.0);
    
    assert!(shard.try_set_karma_floor(101.0));
    assert_eq!(shard.karma_floor, 101.0);
    
    assert!(shard.try_set_karma_floor(150.0));
    assert_eq!(shard.karma_floor, 150.0);
    
    assert!(!shard.try_set_karma_floor(149.0));
    assert_eq!(shard.karma_floor, 150.0);
    
    assert!(!shard.try_set_karma_floor(100.0));
    assert_eq!(shard.karma_floor, 150.0);
    
    assert!(!shard.try_set_karma_floor(0.0));
    assert_eq!(shard.karma_floor, 150.0);
}

#[test]
fn test_neurorights_status_risk_mapping() {
    let mut shard = create_test_shard(KARMA_INITIAL);
    
    shard.update_neurorights(NeurorightsStatus::Active);
    assert_eq!(shard.r_neurorights().value(), 0.0);
    
    shard.update_neurorights(NeurorightsStatus::Restricted);
    assert_eq!(shard.r_neurorights().value(), 0.5);
    
    shard.update_neurorights(NeurorightsStatus::Suspended);
    assert_eq!(shard.r_neurorights().value(), 1.0);
}

#[test]
fn test_bi_risk_vector_computation() {
    let physical_rv = create_physical_risk_vector(0.3, 0.2, 0.25, 0.35, 0.28);
    let mut shard = create_test_shard(KARMA_INITIAL);
    
    shard.update_rsoul(0.08);
    shard.update_social_exposure(0.15);
    shard.update_ecoimpactscore(0.12);
    shard.update_neurorights(NeurorightsStatus::Active);
    
    let bi_rv = BiRiskVector::from_physical_and_bi(&physical_rv, &shard);
    
    assert_eq!(bi_rv.r_energy.value(), 0.3);
    assert_eq!(bi_rv.r_hydraulic.value(), 0.2);
    assert_eq!(bi_rv.r_biology.value(), 0.25);
    assert_eq!(bi_rv.r_carbon.value(), 0.35);
    assert_eq!(bi_rv.r_materials.value(), 0.28);
    assert_eq!(bi_rv.r_neurorights.value(), 0.0);
    assert_eq!(bi_rv.r_soul.value(), 0.08);
    assert_eq!(bi_rv.r_social.value(), 0.15);
    assert_eq!(bi_rv.r_ecoimpact.value(), 0.12);
}

#[test]
fn test_bi_residual_computation() {
    let mut bi_rv = BiRiskVector::default();
    bi_rv.r_energy = cyboquatic_ecosafety_core::RiskCoord::new_clamped(0.3);
    bi_rv.r_neurorights = cyboquatic_ecosafety_core::RiskCoord::new_clamped(0.0);
    bi_rv.r_soul = cyboquatic_ecosafety_core::RiskCoord::new_clamped(0.1);
    bi_rv.r_social = cyboquatic_ecosafety_core::RiskCoord::new_clamped(0.15);
    bi_rv.r_ecoimpact = cyboquatic_ecosafety_core::RiskCoord::new_clamped(0.12);
    
    let weights = BiLyapunovWeights::default();
    let residual = BiResidual::compute(&bi_rv, &weights);
    
    assert!(residual.vt >= 0.0);
    assert!(residual.vt < 1.0);
}

#[test]
fn test_bi_safestep_accept_scenario() {
    let prev_residual = BiResidual { vt: 0.08 };
    let next_residual = BiResidual { vt: 0.075 };
    
    let mut bi_rv = BiRiskVector::default();
    bi_rv.r_neurorights = cyboquatic_ecosafety_core::RiskCoord::new_clamped(0.0);
    bi_rv.r_soul = cyboquatic_ecosafety_core::RiskCoord::new_clamped(0.1);
    
    let config = BiSafeStepConfig {
        epsilon: EPSILON,
        enforce_karma_nonslash: true,
    };
    
    let decision = bi_safestep(
        prev_residual,
        next_residual,
        &bi_rv,
        100.0,
        100.5,
        &config,
    );
    
    assert_eq!(decision, BiSafeStepDecision::Accept);
}

#[test]
fn test_bi_safestep_karma_violation() {
    let prev_residual = BiResidual { vt: 0.08 };
    let next_residual = BiResidual { vt: 0.075 };
    
    let mut bi_rv = BiRiskVector::default();
    bi_rv.r_neurorights = cyboquatic_ecosafety_core::RiskCoord::new_clamped(0.0);
    
    let config = BiSafeStepConfig {
        epsilon: EPSILON,
        enforce_karma_nonslash: true,
    };
    
    let decision = bi_safestep(
        prev_residual,
        next_residual,
        &bi_rv,
        100.0,
        95.0,
        &config,
    );
    
    assert_eq!(decision, BiSafeStepDecision::StopKarmaViolation);
}

#[test]
fn test_bi_safestep_vt_increase_derate() {
    let prev_residual = BiResidual { vt: 0.08 };
    let next_residual = BiResidual { vt: 0.09 };
    
    let mut bi_rv = BiRiskVector::default();
    bi_rv.r_neurorights = cyboquatic_ecosafety_core::RiskCoord::new_clamped(0.0);
    
    let config = BiSafeStepConfig {
        epsilon: EPSILON,
        enforce_karma_nonslash: true,
    };
    
    let decision = bi_safestep(
        prev_residual,
        next_residual,
        &bi_rv,
        100.0,
        100.5,
        &config,
    );
    
    assert_eq!(decision, BiSafeStepDecision::Derate);
}

#[test]
fn test_bi_safestep_hard_violation_stop() {
    let prev_residual = BiResidual { vt: 0.08 };
    let next_residual = BiResidual { vt: 0.075 };
    
    let mut bi_rv = BiRiskVector::default();
    bi_rv.r_neurorights = cyboquatic_ecosafety_core::RiskCoord::new_clamped(1.0);
    
    let config = BiSafeStepConfig {
        epsilon: EPSILON,
        enforce_karma_nonslash: true,
    };
    
    let decision = bi_safestep(
        prev_residual,
        next_residual,
        &bi_rv,
        100.0,
        100.5,
        &config,
    );
    
    assert_eq!(decision, BiSafeStepDecision::Stop);
}

#[test]
fn test_integrated_controller_accept() {
    let controller = IntegratedController::new(
        default_physical_weights(),
        default_bi_weights(),
        EPSILON,
        true,
    );
    
    let physical_rv = create_physical_risk_vector(0.3, 0.2, 0.25, 0.35, 0.28);
    let physical_residual = Residual::compute(&physical_rv, &default_physical_weights());
    let mut bi_shard = create_test_shard(KARMA_INITIAL);
    
    let result = controller.evaluate_step(
        &physical_rv,
        physical_residual,
        &bi_shard,
        KARMA_INITIAL + 1.0,
    );
    
    assert_eq!(result.combined_decision, IntegratedDecision::Accept);
    assert!(result.karma_preserved);
}

#[test]
fn test_integrated_controller_karma_violation() {
    let controller = IntegratedController::new(
        default_physical_weights(),
        default_bi_weights(),
        EPSILON,
        true,
    );
    
    let physical_rv = create_physical_risk_vector(0.3, 0.2, 0.25, 0.35, 0.28);
    let physical_residual = Residual::compute(&physical_rv, &default_physical_weights());
    let bi_shard = create_test_shard(KARMA_INITIAL);
    
    let result = controller.evaluate_step(
        &physical_rv,
        physical_residual,
        &bi_shard,
        KARMA_INITIAL - 10.0,
    );
    
    assert_eq!(result.combined_decision, IntegratedDecision::StopKarmaViolation);
    assert!(!result.karma_preserved);
}

#[test]
fn test_integrated_ker_tracker() {
    let mut tracker = IntegratedKerTracker::default();
    let controller = IntegratedController::new(
        default_physical_weights(),
        default_bi_weights(),
        EPSILON,
        true,
    );
    
    let physical_rv = create_physical_risk_vector(0.3, 0.2, 0.25, 0.35, 0.28);
    let physical_residual = Residual::compute(&physical_rv, &default_physical_weights());
    let mut bi_shard = create_test_shard(KARMA_INITIAL);
    
    for _ in 0..10 {
        let result = controller.evaluate_step(
            &physical_rv,
            physical_residual,
            &bi_shard,
            KARMA_INITIAL,
        );
        tracker.update(&result);
    }
    
    let summary = tracker.export_ker_summary();
    assert!(summary.combined_deployable);
    assert!(summary.karma_preserved);
    assert_eq!(summary.physical_k, 1.0);
    assert_eq!(summary.bi_k, 1.0);
}

#[test]
fn test_simulation_karma_invariant_held() {
    let config = SimulationConfig {
        total_steps: STEPS_STANDARD,
        epsilon: EPSILON,
        enforce_karma: true,
        perturb_physical: true,
        perturb_bi: true,
        perturb_magnitude: 0.02,
    };
    
    let mut runner = BiSimulationRunner::new(
        config,
        generate_test_identity_id(),
        generate_test_hexstamp(),
        KARMA_INITIAL,
        NullBiAuditLogger,
    );
    
    let report = runner.run_simulation();
    
    assert!(report.completed_steps == report.total_steps);
    assert!(report.invariant_held);
    assert!(report.final_karma_floor >= KARMA_INITIAL);
}

#[test]
fn test_simulation_vt_stability() {
    let config = SimulationConfig {
        total_steps: STEPS_STANDARD,
        epsilon: EPSILON,
        enforce_karma: true,
        perturb_physical: false,
        perturb_bi: false,
        perturb_magnitude: 0.0,
    };
    
    let mut runner = BiSimulationRunner::new(
        config,
        generate_test_identity_id(),
        generate_test_hexstamp(),
        KARMA_INITIAL,
        NullBiAuditLogger,
    );
    
    let report = runner.run_simulation();
    
    assert!(report.vt_violations == 0 || report.vt_violations <= 5);
    assert!(report.final_deployable);
}

#[test]
fn test_audit_log_entry_generation() {
    let bi_shard = create_test_shard(KARMA_INITIAL);
    let vt_previous = 0.075;
    let vt_current = 0.078;
    
    let entry = BiAuditLogEntry::new(
        1704067200,
        &bi_shard,
        vt_previous,
        vt_current,
        IntegratedDecision::Accept,
        KARMA_INITIAL,
        true,
    );
    
    assert_eq!(entry.vt_previous, vt_previous);
    assert_eq!(entry.vt_current, vt_current);
    assert_eq!(entry.vt_delta, vt_current - vt_previous);
    assert!(!entry.karma_violated());
    assert!(!entry.vt_violated(EPSILON));
}

#[test]
fn test_audit_log_karma_violation_detection() {
    let mut bi_shard = create_test_shard(KARMA_INITIAL);
    bi_shard.karma_floor = 100.0;
    
    let entry = BiAuditLogEntry::new(
        1704067200,
        &bi_shard,
        0.075,
        0.078,
        IntegratedDecision::StopKarmaViolation,
        90.0,
        false,
    );
    
    assert!(entry.karma_violated());
}

#[test]
fn test_bi_ker_window_deployable_threshold() {
    let mut ker = BiKerWindow::default();
    let mut bi_rv = BiRiskVector::default();
    
    for i in 0..100 {
        let decision = if i < 90 {
            BiSafeStepDecision::Accept
        } else {
            BiSafeStepDecision::Derate
        };
        
        ker.update(&bi_rv, decision, true);
    }
    
    assert!(ker.k() >= 0.90);
    assert!(ker.e() >= 0.90);
    assert!(ker.r() <= 0.13);
    assert!(ker.karma_preserved);
    assert!(ker.bi_ker_deployable());
}

#[test]
fn test_bi_ker_window_karma_preservation() {
    let mut ker = BiKerWindow::default();
    let bi_rv = BiRiskVector::default();
    
    ker.update(&bi_rv, BiSafeStepDecision::Accept, true);
    ker.update(&bi_rv, BiSafeStepDecision::Accept, true);
    ker.update(&bi_rv, BiSafeStepDecision::Accept, true);
    ker.update(&bi_rv, BiSafeStepDecision::StopKarmaViolation, false);
    ker.update(&bi_rv, BiSafeStepDecision::Accept, true);
    
    assert!(!ker.karma_preserved);
    assert!(!ker.bi_ker_deployable());
}

#[test]
fn test_risk_coordinate_clamping() {
    let mut shard = create_test_shard(KARMA_INITIAL);
    
    shard.update_rsoul(-0.5);
    assert_eq!(shard.rsoul_residual, 0.0);
    
    shard.update_rsoul(1.5);
    assert_eq!(shard.rsoul_residual, 1.0);
    
    shard.update_social_exposure(-0.3);
    assert_eq!(shard.social_exposure_coord, 0.0);
    
    shard.update_social_exposure(2.0);
    assert_eq!(shard.social_exposure_coord, 1.0);
    
    shard.update_ecoimpactscore(-1.0);
    assert_eq!(shard.ecoimpactscore, 0.0);
    
    shard.update_ecoimpactscore(5.0);
    assert_eq!(shard.ecoimpactscore, 1.0);
}

#[test]
fn test_evidence_mode_enum_conversion() {
    use cyboquatic_brain_identity_core::EvidenceMode;
    
    assert_eq!(EvidenceMode::from_u8(0), Some(EvidenceMode::Redacted));
    assert_eq!(EvidenceMode::from_u8(1), Some(EvidenceMode::HashOnly));
    assert_eq!(EvidenceMode::from_u8(2), Some(EvidenceMode::FullTrace));
    assert_eq!(EvidenceMode::from_u8(3), None);
    assert_eq!(EvidenceMode::from_u8(255), None);
}

#[test]
fn test_neurorights_status_enum_conversion() {
    use cyboquatic_brain_identity_core::NeurorightsStatus;
    
    assert_eq!(NeurorightsStatus::from_u8(0), Some(NeurorightsStatus::Active));
    assert_eq!(NeurorightsStatus::from_u8(1), Some(NeurorightsStatus::Restricted));
    assert_eq!(NeurorightsStatus::from_u8(2), Some(NeurorightsStatus::Suspended));
    assert_eq!(NeurorightsStatus::from_u8(3), None);
    assert_eq!(NeurorightsStatus::from_u8(255), None);
}

#[test]
fn test_long_horizon_simulation_stability() {
    let config = SimulationConfig {
        total_steps: 1000,
        epsilon: EPSILON,
        enforce_karma: true,
        perturb_physical: true,
        perturb_bi: true,
        perturb_magnitude: 0.01,
    };
    
    let mut runner = BiSimulationRunner::new(
        config,
        generate_test_identity_id(),
        generate_test_hexstamp(),
        KARMA_INITIAL,
        NullBiAuditLogger,
    );
    
    let report = runner.run_simulation();
    
    assert!(report.completed_steps == 1000);
    assert!(report.avg_vt_physical < 0.5);
    assert!(report.avg_vt_bi < 0.5);
    assert!(report.final_deployable || report.karma_violations_enforced > 0);
}

#[test]
fn test_concurrent_shard_operations() {
    let mut shards: Vec<BrainIdentityShard> = Vec::new();
    
    for i in 0..10 {
        let mut shard = create_test_shard(KARMA_INITIAL + (i as f32 * 10.0));
        shard.update_rsoul(0.05 * (i as f32));
        shard.update_social_exposure(0.03 * (i as f32));
        shards.push(shard);
    }
    
    for shard in &shards {
        assert!(shard.karma_floor >= KARMA_INITIAL);
        assert!(shard.rsoul_residual <= 0.5);
        assert!(shard.social_exposure_coord <= 0.3);
    }
}

#[test]
fn test_bi_risk_vector_hard_violation_detection() {
    let mut bi_rv = BiRiskVector::default();
    
    assert!(!bi_rv.any_hard_violation());
    
    bi_rv.r_neurorights = cyboquatic_ecosafety_core::RiskCoord::new_clamped(1.0);
    assert!(bi_rv.any_hard_violation());
    
    bi_rv.r_neurorights = cyboquatic_ecosafety_core::RiskCoord::new_clamped(0.5);
    bi_rv.r_soul = cyboquatic_ecosafety_core::RiskCoord::new_clamped(1.0);
    assert!(bi_rv.any_hard_violation());
    
    bi_rv.r_soul = cyboquatic_ecosafety_core::RiskCoord::new_clamped(0.5);
    bi_rv.r_social = cyboquatic_ecosafety_core::RiskCoord::new_clamped(1.0);
    assert!(bi_rv.any_hard_violation());
}

#[test]
fn test_simulation_report_generation() {
    let config = SimulationConfig {
        total_steps: 50,
        epsilon: EPSILON,
        enforce_karma: true,
        perturb_physical: true,
        perturb_bi: true,
        perturb_magnitude: 0.05,
    };
    
    let mut runner = BiSimulationRunner::new(
        config,
        generate_test_identity_id(),
        generate_test_hexstamp(),
        KARMA_INITIAL,
        NullBiAuditLogger,
    );
    
    let report = runner.run_simulation();
    
    let summary = report.summary_string();
    assert!(summary.contains("SimulationReport"));
    assert!(summary.contains("steps="));
    assert!(summary.contains("vt_phys="));
    assert!(summary.contains("vt_bi="));
    assert!(summary.contains("karma="));
}

#[test]
fn test_integrated_decision_combination_logic() {
    use cyboquatic_bi_integration::IntegratedController;
    use cyboquatic_ecosafety_core::SafeStepDecision;
    use cyboquatic_brain_identity_core::BiSafeStepDecision;
    
    assert_eq!(
        IntegratedController::<(), ()>::combine_decisions(
            SafeStepDecision::Accept,
            BiSafeStepDecision::Accept
        ),
        IntegratedDecision::Accept
    );
    
    assert_eq!(
        IntegratedController::<(), ()>::combine_decisions(
            SafeStepDecision::Accept,
            BiSafeStepDecision::StopKarmaViolation
        ),
        IntegratedDecision::StopKarmaViolation
    );
    
    assert_eq!(
        IntegratedController::<(), ()>::combine_decisions(
            SafeStepDecision::Stop,
            BiSafeStepDecision::Accept
        ),
        IntegratedDecision::Stop
    );
    
    assert_eq!(
        IntegratedController::<(), ()>::combine_decisions(
            SafeStepDecision::Derate,
            BiSafeStepDecision::Accept
        ),
        IntegratedDecision::Derate
    );
}
