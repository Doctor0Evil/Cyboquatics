//! Conversion utilities to transform shards into ecosafety core types

use crate::industrial_node::CyboNodeShard;
use cyboquatic_ecosafety_core::{CyboRiskVector, CyboNodeType as CoreNodeType, Medium as CoreMedium};
use ecosafety_core::{LyapunovWeights, RiskCoord};

/// Trait for converting a shard into a risk vector
pub trait ToRiskVector {
    fn to_risk_vector(&self) -> CyboRiskVector;
}

/// Trait for converting a shard into Lyapunov weights
pub trait ToLyapunovWeights {
    fn to_lyapunov_weights(&self) -> LyapunovWeights;
}

/// Trait for converting a shard into residual computation input
pub trait ToResidualInput {
    fn to_risk_coords(&self) -> (RiskCoord, RiskCoord, RiskCoord, RiskCoord, RiskCoord);
}

impl ToRiskVector for CyboNodeShard {
    fn to_risk_vector(&self) -> CyboRiskVector {
        CyboRiskVector {
            r_energy: RiskCoord::new(self.renergy),
            r_hydraulics: RiskCoord::new(self.rhydraulics),
            r_biology: RiskCoord::new(self.rbiology),
            r_carbon: RiskCoord::new(self.rcarbon),
            r_materials: RiskCoord::new(self.rmaterials),
        }
    }
}

impl ToLyapunovWeights for CyboNodeShard {
    fn to_lyapunov_weights(&self) -> LyapunovWeights {
        LyapunovWeights {
            w_energy: self.wenergy,
            w_hydraulics: self.whydraulics,
            w_biology: self.wbiology,
            w_carbon: self.wcarbon,
            w_materials: self.wmaterials,
        }
    }
}

impl ToResidualInput for CyboNodeShard {
    fn to_risk_coords(&self) -> (RiskCoord, RiskCoord, RiskCoord, RiskCoord, RiskCoord) {
        (
            RiskCoord::new(self.renergy),
            RiskCoord::new(self.rhydraulics),
            RiskCoord::new(self.rbiology),
            RiskCoord::new(self.rcarbon),
            RiskCoord::new(self.rmaterials),
        )
    }
}

/// Helper to convert shard nodetype to core nodetype
pub fn shard_nodetype_to_core(nodetype: crate::industrial_node::CyboNodeType) -> CoreNodeType {
    match nodetype {
        crate::industrial_node::CyboNodeType::MarModule => CoreNodeType::MarModule,
        crate::industrial_node::CyboNodeType::FogDesiccator => CoreNodeType::FogDesiccator,
        crate::industrial_node::CyboNodeType::AirGlobe => CoreNodeType::AirGlobe,
        crate::industrial_node::CyboNodeType::Cain => CoreNodeType::Cain,
        crate::industrial_node::CyboNodeType::CanalPurifier => CoreNodeType::CanalPurifier,
        crate::industrial_node::CyboNodeType::Other => CoreNodeType::Other,
    }
}

/// Helper to convert shard medium to core medium
pub fn shard_medium_to_core(medium: crate::industrial_node::Medium) -> CoreMedium {
    match medium {
        crate::industrial_node::Medium::Water => CoreMedium::Water,
        crate::industrial_node::Medium::Air => CoreMedium::Air,
        crate::industrial_node::Medium::Fog => CoreMedium::Fog,
        crate::industrial_node::Medium::Mixed => CoreMedium::Mixed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::industrial_node::{CyboNodeType, Medium};

    #[test]
    fn test_shard_to_risk_vector() {
        let shard = CyboNodeShard {
            renergy: 0.3,
            rhydraulics: 0.4,
            rbiology: 0.5,
            rcarbon: 0.2,
            rmaterials: 0.1,
            ..Default::default()
        };

        let rv = shard.to_risk_vector();
        assert_eq!(rv.r_energy.value(), 0.3);
        assert_eq!(rv.r_hydraulics.value(), 0.4);
        assert_eq!(rv.r_biology.value(), 0.5);
        assert_eq!(rv.r_carbon.value(), 0.2);
        assert_eq!(rv.r_materials.value(), 0.1);
    }

    #[test]
    fn test_shard_to_lyapunov_weights() {
        let shard = CyboNodeShard {
            wenergy: 0.25,
            whydraulics: 0.25,
            wbiology: 0.25,
            wcarbon: 0.15,
            wmaterials: 0.10,
            ..Default::default()
        };

        let weights = shard.to_lyapunov_weights();
        assert_eq!(weights.w_energy, 0.25);
        assert_eq!(weights.w_hydraulics, 0.25);
        assert_eq!(weights.w_biology, 0.25);
        assert_eq!(weights.w_carbon, 0.15);
        assert_eq!(weights.w_materials, 0.10);
    }
}
