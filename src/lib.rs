//! cybernetic-evolution-reward-guard-core
//! Core verification logic for EcoNet reward integrity.

#![deny(unsafe_code)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod verification;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ed25519_dalek::{PublicKey, Signature, Verifier};

/// Authorized DID for this reward stream (Doctor Jacob Scott Farmer).
pub const AUTHORIZED_DID: &str =
    "did:ion:EiD8J2b3K8k9Q8x9L7m2n4p1q5r6s7t8u9v0w1x2y3z4A5B6C7D8E9F0";

/// Bostrom address for reward settlement.
pub const AUTHORIZED_BOSTROM: &str =
    "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";

/// Evidence hex for this build.
pub const BUILD_EVIDENCE_HEX: &str = "0xECONET2026REW9D8C7B6A";

/// Reward claim structure signed by the author.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardClaim {
    /// Crate name in Cargo metadata.
    pub crate_name: String,
    /// SHA256 hash of crate metadata/content.
    pub crate_hash: String,
    /// DID of the author claiming rewards.
    pub author_did: String,
    /// Bostrom address for settlement.
    pub bostrom_address: String,
    /// Unix timestamp (seconds).
    pub timestamp: u64,
    /// Raw signature bytes (Ed25519).
    pub signature: Vec<u8>,
}

impl RewardClaim {
    /// Generate a hash of the crate metadata for binding.
    pub fn generate_crate_hash(
        crate_name: &str,
        version: &str,
        content_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(crate_name.as_bytes());
        hasher.update(version.as_bytes());
        hasher.update(content_hash.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify the claim against the authorized DID, Bostrom address, and ALN registry.
    pub fn verify(
        &self,
        public_key: &PublicKey,
    ) -> Result<(), RewardError> {
        // 1. Verify DID match
        if self.author_did != AUTHORIZED_DID {
            return Err(RewardError::UnauthorizedDID);
        }

        // 2. Verify Bostrom address match
        if self.bostrom_address != AUTHORIZED_BOSTROM {
            return Err(RewardError::UnauthorizedAddress);
        }

        // 3. Verify cryptographic signature
        let message = format!(
            "{}:{}:{}:{}",
            self.crate_name, self.crate_hash, self.author_did, self.timestamp
        );
        let sig = Signature::from_bytes(&self.signature)
            .map_err(|_| RewardError::InvalidSignature)?;
        public_key
            .verify(message.as_bytes(), &sig)
            .map_err(|_| RewardError::InvalidSignature)?;

        // 4. Verify against ALN particle registry
        if !verification::check_aln_authority(&self.crate_hash) {
            return Err(RewardError::ALNVerificationFailed);
        }

        Ok(())
    }
}

/// Error types for reward verification.
#[derive(Debug, Clone)]
pub enum RewardError {
    /// DID mismatch with AUTHORIZED_DID.
    UnauthorizedDID,
    /// Bostrom address mismatch.
    UnauthorizedAddress,
    /// Signature could not be verified.
    InvalidSignature,
    /// ALN particle authority check failed.
    ALNVerificationFailed,
    /// Crate content hash mismatch.
    CrateHashMismatch,
}

impl std::fmt::Display for RewardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RewardError::UnauthorizedDID =>
                write!(f, "Claim DID does not match authorized author"),
            RewardError::UnauthorizedAddress =>
                write!(f, "Bostrom address mismatch"),
            RewardError::InvalidSignature =>
                write!(f, "Cryptographic signature invalid"),
            RewardError::ALNVerificationFailed =>
                write!(f, "ALN particle authority check failed"),
            RewardError::CrateHashMismatch =>
                write!(f, "Crate content hash mismatch"),
        }
    }
}

impl std::error::Error for RewardError {}

/// Calculate knowledge-factor for this reward claim.
/// F = α·V + β·R + γ·E + δ·N, clipped to [0, 1].
pub fn calculate_knowledge_factor(
    validation: f32,
    reuse: f32,
    ecological_impact: f32,
    novelty: f32,
) -> f32 {
    let alpha = 0.30;
    let beta = 0.25;
    let gamma = 0.30;
    let delta = 0.15;

    let factor = alpha * validation.min(1.0)
        + beta * reuse.min(1.0)
        + gamma * ecological_impact.min(1.0)
        + delta * novelty.min(1.0);

    factor.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_hash_generation() {
        let hash =
            RewardClaim::generate_crate_hash("test_crate", "1.0.0", "content");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_knowledge_factor_bounds() {
        let factor = calculate_knowledge_factor(1.0, 1.0, 1.0, 1.0);
        assert!((0.0..=1.0).contains(&factor));
    }
}
