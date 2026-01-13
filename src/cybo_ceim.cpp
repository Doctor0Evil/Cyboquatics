#include "cybo_ceim.h"
#include <cmath>

namespace cybo {

namespace {
double hazard_weight_for(const std::string& contaminant, const CeimConfig& cfg) {
    if (contaminant == "PFBS") return cfg.hazard_weight_pfbs;
    if (contaminant == "EColi") return cfg.hazard_weight_ecoli;
    if (contaminant == "TP") return cfg.hazard_weight_tp;
    if (contaminant == "TDS") return cfg.hazard_weight_tds;
    return 1.0;
}
}

std::vector<CeimResult> ceim_integrate_window(
    const std::vector<QpuNodeRow>& nodes,
    const CeimConfig& cfg,
    double window_seconds
) {
    std::vector<CeimResult> out;
    for (const auto& n : nodes) {
        double Cin  = n.baseline_Cin;
        double Cout = n.baseline_Cout;
        double Cref = n.cref;
        double Q    = n.Q_cms;
        double w    = hazard_weight_for(n.contaminant, cfg);

        double ratio = 0.0;
        if (Cref > 0.0) {
            ratio = (Cin - Cout) / Cref;
        }
        double V = Q * window_seconds;
        double Kn = w * ratio * V;

        CeimResult r;
        r.node_id     = n.node_id;
        r.contaminant = n.contaminant;
        r.Kn          = Kn;
        r.Kn_sigma    = std::sqrt(std::abs(Kn)) * 0.05;
        out.push_back(r);
    }
    return out;
}

} // namespace cybo
