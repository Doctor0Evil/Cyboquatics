//! Cyboquatic Industrial Ecosafety Spine (Tier-1 grammar binding)
//! Rust-only, no_std-friendly core for industrial Cyboquatic machinery.
//!
//! This crate does NOT invent new safety primitives. It wraps the
//! universal rx/Vt/KER ecosafety grammar for the specific workload band:
//! MAR modules, FOG desiccators, canal purifiers, AirGlobe/CAIN, and
//! related Cyboquatic industrial nodes.

#![no_std]
#![forbid(unsafe_code)]

pub mod planes;
pub mod node;
pub mod ker;
pub mod controller;
pub mod decisions;
pub mod lane_gate;

pub use planes::{
    EnergyRisk, HydraulicsRisk, BiologyRisk, CarbonRisk, MaterialsRisk,
    IndustrialRiskVector,
};
pub use node::{NodeClass, MediumClass, Lane, NodeState, CommandEnvelope, NodeStateView};
pub use ker::{KerTriad, KerWindow};
pub use controller::{IndustrialSafeController, SafeStepKernel, NodeStateTrait, CommandEnvelopeTrait};
pub use decisions::{CorridorDecision, StepVerdict};
pub use lane_gate::{LaneGatedController, ShardNodeState, LaneGateError, shard_lane_to_core};

// Re-export core ecosafety types from the shared grammar
pub use ecosafety_core::{RiskCoord, RiskVector as BaseRiskVector, Residual, CorridorDecision as CoreCorridorDecision, LyapunovWeights};
