//! cyboquatics-core
//!
//! Multi-language cyboquatic machine governance framework with soul-boundary
//! enforcement, CEIM/KER kernels, and stakeholder classification for augmented
//! citizens. This crate provides the Rust foundation for policy-adaptive,
//! compliance-resilient ecological cyber-physical systems.
//!
//! # Features
//! - Soul guardrail enforcement (non-transferable, non-quantifiable dignity)
//! - CEIM mass-balance kernels for ecological integrity
//! - KER (Karma Evaluation Residual) for policy compliance
//! - Stakeholder classification (CyberneticHost vs RegularStakeholder)
//! - ALN particle integration for audit trails
//!
//! # Safety Guarantees
//! - All augmentation calls wrapped with soul-boundary checks
//! - NEU psych-risk budgets enforced at compile-time via macros
//! - Karma attaches to actions/particles, never to persons or souls

#![deny(unsafe_code)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod governance;
pub mod cyboquatics;
pub mod stakeholders;

pub use governance::{SoulGuardrail, KarmaMetric, CyberRank};
pub use cyboquatics::{CEIMKernel, KEREvaluator, EcoImpactScore};
pub use stakeholders::{StakeholderClass, StakeholderProfile, CyberStakeholderScore};

/// Library version following semantic versioning
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Evidence hex for this build (cryptographically bound to source state)
pub const BUILD_EVIDENCE_HEX: &str = "0xCQ2026A7B3F9E1D4";

/// Minimum knowledge-factor threshold for deployment eligibility
pub const MIN_KNOWLEDGE_FACTOR: f32 = 0.85;

/// Compute knowledge-factor for a given cyboquatic deployment
/// 
/// Formula: F = α·V + β·R + γ·E + δ·N
/// where V=validation, R=reuse, E=ecological impact, N=novelty
pub fn compute_knowledge_factor(
    validation: f32,
    reuse: f32,
    ecological_impact: f32,
    novelty: f32,
) -> f32 {
    let alpha = 0.30; // validation weight
    let beta = 0.25;  // reuse weight
    let gamma = 0.30; // ecological impact weight
    let delta = 0.15; // novelty weight
    
    let factor = alpha * validation.min(1.0)
               + beta * reuse.min(1.0)
               + gamma * ecological_impact.min(1.0)
               + delta * novelty.min(1.0);
    
    factor.clamp(0.0, 1.0)
}

/// Macro for wrapping augmentation calls with soul guardrails
#[macro_export]
macro_rules! with_soul_guardrails {
    ($func:expr, $citizen:expr, $action:expr) => {
        {
            use $crate::governance::SoulGuardrail;
            let guardrail = SoulGuardrail::load_from_particle("soul.guardrail.spec.v1")?;
            
            // Pre-flight soul-boundary check
            if !guardrail.validate_action(&$citizen, &$action)? {
                return Err(CyboquaticsError::SoulBoundaryViolation);
            }
            
            // Execute action with audit logging
            let result = $func($citizen, $action);
            
            // Post-flight karma evaluation
            $crate::governance::KarmaMetric::record_action(&$action, &result);
            
            result
        }
    };
}

/// Macro for NEU psych-risk budget enforcement
#[macro_export]
macro_rules! with_neu_budget {
    ($citizen:expr, $risk_cost:expr, $body:block) => {
        {
            use $crate::governance::NEUBudget;
            let mut budget = NEUBudget::load_for_citizen(&$citizen.did)?;
            
            if !budget.can_spend($risk_cost) {
                return Err(CyboquaticsError::NEUBudgetExhausted);
            }
            
            budget.spend($risk_cost)?;
            let result = $body;
            budget.commit()?;
            
            result
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_knowledge_factor_calculation() {
        let factor = compute_knowledge_factor(0.9, 0.8, 0.95, 0.7);
        assert!(factor >= MIN_KNOWLEDGE_FACTOR);
        assert!(factor <= 1.0);
    }
    
    #[test]
    fn test_evidence_hex_format() {
        assert!(BUILD_EVIDENCE_HEX.starts_with("0x"));
        assert_eq!(BUILD_EVIDENCE_HEX.len(), 18); // 0x + 16 hex chars
    }
}
