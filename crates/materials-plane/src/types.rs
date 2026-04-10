use serde::{Deserialize, Serialize};

use ecosafety_core::types::RiskCoord;

/// Kinetic parameters and environmental context for a substrate or material.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialKinetics {
    pub id: String,
    pub kinetic_model: String, // e.g., "first_order", "arrhenius"
    pub k_rate: f64,
    pub t90_days: f64,
    pub env_temp_c: f64,
    pub env_ph: f64,
    pub env_moisture: f64,

    pub tox_index: f64,
    pub micro_residue_mgkg: f64,
    pub leach_cec_meq100g: f64,
    pub pfas_residue_ugL: f64,
    pub caloric_density_mjkg: f64,
}

/// Normalized material risks and composite materials-plane coordinate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialRisks {
    pub r_t90: RiskCoord,
    pub r_tox: RiskCoord,
    pub r_micro: RiskCoord,
    pub r_leach_cec: RiskCoord,
    pub r_pfas: RiskCoord,
    pub r_caloric: RiskCoord,
    pub r_materials: RiskCoord,
    pub corridor_ok: bool,
}
