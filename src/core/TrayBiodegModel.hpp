// src/core/TrayBiodegModel.hpp
// Non-toxic biodegradable tray kinetics + eco_impact scoring
// Geo stamp: Phoenix AZ ~33.45N, 112.07W
// Hex-proof: 0xa1b2c3d4e5f67890

#pragma once
#include <cmath>
#include <stdexcept>

namespace eco_tray {

struct BiodegParams {
    double k;    // specific degradation rate (day^-1)
    double y;    // yield coefficient (dimensionless)
    double d;    // biomass death rate (day^-1)
    double s0;   // initial substrate mass (kg)
    double x0;   // initial biomass mass (kg)
};

struct BiodegPoint {
    double t_days;
    double s_kg;
    double x_kg;
};

inline BiodegPoint euler_step(const BiodegParams &p, double t,
                              double s, double x, double dt_days) {
    const double ds_dt = -p.k * s * x;
    const double dx_dt =  p.y * p.k * s * x - p.d * x;
    s += ds_dt * dt_days;
    x += dx_dt * dt_days;
    if (s < 0.0) s = 0.0;
    if (x < 0.0) x = 0.0;
    return {t + dt_days, s, x};
}

inline double compute_t90(const BiodegParams &p,
                          double dt_days = 0.25,
                          unsigned int max_steps = 2000) {
    if (p.s0 <= 0.0 || p.x0 <= 0.0 || dt_days <= 0.0) {
        throw std::invalid_argument("Invalid initial conditions for t90.");
    }
    const double threshold = 0.1 * p.s0;
    double t = 0.0;
    double s = p.s0;
    double x = p.x0;

    for (unsigned int i = 0; i < max_steps; ++i) {
        if (s <= threshold) {
            return t;
        }
        BiodegPoint next = euler_step(p, t, s, x, dt_days);
        t = next.t_days;
        s = next.s_kg;
        x = next.x_kg;
    }
    return std::numeric_limits<double>::infinity();
}

inline double eco_impact_from_t90(double t90_days) {
    // Invariant: >90 % degraded within 180 d → eco_impact ≥ 0.90
    if (!std::isfinite(t90_days) || t90_days <= 0.0) {
        return 0.0;
    }
    if (t90_days >= 180.0) {
        return 0.0;
    }
    // Map [0, 180) → (0.90, 0.98]; faster degradation → higher score
    const double base   = 0.90;
    const double span   = 0.08;
    const double factor = std::max(0.0, 1.0 - t90_days / 180.0);
    return base + span * factor;
}

inline double tox_corridor_score(double r_tox) {
    // r_tox = measured_leachate / hard_band (e.g. 1 ppm phthalate limit)
    // Corridor: r_tox <= 0.10 is gold; >0.30 is reject.
    if (r_tox < 0.0) return 0.0;
    if (r_tox > 1.0) return 0.0;
    if (r_tox <= 0.10) return 1.0;
    if (r_tox >= 0.30) return 0.0;
    const double slope = -1.0 / (0.30 - 0.10);
    return 1.0 + slope * (r_tox - 0.10);
}

inline double tray_eco_score(double t90_days,
                             double r_tox,
                             double energy_kwh_per_cycle,
                             double energy_ref_kwh = 20.0) {
    // Normalize energy use (hydrokinetic target: << grid baseline)
    if (energy_ref_kwh <= 0.0) {
        throw std::invalid_argument("energy_ref_kwh must be positive.");
    }
    const double e_norm = std::min(1.0, energy_kwh_per_cycle / energy_ref_kwh);
    const double e_score = 1.0 - e_norm; // lower energy → higher score

    const double b_score = eco_impact_from_t90(t90_days);
    const double tox_score = tox_corridor_score(r_tox);

    if (tox_score <= 0.0) {
        return 0.0; // hard gate: toxic blends forbidden
    }

    const double w_bio = 0.5;
    const double w_tox = 0.3;
    const double w_eng = 0.2;

    return w_bio * b_score + w_tox * tox_score + w_eng * e_score;
}

} // namespace eco_tray
