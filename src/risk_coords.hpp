#pragma once
#include <array>
#include <string>
#include <unordered_map>

namespace ecosafety {

enum class VarId {
    HLR,
    PFAS,
    CEC,
    T90,
    MICRO_RESIDUE,
    CARBON_CYCLE,
    ENERGY_SPEC
};

struct CorridorBand {
    VarId       varid;
    double      safe;
    double      gold;
    double      hard;
    double      weight;      // w_j in V_t = Σ w_j r_j^2
    std::string units;
    std::string lyap_channel;
    bool        mandatory;
};

struct RiskCoord {
    VarId  varid;
    double rx; // normalized in [0, 1], supplied by Rust/ALN

    double clamped(double hard) const noexcept {
        if (!std::isfinite(rx)) return hard;
        if (rx < 0.0) return 0.0;
        if (rx > hard) return hard;
        return rx;
    }
};

using RiskVector = std::array<RiskCoord, 7>;

using CorridorTable = std::unordered_map<VarId, CorridorBand>;

} // namespace ecosafety
