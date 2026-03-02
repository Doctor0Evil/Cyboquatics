use corridor_foundation::{CorridorDefinition, CorridorState, EnvironmentalCorridor, ResidualCorridor};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct CyboNodeCorridor {
    pub id: Uuid,
    pub defn: CorridorDefinition, // e.g., MetricFamily::OC for canal impact
}

impl EnvironmentalCorridor for CyboNodeCorridor {
    fn family(&self) -> corridor_foundation::MetricFamily {
        self.defn.family
    }

    fn definition(&self) -> CorridorDefinition {
        self.defn.clone()
    }

    fn is_violated(&self, current: CorridorState, prev: CorridorState) -> bool {
        let residual = ResidualCorridor { defn: self.defn.clone() };
        residual.is_violated(current, prev)
    }

    fn get_residual(&self, current: CorridorState) -> f64 {
        current.residual
    }

    fn escalate(&self, current: CorridorState) -> corridor_foundation::EscalationClass {
        let residual = ResidualCorridor { defn: self.defn.clone() };
        residual.escalate(current)
    }
}
