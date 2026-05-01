// destination: EcoNetPhoenix/src/risk_coords.cpp
#include "risk_coords.hpp"
#include <algorithm>
#include <stdexcept>

namespace econet {

double normalize_to_corridor(double value,
                             double safe_band,
                             double gold_band,
                             double hard_band)
{
    if (hard_band <= gold_band || gold_band <= safe_band) {
        throw std::runtime_error("Invalid corridor bands");
    }
    if (value <= safe_band) {
        return 0.0;
    }
    if (value >= hard_band) {
        return 1.0;
    }
    if (value <= gold_band) {
        return (value - safe_band) / (gold_band - safe_band);
    }
    return (value - gold_band) / (hard_band - gold_band);
}

double compute_node_impact(double w_x,
                           double c_sup_x,
                           const std::vector<double>& cin_kg_m3,
                           const std::vector<double>& cout_kg_m3,
                           const std::vector<double>& q_m3_s,
                           double dt_s)
{
    const std::size_t n = cin_kg_m3.size();
    if (cout_kg_m3.size() != n || q_m3_s.size() != n) {
        throw std::runtime_error("Telemetry vectors must have equal length");
    }
    if (c_sup_x <= 0.0) {
        throw std::runtime_error("c_sup_x must be positive");
    }
    if (dt_s <= 0.0) {
        throw std::runtime_error("dt_s must be positive");
    }

    double integral = 0.0;
    for (std::size_t i = 0; i < n; ++i) {
        const double delta_c = cin_kg_m3[i] - cout_kg_m3[i];
        const double q = q_m3_s[i];
        integral += (delta_c / c_sup_x) * q * dt_s;
    }
    return w_x * integral;
}

double compute_vt(const std::vector<double>& weights,
                  const std::vector<double>& coords)
{
    if (weights.size() != coords.size()) {
        throw std::runtime_error("weights and coords size mismatch");
    }
    double vt = 0.0;
    for (std::size_t i = 0; i < weights.size(); ++i) {
        vt += weights[i] * coords[i] * coords[i];
    }
    return vt;
}

bool safestep_nonincreasing(double vt_current,
                            double vt_next,
                            double interior_epsilon)
{
    if (interior_epsilon < 0.0) {
        throw std::runtime_error("interior_epsilon must be non-negative");
    }
    if (vt_current <= interior_epsilon) {
        return true;
    }
    return vt_next <= vt_current;
}

} // namespace econet
