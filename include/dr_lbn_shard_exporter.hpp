#ifndef DR_LBN_SHARD_EXPORTER_HPP
#define DR_LBN_SHARD_EXPORTER_HPP

#include <string>
#include <fstream>
#include <sstream>
#include <iomanip>
#include <stdexcept>
#include <chrono>

namespace ecosafety {

struct DRLBNShardRow {
    // Identity / context
    std::string controller_id;    // e.g. "DRLBN-PHX-AIRGLOBE-01"
    std::string swarm_id;         // e.g. "PHX-AIRGLOBE-SWARM-01"
    std::string corridor_id;      // e.g. "CORRIDOR-I17-SCHOOL-SEG-A"

    // Medium / region / node (optional but useful for downstream joins)
    std::string medium;           // "Air", "Water", ...
    std::string region;           // "Phoenix-AZ", ...

    // Time window (ISO 8601, UTC recommended)
    std::string twindow_start;    // "2026-01-20T00:00:00Z"
    std::string twindow_end;      // "2026-01-21T00:00:00Z"

    // Eco-impact (ESPD / CEIM-style)
    double Braw      = 0.0;       // raw eco-benefit score
    double Rraw      = 0.0;       // raw eco-risk score
    double Dt        = 1.0;       // sensor trust scalar in [0,1]
    double Ki        = 0.0;       // Karma / integrity index
    double Ti        = 0.0;       // tolerance index
    double Badj      = 0.0;       // adjusted eco-benefit (e.g. Braw * Dt)
    std::string security_response_cap; // "LOW", "MEDIUM", "HIGH"

    // DR-LBN controller metadata
    std::string dr_lbn_id;        // usually same as controller_id
    std::string dr_lbn_version;   // e.g. "1.0"
    double      dr_lbn_robust_radius = 0.0; // robustness radius (disturbance ball)

    // Verification artifacts (hex hashes or content IDs)
    std::string cbf_cert_hex;     // ReLU CBF/DR-LBN verifier certificate hash
    std::string tla_model_hex;    // TLA+/Apalache model hash
    std::string prism_psy_hex;    // PRISM-PSY model/result hash

    // Probabilistic safety / eco-benefit bounds
    double psafe_min      = 0.0;  // minimum probability of staying safe
    double eco_benefit_p05 = 0.0; // 5th percentile eco-benefit
    double eco_benefit_p95 = 0.0; // 95th percentile eco-benefit

    // Evidence / notes
    std::string evidence_hex;     // master evidence hash
    std::string notes;            // single-line, may be quoted in CSV
};

class DRLBNShardExporter {
public:
    explicit DRLBNShardExporter(const std::string& filename,
                                bool append = false,
                                bool write_header_if_new = true)
        : filename_(filename)
    {
        std::ios::openmode mode = std::ios::out;
        if (append) {
            mode |= std::ios::app;
        }
        file_.open(filename_, mode);
        if (!file_.is_open()) {
            throw std::runtime_error("DRLBNShardExporter: cannot open file: " + filename_);
        }

        if (write_header_if_new && is_file_empty()) {
            write_header();
        }
    }

    ~DRLBNShardExporter() {
        if (file_.is_open()) {
            file_.flush();
            file_.close();
        }
    }

    void write_row(const DRLBNShardRow& row) {
        if (!file_.is_open()) {
            throw std::runtime_error("DRLBNShardExporter: file is not open");
        }
        file_ << escape(row.controller_id)        << ','
              << escape(row.swarm_id)             << ','
              << escape(row.corridor_id)          << ','
              << escape(row.medium)               << ','
              << escape(row.region)               << ','
              << escape(row.twindow_start)        << ','
              << escape(row.twindow_end)          << ','
              << format_double(row.Braw)          << ','
              << format_double(row.Rraw)          << ','
              << format_double(row.Dt)            << ','
              << format_double(row.Ki)            << ','
              << format_double(row.Ti)            << ','
              << format_double(row.Badj)          << ','
              << escape(row.security_response_cap)<< ','
              << escape(row.dr_lbn_id)            << ','
              << escape(row.dr_lbn_version)       << ','
              << format_double(row.dr_lbn_robust_radius) << ','
              << escape(row.cbf_cert_hex)         << ','
              << escape(row.tla_model_hex)        << ','
              << escape(row.prism_psy_hex)        << ','
              << format_double(row.psafe_min)     << ','
              << format_double(row.eco_benefit_p05) << ','
              << format_double(row.eco_benefit_p95) << ','
              << escape(row.evidence_hex)         << ','
              << escape(row.notes)
              << '\n';
        file_.flush();
    }

    static std::string now_iso8601_utc() {
        using clock = std::chrono::system_clock;
        auto now = clock::now();
        auto t   = clock::to_time_t(now);
        std::tm tm{};
#if defined(_WIN32)
        gmtime_s(&tm, &t);
#else
        gmtime_r(&t, &tm);
#endif
        char buf[32];
        if (std::strftime(buf, sizeof(buf), "%Y-%m-%dT%H:%M:%SZ", &tm) == 0) {
            return "";
        }
        return std::string(buf);
    }

private:
    std::string filename_;
    std::ofstream file_;

    bool is_file_empty() {
        file_.seekp(0, std::ios::end);
        return file_.tellp() == std::streampos(0);
    }

    void write_header() {
        file_ << "controller_id,"
              << "swarm_id,"
              << "corridor_id,"
              << "medium,"
              << "region,"
              << "twindow_start,"
              << "twindow_end,"
              << "Braw,"
              << "Rraw,"
              << "Dt,"
              << "Ki,"
              << "Ti,"
              << "Badj,"
              << "securityresponsecap,"
              << "dr_lbn_id,"
              << "dr_lbn_version,"
              << "dr_lbn_robust_radius,"
              << "cbf_cert_hex,"
              << "tla_model_hex,"
              << "prism_psy_hex,"
              << "psafe_min,"
              << "eco_benefit_p05,"
              << "eco_benefit_p95,"
              << "evidencehex,"
              << "notes"
              << '\n';
        file_.flush();
    }

    static std::string escape(const std::string& s) {
        bool need_quotes = false;
        for (char c : s) {
            if (c == ',' || c == '"' || c == '\n' || c == '\r') {
                need_quotes = true;
                break;
            }
        }
        if (!need_quotes) {
            return s;
        }
        std::ostringstream oss;
        oss << '"';
        for (char c : s) {
            if (c == '"') {
                oss << "\"\"";
            } else {
                oss << c;
            }
        }
        oss << '"';
        return oss.str();
    }

    static std::string format_double(double v) {
        std::ostringstream oss;
        oss << std::setprecision(10);
        oss << v;
        return oss.str();
    }
};

} // namespace ecosafety

#endif // DR_LBN_SHARD_EXPORTER_HPP
