// ============================================================================
// FILE: cyboquatics_hardware/src/nanoswarm_controller.cpp
// DESTINATION: /cyboquatics/cyboquatics_hardware/src/nanoswarm_controller.cpp
// LICENSE: MIT Public Good License (Non-Commercial, Open Ecosafety)
// VERSION: 1.0.0-alpha
// ============================================================================
// Cyboquatics C++ Hardware Abstraction Layer - Nanoswarm Control & Safety
// Interfaces with Rust kernel, enforces corridor invariants at hardware level
// ============================================================================

#include <iostream>
#include <vector>
#include <string>
#include <memory>
#include <atomic>
#include <mutex>
#include <chrono>
#include <cstdint>
#include <functional>
#include <optional>
#include <variant>
#include <unordered_map>
#include <fstream>
#include <sstream>
#include <iomanip>
#include <cstring>
#include <cmath>
#include <algorithm>
#include <thread>
#include <condition_variable>

// ============================================================================
// EXTERNAL DEPENDENCIES (Simulated for standalone compilation)
// ============================================================================

extern "C" {
    // Rust Kernel FFI Bindings
    typedef struct EcosafetyKernel EcosafetyKernel;
    typedef struct QpuDatashard QpuDatashard;
    
    EcosafetyKernel* kernel_new();
    void kernel_free(EcosafetyKernel* k);
    int kernel_register_corridor(EcosafetyKernel* k, const char* corridor_json);
    int kernel_execute_safe_action(EcosafetyKernel* k, const char* action_data, const char* corridor_id);
    const char* kernel_get_last_shard(EcosafetyKernel* k);
    int kernel_emergency_stop(EcosafetyKernel* k);
    int kernel_validate_invariants(EcosafetyKernel* k);
}

// ============================================================================
// CONSTANTS & SAFETY THRESHOLDS
// ============================================================================

namespace cyboquatics {
namespace constants {

constexpr double KER_K_THRESHOLD = 0.90;
constexpr double KER_E_THRESHOLD = 0.90;
constexpr double KER_R_THRESHOLD = 0.13;
constexpr double KER_R_PRODUCTION = 0.10;
constexpr double LYAPUNOV_THRESHOLD = 1.0;
constexpr double MAX_RISK_COORDINATE = 0.13;
constexpr uint64_t NANOSWARM_MAX_COUNT = 1000000;
constexpr double NANOSWARM_MAX_VELOCITY = 0.5; // meters per second
constexpr double NANOSWARM_MAX_DENSITY = 100.0; // units per cubic meter
constexpr std::chrono::milliseconds SAFETY_CHECK_INTERVAL{100};
constexpr std::chrono::milliseconds AUDIT_INTERVAL{1000};

} // namespace constants
} // namespace cyboquatics

// ============================================================================
// TYPE DEFINITIONS - Ecosafety Domain Types
// ============================================================================

namespace cyboquatics {
namespace types {

using RiskCoordinate = double;
using TimestampNs = uint64_t;
using CorridorId = std::string;
using ShardId = std::string;
using DidSignature = std::string;
using HexStamp = std::string;

struct KERScore {
    double knowledge_factor;
    double eco_impact;
    double risk_of_harm;
    
    bool is_deployable() const {
        return knowledge_factor >= constants::KER_K_THRESHOLD
            && eco_impact >= constants::KER_E_THRESHOLD
            && risk_of_harm <= constants::KER_R_THRESHOLD;
    }
    
    bool is_production_ready() const {
        return is_deployable() && risk_of_harm <= constants::KER_R_PRODUCTION;
    }
    
    std::string to_string() const {
        std::ostringstream oss;
        oss << std::fixed << std::setprecision(2);
        oss << "K=" << knowledge_factor 
            << " E=" << eco_impact 
            << " R=" << risk_of_harm;
        return oss.str();
    }
};

struct LyapunovResidual {
    TimestampNs timestamp_ns;
    double value;
    double derivative;
    bool is_stable;
    
    LyapunovResidual() : timestamp_ns(0), value(0.0), derivative(0.0), is_stable(true) {}
    
    LyapunovResidual(TimestampNs ts, double val, double deriv)
        : timestamp_ns(ts), value(val), derivative(deriv) {
        is_stable = derivative <= 0.0;
    }
    
    static TimestampNs current_timestamp_ns() {
        auto now = std::chrono::system_clock::now();
        auto epoch = now.time_since_epoch();
        return std::chrono::duration_cast<std::chrono::nanoseconds>(epoch).count();
    }
};

enum class CorridorDimensionType {
    WaterQuality,
    AirQuality,
    SoilHealth,
    HabitatSafety
};

struct WaterQualityDimension {
    RiskCoordinate ph;
    RiskCoordinate turbidity;
    RiskCoordinate contaminants;
    
    bool is_safe() const {
        return ph <= constants::MAX_RISK_COORDINATE
            && turbidity <= constants::MAX_RISK_COORDINATE
            && contaminants <= constants::MAX_RISK_COORDINATE;
    }
};

struct AirQualityDimension {
    RiskCoordinate pm25;
    RiskCoordinate pm10;
    RiskCoordinate voc;
    
