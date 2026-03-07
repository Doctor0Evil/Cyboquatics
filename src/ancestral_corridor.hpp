#pragma once

// Hex-stamp for authorship anchoring (replace with real DID-linked value in CI):
// 0xa1b2c3d4e5f67890ec0c9a8b7766554433221100ffeeddcc

#include <string>
#include <vector>
#include <stdexcept>
#include <sstream>
#include <limits>
#include <cmath>

namespace econet {
namespace cyboquatic {

// -----------------------------
// Core corridor structures
// -----------------------------

// Normalized risk coordinate r_x in [0, 1], with full corridor metadata.
// This mirrors your universal rx + CorridorBands grammar and Lyapunov channels.[file:88][file:105]
struct RiskCoord {
    std::string var_id;      // e.g. "PFBS_ngL", "fish_habitat_stress", "rsoul"
    std::string units;       // e.g. "ng/L", "dimensionless"
    double safe;             // lower edge of gold/safe band (physical units)
    double gold;             // aspirational target (physical units) if applicable
    double hard;             // hard limit (physical units)
    double weight;           // contribution weight into V_t
    unsigned int lyap_channel; // residual channel index
    bool mandatory;          // true => no corridor, no build
    double value_raw;        // last raw measurement in physical units
    double r;                // normalized 0–1 coordinate, computed by normalize()
};

// A single ancestral corridor scalar bundle used in KER:
// - rspecies*: species-specific residual (aggregated over species corridors)
// - rsoul: soulsafety coordinate
// - ecoimpactscore_dignity: dignity-weighted eco-impact score.[file:105]
struct AncestralScalarBundle {
    double rspecies_star;           // 0–1, where 1 is maximum allowed species residual
    double rsoul;                   // 0–1, soulsafety coordinate
    double ecoimpactscore_dignity;  // 0–1, weighted eco-impact
};

// Residual V_t over a set of risk coordinates, Lyapunov-style.[file:104][file:88]
struct Residual {
    double vt;                      // current residual
    std::vector<RiskCoord> coords;  // all risk coordinates

    // Recompute V_t = sum_j w_j * r_j from coords.
    void recompute() {
        double v = 0.0;
        for (const auto& rc : coords) {
            v += rc.weight * rc.r;
        }
        vt = v;
    }
};

// Safety decision for a proposed step; higher-level controllers must honour this.[file:104][file:88]
struct CorridorDecision {
    bool derate;   // true => reduce load/scale back
    bool stop;     // true => immediate stop / kerdeployable=false
    std::string reason;

    static CorridorDecision ok() {
        return CorridorDecision{false, false, "within corridors"};
    }

    static CorridorDecision hard_violation(const std::string& msg) {
        return CorridorDecision{true, true, msg};
    }

