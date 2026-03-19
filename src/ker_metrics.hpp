#pragma once
#include <deque>
#include "risk_coords.hpp"
#include "lyapunov_residual.hpp"

namespace ecosafety {

struct KerWindow {
    std::deque<bool>  safe_steps;
    std::deque<double> max_r;
    std::deque<double> vt;
    std::size_t        window_size;
};

struct KerScores {
    double k_score; // fraction of Lyapunov‑safe steps
    double e_score; // eco‑impact complement
    double r_score; // max coordinate over window
};

inline KerScores update_ker(KerWindow& win,
                            const ResidualState& state,
                            bool lyap_safe) {
    const double current_max_r = [&]{
        double m = 0.0;
        for (const auto& r : state.rx) {
            if (r.rx > m) m = r.rx;
        }
        return m;
    }();

    win.safe_steps.push_back(lyap_safe);
    win.max_r.push_back(current_max_r);
    win.vt.push_back(state.vt);

    if (win.safe_steps.size() > win.window_size) {
        win.safe_steps.pop_front();
        win.max_r.pop_front();
        win.vt.pop_front();
    }

    const std::size_t n = win.safe_steps.size();
    std::size_t safe_count = 0;
    double max_r_window = 0.0;
    for (std::size_t i = 0; i < n; ++i) {
        if (win.safe_steps[i]) ++safe_count;
        if (win.max_r[i] > max_r_window) max_r_window = win.max_r[i];
    }

    KerScores out;
    out.k_score = n ? static_cast<double>(safe_count) / n : 0.0;
    out.r_score = max_r_window;
    out.e_score = 1.0 - out.r_score; // complement

    return out;
}

inline bool kerdeployable(const KerScores& s) {
    return (s.k_score >= 0.90) &&
           (s.e_score >= 0.90) &&
           (s.r_score <= 0.13);
}

} // namespace ecosafety