    bool is_safe() const {
        return pm25 <= constants::MAX_RISK_COORDINATE
            && pm10 <= constants::MAX_RISK_COORDINATE
            && voc <= constants::MAX_RISK_COORDINATE;
    }
};

struct SoilHealthDimension {
    RiskCoordinate toxicity;
    RiskCoordinate erosion;
    RiskCoordinate biodiversity;
    
    bool is_safe() const {
        return toxicity <= constants::MAX_RISK_COORDINATE
            && erosion <= constants::MAX_RISK_COORDINATE
            && biodiversity <= constants::MAX_RISK_COORDINATE;
    }
};

struct HabitatSafetyDimension {
    RiskCoordinate species_risk;
    RiskCoordinate displacement;
    RiskCoordinate recovery;
    
    bool is_safe() const {
        return species_risk <= constants::MAX_RISK_COORDINATE
            && displacement <= constants::MAX_RISK_COORDINATE
            && recovery <= constants::MAX_RISK_COORDINATE;
    }
};

using CorridorDimension = std::variant<
    WaterQualityDimension,
    AirQualityDimension,
    SoilHealthDimension,
    HabitatSafetyDimension
>;

struct EcosafetyCorridor {
    CorridorId corridor_id;
    std::vector<CorridorDimension> dimensions;
    TimestampNs created_ns;
    TimestampNs last_validated_ns;
    bool is_active;
    KERScore ker_score;
    
    EcosafetyCorridor() : created_ns(0), last_validated_ns(0), is_active(false) {}
    
    EcosafetyCorridor(const CorridorId& id, 
                      const std::vector<CorridorDimension>& dims,
                      const KERScore& score)
        : corridor_id(id), 
          dimensions(dims),
          created_ns(LyapunovResidual::current_timestamp_ns()),
          last_validated_ns(LyapunovResidual::current_timestamp_ns()),
          is_active(true),
          ker_score(score) {}
    
    bool validate_all_dimensions() const {
        for (const auto& dim : dimensions) {
            bool dim_safe = std::visit([](const auto& d) { return d.is_safe(); }, dim);
            if (!dim_safe) {
                return false;
            }
        }
        return true;
    }
};

struct QpuDatashard {
    ShardId shard_id;
    HexStamp hex_stamp;
    DidSignature did_signature;
    TimestampNs timestamp_ns;
    CorridorId corridor_id;
    KERScore ker_snapshot;
    LyapunovResidual lyapunov_snapshot;
    std::string action_hash;
    std::optional<std::string> previous_shard_hash;
    
    std::string to_json() const {
        std::ostringstream oss;
        oss << "{";
        oss << "\"shard_id\":\"" << shard_id << "\",";
        oss << "\"hex_stamp\":\"" << hex_stamp << "\",";
        oss << "\"did_signature\":\"" << did_signature << "\",";
        oss << "\"timestamp_ns\":" << timestamp_ns << ",";
        oss << "\"corridor_id\":\"" << corridor_id << "\",";
        oss << "\"ker_snapshot\":{\"k\":" << ker_snapshot.knowledge_factor
            << ",\"e\":" << ker_snapshot.eco_impact
            << ",\"r\":" << ker_snapshot.risk_of_harm << "},";
        oss << "\"lyapunov\":{\"value\":" << lyapunov_snapshot.value
            << ",\"derivative\":" << lyapunov_snapshot.derivative << "},";
        oss << "\"action_hash\":\"" << action_hash << "\"";
        if (previous_shard_hash) {
            oss << ",\"previous_shard_hash\":\"" << *previous_shard_hash << "\"";
        }
        oss << "}";
        return oss.str();
    }
};

} // namespace types
} // namespace cyboquatics

// ============================================================================
// ERROR TYPES - Ecosafety Exception Handling
// ============================================================================

namespace cyboquatics {
namespace errors {

enum class EcosafetyErrorCode {
    Success = 0,
    InvalidRiskCoordinate,
    InvalidKERScore,
    CorridorNotDeployable,
    InvariantViolation,
    LyapunovInstability,
    ShardChainBroken,
    HardwareDerateTriggered,
    EmergencyStopActivated,
    KernelInitializationFailed,
    CorridorNotFound,
    NanoswarmLimitExceeded
};

struct EcosafetyError {
    EcosafetyErrorCode code;
    std::string message;
    std::string invariant_name;
    std::string violation_reason;
    
    EcosafetyError() : code(EcosafetyErrorCode::Success) {}
    
    EcosafetyError(EcosafetyErrorCode c, const std::string& msg)
        : code(c), message(msg) {}
    
    EcosafetyError(EcosafetyErrorCode c, const std::string& inv, const std::string& reason)
        : code(c), invariant_name(inv), violation_reason(reason) {
        message = "Invariant violation: " + inv + " - " + reason;
    }
    
    bool is_critical() const {
        return code == EcosafetyErrorCode::EmergencyStopActivated
            || code == EcosafetyErrorCode::HardwareDerateTriggered
            || code == EcosafetyErrorCode::LyapunovInstability;
    }
};

} // namespace errors
} // namespace cyboquatics

// ============================================================================
// SAFETY INVARIANT INTERFACE - Hardware-Level Enforcement
// ============================================================================

