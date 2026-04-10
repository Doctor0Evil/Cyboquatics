// File: src/ecosafety/lyapunov_ker.hpp
#pragma once
#include <vector>
#include <algorithm>

namespace ecosafety {

struct RiskVector {
    // physical coordinates r_j in [0,1]
    std::vector<double> r_physical;
    double r_calib = 0.0; // from normalize_rcalib
    double r_sigma = 0.0; // from compute_rsigma
};

struct LyapunovWeights {
    std::vector<double> w_physical;
    double w_calib = 0.0;
    double w_sigma = 0.0;
};

inline double vt_full(const RiskVector& r,
                      const LyapunovWeights& w) noexcept {
    const std::size_t n = std::min(r.r_physical.size(), w.w_physical.size());
    double V = 0.0;
    for (std::size_t i = 0; i < n; ++i) {
        const double ri = std::clamp(r.r_physical[i], 0.0, 1.0);
        V += w.w_physical[i] * ri * ri;
    }
    const double rc = std::clamp(r.r_calib, 0.0, 1.0);
    const double rs = std::clamp(r.r_sigma, 0.0, 1.0);
    V += w.w_calib * rc * rc;
    V += w.w_sigma * rs * rs;
    return V;
}

struct KerWindowState {
    double K_raw = 0.0;
    double R_raw = 0.0;
    double E_raw = 1.0;
    double K_adj = 0.0;
    double E_adj = 1.0;
    std::size_t samples = 0;
};

inline void kerwindow_update(KerWindowState& win,
                             bool safestep_ok,
                             const RiskVector& r,
                             double D_sensor,
                             double Vt,
                             double Vint) noexcept {
    ++win.samples;
    if (safestep_ok) {
        const double k_prev = win.K_raw;
        win.K_raw = k_prev + (1.0 - k_prev) / static_cast<double>(win.samples);
    }
    const double r_max_phys = r.r_physical.empty()
        ? 0.0
        : *std::max_element(r.r_physical.begin(), r.r_physical.end());
    const double r_max = std::max({r_max_phys, r.r_calib, r.r_sigma});
    win.R_raw = std::max(win.R_raw, r_max);
    win.E_raw = 1.0 - win.R_raw;

    const double D_data = 1.0 - std::clamp(r.r_calib, 0.0, 1.0);
    const double D_combined = std::clamp(D_sensor * D_data, 0.0, 1.0);
    win.K_adj = win.K_raw * D_combined;
    win.E_adj = win.E_raw * D_combined;

    (void)Vt;
    (void)Vint;
    // Vt and Vint are enforced at the controller level; here we just record K,E,R.
}

} // namespace ecosafety
