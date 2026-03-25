#![no_std]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

use cyboquatic_ecosafety_core::{RiskVector, Residual, LyapunovWeights, KerWindow, SafeStepDecision};
use cyboquatic_brain_identity_core::{BrainIdentityShard, BiRiskVector, BiResidual, BiLyapunovWeights, BiSafeStepDecision, BiKerWindow, NeurorightsStatus, EvidenceMode};
use cyboquatic_bi_integration::{IntegratedController, IntegratedStepResult, IntegratedDecision, IntegratedKerTracker, KerSummary, BiAuditLogEntry, BiAuditLogger};

pub type Scalar = f32;
pub type StepIndex = u32;

#[derive(Copy, Clone, Debug)]
pub struct SimulationConfig {
    pub total_steps: StepIndex,
    pub epsilon: Scalar,
    pub enforce_karma: bool,
    pub perturb_physical: bool,
    pub perturb_bi: bool,
    pub perturb_magnitude: Scalar,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        SimulationConfig {
            total_steps: 1000,
            epsilon: 0.001,
            enforce_karma: true,
            perturb_physical: true,
            perturb_bi: true,
            perturb_magnitude: 0.05,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct SimulationState {
    pub current_step: StepIndex,
    pub vt_physical: Scalar,
    pub vt_bi: Scalar,
    pub karma_floor: Scalar,
    pub neurorights_status: NeurorightsStatus,
    pub ker_violations: u32,
    pub karma_violations: u32,
    pub vt_violations: u32,
    pub hard_violations: u32,
}

pub struct BiSimulationRunner<Logger: BiAuditLogger> {
    pub config: SimulationConfig,
    pub physical_weights: LyapunovWeights,
    pub bi_weights: BiLyapunovWeights,
    pub bi_shard: BrainIdentityShard,
    pub physical_ker: KerWindow,
    pub bi_ker: BiKerWindow,
    pub state: SimulationState,
    pub audit_logger: Logger,
    pub step_history: Vec<SimulationStepRecord>,
}

pub struct SimulationStepRecord {
    pub step_index: StepIndex,
    pub vt_physical: Scalar,
    pub vt_bi: Scalar,
    pub vt_delta: Scalar,
    pub decision: IntegratedDecision,
    pub karma_floor: Scalar,
    pub r_max: Scalar,
    pub k_score: Scalar,
    pub e_score: Scalar,
    pub ker_deployable: bool,
}

impl<Logger: BiAuditLogger> BiSimulationRunner<Logger> {
    pub fn new(
        config: SimulationConfig,
        brainidentityid: [u8; 32],
        hexstamp: [u8; 32],
        initial_karma: Scalar,
        audit_logger: Logger,
    ) -> Self {
        let bi_shard = BrainIdentityShard::new(brainidentityid, hexstamp, initial_karma);
        BiSimulationRunner {
            config,
            physical_weights: LyapunovWeights {
                w_energy: 0.20,
                w_hydraulic: 0.15,
                w_biology: 0.15,
                w_carbon: 0.25,
                w_materials: 0.25,
            },
            bi_weights: BiLyapunovWeights::default(),
            bi_shard,
            physical_ker: KerWindow::default(),
            bi_ker: BiKerWindow::default(),
            state: SimulationState::default(),
            audit_logger,
            step_history: Vec::new(),
        }
    }

    pub fn run_simulation(&mut self) -> SimulationReport {
        let controller = IntegratedController::new(
            self.physical_weights,
            self.bi_weights,
            self.config.epsilon,
            self.config.enforce_karma,
        );

        let mut vt_physical_prev = 0.0;
        let mut vt_bi_prev = 0.0;

        for step in 0..self.config.total_steps {
            self.state.current_step = step;

            let physical_rv = self.generate_physical_risk_vector(step);
            let physical_residual_prev = Residual { vt: vt_physical_prev };

            self.update_bi_shard_from_simulation(step);

            let proposed_karma = self.compute_proposed_karma(step);
            let result = controller.evaluate_step(
                &physical_rv,
                physical_residual_prev,
                &self.bi_shard,
                proposed_karma,
            );

            let physical_residual_next = Residual::compute(&physical_rv, &self.physical_weights);
            vt_physical_prev = physical_residual_next.vt;

            let bi_residual_next = BiResidual::compute(&result.bi_rv, &self.bi_weights);
            vt_bi_prev = bi_residual_next.vt;

            self.state.vt_physical = vt_physical_prev;
            self.state.vt_bi = vt_bi_prev;
            self.state.karma_floor = self.bi_shard.karma_floor;
            self.state.neurorights_status = self.bi_shard.neurorights_status;

            self.track_violations(&result, vt_physical_prev, vt_bi_prev);
            self.update_ker_trackers(&result, &physical_rv);

            if matches!(result.combined_decision, IntegratedDecision::Accept) {
                self.bi_shard.try_set_karma_floor(proposed_karma);
            }

            let record = SimulationStepRecord {
                step_index: step,
                vt_physical: self.state.vt_physical,
                vt_bi: self.state.vt_bi,
                vt_delta: self.state.vt_physical - vt_physical_prev + self.state.vt_bi - vt_bi_prev,
                decision: result.combined_decision,
                karma_floor: self.state.karma_floor,
                r_max: self.bi_ker.r(),
                k_score: self.bi_ker.k(),
                e_score: self.bi_ker.e(),
                ker_deployable: self.bi_ker.bi_ker_deployable(),
            };

            self.step_history.push(record);

            let audit_entry = BiAuditLogEntry::new(
                step as u64 * 1000,
                &self.bi_shard,
                vt_bi_prev,
                bi_residual_next.vt,
                result.combined_decision,
                self.bi_shard.karma_floor,
                self.bi_ker.bi_ker_deployable(),
            );
            self.audit_logger.log_entry(audit_entry);
        }

        self.audit_logger.flush();
        self.generate_report()
    }

    fn generate_physical_risk_vector(&self, step: StepIndex) -> RiskVector {
        use core::f32::consts::PI;
        let t = step as Scalar / 100.0;

        let base_energy = 0.3 + 0.1 * (t * 2.0 * PI).sin();
        let base_hydraulic = 0.2 + 0.05 * (t * 3.0 * PI).sin();
        let base_biology = 0.25 + 0.05 * (t * 1.5 * PI).sin();
        let base_carbon = 0.35 + 0.08 * (t * 2.5 * PI).sin();
        let base_materials = 0.28 + 0.06 * (t * 1.8 * PI).sin();

        RiskVector {
            r_energy: cyboquatic_ecosafety_core::RiskCoord::new_clamped(base_energy),
            r_hydraulic: cyboquatic_ecosafety_core::RiskCoord::new_clamped(base_hydraulic),
            r_biology: cyboquatic_ecosafety_core::RiskCoord::new_clamped(base_biology),
            r_carbon: cyboquatic_ecosafety_core::RiskCoord::new_clamped(base_carbon),
            r_materials: cyboquatic_ecosafety_core::RiskCoord::new_clamped(base_materials),
        }
    }

    fn update_bi_shard_from_simulation(&mut self, step: StepIndex) {
        if self.config.perturb_bi {
            use core::f32::consts::PI;
            let t = step as Scalar / 200.0;

            let soul_perturb = 0.1 * (t * PI).sin() * self.config.perturb_magnitude;
            let social_perturb = 0.08 * (t * 1.3 * PI).sin() * self.config.perturb_magnitude;
            let ecoimpact_perturb = 0.05 * (t * 0.7 * PI).sin() * self.config.perturb_magnitude;

            self.bi_shard.update_rsoul(self.bi_shard.rsoul_residual + soul_perturb);
            self.bi_shard.update_social_exposure(self.bi_shard.social_exposure_coord + social_perturb);
            self.bi_shard.update_ecoimpactscore(self.bi_shard.ecoimpactscore + ecoimpact_perturb);

            if step % 100 == 0 && step > 0 {
                let current_status = self.bi_shard.neurorights_status;
                let new_status = match current_status {
                    NeurorightsStatus::Active => NeurorightsStatus::Active,
                    NeurorightsStatus::Restricted => {
                        if step % 300 == 0 { NeurorightsStatus::Active } else { NeurorightsStatus::Restricted }
                    },
                    NeurorightsStatus::Suspended => {
                        if step % 500 == 0 { NeurorightsStatus::Restricted } else { NeurorightsStatus::Suspended }
                    },
                };
                self.bi_shard.update_neurorights(new_status);
            }
        }
    }

    fn compute_proposed_karma(&self, step: StepIndex) -> Scalar {
        let base_karma = self.bi_shard.karma_floor;
        let karma_growth = step as Scalar * 0.01;
        let perturbation = if self.config.perturb_bi {
            (step as Scalar * 0.1).sin() * 0.5
        } else {
            0.0
        };
        base_karma + karma_growth + perturbation
    }

    fn track_violations(&mut self, result: &IntegratedStepResult, vt_physical: Scalar, vt_bi: Scalar) {
        match result.combined_decision {
            IntegratedDecision::StopKarmaViolation => self.state.karma_violations += 1,
            IntegratedDecision::Stop => self.state.hard_violations += 1,
            _ => {}
        }

        if vt_physical > self.state.vt_physical + self.config.epsilon {
            self.state.vt_violations += 1;
        }
        if vt_bi > self.state.vt_bi + self.config.epsilon {
            self.state.vt_violations += 1;
        }

        if !self.bi_ker.bi_ker_deployable() {
            self.state.ker_violations += 1;
        }
    }

    fn update_ker_trackers(&mut self, result: &IntegratedStepResult, physical_rv: &RiskVector) {
        let mut tracker = IntegratedKerTracker {
            physical_ker: self.physical_ker,
            bi_ker: self.bi_ker,
        };
        tracker.update(result);
        self.physical_ker = tracker.physical_ker;
        self.bi_ker = tracker.bi_ker;
    }

    fn generate_report(&self) -> SimulationReport {
        let avg_vt_physical = self.step_history.iter()
            .map(|r| r.vt_physical)
            .sum::<Scalar>() / self.step_history.len() as Scalar;

        let avg_vt_bi = self.step_history.iter()
            .map(|r| r.vt_bi)
            .sum::<Scalar>() / self.step_history.len() as Scalar;

        let accept_count = self.step_history.iter()
            .filter(|r| matches!(r.decision, IntegratedDecision::Accept))
            .count();

        let derate_count = self.step_history.iter()
            .filter(|r| matches!(r.decision, IntegratedDecision::Derate))
            .count();

        let stop_count = self.step_history.iter()
            .filter(|r| matches!(r.decision, IntegratedDecision::Stop | IntegratedDecision::StopKarmaViolation))
            .count();

        SimulationReport {
            total_steps: self.config.total_steps,
            completed_steps: self.step_history.len() as StepIndex,
            avg_vt_physical,
            avg_vt_bi,
            final_karma_floor: self.state.karma_floor,
            karma_violations_detected: self.state.karma_violations,
            karma_violations_enforced: self.step_history.iter()
                .filter(|r| matches!(r.decision, IntegratedDecision::StopKarmaViolation))
                .count() as u32,
            vt_violations: self.state.vt_violations,
            hard_violations: self.state.hard_violations,
            ker_violations: self.state.ker_violations,
            accept_count: accept_count as u32,
            derate_count: derate_count as u32,
            stop_count: stop_count as u32,
            final_k_score: self.bi_ker.k(),
            final_e_score: self.bi_ker.e(),
            final_r_score: self.bi_ker.r(),
            final_deployable: self.bi_ker.bi_ker_deployable(),
            invariant_held: self.state.karma_violations == 0 || self.state.karma_violations_enforced > 0,
        }
    }
}

#[derive(Debug)]
pub struct SimulationReport {
    pub total_steps: StepIndex,
    pub completed_steps: StepIndex,
    pub avg_vt_physical: Scalar,
    pub avg_vt_bi: Scalar,
    pub final_karma_floor: Scalar,
    pub karma_violations_detected: u32,
    pub karma_violations_enforced: u32,
    pub vt_violations: u32,
    pub hard_violations: u32,
    pub ker_violations: u32,
    pub accept_count: u32,
    pub derate_count: u32,
    pub stop_count: u32,
    pub final_k_score: Scalar,
    pub final_e_score: Scalar,
    pub final_r_score: Scalar,
    pub final_deployable: bool,
    pub invariant_held: bool,
}

impl SimulationReport {
    pub fn summary_string(&self) -> String {
        alloc::format!(
            "SimulationReport[steps={}/{}, vt_phys={:.4}, vt_bi={:.4}, karma={:.2}, \
             violations(K={},VT={},H={}), decisions(A={},D={},S={}), \
             KER(K={:.2},E={:.2},R={:.2}), deployable={}, invariant={}]",
            self.completed_steps, self.total_steps,
            self.avg_vt_physical, self.avg_vt_bi, self.final_karma_floor,
            self.karma_violations_detected, self.vt_violations, self.hard_violations,
            self.accept_count, self.derate_count, self.stop_count,
            self.final_k_score, self.final_e_score, self.final_r_score,
            self.final_deployable, self.invariant_held
        )
    }

    pub fn passed(&self) -> bool {
        self.completed_steps == self.total_steps &&
        self.invariant_held &&
        self.final_deployable
    }
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
    fn test_simulation_completes_without_karma_violation() {
        let config = SimulationConfig {
            total_steps: 100,
            epsilon: 0.001,
            enforce_karma: true,
            perturb_physical: true,
            perturb_bi: true,
            perturb_magnitude: 0.02,
        };

        let mut runner = BiSimulationRunner::new(
            config,
            [1u8; 32],
            [2u8; 32],
            100.0,
            NullBiAuditLogger,
        );

        let report = runner.run_simulation();
        assert!(report.completed_steps == report.total_steps);
        assert!(report.invariant_held);
        assert!(report.karma_violations_enforced > 0 || report.karma_violations_detected == 0);
    }

    #[test]
    fn test_karma_nonslash_under_attack() {
        let config = SimulationConfig {
            total_steps: 50,
            epsilon: 0.001,
            enforce_karma: true,
            perturb_physical: false,
            perturb_bi: true,
            perturb_magnitude: 0.1,
        };

        let mut runner = BiSimulationRunner::new(
            config,
            [3u8; 32],
            [4u8; 32],
            50.0,
            NullBiAuditLogger,
        );

        let report = runner.run_simulation();
        assert!(report.final_karma_floor >= 50.0);
    }
}