namespace cyboquatics {
namespace invariants {

using namespace types;
using namespace errors;

class SafetyInvariant {
public:
    virtual ~SafetyInvariant() = default;
    virtual std::string name() const = 0;
    virtual EcosafetyError check() const = 0;
    virtual EcosafetyError enforce() const = 0;
};

class CorridorCompleteInvariant : public SafetyInvariant {
private:
    EcosafetyCorridor corridor_;
    
public:
    explicit CorridorCompleteInvariant(const EcosafetyCorridor& corridor)
        : corridor_(corridor) {}
    
    std::string name() const override {
        return "invariant.corridorcomplete";
    }
    
    EcosafetyError check() const override {
        if (!corridor_.is_active) {
            return EcosafetyError(EcosafetyErrorCode::InvariantViolation,
                                  name(), "corridor_inactive");
        }
        if (!corridor_.validate_all_dimensions()) {
            return EcosafetyError(EcosafetyErrorCode::InvariantViolation,
                                  name(), "dimension_violation");
        }
        if (!corridor_.ker_score.is_deployable()) {
            return EcosafetyError(EcosafetyErrorCode::InvariantViolation,
                                  name(), "ker_not_deployable");
        }
        return EcosafetyError();
    }
    
    EcosafetyError enforce() const override {
        EcosafetyError err = check();
        if (err.code != EcosafetyErrorCode::Success) {
            return err;
        }
        return EcosafetyError();
    }
};

class ResidualSafeInvariant : public SafetyInvariant {
private:
    LyapunovResidual lyapunov_;
    double threshold_;
    
public:
    ResidualSafeInvariant(const LyapunovResidual& lyapunov, double threshold)
        : lyapunov_(lyapunov), threshold_(threshold) {}
    
    std::string name() const override {
        return "invariant.residualsafe";
    }
    
    EcosafetyError check() const override {
        if (!lyapunov_.is_stable) {
            return EcosafetyError(EcosafetyErrorCode::InvariantViolation,
                                  name(), "residual_unstable");
        }
        if (lyapunov_.value > threshold_) {
            return EcosafetyError(EcosafetyErrorCode::InvariantViolation,
                                  name(), "residual_exceeds_threshold");
        }
        return EcosafetyError();
    }
    
    EcosafetyError enforce() const override {
        EcosafetyError err = check();
        if (err.code != EcosafetyErrorCode::Success) {
            return err;
        }
        return EcosafetyError();
    }
};

class KerDeployableInvariant : public SafetyInvariant {
private:
    KERScore ker_score_;
    
public:
    explicit KerDeployableInvariant(const KERScore& score)
        : ker_score_(score) {}
    
    std::string name() const override {
        return "invariant.kerdeployable";
    }
    
    EcosafetyError check() const override {
        if (!ker_score_.is_deployable()) {
            return EcosafetyError(EcosafetyErrorCode::InvariantViolation,
                                  name(), "ker_threshold_not_met");
        }
        return EcosafetyError();
    }
    
    EcosafetyError enforce() const override {
        EcosafetyError err = check();
        if (err.code != EcosafetyErrorCode::Success) {
            return err;
        }
        return EcosafetyError();
    }
};

} // namespace invariants
} // namespace cyboquatics

// ============================================================================
// NANOSWARM HARDWARE ABSTRACTION - Device Control Layer
// ============================================================================

namespace cyboquatics {
namespace hardware {

using namespace types;
using namespace errors;
using namespace invariants;

enum class NanoswarmType {
    Filter,
    Sensor,
    Actuator,
    Communicator,
    PowerUnit
};

enum class NanoswarmState {
    Idle,
    Deploying,
    Active,
    Returning,
    Charging,
    EmergencyStop,
    Derated
};

struct NanoswarmUnit {
    uint64_t unit_id;
    NanoswarmType type;
    NanoswarmState state;
    double position_x;
    double position_y;
    double position_z;
    double velocity;
    double battery_level;
    TimestampNs last_heartbeat_ns;
    bool is_safe;
    
    NanoswarmUnit() 
        : unit_id(0), type(NanoswarmType::Sensor), state(NanoswarmState::Idle),
          position_x(0), position_y(0), position_z(0), velocity(0),
          battery_level(1.0), last_heartbeat_ns(0), is_safe(true) {}
    
    NanoswarmUnit(uint64_t id, NanoswarmType t)
        : unit_id(id), type(t), state(NanoswarmState::Idle),
          position_x(0), position_y(0), position_z(0), velocity(0),
          battery_level(1.0), last_heartbeat_ns(LyapunovResidual::current_timestamp_ns()),
          is_safe(true) {}
    
    bool is_within_safe_velocity() const {
        return velocity <= constants::NANOSWARM_MAX_VELOCITY;
    }
    
    bool is_within_safe_battery() const {
        return battery_level >= 0.1; // 10% minimum
    }
};

struct NanoswarmCluster {
    std::vector<NanoswarmUnit> units;
    CorridorId corridor_id;
    std::atomic<uint64_t> active_count{0};
    std::atomic<bool> emergency_stop{false};
    mutable std::mutex cluster_mutex;
    
    NanoswarmCluster() : active_count(0), emergency_stop(false) {}
    
    explicit NanoswarmCluster(const CorridorId& id)
        : corridor_id(id), active_count(0), emergency_stop(false) {}
    
    bool add_unit(const NanoswarmUnit& unit) {
        std::lock_guard<std::mutex> lock(cluster_mutex);
        if (units.size() >= constants::NANOSWARM_MAX_COUNT) {
            return false;
        }
        units.push_back(unit);
        active_count++;
        return true;
    }
    
