#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Shared K/E/R triad wrapper, aligned with existing ecosafety grammar.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KerTriad {
    /// Knowledge-factor in [0,1].
    pub k: f64,
    /// Eco-impact in [0,1].
    pub e: f64,
    /// Risk-of-harm in [0,1].
    pub r: f64,
}

impl KerTriad {
    pub fn validate(&self) -> Result<(), String> {
        for (name, v) in [("k", self.k), ("e", self.e), ("r", self.r)] {
            if !(0.0..=1.0).contains(&v) {
                return Err(format!("KerTriad: {} out of [0,1]: {}", name, v));
            }
        }
        Ok(())
    }
}

/// Immutable kernel authorship / design particle.
/// Maps to design.authorship.v1 shards.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DesignAuthorship {
    pub design_kernel_id: String,
    pub design_version: String,
    pub design_author_id: String,   // Bostrom / DID
    pub authorshiphex: String,      // hash over spec + signature
    pub corridor_spec_id: String,   // canonical PolicyCorridorSpec / corridors table
    pub domain: String,             // cyboquatic, cybocinder, biopack, etc.
    pub ker: KerTriad,              // design-side K/E/R
    pub ncorridors: u32,            // mandatory corridors in canonical spec
    pub n_verified_invariants: u32, // ALN/Rust proofs satisfied
    pub timestamp_utc: i64,
    pub design_notes: String,
}

impl DesignAuthorship {
    /// Minimal sanity checks; deeper checks are performed in CI/ALN.
    pub fn validate(&self) -> Result<(), String> {
        self.ker.validate()?;
        if self.ncorridors == 0 {
            return Err("DesignAuthorship: ncorridors must be >= 1".into());
        }
        if self.n_verified_invariants == 0 {
            return Err("DesignAuthorship: n_verified_invariants must be >= 1".into());
        }
        if self.design_kernel_id.is_empty() || self.design_author_id.is_empty() {
            return Err("DesignAuthorship: kernel_id and author_id must be non-empty".into());
        }
        Ok(())
    }
}

/// Deployment-side accountability particle.
/// Maps to deployment.accountability.v1 shards.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeploymentAccountability {
    // Identity and linkage
    pub node_id: String,
    pub node_type: String,
    pub region: String,
    pub design_kernel_id: String,
    pub design_authorship_id: String, // shard id / hash of DesignAuthorship
    pub operator_id: String,
    pub stakeholder_id: String,

    // Host/corridor instantiation
    pub corridor_instance_id: String,
    pub ncorridors_instance: u32,
    pub n_gate_predicates: u32,
    pub n_verified_invariants_runtime: u32,

    // Runtime K/E/R under actual host corridors
    pub ker_runtime: KerTriad,
    pub kerdeployable: bool,

    // Incident / audit trail
    pub incidents_count: u32,
    pub last_incident_hex: String,

    // Legal / jurisdiction envelope (e.g. Globe capsule)
    pub jurisdiction_capsule: String,

    pub timestamp_utc: i64,
}

impl DeploymentAccountability {
    /// Validate basic invariants; must be paired with a DesignAuthorship
    /// to enforce no-corridor-drop and no-invariant-weakening rules.
    pub fn validate_against_design(
        &self,
        design: &DesignAuthorship,
        max_r_threshold: f64,
    ) -> Result<(), String> {
        self.ker_runtime.validate()?;

        if self.design_kernel_id != design.design_kernel_id {
            return Err("DeploymentAccountability: kernel_id mismatch with design".into());
        }
        if self.ncorridors_instance < design.ncorridors {
            return Err(format!(
                "DeploymentAccountability: ncorridors_instance {} < design.ncorridors {}",
                self.ncorridors_instance, design.ncorridors
            ));
        }
        if self.n_verified_invariants_runtime < design.n_verified_invariants {
            return Err(format!(
                "DeploymentAccountability: n_verified_invariants_runtime {} < design.n_verified_invariants {}",
                self.n_verified_invariants_runtime, design.n_verified_invariants
            ));
        }

        // Policy: kerdeployable must be false if runtime R exceeds threshold.
        let should_be_deployable = self.ker_runtime.r <= max_r_threshold;
        if self.kerdeployable && !should_be_deployable {
            return Err(format!(
                "DeploymentAccountability: kerdeployable=true but runtime R={} exceeds threshold {}",
                self.ker_runtime.r, max_r_threshold
            ));
        }

        Ok(())
    }
}
