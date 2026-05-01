// destination: EcoNetPhoenix/include/ceim_energy_mass.hpp
#pragma once
#include <vector>

namespace econet {

struct EnergyMassWindow {
    double mass_removed_kg;
    double energy_j;
    double energy_per_kg;
};

EnergyMassWindow compute_energy_mass_window(
    const std::vector<double>& cin_kg_m3,
    const std::vector<double>& cout_kg_m3,
    const std::vector<double>& q_m3_s,
    const std::vector<double>& power_w,
    double dt_s);

} // namespace econet
