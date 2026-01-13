#pragma once
#include <cstdint>
#include <vector>
#include <string>
#include "cybo_qpudata.h"

namespace cybo {

struct CeimConfig {
    double dt_seconds;
    double hazard_weight_pfbs;
    double hazard_weight_ecoli;
    double hazard_weight_tp;
    double hazard_weight_tds;
};

struct CeimResult {
    std::string node_id;
    std::string contaminant;
    double Kn;
    double Kn_sigma;
};

std::vector<CeimResult> ceim_integrate_window(
    const std::vector<QpuNodeRow>& nodes,
    const CeimConfig& cfg,
    double window_seconds
);

} // namespace cybo
