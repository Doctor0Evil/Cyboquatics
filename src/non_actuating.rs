// src/non_actuating.rs
use ecosafety_core::{RiskVector, Residual, KerWindow};

pub trait NonActuatingCalibrator {
    /// Pure function: read input shards, compute data-quality risk, update residual/KER.
    fn run_calibration(
        &self,
        ingest_window: &[IngestErrorCounts],
        sensor_stats: &[SigmaComponents],
        risk_vector: &mut RiskVector,
        residual: &mut Residual,
        ker_window: &mut KerWindow,
    ) -> CalibrationOutcome;
}

pub struct CalibrationOutcome {
    pub r_calib: f64,
    pub r_sigma: f64,
    pub vt_before: f64,
    pub vt_after: f64,
    pub k_window: f64,
    pub e_window: f64,
    pub r_window: f64,
}

pub fn apply_calibration_to_residual(
    risk_vector: &mut RiskVector,
    residual: &mut Residual,
    r_calib: f64,
    r_sigma: f64,
    w_calib: f64,
    w_sigma: f64,
) {
    risk_vector.set("r_calib", r_calib);
    risk_vector.set("r_sigma", r_sigma);

    residual.add_term("r_calib", r_calib, w_calib);
    residual.add_term("r_sigma", r_sigma, w_sigma);
}

pub fn update_ker_with_trust(
    ker_window: &mut KerWindow,
    r_calib: f64,
) {
    let d_data = 1.0 - r_calib;           // trust in [0,1]
    ker_window.k_adj = ker_window.k_raw * d_data;
    ker_window.e_adj = ker_window.e_raw * d_data;
    // R remains max over all coordinates; do not scale by trust.
}
