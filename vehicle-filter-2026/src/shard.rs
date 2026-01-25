use serde::{Deserialize, Serialize};
use crate::contracts::CorridorBands;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShardHeader {
    pub shard_id: String,
    pub module_type: String, // "VehicleFilter2026v1"
    pub region: String,
    pub sim_or_live: String, // "sim" | "live"
    pub timestamp_utc: String,
    pub did_signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CorridorRow {
    pub varid: String,
    pub units: String,
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub weight_w: f64,
    pub lyap_channel: u16,
    pub mandatory: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RiskState {
    pub rx: std::collections::HashMap<String, f64>,
    pub vt: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KerScores {
    pub knowledge_factor: f64,
    pub eco_impact_value: f64,
    pub risk_of_harm: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VehicleFilterShard {
    pub header: ShardHeader,
    pub corridors: Vec<CorridorRow>,
    pub risk_state: RiskState,
    pub ker: KerScores,
}

impl VehicleFilterShard {
    pub fn corridors_to_bands(&self) -> Vec<CorridorBands> {
        self.corridors
            .iter()
            .map(|row| CorridorBands {
                varid: row.varid.clone(),
                units: row.units.clone(),
                safe: row.safe,
                gold: row.gold,
                hard: row.hard,
                weight_w: row.weight_w,
                lyap_channel: row.lyap_channel,
            })
            .collect()
    }
}