    static CorridorDecision lyap_violation(const std::string& msg) {
        return CorridorDecision{true, false, msg};
    }
};

// -----------------------------
// Normalization kernel
// -----------------------------

// Piecewise-linear normalization to r in [0, 1] using safe/gold/hard bands.
// This is the single normalizemetric(x, CorridorBands) kernel you froze
// as a universal ecosafety primitive.[file:88][file:105]
inline double normalize_metric(double x, double safe, double hard) {
    if (hard <= safe) {
        throw std::runtime_error("Invalid corridor: hard <= safe");
    }
    if (x <= safe) {
        return 0.0; // fully within safe band
    }
    if (x >= hard) {
        return 1.0; // at or beyond hard limit
    }
    return (x - safe) / (hard - safe);
}

// Normalize all coords in place, using their value_raw and corridor bands.
inline void normalize_coords(std::vector<RiskCoord>& coords) {
    for (auto& rc : coords) {
        rc.r = normalize_metric(rc.value_raw, rc.safe, rc.hard);
    }
}

// -----------------------------
// Corridor presence gate
// -----------------------------

// corridor_present: "no corridor, no build" gate.[file:88][file:105]
// Throws if any mandatory corridor is missing or malformed.
inline void corridor_present(const std::vector<RiskCoord>& coords) {
    if (coords.empty()) {
        throw std::runtime_error("No risk coordinates defined: no corridor, no deployment");
    }

    for (const auto& rc : coords) {
        if (rc.mandatory) {
            if (!std::isfinite(rc.safe) || !std::isfinite(rc.hard)) {
                std::ostringstream oss;
                oss << "Mandatory corridor '" << rc.var_id << "' has non-finite bands";
                throw std::runtime_error(oss.str());
            }
            if (rc.hard <= rc.safe) {
                std::ostringstream oss;
                oss << "Mandatory corridor '" << rc.var_id << "' has invalid band ordering (hard <= safe)";
                throw std::runtime_error(oss.str());
            }
        }
    }
}

// -----------------------------
// Safestep invariant
// -----------------------------

// safestep: per-step invariant combining corridor and Lyapunov checks.[file:88][file:104]
// - any r >= 1.0 => derate + stop (hard corridor breach)
// - V_next > V_prev (outside safe interior) => derate, but not necessarily stop
inline CorridorDecision safestep(const Residual& prev, const Residual& next) {
    // Hard corridor violation: any coordinate at or beyond r=1.
    for (const auto& rc : next.coords) {
        if (rc.r >= 1.0) {
            std::ostringstream oss;
            oss << "Hard corridor limit exceeded for '" << rc.var_id
                << "' (r=" << rc.r << " >= 1)";
            return CorridorDecision::hard_violation(oss.str());
        }
    }

    // Lyapunov residual must not increase (with small epsilon for float noise).
    const double eps = 1e-9;
    if (next.vt > prev.vt + eps) {
        std::ostringstream oss;
        oss << "Lyapunov residual increased (V_prev=" << prev.vt
            << ", V_next=" << next.vt << ")";
        return CorridorDecision::lyap_violation(oss.str());
    }

    return CorridorDecision::ok();
}

// -----------------------------
// Ancestral bundle computation
// -----------------------------

// Utility: compute rspecies_star as max r over all species-related coords
// (e.g. var_id prefix "rspecies_" or tagged units).[file:105]
inline double compute_rspecies_star(const std::vector<RiskCoord>& coords) {
    double max_r = 0.0;
    for (const auto& rc : coords) {
        if (rc.var_id.rfind("rspecies_", 0) == 0) {
            if (rc.r > max_r) {
                max_r = rc.r;
            }
        }
    }
    return max_r;
}

// Utility: extract rsoul from a dedicated corridor (var_id == "rsoul").[file:105]
inline double compute_rsoul(const std::vector<RiskCoord>& coords) {
    for (const auto& rc : coords) {
        if (rc.var_id == "rsoul") {
            return rc.r;
        }
    }
    // If not explicitly present, treat as unknown/hard: r=1 (non-deployable).
    return 1.0;
}

// Utility: compute dignity-weighted ecoimpactscore from
// existing ecoimpactscore field and rsoul, etc.
// Here we use a simple pattern: ecoimpact_dignity = ecoimpact_raw * (1 - rsoul).[file:105]
inline double compute_ecoimpact_dignity(double ecoimpact_raw, double rsoul) {
    if (ecoimpact_raw < 0.0 || ecoimpact_raw > 1.0) {
        throw std::runtime_error("ecoimpact_raw must be in [0,1]");
    }
    if (rsoul < 0.0 || rsoul > 1.0) {
        throw std::runtime_error("rsoul must be in [0,1]");
    }
    return ecoimpact_raw * (1.0 - rsoul);
}

// Compute the ancestral scalar bundle from a fully normalized set of coords
// and a raw eco-impact score produced by the CEIM mass kernel.[file:104][file:105]
inline AncestralScalarBundle compute_ancestral_bundle(
    const std::vector<RiskCoord>& coords,
    double ecoimpact_raw
) {
    AncestralScalarBundle out{};
    out.rspecies_star = compute_rspecies_star(coords);
    out.rsoul = compute_rsoul(coords);
    out.ecoimpactscore_dignity = compute_ecoimpact_dignity(ecoimpact_raw, out.rsoul);
    return out;
}

// -----------------------------
// High-level helper for controllers
// -----------------------------

// Given a previous state residual, and a new set of raw corridor readings,
// this helper performs the entire pipeline:
// 1) corridor_present gate,
// 2) normalize coords,
// 3) recompute residual,
// 4) compute ancestral scalars,
// 5) run safestep invariant.[file:88][file:104][file:105]
struct AncestralStepResult {
    Residual residual_next;
    AncestralScalarBundle ancestral;
    CorridorDecision decision;
};

inline AncestralStepResult calculate_ethical_cost(
    const Residual& prev_residual,
    std::vector<RiskCoord> next_coords,
    double ecoimpact_raw
) {
    // 1) Check corridors.
    corridor_present(next_coords);

    // 2) Normalize.
    normalize_coords(next_coords);

    // 3) Residual.
    Residual next_residual;
    next_residual.coords = std::move(next_coords);
    next_residual.recompute();

    // 4) Ancestral scalars.
    AncestralScalarBundle ancestral = compute_ancestral_bundle(
        next_residual.coords, ecoimpact_raw
    );

    // 5) Safestep decision.
    CorridorDecision dec = safestep(prev_residual, next_residual);

    return AncestralStepResult{
        next_residual,
        ancestral,
        dec
    };
}

} // namespace cyboquatic
} // namespace econet
