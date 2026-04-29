// src/calibrator_kernels.rs
use crate::calibrator_types::*;

pub fn ingest_error(counts: &IngestErrorCounts, w: &RcalibWeights) -> f64 {
    w.w_missing  * (counts.n_missing  as f64) +
    w.w_rfc4180  * (counts.n_rfc4180 as f64) +
    w.w_type_mismatch * (counts.n_type_mismatch as f64) +
    w.w_schema_mismatch * (counts.n_schema_mismatch as f64) +
    w.w_corridor_missing * (counts.n_corridor_missing as f64) +
    w.w_unit_mismatch * (counts.n_unit_mismatch as f64) +
    w.w_varid_unknown * (counts.n_varid_unknown as f64)
}

pub fn normalize_rcalib(i: f64, bands: &RcalibBands) -> f64 {
    if i <= bands.i_safe {
        0.0
    } else if i <= bands.i_gold {
        let t = (i - bands.i_safe) / (bands.i_gold - bands.i_safe);
        0.33 * t
    } else if i <= bands.i_hard {
        let t = (i - bands.i_gold) / (bands.i_hard - bands.i_gold);
        0.33 + 0.67 * t
    } else {
        1.0
    }
}

pub fn combine_rsigma(c: &SigmaComponents, w: &SigmaWeights) -> f64 {
    let v =
        w.w_drift * c.r_drift * c.r_drift +
        w.w_noise * c.r_noise * c.r_noise +
        w.w_bias  * c.r_bias  * c.r_bias  +
        w.w_loss  * c.r_loss  * c.r_loss;
    let r = v.sqrt();
    if r > 1.0 { 1.0 } else { r }
}
