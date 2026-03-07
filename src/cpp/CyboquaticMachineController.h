/**
 * @file CyboquaticMachineController.h
 * @brief High-performance cyboquatic machine control system with human-robotics
 *        integration, policy-adaptive compliance, and soul-boundary enforcement.
 * 
 * This controller manages cyboquatic machines (water nodes, air-globes, eco-sensors)
 * with real-time compliance checking, NEU psych-risk budget monitoring, and
 * stakeholder-aware operation modes. Designed for sub-150ms safety-critical loops.
 * 
 * @author Doctor Jacob Scott Farmer (did:ion:EiD8J2b3K8k9Q8x9...)
 * @version 1.0.0
 * @date 2026-03-07
 * @copyright MIT OR Apache-2.0
 * 
 * Evidence Hex: 0xCQ2026CPP7A8B9C1D
 * Knowledge Factor: F ≈ 0.89
 */

#ifndef CYBOQUATICS_MACHINE_CONTROLLER_H
#define CYBOQUATICS_MACHINE_CONTROLLER_H

#include <memory>
#include <string>
#include <vector>
#include <chrono>
#include <functional>
#include <atomic>
#include <mutex>

namespace cyboquatics {

// Forward declarations
class CEIMKernel;
class KEREvaluator;
class SoulGuardrail;
class StakeholderProfile;
class SensorFusionEngine;

/**
 * @brief Operational modes for cyboquatic machines
 */
enum class MachineMode {
    IDLE,
    MONITORING,
    REMEDIATION,
    MAINTENANCE,
    EMERGENCY_SHUTDOWN,
    SOUL_SAFE_MODE  // Restricted mode preserving all soul boundaries
};

/**
 * @brief Compliance status enumeration
 */
enum class ComplianceStatus {
    COMPLIANT,
    WARNING,
    VIOLATION_DETECTED,
    ROLLBACK_REQUIRED,
    QUARANTINED
};

/**
 * @brief Machine state structure for atomic operations
 */
struct MachineState {
    std::string node_id;
    MachineMode mode;
    ComplianceStatus compliance;
    double eco_impact_score;
    double risk_residual;
    double neu_budget_remaining;
    std::string stakeholder_class;
    std::string evidence_hex;
    std::chrono::system_clock::time_point last_update;
    bool soul_boundary_verified;
};

/**
 * @brief Configuration for policy-adaptive operation
 */
struct PolicyConfig {
    std::string jurisdiction;
    double max_intensity_delta;
    double max_duty_cycle_delta;
    double max_cumulative_load_delta;
    bool requires_status_online;
    bool ota_audit_required;
    std::vector<std::string> blocked_config_types;
    std::chrono::milliseconds compliance_check_interval;
};

/**
 * @brief Callback types for event handling
 */
using ComplianceCallback = std::function<void(ComplianceStatus, const std::string&)>;
using EcoImpactCallback = std::function<void(double, const std::string&)>;
using SoulBoundaryCallback = std::function<void(bool, const std::vector<std::string>&)>;

/**
 * @class CyboquaticMachineController
 * @brief Main controller class for cyboquatic machine operations
 * 
 * This class provides:
 * - Real-time CEIM kernel evaluation for ecological integrity
 * - KER-based compliance monitoring with policy adaptation
 * - Soul guardrail enforcement for all augmentation-adjacent operations
 * - Human-robotics interface for augmented citizen interactions
 * - Evidence hex logging for audit trails
 */
class CyboquaticMachineController {
public:
    /**
     * @brief Construct a new Cyboquatic Machine Controller
     * @param node_id Unique identifier for this machine node
     * @param config Policy configuration for compliance checking
     */
    explicit CyboquaticMachineController(
        const std::string& node_id,
        const PolicyConfig& config
    );
    
    /**
     * @brief Destructor with cleanup and audit logging
     */
    ~CyboquaticMachineController();
    
