#![no_std]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

extern crate alloc;

use cyboquatic_ecosafety_core::{RiskVector as PhysicalRiskVector, Residual as PhysicalResidual, LyapunovWeights as PhysicalLyapunovWeights, SafeStepDecision, safestep, SafeStepConfig};
use cyboquatic_brain_identity_core::{BrainIdentityShard, BiRiskVector, BiResidual, BiLyapunovWeights, BiSafeStepDecision, bi_safestep, BiSafeStepConfig, BiKerWindow};

pub struct IntegratedController<State, Actuation> {
    pub physical_weights: PhysicalLyapunovWeights,
    pub bi_weights: BiLyapunovWeights,
    pub safe_config: SafeStepConfig,
    pub bi_safe_config: BiSafeStepConfig,
    _phantom: core::marker::PhantomData<(State, Actuation)>,
}

impl<State, Actuation> IntegratedController<State, Actuation> {
    pub fn new(
        physical_weights: PhysicalLyapunovWeights,
        bi_weights: BiLyapunovWeights,
        epsilon: Scalar,
        enforce_karma: bool,
    ) -> Self {
        IntegratedController {
            physical_weights,
            bi_weights,
            safe_config: SafeStepConfig { epsilon },
            bi_safe_config: BiSafeStepConfig { epsilon, enforce_karma_nonslash: enforce_karma },
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn evaluate_step(
        &self,
        physical_rv: &PhysicalRiskVector,
        physical_residual_prev: PhysicalResidual,
        bi_shard: &BrainIdentityShard,
        proposed_karma: Scalar,
    ) -> IntegratedStepResult {
        let bi_rv = BiRiskVector::from_physical_and_bi(physical_rv, bi_shard);
        let bi_residual_prev = BiResidual::compute(&bi_rv, &self.bi_weights);
        let bi_residual_next = bi_residual_prev;

        let physical_decision = safestep(
            physical_residual_prev,
            physical_residual_prev,
            physical_rv,
            &self.safe_config,
        );

        let bi_decision = bi_safestep(
            bi_residual_prev,
            bi_residual_next,
            &bi_rv,
            bi_shard.karma_floor,
            proposed_karma,
            &self.bi_safe_config,
        );

        let combined_decision = Self::combine_decisions(physical_decision, bi_decision);
        let karma_preserved = matches!(bi_decision, BiSafeStepDecision::Accept | BiSafeStepDecision::Derate);

        IntegratedStepResult {
            physical_decision,
            bi_decision,
            combined_decision,
            karma_preserved,
            bi_rv,
            bi_residual_next,
        }
    }

    fn combine_decisions(physical: SafeStepDecision, bi: BiSafeStepDecision) -> IntegratedDecision {
        match (physical, bi) {
            (_, BiSafeStepDecision::StopKarmaViolation) => IntegratedDecision::StopKarmaViolation,
            (SafeStepDecision::Stop, _) => IntegratedDecision::Stop,
            (_, BiSafeStepDecision::Stop) => IntegratedDecision::Stop,
            (SafeStepDecision::Derate, _) => IntegratedDecision::Derate,
            (_, BiSafeStepDecision::Derate) => IntegratedDecision::Derate,
            (SafeStepDecision::Accept, BiSafeStepDecision::Accept) => IntegratedDecision::Accept,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IntegratedDecision {
    Accept,
    Derate,
    Stop,
    StopKarmaViolation,
}

pub struct IntegratedStepResult {
    pub physical_decision: SafeStepDecision,
    pub bi_decision: BiSafeStepDecision,
    pub combined_decision: IntegratedDecision,
    pub karma_preserved: bool,
    pub bi_rv: BiRiskVector,
    pub bi_residual_next: BiResidual,
}

pub struct IntegratedKerTracker {
    pub physical_ker: cyboquatic_ecosafety_core::KerWindow,
    pub bi_ker: BiKerWindow,
}

impl Default for IntegratedKerTracker {
    fn default() -> Self {
        IntegratedKerTracker {
            physical_ker: cyboquatic_ecosafety_core::KerWindow::default(),
            bi_ker: BiKerWindow::default(),
        }
    }
}

impl IntegratedKerTracker {
    pub fn update(&mut self, result: &IntegratedStepResult) {
        let physical_rv = PhysicalRiskVector {
            r_energy: result.bi_rv.r_energy,
            r_hydraulic: result.bi_rv.r_hydraulic,
            r_biology: result.bi_rv.r_biology,
            r_carbon: result.bi_rv.r_carbon,
            r_materials: result.bi_rv.r_materials,
        };

        self.physical_ker.update(&physical_rv, matches!(result.physical_decision, SafeStepDecision::Accept));
        self.bi_ker.update(&result.bi_rv, result.bi_decision, result.karma_preserved);
    }

    pub fn both_deployable(&self) -> bool {
        self.physical_ker.ker_deployable() && self.bi_ker.bi_ker_deployable()
    }

    pub fn export_ker_summary(&self) -> KerSummary {
        KerSummary {
            physical_k: self.physical_ker.k(),
            physical_e: self.physical_ker.e(),
            physical_r: self.physical_ker.r(),
            bi_k: self.bi_ker.k(),
            bi_e: self.bi_ker.e(),
            bi_r: self.bi_ker.r(),
            karma_preserved: self.bi_ker.karma_preserved,
            combined_deployable: self.both_deployable(),
        }
    }
}

pub struct KerSummary {
    pub physical_k: Scalar,
    pub physical_e: Scalar,
    pub physical_r: Scalar,
    pub bi_k: Scalar,
    pub bi_e: Scalar,
    pub bi_r: Scalar,
    pub karma_preserved: bool,
    pub combined_deployable: bool,
}

pub type Scalar = f32;

pub struct BiAuditLogEntry {
    pub timestamp_unix: u64,
    pub brainidentityid: [u8; 32],
    pub hexstamp: [u8; 32],
    pub vt_previous: Scalar,
    pub vt_current: Scalar,
    pub vt_delta: Scalar,
    pub decision: IntegratedDecision,
    pub karma_floor_before: Scalar,
    pub karma_floor_after: Scalar,
    pub ker_deployable: bool,
}

impl BiAuditLogEntry {
    pub fn new(
        timestamp_unix: u64,
        bi_shard: &BrainIdentityShard,
        vt_previous: Scalar,
        vt_current: Scalar,
        decision: IntegratedDecision,
        karma_floor_after: Scalar,
        ker_deployable: bool,
    ) -> Self {
        BiAuditLogEntry {
            timestamp_unix,
            brainidentityid: bi_shard.brainidentityid,
            hexstamp: bi_shard.hexstamp,
            vt_previous,
            vt_current,
            vt_delta: vt_current - vt_previous,
            decision,
            karma_floor_before: bi_shard.karma_floor,
            karma_floor_after,
            ker_deployable,
        }
    }

    pub fn karma_violated(&self) -> bool {
        self.karma_floor_after < self.karma_floor_before
    }

    pub fn vt_violated(&self, epsilon: Scalar) -> bool {
        self.vt_delta > epsilon
    }
}

pub trait BiAuditLogger {
    fn log_entry(&mut self, entry: BiAuditLogEntry);
    fn flush(&mut self);
}

pub struct NullBiAuditLogger;

impl BiAuditLogger for NullBiAuditLogger {
    fn log_entry(&mut self, _entry: BiAuditLogEntry) {}
    fn flush(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_karma_nonslash_enforcement() {
        let physical_weights = PhysicalLyapunovWeights {
            w_energy: 0.2, w_hydraulic: 0.2, w_biology: 0.2, w_carbon: 0.2, w_materials: 0.2,
        };
        let bi_weights = BiLyapunovWeights::default();
        let controller = IntegratedController::new(physical_weights, bi_weights, 0.001, true);

        let mut bi_shard = BrainIdentityShard::new([1u8; 32], [2u8; 32], 100.0);
        let physical_rv = PhysicalRiskVector::default();
        let physical_residual = PhysicalResidual::default();

        let result = controller.evaluate_step(&physical_rv, physical_residual, &bi_shard, 99.0);
        assert_eq!(result.combined_decision, IntegratedDecision::StopKarmaViolation);
        assert!(!result.karma_preserved);

        let result = controller.evaluate_step(&physical_rv, physical_residual, &bi_shard, 100.0);
        assert_eq!(result.combined_decision, IntegratedDecision::Accept);
        assert!(result.karma_preserved);

        let result = controller.evaluate_step(&physical_rv, physical_residual, &bi_shard, 101.0);
        assert_eq!(result.combined_decision, IntegratedDecision::Accept);
        assert!(result.karma_preserved);
    }

    #[test]
    fn test_integrated_ker_tracker() {
        let mut tracker = IntegratedKerTracker::default();
        let mut bi_shard = BrainIdentityShard::new([1u8; 32], [2u8; 32], 100.0);
        let physical_rv = PhysicalRiskVector::default();
        let physical_residual = PhysicalResidual::default();

        let physical_weights = PhysicalLyapunovWeights {
            w_energy: 0.2, w_hydraulic: 0.2, w_biology: 0.2, w_carbon: 0.2, w_materials: 0.2,
        };
        let bi_weights = BiLyapunovWeights::default();
        let controller = IntegratedController::new(physical_weights, bi_weights, 0.001, true);

        for _ in 0..10 {
            let result = controller.evaluate_step(&physical_rv, physical_residual, &bi_shard, 100.0);
            tracker.update(&result);
        }

        let summary = tracker.export_ker_summary();
        assert!(summary.combined_deployable);
        assert!(summary.karma_preserved);
        assert_eq!(summary.physical_k, 1.0);
        assert_eq!(summary.bi_k, 1.0);
    }
}
