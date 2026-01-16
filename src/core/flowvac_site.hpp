#pragma once
#include "flowvac_safety.hpp"

struct FlowVacSiteConfig {
    // Location and zoning
    std::string site_id;
    double lat_deg;
    double lon_deg;
    FlowVacContext context;   // URBAN vs COASTAL

    // Geometry
    double A_footprint_m2;
    double H_clear_m;
    double d_soil_m;
    double d_pipe_m;

    // Hydraulics
    double Q_min_m3s;
    double Q_max_m3s;
    double v_max_ms;
    double h_loss_max_m;
    bool   backflow_allowed;

    // Resource budgets
    double P_avail_kW;
    double E_daily_kWh;
    double crew_hours_per_month;
    int    maintenance_interval_days;

    // Bio-safety envelope (context-dependent)
    double exclusion_radius_m;
    int    sensitive_species_count;
    double noise_limit_dB;
    double em_limit_uT;
    double bio_stress_max;

    // CEIM/CPVM coupling
    double ecoimpact_score_baseline;
    double karma_per_unit;
};
