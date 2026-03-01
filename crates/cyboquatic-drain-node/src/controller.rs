use chrono::Utc;
use ecosafety-core::{
    ResidualState, ResidualUpdateError, RiskCoord, RiskId,
};
use qpudatashard-schema::{DrainShard, MetricSample, PilotSummary, ShardHeader};
use uuid::Uuid;

use crate::config::NodeConfig;

/// Placeholder for sensor readings; in production this would be replaced
/// by hardware IO / fieldbus integration.
#[derive(Debug, Clone)]
pub struct SensorSnapshot {
    pub fog_index: f64,
    pub microplastics_index: f64,
    pub pfas_index: f64,
    pub blockage_risk: f64,
    pub deforestation_index: f64,
}

pub struct DrainController {
    pub cfg: NodeConfig,
    pub residual: ResidualState,
    pub shard_samples: Vec<MetricSample>,
}

impl DrainController {
    pub fn new(cfg: NodeConfig) -> Self {
        let residual = ResidualState::new(cfg.risk_weights.clone());
        Self {
            cfg,
            residual,
            shard_samples: Vec::new(),
        }
    }

    fn snapshot_to_coords(&self, snap: &SensorSnapshot) -> Vec<RiskCoord> {
        vec![
            RiskCoord::new(RiskId::FOG, snap.fog_index),
            RiskCoord::new(RiskId::Microplastics, snap.microplastics_index),
            RiskCoord::new(RiskId::PFAS, snap.pfas_index),
            RiskCoord::new(RiskId::SewerBlockage, snap.blockage_risk),
            RiskCoord::new(RiskId::Deforestation, snap.deforestation_index),
        ]
    }

    /// Decide an abstract control action based on next-state feasibility.
    pub fn step(
        &mut self,
        current: &SensorSnapshot,
        candidate_next: &SensorSnapshot,
        u_curr: f64,
        u_next: f64,
    ) -> Result<(), ResidualUpdateError> {
        let coords_curr = self.snapshot_to_coords(current);
        let coords_next = self.snapshot_to_coords(candidate_next);

        self.residual.update_checked(
            &coords_curr,
            &coords_next,
            u_curr,
            u_next,
            self.cfg.safe_interior_eps,
        )?;

        let ts = Utc::now();
        self.shard_samples.push(MetricSample {
            timestamp: ts,
            risk_coords: coords_next,
            v_residual: self.residual.v,
            u_residual: self.residual.u,
        });

        Ok(())
    }

    pub fn finalize_shard(&self, ker_summary: ecosafety-core::KerScore, signer_did: &str) -> DrainShard {
        let header = ShardHeader {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            pilot_name: "Phoenix-Cyboquatic-Drain-Pilot".to_string(),
            location: self.cfg.location.clone(),
            version: "0.1.0".to_string(),
            signer_did: signer_did.to_string(),
        };
        let summary = PilotSummary {
            ker: ker_summary,
            notes: "Automatically generated shard from cyboquatic-drain-node.".to_string(),
        };
        DrainShard {
            header,
            samples: self.shard_samples.clone(),
            summary,
        }
    }
}
