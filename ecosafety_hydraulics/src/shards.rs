#[derive(Clone, Debug)]
pub struct CorridorsRow {
    pub varid: String,
    pub units: String,
    pub safe: f64,
    pub gold: f64,
    pub hard: f64,
    pub w: f64,
    pub lyapchannel: String,
    pub mandatory: bool,
    pub description: String,
} // Canonical corridors row; grammar already frozen.[file:26]

#[derive(Clone, Debug)]
pub struct HydraulicDecayFrameShardRow {
    // Topology
    pub node_id: String,
    pub region: String,
    pub upstream_ids: Vec<String>,
    pub downstream_ids: Vec<String>,

    // Geometry + hydraulics
    pub reach_length_m: f64,
    pub bed_slope: f64,
    pub q_m3s: f64,
    pub hlr_m_per_h: f64,

    // Decay / clogging parameters
    pub k_fast: f64,
    pub k_slow: f64,
    pub clog_index: f64,

    // Normalized risk coordinates
    pub r_hlr: f64,        // HLR corridor 0–1
    pub r_surcharge: f64,  // surcharge risk 0–1
    pub r_fouling: f64,    // fouling risk 0–1

    // Residual and scoring
    pub vt: f64,           // Lyapunov residual
    pub k_factor: f64,     // Knowledgefactor K
    pub e_factor: f64,     // Eco-impact E
    pub r_factor: f64,     // Risk-of-harm R

    // Governance
    pub lane: String,          // "RESEARCH" | "PRODUCTION"
    pub kerdeployable: bool,   // eligibility gate
    pub shard_id: String,
    pub evidencehex: String,
} // All fields are shard-only; no actuator handles. [file:21][file:26]

#[derive(Clone, Debug)]
pub struct ChannelMergeAccountingFrameShardRow {
    pub junction_id: String,
    pub region: String,

    // Topology for cascade reasoning
    pub inflow_ids: Vec<String>,   // upstream node_ids
    pub outflow_ids: Vec<String>,  // downstream node_ids

    // Mass/volume balance
    pub q_in_total_m3s: f64,
    pub q_out_total_m3s: f64,
    pub q_balance_err: f64,        // |ΣQ_in - ΣQ_out| / ΣQ_in
    pub load_in_kg_s: f64,         // Σ Q_in * C_in
    pub load_out_kg_s: f64,        // Σ Q_out * C_out
    pub load_balance_err: f64,     // |ΣQ C_in - ΣQ C_out| / ΣQ C_in

    // Mixed water quality and risks
    pub c_mix_mg_l: f64,
    pub r_cec: f64,
    pub r_sat: f64,
    pub r_plume: f64,

    // Residual and scoring
    pub vt: f64,
    pub k_factor: f64,
    pub e_factor: f64,
    pub r_factor: f64,

    pub lane: String,
    pub kerdeployable: bool,
    pub shard_id: String,
    pub evidencehex: String,
}
