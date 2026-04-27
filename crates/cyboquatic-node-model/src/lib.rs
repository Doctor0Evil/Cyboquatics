// cyboquatic-node-model/src/lib.rs

#![forbid(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use cyboquatic_ecosafety_core::{
    Residual, ResidualWeights, RiskCoord, RiskVector, SafeController,
};
use core::time::Duration;

/// Telemetry snapshot from a Cyboquatic node.
#[derive(Clone, Copy, Debug)]
pub struct NodeTelemetry {
    pub q_m3s: f32,           // flow
    pub head_m: f32,          // hydraulic head
    pub p_margin_kw: f32,     // recoverable power margin
    pub c_pathogen_mpn: f32,  // microbial
    pub c_pfas_ngl: f32,      // PFAS/CEC
    pub co2e_kg_per_cycle: f32,
    pub t90_days: f32,        // time to 90% degradation
    pub rtox: f32,            // toxicity risk proxy
    pub micro_residue_idx: f32,
}

/// Normalization corridors for physical → risk mapping.
#[derive(Clone, Copy, Debug)]
pub struct Corridors {
    pub q_gold_min: f32,
    pub q_gold_max: f32,
    pub head_gold_max: f32,
    pub c_pathogen_gold: f32,
    pub c_pfas_gold: f32,
    pub co2e_neutral_kg: f32,
    pub co2e_sequestration_kg: f32,
    pub t90_gold_max_days: f32,
    pub t90_hard_max_days: f32,
    pub rtox_gold_max: f32,
    pub micro_residue_gold_max: f32,
}

/// Map telemetry to a normalized RiskVector.
pub fn telemetry_to_risk(tele: &NodeTelemetry, c: &Corridors) -> RiskVector {
    // Energy plane: higher margin → lower risk (inverted corridor).
    let r_energy = {
        let margin = tele.p_margin_kw;
        // Example: margin ≤ 0 → high risk, ≥ 4 kW → low risk.
        let r = if margin >= 4.0 {
            0.0
        } else if margin <= 0.0 {
            1.0
        } else {
            1.0 - (margin / 4.0)
        };
        RiskCoord::new_clamped(r)
    };

    // Hydraulics: flow/head near gold band, surcharge near 1.0 is unsafe.
    let r_hydraulics = {
        let q = tele.q_m3s;
        let head = tele.head_m;
        // Penalize deviations from gold flow and excessive head.
        let dq = if q < c.q_gold_min {
            (c.q_gold_min - q) / c.q_gold_min
        } else if q > c.q_gold_max {
            (q - c.q_gold_max) / c.q_gold_max
        } else {
            0.0
        };
        let dh = if head > c.head_gold_max {
            (head - c.head_gold_max) / c.head_gold_max
        } else {
            0.0
        };
        let r = (dq * dq + dh * dh).sqrt();
        RiskCoord::new_clamped(r)
    };

    // Biology: pathogen and PFAS/CEC vs gold limits.
    let r_biology = {
        let rp = tele.c_pathogen_mpn / (c.c_pathogen_gold + 1e-6);
        let rf = tele.c_pfas_ngl / (c.c_pfas_gold + 1e-6);
        let r = rp.max(rf);
        RiskCoord::new_clamped(r)
    };

    // Carbon: sequestration near 0, neutrality at gold, positive emissions → 1.
    let r_carbon = {
        let co2e = tele.co2e_kg_per_cycle;
        let r = if co2e <= c.co2e_sequestration_kg {
            0.0
        } else if co2e <= c.co2e_neutral_kg {
            // gold band between sequestration and neutrality.
            0.3
        } else {
            // Scale beyond neutrality up to hard failure at 3x neutral.
            let span = (3.0 * c.co2e_neutral_kg).max(c.co2e_neutral_kg);
            ((co2e - c.co2e_neutral_kg) / span).min(1.0)
        };
        RiskCoord::new_clamped(r)
    };

    // Materials: t90, toxicity, micro-residue.
    let r_materials = {
        let t = tele.t90_days;
        let r_t = if t <= c.t90_gold_max_days {
            0.1
        } else if t >= c.t90_hard_max_days {
            1.0
        } else {
            (t - c.t90_gold_max_days)
                / (c.t90_hard_max_days - c.t90_gold_max_days + 1e-6)
        };

        let r_tox = tele.rtox / (c.rtox_gold_max + 1e-6);
        let r_micro = tele.micro_residue_idx / (c.micro_residue_gold_max + 1e-6);

        let r = r_t.max(r_tox.max(r_micro));
        RiskCoord::new_clamped(r)
    };

    RiskVector {
        r_energy,
        r_hydraulics,
        r_biology,
        r_carbon,
        r_materials,
    }
}

/// Example actuation for a Cyboquatic tray node.
#[derive(Clone, Copy, Debug)]
pub struct TrayActuation {
    pub pump_duty_fraction: f32,
    pub turbine_setpoint_kw: f32,
    pub tray_throughput_kg_per_cycle: f32,
}

/// Simple controller that tries to keep V_t low while maintaining throughput.
pub struct TrayController {
    pub corridors: Corridors,
    pub weights: ResidualWeights,
    pub target_throughput_kg: f32,
}

impl SafeController for TrayController {
    type Actuation = TrayActuation;

    fn propose_step(&self, _dt: Duration) -> (Self::Actuation, RiskVector, Residual) {
        // In a real system, tele would come from live sensors.
        let tele = NodeTelemetry {
            q_m3s: 5.0,
            head_m: 2.0,
            p_margin_kw: 3.5,
            c_pathogen_mpn: 200.0,
            c_pfas_ngl: 4.0,
            co2e_kg_per_cycle: -0.02,
            t90_days: 90.0,
            rtox: 0.05,
            micro_residue_idx: 0.03,
        };

        let rv = telemetry_to_risk(&tele, &self.corridors);
        let residual = Residual::from_vector(&rv, &self.weights);

        let act = TrayActuation {
            pump_duty_fraction: 0.75,
            turbine_setpoint_kw: 3.0,
            tray_throughput_kg_per_cycle: self.target_throughput_kg,
        };

        (act, rv, residual)
    }
}
