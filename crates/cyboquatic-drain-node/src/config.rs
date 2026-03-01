use ecosafety-core::risk_coord::RiskId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: String,
    pub location: String,
    pub safe_interior_eps: f64,
    pub risk_weights: HashMap<RiskId, f64>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        let mut risk_weights = HashMap::new();
        risk_weights.insert(RiskId::PFAS, 2.0);
        risk_weights.insert(RiskId::Microplastics, 2.0);
        risk_weights.insert(RiskId::FOG, 1.5);
        risk_weights.insert(RiskId::SewerBlockage, 1.5);
        risk_weights.insert(RiskId::Deforestation, 1.2);
        Self {
            node_id: "drain-node-001".to_string(),
            location: "Phoenix-AZ".to_string(),
            safe_interior_eps: 0.05,
            risk_weights,
        }
    }
}
