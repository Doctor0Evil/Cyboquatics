use crate::risk_coord::{RiskCoord, RiskId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualState {
    pub v: f64,
    pub u: f64,
    /// Weights per risk dimension.
    pub weights: HashMap<RiskId, f64>,
}

#[derive(Debug, Error)]
pub enum ResidualUpdateError {
    #[error("hard corridor violation for risk {0:?}")]
    HardViolation(RiskId),
    #[error("residual increased outside safe interior: v_next={next}, v_curr={curr}")]
    VIncreased { curr: f64, next: f64 },
    #[error("uncertainty residual increased outside safe interior: u_next={next}, u_curr={curr}")]
    UIncreased { curr: f64, next: f64 },
}

impl ResidualState {
    pub fn new(weights: HashMap<RiskId, f64>) -> Self {
        Self { v: 0.0, u: 0.0, weights }
    }

    pub fn compute_v(&self, coords: &[RiskCoord]) -> f64 {
        coords.iter().fold(0.0, |acc, rc| {
            let w = self.weights.get(&rc.id).copied().unwrap_or(1.0);
            acc + w * rc.value
        })
    }

    pub fn update_checked(
        &mut self,
        coords_curr: &[RiskCoord],
        coords_next: &[RiskCoord],
        u_curr: f64,
        u_next: f64,
        safe_interior_eps: f64,
    ) -> Result<(), ResidualUpdateError> {
        // Hard-band guard: any rc_next >= 1.0 fails.
        for rc in coords_next {
            if rc.is_hard_violation() {
                return Err(ResidualUpdateError::HardViolation(rc.id));
            }
        }

        let v_curr = self.compute_v(coords_curr);
        let v_next = self.compute_v(coords_next);

        // Outside safe interior, enforce V_next <= V_curr and U_next <= U_curr.
        if v_curr > safe_interior_eps {
            if v_next > v_curr + 1e-9 {
                return Err(ResidualUpdateError::VIncreased { curr: v_curr, next: v_next });
            }
            if u_next > u_curr + 1e-9 {
                return Err(ResidualUpdateError::UIncreased { curr: u_curr, next: u_next });
            }
        }

        self.v = v_next;
        self.u = u_next;
        Ok(())
    }
}
