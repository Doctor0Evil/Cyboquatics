// File: crates/cyboquatic-gateway/src/lib.rs

use cyboquatic_ecosafety_core::{
    EcoSafetyKernel, SafeController, SafeStepDecision, RiskVector, Residual,
};
use econet_material_cybo::{AntSafeSubstrate, CyboNodeCompatible, material_risk_from_kinetics};

pub struct GatewayState<S> {
    pub inner_state: S,
    pub residual_v: Residual,
    pub last_rv: RiskVector,
}

pub trait LegacyHcs {
    type RawCommand;
    type RawState;

    fn read_state(&self) -> Self::RawState;
    fn propose_command(&self, state: &Self::RawState) -> Self::RawCommand;
    fn apply_command(&self, cmd: &Self::RawCommand);
}

pub struct Gateway<C, H, Sub> {
    pub controller: C,
    pub ecosafety: EcoSafetyKernel<'static>,
    pub hcs: H,
    pub substrate: Sub,
}

impl<C, H, Sub> Gateway<C, H, Sub>
where
    C: SafeController,
    H: LegacyHcs,
    Sub: AntSafeSubstrate + CyboNodeCompatible,
{
    pub fn step(&mut self, state: &mut GatewayState<C::State>) {
        // Enforce substrate safety at runtime as a hard gate.[file:19]
        if !self.substrate.is_ant_safe() {
            return; // or trigger safe shutdown
        }

        let (cmd, rv_next) = self.controller.propose_step(&state.inner_state);
        let (decision, v_next) = self
            .ecosafety
            .evaluate_step(state.residual_v, &state.last_rv, &rv_next);

        match decision {
            SafeStepDecision::Accept => {
                self.hcs.apply_command(&cmd);
            }
            SafeStepDecision::Derate => {
                // domain-specific derating (e.g., scale valve opening).
                let derated_cmd = cmd; // TODO: implement scaling
                self.hcs.apply_command(&derated_cmd);
            }
            SafeStepDecision::Stop => {
                // Do not actuate; could also send emergency stop to HCS.
            }
        }

        state.residual_v = v_next;
        state.last_rv = rv_next;
    }
}
