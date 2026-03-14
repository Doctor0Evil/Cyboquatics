// Filename: cyboquatic-substrate-kernel/src/lib.rs

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Medium {
    Compost,
    CanalSediment,
    AridSoil,
    SyntheticWastewater,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateRecipe {
    pub material_stack: String, // e.g. "70 bagasse 25 starch 5 mineral"
    pub medium: Medium,
    pub temperature_c: f64,
    pub ph: f64,
    pub target_t90_days: f64,
    pub k_day_inv: f64,  // effective decay constant
    pub caloric_fraction: f64, // starch+protein fraction, 0–1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationResult {
    pub modeled_t90_days: f64,
    pub r_t90: f64,   // 0–1 normalized corridor
}

pub fn t90_from_k(k_day_inv: f64) -> f64 {
    // t90 = ln(10) / k
    (10.0_f64.ln() / k_day_inv)
}

pub fn t90_corridor(t90_days: f64, target: f64, hard_max: f64) -> f64 {
    if t90_days <= target {
        0.0
    } else if t90_days >= hard_max {
        1.0
    } else {
        (t90_days - target) / (hard_max - target)
    }
}

pub fn degradation(recipe: &SubstrateRecipe, hard_max: f64) -> DegradationResult {
    let t90 = t90_from_k(recipe.k_day_inv);
    let r = t90_corridor(t90, recipe.target_t90_days, hard_max);
    DegradationResult {
        modeled_t90_days: t90,
        r_t90: r,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeachateBands {
    pub r_tox: f64,     // 0–1 normalized toxicity corridor
    pub r_micro: f64,   // 0–1 micro-residue risk
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateEcoScore {
    pub ecoimpact_score: f64,
    pub knowledge_factor: f64,
    pub risk_of_harm: f64,
}

pub fn eco_score(
    t90: &DegradationResult,
    leachate: &LeachateBands,
    caloric_fraction: f64,
) -> SubstrateEcoScore {
    // Hard safety envelopes
    let t90_ok = t90.modeled_t90_days <= 180.0;
    let tox_ok = leachate.r_tox <= 0.10;
    let micro_ok = leachate.r_micro <= 0.05;
    let caloric_ok = caloric_fraction <= 0.30;

    if !(t90_ok && tox_ok && micro_ok && caloric_ok) {
        return SubstrateEcoScore {
            ecoimpact_score: 0.0,
            knowledge_factor: 0.9,
            risk_of_harm: 0.9,
        };
    }

    // Faster t90, low toxicity, low micro-risk → higher eco-impact
    let t90_norm = (180.0 - t90.modeled_t90_days).max(0.0) / 120.0; // 60–180 → 0–1
    let eco = 0.4 * t90_norm + 0.3 * (1.0 - leachate.r_tox) + 0.3 * (1.0 - leachate.r_micro);

    SubstrateEcoScore {
        ecoimpact_score: eco.min(1.0),
        knowledge_factor: 0.94,
        risk_of_harm: 0.11,
    }
}

pub fn substrate_corridor_ok(score: &SubstrateEcoScore) -> bool {
    score.ecoimpact_score >= 0.9 && score.risk_of_harm <= 0.2
}
