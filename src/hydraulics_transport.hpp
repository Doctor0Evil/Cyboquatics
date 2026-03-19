#pragma once
#include <vector>
#include "risk_coords.hpp"

namespace ecosafety {

struct CellState {
    double q;       // flow [m3/s]
    double h;       // head [m]
    double c_pfas;  // PFAS [ng/L]
    double c_cec;   // CEC [ng/L]
    double t90;     // substrate decay [days]
};

using Reach = std::vector<CellState>;

struct HydroStepConfig {
    double dt_s;
    double dx_m;
};

void step_advection_reaction(Reach& reach,
                             const HydroStepConfig& cfg);

// Map raw fields into normalized rx using Rust/ALN‑supplied corridors.
RiskVector map_to_risk(const Reach& reach,
                       const CorridorTable& corridors);

} // namespace ecosafety
