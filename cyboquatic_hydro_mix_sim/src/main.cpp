// Filename: cyboquatic_hydro_mix_sim/src/main.cpp
// High-performance 1D canal + SAT offload simulator with ecosafety corridors.

#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <string>
#include <vector>
#include <iostream>
#include <fstream>
#include <sstream>
#include <iomanip>

struct Reach {
    // Physical
    double length_m;
    double q_m3s;       // discharge
    double area_m2;     // wetted area
    double hlr_m_per_h; // SAT hydraulic loading rate where applicable
    bool   has_sat;

    // Quality (PFAS / generic CEC)
    double cin_ngL;     // inflow concentration
    double k_day;       // first-order decay constant
};

struct CorridorBands {
    double safe;
    double gold;
    double hard;
};

struct RiskCoords {
    double r_sat;       // HLR corridor 0–1
    double r_cec;       // CEC corridor 0–1
    double r_pfas;      // PFAS-specific corridor 0–1
};

struct State {
    double c_ngL;       // reach concentration
    double hlr_m_per_h; // realized HLR
};

struct Residual {
    double vt;
};

struct SimConfig {
    double dt_s;
    double t_end_s;
    CorridorBands hlr_corr;
    CorridorBands cec_corr;
    CorridorBands pfas_corr;
};

static inline double clamp01(double x) {
    if (x < 0.0) return 0.0;
    if (x > 1.0) return 1.0;
    return x;
}

// Normalize x into 0–1 risk against corridor bands.
double corridor_risk(double x, const CorridorBands& bands) {
    if (x <= bands.safe) return 0.0;
    if (x >= bands.hard) return 1.0;
    if (x <= bands.gold) {
        return 0.5 * (x - bands.safe) / (bands.gold - bands.safe + 1e-12);
    }
    return 0.5 + 0.5 * (x - bands.gold) / (bands.hard - bands.gold + 1e-12);
}

// Discrete Lyapunov kernel V_t = sum w_j r_j^2 (w_j >= 0).
double compute_residual(const RiskCoords& r, const double w_sat,
                        const double w_cec, const double w_pfas) {
    return w_sat  * r.r_sat  * r.r_sat +
           w_cec  * r.r_cec  * r.r_cec +
           w_pfas * r.r_pfas * r.r_pfas;
}

// Safestep invariant: forbid steps that increase V_t outside safe interior.
bool safestep(const Residual& prev, const Residual& next, double eps = 1e-9) {
    return next.vt <= prev.vt + eps;
}

// RFC-4180 shard writer.
class ShardWriter {
public:
    explicit ShardWriter(const std::string& path)
        : path_(path), header_written_(false) {}

    void write_header_if_needed() {
        if (header_written_) return;
        std::ofstream f(path_, std::ios::out | std::ios::trunc);
        if (!f) {
            throw std::runtime_error("Cannot open shard file for writing header");
        }
        f << "reachid,region,lat,lon,tstart_s,tend_s,"
          << "cin_ngL,cout_ngL,hlr_m_per_h,"
          << "r_sat,r_cec,r_pfas,vt,"
          << "knowledgefactor01,ecoimpact01,riskofharm01,"
          << "evidencehex,notes\n";
        header_written_ = true;
    }

    void append_row(int reachid,
                    const std::string& region,
                    double lat, double lon,
                    double tstart_s, double tend_s,
                    double cin_ngL, double cout_ngL,
                    double hlr_m_per_h,
                    const RiskCoords& r,
                    const Residual& res,
                    double kf, double eco, double risk,
                    const std::string& hexstamp,
                    const std::string& notes) {
        if (!header_written_) {
            write_header_if_needed();
        }
        std::ofstream f(path_, std::ios::out | std::ios::app);
        if (!f) {
            throw std::runtime_error("Cannot open shard file for appending");
        }
        f << reachid << "," << region << ","
          << std::fixed << std::setprecision(5) << lat << ","
          << std::fixed << std::setprecision(5) << lon << ","
          << std::fixed << std::setprecision(1) << tstart_s << ","
          << std::fixed << std::setprecision(1) << tend_s << ","
          << std::fixed << std::setprecision(3) << cin_ngL << ","
          << std::fixed << std::setprecision(3) << cout_ngL << ","
          << std::fixed << std::setprecision(3) << hlr_m_per_h << ","
          << std::fixed << std::setprecision(3) << r.r_sat << ","
          << std::fixed << std::setprecision(3) << r.r_cec << ","
          << std::fixed << std::setprecision(3) << r.r_pfas << ","
          << std::fixed << std::setprecision(4) << res.vt << ","
          << std::fixed << std::setprecision(2) << kf << ","
          << std::fixed << std::setprecision(2) << eco << ","
          << std::fixed << std::setprecision(2) << risk << ","
          << hexstamp << ",\"" << escape(notes) << "\"\n";
    }

private:
    std::string path_;
    bool header_written_;

    static std::string escape(const std::string& s) {
        std::string out;
        out.reserve(s.size());
        for (char c : s) {
            if (c == '"') out.push_back('\'');
            else if (c == '\n' || c == '\r') out.push_back(' ');
            else out.push_back(c);
        }
        return out;
    }
};

// Simple PFAS / CEC transport + decay in a reach (explicit Euler).
void step_reach(const Reach& r, double dt_s, State& st) {
    double tau_s = r.length_m * r.area_m2 / (r.q_m3s * r.area_m2 + 1e-12);
    double q_in = r.q_m3s;
    double v = r.area_m2 / (r.area_m2 + 1e-12);

    double lambda_s = r.k_day / 86400.0;
    double c_old = st.c_ngL;

    double adv_mix = (r.cin_ngL - c_old) / (tau_s + 1e-12);
    double decay = -lambda_s * c_old;

    double dc_dt = adv_mix + decay;
    st.c_ngL = std::max(0.0, c_old + dt_s * dc_dt);

    if (r.has_sat) {
        st.hlr_m_per_h = r.hlr_m_per_h;
    } else {
        st.hlr_m_per_h = 0.0;
    }

    (void)q_in;
    (void)v;
}

