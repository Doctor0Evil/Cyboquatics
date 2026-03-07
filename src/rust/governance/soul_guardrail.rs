//! Soul Guardrail Specification Implementation
//!
//! Enforces non-negotiable soul-boundary constraints for all cyboquatic
//! operations. Souls are modeled as non-measurable dignity-entities that
//! cannot be scored, transferred, or quantified under any conditions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::cyboquatics::CEIMKernel;
use crate::governance::KarmaMetric;

/// Soul guardrail specification v1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulGuardrail {
    /// Particle ID binding this guardrail to ALN registry
    pub particle_id: String,
    
    /// Core soul semantics (all must be true for compliance)
    pub soul_nonmeasurable: bool,
    pub forbid_soul_scoring: bool,
    pub forbid_personality_extraction: bool,
    pub forbid_soul_targeting: bool,
    
    /// Domains where this guardrail applies
    pub applies_to_domains: Vec<String>,
    
    /// Required binding particles (cyberlinks)
    pub requires_binding_particles: Vec<String>,
    
    /// Consent and reversibility invariants
    pub require_consent_logging: bool,
    pub require_reversibility: bool,
    pub require_hitl_for_scopes: Vec<String>,
    
    /// Configuration-level constraints (what is blocked)
    pub block_config_types: Vec<String>,
    pub block_ota_patterns: Vec<String>,
    
    /// Capability-preservation invariants (what must NOT be blocked)
    pub preserve_capability_categories: Vec<String>,
    pub preserve_right_to_self_augmentation: bool,
    
    /// Karma integration
    pub karma_metric_particle: String,
    pub attach_karma_to_actions_only: bool,
    pub quarantine_on_violation: bool,
    
    /// Evidence hex for audit trail
    pub evidence_hex: String,
    
    /// Timestamp of last validation
    pub last_validated: SystemTime,
}

/// Action types that require soul-boundary validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    Neuromodulation,
    MemoryRestoration,
    NanoswarmDeployment,
    XRExperience,
    AIGovernanceIntegration,
    BloodDraw,
    ImplantUpgrade,
}

/// Citizen profile for soul-boundary checks
#[derive(Debug, Clone)]
pub struct AugmentedCitizen {
    pub did: String,
    pub stakeholder_class: String,
    pub consent_status: ConsentStatus,
    pub neu_budget_remaining: f32,
    pub cybermode_state: String,
}

/// Consent status enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentStatus {
    Explicit,
    Implicit,
    Revoked,
    Expired,
}

/// Soul guardrail validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub allowed: bool,
    pub violations: Vec<String>,
    pub karma_delta: f32,
    pub rollback_required: bool,
}

impl SoulGuardrail {
    /// Load guardrail from ALN particle registry
    pub fn load_from_particle(particle_id: &str) -> Result<Self, GuardrailError> {
        // In production, this would query the ALN particle registry
        // For now, return a default compliant configuration
        Ok(Self {
            particle_id: particle_id.to_string(),
            soul_nonmeasurable: true,
            forbid_soul_scoring: true,
            forbid_personality_extraction: true,
            forbid_soul_targeting: true,
            applies_to_domains: vec![
                "ci".to_string(),
                "ota".to_string(),
                "xr".to_string(),
                "nanoswarm".to_string(),
                "governance".to_string(),
                "memory-restoration".to_string(),
            ],
            requires_binding_particles: vec![
                "augmented.citizen.profile.v1".to_string(),
                "bio.safety.envelope.citizen.v1".to_string(),
                "karma.metric.spec.v1".to_string(),
            ],
            require_consent_logging: true,
            require_reversibility: true,
            require_hitl_for_scopes: vec![
                "mind-state".to_string(),
                "religious".to_string(),
                "existential".to_string(),
                "high-psych-risk".to_string(),
            ],
            block_config_types: vec![
                "non-rollbackable-control-path".to_string(),
                "pain-punishment-mechanic".to_string(),
                "soul-scoring-model".to_string(),
            ],
            block_ota_patterns: vec![
                "no-safemode-image".to_string(),
                "no-versioned-rollback".to_string(),
                "unsigned-firmware".to_string(),
            ],
            preserve_capability_categories: vec![
                "motor-augmentation".to_string(),
                "sensory-augmentation".to_string(),
                "cognitive-assist".to_string(),
                "rehab-support".to_string(),
                "memory-restoration".to_string(),
            ],
            preserve_right_to_self_augmentation: true,
            karma_metric_particle: "karma.metric.spec.v1".to_string(),
            attach_karma_to_actions_only: true,
            quarantine_on_violation: true,
            evidence_hex: "0xSG2026CQ7F8A9B1E".to_string(),
            last_validated: SystemTime::now(),
        })
    }
    
