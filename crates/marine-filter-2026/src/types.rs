use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorBands {
    pub var_id: String,   // e.g. "salinity", "rCEC", "rtox", "rpathogen"
    pub safe: f64,        // upper bound of safe band
    pub gold: f64,        // upper bound of gold band (ideal)
    pub hard: f64,        // hard ceiling (normalized so rx < 1 == below hard)
    pub weight_w: f64,    // weight in residual V_t
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskCoord {
    pub value: f64,          // current measured/estimated value
    pub sigma: f64,          // measurement uncertainty
    pub bands: CorridorBands,
}

impl RiskCoord {
    pub fn rx(&self) -> f64 {
        if self.bands.hard <= 0.0 {
            return 0.0;
        }
        (self.value / self.bands.hard).max(0.0)
    }

    pub fn violates_hard(&self) -> bool {
        self.rx() >= 1.0
    }

    pub fn weighted_harm(&self) -> f64 {
        let x = self.rx();
        self.bands.weight_w * x * x // convex in rx
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Residual {
    pub vt: f64,                 // Lyapunov residual V_t
    pub coords: Vec<RiskCoord>,  // ordered set of risk coordinates
}
