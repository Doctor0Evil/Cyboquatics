//! cyboquatics-quantum-circuits
//!
//! Quantum-learning circuit implementations for Cyboquatics governance,
//! soul-boundary enforcement, and autonomous compliance verification.
//!
//! This crate provides:
//! - Quantum-safe encryption for ALN particle signatures
//! - Variational quantum circuits for governance optimization
//! - Quantum verification protocols for soul-boundary checks
//! - Integration with classical Rust enforcement modules
//!
//! All quantum operations are subordinate to:
//! - soul.guardrail.spec.v1
//! - bio.safety.envelope.citizen.v1
//! - nanoswarm.compliance.field.v1

#![deny(unsafe_code)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod encryption;
pub mod circuits;
pub mod verification;
pub mod optimization;

pub use encryption::{QuantumSafeEncryption, Kyber1024, Dilithium5};
pub use circuits::{
    VariationalQuantumCircuit, QuantumGovernanceCircuit, SoulBoundaryCircuit,
    CircuitConfig, CircuitResult,
};
pub use verification::{QuantumVerifier, VerificationProof, VerificationResult};
pub use optimization::{QuantumOptimizer, OptimizationObjective, OptimizationResult};

/// Library version following semantic versioning
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Evidence hex for this build
pub const BUILD_EVIDENCE_HEX: &str = "0xCQ2026QUANTUM9F8E7D6C";

/// Minimum quantum safety threshold for governance operations
pub const MIN_QUANTUM_SAFETY_THRESHOLD: f64 = 0.95;

/// Quantum circuit configuration for Cyboquatics governance
#[derive(Debug, Clone)]
pub struct CyboquaticsQuantumConfig {
    /// Number of qubits for governance circuits
    pub governance_qubits: usize,
    /// Number of qubits for soul-boundary circuits
    pub soul_boundary_qubits: usize,
    /// Quantum encryption algorithm (Kyber-1024, Dilithium-5, etc.)
    pub encryption_algorithm: String,
    /// Number of optimization iterations
    pub optimization_iterations: usize,
    /// Target fidelity for quantum operations
    pub target_fidelity: f64,
}

impl Default for CyboquaticsQuantumConfig {
    fn default() -> Self {
        Self {
            governance_qubits: 16,
            soul_boundary_qubits: 8,
            encryption_algorithm: "kyber-1024".to_string(),
            optimization_iterations: 1000,
            target_fidelity: 0.99,
        }
    }
}

/// Initialize quantum backend for Cyboquatics operations
///
/// This function sets up the quantum simulation or hardware backend
/// based on available resources and configuration.
pub fn initialize_quantum_backend(
    config: &CyboquaticsQuantumConfig,
) -> Result<QuantumBackend, QuantumError> {
    let backend = QuantumBackend::new(
        config.governance_qubits + config.soul_boundary_qubits,
        &config.encryption_algorithm,
    )?;

    // Verify quantum safety threshold
    let safety_score = backend.verify_safety_threshold(MIN_QUANTUM_SAFETY_THRESHOLD)?;

    if !safety_score.passed {
        return Err(QuantumError::SafetyThresholdNotMet {
            required: MIN_QUANTUM_SAFETY_THRESHOLD,
            actual: safety_score.actual,
        });
    }

    Ok(backend)
}

/// Execute soul-boundary verification using quantum circuits
///
/// This function runs a quantum verification protocol to ensure
/// all ALN particles comply with soul.guardrail.spec constraints.
pub fn verify_soul_boundaries_quantum(
    backend: &QuantumBackend,
    particles: &[ALNParticle],
    guardrail: &SoulGuardrailSpec,
) -> Result<VerificationResult, QuantumError> {
    let circuit = SoulBoundaryCircuit::new(backend, guardrail)?;

    let mut verification_results = Vec::new();

    for particle in particles {
        let proof = circuit.verify_particle(particle)?;
        verification_results.push(proof);
    }

    let overall_result = VerificationResult::aggregate(&verification_results)?;

    // Log quantum verification to audit trail
    audit_log_quantum_verification(&overall_result)?;

    Ok(overall_result)
}

/// Optimize governance parameters using variational quantum circuits
///
/// This function uses quantum optimization to find optimal governance
/// parameters while respecting soul-boundary constraints.
pub fn optimize_governance_quantum(
    backend: &QuantumBackend,
    objective: OptimizationObjective,
    constraints: Vec<GovernanceConstraint>,
) -> Result<OptimizationResult, QuantumError> {
    let circuit = QuantumGovernanceCircuit::new(backend, &objective)?;

    let optimizer = QuantumOptimizer::new(
        circuit,
        constraints,
        backend.config.optimization_iterations,
    );

    let result = optimizer.optimize()?;

    // Verify optimization result doesn't violate soul boundaries
    if !result.soul_boundary_compliant {
        return Err(QuantumError::OptimizationViolatesSoulBoundaries);
    }

    Ok(result)
}

/// Audit log for quantum verification operations
fn audit_log_quantum_verification(result: &VerificationResult) -> Result<(), AuditError> {
    let audit_entry = QuantumAuditEntry {
        timestamp: std::time::SystemTime::now(),
        verification_result: result.clone(),
        evidence_hex: BUILD_EVIDENCE_HEX.to_string(),
        quantum_backend_id: result.backend_id.clone(),
    };

    audit_entry.write_to_chain()?;

    Ok(())
}

/// Quantum error types for Cyboquatics operations
#[derive(Debug, Clone)]
pub enum QuantumError {
    BackendInitializationFailed(String),
    SafetyThresholdNotMet { required: f64, actual: f64 },
    CircuitExecutionFailed(String),
    VerificationFailed(String),
    OptimizationViolatesSoulBoundaries,
    EncryptionError(String),
    DecryptionError(String),
}

impl std::fmt::Display for QuantumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantumError::BackendInitializationFailed(msg) => {
                write!(f, "Quantum backend initialization failed: {}", msg)
            }
            QuantumError::SafetyThresholdNotMet { required, actual } => {
                write!(
                    f,
                    "Quantum safety threshold not met: required {}, actual {}",
                    required, actual
                )
            }
            QuantumError::CircuitExecutionFailed(msg) => {
                write!(f, "Quantum circuit execution failed: {}", msg)
            }
            QuantumError::VerificationFailed(msg) => {
                write!(f, "Quantum verification failed: {}", msg)
            }
            QuantumError::OptimizationViolatesSoulBoundaries => {
                write!(f, "Optimization result violates soul boundaries")
            }
            QuantumError::EncryptionError(msg) => {
                write!(f, "Quantum encryption error: {}", msg)
            }
            QuantumError::DecryptionError(msg) => {
                write!(f, "Quantum decryption error: {}", msg)
            }
        }
    }
}

impl std::error::Error for QuantumError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_config_default() {
        let config = CyboquaticsQuantumConfig::default();
        assert_eq!(config.governance_qubits, 16);
        assert_eq!(config.soul_boundary_qubits, 8);
        assert_eq!(config.encryption_algorithm, "kyber-1024");
        assert!(config.target_fidelity >= 0.95);
    }

    #[test]
    fn test_evidence_hex_format() {
        assert!(BUILD_EVIDENCE_HEX.starts_with("0x"));
        assert_eq!(BUILD_EVIDENCE_HEX.len(), 22); // 0x + 20 hex chars
    }
}
