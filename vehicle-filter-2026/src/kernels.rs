use crate::contracts::{CorridorBands, RiskCoord};

pub fn to_rx(measured: f64, bands: &CorridorBands) -> RiskCoord {
    let v = if measured <= bands.safe {
        0.0
    } else if measured >= bands.hard {
        1.0
    } else {
        (measured - bands.safe) / (bands.hard - bands.safe)
    };

    RiskCoord {
        value: v,
        sigma: 0.0,
        bands: bands.clone(),
    }
}

pub fn residual_from_coords(coords: Vec<RiskCoord>) -> crate::contracts::Residual {
    let vt = coords
        .iter()
        .map(|rc| rc.bands.weight_w * rc.value)
        .sum::<f64>();
    crate::contracts::Residual { vt, coords }
}
