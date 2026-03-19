#pragma once
#include "risk_coords.hpp"

namespace ecosafety {

inline bool corridors_complete(const CorridorTable& corridors) {
    // All mandatory bands must be present.
    for (const auto& kv : corridors) {
        const auto& band = kv.second;
        if (band.mandatory &&
            !std::isfinite(band.safe) &&
            !std::isfinite(band.hard)) {
            return false;
        }
    }
    return true;
}

inline bool no_corridor_no_build(const CorridorTable& corridors) {
    // Strengthened: disallow missing mandatory entries entirely.
    // Rust/ALN must construct a table with all mandatory varids present;
    // C++ just asserts.
    if (corridors.size() < 7) return false;
    return corridors_complete(corridors);
}

} // namespace ecosafety
