#[derive(Clone, Copy)]
pub struct CeimConfig {
    pub dt_seconds: f64,
    pub hazard_weight_pfbs: f64,
    pub hazard_weight_ecoli: f64,
    pub hazard_weight_tp: f64,
    pub hazard_weight_tds: f64,
}

#[repr(C)]
pub struct QpuNodeRow {
    pub baseline_Cin: f64,
    pub baseline_Cout: f64,
    pub Q_cms: f64,
    pub cref: f64,
    pub contaminant_kind: u8, // 0 PFBS,1 EColi,2 TP,3 TDS
}

fn hazard_weight(kind: u8, cfg: &CeimConfig) -> f64 {
    match kind {
        0 => cfg.hazard_weight_pfbs,
        1 => cfg.hazard_weight_ecoli,
        2 => cfg.hazard_weight_tp,
        3 => cfg.hazard_weight_tds,
        _ => 1.0,
    }
}

pub fn ceim_kn_for_node(node: &QpuNodeRow, cfg: &CeimConfig, window_seconds: f64) -> f64 {
    let cin = node.baseline_Cin;
    let cout = node.baseline_Cout;
    let cref = node.cref;
    let q = node.Q_cms;
    let w = hazard_weight(node.contaminant_kind, cfg);

    let ratio = if cref > 0.0 { (cin - cout) / cref } else { 0.0 };
    let v = q * window_seconds;
    w * ratio * v
}
