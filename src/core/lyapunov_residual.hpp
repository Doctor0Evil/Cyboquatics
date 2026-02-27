#ifndef ECOSAFETY_LYAPUNOV_RESIDUAL_HPP
#define ECOSAFETY_LYAPUNOV_RESIDUAL_HPP

#include <vector>
#include <stdexcept>
#include <cmath>

struct RiskCoordinate {
    double rx;      // normalized risk in [0, +inf)
    double w;       // hazard weight >= 0
    double rmin;    // corridor min (usually 0)
    double rmax;    // corridor max (usually 1)
};

struct ResidualState {
    double Vt;          // current Lyapunov residual
    bool insideInterior;
};

inline double compute_residual(const std::vector<RiskCoordinate>& coords) {
    double V = 0.0;
    for (const auto& c : coords) {
        if (c.w < 0.0) {
            throw std::invalid_argument("Negative weight not allowed");
        }
        V += c.w * c.rx * c.rx;
    }
    return V;
}

inline bool corridor_present(const std::vector<RiskCoordinate>& coords) {
    if (coords.empty()) return false;
    for (const auto& c : coords) {
        if (!(c.rmax > c.rmin)) return false;
    }
    return true;
}

inline bool safe_step(const ResidualState& prev,
                      const ResidualState& next,
                      const std::vector<RiskCoordinate>& coords)
{
    // Hard coordinate check: no corridor violation
    for (const auto& c : coords) {
        if (c.rx > c.rmax || c.rx < c.rmin) {
            return false;
        }
    }

    // Lyapunov non-expansive check outside interior
    if (!prev.insideInterior && !next.insideInterior) {
        if (next.Vt > prev.Vt + 1e-9) {
            return false;
        }
    }

    return true;
}

#endif // ECOSAFETY_LYAPUNOV_RESIDUAL_HPP
