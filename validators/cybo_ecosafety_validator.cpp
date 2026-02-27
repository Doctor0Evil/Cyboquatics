#include <string>
#include <vector>
#include <stdexcept>
#include <iostream>

// Assume you have a thin JSON wrapper; replace with your actual library.
#include "json.hpp" // nlohmann/json or equivalent

using json = nlohmann::json;

struct ValidationConfig {
    std::string expected_grammar_id;
    std::string owner_did;
    std::vector<std::string> forbidden_labels;  // e.g. {"DigitalWaterTwin", "Digital Water Twin"};
};

static std::string compute_hex_stamp(const json& shard_core);

bool verify_hex_stamp(const json& shard, const ValidationConfig& cfg) {
    if (!shard.contains("hex_stamp")) return false;

    // Define the "core" to be hashed: core_math ∥ grammar_id ∥ did_author
    json core;
    core["grammar_id"]  = shard.value("grammar_id", "");
    core["did_author"]  = shard.value("did_author", "");
    core["ker"]         = shard.value("ker", json::object());
    core["residual"]    = shard.value("residual", json::object());
    core["fields"]      = shard.value("fields", json::array());

    const std::string expected = compute_hex_stamp(core);
    const std::string provided = shard.value("hex_stamp", "");

    return (expected == provided);
}

bool labels_ok(const json& shard, const ValidationConfig& cfg) {
    if (!shard.contains("labels")) return true;
    const auto labels = shard.at("labels");
    if (!labels.is_array()) return false;

    for (const auto& lbl_json : labels) {
        if (!lbl_json.is_string()) continue;
        const std::string lbl = lbl_json.get<std::string>();
        for (const auto& forbidden : cfg.forbidden_labels) {
            if (lbl.find(forbidden) != std::string::npos) {
                return false;
            }
        }
    }
    return true;
}

bool check_origin_and_grammar(const json& shard, const ValidationConfig& cfg) {
    const std::string grammar_id = shard.value("grammar_id", "");
    const std::string origin     = shard.value("origin", "");
    const std::string did_author = shard.value("did_author", "");

    if (grammar_id == cfg.expected_grammar_id) {
        // Shards using your grammar_id must be authored by you and marked internal
        if (did_author != cfg.owner_did) return false;
        if (origin != "internal")        return false;
    }
    return true;
}

bool validate_shard(const json& shard, const ValidationConfig& cfg) {
    if (!check_origin_and_grammar(shard, cfg)) {
        std::cerr << "Origin/grammar/DID mismatch.\n";
        return false;
    }
    if (!labels_ok(shard, cfg)) {
        std::cerr << "Forbidden external branding label detected.\n";
        return false;
    }
    if (!verify_hex_stamp(shard, cfg)) {
        std::cerr << "Hex stamp verification failed.\n";
        return false;
    }
    return true;
}

// ---- placeholder hash implementation ----
// Replace this with your chosen hash (e.g., SHA-256 via OpenSSL/libsodium/etc.).
#include <sstream>
static std::string compute_hex_stamp(const json& shard_core) {
    std::string s = shard_core.dump();
    // Simple non-cryptographic placeholder: FNV-1a 64-bit
    const uint64_t FNV_OFFSET = 1469598103934665603ULL;
    const uint64_t FNV_PRIME  = 1099511628211ULL;
    uint64_t hash = FNV_OFFSET;
    for (unsigned char c : s) {
        hash ^= c;
        hash *= FNV_PRIME;
    }
    std::ostringstream oss;
    oss << std::hex << hash;
    return oss.str();
}

// Simple CLI entrypoint
int main(int argc, char** argv) {
    if (argc < 4) {
        std::cerr << "Usage: cybo_ecosafety_validator <json_file> <expected_grammar_id> <owner_did>\n";
        return 1;
    }

    const std::string json_path         = argv[1];
    const std::string expected_grammar  = argv[2];
    const std::string owner_did         = argv[3];

    ValidationConfig cfg;
    cfg.expected_grammar_id = expected_grammar;
    cfg.owner_did           = owner_did;
    cfg.forbidden_labels    = {"DigitalWaterTwin", "Digital Water Twin", "SmartWaterTwin"};

    std::ifstream in(json_path);
    if (!in) {
        std::cerr << "Cannot open file: " << json_path << "\n";
        return 1;
    }
    json shard;
    in >> shard;

    if (!validate_shard(shard, cfg)) {
        std::cerr << "Shard validation FAILED for " << json_path << "\n";
        return 2;
    }

    std::cout << "Shard validation OK for " << json_path << "\n";
    return 0;
}
