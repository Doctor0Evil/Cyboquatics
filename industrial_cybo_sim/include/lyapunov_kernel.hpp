#pragma once

#include <array>
#include <cstddef>

namespace industrial_cybo_sim {

struct RiskVector {
    std::array<float, 5> r; // E, H, B, C, M in fixed order
};

struct Weights {
    std::array<float, 5> w;
};

struct ResidualState {
    float v_prev;
};

struct StepCheck {
    float v_next;
    bool  safestep_ok;
};

inline float compute_residual(const RiskVector& rv, const Weights& w) {
    float v = 0.0f;
    for (std::size_t i = 0; i < 5; ++i) {
        const float ri = rv.r[i];
        const float wi = w.w[i];
        v += wi * ri * ri;
    }
    return v;
}

/// Enforce V(t+1) <= V(t) outside safe interior; caller clamps
/// rx via corridor tables imported from Rust/ALN.
inline StepCheck check_step(
    const RiskVector& rv_next,
    const Weights& w,
    const ResidualState& st,
    float epsilon
) {
    const float v_next = compute_residual(rv_next, w);
    const bool ok = (v_next <= st.v_prev + epsilon);
    return StepCheck{ v_next, ok };
}

} // namespace industrial_cybo_sim
