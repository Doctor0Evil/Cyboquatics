// ============================================================================
// FILE: cyboquatics_core/src/ecosafety_kernel.rs
// DESTINATION: /cyboquatics/cyboquatics_core/src/ecosafety_kernel.rs
// LICENSE: MIT Public Good License (Non-Commercial, Open Ecosafety)
// VERSION: 1.0.0-alpha
// ============================================================================
// Cyboquatics Ecosafety Kernel - Core Rust/ALN Safety Enforcement Layer
// Implements Lyapunov residual tracking, K/E/R scoring, and corridor invariants
// ============================================================================

#![deny(warnings)]
#![deny(clippy::all)]
#![feature(never_type)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use sha3::{Digest, Sha3_256};

/// ============================================================================
/// CORE TYPE DEFINITIONS - Ecosafety Coordinate System
/// ============================================================================

/// Normalized risk coordinate in range [0.0, 1.0]
/// 0.0 = no risk, 1.0 = maximum acceptable risk threshold
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RiskCoordinate(pub f64);

impl RiskCoordinate {
    pub const ZERO: Self = Self(0.0);
    pub const MAX_SAFE: Self = Self(0.13);
    pub const THRESHOLD: Self = Self(1.0);
    
    pub fn new(value: f64) -> Result<Self, EcosafetyError> {
        if value.is_finite() && value >= 0.0 && value <= 1.0 {
            Ok(Self(value))
        } else {
            Err(EcosafetyError::InvalidRiskCoordinate(value))
        }
    }
    
    #[inline]
    pub fn value(&self) -> f64 { self.0 }
    
    #[inline]
    pub fn is_safe(&self) -> bool { self.0 <= Self::MAX_SAFE.0 }
}

/// Lyapunov residual V_t for ecosystem risk stability tracking
/// Must be non-increasing over time for system stability guarantee
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyapunovResidual {
    pub timestamp_ns: u64,
    pub value: f64,
    pub derivative: f64,
    pub is_stable: bool,
}

impl LyapunovResidual {
    pub fn new(timestamp_ns: u64, value: f64, derivative: f64) -> Self {
        Self {
            timestamp_ns,
            value,
            derivative,
            is_stable: derivative <= 0.0,
        }
    }
    
    pub fn current_timestamp_ns() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

/// K/E/R Scoring Triplet - Knowledge, Eco-impact, Risk-of-harm
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KERScore {
    pub knowledge_factor: f64,    // K ≈ 0.90-0.95 target
    pub eco_impact: f64,          // E ≈ 0.90-0.95 target
    pub risk_of_harm: f64,        // R ≈ 0.10-0.13 target
}

impl KERScore {
    pub const K_THRESHOLD: f64 = 0.90;
    pub const E_THRESHOLD: f64 = 0.90;
    pub const R_THRESHOLD: f64 = 0.13;
    
    pub fn new(k: f64, e: f64, r: f64) -> Result<Self, EcosafetyError> {
        if k.is_finite() && e.is_finite() && r.is_finite()
            && k >= 0.0 && k <= 1.0
            && e >= 0.0 && e <= 1.0
            && r >= 0.0 && r <= 1.0 {
            Ok(Self { knowledge_factor: k, eco_impact: e, risk_of_harm: r })
        } else {
            Err(EcosafetyError::InvalidKERScore(k, e, r))
        }
    }
    
    #[inline]
    pub fn is_deployable(&self) -> bool {
        self.knowledge_factor >= Self::K_THRESHOLD
            && self.eco_impact >= Self::E_THRESHOLD
            && self.risk_of_harm <= Self::R_THRESHOLD
    }
    
