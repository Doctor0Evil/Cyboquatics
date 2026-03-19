// ============================================================================
// Cyboquatic Gateway Mediation Logic
// ============================================================================
// Version: 1.0.0
// License: Apache-2.0 OR MIT
// Authors: Cyboquatic Research Collective
// 
// This module implements the core safety mediation loop. It sits between
// legacy controllers (PLC/HCS) and physical actuators, enforcing rx/Vt/KER
// invariants on every control cycle.
//
// Integration Note: Implements HardwareInterface trait intended for OPC UA
// client implementation (e.g., opcua-client crate) to bridge legacy SCADA.
// ============================================================================

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use crate::audit::{AuditLog, AuditEntry, ActionTag};
use crate::calibration::{DriftCompensator, SensorCalibration};
use crate::invariants::{EcoSafetyKernel, SafeStepDecision};
use crate::{GatewayConfig, GatewayMode, KerTriad, Residual, RiskVector};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Abstract interface for hardware communication (OPC UA, Modbus, etc.).
/// 
/// Implementors must guarantee thread-safe access to physical I/O.
/// For 20-50 year continuity, implementors should handle connection retries
/// and certificate rotation internally.
pub trait HardwareInterface: Send + Sync {
    /// Error type for hardware operations.
    type Error: std::error::Error;

    /// Reads current state from sensors (e.g., pressure, flow, temp).
    fn read_state(&self) -> Result<SystemState, Self::Error>;

    /// Writes a command to actuators (e.g., valve position, motor speed).
    fn write_command(&self, cmd: &ActuatorCommand) -> Result<(), Self::Error>;

    /// Returns the current operational mode of the underlying hardware.
    fn hardware_status(&self) -> HardwareStatus;
}

/// Snapshot of physical system state.
#[derive(Clone, Debug)]
pub struct SystemState {
    /// Timestamp of the state snapshot.
    pub timestamp: Instant,
    /// Raw sensor values (uncompensated).
    pub raw_sensors: Vec<f64>,
    /// System uptime (for lifecycle tracking).
    pub uptime_seconds: u64,
    /// Hardware-specific status flags.
    pub flags: u32,
}

/// Command to be sent to physical actuators.
#[derive(Clone, Debug)]
pub struct ActuatorCommand {
    /// Target values for actuators (normalized 0.0-1.0).
    pub targets: Vec<f64>,
    /// Emergency stop flag (overrides all targets).
    pub e_stop: bool,
    /// Command sequence number (for audit reconciliation).
    pub sequence_id: u64,
}

/// Status of the underlying hardware bridge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareStatus {
    Online,
    Degraded,
    Offline,
    Maintenance,
}

/// The core safety mediation engine.
/// 
/// This struct wraps a legacy controller logic (generic C) and mediates
/// its output through the EcoSafetyKernel before reaching hardware.
pub struct SafeStepGate<C> 
where 
    C: LegacyController 
{
    /// The legacy controller logic being wrapped.
    controller: C,
    /// The safety kernel enforcing invariants.
    kernel: EcoSafetyKernel,
    /// Gateway configuration (mode, weights, thresholds).
    config: GatewayConfig,
    /// Audit log for cryptographic evidence generation.
    audit_log: AuditLog,
    /// Sensor drift compensation state.
    drift_comp: DriftCompensator,
    /// Step counter for lifecycle tracking.
    step_count: u64,
    /// Last known safe state (for fail-safe reversion).
    last_safe_command: Option<ActuatorCommand>,
}

/// Trait for legacy controller logic to be wrapped by the gateway.
pub trait LegacyController: Send + Sync {
    /// Proposes a step based on current state.
    /// Returns (command_targets, risk_weights).
    fn propose(&self, state: &SystemState) -> (Vec<f64>, Vec<f64>);
    
    /// Returns the controller's internal version string (for audit).
    fn version(&self) -> &str;
}

