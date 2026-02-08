use ecosafety_core::types::{Residual, RiskCoord, CorridorBands};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceMeta {
    pub shard_id:        String,
    pub module_type:     String, // "PhoenixMARCell.v1"
    pub region:          String, // "Phoenix-AZ"
    pub sim_or_live:     String, // "sim" | "live"
    pub timestamp_utc:   String,
    pub did_signature:   String, // Bostrom DID hex
    pub rust_build_hash: String,
    pub aln_schema_ver:  String,
    pub hex_stamp:       String, // governance hex-stamp for this shard
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KER {
    pub knowledge_factor_01: f64, // K ∈ [0,1]
    pub eco_impact_01:       f64, // E ∈ [0,1]
    pub risk_of_harm_01:     f64, // R ∈ [0,1]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhoenixMarShard {
    pub meta:        GovernanceMeta,
    pub corridors:   Vec<CorridorBands>, // r_SAT, r_PFAS, r_nutrient, r_temp, r_foul, r_surcharge
    pub risk_state:  Vec<RiskCoord>,     // current r_x values
    pub residual:    Residual,           // V_t
    pub ker:         KER,                // triad for this run or design
    pub q_design_m3s: f64,               // design flow
    pub recharge_m3_per_year: f64,       // modeled/observed
    pub m_pollutant_removed_kg_per_y: f64,
}
