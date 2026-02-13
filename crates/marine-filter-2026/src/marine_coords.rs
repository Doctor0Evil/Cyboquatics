use crate::types::{CorridorBands, RiskCoord};

pub struct MarineInputs {
    pub salinity: (f64, f64),
    pub rcec: (f64, f64),
    pub rtox: (f64, f64),
    pub rpathogen: (f64, f64),
    pub rplume: (f64, f64),
    pub rbiodegspeed: (f64, f64),
    pub rmicroplastics: (f64, f64),
    pub routofband: (f64, f64),
}

/// Build the canonical marine risk coordinate vector in a fixed order.
pub fn build_marine_coords(inp: MarineInputs) -> Vec<RiskCoord> {
    vec![
        RiskCoord {
            value: inp.salinity.0,
            sigma: inp.salinity.1,
            bands: CorridorBands {
                var_id: "salinity".to_string(),
                safe: 1.0,
                gold: 0.8,
                hard: 1.0,
                weight_w: 0.5,
            },
        },
        RiskCoord {
            value: inp.rcec.0,
            sigma: inp.rcec.1,
            bands: CorridorBands {
                var_id: "rCEC".to_string(),
                safe: 0.3,
                gold: 0.1,
                hard: 1.0,
                weight_w: 1.0,
            },
        },
        RiskCoord {
            value: inp.rtox.0,
            sigma: inp.rtox.1,
            bands: CorridorBands {
                var_id: "rtox".to_string(),
                safe: 0.3,
                gold: 0.1,
                hard: 1.0,
                weight_w: 1.0,
            },
        },
        RiskCoord {
            value: inp.rpathogen.0,
            sigma: inp.rpathogen.1,
            bands: CorridorBands {
                var_id: "rpathogen".to_string(),
                safe: 0.1,
                gold: 0.05,
                hard: 1.0,
                weight_w: 0.8,
            },
        },
        RiskCoord {
            value: inp.rplume.0,
            sigma: inp.rplume.1,
            bands: CorridorBands {
                var_id: "rplume".to_string(),
                safe: 0.3,
                gold: 0.1,
                hard: 1.0,
                weight_w: 0.7,
            },
        },
        RiskCoord {
            value: inp.rbiodegspeed.0,
            sigma: inp.rbiodegspeed.1,
            bands: CorridorBands {
                var_id: "rbiodegspeed".to_string(),
                safe: 0.5,
                gold: 0.7,  // faster is better; corridor logic can invert as needed
                hard: 1.0,
                weight_w: 0.4,
            },
        },
        RiskCoord {
            value: inp.rmicroplastics.0,
            sigma: inp.rmicroplastics.1,
            bands: CorridorBands {
                var_id: "rmicroplastics".to_string(),
                safe: 0.2,
                gold: 0.05,
                hard: 1.0,
                weight_w: 0.8,
            },
        },
        RiskCoord {
            value: inp.routofband.0,
            sigma: inp.routofband.1,
            bands: CorridorBands {
                var_id: "routofband".to_string(),
                safe: 0.0,  // should stay at zero
                gold: 0.0,
                hard: 1.0,
                weight_w: 2.0,
            },
        },
    ]
}