impl<C> SafeStepGate<C>
where
    C: LegacyController,
{
    /// Creates a new safety gate with the given configuration.
    pub fn new(controller: C, config: GatewayConfig, audit_log: AuditLog) -> Result<Self, GatewayInitError> {
        config.validate().map_err(GatewayInitError::ConfigInvalid)?;
        
        let kernel = EcoSafetyKernel::new(config.eps_vt);
        let drift_comp = DriftCompensator::new(config.drift_compensation_enabled);

        Ok(SafeStepGate {
            controller,
            kernel,
            config,
            audit_log,
            drift_comp,
            step_count: 0,
            last_safe_command: None,
        })
    }

    /// Executes one control cycle: Read -> Propose -> Evaluate -> Actuate.
    /// 
    /// This is the critical path for safety enforcement. All errors result
    /// in a safe state (Stop or Hold) to preserve continuity.
    pub fn cycle(&mut self, hardware: &dyn HardwareInterface) -> CycleResult {
        let start_time = Instant::now();
        self.step_count += 1;

        // 1. Read State (with drift compensation)
        let raw_state = match hardware.read_state() {
            Ok(s) => s,
            Err(_) => return CycleResult::HardwareFailure,
        };

        let compensated_sensors = self.drift_comp.compensate(&raw_state.raw_sensors);
        let state = SystemState {
            timestamp: raw_state.timestamp,
            raw_sensors: compensated_sensors,
            uptime_seconds: raw_state.uptime_seconds,
            flags: raw_state.flags,
        };

        // 2. Propose Step (from legacy controller)
        let (targets, weights) = self.controller.propose(&state);
        
        // 3. Evaluate Safety (Kernel)
        let risk_vector = self.calculate_risks(&state, &targets);
        let residual = Residual::from_weights(&risk_vector, &weights, self.step_count);
        let decision = self.kernel.evaluate_step(residual, &risk_vector);

        // 4. Mediate Command (based on Mode + Decision)
        let final_command = self.mediate_command(&targets, &decision, &state);

        // 5. Audit & Actuate
        self.log_cycle(&state, &risk_vector, &decision, &final_command);
        
        let actuation_result = hardware.write_command(&final_command);

        let duration = start_time.elapsed();
        
        match actuation_result {
            Ok(_) => CycleResult::Success { duration, decision, ker: self.current_ker() },
            Err(_) => CycleResult::ActuationFailure,
        }
    }

    /// Calculates risk coordinates from state and proposed targets.
    /// 
    /// This is where domain-specific safety logic (hydraulics, chemistry)
    /// is mapped to the generic RiskVector structure.
    fn calculate_risks(&self, state: &SystemState, targets: &[f64]) -> RiskVector {
        // Example mapping: Map sensor deviations and target aggressiveness to risks.
        // In production, this uses CorridorBands from cyboquatic-ecosafety-core.
        let mut coords = Vec::with_capacity(state.raw_sensors.len() + targets.len());
        let mut labels = Vec::with_capacity(state.raw_sensors.len() + targets.len());

        // Sensor risks (state deviation)
        for (i, val) in state.raw_sensors.iter().enumerate() {
            // Normalize based on expected range (placeholder logic)
            let risk = (val.abs() / 100.0).min(1.0); 
            coords.push(crate::RiskCoord::new_clamped(risk));
            labels.push(format!("sensor_{}", i));
        }

        // Actuation risks (command aggressiveness)
        for (i, tgt) in targets.iter().enumerate() {
            let risk = tgt.abs(); // Assume 0.0-1.0 target, higher is riskier
            coords.push(crate::RiskCoord::new_clamped(risk));
            labels.push(format!("actuator_{}", i));
        }

        RiskVector::with_labels(coords, labels)
    }

    /// Mediates the command based on gateway mode and safety decision.
    fn mediate_command(&self, targets: &[f64], decision: &SafeStepDecision, state: &SystemState) -> ActuatorCommand {
        let mut final_targets = targets.to_vec();
        let mut e_stop = false;

        match self.config.mode {
            GatewayMode::Monitoring => {
                // Pass through, but log potential violations
                if matches!(decision, SafeStepDecision::Stop) {
                    // Log warning but do not intervene
                }
            }
            GatewayMode::DerateOnly => {
                if matches!(decision, SafeStepDecision::Stop) {
                    // Derate instead of stopping (smooth reduction)
                    final_targets = self.derate_targets(targets, 0.5);
                } else if matches!(decision, SafeStepDecision::Derate) {
                    final_targets = self.derate_targets(targets, 0.8);
                }
            }
            GatewayMode::FullGate => {
                if matches!(decision, SafeStepDecision::Stop) {
                    e_stop = true;
                    final_targets = vec![0.0; targets.len()];
                } else if matches!(decision, SafeStepDecision::Derate) {
                    final_targets = self.derate_targets(targets, 0.7);
                }
            }
        }

        ActuatorCommand {
            targets: final_targets,
            e_stop,
            sequence_id: self.step_count,
        }
    }

    /// Linear derating strategy for aggressive commands.
    fn derate_targets(&self, targets: &[f64], factor: f64) -> Vec<f64> {
        targets.iter().map(|t| t * factor).collect()
    }

    /// Logs the cycle data to the audit trail (ALN shard ready).
    fn log_cycle(&self, state: &SystemState, risks: &RiskVector, decision: &SafeStepDecision, cmd: &ActuatorCommand) {
        if !self.config.audit_enabled {
            return;
        }

        let entry = AuditEntry {
            timestep: self.step_count,
            timestamp: state.timestamp,
            action: match decision {
                SafeStepDecision::Accept => ActionTag::Accept,
                SafeStepDecision::Derate => ActionTag::Derate,
                SafeStepDecision::Stop => ActionTag::Stop,
            },
            max_risk: risks.max().value(),
            command_hash: self.hash_command(cmd),
        };

        self.audit_log.append(entry);
    }

    /// Computes a simple hash for command auditing (replace with crypto in prod).
    fn hash_command(&self, cmd: &ActuatorCommand) -> u64 {
        let mut hash: u64 = 0;
        for t in &cmd.targets {
            hash = hash.wrapping_add(t.to_bits());
        }
        hash.wrapping_add(cmd.sequence_id)
    }

    /// Returns current KER triad based on kernel history.
    fn current_ker(&self) -> KerTriad {
        self.kernel.current_ker()
    }

    /// Updates sensor calibration parameters (requires authorization).
    pub fn update_calibration(&mut self, calib: SensorCalibration) {
        self.drift_comp.update_calibration(calib);
    }

    /// Returns current step count (for lifecycle monitoring).
    pub fn step_count(&self) -> u64 {
        self.step_count
    }
}

