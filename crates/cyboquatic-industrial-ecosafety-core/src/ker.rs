//! KER triads and rolling windows for governance.

/// Knowledge-factor, Eco-impact, Risk-of-harm triad.
#[derive(Clone, Copy, Debug)]
pub struct KerTriad {
    pub k_knowledge: f32,   // fraction of Lyapunov-safe steps
    pub e_eco_impact: f32,  // 1 - max(rx) or eco-benefit
    pub r_risk_of_harm: f32 // max(rx) over window
}

/// Rolling-window KER, sized for embedded use.
///
/// Implementation is intentionally simple; heavy analytics can live
/// in higher layers or offloaded diagnostics.
#[derive(Debug)]
pub struct KerWindow<const N: usize> {
    idx: usize,
    len: usize,
    history: [KerTriad; N],
}

impl<const N: usize> KerWindow<N> {
    pub fn new(empty: KerTriad) -> Self {
        Self {
            idx: 0,
            len: 0,
            history: [empty; N],
        }
    }

    pub fn push(&mut self, triad: KerTriad) {
        self.history[self.idx] = triad;
        self.idx = (self.idx + 1) % N;
        if self.len < N {
            self.len += 1;
        }
    }

    pub fn aggregate(&self) -> KerTriad {
        if self.len == 0 {
            return KerTriad { k_knowledge: 0.0, e_eco_impact: 0.0, r_risk_of_harm: 0.0 };
        }
        let mut k_sum = 0.0;
        let mut e_sum = 0.0;
        let mut r_max = 0.0;
        for i in 0..self.len {
            let t = self.history[i];
            k_sum += t.k_knowledge;
            e_sum += t.e_eco_impact;
            if t.r_risk_of_harm > r_max {
                r_max = t.r_risk_of_harm;
            }
        }
        KerTriad {
            k_knowledge: k_sum / (self.len as f32),
            e_eco_impact: e_sum / (self.len as f32),
            r_risk_of_harm: r_max,
        }
    }
}