    bool remove_unit(uint64_t unit_id) {
        std::lock_guard<std::mutex> lock(cluster_mutex);
        auto it = std::find_if(units.begin(), units.end(),
            [unit_id](const NanoswarmUnit& u) { return u.unit_id == unit_id; });
        if (it != units.end()) {
            units.erase(it);
            active_count--;
            return true;
        }
        return false;
    }
    
    bool update_unit_state(uint64_t unit_id, NanoswarmState new_state) {
        std::lock_guard<std::mutex> lock(cluster_mutex);
        auto it = std::find_if(units.begin(), units.end(),
            [unit_id](const NanoswarmUnit& u) { return u.unit_id == unit_id; });
        if (it != units.end()) {
            it->state = new_state;
            it->last_heartbeat_ns = LyapunovResidual::current_timestamp_ns();
            return true;
        }
        return false;
    }
    
    std::vector<NanoswarmUnit> get_active_units() const {
        std::lock_guard<std::mutex> lock(cluster_mutex);
        std::vector<NanoswarmUnit> active;
        for (const auto& unit : units) {
            if (unit.state != NanoswarmState::Idle && 
                unit.state != NanoswarmState::EmergencyStop) {
                active.push_back(unit);
            }
        }
        return active;
    }
    
    bool validate_all_units_safe() const {
        std::lock_guard<std::mutex> lock(cluster_mutex);
        for (const auto& unit : units) {
            if (!unit.is_safe || 
                !unit.is_within_safe_velocity() || 
                !unit.is_within_safe_battery()) {
                return false;
            }
        }
        return true;
    }
    
    void trigger_emergency_stop() {
        std::lock_guard<std::mutex> lock(cluster_mutex);
        emergency_stop = true;
        for (auto& unit : units) {
            unit.state = NanoswarmState::EmergencyStop;
            unit.velocity = 0;
        }
    }
    
    void derate_cluster() {
        std::lock_guard<std::mutex> lock(cluster_mutex);
        for (auto& unit : units) {
            if (unit.state != NanoswarmState::EmergencyStop) {
                unit.state = NanoswarmState::Derated;
                unit.velocity = std::min(unit.velocity, constants::NANOSWARM_MAX_VELOCITY * 0.5);
            }
        }
    }
};

} // namespace hardware
} // namespace cyboquatics

// ============================================================================
// SHARD CHAIN MANAGER - Audit Trail & Cryptographic Verification
// ============================================================================

namespace cyboquatics {
namespace audit {

using namespace types;
using namespace errors;

class ShardChainManager {
private:
    std::vector<QpuDatashard> shard_chain_;
    std::string genesis_hash_;
    mutable std::mutex chain_mutex_;
    EcosafetyKernel* kernel_ptr_;
    
    std::string compute_hash(const std::string& data) const {
        // Simplified hash - in production use SHA3-256
        std::hash<std::string> hasher;
        std::ostringstream oss;
        oss << "0x" << std::hex << hasher(data);
        return oss.str();
    }
    
    std::string generate_hex_stamp(const ShardId& shard_id, TimestampNs timestamp_ns) const {
        std::ostringstream oss;
        oss << shard_id << "_" << std::hex << (timestamp_ns & 0xFFFFFFFF);
        return oss.str();
    }
    
    std::string generate_did_signature(const ShardId& shard_id, const HexStamp& hex_stamp) const {
        return "did:bostrom:cyboquatics:" + shard_id + ":" + hex_stamp;
    }
    
public:
    ShardChainManager() : kernel_ptr_(nullptr) {
        kernel_ptr_ = kernel_new();
        if (kernel_ptr_ == nullptr) {
            throw std::runtime_error("Failed to initialize Rust kernel for shard chain");
        }
        genesis_hash_ = compute_hash("cyboquatics_genesis_" + 
                                     std::to_string(LyapunovResidual::current_timestamp_ns()));
    }
    
    ~ShardChainManager() {
        if (kernel_ptr_ != nullptr) {
            kernel_free(kernel_ptr_);
            kernel_ptr_ = nullptr;
        }
    }
    
    QpuDatashard create_shard(const CorridorId& corridor_id,
                              const KERScore& ker_score,
                              const LyapunovResidual& lyapunov,
                              const std::string& action_data) {
        std::lock_guard<std::mutex> lock(chain_mutex_);
        
        TimestampNs timestamp_ns = LyapunovResidual::current_timestamp_ns();
        ShardId shard_id = "shard_" + std::to_string(timestamp_ns);
        HexStamp hex_stamp = generate_hex_stamp(shard_id, timestamp_ns);
        DidSignature did_signature = generate_did_signature(shard_id, hex_stamp);
        std::string action_hash = compute_hash(action_data);
        
        std::optional<std::string> prev_hash;
        if (!shard_chain_.empty()) {
            prev_hash = compute_hash(shard_chain_.back().shard_id);
        }
        
        QpuDatashard shard;
        shard.shard_id = shard_id;
        shard.hex_stamp = hex_stamp;
        shard.did_signature = did_signature;
        shard.timestamp_ns = timestamp_ns;
        shard.corridor_id = corridor_id;
        shard.ker_snapshot = ker_score;
        shard.lyapunov_snapshot = lyapunov;
        shard.action_hash = action_hash;
        shard.previous_shard_hash = prev_hash;
        
        // Verify chain integrity
        if (prev_hash.has_value() && !shard_chain_.empty()) {
            std::string expected = compute_hash(shard_chain_.back().shard_id);
            if (shard.previous_shard_hash.value() != expected) {
                throw EcosafetyError(EcosafetyErrorCode::ShardChainBroken,
                                    "Chain integrity verification failed");
            }
        }
        
        shard_chain_.push_back(shard);
        
        // Submit to Rust kernel for immutable storage
        std::string json_shard = shard.to_json();
        int result = kernel_execute_safe_action(kernel_ptr_, json_shard.c_str(), corridor_id.c_str());
        if (result != 0) {
            throw EcosafetyError(EcosafetyErrorCode::HardwareDerateTriggered,
                                "Kernel shard submission failed");
        }
        
        return shard;
    }
    
