// bins/simphoenixtrays/src/main.rs

#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use cyboquatic_ecosafety_core::{KerWindow, Residual, ResidualWeights, RiskCoord, RiskVector};

#[derive(Clone, Debug)]
struct Recipe {
    id: String,
    location: String,
    lat: f32,
    lon: f32,
    bagasse_pct: f32,
    straw_pct: f32,
    cardboard_pct: f32,
    starch_pct: f32,
    protein_pct: f32,
    mineral_pct: f32,
    target_t90_days: f32,
    max_caloric_fraction: f32,
}

#[derive(Clone, Debug)]
struct SimParams {
    // Phoenix-style kinetic and corridor parameters.
    k_day: f32,
    y_yield: f32,
    d_decay: f32,
    t90_hard_max_days: f32,
    t90_gold_max_days: f32,
    rtox_gold_max: f32,
    microrisk_max: f32,
}

#[derive(Clone, Debug)]
struct SimResultRow {
    machine_id: String,
    location: String,
    lat: f32,
    lon: f32,
    material_mix: String,
    target_t90_days: f32,
    modeled_t90_days: f32,
    iso14851_class: String,
    ecoimpact_score: f32,
    waste_reduced_kg_per_cycle: f32,
    tox_risk_corridor: f32,
    energy_kwh_per_cycle: f32,
    caloric_fraction: f32,
    ant_safety_class: String,
    notes: String,
}

/// Very simplified Monod-like biodegradation surrogate.
fn simulate_t90_days(recipe: &Recipe, params: &SimParams) -> f32 {
    let base_t90 = 90.0; // from Phoenix literature for bagasse/starch blends.
    let fiber_frac = recipe.bagasse_pct + recipe.straw_pct + recipe.cardboard_pct;
    let starch_frac = recipe.starch_pct;
    let protein_frac = recipe.protein_pct;

    let density_factor = 1.0 + 0.3 * (fiber_frac / 100.0);
    let protein_factor = 1.0 + 0.2 * (protein_frac / 30.0);
    let starch_factor = 1.0 - 0.2 * (starch_frac / 40.0);

    let t90 = base_t90 * density_factor * protein_factor * starch_factor;
    t90.clamp(30.0, params.t90_hard_max_days * 1.5)
}

/// Map t90 → ecoimpactscore (higher is better, ≤180 days gold).
fn map_t90_to_ecoimpact(t90: f32, params: &SimParams) -> f32 {
    if t90 <= params.t90_gold_max_days {
        0.95
    } else if t90 <= params.t90_hard_max_days {
        0.90
    } else {
        0.75
    }
}

/// Simple toxicity corridor score (0..1).
fn estimate_tox_corridor(recipe: &Recipe, params: &SimParams) -> f32 {
    let protein_frac = recipe.protein_pct / 100.0;
    let base_rtox = 0.05 + 0.5 * protein_frac;
    (base_rtox / params.rtox_gold_max).clamp(0.0, 1.0)
}

/// Microrisk proxy (0..1) from fiber & processing.
fn estimate_micro_risk(recipe: &Recipe, params: &SimParams) -> f32 {
    let fiber_frac = (recipe.bagasse_pct + recipe.straw_pct + recipe.cardboard_pct) / 100.0;
    let micro = 0.02 + 0.4 * (1.0 - fiber_frac);
    (micro / params.microrisk_max).clamp(0.0, 1.0)
}

/// Caloric fraction from starch + protein.
fn estimate_caloric_fraction(recipe: &Recipe) -> f32 {
    (recipe.starch_pct + recipe.protein_pct) / 100.0
}

/// AntRecycling safety classification.
fn classify_ant_safety(
    t90: f32,
    rtox: f32,
    microrisk: f32,
    caloric_fraction: f32,
    recipe: &Recipe,
    params: &SimParams,
) -> (String, String) {
    if t90 > recipe.target_t90_days || t90 > params.t90_hard_max_days {
        return ("rejected_slow".to_string(), "Fails t90 corridor".to_string());
    }
    if rtox > params.rtox_gold_max {
        return ("rejected_toxic".to_string(), "Toxicity above gold band".to_string());
    }
    if microrisk > params.microrisk_max {
        return ("rejected_microrisk".to_string(), "Microrisk above corridor".to_string());
    }
    if caloric_fraction > recipe.max_caloric_fraction {
        return ("rejected_antbait".to_string(), "Caloric fraction too high".to_string());
    }
    let cls = if caloric_fraction <= recipe.max_caloric_fraction {
        "incidentalediblesafe"
    } else {
        "flagforreview"
    };
    (cls.to_string(), String::from("Passes AntRecycling corridors"))
}

