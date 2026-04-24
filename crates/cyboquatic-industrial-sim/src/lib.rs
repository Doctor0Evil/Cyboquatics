#![forbid(unsafe_code)]

//! C-facing simulation kernel mirroring Cyboquatic industrial ecosafety logic.
//!
//! This crate exports CEIM/CPVM functions and Vt computation through FFI,
//! while pulling corridor tables and kernels from the Rust side.

use cyboquatic_industrial_ecosafety_core::IndustrialRiskVector;
use ecosafety_core::{LyapunovWeights, compute_residual, RiskCoord};

/// CEIM mass balance: Mx = (Cin - Cout) * Q * dt, scaled to kg.
///
/// # Parameters
/// - `cin_mg_l`: Input concentration in mg/L
/// - `cout_mg_l`: Output concentration in mg/L
/// - `flow_m3_s`: Flow rate in m³/s
/// - `dt_s`: Time delta in seconds
///
/// # Returns
/// Mass change in kg
#[no_mangle]
pub extern "C" fn ceim_mass_balance(
    cin_mg_l: f64,
    cout_mg_l: f64,
    flow_m3_s: f64,
    dt_s: f64,
) -> f64 {
    let delta_mg_l = cin_mg_l - cout_mg_l;
    let delta_kg = delta_mg_l * 1.0e-6 * flow_m3_s * dt_s;
    delta_kg
}

/// Compute Lyapunov residual V(t) from risk coordinates.
///
/// # Parameters
/// - `r_energy`: Energy plane risk coordinate (0..=1)
/// - `r_hydraulics`: Hydraulics plane risk coordinate (0..=1)
/// - `r_biology`: Biology plane risk coordinate (0..=1)
/// - `r_carbon`: Carbon plane risk coordinate (0..=1)
/// - `r_materials`: Materials plane risk coordinate (0..=1)
///
/// # Returns
/// Lyapunov residual value V(t)
#[no_mangle]
pub extern "C" fn cybo_vt_from_risks(
    r_energy: f64,
    r_hydraulics: f64,
    r_biology: f64,
    r_carbon: f64,
    r_materials: f64,
) -> f64 {
    let rv = IndustrialRiskVector {
        energy: RiskCoord::new_clamped(r_energy),
        hydraulics: RiskCoord::new_clamped(r_hydraulics),
        biology: RiskCoord::new_clamped(r_biology),
        carbon: RiskCoord::new_clamped(r_carbon),
        materials: RiskCoord::new_clamped(r_materials),
    };
    let weights = LyapunovWeights {
        w_energy: 1.0,
        w_hydraulics: 1.0,
        w_biology: 1.0,
        w_carbon: 1.0,
        w_materials: 1.0,
        w_biodiversity: 0.0,
        w_sigma: 0.0,
    };
    let residual = compute_residual(&ecosafety_core::RiskVector {
        r_energy: rv.energy,
        r_hydraulics: rv.hydraulics,
        r_biology: rv.biology,
        r_carbon: rv.carbon,
        r_materials: rv.materials,
        r_biodiversity: RiskCoord::new_clamped(0.0),
        r_sigma: RiskCoord::new_clamped(0.0),
    }, &weights);
    residual.value
}

/// Compute full risk vector and residual for a proposed step.
///
/// # Parameters
/// - `r_energy`: Energy plane risk coordinate
/// - `r_hydraulics`: Hydraulics plane risk coordinate
/// - `r_biology`: Biology plane risk coordinate
/// - `r_carbon`: Carbon plane risk coordinate
/// - `r_materials`: Materials plane risk coordinate
/// - `v_out`: Pointer to output buffer for [v_t, v_t_max, is_stable]
///
/// # Safety
/// `v_out` must point to at least 3 f64 values.
#[no_mangle]
pub unsafe extern "C" fn cybo_compute_residual_full(
    r_energy: f64,
    r_hydraulics: f64,
    r_biology: f64,
    r_carbon: f64,
    r_materials: f64,
    v_out: *mut f64,
) {
    if v_out.is_null() {
        return;
    }

    let weights = LyapunovWeights {
        w_energy: 1.0,
        w_hydraulics: 1.0,
        w_biology: 1.0,
        w_carbon: 1.0,
        w_materials: 1.0,
        w_biodiversity: 0.0,
        w_sigma: 0.0,
    };

    let rv = ecosafety_core::RiskVector {
        r_energy: RiskCoord::new_clamped(r_energy),
        r_hydraulics: RiskCoord::new_clamped(r_hydraulics),
        r_biology: RiskCoord::new_clamped(r_biology),
        r_carbon: RiskCoord::new_clamped(r_carbon),
        r_materials: RiskCoord::new_clamped(r_materials),
        r_biodiversity: RiskCoord::new_clamped(0.0),
        r_sigma: RiskCoord::new_clamped(0.0),
    };

    let residual = compute_residual(&rv, &weights);

    *v_out.offset(0) = residual.value;
    *v_out.offset(1) = 1.0; // v_t_max placeholder
    *v_out.offset(2) = if residual.value <= 1.0 { 1.0 } else { 0.0 };
}
