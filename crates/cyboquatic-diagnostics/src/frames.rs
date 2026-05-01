// File: crates/cyboquatic-diagnostics/src/frames.rs

#![forbid(unsafe_code)]

use cyboquatic_ecosafety_core::{LyapunovWeights, Residual, RiskCoord, RiskVector};

pub type ReachId = u64;

/// Minimal state required per reach / node.
#[derive(Clone, Copy, Debug)]
pub struct ReachState {
    pub q_m3s: f64,
    pub head_m: f64,
    pub clog_index: f64,
    pub sat_risk: f64,
    pub temp_c: f64,
    pub bod_mg_l: f64,
    pub tss_mg_l: f64,
    pub n_mg_l: f64,
    pub p_mg_l: f64,
    pub cec_index: f64,
    pub pfas_ng_l: f64,
}

/// Update for a shard-like record (field_name, value).
#[derive(Clone, Debug)]
pub struct ShardUpdate {
    pub reach_id: ReachId,
    pub fields: Vec<(&'static str, f64)>,
    pub tags: Vec<String>,
}

pub trait Frame {
    fn name(&self) -> &'static str;
    fn evaluate(&self, reaches: &[ReachState]) -> Vec<ShardUpdate>;
}

/// Hydraulic decay and clogging diagnostics.
pub struct HydraulicDecayFrame;

impl Frame for HydraulicDecayFrame {
    fn name(&self) -> &'static str {
        "HydraulicDecayFrame"
    }

    fn evaluate(&self, reaches: &[ReachState]) -> Vec<ShardUpdate> {
        reaches
            .iter()
            .enumerate()
            .map(|(idx, r)| {
                let k0 = 1.0f64;
                let alpha = 0.5f64;
                let keff = k0 * (-alpha * r.clog_index.max(0.0)).exp();
                ShardUpdate {
                    reach_id: idx as u64,
                    fields: vec![
                        ("clog_index", r.clog_index),
                        ("decay_keff", keff),
                    ],
                    tags: vec![],
                }
            })
            .collect()
    }
}

/// Simple mixing diagnostics for BOD, nutrients, CEC / PFAS.
pub struct QualityMixingFrame;

impl Frame for QualityMixingFrame {
    fn name(&self) -> &'static str {
        "QualityMixingFrame"
    }

    fn evaluate(&self, reaches: &[ReachState]) -> Vec<ShardUpdate> {
        let total_q: f64 = reaches.iter().map(|r| r.q_m3s.max(0.0)).sum();
        if total_q <= 0.0 {
            return Vec::new();
        }
        let mix = |f: fn(&ReachState) -> f64| -> f64 {
            reaches
                .iter()
                .map(|r| r.q_m3s.max(0.0) * f(r))
                .sum::<f64>()
                / total_q
        };
        let bod_mix = mix(|r| r.bod_mg_l);
        let cec_mix = mix(|r| r.cec_index);
        let pfas_mix = mix(|r| r.pfas_ng_l);
        let r_sat = (bod_mix / 30.0).min(1.0);
        let r_cec = cec_mix.min(1.0);
        let r_pfas = (pfas_mix / 70.0).min(1.0);
        reaches
            .iter()
            .enumerate()
            .map(|(idx, _)| ShardUpdate {
                reach_id: idx as u64,
                fields: vec![
                    ("bod_mix_mg_l", bod_mix),
                    ("cec_mix_index", cec_mix),
                    ("pfas_mix_ng_l", pfas_mix),
                    ("r_sat", r_sat),
                    ("r_cec", r_cec.max(r_pfas)),
                ],
                tags: vec![],
            })
            .collect()
    }
}

/// Lyapunov residual update based on r_sat and r_cec.
pub struct ResidualUpdateFrame {
    pub weights: LyapunovWeights,
}

impl Frame for ResidualUpdateFrame {
    fn name(&self) -> &'static str {
        "ResidualUpdateFrame"
    }

    fn evaluate(&self, reaches: &[ReachState]) -> Vec<ShardUpdate> {
        // Here we treat sat_risk as hydraulics and cec_index as biology/tox.
        reaches
            .iter()
            .enumerate()
            .map(|(idx, r)| {
                let rv = RiskVector {
                    r_energy: RiskCoord::new(0.0),
                    r_hydraulics: RiskCoord::new(r.sat_risk),
                    r_biology: RiskCoord::new(r.cec_index),
                    r_carbon: RiskCoord::new(0.0),
                    r_materials: RiskCoord::new(0.0),
                    r_biodiversity: RiskCoord::new(0.0),
                    r_sigma: RiskCoord::new(0.0),
                };
                let res: Residual = self.weights.evaluate(&rv);
                ShardUpdate {
                    reach_id: idx as u64,
                    fields: vec![
                        ("vt", res.vt),
                        ("r_hydraulics", rv.r_hydraulics.value),
                        ("r_biology", rv.r_biology.value),
                    ],
                    tags: vec![],
                }
            })
            .collect()
    }
}
