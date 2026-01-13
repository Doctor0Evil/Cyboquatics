#include <iostream>
#include "cybo_qpudata.h"
#include "cybo_cpvm.h"
#include "cybo_ceim.h"

int main() {
    using namespace cybo;

    std::vector<QpuNodeRow> nodes;
    if (!load_nodes_csv("qpudatashards/phoenix/CyboquaticsPhoenixWaterNodes2026v1.csv", nodes)) {
        std::cerr << "Failed to load nodes CSV\n";
        return 1;
    }

    CeimConfig cfg{
        .dt_seconds = 86400.0,
        .hazard_weight_pfbs = 1.0,
        .hazard_weight_ecoli = 1.5,
        .hazard_weight_tp = 1.2,
        .hazard_weight_tds = 0.8
    };

    auto results = ceim_integrate_window(nodes, cfg, 86400.0);

    CpvmState cpvm_in{0.0, 0.0};
    CpvmState cpvm_out{0.0, 0.0};

    for (const auto& n : nodes) {
        cpvm_step(&n, &cpvm_in, &cpvm_out);
        cpvm_in = cpvm_out;
    }

    std::cout << "Phoenix CEIM daily Kn estimates:\n";
    for (const auto& r : results) {
        std::cout << r.node_id << "," << r.contaminant
                  << ",Kn=" << r.Kn
                  << ",Kn_sigma=" << r.Kn_sigma << "\n";
    }
    std::cout << "CPVM final mass=" << cpvm_out.state_mass
              << " risk=" << cpvm_out.state_risk << "\n";

    return 0;
}
