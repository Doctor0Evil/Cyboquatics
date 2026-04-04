// File: crates/cyboquatic-gateway/src/lib.rs

//! cyboquatic-gateway
//! Hybrid gateway that wraps a legacy HCS/PLC and enforces ecosafety.
//!
//! This crate is non-actuating with respect to invariants: it mediates
//! legacy commands through the ecosafety kernel and material gates.

#![no_std]

use cyboquatic_ecosafety_core::{
    CorridorBands, KerTriad, RiskCoord, RiskPlane, RiskVector, Residual, SafeStepDecision,
    Weight, lyapunov_residual, safestep_decision, RISK_DIM,
};
use econetmaterialcybo::{AntSafeSubstrate, CyboNodeCompatible, MaterialKinetics, material_risks_to_vector};

/// Minimal abstract command from a legacy controller.
#[derive(Copy, Clone)]
pub struct LegacyCommand {
    pub valve_open_pct: f32,
    pub pump_rpm: f32,
}

/// Derated command returned when ecosafety requires softening.
#[derive(Copy, Clone)]
pub struct DeratedCommand {
    pub valve_open_pct: f32,
    pub pump_rpm: f32,
}

/// State snapshot of the gateway’s ecosafety view.
pub struct GatewayState<S> {
    pub inner_state: S,
    pub residual_v: Residual,
    pub last_rv: RiskVector,
    pub ker_window: KerTriad,
    pub safe_steps: u64,
    pub total_steps: u64,
    pub max_risk_coord: RiskCoord,
}

/// Legacy hardware control system (PLC / HCS).
pub trait LegacyHcs {
    type RawCommand;
    type RawState;

    fn read_state(&self) -> Self::RawState;
    fn apply_command(&self, cmd: &Self::RawCommand);
}

/// Controller that proposes commands and risk vectors (non‑actuating).
pub trait SafeController {
    type State;
    type Command;

    fn propose_step(&self, state: &Self::State) -> (Self::Command, RiskVector);
}

/// Fold current plant state and command into a risk vector.
pub trait StateFold {
    fn fold_to_risk(&self, cmd: &LegacyCommand) -> RiskVector;
}

/// Domain‑specific derating logic.
pub trait Derater {
    fn derate(&self, cmd: &LegacyCommand, r: &RiskVector) -> DeratedCommand;
}

/// Ecosafety kernel used by the gateway to gate legacy commands.
pub struct EcoSafetyKernel<'a, F: StateFold, D: Derater> {
    pub weights: [Weight; RISK_DIM],
    pub corridors: [CorridorBands; RISK_DIM],
    pub eps_vt: Residual,
    pub state_fold: &'a F,
    pub derater: &'a D,
}

impl<'a, F: StateFold, D: Derater> EcoSafetyKernel<'a, F, D> {
    pub fn evaluate(
        &self,
        prev_v: Residual,
        cmd: &LegacyCommand,
    ) -> (SafeStepDecision, Residual, Option<DeratedCommand>, RiskVector) {
        let mut r_next = self.state_fold.fold_to_risk(cmd);
        // r_next may already carry material risk; if not, callers can patch it.
        let next_v = lyapunov_residual(&r_next, &self.weights);
        let decision = safestep_decision(prev_v, next_v, self.eps_vt, &r_next, &self.corridors);

        match decision {
            SafeStepDecision::Accept => (decision, next_v, None, r_next),
            SafeStepDecision::Derate => {
                let d = self.derater.derate(cmd, &r_next);
                (decision, next_v, Some(d), r_next)
            }
            SafeStepDecision::Stop => (decision, prev_v, None, r_next),
        }
    }
}

/// Gateway tying controller, ecosafety kernel, legacy HCS, and substrate together.
pub struct Gateway<C, H, Sub, F, D>
where
    C: SafeController<Command = LegacyCommand>,
    H: LegacyHcs<RawCommand = LegacyCommand>,
    Sub: AntSafeSubstrate + CyboNodeCompatible,
    F: StateFold,
    D: Derater,
{
    pub controller: C,
    pub ecosafety: EcoSafetyKernel<'static, F, D>,
    pub hcs: H,
    pub substrate: Sub,
    pub material_kinetics: MaterialKinetics,
}

impl<C, H, Sub, F, D> Gateway<C, H, Sub, F, D>
where
    C: SafeController<Command = LegacyCommand>,
    H: LegacyHcs<RawCommand = LegacyCommand>,
    Sub: AntSafeSubstrate + CyboNodeCompatible,
    F: StateFold,
    D: Derater,
{
    /// One ecosafety‑gated step.
    pub fn step(&mut self, state: &mut GatewayState<C::State>) {
        // Hard gate: substrate must remain within AntSafe corridors.
        if !self.substrate.hard_gate_ok() || !self.substrate.compatible_with_node() {
            return;
        }

        // Controller proposes a command and an initial RiskVector shell.
        let (cmd, mut rv_next) = self.controller.propose_step(&state.inner_state);

        // Inject material risk into materials plane of RiskVector.
        rv_next = material_risks_to_vector(&self.material_kinetics, rv_next);

        // Evaluate ecosafety decision.
        let (decision, v_next, maybe_derate, r_eval) =
            self.ecosafety.evaluate(state.residual_v, &cmd);

        // Apply or block command at the HCS boundary.
        match decision {
            SafeStepDecision::Accept => {
                self.hcs.apply_command(&cmd);
            }
            SafeStepDecision::Derate => {
                let dcmd = maybe_derate.unwrap_or(DeratedCommand {
                    valve_open_pct: cmd.valve_open_pct,
                    pump_rpm: cmd.pump_rpm,
                });
                self.hcs.apply_command(&LegacyCommand {
                    valve_open_pct: dcmd.valve_open_pct,
                    pump_rpm: dcmd.pump_rpm,
                });
            }
            SafeStepDecision::Stop => {
                // Intentionally do nothing; optional emergency stop channel can be added.
            }
        }

        // Update Lyapunov residual and KER window.
        state.residual_v = v_next;
        state.last_rv = r_eval;
        state.total_steps += 1;
        if matches!(decision, SafeStepDecision::Accept | SafeStepDecision::Derate) {
            state.safe_steps += 1;
        }
        let max_r = r_eval.max_coord();
        if max_r > state.max_risk_coord {
            state.max_risk_coord = max_r;
        }
        state.ker_window = cyboquatic_ecosafety_core::ker_from_window(
            state.safe_steps,
            state.total_steps,
            state.max_risk_coord,
        );
    }
}