    bool verify_chain_integrity() const {
        std::lock_guard<std::mutex> lock(chain_mutex_);
        for (size_t i = 1; i < shard_chain_.size(); i++) {
            std::string expected = compute_hash(shard_chain_[i-1].shard_id);
            if (!shard_chain_[i].previous_shard_hash.has_value() ||
                shard_chain_[i].previous_shard_hash.value() != expected) {
                return false;
            }
            if (shard_chain_[i].timestamp_ns <= shard_chain_[i-1].timestamp_ns) {
                return false;
            }
        }
        return true;
    }
    
    size_t get_chain_length() const {
        std::lock_guard<std::mutex> lock(chain_mutex_);
        return shard_chain_.size();
    }
    
    std::vector<QpuDatashard> get_shards_since(TimestampNs since_ns) const {
        std::lock_guard<std::mutex> lock(chain_mutex_);
        std::vector<QpuDatashard> result;
        for (const auto& shard : shard_chain_) {
            if (shard.timestamp_ns >= since_ns) {
                result.push_back(shard);
            }
        }
        return result;
    }
    
    void export_audit_log(const std::string& filepath) const {
        std::lock_guard<std::mutex> lock(chain_mutex_);
        std::ofstream file(filepath);
        if (!file.is_open()) {
            throw std::runtime_error("Failed to open audit log file: " + filepath);
        }
        
        file << "[\n";
        for (size_t i = 0; i < shard_chain_.size(); i++) {
            file << "  " << shard_chain_[i].to_json();
            if (i < shard_chain_.size() - 1) {
                file << ",";
            }
            file << "\n";
        }
        file << "]\n";
        file.close();
    }
};

} // namespace audit
} // namespace cyboquatics

// ============================================================================
// NANOSWARM CONTROLLER - Main Hardware Control Interface
// ============================================================================

namespace cyboquatics {
namespace controller {

using namespace types;
using namespace errors;
using namespace invariants;
using namespace hardware;
using namespace audit;

enum class SafetyMode {
    Research,
    Production,
    Emergency
};

struct SafetyModeConfig {
    double ker_k_min;
    double ker_e_min;
    double ker_r_max;
    double lyapunov_threshold;
    std::chrono::milliseconds audit_interval;
    std::string deployment_scope;
    
    static SafetyModeConfig research() {
        return {0.85, 0.85, 0.20, 2.0, std::chrono::milliseconds(5000), "sandbox_only"};
    }
    
    static SafetyModeConfig production() {
        return {constants::KER_K_THRESHOLD, constants::KER_E_THRESHOLD, 
                constants::KER_R_THRESHOLD, constants::LYAPUNOV_THRESHOLD,
                std::chrono::milliseconds(1000), "full_scale"};
    }
    
    static SafetyModeConfig emergency() {
        return {0.95, 0.95, 0.05, 0.5, std::chrono::milliseconds(100), "minimal_safe_subset"};
    }
};

class NanoswarmController {
private:
    EcosafetyKernel* kernel_ptr_;
    std::unique_ptr<ShardChainManager> shard_manager_;
    std::unordered_map<CorridorId, EcosafetyCorridor> corridors_;
    std::unordered_map<CorridorId, NanoswarmCluster> clusters_;
    std::vector<std::unique_ptr<SafetyInvariant>> invariants_;
    SafetyModeConfig safety_config_;
    SafetyMode current_mode_;
    std::atomic<bool> running_{false};
    std::atomic<bool> emergency_stop_{false};
    std::mutex controller_mutex_;
    std::thread audit_thread_;
    std::condition_variable audit_cv_;
    std::mutex audit_mutex_;
    
    void audit_loop() {
        while (running_ && !emergency_stop_) {
            std::unique_lock<std::mutex> lock(audit_mutex_);
            audit_cv_.wait_for(lock, safety_config_.audit_interval, [this]() {
                return !running_ || emergency_stop_;
            });
            
            if (!running_ || emergency_stop_) {
                break;
            }
            
            perform_safety_audit();
        }
    }
    
