
use crate::RiskCoord;

/// Net CO₂e per cycle, normalized to [0,1] with net-negative operation near 0
/// and net-positive emissions approaching 1.
pub fn carbon_coord(
    net_kg_co2e_per_cycle: f32,
    sequestration_ref: f32,
    emission_ref: f32,
) -> RiskCoord {
    if net_kg_co2e_per_cycle <= sequestration_ref {
        0.0
    } else if net_kg_co2e_per_cycle >= emission_ref {
        1.0
    } else {
        (net_kg_co2e_per_cycle - sequestration_ref) / (emission_ref - sequestration_ref)
    }
}
