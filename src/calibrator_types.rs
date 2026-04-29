// src/calibrator_types.rs
pub struct IngestErrorCounts {
    pub n_missing: u32,
    pub n_rfc4180: u32,
    pub n_type_mismatch: u32,
    pub n_schema_mismatch: u32,
    pub n_corridor_missing: u32,
    pub n_unit_mismatch: u32,
    pub n_varid_unknown: u32,
}

pub struct RcalibWeights {
    pub w_missing: f64,
    pub w_rfc4180: f64,
    pub w_type_mismatch: f64,
    pub w_schema_mismatch: f64,
    pub w_corridor_missing: f64,
    pub w_unit_mismatch: f64,
    pub w_varid_unknown: f64,
}

pub struct RcalibBands {
    pub i_safe: f64,
    pub i_gold: f64,
    pub i_hard: f64,
}

pub struct SigmaComponents {
    pub r_drift: f64,
    pub r_noise: f64,
    pub r_bias: f64,
    pub r_loss: f64,
}

pub struct SigmaWeights {
    pub w_drift: f64,
    pub w_noise: f64,
    pub w_bias: f64,
    pub w_loss: f64,
}