    void perform_safety_audit() {
        std::lock_guard<std::mutex> lock(controller_mutex_);
        
        for (const auto& [corridor_id, corridor] : corridors_) {
            CorridorCompleteInvariant inv(corridor);
            EcosafetyError err = inv.enforce();
            if (err.code != EcosafetyErrorCode::Success) {
                std::cerr << "AUDIT: Corridor violation detected: " << corridor_id 
                          << " - " << err.message << std::endl;
                trigger_corridor_derate(corridor_id);
            }
            
            auto cluster_it = clusters_.find(corridor_id);
            if (cluster_it != clusters_.end()) {
                if (!cluster_it->second.validate_all_units_safe()) {
                    std::cerr << "AUDIT: Unit safety violation in cluster: " << corridor_id << std::endl;
                    cluster_it->second.derate_cluster();
                }
            }
        }
        
        if (shard_manager_ && !shard_manager_->verify_chain_integrity()) {
            std::cerr << "AUDIT: Shard chain integrity verification failed!" << std::endl;
            trigger_emergency_stop();
        }
    }
    
    void trigger_corridor_derate(const CorridorId& corridor_id) {
        auto it = clusters_.find(corridor_id);
        if (it != clusters_.end()) {
            it->second.derate_cluster();
        }
        auto corr_it = corridors_.find(corridor_id);
        if (corr_it != corridors_.end()) {
            corr_it->second.is_active = false;
        }
    }

public:
    explicit NanoswarmController(SafetyMode mode = SafetyMode::Research)
        : kernel_ptr_(nullptr), current_mode_(mode) {
        
        kernel_ptr_ = kernel_new();
        if (kernel_ptr_ == nullptr) {
            throw EcosafetyError(EcosafetyErrorCode::KernelInitializationFailed,
                                "Failed to initialize Rust Ecosafety Kernel");
        }
        
        shard_manager_ = std::make_unique<ShardChainManager>();
        
        switch (mode) {
            case SafetyMode::Research:
                safety_config_ = SafetyModeConfig::research();
                break;
            case SafetyMode::Production:
                safety_config_ = SafetyModeConfig::production();
                break;
            case SafetyMode::Emergency:
                safety_config_ = SafetyModeConfig::emergency();
                break;
        }
        
        running_ = true;
        emergency_stop_ = false;
        audit_thread_ = std::thread(&NanoswarmController::audit_loop, this);
        
        std::cout << "NanoswarmController initialized in " 
                  << (mode == SafetyMode::Research ? "Research" :
                      mode == SafetyMode::Production ? "Production" : "Emergency")
                  << " mode" << std::endl;
    }
    
    ~NanoswarmController() {
        stop();
        if (kernel_ptr_ != nullptr) {
            kernel_free(kernel_ptr_);
            kernel_ptr_ = nullptr;
        }
    }
    
    void stop() {
        running_ = false;
        audit_cv_.notify_one();
        if (audit_thread_.joinable()) {
            audit_thread_.join();
        }
    }
    
    EcosafetyError register_corridor(const EcosafetyCorridor& corridor) {
        std::lock_guard<std::mutex> lock(controller_mutex_);
        
        if (!corridor.validate_all_dimensions()) {
            return EcosafetyError(EcosafetyErrorCode::CorridorNotDeployable,
                                 "Corridor dimensions exceed safe limits");
        }
        
        if (!corridor.ker_score.is_deployable()) {
            return EcosafetyError(EcosafetyErrorCode::CorridorNotDeployable,
                                 "K/E/R score not deployable: " + corridor.ker_score.to_string());
        }
        
        // Enforce safety mode thresholds
        if (corridor.ker_score.knowledge_factor < safety_config_.ker_k_min ||
            corridor.ker_score.eco_impact < safety_config_.ker_e_min ||
            corridor.ker_score.risk_of_harm > safety_config_.ker_r_max) {
            return EcosafetyError(EcosafetyErrorCode::CorridorNotDeployable,
                                 "K/E/R below safety mode thresholds");
        }
        
        CorridorCompleteInvariant inv(corridor);
        EcosafetyError err = inv.enforce();
        if (err.code != EcosafetyErrorCode::Success) {
            return err;
        }
        
        corridors_[corridor.corridor_id] = corridor;
        clusters_[corridor.corridor_id] = NanoswarmCluster(corridor.corridor_id);
        
        std::cout << "Corridor registered: " << corridor.corridor_id << std::endl;
        return EcosafetyError();
    }
    
    EcosafetyError deploy_nanoswarm(const CorridorId& corridor_id,
                                    uint64_t count,
                                    NanoswarmType type) {
        std::lock_guard<std::mutex> lock(controller_mutex_);
        
        if (emergency_stop_) {
            return EcosafetyError(EcosafetyErrorCode::EmergencyStopActivated,
                                 "Cannot deploy during emergency stop");
        }
        
        auto corr_it = corridors_.find(corridor_id);
        if (corr_it == corridors_.end()) {
            return EcosafetyError(EcosafetyErrorCode::CorridorNotFound,
                                 "Corridor not found: " + corridor_id);
        }
        
        if (!corr_it->second.is_active) {
            return EcosafetyError(EcosafetyErrorCode::InvariantViolation,
                                 "invariant.corridorcomplete", "corridor_inactive");
        }
        
        if (count > constants::NANOSWARM_MAX_COUNT) {
            return EcosafetyError(EcosafetyErrorCode::NanoswarmLimitExceeded,
                                 "Deployment count exceeds maximum limit");
        }
        
        auto cluster_it = clusters_.find(corridor_id);
        if (cluster_it == clusters_.end()) {
            return EcosafetyError(EcosafetyErrorCode::CorridorNotFound,
                                 "Cluster not found for corridor");
        }
        
        // Deploy units
        for (uint64_t i = 0; i < count; i++) {
            NanoswarmUnit unit(cluster_it->second.active_count.load() + i, type);
            if (!cluster_it->second.add_unit(unit)) {
                return EcosafetyError(EcosafetyErrorCode::NanoswarmLimitExceeded,
                                     "Failed to add unit to cluster");
            }
        }
        
        // Create audit shard
        LyapunovResidual lyapunov(LyapunovResidual::current_timestamp_ns(), 0.5, -0.01);
        std::ostringstream action_data;
        action_data << "{\"action\":\"deploy\",\"count\":" << count 
                    << ",\"type\":" << static_cast<int>(type) << "}";
        
        try {
            shard_manager_->create_shard(corridor_id, corr_it->second.ker_score, 
                                         lyapunov, action_data.str());
        } catch (const EcosafetyError& e) {
            return e;
        }
        
        std::cout << "Deployed " << count << " nanoswarm units to " << corridor_id << std::endl;
        return EcosafetyError();
    }
    
