use crate::types::CyboquaticPilotMetrics;

pub struct PilotCorridor {
    pub bod_mg_l_max: f64,
    pub tss_mg_l_max: f64,
    pub n_total_mg_l_max: f64,
    pub p_total_mg_l_max: f64,
    pub cec_index_max: f64,
    pub pfbs_index_max: f64,
    pub sat_hlr_mday_min: f64,
    pub sat_hlr_mday_max: f64,
    pub fouling_rate_rel_min: f64,
    pub fouling_rate_rel_max: f64,
    pub social_trust_min: f64,
    pub violation_residual_max: f64,
}

impl PilotCorridor {
    pub fn hydraulic_structural_ok(&self, m: &CyboquaticPilotMetrics) -> bool {
        m.sewer_surcharge_events == 0
            && m.backflow_incidents == 0
            && m.violation_residual <= self.violation_residual_max
    }

    pub fn treatment_sat_ok(&self, m: &CyboquaticPilotMetrics) -> bool {
        m.bod_mg_l <= self.bod_mg_l_max
            && m.tss_mg_l <= self.tss_mg_l_max
            && m.n_total_mg_l <= self.n_total_mg_l_max
            && m.p_total_mg_l <= self.p_total_mg_l_max
            && m.cec_index <= self.cec_index_max
            && m.pfbs_index <= self.pfbs_index_max
            && m.sat_hlr_mday >= self.sat_hlr_mday_min
            && m.sat_hlr_mday <= self.sat_hlr_mday_max
    }

    pub fn fouling_om_ok(&self, m: &CyboquaticPilotMetrics) -> bool {
        m.fouling_rate_rel >= self.fouling_rate_rel_min
            && m.fouling_rate_rel <= self.fouling_rate_rel_max
            && m.om_cost_rel <= 1.0
            && m.violation_residual <= self.violation_residual_max
    }

    pub fn social_governance_ok(&self, m: &CyboquaticPilotMetrics) -> bool {
        m.social_trust_score >= self.social_trust_min
            && m.dashboard_uptime >= 0.97
            && m.violation_residual <= self.violation_residual_max
    }

    /// Gate that must pass before any replication or scale-up.
    pub fn pilot_scale_up_ok(&self, m: &CyboquaticPilotMetrics) -> bool {
        self.hydraulic_structural_ok(m)
            && self.treatment_sat_ok(m)
            && self.fouling_om_ok(m)
            && self.social_governance_ok(m)
    }
}
