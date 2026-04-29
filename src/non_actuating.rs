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
