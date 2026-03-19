#pragma once
#include "risk_coords.hpp"

namespace ecosafety {

struct CarbonCycleInput {
    double kg_co2e_per_cycle;
};

inline double normalize_carbon(const CarbonCycleInput& in,
                               const CorridorTable& corridors) {
    auto it = corridors.find(VarId::CARBON_CYCLE);
    if (it == corridors.end()) return 1.0;
    const auto& band = it->second;

    const double x = in.kg_co2e_per_cycle;
    // Piecewise linear map: safe < 0, gold ≈ 0, hard > 0
    if (x <= band.safe) return 0.0;     // net-negative
    if (x >= band.hard) return 1.0;     // unacceptable
    if (x <= band.gold) {
        // safe → gold mapped into [0, 0.5]
        const double t = (x - band.safe) / (band.gold - band.safe);
        return 0.5 * t;
    } else {
        // gold → hard mapped into [0.5, 1]
        const double t = (x - band.gold) / (band.hard - band.gold);
        return 0.5 + 0.5 * t;
    }
}

} // namespace ecosafety
