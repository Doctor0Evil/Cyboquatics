//! Lane-aware controller wrapper that gates actuation based on KER and lane.

use crate::{
    IndustrialSafeController, NodeStateView, CommandEnvelope as CoreCommandEnvelope,
    IndustrialRiskVector, Lane, NodeClass, MediumClass,
};
use cyboquatic_industrial_shards::{
    CyboNodeShard, ToRiskVector, ToLyapunovWeights, Lane as ShardLane,
    validate_admissibility, AdmissibilityError, lane_permits_actuation,
};
use ecosafety_core::{Residual, compute_residual};

/// Error types for lane-gated controller operations
#[derive(Debug, Clone, PartialEq)]
pub enum LaneGateError {
    AdmissibilityFailed(AdmissibilityError),
    LaneNotActuating,
    KerThresholdsNotMet { k: f64, e: f64, r: f64 },
}

/// Lane-gated wrapper around an industrial controller
/// 
/// This wrapper enforces:
/// - Only Production lane permits actuation
/// - KER thresholds must be met for the lane
/// - Admissibility checks must pass before any command is emitted
pub struct LaneGatedController<C, S> {
    inner: C,
    _state_marker: std::marker::PhantomData<S>,
}

impl<C, S> LaneGatedController<C, S> 
where
    C: IndustrialSafeController<S, S::Command>,
    S: ShardNodeState,
{
    pub fn new(controller: C) -> Self {
        Self {
            inner: controller,
            _state_marker: std::marker::PhantomData,
        }
    }

    /// Attempt to propose a step, gated by lane and admissibility
    /// 
    /// Returns None if the lane does not permit actuation or admissibility fails
    pub fn try_propose_step(&self, shard: &CyboNodeShard, state: &S) 
        -> Result<Option<(S::Command, IndustrialRiskVector)>, LaneGateError> 
    {
        // Validate admissibility first
        if let Err(e) = validate_admissibility(shard) {
            return Err(LaneGateError::AdmissibilityFailed(e));
        }

        // Check if lane permits actuation
        if !lane_permits_actuation(shard.lane) {
            return Ok(None); // Diagnostics-only mode
        }

        // Check KER thresholds
        let thresholds = cyboquatic_industrial_shards::lane_ker_thresholds(shard.lane);
        if !thresholds.meets_thresholds(shard) {
            return Err(LaneGateError::KerThresholdsNotMet {
                k: shard.kknowledge,
                e: shard.eecoimpact,
                r: shard.rriskofharm,
            });
        }

        // All checks passed, allow the controller to propose a step
        let (cmd, risk_vec) = self.inner.propose_step(state);
        Ok(Some((cmd, risk_vec)))
    }

    /// Get diagnostics without attempting actuation
    /// 
    /// This works in any lane and returns risk estimates without commands
    pub fn get_diagnostics(&self, shard: &CyboNodeShard, state: &S) 
        -> (IndustrialRiskVector, Residual, bool) 
    {
        let (cmd, risk_vec) = self.inner.propose_step(state);
        
        let weights = shard.to_lyapunov_weights();
        let base_rv = ecosafety_core::RiskVector {
            r_energy: risk_vec.energy,
            r_hydraulics: risk_vec.hydraulics,
            r_biology: risk_vec.biology,
            r_carbon: risk_vec.carbon,
            r_materials: risk_vec.materials,
            r_biodiversity: ecosafety_core::RiskCoord::new_clamped(0.0),
            r_sigma: ecosafety_core::RiskCoord::new_clamped(0.0),
        };
        let residual = compute_residual(&base_rv, &weights);
        
        let can_actuate = lane_permits_actuation(shard.lane)
            && shard.corridorpresent
            && shard.vresidual <= shard.vresidualmax;

        (risk_vec, residual, can_actuate)
    }

    /// Access the inner controller for configuration
    pub fn inner(&self) -> &C {
        &self.inner
    }

    /// Access the inner controller mutably
    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.inner
    }
}

/// Trait bridging shard state to controller state
pub trait ShardNodeState {
    type Command: CoreCommandEnvelope;
    
    fn node_class(&self) -> NodeClass;
    fn medium(&self) -> MediumClass;
    fn current_risks(&self) -> IndustrialRiskVector;
}

/// Convert shard lane to core lane
pub fn shard_lane_to_core(lane: ShardLane) -> Lane {
    match lane {
        ShardLane::Research => Lane::Research,
        ShardLane::Experimental => Lane::Experimental,
        ShardLane::Production => Lane::Production,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lane_gate_research_no_actuation() {
        let shard = CyboNodeShard {
            nodeid: "test-node".to_string(),
            lane: ShardLane::Research,
            corridorpresent: true,
            vresidual: 0.5,
            vresidualmax: 1.0,
            rriskofharm: 0.10,
            kknowledge: 0.95,
            eecoimpact: 0.95,
            ..Default::default()
        };

        assert!(!lane_permits_actuation(shard.lane));
    }

    #[test]
    fn test_lane_gate_production_allows_actuation() {
        let shard = CyboNodeShard {
            nodeid: "test-node".to_string(),
            lane: ShardLane::Production,
            corridorpresent: true,
            vresidual: 0.5,
            vresidualmax: 1.0,
            rriskofharm: 0.10,
            kknowledge: 0.95,
            eecoimpact: 0.95,
            ..Default::default()
        };

        assert!(lane_permits_actuation(shard.lane));
        assert!(validate_admissibility(&shard).is_ok());
    }
}
