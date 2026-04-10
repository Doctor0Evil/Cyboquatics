// File: src/ecosafety/ingest_rcalib.hpp
#pragma once
#include <cstdint>
#include <cmath>

namespace ecosafety {

struct IngestErrorCounts {
    std::uint64_t n_missing   = 0; // missing required columns/fields
    std::uint64_t n_schema    = 0; // structural spec violations
    std::uint64_t n_corridor  = 0; // varids with no corridor
    std::uint64_t n_unit      = 0; // disallowed units
};

struct IngestErrorWeights {
    double m_missing  = 1.0; // highest hazard
    double m_schema   = 1.0;
    double m_corridor = 0.5; // medium
    double m_unit     = 0.5;
};

inline double compute_ingest_I(const IngestErrorCounts& c,
                               const IngestErrorWeights& w) noexcept {
    return w.m_missing  * static_cast<double>(c.n_missing) +
           w.m_schema   * static_cast<double>(c.n_schema)  +
           w.m_corridor * static_cast<double>(c.n_corridor)+
           w.m_unit     * static_cast<double>(c.n_unit);
}

struct Corridor1D {
    double safe_min  = 0.0;
    double safe_max  = 0.0;
    double gold_min  = 0.0;
    double gold_max  = 0.0;
    double hard_min  = 0.0;
    double hard_max  = 1.0;
    double r_gold    = 0.5; // risk at gold band
};

inline double normalize_rcalib(double I, const Corridor1D& c) noexcept {
    if (I <= c.safe_max) {
        return 0.0;
    }
    if (I >= c.hard_min) {
        return 1.0;
    }
    if (I <= c.gold_max) {
        const double denom = (c.gold_max - c.safe_max);
        if (denom <= 0.0) return std::min(1.0, std::max(0.0, I));
        return (I - c.safe_max) / denom * c.r_gold;
    }
    const double denom = (c.hard_min - c.gold_max);
    if (denom <= 0.0) return std::min(1.0, std::max(0.0, I));
    const double frac = (I - c.gold_max) / denom;
    return c.r_gold + frac * (1.0 - c.r_gold);
}

} // namespace ecosafety
