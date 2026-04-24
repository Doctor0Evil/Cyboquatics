//! Admissibility validation for industrial node shards

use crate::industrial_node::{CyboNodeShard, Lane};
use thiserror::Error;

/// Errors that can occur during admissibility validation
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AdmissibilityError {
    #[error("Corridor not present for node {0}")]
    CorridorMissing(String),

    #[error("Residual exceeds maximum: v_t={v_t}, v_max={v_max} for node {nodeid}")]
    ResidualExceeded {
        nodeid: String,
        v_t: f64,
        v_max: f64,
    },

    #[error("Risk of harm exceeds threshold: r={r}, max=0.13 for node {nodeid}")]
    RiskOfHarmExceeded {
        nodeid: String,
        r: f64,
    },

    #[error("Production lane requires K >= 0.90: K={k} for node {nodeid}")]
    ProductionKnowledgeInsufficient {
        nodeid: String,
        k: f64,
    },

    #[error("Production lane requires E >= 0.90: E={e} for node {nodeid}")]
    ProductionEcoImpactInsufficient {
        nodeid: String,
        e: f64,
    },

    #[error("Lane {lane:?} does not permit actuation for node {nodeid}")]
    LaneNotActuating {
        nodeid: String,
        lane: Lane,
    },
}

/// Validate admissibility according to ALN schema rules:
/// - corridorpresent = true
/// - vresidual <= vresidualmax
/// - rriskofharm <= 0.13
/// - If lane = PRODUCTION: kknowledge >= 0.90, eecoimpact >= 0.90
pub fn validate_admissibility(shard: &CyboNodeShard) -> Result<(), AdmissibilityError> {
    let nodeid = &shard.nodeid;

    // Assert corridorpresent = true
    if !shard.corridorpresent {
        return Err(AdmissibilityError::CorridorMissing(nodeid.clone()));
    }

    // Assert vresidual <= vresidualmax
    if shard.vresidual > shard.vresidualmax {
        return Err(AdmissibilityError::ResidualExceeded {
            nodeid: nodeid.clone(),
            v_t: shard.vresidual,
            v_max: shard.vresidualmax,
        });
    }

    // Assert rriskofharm <= 0.13
    if shard.rriskofharm > 0.13 {
        return Err(AdmissibilityError::RiskOfHarmExceeded {
            nodeid: nodeid.clone(),
            r: shard.rriskofharm,
        });
    }

    // Lane-specific checks
    match shard.lane {
        Lane::Production => {
            if shard.kknowledge < 0.90 {
                return Err(AdmissibilityError::ProductionKnowledgeInsufficient {
                    nodeid: nodeid.clone(),
                    k: shard.kknowledge,
                });
            }
            if shard.eecoimpact < 0.90 {
                return Err(AdmissibilityError::ProductionEcoImpactInsufficient {
                    nodeid: nodeid.clone(),
                    e: shard.eecoimpact,
                });
            }
        }
        Lane::Experimental | Lane::Research => {
            // Research and Experimental lanes are read-only (no actuation)
            // This is not an error, just informational
        }
    }

    Ok(())
}

/// Check if a shard's lane permits actuation
/// Only Production lane permits actuation; Research and Experimental are diagnostics-only
pub fn lane_permits_actuation(lane: Lane) -> bool {
    matches!(lane, Lane::Production)
}

/// Get KER thresholds for a given lane
pub fn lane_ker_thresholds(lane: Lane) -> LaneKerThresholds {
    match lane {
        Lane::Research => LaneKerThresholds {
            min_k: 0.0,
            min_e: 0.0,
            max_r: 1.0,
            permits_actuation: false,
        },
        Lane::Experimental => LaneKerThresholds {
            min_k: 0.70,
            min_e: 0.70,
            max_r: 0.30,
            permits_actuation: false,
        },
        Lane::Production => LaneKerThresholds {
            min_k: 0.90,
            min_e: 0.90,
            max_r: 0.13,
            permits_actuation: true,
        },
    }
}

/// KER thresholds for a lane
#[derive(Clone, Copy, Debug)]
pub struct LaneKerThresholds {
    pub min_k: f64,
    pub min_e: f64,
    pub max_r: f64,
    pub permits_actuation: bool,
}

impl LaneKerThresholds {
    /// Check if a shard meets the thresholds for this lane
    pub fn meets_thresholds(&self, shard: &CyboNodeShard) -> bool {
        shard.kknowledge >= self.min_k
            && shard.eecoimpact >= self.min_e
            && shard.rriskofharm <= self.max_r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_production_shard() {
        let shard = CyboNodeShard {
            nodeid: "test-node-1".to_string(),
            corridorpresent: true,
            vresidual: 0.5,
            vresidualmax: 1.0,
            rriskofharm: 0.10,
            kknowledge: 0.95,
            eecoimpact: 0.92,
            lane: Lane::Production,
            ..Default::default()
        };

        assert!(validate_admissibility(&shard).is_ok());
        assert!(lane_permits_actuation(shard.lane));
    }

    #[test]
    fn test_missing_corridor() {
        let shard = CyboNodeShard {
            nodeid: "test-node-2".to_string(),
            corridorpresent: false,
            ..Default::default()
        };

        let result = validate_admissibility(&shard);
        assert!(matches!(result, Err(AdmissibilityError::CorridorMissing(_))));
    }

    #[test]
    fn test_residual_exceeded() {
        let shard = CyboNodeShard {
            nodeid: "test-node-3".to_string(),
            corridorpresent: true,
            vresidual: 1.5,
            vresidualmax: 1.0,
            rriskofharm: 0.10,
            ..Default::default()
        };

        let result = validate_admissibility(&shard);
        assert!(matches!(result, Err(AdmissibilityError::ResidualExceeded { .. })));
    }

    #[test]
    fn test_risk_of_harm_exceeded() {
        let shard = CyboNodeShard {
            nodeid: "test-node-4".to_string(),
            corridorpresent: true,
            vresidual: 0.5,
            vresidualmax: 1.0,
            rriskofharm: 0.20,
            ..Default::default()
        };

        let result = validate_admissibility(&shard);
        assert!(matches!(result, Err(AdmissibilityError::RiskOfHarmExceeded { .. })));
    }

    #[test]
    fn test_production_knowledge_insufficient() {
        let shard = CyboNodeShard {
            nodeid: "test-node-5".to_string(),
            corridorpresent: true,
            vresidual: 0.5,
            vresidualmax: 1.0,
            rriskofharm: 0.10,
            kknowledge: 0.80,
            eecoimpact: 0.95,
            lane: Lane::Production,
            ..Default::default()
        };

        let result = validate_admissibility(&shard);
        assert!(matches!(result, Err(AdmissibilityError::ProductionKnowledgeInsufficient { .. })));
    }

    #[test]
    fn test_research_lane_no_actuation() {
        assert!(!lane_permits_actuation(Lane::Research));
        assert!(!lane_permits_actuation(Lane::Experimental));
        assert!(lane_permits_actuation(Lane::Production));
    }
}