    EcosafetyError execute_action(const CorridorId& corridor_id,
                                  const std::string& action_name,
                                  const std::string& action_params) {
        std::lock_guard<std::mutex> lock(controller_mutex_);
        
        if (emergency_stop_) {
            return EcosafetyError(EcosafetyErrorCode::EmergencyStopActivated,
                                 "Cannot execute action during emergency stop");
        }
        
        auto corr_it = corridors_.find(corridor_id);
        if (corr_it == corridors_.end()) {
            return EcosafetyError(EcosafetyErrorCode::CorridorNotFound,
                                 "Corridor not found: " + corridor_id);
        }
        
        if (!corr_it->second.is_active) {
            return EcosafetyError(EcosafetyErrorCode::InvariantViolation,
                                 "invariant.corridorcomplete", "corridor_inactive");
        }
        
        if (!corr_it->second.validate_all_dimensions()) {
            trigger_corridor_derate(corridor_id);
            return EcosafetyError(EcosafetyErrorCode::HardwareDerateTriggered,
                                 "Corridor dimension violation detected");
        }
        
        // Update Lyapunov residual
        LyapunovResidual lyapunov(LyapunovResidual::current_timestamp_ns(), 0.5, -0.01);
        ResidualSafeInvariant residual_inv(lyapunov, safety_config_.lyapunov_threshold);
        EcosafetyError err = residual_inv.enforce();
        if (err.code != EcosafetyErrorCode::Success) {
            return err;
        }
        
        // Create audit shard
        std::ostringstream action_data;
        action_data << "{\"action\":\"" << action_name << "\",\"params\":" << action_params << "}";
        
        try {
            QpuDatashard shard = shard_manager_->create_shard(
                corridor_id, corr_it->second.ker_score, lyapunov, action_data.str());
            std::cout << "Action executed: " << action_name 
                      << " | Shard: " << shard.shard_id << std::endl;
        } catch (const EcosafetyError& e) {
            return e;
        }
        
        return EcosafetyError();
    }
    
    EcosafetyError update_ker_score(const CorridorId& corridor_id,
                                    double new_k, double new_e, double new_r) {
        std::lock_guard<std::mutex> lock(controller_mutex_);
        
        auto corr_it = corridors_.find(corridor_id);
        if (corr_it == corridors_.end()) {
            return EcosafetyError(EcosafetyErrorCode::CorridorNotFound,
                                 "Corridor not found: " + corridor_id);
        }
        
        KERScore new_score{new_k, new_e, new_r};
        
        if (new_score.knowledge_factor < safety_config_.ker_k_min ||
            new_score.eco_impact < safety_config_.ker_e_min ||
            new_score.risk_of_harm > safety_config_.ker_r_max) {
            std::cout << "K/E/R update rejected - below safety mode thresholds: " 
                      << new_score.to_string() << std::endl;
            return EcosafetyError(EcosafetyErrorCode::InvalidKERScore,
                                 "K/E/R below safety mode thresholds");
        }
        
        KerDeployableInvariant inv(new_score);
        EcosafetyError err = inv.enforce();
        if (err.code != EcosafetyErrorCode::Success) {
            return err;
        }
        
        corr_it->second.ker_score = new_score;
        std::cout << "K/E/R updated for " << corridor_id << ": " 
                  << new_score.to_string() << std::endl;
        return EcosafetyError();
    }
    
    void trigger_emergency_stop() {
        std::lock_guard<std::mutex> lock(controller_mutex_);
        emergency_stop_ = true;
        
        for (auto& [corridor_id, cluster] : clusters_) {
            cluster.trigger_emergency_stop();
        }
        
        for (auto& [corridor_id, corridor] : corridors_) {
            corridor.is_active = false;
        }
        
        if (kernel_ptr_ != nullptr) {
            kernel_emergency_stop(kernel_ptr_);
        }
        
        std::cout << "EMERGENCY STOP ACTIVATED - All systems halted" << std::endl;
    }
    
    void export_audit_log(const std::string& filepath) {
        if (shard_manager_) {
            shard_manager_->export_audit_log(filepath);
            std::cout << "Audit log exported to: " << filepath << std::endl;
        }
    }
    
    size_t get_shard_chain_length() const {
        return shard_manager_ ? shard_manager_->get_chain_length() : 0;
    }
    