/// Example: derive waste reduction and energy from recipe.
fn estimate_waste_and_energy(recipe: &Recipe) -> (f32, f32) {
    // Match Phoenix shard order-of-magnitude (220–320 kg/cycle, 9–14 kWh).
    let base_waste = 220.0;
    let mix_factor = 1.0
        + 0.2 * (recipe.bagasse_pct / 70.0)
        + 0.1 * (recipe.cardboard_pct / 30.0);
    let waste = base_waste * mix_factor;
    let energy = 9.0 + 2.0 * (waste / 320.0);
    (waste, energy)
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let config_path: PathBuf = args
        .next()
        .expect("config path required")
        .into();
    let out_path: PathBuf = args
        .next()
        .unwrap_or_else(|| "qpudatashards/particlesAntRecyclingBioPackPhoenix2026v1.csv".into())
        .into();

    let file = File::open(config_path)?;
    let reader = BufReader::new(file);

    let mut recipes = Vec::new();
    for line in reader.lines().skip(1) {
        let line = line?;
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 13 {
            continue;
        }
        let r = Recipe {
            id: cols[0].to_string(),
            location: cols[1].to_string(),
            lat: cols[2].parse()?,
            lon: cols[3].parse()?,
            bagasse_pct: cols[4].parse()?,
            straw_pct: cols[5].parse()?,
            cardboard_pct: cols[6].parse()?,
            starch_pct: cols[7].parse()?,
            protein_pct: cols[8].parse()?,
            mineral_pct: cols[9].parse()?,
            target_t90_days: cols[10].parse()?,
            max_caloric_fraction: cols[11].parse()?,
        };
        recipes.push(r);
    }

    let params = SimParams {
        k_day: 0.05,
        y_yield: 0.5,
        d_decay: 0.01,
        t90_hard_max_days: 180.0,
        t90_gold_max_days: 120.0,
        rtox_gold_max: 0.10,
        microrisk_max: 0.05,
    };

    let weights = ResidualWeights::normalized_default();
    let mut ker_window = KerWindow::new();
    let mut rows = Vec::new();

    for recipe in recipes.iter() {
        let t90 = simulate_t90_days(recipe, &params);
        let ecoimpact = map_t90_to_ecoimpact(t90, &params);
        let rtox_corridor = estimate_tox_corridor(recipe, &params);
        let microrisk = estimate_micro_risk(recipe, &params);
        let caloric_fraction = estimate_caloric_fraction(recipe);
        let (waste, energy) = estimate_waste_and_energy(recipe);
        let (ant_class, notes) = classify_ant_safety(
            t90,
            rtox_corridor,
            microrisk,
            caloric_fraction,
            recipe,
            &params,
        );

        if ant_class.starts_with("rejected_") {
            continue;
        }

        let rv = RiskVector {
            r_energy: RiskCoord::new_clamped(energy / 20.0),
            r_hydraulics: RiskCoord::new_clamped(0.1),
            r_biology: RiskCoord::new_clamped(rtox_corridor),
            r_carbon: RiskCoord::new_clamped(0.05),
            r_materials: RiskCoord::new_clamped(microrisk),
        };
        let residual = Residual::from_vector(&rv, &weights);
        ker_window.observe_step(cyboquatic_ecosafety_core::CorridorDecision::Ok, &rv);

        let modeled_t90 = t90;
        let iso_class = if modeled_t90 <= 90.0 {
            "Phoenix-ISO14851-StrongPass"
        } else if modeled_t90 <= 120.0 {
            "Phoenix-ISO14851-Pass"
        } else {
            "Phoenix-ISO14851-Marginal"
        };

        let material_mix = format!(
            "{} bagasse {} straw {} cardboard {} starch {} protein {} mineral",
            recipe.bagasse_pct as i32,
            recipe.straw_pct as i32,
            recipe.cardboard_pct as i32,
            recipe.starch_pct as i32,
            recipe.protein_pct as i32,
            recipe.mineral_pct as i32,
        );

        let row = SimResultRow {
            machine_id: recipe.id.clone(),
            location: recipe.location.clone(),
            lat: recipe.lat,
            lon: recipe.lon,
            material_mix,
            target_t90_days: recipe.target_t90_days,
            modeled_t90_days: modeled_t90,
            iso14851_class: iso_class.to_string(),
            ecoimpact_score: ecoimpact,
            waste_reduced_kg_per_cycle: waste,
            tox_risk_corridor: rtox_corridor,
            energy_kwh_per_cycle: energy,
            caloric_fraction,
            ant_safety_class: ant_class,
            notes,
        };
        rows.push(row);
    }

    let mut out = File::create(out_path)?;
    writeln!(
        out,
        "machineid,location,lat,lon,materialmix,targett90days,measuredt90days,iso14851class,\
         ecoimpactscore,wastereducedkgpercycle,toxriskcorridor,energykwhpercycle,caloricfraction,\
         antsafetyclass,notes"
    )?;

    for r in rows.iter() {
        writeln!(
            out,
            "{},{},{:.5},{:.5},\"{}\",{:.1},{:.1},{},{:.2},{:.1},{:.2},{:.1},{:.2},{},\"{}\"",
            r.machine_id,
            r.location,
            r.lat,
            r.lon,
            r.material_mix,
            r.target_t90_days,
            r.modeled_t90_days,
            r.iso14851_class,
            r.ecoimpact_score,
            r.waste_reduced_kg_per_cycle,
            r.tox_risk_corridor,
            r.energy_kwh_per_cycle,
            r.caloric_fraction,
            r.ant_safety_class,
            r.notes
        )?;
    }

    let triad = ker_window.triad();
    eprintln!(
        "KER window: K={:.2}, E={:.2}, R={:.2}",
        triad.k_knowledge, triad.e_eco_impact, triad.r_risk_of_harm
    );

    Ok(())
}
