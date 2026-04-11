// sim-phoenix-trays/src/main.rs

#![forbid(unsafe_code)]
#![deny(warnings)]

use std::{error::Error, fs::File, io::Write, path::PathBuf};

use serde::Deserialize;

use cyboquatic_ecosafety_core::{CorridorBands, RiskCoord};
use cyboquatic_eco_kernel::{
    ant_safe_substrate_ok, EcoKernelConfig, EnergyCycle, MaterialKinetics, MaterialToxicology,
};

/// **Config** for Phoenix AntRecycling recipes. [file:21]
#[derive(Debug, Deserialize)]
struct PhoenixConfig {
    pub region: String,
    pub lat: f64,
    pub lon: f64,
    pub target_t90_max_days: f64,
    pub max_caloric_fraction: f64,
    pub recipes: Vec<RecipeConfig>,
}

#[derive(Debug, Deserialize)]
struct RecipeConfig {
    pub machine_id: String,
    pub location: String,
    pub material_mix: String,
    pub t90_days: f64,
    pub measured_t90_days: f64,
    pub eco_impact_score_hint: f64,
    pub waste_reduced_kg_per_cycle: f64,
    pub tox_risk_corridor: f64,
    pub energy_kwh_per_cycle: f64,
    pub caloric_fraction: f64,
    pub r_tox: f64,
    pub r_micro: f64,
    pub r_cec: f64,
}

#[derive(Debug)]
struct AntRecord {
    machine_id: String,
    location: String,
    lat: f64,
    lon: f64,
    material_mix: String,
    target_t90_days: f64,
    measured_t90_days: f64,
    iso14851_class: String,
    eco_impact_score: f64,
    waste_reduced_kg_per_cycle: f64,
    tox_risk_corridor: f64,
    energy_kwh_per_cycle: f64,
    caloric_fraction: f64,
    ant_safety_class: String,
    notes: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let config_path: PathBuf = args
        .next()
        .unwrap_or_else(|| "antrecycling-phoenix.json".to_string())
        .into();
    let out_path: PathBuf = args
        .next()
        .unwrap_or_else(|| "qpudatashards/particlesAntRecyclingBioPackPhoenix2026v1.csv".to_string())
        .into();

    let cfg_file = File::open(&config_path)?;
    let cfg: PhoenixConfig = serde_json::from_reader(cfg_file)?;

    let eco_cfg = EcoKernelConfig {
        carbon_corridor: CorridorBands {
            safe_min: -10.0,
            safe_max: 0.0,
            gold_min: 0.0,
            gold_max: 2.0,
            hard_min: -20.0,
            hard_max: 10.0,
        },
        t90_corridor: CorridorBands {
            safe_min: 0.0,
            safe_max: cfg.target_t90_max_days,
            gold_min: cfg.target_t90_max_days,
            gold_max: 180.0,
            hard_min: 0.0,
            hard_max: 180.0,
        },
        energy_corridor: CorridorBands {
            safe_min: 0.0,
            safe_max: 12.0,
            gold_min: 12.0,
            gold_max: 18.0,
            hard_min: 0.0,
            hard_max: 24.0,
        },
    };

    let mut records: Vec<AntRecord> = Vec::new();

    for r in &cfg.recipes {
        let kin = MaterialKinetics {
            t90_days: r.t90_days,
            residual_fraction: 0.1,
        };
        let tox = MaterialToxicology {
            r_tox: r.r_tox,
            r_micro: r.r_micro,
            r_cec: r.r_cec,
        };
        let energy = EnergyCycle {
            grid_kwh: r.energy_kwh_per_cycle,
            hydro_kwh: 0.0,
        };

        let r_t90 = eco_cfg.t90_corridor.normalize(kin.t90_days);
        let r_energy = eco_cfg.energy_corridor.normalize(energy.net_grid_equiv());
        let r_tox_coord = RiskCoord::new_clamped(tox.r_tox);

        // ISO14851 classification based on t90 and residuals. [file:21]
        let iso_class = if kin.t90_days <= cfg.target_t90_max_days && r_tox_coord.value() <= 0.10 {
            "Phoenix-ISO14851-StrongPass"
        } else if kin.t90_days <= 180.0 {
            "Phoenix-ISO14851-Pass"
        } else {
            "Phoenix-ISO14851-Fail"
        }
        .to_string();

        let eco_impact_score =
            (r_t90.value().max(r_energy.value()).max(tox.r_micro.max(tox.r_cec))).neg()
                + 1.0;

        let ant_ok = ant_safe_substrate_ok(
            &kin,
            &tox,
            cfg.max_caloric_fraction,
            r.caloric_fraction,
        );

        let ant_class = if ant_ok {
            "incidentalediblesafe".to_string()
        } else if r.caloric_fraction > cfg.max_caloric_fraction {
            "rejectedantbait".to_string()
        } else if tox.r_tox > 0.10 {
            "rejectedtoxicity".to_string()
        } else if kin.t90_days > cfg.target_t90_max_days {
            "rejectedslow".to_string()
        } else {
            "flagforreview".to_string()
        };

        let notes = format!(
            "Auto-scored from sim-phoenix-trays; r_t90={:.3}, r_energy={:.3}, r_tox={:.3}",
            r_t90.value(),
            r_energy.value(),
            r_tox_coord.value()
        );

        records.push(AntRecord {
            machine_id: r.machine_id.clone(),
            location: r.location.clone(),
            lat: cfg.lat,
            lon: cfg.lon,
            material_mix: r.material_mix.clone(),
            target_t90_days: cfg.target_t90_max_days,
            measured_t90_days: r.measured_t90_days,
            iso14851_class: iso_class,
            eco_impact_score,
            waste_reduced_kg_per_cycle: r.waste_reduced_kg_per_cycle,
            tox_risk_corridor: r.tox_risk_corridor,
            energy_kwh_per_cycle: r.energy_kwh_per_cycle,
            caloric_fraction: r.caloric_fraction,
            ant_safety_class: ant_class,
            notes,
        });
    }

    std::fs::create_dir_all(out_path.parent().unwrap_or_else(|| std::path::Path::new(".")))?;
    let mut out = File::create(&out_path)?;
    writeln!(
        out,
        "machineid,location,lat,lon,materialmix,targett90days,measuredt90days,iso14851class,ecoimpactscore,wastereducedkgpercycle,toxriskcorridor,energykwhpercycle,caloricfraction,antsafetyclass,notes"
    )?;

    for rec in &records {
        writeln!(
            out,
            "{},{},{:.5},{:.5},{},{:.1},{:.1},{},{:.3},{:.1},{:.2},{:.2},{:.2},{},\"{}\"",
            rec.machine_id,
            rec.location,
            rec.lat,
            rec.lon,
            rec.material_mix,
            rec.target_t90_days,
            rec.measured_t90_days,
            rec.iso14851_class,
            rec.eco_impact_score,
            rec.waste_reduced_kg_per_cycle,
            rec.tox_risk_corridor,
            rec.energy_kwh_per_cycle,
            rec.caloric_fraction,
            rec.ant_safety_class,
            rec.notes.replace('"', "'")
        )?;
    }

    Ok(())
}
