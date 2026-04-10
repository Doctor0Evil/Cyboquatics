// File: crates/cyboquatic_gateway/src/lib.rs

#![no_std]

use cyboquatic_ecosafety_core::{
    EcoSafetyKernel, KerWindowStats, LyapunovWeights, RiskVector, SafeController, SafeStepDecision,
};
use econet_material_cybo::{CyboMaterial, NodeClass};

/// Abstract machine state (to be provided by integration layer).
pub struct MachineState {
    pub r_energy: f32,
    pub r_hydraulic: f32,
    pub r_biology: f32,
    pub r_carbon: f32,
    pub r_materials: f32,
    pub r_biodiversity: f32,
}

/// Abstract low-level command (to be translated by HAL).
#[derive(Clone, Copy)]
pub struct RawCommand {
    pub valve_open_pct: f32,
    pub pump_duty_pct: f32,
}

/// Example controller: legacy logic wrapped to emit RiskVector.
pub struct WrappedController<M: CyboMaterial> {
    pub material: M,
    pub node_class: NodeClass,
}

impl<M: CyboMaterial> SafeController<MachineState, RawCommand> for WrappedController<M> {
    fn propose_step(&self, state: &MachineState) -> (RawCommand, RiskVector) {
        let base_valve = 15.0_f32;
        let base_pump = 75.0_f32;

        let cmd = RawCommand {
            valve_open_pct: base_valve,
            pump_duty_pct: base_pump,
        };

        let rv = RiskVector {
            r_energy: state.r_energy,
            r_hydraulic: state.r_hydraulic,
            r_biology: state.r_biology,
            r_carbon: state.r_carbon,
            r_materials: state.r_materials,
            r_biodiversity: state.r_biodiversity,
        };

        (cmd, rv)
    }
}

/// Gateway that wraps controller + ecosafety kernel and maintains KER scores.
pub struct CyboquaticGateway<C, M>
where
    C: SafeController<MachineState, RawCommand>,
    M: CyboMaterial,
{
    pub kernel: EcoSafetyKernel,
    pub controller: C,
    pub material: M,
    pub node_class: NodeClass,
    vt_prev: f32,
    ker_stats: KerWindowStats,
}

impl<C, M> CyboquaticGateway<C, M>
where
    C: SafeController<MachineState, RawCommand>,
    M: CyboMaterial,
{
    pub fn new(kernel: EcoSafetyKernel, controller: C, material: M, node_class: NodeClass) -> Self {
        CyboquaticGateway {
            kernel,
            controller,
            material,
            node_class,
            vt_prev: 0.0,
            ker_stats: KerWindowStats::new(),
        }
    }

    /// Evaluate one control tick: returns decision and possibly derated command.
    pub fn step(&mut self, state: &MachineState) -> (SafeStepDecision, RawCommand) {
        if !self.material.corridors_ok() || !self.material.is_compatible_with_node(self.node_class) {
            return (SafeStepDecision::Stop, RawCommand { valve_open_pct: 0.0, pump_duty_pct: 0.0 });
        }

        let (cmd, next_risk) = self.controller.propose_step(state);

        let prev_risk = RiskVector {
            r_energy: state.r_energy,
            r_hydraulic: state.r_hydraulic,
            r_biology: state.r_biology,
            r_carbon: state.r_carbon,
            r_materials: state.r_materials,
            r_biodiversity: state.r_biodiversity,
        };

        let decision = self.kernel.evaluate_step(self.vt_prev, &prev_risk, &next_risk);

        let vt_next = cyboquatic_ecosafety_core::lyapunov_residual(&next_risk, &self.kernel.weights);
        self.ker_stats.update(self.vt_prev, vt_next, &next_risk);
        self.vt_prev = vt_next;

        let final_cmd = match decision {
            SafeStepDecision::Accept => cmd,
            SafeStepDecision::Derate => RawCommand {
                valve_open_pct: cmd.valve_open_pct * 0.67,
                pump_duty_pct: cmd.pump_duty_pct * 0.67,
            },
            SafeStepDecision::Stop => RawCommand {
                valve_open_pct: 0.0,
                pump_duty_pct: 0.0,
            },
        };

        (decision, final_cmd)
    }

    pub fn ker(&self) -> cyboquatic_ecosafety_core::KerTriad {
        self.ker_stats.ker()
    }
}

/// Example factory for weights emphasizing ecological planes.
pub fn default_lyapunov_weights() -> LyapunovWeights {
    LyapunovWeights {
        w_energy: 1.0,
        w_hydraulic: 1.2,
        w_biology: 1.5,
        w_carbon: 1.5,       // push hard toward carbon-negative
        w_materials: 1.3,    // penalize persistent/toxic materials
        w_biodiversity: 1.4, // protect ecosystems
    }
}
