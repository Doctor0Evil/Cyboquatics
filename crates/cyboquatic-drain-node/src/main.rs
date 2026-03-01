mod config;
mod controller;

use config::NodeConfig;
use controller::{DrainController, SensorSnapshot};
use ecosafety-core::{KerInputs, KerScore};
use qpudatashard-schema::DrainShard;
use std::fs::File;
use std::io::Write;

fn main() -> anyhow::Result<()> {
    let cfg = NodeConfig::default();
    let mut controller = DrainController::new(cfg);

    // Example: two timesteps of fake data.
    let s0 = SensorSnapshot {
        fog_index: 0.40,
        microplastics_index: 0.55,
        pfas_index: 0.60,
        blockage_risk: 0.50,
        deforestation_index: 0.80,
    };
    let s1 = SensorSnapshot {
        fog_index: 0.35,
        microplastics_index: 0.50,
        pfas_index: 0.55,
        blockage_risk: 0.45,
        deforestation_index: 0.75,
    };

    controller.step(&s0, &s1, 0.30, 0.28)?;

    let ker_inputs = KerInputs {
        num_external_studies: 25,
        num_pilots: 2,
        corridor_coverage: 0.85,
        impact_deforestation: 0.90,
        impact_pollutants: 0.88,
        impact_resilience: 0.89,
        residual_uncertainty: 0.20,
    };
    let ker_score = KerScore::from_inputs(&ker_inputs);

    let shard: DrainShard = controller.finalize_shard(ker_score, "did:example:phoenix-node-1");
    let json = serde_json::to_string_pretty(&shard)?;
    let mut f = File::create("drain_pilot_shard.json")?;
    f.write_all(json.as_bytes())?;

    Ok(())
}
