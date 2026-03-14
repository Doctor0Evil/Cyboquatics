// Filename: EcoNetHydroBioSAT/src/ScenarioKernel.hpp
#pragma once
#include <vector>
#include <string>

namespace ecosat {

struct CorridorBands {
    double safe;
    double gold;
    double hard;
};

struct CorridorRow {
    std::string varid;      // e.g. "rSAT", "rplume", "rPFAS"
    std::string units;      // e.g. "m/h", "°C", "ng/L"
    CorridorBands bands;    // safe/gold/hard
    double weight;          // w_j for later Vt, stored, not used in C++
    int    lyap_channel;    // channel index, for Rust use
    bool   mandatory;       // must be present for this node
};

struct NodeParams {
    std::string node_id;
    double x, y, z;
    // MODFLOW/ECO-SCAPE parameters from shards:
    double sat_k1;      // fast decay, 1/d
    double sat_k2;      // slow decay, 1/d
    double hlr0;        // baseline HLR, m/h
    double disp_pfas;   // dispersion coeff, m^2/s
    double vdarcy;      // Darcy velocity, m/s
    double porosity;    // -
    // Biodegradation parameters (ISO 14851 / OECD-like):
    double k_biodeg;    // 1/d
    double t90_target;  // d
};

struct Boundary {
    double head_in;
    double head_out;
    double temp_ambient;
    double tcrit_depth_m;   // Z_crit
};

struct ScenarioConfig {
    double t_start_s;
    double t_end_s;
    double dt_s_macro;  // coarse time step to call MODFLOW
    double dt_s_micro;  // inner ECO-SCAPE chem step
};

struct StateSnapshot {
    double t_s;
    double head;            // groundwater head at node
    double q_m3s;           // flow through reach
    double hlr_m_per_h;     // SAT loading
    double c_pfas_ngL;      // PFAS conc
    double c_nutrient_mgL;  // nutrient
    double temp_zcrit_C;    // T(Z_crit)
    double mass_loss_frac;  // biodegradation fraction
    double t90_est_d;       // estimated t90
};

struct ScenarioKernelInput {
    std::vector<NodeParams> nodes;
    Boundary boundary;
    ScenarioConfig cfg;
    // corridors: passed in for documentation/echo only;
    // C++ never applies gating logic.
    std::vector<CorridorRow> corridors;
};

struct ScenarioKernelOutput {
    std::vector<StateSnapshot> snapshots;
    // raw, unnormalized values; no r_j, no V_t here.
};

ScenarioKernelOutput run_scenario(const ScenarioKernelInput& in);

} // namespace ecosat
