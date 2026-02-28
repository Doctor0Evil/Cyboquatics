#![no_std]

pub mod types;
pub mod contracts;
pub mod nanoswarm;

/// Re‑export core ecosafety types so downstream crates only need this kernel.
pub use types::{CorridorBands, RiskCoord, Residual};
pub use contracts::{CorridorDecision, safestep};
pub use nanoswarm::{
    NanoswarmEnvInputs,
    NanoswarmControlIntent,
    NanoswarmSafetyKernel,
    DefaultNanoswarmSafetyKernel,
};
