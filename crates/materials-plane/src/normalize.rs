use serde::{Deserialize, Serialize};

use ecosafety_core::types::RiskCoord;

use crate::{MaterialKinetics, MaterialRisks};

/// Corridor parameters for each materials sub-risk.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialsCorridors {
    pub t90_safe_days: f64,
    pub t90_gold_days: f64,
    pub t90_hard_days: f64,
    pub tox_safe: f64,
    pub tox_gold: f64,
    pub tox_hard: f64,
    pub micro_safe_mgkg: f64,
    pub micro_gold_mgkg: f64,
    pub micro_hard_mgkg: f64,
    pub leach_cec_safe: f64,
    pub leach_cec_gold: f64,
    pub leach_cec_hard: f64,
    pub pfas_safe_ugL: f64,
    pub pfas_gold_ugL: f64,
    pub pfas_hard_ugL: f64,
    pub caloric_safe_mjkg: f64,
    pub caloric_gold_mjkg: f64,
    pub caloric_hard_mjkg: f64,

    pub w_t90: f64,
    pub w_tox: f64,
    pub w_micro: f64,
    pub w_leach: f64,
    pub w_pfas: f64,
    pub w_caloric: f64,
}

fn normalize_piecewise(value: f64, safe: f64, gold: f64, hard: f64) -> RiskCoord {
    // Monotone non-decreasing in harmful direction: safe <= gold <= hard.
    if value <= safe {
        RiskCoord::new_clamped(0.0)
    } else if value >= hard {
        RiskCoord::new_clamped(1.0)
    } else {
        let span = hard - safe;
        let r = if span <= 0.0 {
            1.0
        } else {
            (value - safe) / span
        };
        RiskCoord::new_clamped(r)
    }
}

/// Compute MaterialRisks from MaterialKinetics and corridor definitions.
pub fn compute_material_risks(
    kin: &MaterialKinetics,
    c: &MaterialsCorridors,
) -> MaterialRisks {
    let r_t90 = normalize_piecewise(kin.t90_days, c.t90_safe_days, c.t90_gold_days, c.t90_hard_days);
    let r_tox = normalize_piecewise(kin.tox_index, c.tox_safe, c.tox_gold, c.tox_hard);
    let r_micro = normalize_piecewise(
        kin.micro_residue_mgkg,
        c.micro_safe_mgkg,
        c.micro_gold_mgkg,
        c.micro_hard_mgkg,
    );
    let r_leach = normalize_piecewise(
        kin.leach_cec_meq100g,
        c.leach_cec_safe,
        c.leach_cec_gold,
        c.leach_cec_hard,
    );
    let r_pfas = normalize_piecewise(
        kin.pfas_residue_ugL,
        c.pfas_safe_ugL,
        c.pfas_gold_ugL,
        c.pfas_hard_ugL,
    );
    let r_caloric = normalize_piecewise(
        kin.caloric_density_mjkg,
        c.caloric_safe_mjkg,
        c.caloric_gold_mjkg,
        c.caloric_hard_mjkg,
    );

    // Weighted quadratic aggregation.
    let w_sum = c.w_t90 + c.w_tox + c.w_micro + c.w_leach + c.w_pfas + c.w_caloric;
    let (w_t90, w_tox, w_micro, w_leach, w_pfas, w_caloric) = if w_sum > 0.0 {
        (
            c.w_t90 / w_sum,
            c.w_tox / w_sum,
            c.w_micro / w_sum,
            c.w_leach / w_sum,
            c.w_pfas / w_sum,
            c.w_caloric / w_sum,
        )
    } else {
        // Default equal weighting if not specified.
        (1.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0, 1.0 / 6.0)
    };

    let r_t90_val = r_t90.value();
    let r_tox_val = r_tox.value();
    let r_micro_val = r_micro.value();
    let r_leach_val = r_leach.value();
    let r_pfas_val = r_pfas.value();
    let r_caloric_val = r_caloric.value();

    let r_sq = w_t90 * r_t90_val.powi(2)
        + w_tox * r_tox_val.powi(2)
        + w_micro * r_micro_val.powi(2)
        + w_leach * r_leach_val.powi(2)
        + w_pfas * r_pfas_val.powi(2)
        + w_caloric * r_caloric_val.powi(2);

    let r_materials = RiskCoord::new_clamped(r_sq.sqrt());

    let corridor_ok = kin.t90_days <= c.t90_hard_days
        && kin.tox_index <= c.tox_hard
        && kin.micro_residue_mgkg <= c.micro_hard_mgkg
        && kin.leach_cec_meq100g <= c.leach_cec_hard
        && kin.pfas_residue_ugL <= c.pfas_hard_ugL
        && kin.caloric_density_mjkg <= c.caloric_hard_mjkg;

    MaterialRisks {
        r_t90,
        r_tox,
        r_micro,
        r_leach_cec: r_leach,
        r_pfas,
        r_caloric,
        r_materials,
        corridor_ok,
    }
}