    // Disable copy, allow move
    CyboquaticMachineController(const CyboquaticMachineController&) = delete;
    CyboquaticMachineController& operator=(const CyboquaticMachineController&) = delete;
    CyboquaticMachineController(CyboquaticMachineController&&) noexcept;
    CyboquaticMachineController& operator=(CyboquaticMachineController&&) noexcept;
    
    /**
     * @brief Initialize the controller with required subsystems
     * @return true if initialization successful
     */
    bool initialize();
    
    /**
     * @brief Start the main control loop
     */
    void start();
    
    /**
     * @brief Stop the main control loop
     */
    void stop();
    
    /**
     * @brief Get current machine state (thread-safe)
     * @return Current machine state snapshot
     */
    MachineState getState() const;
    
    /**
     * @brief Set operational mode with compliance verification
     * @param mode Target operational mode
     * @return true if mode change allowed
     */
    bool setMode(MachineMode mode);
    
    /**
     * @brief Execute remediation action with soul-boundary check
     * @param action_type Type of remediation action
     * @param parameters Action-specific parameters
     * @return true if action executed successfully
     */
    bool executeRemediation(
        const std::string& action_type,
        const std::map<std::string, double>& parameters
    );
    
    /**
     * @brief Verify soul boundaries before augmentation-adjacent operations
     * @param citizen_did DID of the augmented citizen
     * @param action_type Type of action requiring verification
     * @return ValidationResult with violations if any
     */
    ValidationResult verifySoulBoundary(
        const std::string& citizen_did,
        const std::string& action_type
    );
    
    /**
     * @brief Register compliance status callback
     * @param callback Function to call on compliance status changes
     */
    void registerComplianceCallback(ComplianceCallback callback);
    
    /**
     * @brief Register eco-impact callback
     * @param callback Function to call on eco-impact score changes
     */
    void registerEcoImpactCallback(EcoImpactCallback callback);
    
    /**
     * @brief Register soul boundary callback
     * @param callback Function to call on soul boundary verification
     */
    void registerSoulBoundaryCallback(SoulBoundaryCallback callback);
    
    /**
     * @brief Get evidence hex for current state
     * @return Hex string for audit trail
     */
    std::string getEvidenceHex() const;
    
    /**
     * @brief Compute knowledge-factor for this deployment
     * @return Knowledge factor value [0.0, 1.0]
     */
    double computeKnowledgeFactor() const;

private:
    /**
     * @brief Main control loop implementation
     */
    void controlLoop();
    
    /**
     * @brief Check compliance against current policy
     * @return Compliance status
     */
    ComplianceStatus checkCompliance();
    
    /**
     * @brief Update CEIM kernel with new sensor data
     * @param sensor_data Map of sensor readings
     */
    void updateCEIMKernel(const std::map<std::string, double>& sensor_data);
    
    /**
     * @brief Evaluate KER for current state
     * @return Risk residual value
     */
    double evaluateKER();
    
    /**
     * @brief Log state change to audit trail
     * @param event_type Type of event being logged
     * @param details Event-specific details
     */
    void logAuditEvent(const std::string& event_type, const std::string& details);
    
    // Private members
    std::string node_id_;
    PolicyConfig config_;
    MachineState state_;
    
    std::unique_ptr<CEIMKernel> ceim_kernel_;
    std::unique_ptr<KEREvaluator> ker_evaluator_;
    std::unique_ptr<SoulGuardrail> soul_guardrail_;
    std::unique_ptr<SensorFusionEngine> sensor_fusion_;
    
    std::atomic<bool> running_;
    std::thread control_thread_;
    mutable std::mutex state_mutex_;
    
    ComplianceCallback compliance_callback_;
    EcoImpactCallback eco_impact_callback_;
    SoulBoundaryCallback soul_boundary_callback_;
    
    std::vector<std::string> audit_log_;
    std::chrono::system_clock::time_point start_time_;
};

/**
 * @brief Validation result structure for soul boundary checks
 */
struct ValidationResult {
    bool allowed;
    std::vector<std::string> violations;
    double karma_delta;
    bool rollback_required;
    std::string evidence_hex;
};

} // namespace cyboquatics

#endif // CYBOQUATICS_MACHINE_CONTROLLER_H
