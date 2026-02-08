#[derive(Clone, Debug)]
pub struct CorridorBands {
    pub var_id:       String,   // e.g. "r_SAT", "r_PFAS"
    pub units:        String,   // e.g. "m/d", "ng/L"
    pub safe:         f64,      // science-safe band edge (r = 0 at/inside)
    pub gold:         f64,      // preferred operating band
    pub hard:         f64,      // r = 1.0 hard limit
    pub weight_w:     f64,      // contribution to V_t
    pub lyap_channel: u16,      // residual channel index
    pub mandatory:    bool,     // true for hard-safety corridors
}

#[derive(Clone, Debug)]
pub struct RiskCoord {
    pub value: f64,            // r_x ∈ [0, 1]
    pub sigma: f64,            // uncertainty
    pub bands: CorridorBands,
}

#[derive(Clone, Debug)]
pub struct Residual {
    pub vt:      f64,          // Lyapunov-style residual
    pub coords:  Vec<RiskCoord>,
}

#[derive(Clone, Debug)]
pub struct CorridorDecision {
    pub derate: bool,
    pub stop:   bool,
    pub reason: String,
}
