// src/core/HydroExtrusionModel.hpp
// Hydrokinetic power + slurry extrusion throughput for tray lines
// Hex-proof: 0xf1e2d3c4b5a69788

#pragma once
#include <cmath>
#include <stdexcept>

namespace eco_tray {

struct HydroParams {
    double rho_water;   // kg/m^3, usually ~1000
    double swept_area;  // m^2, turbine projected area
    double flow_vel;    // m/s, canal velocity
    double cp;          // power coefficient [0, 0.59]
    double micro_risk;  // r_microplastics in [0,1]
};

inline double compute_hydro_power_w(const HydroParams &p) {
    if (p.rho_water <= 0.0 || p.swept_area <= 0.0 || p.flow_vel <= 0.0 || p.cp <= 0.0) {
        throw std::invalid_argument("Invalid hydro parameters.");
    }
    return 0.5 * p.rho_water * p.swept_area *
           std::pow(p.flow_vel, 3.0) * p.cp;
}

inline double extrusion_flow_m3s(const HydroParams &p, double radius_m) {
    if (radius_m <= 0.0) {
        throw std::invalid_argument("Radius must be positive.");
    }
    const double pi = 3.141592653589793;
    const double area = pi * radius_m * radius_m;
    const double factor = (1.0 - p.micro_risk);
    if (factor <= 0.0) return 0.0;
    return area * p.flow_vel * factor;
}

inline double hydro_eco_impact(double power_w,
                               double power_threshold_w,
                               double micro_risk) {
    // Require enough power and low microplastic risk
    if (power_threshold_w <= 0.0) {
        throw std::invalid_argument("power_threshold_w must be positive.");
    }
    if (micro_risk < 0.0 || micro_risk > 1.0) {
        return 0.0;
    }
    if (power_w < power_threshold_w) {
        return 0.0;
    }
    if (micro_risk >= 0.05) {
        return 0.0;
    }
    const double base = 0.90;
    const double span = 0.05;
    const double factor = 1.0 - (micro_risk / 0.05); // 0→0.05 → 1→0
    return base + span * std::max(0.0, factor);
}

} // namespace eco_tray
