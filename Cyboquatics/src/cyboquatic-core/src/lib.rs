pub mod hydrokinetic;
pub mod intake_safety;
pub mod pfbs_remediation;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Fundamental error type for cyboquatic-core operations.
#[derive(Debug, Error)]
pub enum CyboquaticError {
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Basic description of a cyboquatic power node, aligned with cyboquatic.power.node.v1.[file:5]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerNode {
    pub node_id: String,
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub depth_m: f64,
    pub mean_flow_ms: f64,
    pub flow_variance_ms2: f64,
    pub rated_power_kw: f64,
    pub pfbs_removal_kg_per_h: f64,
    pub max_intake_flow_ms: f64,
}

impl PowerNode {
    pub fn new(
        node_id: &str,
        latitude_deg: f64,
        longitude_deg: f64,
        depth_m: f64,
        mean_flow_ms: f64,
        flow_variance_ms2: f64,
        rated_power_kw: f64,
        pfbs_removal_kg_per_h: f64,
        max_intake_flow_ms: f64,
    ) -> Result<Self, CyboquaticError> {
        if mean_flow_ms <= 0.0 {
            return Err(CyboquaticError::InvalidParameter(
                "mean_flow_ms must be > 0".into(),
            ));
        }
        if depth_m <= 0.0 {
            return Err(CyboquaticError::InvalidParameter(
                "depth_m must be > 0".into(),
            ));
        }
        Ok(Self {
            node_id: node_id.to_string(),
            latitude_deg,
            longitude_deg,
            depth_m,
            mean_flow_ms,
            flow_variance_ms2,
            rated_power_kw,
            pfbs_removal_kg_per_h,
            max_intake_flow_ms,
        })
    }
}
