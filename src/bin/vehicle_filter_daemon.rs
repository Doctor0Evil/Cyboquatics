use vehicle_filter_2026::contracts::{safestep, CorridorDecision, Residual};
use vehicle_filter_2026::normalize_exhaust::{normalize_exhaust, VehicleFilterBands};
use vehicle_filter_2026::hardware_api::ExhaustHardware;

fn control_tick<H: ExhaustHardware>(
    hw: &mut H,
    bands: &VehicleFilterBands,
    prev_residual: &Residual,
) -> (Residual, CorridorDecision) {
    let sensors = hw.read_sensors();
    let risk = normalize_exhaust(&sensors, bands);
    let decision = safestep(prev_residual, &risk.residual);

    if decision.stop {
        hw.command_bypass(true);
        hw.set_flow_rate(0.0);
    } else if decision.derate {
        hw.command_bypass(false);
        hw.set_flow_rate(0.3);
    } else {
        hw.command_bypass(false);
        hw.set_flow_rate(1.0);
    }

    (risk.residual, decision)
}

fn main() {
    // TODO: wire real hardware + shard I/O + DID signing.
}
