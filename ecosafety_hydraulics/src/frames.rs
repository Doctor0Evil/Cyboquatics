#[derive(Clone, Debug)]
pub struct ReachState {
    pub node_id: String,
    pub region: String,
    pub q_m3s: f64,
    pub head_m: f64,
    pub hlr_m_per_h: f64,
    pub clog_index: f64,
    pub c_cec_mg_l: f64,
    pub temp_c: f64,
    pub upstream_ids: Vec<String>,
    pub downstream_ids: Vec<String>,
} // Raw physics state used by frames.[file:21]

#[derive(Clone, Debug)]
pub struct ShardUpdate {
    pub shard_type: String,      // "HydraulicDecayFrame" | "ChannelMergeAccountingFrame" | ...
    pub shard_id: String,
    pub fields: Vec<(String, f64)>,
    pub tags: Vec<String>,
} // Diagnostic-only; writer later maps into CSV qpudatashards.[file:21][file:26]

pub trait Frame {
    fn name(&self) -> &'static str;

    /// Pure function: read-only over state, no actuator access.
    fn evaluate(&self, reaches: &[ReachState]) -> Vec<ShardUpdate>;
} // Matches your existing nonactuating grammar.[file:21]

pub struct HydraulicDecayFrame;
pub struct ChannelMergeAccountingFrame;

impl Frame for HydraulicDecayFrame {
    fn name(&self) -> &'static str { "HydraulicDecayFrame" }

    fn evaluate(&self, reaches: &[ReachState]) -> Vec<ShardUpdate> {
        reaches
            .iter()
            .map(|r| {
                // Example: compute effective k_fast/k_slow, rsurcharge, rHLR, r_fouling
                // using frozen corridor tables and normalization kernels from ecosafety_core.[file:21][file:26]
                let k_fast = /* f(clog_index, hlr, ...) */;
                let k_slow = /* ... */;
                let r_hlr = /* normalize hlr */;
                let r_surcharge = /* rsurcharge(Q, HLR) */;
                let r_fouling = /* normalize clog_index */;
                let vt = /* compute residual Vt from r's */;

                ShardUpdate {
                    shard_type: "HydraulicDecayFrame".into(),
                    shard_id: format!("HDF-{}",
                        r.node_id),
                    fields: vec![
                        ("k_fast".into(), k_fast),
                        ("k_slow".into(), k_slow),
                        ("clog_index".into(), r.clog_index),
                        ("r_hlr".into(), r_hlr),
                        ("r_surcharge".into(), r_surcharge),
                        ("r_fouling".into(), r_fouling),
                        ("vt".into(), vt),
                    ],
                    tags: vec![],
                }
            })
            .collect()
    }
}

impl Frame for ChannelMergeAccountingFrame {
    fn name(&self) -> &'static str { "ChannelMergeAccountingFrame" }

    fn evaluate(&self, reaches: &[ReachState]) -> Vec<ShardUpdate> {
        // Group reaches into junctions based on topology metadata, then compute
        // ΣQ, ΣQ C, mass/volume balance errors, c_mix, r_cec, r_sat, r_plume, vt.[file:21][file:2]
        Vec::new() // skeleton
    }
}
