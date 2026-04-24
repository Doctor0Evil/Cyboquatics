//! Integration tests for Cyboquatic industrial shard validation and lane gating.
//!
//! These tests load fixture shards from `tests/data/` and verify:
//! - ALN schema alignment
//! - Admissibility validation (corridor, residual, KER thresholds)
//! - Lane-gated actuation behavior
//! - Controller step proposal behavior per lane

use cyboquatic_industrial_shards::{
    CyboNodeShard, validate_admissibility, ToRiskVector, ToLyapunovWeights,
};
use cyboquatic_ecosafety_core::compute_residual;
use std::fs;

/// Load a JSON fixture into a CyboNodeShard
fn load_fixture(path: &str) -> CyboNodeShard {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e));
    
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", path, e))
}

#[test]
fn test_research_lane_no_corridor_is_inadmissible() {
    let shard = load_fixture("tests/data/research_no_corridor.json");
    
    assert_eq!(shard.lane, cyboquatic_industrial_shards::Lane::RESEARCH);
    assert!(!shard.corridorpresent);
    
    let result = validate_admissibility(&shard);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, cyboquatic_industrial_shards::AdmissibilityError::CorridorMissing));
}

#[test]
fn test_pilot_lane_admissible() {
    let shard = load_fixture("tests/data/pilot_admissible.json");
    
    assert_eq!(shard.lane, cyboquatic_industrial_shards::Lane::EXPERIMENTAL);
    assert!(shard.corridorpresent);
    
    let result = validate_admissibility(&shard);
    assert!(result.is_ok());
    
    // Verify risk vector conversion
    let risk_vector = shard.to_risk_vector();
    assert!(risk_vector.r_energy.value >= 0.0);
    
    // Verify Lyapunov weights conversion
    let weights = shard.to_lyapunov_weights();
    assert!(weights.w_energy > 0.0);
}

#[test]
fn test_production_lane_admissible() {
    let shard = load_fixture("tests/data/production_admissible.json");
    
    assert_eq!(shard.lane, cyboquatic_industrial_shards::Lane::PRODUCTION);
    assert!(shard.corridorpresent);
    
    let result = validate_admissibility(&shard);
    assert!(result.is_ok());
    
    // Compute residual and verify it's within bounds
    let risk_vector = shard.to_risk_vector();
    let weights = shard.to_lyapunov_weights();
    let residual = compute_residual(&risk_vector.into(), &weights);
    
    assert!(residual.v_t <= shard.vresidualmax);
}

#[test]
fn test_production_residual_exceeded_is_inadmissible() {
    let shard = load_fixture("tests/data/production_residual_exceeded.json");
    
    assert_eq!(shard.lane, cyboquatic_industrial_shards::Lane::PRODUCTION);
    assert!(shard.corridorpresent);
    
    let result = validate_admissibility(&shard);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, cyboquatic_industrial_shards::AdmissibilityError::ResidualExceeded { .. }));
}

#[test]
fn test_ker_thresholds_by_lane() {
    // RESEARCH: K < 0.90, E < 0.90, R > 0.15 -> should fail production thresholds
    let research_shard = load_fixture("tests/data/research_no_corridor.json");
    assert!(research_shard.kknowledge < 0.90);
    assert!(research_shard.rriskofharm > 0.13);
    
    // PRODUCTION admissible: K >= 0.94, E >= 0.91, R <= 0.13
    let prod_shard = load_fixture("tests/data/production_admissible.json");
    assert!(prod_shard.kknowledge >= 0.94);
    assert!(prod_shard.eecoimpact >= 0.91);
    assert!(prod_shard.rriskofharm <= 0.13);
}

#[test]
fn test_lane_permits_actuation() {
    use cyboquatic_industrial_shards::lane_permits_actuation;
    
    let research_shard = load_fixture("tests/data/research_no_corridor.json");
    assert!(!lane_permits_actuation(research_shard.lane));
    
    let pilot_shard = load_fixture("tests/data/pilot_admissible.json");
    // EXPERIMENTAL/PILOT does not permit actuation (diagnostics only)
    assert!(!lane_permits_actuation(pilot_shard.lane));
    
    let prod_shard = load_fixture("tests/data/production_admissible.json");
    assert!(lane_permits_actuation(prod_shard.lane));
}
