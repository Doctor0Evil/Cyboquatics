#pragma once
#include <string>
#include <vector>
#include <cstdint>

namespace cybo {

struct QpuStackRow {
    std::string destination_path;
    std::string module;
    std::string version;
    std::string role;
    std::string security_protocol;
    std::string interop_standard;
    std::string identity_mgmt;
    std::string ai_agent_integration;
    std::string device_type;
    std::string authentication;
    std::string digital_twin;
    std::string edge_analytics;
    std::string compliance;
    std::string log_persistence;
};

struct QpuNodeRow {
    std::string node_id;
    std::string location;
    std::string system;
    std::string contaminant;
    std::string parameter;
    std::string unit;
    double baseline_Cin;
    double baseline_Cout;
    double Q_cms;
    double cref;
    double wsup;
    double ecoimpactscore;
    double karmaperunit;
    std::string notes;
};

struct KarmaRow {
    std::string node_id;
    std::string contaminant;
    std::string jurisdictionsup;
    double Craw;
    double Creported;
    double Qavg_cms;
    double Kn;
    std::string window_start;
    std::string window_end;
    std::string unitsC;
    std::string unitsQ;
};

struct QpuShard {
    std::vector<QpuStackRow> stack_rows;
    std::vector<QpuNodeRow> node_rows;
    std::vector<KarmaRow>    karma_rows;
};

bool load_stack_csv(const std::string& path, std::vector<QpuStackRow>& out);
bool load_nodes_csv(const std::string& path, std::vector<QpuNodeRow>& out);
bool load_karma_csv(const std::string& path, std::vector<KarmaRow>& out);

} // namespace cybo
