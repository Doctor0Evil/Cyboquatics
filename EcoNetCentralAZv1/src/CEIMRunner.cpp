#include <fstream>
#include <sstream>
#include <string>
#include <vector>
#include <stdexcept>

struct EnvWindowRow {
    std::string nodeid;
    std::string stationid;
    std::string stakeholderid;
    std::string contaminant;
    double cin;            // concentration in (unit in unitsC)
    double cout;           // concentration out
    double flow;           // flow in cfs
    std::string windowstart;
    std::string windowend;
    double cref;           // reference concentration
    double hazardweight;   // CEIM weight
    double ecoimpactscore; // 0–1 from shard
    std::string unitsC;
    std::string unitsQ;
    std::string sourceprogram;
    double kn;             // computed CEIM node score
};

// Simplified: monthly vs annual durations (seconds)
static double parseDurationSeconds(const std::string &windowstart,
                                   const std::string &windowend) {
    // Very simple heuristic: if start is Jan and end is Dec -> annual
    if (windowstart.substr(5, 2) == "01" && windowend.substr(5, 2) == "12") {
        return 365.0 * 24.0 * 3600.0;
    }
    // Otherwise treat as ~30‑day window
    return 30.0 * 24.0 * 3600.0;
}

// Mass‑conservative CEIM kernel: Kn = w * (Cin‑Cout)/Cref * Q * Δt
static double computeKn(const EnvWindowRow &r) {
    double deltaC = r.cin - r.cout;
    if (deltaC <= 0.0 || r.cref <= 0.0 || r.flow <= 0.0) {
        return 0.0;
    }
    double duration = parseDurationSeconds(r.windowstart, r.windowend);
    double loadTerm = (deltaC / r.cref) * r.flow * duration;
    return r.hazardweight * loadTerm;
}

static std::vector<EnvWindowRow> loadWindows(const std::string &path) {
    std::ifstream file(path);
    if (!file.is_open()) {
        throw std::runtime_error("Unable to open CSV: " + path);
    }
    std::vector<EnvWindowRow> rows;
    std::string line;

    // Skip header
    if (!std::getline(file, line)) {
        return rows;
    }

    while (std::getline(file, line)) {
        if (line.empty()) continue;
        std::stringstream ss(line);
        std::string field;
        std::vector<std::string> fields;

        while (std::getline(ss, field, ',')) {
            fields.push_back(field);
        }
        if (fields.size() < 15) continue;

        EnvWindowRow r;
        r.nodeid        = fields[0];
        r.stationid     = fields[1];
        r.stakeholderid = fields[2];
        r.contaminant   = fields[3];
        r.cin           = std::stod(fields[4]);
        r.cout          = std::stod(fields[5]);
        r.flow          = std::stod(fields[6]);
        r.windowstart   = fields[7];
        r.windowend     = fields[8];
        r.cref          = std::stod(fields[9]);
        r.hazardweight  = std::stod(fields[10]);
        r.ecoimpactscore= std::stod(fields[11]);
        r.unitsC        = fields[12];
        r.unitsQ        = fields[13];
        r.sourceprogram = fields[14];
        r.kn            = 0.0;
        rows.push_back(r);
    }
    return rows;
}

static void writeKarmaCSV(const std::string &outPath,
                          const std::vector<EnvWindowRow> &rows) {
    std::ofstream out(outPath);
    if (!out.is_open()) {
        throw std::runtime_error("Unable to write CSV: " + outPath);
    }
    out << "nodeid,contaminant,kn,ecoimpactscore,windowstart,windowend,unitsC,unitsQ,sourceprogram\n";
    for (const auto &r : rows) {
        out << r.nodeid << ','
            << r.contaminant << ','
            << r.kn << ','
            << r.ecoimpactscore << ','
            << r.windowstart << ','
            << r.windowend << ','
            << r.unitsC << ','
            << r.unitsQ << ','
            << r.sourceprogram << '\n';
    }
}

int main() {
    const std::string inPath  = "qpudatashardsparticles/EcoNetCentralAZKarmaWindowTemplate.csv";
    const std::string outPath = "qpudatashardsparticles/CEIMXJKarma20241130.csv";

    auto rows = loadWindows(inPath);
    for (auto &r : rows) {
        r.kn = computeKn(r);
    }
    writeKarmaCSV(outPath, rows);
    return 0;
}
