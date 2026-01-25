// src/firmware/TrayNodeController.hpp
// Control shell that fuses biodegradation, toxicity, energy, and hydro power
// into a node-level eco score for biodegradable tray machinery.
// Hex-proof: 0xabcdeffedcba0123

#pragma once
#include "../core/TrayBiodegModel.hpp"
#include "../core/HydroExtrusionModel.hpp"

namespace eco_tray {

struct TrayNodeInputs {
    BiodegParams biodeg;
    double r_tox;                // dimensionless toxicity ratio
    double energy_kwh_per_cycle; // kWh per full production cycle
    HydroParams hydro;
    double nozzle_radius_m;      // for extrusion flow calculation
    double power_threshold_w;    // minimum hydro power to be considered valid
};

struct TrayNodeOutputs {
    double t90_days;
    double biodeg_score;
    double hydro_power_w;
    double hydro_score;
    double node_eco_score;
    double throughput_m3s;
};

inline TrayNodeOutputs evaluate_node(const TrayNodeInputs &in) {
    TrayNodeOutputs out{};
    out.t90_days = compute_t90(in.biodeg);
    out.biodeg_score = eco_impact_from_t90(out.t90_days);
    out.hydro_power_w = compute_hydro_power_w(in.hydro);
    out.hydro_score = hydro_eco_impact(out.hydro_power_w,
                                       in.power_threshold_w,
                                       in.hydro.micro_risk);
    out.throughput_m3s = extrusion_flow_m3s(in.hydro,
                                            in.nozzle_radius_m);

    // fuse with toxicity and energy
    const double tray_score = tray_eco_score(
        out.t90_days,
        in.r_tox,
        in.energy_kwh_per_cycle
    );

    // final node eco score: product of hydro & tray subsystems
    if (out.hydro_score <= 0.0) {
        out.node_eco_score = 0.0;
    } else {
        out.node_eco_score = 0.5 * tray_score + 0.5 * out.hydro_score;
    }
    return out;
}

} // namespace eco_tray
