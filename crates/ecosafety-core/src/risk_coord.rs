use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskId {
    FOG,
    BOD,
    COD,
    Nutrients,
    Microplastics,
    PFAS,
    Pathogens,
    Deforestation,
    SewerBlockage,
    EnergyUse,
    Custom(u16),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RiskCoord {
    pub id: RiskId,
    /// Normalized value in [0.0, 1.0].
    pub value: f64,
}

impl RiskCoord {
    pub fn new(id: RiskId, value: f64) -> Self {
        let clamped = value.clamp(0.0, 1.0);
        Self { id, value: clamped }
    }

    pub fn zero(id: RiskId) -> Self {
        Self { id, value: 0.0 }
    }

    pub fn is_hard_violation(&self) -> bool {
        self.value >= 1.0
    }
}