    #[inline]
    pub fn production_ready(&self) -> bool {
        self.is_deployable() && self.risk_of_harm <= 0.10
    }
}

/// ============================================================================
/// ECOSAFETY CORRIDOR DEFINITIONS - Multi-dimensional Safe State Space
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorridorDimension {
    WaterQuality { ph: RiskCoordinate, turbidity: RiskCoordinate, contaminants: RiskCoordinate },
    AirQuality { pm25: RiskCoordinate, pm10: RiskCoordinate, voc: RiskCoordinate },
    SoilHealth { toxicity: RiskCoordinate, erosion: RiskCoordinate, biodiversity: RiskCoordinate },
    HabitatSafety { species_risk: RiskCoordinate, displacement: RiskCoordinate, recovery: RiskCoordinate },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosafetyCorridor {
    pub corridor_id: String,
    pub dimensions: Vec<CorridorDimension>,
    pub created_ns: u64,
    pub last_validated_ns: u64,
    pub is_active: bool,
    pub ker_score: KERScore,
}

impl EcosafetyCorridor {
    pub fn new(corridor_id: String, dimensions: Vec<CorridorDimension>, ker_score: KERScore) 
        -> Result<Self, EcosafetyError> {
        if !ker_score.is_deployable() {
            return Err(EcosafetyError::CorridorNotDeployable(ker_score));
        }
        Ok(Self {
            corridor_id,
            dimensions,
            created_ns: LyapunovResidual::current_timestamp_ns(),
            last_validated_ns: LyapunovResidual::current_timestamp_ns(),
            is_active: true,
            ker_score,
        })
    }
    
    #[inline]
    pub fn validate_all_dimensions(&self) -> bool {
        self.dimensions.iter().all(|dim| self.validate_dimension(dim))
    }
    
    fn validate_dimension(&self, dimension: &CorridorDimension) -> bool {
        match dimension {
            CorridorDimension::WaterQuality { ph, turbidity, contaminants } => {
                ph.is_safe() && turbidity.is_safe() && contaminants.is_safe()
            }
            CorridorDimension::AirQuality { pm25, pm10, voc } => {
                pm25.is_safe() && pm10.is_safe() && voc.is_safe()
            }
            CorridorDimension::SoilHealth { toxicity, erosion, biodiversity } => {
                toxicity.is_safe() && erosion.is_safe() && biodiversity.is_safe()
            }
            CorridorDimension::HabitatSafety { species_risk, displacement, recovery } => {
                species_risk.is_safe() && displacement.is_safe() && recovery.is_safe()
            }
        }
    }
}

/// ============================================================================
/// QPUDATASHARD - Cryptographically Signed Audit Trail Entry
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QpuDatashard {
    pub shard_id: String,
    pub hex_stamp: String,
    pub did_signature: String,
    pub timestamp_ns: u64,
    pub corridor_id: String,
    pub ker_snapshot: KERScore,
    pub lyapunov_snapshot: LyapunovResidual,
    pub action_hash: String,
    pub previous_shard_hash: Option<String>,
}

impl QpuDatashard {
    pub fn new(
        corridor_id: String,
        ker_snapshot: KERScore,
        lyapunov_snapshot: LyapunovResidual,
        action_data: &[u8],
        previous_shard_hash: Option<String>,
    ) -> Self {
        let timestamp_ns = LyapunovResidual::current_timestamp_ns();
        let shard_id = Self::generate_shard_id(timestamp_ns);
        let action_hash = Self::hash_action(action_data);
        let hex_stamp = Self::generate_hex_stamp(&shard_id, timestamp_ns);
        let did_signature = Self::generate_did_signature(&shard_id, &hex_stamp);
        
        Self {
            shard_id,
            hex_stamp,
            did_signature,
            timestamp_ns,
            corridor_id,
            ker_snapshot,
            lyapunov_snapshot,
            action_hash,
            previous_shard_hash,
        }
    }
    
    fn generate_shard_id(timestamp_ns: u64) -> String {
        format!("shard_{:016x}", timestamp_ns)
    }
    
    fn hash_action(data: &[u8]) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        format!("0x{}", hex::encode(hasher.finalize()))
    }
    
    fn generate_hex_stamp(shard_id: &str, timestamp_ns: u64) -> String {
        format!("{}_{:08x}", shard_id, timestamp_ns as u32)
    }
    
    fn generate_did_signature(shard_id: &str, hex_stamp: &str) -> String {
        // Bostrom DID signature simulation
        format!("did:bostrom:cyboquatics:{}:{}", shard_id, hex_stamp)
    }
    
    #[inline]
    pub fn verify_chain_integrity(&self, previous_shard: Option<&QpuDatashard>) -> bool {
        match (previous_shard, &self.previous_shard_hash) {
            (None, None) => true,
            (Some(prev), Some(expected)) => {
                let prev_hash = QpuDatashard::hash_action(prev.shard_id.as_bytes());
                &prev_hash == expected
            }
            _ => false,
        }
    }
}

/// ============================================================================
/// SAFETY INVARIANTS - Rust/ALN Contract Enforcement
/// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantStatus {
    Satisfied,
    Violated { reason: &'static str },
    Unknown,
}

pub trait SafetyInvariant {
    fn name(&self) -> &'static str;
    fn check(&self) -> InvariantStatus;
    fn enforce(&self) -> Result<(), EcosafetyError>;
}

#[derive(Debug)]
pub struct CorridorCompleteInvariant {
    corridor: EcosafetyCorridor,
}

impl CorridorCompleteInvariant {
    pub fn new(corridor: EcosafetyCorridor) -> Self { Self { corridor } }
}

impl SafetyInvariant for CorridorCompleteInvariant {
    fn name(&self) -> &'static str { "invariant.corridorcomplete" }
    
