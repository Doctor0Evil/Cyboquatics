use crate::risk_coord::{RiskCoord, RiskId};
use crate::types::{LyapunovWeights, Residual, RiskVector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Residual state tracking Lyapunov residual V and uncertainty residual U.
///
/// V is the scalar Lyapunov residual V_t = Σ_j w_j r_j^2 over the active
/// risk coordinates. U is a separate uncertainty residual (e.g., aggregated
/// rsigma-style metric) governed by the same non-increase invariant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualState {
    pub v: f64,
    pub u: f64,
    /// Weights per risk dimension (by RiskId) used for V computation.
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

/// Compute V_t = Σ_j w_j r_j^2 over the canonical planes.
///
/// This is the Spine v1 residual for a full RiskVector; it can be used to
/// initialize or cross-check ResidualState.v when working at the plane level.
pub fn compute_residual(rv: &RiskVector, w: &LyapunovWeights) -> Residual {
    let mut vt = 0.0;
    vt += w.w_energy * rv.r_energy.value().powi(2);
    vt += w.w_hydraulics * rv.r_hydraulics.value().powi(2);
    vt += w.w_biology * rv.r_biology.value().powi(2);
    vt += w.w_carbon * rv.r_carbon.value().powi(2);
    vt += w.w_materials * rv.r_materials.value().powi(2);
    vt += w.w_biodiversity * rv.r_biodiversity.value().powi(2);
    vt += w.w_sigma * rv.r_sigma.value().powi(2);
    Residual::new(vt)
}

impl ResidualState {
    pub fn new(weights: HashMap<RiskId, f64>) -> Self {
        Self {
            v: 0.0,
            u: 0.0,
            weights,
        }
    }

    /// Compute V = Σ_j w_j r_j^2 for an arbitrary slice of RiskCoord,
    /// using the per-id weights stored in this ResidualState.
    pub fn compute_v(&self, coords: &[RiskCoord]) -> f64 {
        coords.iter().fold(0.0, |acc, rc| {
            let w = self.weights.get(&rc.id).copied().unwrap_or(1.0);
            acc + w * rc.value.powi(2)
        })
    }

    /// Update V and U with Lyapunov-style checks and hard-band guards.
    ///
    /// - Any coord in coords_next that violates its hard band causes HardViolation.
    /// - Outside the safe interior (v_curr > safe_interior_eps), require
    ///   V_next <= V_curr and U_next <= U_curr (within a small numerical tolerance).
    pub fn update_checked(
        &mut self,
        coords_curr: &[RiskCoord],
        coords_next: &[RiskCoord],
        u_curr: f64,
        u_next: f64,
        safe_interior_eps: f64,
    ) -> Result<(), ResidualUpdateError> {
        // Hard-band guard: any rc_next >= hard threshold fails.
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
                return Err(ResidualUpdateError::VIncreased {
                    curr: v_curr,
                    next: v_next,
                });
            }
            if u_next > u_curr + 1e-9 {
                return Err(ResidualUpdateError::UIncreased {
                    curr: u_curr,
                    next: u_next,
                });
            }
        }

        self.v = v_next;
        self.u = u_next;
        Ok(())
    }
}
