#pragma once
#include <cmath>
#include <stdexcept>

struct FogNodeState {
    double Cin_mg_L;   // inlet FOG concentration
    double Cout_mg_L;  // outlet FOG concentration
    double Q_L_min;    // average flow
    double dt_s;       // timestep seconds
};

struct FogImpactConfig {
    double Cref_mg_L;      // reference FOG limit, e.g. 100 mg/L
    double hazard_weight;  // w_FOG
    double karma_per_kg;   // Karma units per kg FOG avoided
};

struct FogImpactResult {
    double mass_avoided_kg;
    double node_impact_K;
};

inline FogImpactResult computeFogImpact(
    const FogNodeState& s,
    const FogImpactConfig& cfg
) {
    if (cfg.Cref_mg_L <= 0.0) {
        throw std::invalid_argument("Cref_mg_L must be positive.");
    }
    if (s.dt_s <= 0.0 || s.Q_L_min < 0.0) {
        throw std::invalid_argument("Invalid timestep or flow.");
    }

    // Convert flow to m3/s and mg/L to kg/m3
    const double Q_m3_s = (s.Q_L_min / 1000.0) / 60.0;
    const double Cin_kg_m3  = s.Cin_mg_L  / 1.0e6;
    const double Cout_kg_m3 = s.Cout_mg_L / 1.0e6;

    const double deltaC_kg_m3 = Cin_kg_m3 - Cout_kg_m3;
    if (deltaC_kg_m3 <= 0.0) {
        return {0.0, 0.0};
    }

    // Mass avoided over dt: M = (Cin - Cout) * Q * dt
    const double mass_avoided_kg = deltaC_kg_m3 * Q_m3_s * s.dt_s;

    // CEIM-style normalized node impact
    const double risk_unit =
        (s.Cin_mg_L - s.Cout_mg_L) / cfg.Cref_mg_L;

    const double K_node =
        cfg.hazard_weight * risk_unit * mass_avoided_kg * cfg.karma_per_kg;

    return {mass_avoided_kg, K_node};
}
