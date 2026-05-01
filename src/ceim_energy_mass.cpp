// destination: EcoNetPhoenix/src/ceim_energy_mass.cpp
#include "ceim_energy_mass.hpp"
#include <stdexcept>

namespace econet {

EnergyMassWindow compute_energy_mass_window(
    const std::vector<double>& cin_kg_m3,
    const std::vector<double>& cout_kg_m3,
    const std::vector<double>& q_m3_s,
    const std::vector<double>& power_w,
    double dt_s)
{
    const std::size_t n = cin_kg_m3.size();
    if (cout_kg_m3.size() != n || q_m3_s.size() != n || power_w.size() != n) {
        throw std::runtime_error("Telemetry vectors must have equal length");
    }
    if (dt_s <= 0.0) {
        throw std::runtime_error("dt_s must be positive");
    }

    double m_x_kg = 0.0;
    double e_j = 0.0;

    for (std::size_t i = 0; i < n; ++i) {
        const double delta_c = cin_kg_m3[i] - cout_kg_m3[i]; // kg/m^3
        const double q = q_m3_s[i];                          // m^3/s
        const double p = power_w[i];                         // J/s
        m_x_kg += delta_c * q * dt_s;                        // kg
        e_j    += p * dt_s;                                  // J
    }

    EnergyMassWindow result;
    result.mass_removed_kg = m_x_kg;
    result.energy_j        = e_j;
    if (m_x_kg > 0.0) {
        result.energy_per_kg = e_j / m_x_kg;                 // J/kg
    } else {
        result.energy_per_kg = 0.0;
    }
    return result;
}

} // namespace econet
