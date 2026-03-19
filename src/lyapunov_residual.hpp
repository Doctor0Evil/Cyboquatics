#pragma once
#include "risk_coords.hpp"

namespace ecosafety {

struct ResidualState {
    double    vt;   // current Lyapunov residual
    RiskVector rx;  // normalized coordinates
};

inline double compute_vt(const ResidualState& s,
                         const CorridorTable& corridors) {
    double vt = 0.0;
    for (const auto& r : s.rx) {
        auto it = corridors.find(r.varid);
        if (it == corridors.end()) continue; // mandatory checked on Rust side
        const auto& band = it->second;
        const double v = r.clamped(band.hard);
        vt += band.weight * v * v;
    }
    return vt;
}

struct SafeStepDecision {
    bool accept;
    bool derate;
    bool stop;
};

struct SafeStepConfig {
    double vt_interior_max;
    double lyap_eps;
};

inline SafeStepDecision safestep(const SafeStepConfig& cfg,
                                 const ResidualState& prev,
                                 ResidualState& next,
                                 const CorridorTable& corridors) {
    // 1. Hard‑band breach: reject, derate, stop.
    for (const auto& r : next.rx) {
        auto it = corridors.find(r.varid);
        if (it == corridors.end()) continue;
        const auto& band = it->second;
        if (r.rx >= band.hard) {
            return {false, true, true};
        }
    }

    // 2. Lyapunov residual non‑increase outside interior.
    const double vt_prev = prev.vt;
    const double vt_next = compute_vt(next, corridors);
    next.vt = vt_next;

    if (vt_prev > cfg.vt_interior_max && vt_next > vt_prev + cfg.lyap_eps) {
        return {false, true, false};
    }
    return {true, false, false};
}

} // namespace ecosafety
