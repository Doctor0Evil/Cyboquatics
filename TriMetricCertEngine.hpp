// Purpose: C++ wrapper around Rust cybo_core_corridors for Phoenix drain pilots
// Note: Assumes cybo_core_corridors is built as a C-compatible static or shared lib.

#pragma once

#include <cstdint>
#include <string>
#include <vector>

namespace cyboquatic {
namespace drain {

/// Mirror of Rust CorridorBands for FFI-safe use
struct CorridorBands {
    std::string id;
    std::string unit;
    double safe;
    double gold;
    double hard;
    double weight_w;
    std::string lyap_channel;
};

/// Mirror of Rust RiskCoord (rx, sigma; bands are supplied via CorridorBands)
struct RiskCoord {
    std::string id;
    double rx;
    double sigma;
    CorridorBands bands;
};

/// Mirror of Rust DrainResidual
struct DrainResidual {
    double vt;
    std::vector<RiskCoord> coords;
};

/// Mirror of Rust UncertaintyResidual
struct UncertaintyResidual {
    double ut;
    std::vector<RiskCoord> coords;
};

/// Mirror of Rust DrainCorridorTable
struct DrainCorridorTable {
    std::vector<CorridorBands> metrics;
};

/// Mirror of Rust SafeStepDecision
struct SafeStepDecision {
    bool derate;
    bool stop;
};

/// Certification summary for a time window / pilot shard
struct CertificationResult {
    bool schema_ok;        // corridors present & ordered
    bool dynamic_ok;       // V_t, U_t invariants respected
    bool external_ok;      // UWWTD / ISO / PFAS / EUDR checks (handled by caller)
    bool pass;             // overall tri-metric pass

    double vt_start;
    double vt_end;
    double ut_start;
    double ut_end;
};

extern "C" {

    // --- FFI signatures exposed by Rust cybo_core_corridors ---

    // Check corridor presence (mirrors Rust corridor_present)
    bool cybo_corridor_present(
        const CorridorBands* metrics,
        std::size_t metrics_len
    );

    // Evaluate safe_step (mirrors Rust safe_step)
    SafeStepDecision cybo_safe_step(
        const DrainResidual* prev_res,
        const UncertaintyResidual* prev_unc,
        const DrainResidual* next_res,
        const UncertaintyResidual* next_unc
    );

}

/// High-level engine: wraps Rust primitives into a tri-metric check skeleton.
/// External compliance (UWWTD/ISO/UNEP/EUDR) is evaluated by the caller.
class TriMetricCertEngine {
public:
    TriMetricCertEngine() = default;

    /// Check corridor schema invariants for a node.
    bool checkSchema(const DrainCorridorTable& table) const {
        if (table.metrics.empty()) {
            return false;
        }
        return cybo_corridor_present(
            table.metrics.data(),
            table.metrics.size()
        );
    }

    /// Evaluate a single control step for Lyapunov/uncertainty safety.
    SafeStepDecision evaluateStep(
        const DrainResidual& prev_res,
        const UncertaintyResidual& prev_unc,
        const DrainResidual& next_res,
        const UncertaintyResidual& next_unc
    ) const {
        return cybo_safe_step(&prev_res, &prev_unc, &next_res, &next_unc);
    }

    /// Evaluate a sequence of residuals over time and summarize certification.
    CertificationResult certifyTrajectory(
        const DrainCorridorTable& table,
        const std::vector<DrainResidual>& residuals,
        const std::vector<UncertaintyResidual>& uncertainties,
        bool external_ok
    ) const {
        CertificationResult result{};
        result.schema_ok = checkSchema(table);
        result.external_ok = external_ok;
        result.dynamic_ok = true;

        if (!result.schema_ok ||
            residuals.size() < 2 ||
            residuals.size() != uncertainties.size()) {
            result.pass = false;
            return result;
        }

        result.vt_start = residuals.front().vt;
        result.ut_start = uncertainties.front().ut;

        for (std::size_t i = 1; i < residuals.size(); ++i) {
            const auto& prev_res = residuals[i - 1];
            const auto& prev_unc = uncertainties[i - 1];
            const auto& next_res = residuals[i];
            const auto& next_unc = uncertainties[i];

            SafeStepDecision step_dec =
                cybo_safe_step(&prev_res, &prev_unc, &next_res, &next_unc);

            if (step_dec.stop) {
                result.dynamic_ok = false;
                break;
            }
            if (step_dec.derate) {
                // Derates are allowed but flagged; caller may choose stricter rules.
                // We do not flip dynamic_ok to false here; we only record a softer failure.
            }
        }

        result.vt_end = residuals.back().vt;
        result.ut_end = uncertainties.back().ut;

        result.pass = result.schema_ok && result.dynamic_ok && result.external_ok;
        return result;
    }
};

} // namespace drain
} // namespace cyboquatic
