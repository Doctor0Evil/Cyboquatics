use crate::normalize_exhaust::ExhaustSensors;

pub trait ExhaustHardware {
    fn read_sensors(&mut self) -> ExhaustSensors;
    fn command_bypass(&mut self, enabled: bool);
    fn set_flow_rate(&mut self, fraction: f64); // 0..1
}