    SafetyMode get_current_mode() const {
        return current_mode_;
    }
    
    bool is_running() const {
        return running_ && !emergency_stop_;
    }
};

} // namespace controller
} // namespace cyboquatics

// ============================================================================
// PILOT TEMPLATES - Pre-configured Deployment Scenarios
// ============================================================================

namespace cyboquatics {
namespace pilots {

using namespace controller;
using namespace types;
using namespace hardware;

EcosafetyCorridor create_phoenix_mar_corridor() {
    KERScore ker{0.93, 0.92, 0.14};
    std::vector<CorridorDimension> dimensions;
    dimensions.push_back(WaterQualityDimension{0.05, 0.08, 0.10});
    dimensions.push_back(SoilHealthDimension{0.07, 0.06, 0.09});
    return EcosafetyCorridor("phoenix_mar_001", dimensions, ker);
}

EcosafetyCorridor create_airglobe_urban_corridor() {
    KERScore ker{0.91, 0.89, 0.12};
    std::vector<CorridorDimension> dimensions;
    dimensions.push_back(AirQualityDimension{0.08, 0.09, 0.07});
    dimensions.push_back(HabitatSafetyDimension{0.05, 0.04, 0.06});
    return EcosafetyCorridor("airglobe_urban_001", dimensions, ker);
}

EcosafetyCorridor create_wetland_biofilter_corridor() {
    KERScore ker{0.94, 0.93, 0.11};
    std::vector<CorridorDimension> dimensions;
    dimensions.push_back(WaterQualityDimension{0.04, 0.06, 0.08});
    dimensions.push_back(HabitatSafetyDimension{0.03, 0.02, 0.05});
    return EcosafetyCorridor("wetland_biofilter_001", dimensions, ker);
}

} // namespace pilots
} // namespace cyboquatics

// ============================================================================
// MAIN EXECUTION - Example Pilot Run
// ============================================================================

int main(int argc, char* argv[]) {
    std::cout << "============================================" << std::endl;
    std::cout << "Cyboquatics Nanoswarm Controller v1.0.0-alpha" << std::endl;
    std::cout << "============================================" << std::endl;
    
    try {
        // Initialize controller in Research mode
        cyboquatics::controller::NanoswarmController controller(
            cyboquatics::controller::SafetyMode::Research);
        
        // Register Phoenix MAR Corridor
        auto phoenix_corridor = cyboquatics::pilots::create_phoenix_mar_corridor();
        auto err = controller.register_corridor(phoenix_corridor);
        if (err.code != cyboquatics::errors::EcosafetyErrorCode::Success) {
            std::cerr << "Failed to register corridor: " << err.message << std::endl;
            return 1;
        }
        
        // Deploy nanoswarm units
        err = controller.deploy_nanoswarm("phoenix_mar_001", 1000, 
                                          cyboquatics::hardware::NanoswarmType::Filter);
        if (err.code != cyboquatics::errors::EcosafetyErrorCode::Success) {
            std::cerr << "Failed to deploy nanoswarm: " << err.message << std::endl;
            return 1;
        }
        
        // Execute pilot actions
        err = controller.execute_action("phoenix_mar_001", "water_sampling", 
                                        "{\"depth\":5.0,\"location\":\"basin_A\"}");
        if (err.code != cyboquatics::errors::EcosafetyErrorCode::Success) {
            std::cerr << "Action failed: " << err.message << std::endl;
        }
        
        err = controller.execute_action("phoenix_mar_001", "nanoswarm_deploy", 
                                        "{\"count\":1000,\"type\":\"filter\"}");
        if (err.code != cyboquatics::errors::EcosafetyErrorCode::Success) {
            std::cerr << "Action failed: " << err.message << std::endl;
        }
        
        err = controller.execute_action("phoenix_mar_001", "aquifer_recharge", 
                                        "{\"rate\":50.0,\"duration\":3600}");
        if (err.code != cyboquatics::errors::EcosafetyErrorCode::Success) {
            std::cerr << "Action failed: " << err.message << std::endl;
        }
        
        // Update K/E/R based on pilot data
        err = controller.update_ker_score("phoenix_mar_001", 0.94, 0.93, 0.12);
        if (err.code != cyboquatics::errors::EcosafetyErrorCode::Success) {
            std::cerr << "K/E/R update failed: " << err.message << std::endl;
        }
        
        // Export audit log
        controller.export_audit_log("cyboquatics_audit_log.json");
        
        std::cout << "============================================" << std::endl;
        std::cout << "Pilot execution completed successfully" << std::endl;
        std::cout << "Shard chain length: " << controller.get_shard_chain_length() << std::endl;
        std::cout << "============================================" << std::endl;
        
        // Graceful shutdown
        controller.stop();
        
    } catch (const cyboquatics::errors::EcosafetyError& e) {
        std::cerr << "Ecosafety Error: " << e.message << std::endl;
        if (e.is_critical()) {
            std::cerr << "CRITICAL ERROR - System halted" << std::endl;
            return 2;
        }
        return 1;
    } catch (const std::exception& e) {
        std::cerr << "Standard Exception: " << e.what() << std::endl;
        return 1;
    }
    
    return 0;
}

// ============================================================================
// END OF FILE: cyboquatics_hardware/src/nanoswarm_controller.cpp
// ============================================================================
