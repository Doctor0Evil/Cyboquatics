// cyboquatic-hydro-plant/src/main.rs

#![forbid(unsafe_code)]
#![deny(warnings)]

use std::time::Duration;

use cyboquatic_ecosafety_core::{
    EcoSafetyKernel, KerWindow, RiskCoord, RiskVector, SafeController, SafeStepDecision,
};
use cyboquatic_eco_kernel::{
    ant_safe_substrate_ok, build_risk_vector, CarbonCycle, EcoKernelConfig, EnergyCycle,
    MaterialKinetics, MaterialToxicology,
};

/// **Shard** representation for a canal node or tray site. [file:21]
#[derive(Clone, Debug)]
pub struct NodeShard {
    pub node_id: String,
    pub lat: f64,
    pub lon: f64,
    pub q_m3s: f64,
    pub c_baseline_ng_l: f64,
    pub c_unit: String,
    pub eco_impact_score: f64,
    pub waste_reduced_kg_per_cycle: f64,
    pub tox_risk_corridor: f64,
    pub energy_kwh_per_cycle: f64,
}

/// High-level **machine state** for a hydro tray line. [file:21]
#[derive(Clone, Debug)]
pub struct TrayPlantState {
    pub shard: NodeShard,
    pub t90_days: f64,
    pub caloric_fraction: f64,
    pub r_tox: f64,
    pub r_micro: f64,
    pub r_cec: f64,
    pub net_kg_co2e: f64,
}

/// **Low-level actuation command** – here, simplified as a throughput scaling. [file:23]
#[derive(Clone, Copy, Debug)]
pub struct PlantCommand {
    pub throughput_scale: f64, // 0–1 fraction of design capacity
}

/// **SafeController** implementation for the tray plant. [file:18][file:23]
pub struct TrayPlantController {
    pub eco_cfg: EcoKernelConfig,
    pub max_caloric_fraction: f64,
}

impl SafeController<TrayPlantState, PlantCommand> for TrayPlantController {
    fn propose_step(&self, state: &TrayPlantState) -> (PlantCommand, RiskVector) {
        let kin = MaterialKinetics {
            t90_days: state.t90_days,
            residual_fraction: 0.1,
        };
        let tox = MaterialToxicology {
            r_tox: state.r_tox,
            r_micro: state.r_micro,
            r_cec: state.r_cec,
        };
        let carbon = CarbonCycle {
            net_kg_co2e: state.net_kg_co2e,
        };
        let energy = EnergyCycle {
            grid_kwh: state.shard.energy_kwh_per_cycle,
            hydro_kwh: 0.6 * state.shard.energy_kwh_per_cycle,
        };
        let r_hydraulic = RiskCoord::new_clamped(state.shard.tox_risk_corridor);
        let r_bio = RiskCoord::new_clamped(0.1);

        let rv = build_risk_vector(
            &self.eco_cfg,
            &carbon,
            &kin,
            &tox,
            &energy,
            r_hydraulic,
            r_bio,
        );

        let ant_ok = ant_safe_substrate_ok(
            &kin,
            &tox,
            self.max_caloric_fraction,
            state.caloric_fraction,
        );

        let cmd = if ant_ok {
            PlantCommand {
                throughput_scale: 1.0,
            }
        } else {
            PlantCommand {
                throughput_scale: 0.0,
            }
        };

        (cmd, rv)
    }
}

fn main() {
    let carbon_corridor = cyboquatic_ecosafety_core::CorridorBands {
        safe_min: -10.0,
        safe_max: 0.0,
        gold_min: 0.0,
        gold_max: 2.0,
        hard_min: -20.0,
        hard_max: 10.0,
    };
    let t90_corridor = cyboquatic_ecosafety_core::CorridorBands {
        safe_min: 0.0,
        safe_max: 120.0,
        gold_min: 120.0,
        gold_max: 180.0,
        hard_min: 0.0,
        hard_max: 180.0,
    };
    let energy_corridor = cyboquatic_ecosafety_core::CorridorBands {
        safe_min: 0.0,
        safe_max: 2.0,
        gold_min: 2.0,
        gold_max: 4.0,
        hard_min: 0.0,
        hard_max: 6.0,
    };

    let eco_cfg = EcoKernelConfig {
        carbon_corridor,
        t90_corridor,
        energy_corridor,
    };

    let controller = TrayPlantController {
        eco_cfg,
        max_caloric_fraction: 0.30,
    };

    let state = TrayPlantState {
        shard: NodeShard {
            node_id: "HK-PHX-TRAY-01".to_string(),
            lat: 33.45,
            lon: -112.07,
            q_m3s: 50.0,
            c_baseline_ng_l: 3.9,
            c_unit: "ng/L".to_string(),
            eco_impact_score: 0.93,
            waste_reduced_kg_per_cycle: 320.0,
            tox_risk_corridor: 0.08,
            energy_kwh_per_cycle: 14.2,
        },
        t90_days: 90.0,
        caloric_fraction: 0.27,
        r_tox: 0.08,
        r_micro: 0.04,
        r_cec: 0.08,
        net_kg_co2e: -0.35,
    };

    let mut kernel = EcoSafetyKernel::new(0.0);
    let mut ker_window = KerWindow::new(300);

    for _ in 0..300 {
        let (cmd, rv) = controller.propose_step(&state);

        let (decision, v_next) =
            kernel.evaluate_step(&rv, 1.0, 1.0, 1.0, 1.0, 1.0);

        ker_window.update(decision, &rv);

        if matches!(decision, SafeStepDecision::Stop) {
            eprintln!(
                "Safety STOP at Vt={} for node {}",
                v_next.v, state.shard.node_id
            );
            break;
        }

        if matches!(decision, SafeStepDecision::Derate) && cmd.throughput_scale > 0.0 {
            eprintln!(
                "Derating throughput for node {} Vt={}",
                state.shard.node_id, v_next.v
            );
        }

        std::thread::sleep(Duration::from_millis(5));
    }

    let triad = ker_window.triad();
    println!(
        "KER for hydro tray plant: K={:.3}, E={:.3}, R={:.3}",
        triad.k_knowledge, triad.e_eco_impact, triad.r_risk_of_harm
    );
}