    fn check(&self) -> InvariantStatus {
        if self.corridor.is_active && self.corridor.validate_all_dimensions() {
            InvariantStatus::Satisfied
        } else if !self.corridor.is_active {
            InvariantStatus::Violated { reason: "corridor_inactive" }
        } else {
            InvariantStatus::Violated { reason: "dimension_violation" }
        }
    }
    
    fn enforce(&self) -> Result<(), EcosafetyError> {
        match self.check() {
            InvariantStatus::Satisfied => Ok(()),
            InvariantStatus::Violated { reason } => {
                Err(EcosafetyError::InvariantViolation(self.name(), reason))
            }
            InvariantStatus::Unknown => {
                Err(EcosafetyError::InvariantUnknown(self.name()))
            }
        }
    }
}

#[derive(Debug)]
pub struct ResidualSafeInvariant {
    lyapunov: LyapunovResidual,
    threshold: f64,
}

impl ResidualSafeInvariant {
    pub fn new(lyapunov: LyapunovResidual, threshold: f64) -> Self {
        Self { lyapunov, threshold }
    }
}

impl SafetyInvariant for ResidualSafeInvariant {
    fn name(&self) -> &'static str { "invariant.residualsafe" }
    
    fn check(&self) -> InvariantStatus {
        if self.lyapunov.is_stable && self.lyapunov.value <= self.threshold {
            InvariantStatus::Satisfied
        } else if !self.lyapunov.is_stable {
            InvariantStatus::Violated { reason: "residual_unstable" }
        } else {
            InvariantStatus::Violated { reason: "residual_exceeds_threshold" }
        }
    }
    
    fn enforce(&self) -> Result<(), EcosafetyError> {
        match self.check() {
            InvariantStatus::Satisfied => Ok(()),
            InvariantStatus::Violated { reason } => {
                Err(EcosafetyError::InvariantViolation(self.name(), reason))
            }
            InvariantStatus::Unknown => {
                Err(EcosafetyError::InvariantUnknown(self.name()))
            }
        }
    }
}

#[derive(Debug)]
pub struct KerDeployableInvariant {
    ker_score: KERScore,
}

impl KerDeployableInvariant {
    pub fn new(ker_score: KERScore) -> Self { Self { ker_score } }
}

impl SafetyInvariant for KerDeployableInvariant {
    fn name(&self) -> &'static str { "invariant.kerdeployable" }
    
    fn check(&self) -> InvariantStatus {
        if self.ker_score.is_deployable() {
            InvariantStatus::Satisfied
        } else {
            InvariantStatus::Violated { reason: "ker_threshold_not_met" }
        }
    }
    
