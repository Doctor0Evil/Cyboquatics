#pragma once

struct FlowVacDeviceSpec {
    std::string device_id;
    FlowVacContext context_supported; // URBAN, COASTAL, or both

    // Power and hydraulics
    double P_nominal_kW;
    double Q_design_m3s;
    double v_intake_max_ms;

    // Materials (for embodied impact audits)
    double m_steel_kg;
    double m_poly_kg;
    double m_filter_kg;
    double L_cable_m;
    double L_pipe_m;

    // Marine/urban safety envelope
    double max_sound_pressure_dB;
    double max_em_field_uT;
};

struct FlowVacPlacementDecision {
    std::string site_id;
    std::string device_id;

    bool feasible_geom;
    bool feasible_hyd;
    bool feasible_eco;

    double delta_Kn;
    double delta_energy_kWh;
    double delta_ecoimpact;

    double bio_stress_peak;
};
