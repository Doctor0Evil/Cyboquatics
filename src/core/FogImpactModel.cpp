#include <stdexcept>

struct FogNodeState {
    double Cin_mg_L;
    double Cout_mg_L;
    double Q_L_min;
    double dt_s;
};

struct FogImpactConfig {
    double Cref_mg_L;
    double hazard_weight;
    double karma_per_kg;
};

struct FogImpactResult {
    double mass_avoided_kg;
    double node_impact_K;
};

FogImpactResult computeFogImpact(const FogNodeState& s, const FogImpactConfig& cfg) {
    if (cfg.Cref_mg_L <= 0.0) {
        throw std::invalid_argument("Cref_mg_L must be positive.");
    }
    if (s.dt_s <= 0.0 || s.Q_L_min < 0.0) {
        throw std::invalid_argument("Invalid timestep or flow.");
    }

    const double deltaC_mg_L = s.Cin_mg_L - s.Cout_mg_L;
    if (deltaC_mg_L <= 0.0) {
        return {0.0, 0.0};
    }

    const double Q_m3_s = (s.Q_L_min / 1000.0) / 60.0;
    const double Cin_kg_m3 = s.Cin_mg_L / 1.0e6;
    const double Cout_kg_m3 = s.Cout_mg_L / 1.0e6;
    const double deltaC_kg_m3 = Cin_kg_m3 - Cout_kg_m3;
    const double mass_avoided_kg = deltaC_kg_m3 * Q_m3_s * s.dt_s;

    const double risk_unit = deltaC_mg_L / cfg.Cref_mg_L;
    const double K_node = cfg.hazard_weight * risk_unit * mass_avoided_kg * cfg.karma_per_kg;

    return {mass_avoided_kg, K_node};
}
