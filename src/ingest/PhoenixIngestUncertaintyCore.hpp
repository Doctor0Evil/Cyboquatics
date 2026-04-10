// src/ingest/PhoenixIngestUncertaintyCore.hpp
// Eco-impact level: High – ties ingest quality and uncertainty into K/E/R and Lyapunov V_t.
// Geo stamp: Phoenix, AZ (approx. 33.45 N, -112.07 W)
// Hex-proof: 0xa1b2c3d4e5f67890

#pragma once

#include <cmath>
#include <stdexcept>
#include <algorithm>

namespace econet {

struct IngestFeatureVector {
    double missing_fraction;   // 0..1
    double spike_fraction;     // 0..1
    double clip_fraction;      // 0..1
    double reorder_fraction;   // 0..1
};

struct IngestWeights {
    double w_missing;  // m weight
    double w_spike;    // s weight
    double w_clip;     // c weight
    double w_reorder;  // u weight
};

inline double compute_ingest_error_scalar(
    const IngestFeatureVector& f,
    const IngestWeights& w
) {
    const double I_raw =
        w.w_missing * f.missing_fraction +
        w.w_spike   * f.spike_fraction   +
        w.w_clip    * f.clip_fraction    +
        w.w_reorder * f.reorder_fraction;

    return std::max(0.0, I_raw);
}

inline double normalize_ingest_error(
    double I_raw,
    double I_ref_min,
    double I_ref_max
) {
    if (I_ref_max <= I_ref_min) {
        throw std::invalid_argument("Invalid ingest reference band.");
    }
    const double x = (I_raw - I_ref_min) / (I_ref_max - I_ref_min);
    return std::clamp(x, 0.0, 1.0);
}

struct RsigmaComponents {
    double drift; // 0..1
    double noise; // 0..1
    double bias;  // 0..1
    double loss;  // 0..1
};

struct RsigmaWeights {
    double w_drift;
    double w_noise;
    double w_bias;
    double w_loss;
};

inline double compute_rsigma_composite(
    const RsigmaComponents& r,
    const RsigmaWeights& w
) {
    const double term_d = w.w_drift * r.drift * r.drift;
    const double term_n = w.w_noise * r.noise * r.noise;
    const double term_b = w.w_bias  * r.bias  * r.bias;
    const double term_l = w.w_loss  * r.loss  * r.loss;

    const double sum = term_d + term_n + term_b + term_l;
    return std::sqrt(std::max(0.0, sum));
}

enum class CorridorBand {
    SAFE,
    GOLD,
    MONITOR,
    HARD
};

inline CorridorBand band_for_coordinate(
    double r_value,
    double safe_max,
    double gold_max,
    double hard_max
) {
    if (r_value < 0.0) r_value = 0.0;
    if (r_value > 1.0) r_value = 1.0;

    if (r_value <= safe_max)  return CorridorBand::SAFE;
    if (r_value <= gold_max)  return CorridorBand::GOLD;
    if (r_value <= hard_max)  return CorridorBand::MONITOR;
    return CorridorBand::HARD;
}

struct TrustInputs {
    double Dt_sensor; // 0..1
    double r_calib;   // 0..1 (normalized ingest quality)
};

struct TrustOutputs {
    double D_combined;
    double K_adj;
    double E_adj;
};

inline double clamp01(double x) {
    return std::clamp(x, 0.0, 1.0);
}

inline TrustOutputs compute_trust_adjusted_scores(
    const TrustInputs& in,
    double K_raw,
    double E_raw
) {
    const double Dt = clamp01(in.Dt_sensor);
    const double rc = clamp01(in.r_calib);

    const double D_combined = Dt * (1.0 - rc);

    TrustOutputs out;
    out.D_combined = D_combined;
    out.K_adj = K_raw * D_combined;
    out.E_adj = E_raw * D_combined;
    return out;
}

struct ResidualWeights {
    double w_pfbs;
    double w_ecoli;
    double w_salinity;
    double w_other;   // aggregate of other physical coords
    double w_rcalib;
    double w_rsigma;
};

struct ResidualCoords {
    double r_pfbs;
    double r_ecoli;
    double r_salinity;
    double r_other;
    double r_calib;
    double r_sigma_comp;
};

inline double compute_lyapunov_residual(
    const ResidualCoords& r,
    const ResidualWeights& w
) {
    const double term_pfbs     = w.w_pfbs     * r.r_pfbs      * r.r_pfbs;
    const double term_ecoli    = w.w_ecoli    * r.r_ecoli     * r.r_ecoli;
    const double term_salinity = w.w_salinity * r.r_salinity  * r.r_salinity;
    const double term_other    = w.w_other    * r.r_other     * r.r_other;
    const double term_rcalib   = w.w_rcalib   * r.r_calib     * r.r_calib;
    const double term_rsigma   = w.w_rsigma   * r.r_sigma_comp* r.r_sigma_comp;

    const double sum = term_pfbs + term_ecoli + term_salinity +
                       term_other + term_rcalib + term_rsigma;

    return std::max(0.0, sum);
}

inline double compute_R_global(
    double R_rcalib,
    double R_rsigma,
    double R_other_max
) {
    return std::max({R_rcalib, R_rsigma, R_other_max});
}

} // namespace econet