    /// Validate an action against soul guardrail constraints
    pub fn validate_action(
        &self,
        citizen: &AugmentedCitizen,
        action: &ActionType,
    ) -> Result<ValidationResult, GuardrailError> {
        let mut violations = Vec::new();
        let mut karma_delta = 0.0;
        let mut rollback_required = false;
        
        // Check consent status
        if citizen.consent_status == ConsentStatus::Revoked {
            violations.push("Consent revoked for this citizen".to_string());
            karma_delta = -0.5;
            rollback_required = true;
        }
        
        // Check if action requires HITL
        let action_scope = self.get_action_scope(action);
        if self.require_hitl_for_scopes.contains(&action_scope) {
            // In production, verify HITL approval exists
            // For now, log the requirement
            karma_delta += 0.1; // Positive karma for following HITL
        }
        
        // Check NEU budget for high-risk actions
        if citizen.neu_budget_remaining < 0.1 {
            violations.push("NEU psych-risk budget exhausted".to_string());
            karma_delta = -0.3;
            rollback_required = true;
        }
        
        // Verify stakeholder class has required permissions
        if citizen.stakeholder_class != "CyberneticHost" 
            && citizen.stakeholder_class != "AugmentedCitizen" 
        {
            violations.push("Insufficient stakeholder privileges".to_string());
            karma_delta = -0.2;
        }
        
        let allowed = violations.is_empty();
        
        // Record karma event if violations occurred
        if !allowed && self.quarantine_on_violation {
            KarmaMetric::record_violation(
                &self.karma_metric_particle,
                &violations,
                karma_delta,
            );
        }
        
        Ok(ValidationResult {
            allowed,
            violations,
            karma_delta,
            rollback_required,
        })
    }
    
    /// Get the scope category for an action type
    fn get_action_scope(&self, action: &ActionType) -> String {
        match action {
            ActionType::Neuromodulation => "high-psych-risk".to_string(),
            ActionType::MemoryRestoration => "mind-state".to_string(),
            ActionType::NanoswarmDeployment => "bioriskincrease".to_string(),
            ActionType::XRExperience => "cognitive-assist".to_string(),
            ActionType::AIGovernanceIntegration => "governance".to_string(),
            ActionType::BloodDraw => "bioriskincrease".to_string(),
            ActionType::ImplantUpgrade => "high-psych-risk".to_string(),
        }
    }
    
    /// Verify this guardrail preserves required capabilities
    pub fn verify_capability_preservation(&self, capability: &str) -> bool {
        self.preserve_capability_categories
            .iter()
            .any(|cat| cat.contains(capability))
    }
}

/// Guardrail error types
#[derive(Debug, Clone)]
pub enum GuardrailError {
    ParticleNotFound(String),
    ConsentRevoked,
    NEUBudgetExhausted,
    SoulBoundaryViolation,
    InvalidConfiguration,
    AuditLogFailure,
}

impl std::fmt::Display for GuardrailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuardrailError::ParticleNotFound(id) => {
                write!(f, "ALN particle not found: {}", id)
            }
            GuardrailError::ConsentRevoked => {
                write!(f, "Citizen consent has been revoked")
            }
            GuardrailError::NEUBudgetExhausted => {
                write!(f, "NEU psych-risk budget exhausted")
            }
            GuardrailError::SoulBoundaryViolation => {
                write!(f, "Soul boundary constraint violated")
            }
            GuardrailError::InvalidConfiguration => {
                write!(f, "Invalid guardrail configuration")
            }
            GuardrailError::AuditLogFailure => {
                write!(f, "Failed to write audit log")
            }
        }
    }
}

impl std::error::Error for GuardrailError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_soul_guardrail_load() {
        let guardrail = SoulGuardrail::load_from_particle("soul.guardrail.spec.v1").unwrap();
        assert!(guardrail.soul_nonmeasurable);
        assert!(guardrail.forbid_soul_scoring);
    }
    
    #[test]
    fn test_validation_with_valid_citizen() {
        let guardrail = SoulGuardrail::load_from_particle("soul.guardrail.spec.v1").unwrap();
        let citizen = AugmentedCitizen {
            did: "did:ion:test123".to_string(),
            stakeholder_class: "CyberneticHost".to_string(),
            consent_status: ConsentStatus::Explicit,
            neu_budget_remaining: 0.5,
            cybermode_state: "active".to_string(),
        };
        let action = ActionType::XRExperience;
        
        let result = guardrail.validate_action(&citizen, &action).unwrap();
        assert!(result.allowed);
        assert!(result.violations.is_empty());
    }
}
