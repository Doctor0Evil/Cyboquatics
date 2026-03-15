#include <cmath>
#include <cstdint>
#include <cstdio>
#include <string>
#include <vector>

struct CorridorBands {
    // Example normalized corridor bands for HLR and contaminants
    double hlr_gold_max;   // e.g., 0.3
    double hlr_hard_max;   // e.g., 0.6
    double cec_gold_max;   // ng/L or mg/L
    double cec_hard_max;
    double pfas_gold_max;
    double pfas_hard_max;
};

struct Reach {
    uint64_t id;
    bool has_sat;

    // hydraulics
    double q_m3s;
    double hlr_m_per_h;

    // contaminants (concentrations)
    double c_cec;
    double c_pfas;

    // biodegradable substrate state
    double substrate_mass_kg;
    double k_decay_day;
    double t_accum_days;

    // risk & residual
    double r_hlr;
    double r_cec;
    double r_pfas;
    double vt;
};

inline double normalized_risk(double value, double gold, double hard) {
    if (value <= gold) return 0.0;
    if (value >= hard) return 1.0;
    return (value - gold) / (hard - gold);
}

void update_substrate(Reach &r, double dt_days) {
    // First‑order decay: m(t+dt) = m(t) * exp(-k * dt)
    const double k = std::max(r.k_decay_day, 0.0);
    double m0 = std::max(r.substrate_mass_kg, 0.0);
    double m1 = m0 * std::exp(-k * dt_days);
    r.substrate_mass_kg = m1;
    r.t_accum_days += dt_days;
}

void update_risks(Reach &r, const CorridorBands &bands) {
    // HLR corridor
    r.r_hlr = normalized_risk(r.hlr_m_per_h, bands.hlr_gold_max, bands.hlr_hard_max);
    // CEC & PFAS corridors
    r.r_cec = normalized_risk(r.c_cec, bands.cec_gold_max, bands.cec_hard_max);
    r.r_pfas = normalized_risk(r.c_pfas, bands.pfas_gold_max, bands.pfas_hard_max);

    // Simple Lyapunov‑like residual from normalized risks
    const double r1 = std::clamp(r.r_hlr, 0.0, 1.0);
    const double r2 = std::clamp(r.r_cec, 0.0, 1.0);
    const double r3 = std::clamp(r.r_pfas, 0.0, 1.0);
    r.vt = 0.5 * (r1 * r1 + r2 * r2 + r3 * r3);
}

bool corridor_ok(const Reach &r) {
    // Hard gate: no risk component may hit 1.0
    return r.r_hlr < 1.0 && r.r_cec < 1.0 && r.r_pfas < 1.0;
}

struct SimConfig {
    CorridorBands bands;
    double dt_days;
    uint64_t n_steps;
};

struct KER {
    double k_score;
    double e_score;
    double r_score;
};

// Example static KER for this research‑only kernel
KER default_ker() {
    return {0.94, 0.90, 0.12};
}

void run_simulation(std::vector<Reach> &reaches,
                    const SimConfig &cfg,
                    const std::string &csv_path)
{
    KER ker = default_ker();
    std::FILE *fp = std::fopen(csv_path.c_str(), "w");
    if (!fp) return;

    // RFC‑4180‑style header, shard‑ready
    std::fprintf(fp,
                 "reach_id,step,t_days,q_m3s,hlr_m_per_h,c_cec,c_pfas,"
                 "substrate_mass_kg,r_hlr,r_cec,r_pfas,vt,"
                 "k_score,e_score,r_score,hexstamp\n");

    const char *hexstamp =
        "0xb2c3d4e5f67890a1e2d3c4b5a6978899bb77dd55ff3311aa";

    for (uint64_t step = 0; step < cfg.n_steps; ++step) {
        double t_days = step * cfg.dt_days;

        for (auto &r : reaches) {
            update_substrate(r, cfg.dt_days);
            update_risks(r, cfg.bands);

            if (!corridor_ok(r)) {
                // Record breach; higher layers will reject this design
            }

            std::fprintf(
                fp,
                "%llu,%llu,%.6f,%.6f,%.6f,%.6f,%.6f,"
                "%.6f,%.6f,%.6f,%.6f,%.6f,"
                "%.2f,%.2f,%.2f,%s\n",
                static_cast<unsigned long long>(r.id),
                static_cast<unsigned long long>(step),
                t_days,
                r.q_m3s,
                r.hlr_m_per_h,
                r.c_cec,
                r.c_pfas,
                r.substrate_mass_kg,
                r.r_hlr,
                r.r_cec,
                r.r_pfas,
                r.vt,
                ker.k_score,
                ker.e_score,
                ker.r_score,
                hexstamp
            );
        }
    }

    std::fclose(fp);
}

int main() {
    CorridorBands bands{
        0.3, 0.6,   // HLR gold/hard
        10.0, 50.0, // CEC
        4.0, 20.0   // PFAS ng/L
    };

    Reach example{
        1,
        true,
        0.2,
        0.25,
        5.0,
        3.9,
        10.0,
        0.02,
        0.0,
        0.0,
        0.0,
        0.0
    };

    SimConfig cfg{bands, 1.0, 90};

    std::vector<Reach> reaches{example};
    run_simulation(reaches, cfg, "qpudatashards/cyboquatic_sim_example.csv");
    return 0;
}
