//! Cyboquatic Ecosafety Spine v1
//! Spec ID: Cyboquatic.Ecosafety.Spine.v1
//! Version: 1.0.0

pub mod risk_coord;
pub mod residual;
pub mod corridors;
pub mod ker_score;
pub mod types;
pub mod safestep;
pub mod ker;
pub mod traits;
pub mod riskvector;

pub use risk_coord::{RiskCoord, RiskId};
pub use residual::{ResidualState, ResidualUpdateError, compute_residual};
pub use corridors::{Band, CorridorBands, CorridorError};
pub use ker_score::{KerScore, KerInputs};
pub use types::{LyapunovWeights, Residual, RiskVector, CorridorDecision};
