// Canonical mass-load and eco-impact kernel for cyboquatic under-street sewage nodes.
// Eco-impact level: High (targets large upstream solids/FOG removal with bounded emissions).

#include <cmath>
#include <stdexcept>

namespace econet {

struct SewageSample {
    // Concentrations in mg/L, flow in L/s, timestep in s.
    double Cin_tss_mgL;   // inflow total suspended solids
    double Cout_tss_mgL;  // outflow total suspended solids
    double Cin_fog_mgL;   // inflow FOG proxy
    double Cout_fog_mgL;  // outflow FOG proxy
    double Q_Ls;          // flow rate
    double dt_s;          // timestep length
};

struct SewageNodeConfig {
    double Cref_tss_mgL;   // reference TSS limit (mg/L)
    double Cref_fog_mgL;   // reference FOG limit (mg/L)
    double w_tss;          // hazard weight for TSS
    double w_fog;          // hazard weight for FOG
    double karma_per_kg;   // Karma units per kg pollutant avoided
};

struct SewageNodeResult {
    double mass_tss_avoided_kg;
    double mass_fog_avoided_kg;
    double node_impact_K;      // dimensionless impact score
    double karma_units;        // Karma units for governance
};

inline double mgL_to_kg_per_m3(double mgL) {
    // 1 mg/L = 1e-6 kg/L = 1e-3 kg/m^3
    return mgL * 1.0e-3;
}

SewageNodeResult
accumulateSewageImpact(const SewageSample* samples,
                       std::size_t count,
                       const SewageNodeConfig& cfg)
{
    if (!samples || count == 0) {
        throw std::invalid_argument("samples must be non-null and count > 0");
    }
    if (cfg.Cref_tss_mgL <= 0.0 || cfg.Cref_fog_mgL <= 0.0) {
        throw std::invalid_argument("Reference concentrations must be positive.");
    }

    double total_mass_tss_kg = 0.0;
    double total_mass_fog_kg = 0.0;
    double numer_K = 0.0;
    double denom_K = 0.0;

    for (std::size_t i = 0; i < count; ++i) {
        const SewageSample& s = samples[i];
        if (s.Q_Ls <= 0.0 || s.dt_s <= 0.0) {
            continue; // skip invalid timesteps
        }

        // Flow in m^3/s
        double Q_m3s = s.Q_Ls / 1000.0;
        double volume_m3 = Q_m3s * s.dt_s;

        double dC_tss_mgL = s.Cin_tss_mgL - s.Cout_tss_mgL;
        double dC_fog_mgL = s.Cin_fog_mgL - s.Cout_fog_mgL;

        if (dC_tss_mgL < 0.0) dC_tss_mgL = 0.0;
        if (dC_fog_mgL < 0.0) dC_fog_mgL = 0.0;

        // Convert to kg/m^3 then to kg using volume.
        double dC_tss_kgm3 = mgL_to_kg_per_m3(dC_tss_mgL);
        double dC_fog_kgm3 = mgL_to_kg_per_m3(dC_fog_mgL);

        double mass_tss_kg = dC_tss_kgm3 * volume_m3;
        double mass_fog_kg = dC_fog_kgm3 * volume_m3;

        total_mass_tss_kg += mass_tss_kg;
        total_mass_fog_kg += mass_fog_kg;

        // Normalized risk units (dimensionless)
        double r_tss = dC_tss_mgL / cfg.Cref_tss_mgL;
        double r_fog = dC_fog_mgL / cfg.Cref_fog_mgL;

        // Weighted contribution to impact; mass-weighted averaging.
        double contrib = cfg.w_tss * r_tss * mass_tss_kg +
                         cfg.w_fog * r_fog * mass_fog_kg;
        numer_K += contrib;
        denom_K += (mass_tss_kg + mass_fog_kg);
    }

    SewageNodeResult result{};
    result.mass_tss_avoided_kg = total_mass_tss_kg;
    result.mass_fog_avoided_kg = total_mass_fog_kg;

    if (denom_K > 0.0) {
        result.node_impact_K = numer_K / denom_K;
    } else {
        result.node_impact_K = 0.0;
    }

    const double total_mass_kg = total_mass_tss_kg + total_mass_fog_kg;
    result.karma_units = total_mass_kg * cfg.karma_per_kg;
    return result;
}

} // namespace econet