    fn enforce(&self) -> Result<(), EcosafetyError> {
        match self.check() {
            InvariantStatus::Satisfied => Ok(()),
            InvariantStatus::Violated { reason } => {
                Err(EcosafetyError::InvariantViolation(self.name(), reason))
            }
            InvariantStatus::Unknown => {
                Err(EcosafetyError::InvariantUnknown(self.name()))
            }
        }
    }
}

/// ============================================================================
/// ECOSAFETY ERROR TYPES
/// ============================================================================

#[derive(Debug, Clone)]
pub enum EcosafetyError {
    InvalidRiskCoordinate(f64),
    InvalidKERScore(f64, f64, f64),
    CorridorNotDeployable(KERScore),
    InvariantViolation(&'static str, &'static str),
    InvariantUnknown(&'static str),
    LyapunovInstability { value: f64, derivative: f64 },
    ShardChainBroken { expected: String, actual: String },
    HardwareDerateTriggered,
    EmergencyStopActivated,
}

impl std::fmt::Display for EcosafetyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EcosafetyError::InvalidRiskCoordinate(v) => {
                write!(f, "Invalid risk coordinate: {}", v)
            }
            EcosafetyError::InvalidKERScore(k, e, r) => {
                write!(f, "Invalid K/E/R score: K={}, E={}, R={}", k, e, r)
            }
            EcosafetyError::CorridorNotDeployable(ker) => {
                write!(f, "Corridor not deployable: K={:.2}, E={:.2}, R={:.2}",
                    ker.knowledge_factor, ker.eco_impact, ker.risk_of_harm)
            }
            EcosafetyError::InvariantViolation(name, reason) => {
                write!(f, "Invariant violation: {} - {}", name, reason)
            }
            EcosafetyError::InvariantUnknown(name) => {
                write!(f, "Invariant unknown: {}", name)
            }
            EcosafetyError::LyapunovInstability { value, derivative } => {
                write!(f, "Lyapunov instability: V_t={}, dV/dt={}", value, derivative)
            }
            EcosafetyError::ShardChainBroken { expected, actual } => {
                write!(f, "Shard chain broken: expected={}, actual={}", expected, actual)
            }
            EcosafetyError::HardwareDerateTriggered => {
                write!(f, "Hardware derate triggered - corridor violation detected")
            }
            EcosafetyError::EmergencyStopActivated => {
                write!(f, "Emergency stop activated - immediate shutdown required")
            }
        }
    }
}

impl std::error::Error for EcosafetyError {}

/// ============================================================================
/// ECOSAFETY KERNEL - Main Controller Interface
/// ============================================================================

pub struct EcosafetyKernel {
    corridors: HashMap<String, EcosafetyCorridor>,
    shard_chain: Vec<QpuDatashard>,
    ker_scores: HashMap<String, KERScore>,
    lyapunov_history: Vec<LyapunovResidual>,
    invariants: Vec<Box<dyn SafetyInvariant + Send + Sync>>,
    operation_counter: AtomicU64,
}

impl EcosafetyKernel {
    pub fn new() -> Self {
        Self {
            corridors: HashMap::new(),
            shard_chain: Vec::new(),
            ker_scores: HashMap::new(),
            lyapunov_history: Vec::new(),
            invariants: Vec::new(),
            operation_counter: AtomicU64::new(0),
        }
    }
    
    pub fn register_corridor(&mut self, corridor: EcosafetyCorridor) 
        -> Result<(), EcosafetyError> {
        let invariant = CorridorCompleteInvariant::new(corridor.clone());
        invariant.enforce()?;
        self.corridors.insert(corridor.corridor_id.clone(), corridor);
        Ok(())
    }
    
    pub fn register_ker_score(&mut self, id: String, score: KERScore) 
        -> Result<(), EcosafetyError> {
        let invariant = KerDeployableInvariant::new(score);
        invariant.enforce()?;
        self.ker_scores.insert(id, score);
        Ok(())
    }
    
    pub fn update_lyapunov(&mut self, value: f64, derivative: f64) 
        -> Result<(), EcosafetyError> {
        let residual = LyapunovResidual::new(
            LyapunovResidual::current_timestamp_ns(),
            value,
            derivative,
        );
        
        if !residual.is_stable {
            return Err(EcosafetyError::LyapunovInstability { value, derivative });
        }
        
        let invariant = ResidualSafeInvariant::new(residual.clone(), 1.0);
        invariant.enforce()?;
        
        self.lyapunov_history.push(residual);
        Ok(())
    }
    
