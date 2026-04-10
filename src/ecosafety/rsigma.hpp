// File: src/ecosafety/rsigma.hpp
#pragma once
#include <algorithm>
#include <cmath>

namespace ecosafety {

struct RsigmaBands {
    Corridor1D drift;
    Corridor1D noise;
    Corridor1D bias;
    Corridor1D loss;
    double alpha_drift = 1.0;
    double alpha_noise = 1.0;
    double alpha_bias  = 1.0;
    double alpha_loss  = 0.5;
};

inline double normalize_scalar(double value, const Corridor1D& c) noexcept {
    if (value <= c.safe_max) return 0.0;
    if (value >= c.hard_min) return 1.0;
    if (value <= c.gold_max) {
        const double denom = (c.gold_max - c.safe_max);
        if (denom <= 0.0) return 0.0;
        return (value - c.safe_max) / denom * c.r_gold;
    }
    const double denom = (c.hard_min - c.gold_max);
    if (denom <= 0.0) return 1.0;
    const double frac = (value - c.gold_max) / denom;
    return c.r_gold + frac * (1.0 - c.r_gold);
}

struct RsigmaInputs {
    double drift_raw = 0.0;
    double noise_raw = 0.0;
    double bias_raw  = 0.0;
    double loss_raw  = 0.0;
};

inline double compute_rsigma(const RsigmaInputs& u, const RsigmaBands& bands) noexcept {
    const double r_drift = normalize_scalar(u.drift_raw, bands.drift);
    const double r_noise = normalize_scalar(u.noise_raw, bands.noise);
    const double r_bias  = normalize_scalar(u.bias_raw,  bands.bias);
    const double r_loss  = normalize_scalar(u.loss_raw,  bands.loss);

    const double val =
        bands.alpha_drift * r_drift * r_drift +
        bands.alpha_noise * r_noise * r_noise +
        bands.alpha_bias  * r_bias  * r_bias  +
        bands.alpha_loss  * r_loss  * r_loss;

    return std::min(1.0, std::sqrt(std::max(0.0, val)));
}

} // namespace ecosafety
