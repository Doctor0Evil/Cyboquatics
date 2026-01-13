#include "cybo_qpudata.h"
#include <fstream>
#include <sstream>

namespace cybo {

namespace {
bool getline_csv(std::istream& in, std::vector<std::string>& cols) {
    cols.clear();
    std::string line;
    if (!std::getline(in, line)) return false;
    std::stringstream ss(line);
    std::string cell;
    while (std::getline(ss, cell, ',')) cols.push_back(cell);
    return true;
}
}

bool load_stack_csv(const std::string& path, std::vector<QpuStackRow>& out) {
    std::ifstream f(path);
    if (!f.is_open()) return false;
    std::vector<std::string> cols;
    if (!getline_csv(f, cols)) return false;
    while (getline_csv(f, cols)) {
        if (cols.size() < 14) continue;
        QpuStackRow r;
        r.destination_path   = cols[0];
        r.module             = cols[1];
        r.version            = cols[2];
        r.role               = cols[3];
        r.security_protocol  = cols[4];
        r.interop_standard   = cols[5];
        r.identity_mgmt      = cols[6];
        r.ai_agent_integration = cols[7];
        r.device_type        = cols[8];
        r.authentication     = cols[9];
        r.digital_twin       = cols[10];
        r.edge_analytics     = cols[11];
        r.compliance         = cols[12];
        r.log_persistence    = cols[13];
        out.push_back(r);
    }
    return true;
}

bool load_nodes_csv(const std::string& path, std::vector<QpuNodeRow>& out) {
    std::ifstream f(path);
    if (!f.is_open()) return false;
    std::vector<std::string> cols;
    if (!getline_csv(f, cols)) return false;
    while (getline_csv(f, cols)) {
        if (cols.size() < 13) continue;
        QpuNodeRow r;
        r.node_id          = cols[0];
        r.location         = cols[1];
        r.system           = cols[2];
        r.contaminant      = cols[3];
        r.parameter        = cols[4];
        r.unit             = cols[5];
        r.baseline_Cin     = std::stod(cols[6]);
        r.baseline_Cout    = std::stod(cols[7]);
        r.Q_cms            = std::stod(cols[8]);
        r.cref             = std::stod(cols[9]);
        r.wsup             = std::stod(cols[10]);
        r.ecoimpactscore   = std::stod(cols[11]);
        r.karmaperunit     = std::stod(cols[12]);
        if (cols.size() > 13) r.notes = cols[13];
        out.push_back(r);
    }
    return true;
}

bool load_karma_csv(const std::string& path, std::vector<KarmaRow>& out) {
    std::ifstream f(path);
    if (!f.is_open()) return false;
    std::vector<std::string> cols;
    if (!getline_csv(f, cols)) return false;
    while (getline_csv(f, cols)) {
        if (cols.size() < 10) continue;
        KarmaRow r;
        r.node_id        = cols[0];
        r.contaminant    = cols[1];
        r.jurisdictionsup= cols[2];
        r.Craw           = std::stod(cols[3]);
        r.Creported      = std::stod(cols[4]);
        r.Qavg_cms       = std::stod(cols[5]);
        r.Kn             = std::stod(cols[6]);
        r.window_start   = cols[7];
        r.window_end     = cols[8];
        r.unitsC         = cols[9];
        r.unitsQ         = cols.size() > 10 ? cols[10] : "";
        out.push_back(r);
    }
    return true;
}

} // namespace cybo
