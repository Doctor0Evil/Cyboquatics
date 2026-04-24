//! ALN-backed shard types for Cyboquatic industrial ecosafety nodes.
//! 
//! This crate provides Rust structs that mirror the `CyboquaticIndustrialEcosafety2026v1.aln`
//! schema, ensuring compile-time alignment between telemetry shards and the ecosafety core.
//! 
//! ## Usage
//! 
//! ```rust
//! use cyboquatic_industrial_shards::CyboNodeShard;
//! use cyboquatic_ecosafety_core::{CyboRiskVector, LyapunovWeights};
//! 
//! let shard = CyboNodeShard::default();
//! let risk_vector = shard.to_risk_vector();
//! let weights = shard.to_lyapunov_weights();
//! ```

#![forbid(unsafe_code)]

pub mod industrial_node;
pub mod conversion;
pub mod validation;

pub use industrial_node::{
    CyboNodeType, Medium, Lane, SecurityResponseCap, FogRoutingMode,
    CyboNodeShard, ShardHeader,
};
pub use conversion::{ToRiskVector, ToLyapunovWeights, ToResidualInput};
pub use validation::{validate_admissibility, AdmissibilityError};
