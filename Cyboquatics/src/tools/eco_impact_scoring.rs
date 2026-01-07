use serde::{Deserialize, Serialize};

/// Minimal node record mirroring the CSV and telemetry fields.[web:13]
#[derive(Debug, Clone, Deserialize)]
pub struct NodeRecord {
    pub node_id: String,
    pub mean_flow_ms: f64,
    pub rated_power_kw: f64,
    pub pfbs_removal_kg_per_h: f64,
    pub max_intake_flow_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EcoImpactScore {
    pub node_id: String,
    pub score: f64,
}

fn eco_score(rec: &NodeRecord) -> f64 {
    if rec.rated_power_kw <= 0.0 || rec.pfbs_removal_kg_per_h < 0.0 {
        return 0.0;
    }
    let power_score = (rec.rated_power_kw / 80.0).tanh();
    let pfbs_score = (rec.pfbs_removal_kg_per_h / 2.0).tanh();
    let intake_penalty = if rec.mean_flow_ms > rec.max_intake_flow_ms {
        0.3
    } else {
        0.0
    };
    let score = 0.6 * power_score + 0.4 * pfbs_score - intake_penalty;
    score.clamp(0.0, 1.0)
}
