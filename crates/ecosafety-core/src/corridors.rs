use crate::risk_coord::RiskId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Band {
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
}

impl Band {
    pub fn validate(&self) -> bool {
        self.safe <= self.gold && self.gold <= self.hard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorBands {
    pub risk_id: RiskId,
    pub band: Band,
}

#[derive(Debug, Error)]
pub enum CorridorError {
    #[error("no corridor for risk id: {0:?}")]
    Missing(RiskId),
    #[error("invalid band ordering for risk id: {0:?}")]
    Invalid(RiskId),
}

impl CorridorBands {
    pub fn new(risk_id: RiskId, band: Band) -> Result<Self, CorridorError> {
        if !band.validate() {
            return Err(CorridorError::Invalid(risk_id));
        }
        Ok(Self { risk_id, band })
    }

    /// Normalize a raw measurement x into [0,1] given min/max corridor.
    pub fn normalize(&self, x_center: f64, x_min: f64, x_max: f64, x: f64) -> f64 {
        if (x_max - x_min).abs() < f64::EPSILON {
            return 0.0;
        }
        let r = (x - x_center) / (x_max - x_min);
        r.clamp(0.0, 1.0)
    }
}