    pub fn execute_safe_action(&mut self, action_data: &[u8], corridor_id: &str) 
        -> Result<QpuDatashard, EcosafetyError> {
        self.operation_counter.fetch_add(1, Ordering::SeqCst);
        
        // Enforce all registered invariants before action
        for invariant in &self.invariants {
            invariant.enforce()?;
        }
        
        // Validate corridor exists and is active
        let corridor = self.corridors.get(corridor_id)
            .ok_or(EcosafetyError::InvariantViolation("corridor", "not_found"))?;
        
        if !corridor.validate_all_dimensions() {
            return Err(EcosafetyError::HardwareDerateTriggered);
        }
        
        // Get current K/E/R score
        let ker_score = self.ker_scores.get(corridor_id)
            .ok_or(EcosafetyError::InvariantViolation("ker", "not_registered"))?;
        
        // Get latest Lyapunov residual
        let lyapunov = self.lyapunov_history.last()
            .ok_or(EcosafetyError::InvariantViolation("lyapunov", "no_history"))?;
        
        // Create previous shard hash if chain exists
        let prev_hash = self.shard_chain.last().map(|s| {
            QpuDatashard::hash_action(s.shard_id.as_bytes())
        });
        
        // Generate new shard
        let shard = QpuDatashard::new(
            corridor_id.to_string(),
            *ker_score,
            lyapunov.clone(),
            action_data,
            prev_hash,
        );
        
        // Verify chain integrity
        if let Some(prev_shard) = self.shard_chain.last() {
            if !shard.verify_chain_integrity(Some(prev_shard)) {
                return Err(EcosafetyError::ShardChainBroken {
                    expected: shard.previous_shard_hash.clone().unwrap(),
                    actual: QpuDatashard::hash_action(prev_shard.shard_id.as_bytes()),
                });
            }
        }
        
        self.shard_chain.push(shard.clone());
        Ok(shard)
    }
    
    pub fn add_invariant(&mut self, invariant: Box<dyn SafetyInvariant + Send + Sync>) {
        self.invariants.push(invariant);
    }
    
    pub fn get_operation_count(&self) -> u64 {
        self.operation_counter.load(Ordering::SeqCst)
    }
    
    pub fn get_shard_chain_length(&self) -> usize {
        self.shard_chain.len()
    }
    
    pub fn emergency_stop(&mut self) -> Result<(), EcosafetyError> {
        self.corridors.values_mut().for_each(|c| c.is_active = false);
        Err(EcosafetyError::EmergencyStopActivated)
    }
}

impl Default for EcosafetyKernel {
    fn default() -> Self { Self::new() }
}

/// ============================================================================
/// UNIT TESTS - Ecosafety Kernel Validation
/// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_risk_coordinate_bounds() {
        assert!(RiskCoordinate::new(0.0).is_ok());
        assert!(RiskCoordinate::new(0.13).is_ok());
        assert!(RiskCoordinate::new(1.0).is_ok());
        assert!(RiskCoordinate::new(-0.1).is_err());
        assert!(RiskCoordinate::new(1.5).is_err());
    }
    
    #[test]
    fn test_ker_score_deployable() {
        let ker = KERScore::new(0.94, 0.92, 0.12).unwrap();
        assert!(ker.is_deployable());
        assert!(!ker.production_ready());
        
        let ker_prod = KERScore::new(0.95, 0.93, 0.09).unwrap();
        assert!(ker_prod.is_deployable());
        assert!(ker_prod.production_ready());
    }
    
    #[test]
    fn test_corridor_validation() {
        let ker = KERScore::new(0.94, 0.92, 0.12).unwrap();
        let dimensions = vec![
            CorridorDimension::WaterQuality {
                ph: RiskCoordinate(0.05),
                turbidity: RiskCoordinate(0.08),
                contaminants: RiskCoordinate(0.10),
            }
        ];
        let corridor = EcosafetyCorridor::new("test_corridor".to_string(), dimensions, ker).unwrap();
        assert!(corridor.validate_all_dimensions());
    }
    
    #[test]
    fn test_kernel_safe_action() {
        let mut kernel = EcosafetyKernel::new();
        let ker = KERScore::new(0.94, 0.92, 0.12).unwrap();
        let dimensions = vec![
            CorridorDimension::WaterQuality {
                ph: RiskCoordinate(0.05),
                turbidity: RiskCoordinate(0.08),
                contaminants: RiskCoordinate(0.10),
            }
        ];
        let corridor = EcosafetyCorridor::new("test".to_string(), dimensions, ker).unwrap();
        kernel.register_corridor(corridor).unwrap();
        kernel.register_ker_score("test".to_string(), ker).unwrap();
        kernel.update_lyapunov(0.5, -0.1).unwrap();
        
        let shard = kernel.execute_safe_action(b"test_action", "test").unwrap();
        assert_eq!(kernel.get_shard_chain_length(), 1);
        assert_eq!(kernel.get_operation_count(), 1);
        assert!(shard.hex_stamp.starts_with("shard_"));
    }
}

// ============================================================================
// END OF FILE: cyboquatics_core/src/ecosafety_kernel.rs
// ============================================================================
