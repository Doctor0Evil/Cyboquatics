#![no_std]

#[derive(Clone, Copy)]
pub struct CorridorBands {
    pub varid: &'static str,
    pub units: &'static str,
    pub safe: f64,      // upper edge of safe band (0–1)
    pub gold: f64,      // upper edge of gold band (0–1)
    pub hard: f64,      // hard limit, must be <= 1.0
    pub weight: f64,    // w_j in V_t = Σ w_j r_j
    pub lyap_channel: u8,
    pub mandatory: bool,
}

#[derive(Clone, Copy)]
pub struct RiskCoord {
    pub r: f64,             // normalized 0–1
    pub sigma: f64,         // uncertainty
    pub bands: &'static CorridorBands,
}

#[derive(Clone, Copy)]
pub struct Residual {
    pub vt: f64,
    pub coords: &'static [RiskCoord],
}

impl Residual {
    pub fn recompute(&mut self) {
        let mut v = 0.0;
        for c in self.coords.iter() {
            v += c.r * c.bands.weight;
        }
        self.vt = v;
    }
}

/// Simple linear 0–1 normalization kernel reused across domains.
pub fn to_r_linear(x: f64, bands: &'static CorridorBands) -> RiskCoord {
    debug_assert!(bands.safe <= bands.hard);
    debug_assert!(bands.hard <= 1.0);

    let r = if x <= bands.safe {
        0.0
    } else if x >= bands.hard {
        1.0
    } else {
        (x - bands.safe) / (bands.hard - bands.safe)
    };

    RiskCoord {
        r,
        sigma: 0.0,
        bands,
    }
}