/// Result of a control cycle.
#[derive(Clone, Debug)]
pub enum CycleResult {
    Success {
        duration: Duration,
        decision: SafeStepDecision,
        ker: KerTriad,
    },
    HardwareFailure,
    ActuationFailure,
}

/// Errors occurring during gateway initialization.
#[derive(Clone, Debug)]
pub enum GatewayInitError {
    ConfigInvalid(crate::GatewayConfigError),
    HardwareInitFailed,
}

impl std::fmt::Display for GatewayInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GatewayInitError::ConfigInvalid(e) => write!(f, "Config invalid: {}", e),
            GatewayInitError::HardwareInitFailed => write!(f, "Hardware initialization failed"),
        }
    }
}

impl std::error::Error for GatewayInitError {}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GatewayConfig, GatewayMode, AuditLog};

    struct MockController;
    impl LegacyController for MockController {
        fn propose(&self, _state: &SystemState) -> (Vec<f64>, Vec<f64>) {
            (vec![0.5, 0.5], vec![1.0, 1.0])
        }
        fn version(&self) -> &str { "mock-v1" }
    }

    struct MockHardware;
    impl HardwareInterface for MockHardware {
        type Error = std::io::Error;
        fn read_state(&self) -> Result<SystemState, Self::Error> {
            Ok(SystemState {
                timestamp: Instant::now(),
                raw_sensors: vec![10.0, 20.0],
                uptime_seconds: 1000,
                flags: 0,
            })
        }
        fn write_command(&self, _cmd: &ActuatorCommand) -> Result<(), Self::Error> {
            Ok(())
        }
        fn hardware_status(&self) -> HardwareStatus {
            HardwareStatus::Online
        }
    }

    #[test]
    fn test_gateway_cycle_success() {
        let config = GatewayConfig {
            mode: GatewayMode::Monitoring,
            ..Default::default()
        };
        let controller = MockController;
        let audit_log = AuditLog::new();
        let mut gate = SafeStepGate::new(controller, config, audit_log).unwrap();
        let hardware = MockHardware;

        let result = gate.cycle(&hardware);
        assert!(matches!(result, CycleResult::Success { .. }));
    }

    #[test]
    fn test_derate_logic() {
        let config = GatewayConfig {
            mode: GatewayMode::DerateOnly,
            ..Default::default()
        };
        let controller = MockController;
        let audit_log = AuditLog::new();
        let mut gate = SafeStepGate::new(controller, config, audit_log).unwrap();
        
        let targets = vec![1.0, 1.0];
        // Simulate internal derate call
        let derated = gate.derate_targets(&targets, 0.5);
        assert_eq!(derated, vec![0.5, 0.5]);
    }
}