// Compute risk coordinates from state and corridors.
RiskCoords risk_from_state(const State& st,
                           const CorridorBands& hlr_corr,
                           const CorridorBands& cec_corr,
                           const CorridorBands& pfas_corr) {
    RiskCoords r{};
    r.r_sat  = corridor_risk(st.hlr_m_per_h, hlr_corr);
    r.r_cec  = corridor_risk(st.c_ngL,       cec_corr);
    r.r_pfas = corridor_risk(st.c_ngL,       pfas_corr);
    return r;
}

// K/E/R aggregation for a time window.
void compute_KER_window(const std::vector<Residual>& residuals,
                        const std::vector<RiskCoords>& risks,
                        double& K_out, double& E_out, double& R_out) {
    // Knowledge-factor: fraction of steps with valid residual satisfying safestep.
    size_t n = residuals.size();
    if (n < 2) {
        K_out = 0.0;
        E_out = 0.0;
        R_out = 1.0;
        return;
    }

    size_t ok_steps = 0;
    double max_r = 0.0;
    for (size_t i = 1; i < n; ++i) {
        if (residuals[i].vt <= residuals[i-1].vt + 1e-9) ++ok_steps;
    }
    for (const auto& rc : risks) {
        max_r = std::max(max_r, std::max(rc.r_sat, std::max(rc.r_cec, rc.r_pfas)));
    }

    K_out = static_cast<double>(ok_steps) / static_cast<double>(n - 1);
    E_out = clamp01(1.0 - max_r); // eco-impact higher when risks are low
    R_out = clamp01(max_r);       // risk-of-harm tied to corridor penetration
}

// Main driver: simulate chain of reaches, emit shard rows at coarse windows.
int main(int argc, char** argv) {
    try {
        SimConfig cfg{};
        cfg.dt_s = 10.0;
        cfg.t_end_s = 24.0 * 3600.0;

        cfg.hlr_corr = {0.0, 0.3, 0.6};      // SAT HLR safe/gold/hard
        cfg.cec_corr = {0.0, 30.0, 100.0};   // mg/L -> normalize upstream if needed
        cfg.pfas_corr = {0.0, 20.0, 70.0};   // ng/L corridor as per Phoenix shards

        std::vector<Reach> reaches;
        reaches.push_back(Reach{1000.0, 5.0, 10.0, 0.25, true, 3.9, 0.01});
        reaches.push_back(Reach{800.0,  5.0,  8.0, 0.15, false, 3.0, 0.008});
        reaches.push_back(Reach{600.0,  4.5,  7.0, 0.10, true,  2.0, 0.006});

        const std::string region = "Central-AZ";
        const double lat0 = 33.45;
        const double lon0 = -112.07;

        ShardWriter writer("qpudatashardsparticlesHydroCyboquaticPhoenixCxx2026v1.csv");
        writer.write_header_if_needed();

        std::vector<State> states(reaches.size());
        for (size_t i = 0; i < reaches.size(); ++i) {
            states[i].c_ngL = reaches[i].cin_ngL;
            states[i].hlr_m_per_h = reaches[i].hlr_m_per_h;
        }

        double t = 0.0;
        const double window_s = 3600.0;
        double next_window = window_s;

        std::vector<RiskCoords> window_risks;
        std::vector<Residual>   window_resids;

        const double w_sat = 0.4, w_cec = 0.3, w_pfas = 0.3;

        while (t < cfg.t_end_s + 0.5 * cfg.dt_s) {
            for (size_t i = 0; i < reaches.size(); ++i) {
                Reach r = reaches[i];
                if (i > 0) r.cin_ngL = states[i-1].c_ngL;
                step_reach(r, cfg.dt_s, states[i]);

                RiskCoords rc = risk_from_state(states[i], cfg.hlr_corr,
                                                cfg.cec_corr, cfg.pfas_corr);
                Residual res;
                res.vt = compute_residual(rc, w_sat, w_cec, w_pfas);

                window_risks.push_back(rc);
                window_resids.push_back(res);
            }

            if (t + cfg.dt_s >= next_window || t + cfg.dt_s >= cfg.t_end_s) {
                for (size_t i = 0; i < reaches.size(); ++i) {
                    double K, E, R;
                    compute_KER_window(window_resids, window_risks, K, E, R);

                    double tstart = next_window - window_s;
                    double tend   = next_window;

                    RiskCoords rc = window_risks.back();
                    Residual res  = window_resids.back();

                    std::ostringstream hex;
                    hex << "0xa1b2c3d4e5f6" << std::hex << static_cast<uint64_t>(t) << i;

                    std::ostringstream note;
                    note << "C++ hydro-mix window; reach " << i
                         << " PFAS corridor risk " << std::setprecision(3) << rc.r_pfas;

                    writer.append_row(
                        static_cast<int>(i),
                        region,
                        lat0 + 0.01 * static_cast<double>(i),
                        lon0 - 0.01 * static_cast<double>(i),
                        tstart, tend,
                        reaches[i].cin_ngL,
                        states[i].c_ngL,
                        states[i].hlr_m_per_h,
                        rc,
                        res,
                        K, E, R,
                        hex.str(),
                        note.str()
                    );
                }

                window_risks.clear();
                window_resids.clear();
                next_window += window_s;
            }

            t += cfg.dt_s;
        }

        return 0;
    } catch (const std::exception& ex) {
        std::cerr << "Fatal error: " << ex.what() << "\n";
        return 1;
    }
}
