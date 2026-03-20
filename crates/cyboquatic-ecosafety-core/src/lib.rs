#![forbid(unsafe_code)]

use std::time::Duration;

/// Dimensionless 0–1 risk coordinate (clamped).
#[derive(Clone, Copy, Debug)]
pub struct RiskCoord(f64);

impl RiskCoord {
    pub fn new(raw: f64) -> Self {
        Self(raw.clamp(0.0, 1.0))
    }
    pub fn value(self) -> f64 {
        self.0
    }
}

/// Corridor bands for one physical metric, with units and description in ALN shards.
#[derive(Clone, Debug)]
pub struct CorridorBands {
    pub safe_min: f64,
    pub safe_max: f64,
    pub gold_min: f64,
    pub gold_max: f64,
    pub hard_min: f64,
    pub hard_max: f64,
}

impl CorridorBands {
    /// Piecewise-linear normalization into 0–1 (safe→gold gentle, gold→hard steeper). [file:7]
    pub fn normalize(&self, x: f64) -> RiskCoord {
        let c = self;
        let r = if x <= c.safe_min {
            0.0
        } else if x <= c.gold_min {
            (x - c.safe_min) / (c.gold_min - c.safe_min + f64::EPSILON) * 0.25
        } else if x <= c.gold_max {
            0.25 + (x - c.gold_min) / (c.gold_max - c.gold_min + f64::EPSILON) * 0.25
        } else if x <= c.hard_max {
            0.5 + (x - c.gold_max) / (c.hard_max - c.gold_max + f64::EPSILON) * 0.5
        } else {
            1.0
        };
        RiskCoord::new(r)
    }

    /// Hard corridor check (no corridor, no build). [file:11][file:7]
    pub fn within_hard(&self, x: f64) -> bool {
        x >= self.hard_min && x <= self.hard_max
    }
}

/// Aggregated risk vector for one node / step (planes: energy, hydraulics, biology, carbon, materials). [file:3]
#[derive(Clone, Debug)]
pub struct RiskVector {
    pub r_energy: RiskCoord,
    pub r_hydraulics: RiskCoord,
    pub r_biology: RiskCoord,
    pub r_carbon: RiskCoord,
    pub r_materials: RiskCoord,
}

impl RiskVector {
    pub fn coords(&self) -> [RiskCoord; 5] {
        [
            self.r_energy,
            self.r_hydraulics,
            self.r_biology,
            self.r_carbon,
            self.r_materials,
        ]
    }

    pub fn max_coord(&self) -> RiskCoord {
        self.coords()
            .iter()
            .copied()
            .max_by(|a, b| a.value().partial_cmp(&b.value()).unwrap())
            .unwrap()
    }
}

/// Quadratic Lyapunov residual V_t = Σ w_j r_j^2. [file:3][file:7]
#[derive(Clone, Debug)]
pub struct ResidualState {
    pub vt: f64,
    pub weights: [f64; 5],
}

impl ResidualState {
    pub fn new(weights: [f64; 5]) -> Self {
        Self { vt: 0.0, weights }
    }

    pub fn update(&mut self, risks: &RiskVector) {
        let coords = risks.coords();
        let mut vt_new = 0.0;
        for (w, r) in self.weights.iter().zip(coords.iter()) {
            vt_new += w * r.value().powi(2);
        }
        self.vt = vt_new;
    }
}

/// Decision gate result for a proposed actuation step. [file:11]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorridorDecision {
    Ok,
    Derate,
    Stop,
}

/// Ecosafety kernel enforcing V_{t+1} ≤ V_t + ε and rx < 1. [file:3][file:11]
#[derive(Clone, Debug)]
pub struct EcoSafetyKernel {
    pub residual: ResidualState,
    pub eps_vt: f64,
}

impl EcoSafetyKernel {
    pub fn new(weights: [f64; 5], eps_vt: f64) -> Self {
        Self {
            residual: ResidualState::new(weights),
            eps_vt,
        }
    }

    pub fn safestep(
        &mut self,
        prev_vt: f64,
        risks: &RiskVector,
    ) -> CorridorDecision {
        self.residual.update(risks);
        let vt_new = self.residual.vt;
        let max_r = risks.max_coord().value();

        if max_r >= 1.0 {
            return CorridorDecision::Stop;
        }
        if vt_new > prev_vt + self.eps_vt {
            return CorridorDecision::Derate;
        }
        CorridorDecision::Ok
    }
}

/// Rolling-window KER metrics. [file:3][file:11]
#[derive(Clone, Debug)]
pub struct KerWindow {
    pub steps_total: u64,
    pub steps_lyap_safe: u64,
    pub r_max: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct KerTriad {
    pub k_knowledge: f64,
    pub e_eco_impact: f64,
    pub r_risk_of_harm: f64,
}

impl KerWindow {
    pub fn new() -> Self {
        Self {
            steps_total: 0,
            steps_lyap_safe: 0,
            r_max: 0.0,
        }
    }

    pub fn record_step(&mut self, lyap_safe: bool, risks: &RiskVector) {
        self.steps_total += 1;
        if lyap_safe {
            self.steps_lyap_safe += 1;
        }
        let r = risks.max_coord().value();
        if r > self.r_max {
            self.r_max = r;
        }
    }

    pub fn triad(&self) -> KerTriad {
        let k = if self.steps_total == 0 {
            1.0
        } else {
            self.steps_lyap_safe as f64 / self.steps_total as f64
        };
        let r = self.r_max;
        let e = (1.0 - r).clamp(0.0, 1.0);
        KerTriad {
            k_knowledge: k,
            e_eco_impact: e,
            r_risk_of_harm: r,
        }
    }
}

/// Type-level “no action without a risk estimate” controller contract. [file:3][file:7]
pub trait SafeController<S, A> {
    fn propose_step(&mut self, state: &S, dt: Duration) -> (A, RiskVector);
}

/// Thin adapter that only applies actions when ecosafety passes. [file:3][file:11]
pub fn guarded_step<S, A, C: SafeController<S, A>>(
    controller: &mut C,
    state: &mut S,
    kernel: &mut EcoSafetyKernel,
    ker_window: &mut KerWindow,
    dt: Duration,
    apply: impl Fn(&mut S, &A),
) -> CorridorDecision {
    let prev_vt = kernel.residual.vt;
    let (act, risks) = controller.propose_step(state, dt);
    let decision = kernel.safestep(prev_vt, &risks);
    let lyap_safe = matches!(decision, CorridorDecision::Ok);
    ker_window.record_step(lyap_safe, &risks);

    if decision == CorridorDecision::Ok {
        apply(state, &act);
    }
    decision
}
