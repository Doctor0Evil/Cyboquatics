// Filename: cyboquatic-shard-writer/src/main.rs

use std::fs::OpenOptions;
use std::io::{Write, BufWriter};
use serde::Serialize;
use chrono::Utc;

use cyboquatic_hydro_kernel::{HydroSite, hydro, EcoKernelInput, ecoscore};
use cyboquatic_substrate_kernel::{SubstrateRecipe, degradation, LeachateBands, eco_score};

#[derive(Debug, Serialize)]
struct CyboMachineShardRow {
    nodeid: String,
    assettype: String,
    region: String,
    lat: f64,
    lon: f64,
    materialstack: String,
    t90days: f64,
    rtaxcorridor: f64,
    ecoimpactscore: f64,
    wastereducedkgpercycle: f64,
    energykwhpercycle: f64,
    hydropowerkw: f64,
    caloricdensity: f64,
    knowledgefactor01: f64,
    ecoimpact01: f64,
    riskofharm01: f64,
    rxt9001: f64,
    rxrtox01: f64,
    rxrmicro01: f64,
    rxantcal01: f64,
    violationresidual: f64,
    hexstamp: String,
    notes: String,
}

fn write_header_if_empty(path: &str) -> std::io::Result<()> {
    if !std::path::Path::new(path).exists() {
        let mut f = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(
            f,
            "nodeid,assettype,region,lat,lon,materialstack,t90days,rtaxcorridor,ecoimpactscore,\
             wastereducedkgpercycle,energykwhpercycle,hydropowerkw,caloricdensity,\
             knowledgefactor01,ecoimpact01,riskofharm01,\
             rxt9001,rxrtox01,rxrmicro01,rxantcal01,violationresidual,hexstamp,notes"
        )?;
    }
    Ok(())
}

fn append_row(path: &str, row: &CyboMachineShardRow) -> std::io::Result<()> {
    let f = OpenOptions::new().create(true).append(true).open(path)?;
    let mut w = BufWriter::new(f);
    let csv = format!(
        "{},{},{},{:.5},{:.5},{},{:.1},{:.2},{:.2},{:.1},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{},\"{}\"",
        row.nodeid,
        row.assettype,
        row.region,
        row.lat,
        row.lon,
        row.materialstack,
        row.t90days,
        row.rtaxcorridor,
        row.ecoimpactscore,
        row.wastereducedkgpercycle,
        row.energykwhpercycle,
        row.hydropowerkw,
        row.caloricdensity,
        row.knowledgefactor01,
        row.ecoimpact01,
        row.riskofharm01,
        row.rxt9001,
        row.rxrtox01,
        row.rxrmicro01,
        row.rxantcal01,
        row.violationresidual,
        row.hexstamp,
        row.notes.replace('"', "'"),
    );
    writeln!(w, "{}", csv)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let shard_path = "qpudatashardsparticlesCyboquaticMachinesPhoenix2026v1.csv";
    write_header_if_empty(shard_path)?;

    // Example: one canal-powered tray line node
    let site = HydroSite {
        node_id: "CYB-PHX-TRAY-01".into(),
        region: "Central-AZ".into(),
        lat: 33.45,
        lon: -112.07,
        area_m2: 2.0,
        velocity_ms: 2.0,
        cp: 0.4,
    };
    let hydro_res = hydro::compute_hydropower(&site, 12.0);

    let eco_input = EcoKernelInput {
        hydropower_kw: hydro_res.hydropower_kw,
        grid_intensity_kgco2_per_kwh: 0.35,
        baseline_kwh_per_cycle: 50.0,
        cycles_per_year: 300.0,
        plastic_kg_avoided_per_cycle: 320.0,
    };
    let eco_out = ecoscore::eco_kernel(&eco_input);

    let recipe = SubstrateRecipe {
        material_stack: "70 bagasse 25 starch 5 mineral".into(),
        medium: cyboquatic_substrate_kernel::Medium::Compost,
        temperature_c: 55.0,
        ph: 7.8,
        target_t90_days: 90.0,
        k_day_inv: 0.05,
        caloric_fraction: 0.25,
    };
    let deg = degradation(&recipe, 180.0);
    let leachate = LeachateBands { r_tox: 0.08, r_micro: 0.03 };
    let sub_score = eco_score(&deg, &leachate, recipe.caloric_fraction);

    // KER triad and normalized coordinates
    let k = sub_score.knowledge_factor;
    let e = eco_out.ecoimpact_score.min(sub_score.ecoimpact_score);
    let r = sub_score.risk_of_harm;

    let rxt90 = deg.r_t90;
    let rxrtox = leachate.r_tox;
    let rxrmicro = leachate.r_micro;
    let rxant = (recipe.caloric_fraction / 0.30).min(1.0);
    let violation = rxt90.max(rxrtox).max(rxrmicro).max(rxant);

    // Only write if corridors are respected (research gate)
    if e >= 0.9 && r <= 0.2 && violation <= 1.0 {
        let hexstamp = format!(
            "0x{:x}",
            md5::compute(format!("{}-{}", site.node_id, Utc::now().timestamp()))
        );

        let row = CyboMachineShardRow {
            nodeid: site.node_id.clone(),
            assettype: "CanalTrayPlant".into(),
            region: site.region.clone(),
            lat: site.lat,
            lon: site.lon,
            materialstack: recipe.material_stack.clone(),
            t90days: deg.modeled_t90_days,
            rtaxcorridor: leachate.r_tox,
            ecoimpactscore: e,
            wastereducedkgpercycle: eco_input.plastic_kg_avoided_per_cycle,
            energykwhpercycle: eco_input.baseline_kwh_per_cycle,
            hydropowerkw: hydro_res.hydropower_kw,
            caloricdensity: recipe.caloric_fraction,
            knowledgefactor01: k,
            ecoimpact01: e,
            riskofharm01: r,
            rxt9001: rxt90,
            rxrtox01: rxrtox,
            rxrmicro01: rxrmicro,
            rxantcal01: rxant,
            violationresidual: violation,
            hexstamp,
            notes: "Canal-powered cyboquatic tray line, ant-safe substrate, carbon-negative energy balance.".into(),
        };

        append_row(shard_path, &row)?;
    }

    Ok(())
}
