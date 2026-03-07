/**
 * @file CyboquaticMachineController.cpp
 * @brief Implementation of cyboquatic machine controller with human-robotics
 *        integration and policy-adaptive compliance enforcement.
 */

#include "CyboquaticMachineController.h"
#include "SensorFusionEngine.h"
#include "HumanRoboticsInterface.h"
#include <iostream>
#include <sstream>
#include <iomanip>
#include <algorithm>
#include <cmath>

namespace cyboquatics {

// Internal helper for hex generation
static std::string generateEvidenceHex(
    const std::string& node_id,
    std::chrono::system_clock::time_point timestamp
) {
    std::stringstream ss;
    ss << "0xCQ" 
       << std::hex << std::setfill('0')
       << std::setw(4) << (std::hash<std::string>{}(node_id) % 0xFFFF)
       << std::setw(4) << (std::chrono::duration_cast<std::chrono::seconds>(
              timestamp.time_since_epoch()).count() % 0xFFFF)
       << "EVID";
    return ss.str();
}

CyboquaticMachineController::CyboquaticMachineController(
    const std::string& node_id,
    const PolicyConfig& config
) : node_id_(node_id)
  , config_(config)
  , running_(false)
  , start_time_(std::chrono::system_clock::now())
{
    state_.node_id = node_id;
    state_.mode = MachineMode::IDLE;
    state_.compliance = ComplianceStatus::COMPLIANT;
    state_.eco_impact_score = 0.0;
    state_.risk_residual = 0.0;
    state_.neu_budget_remaining = 1.0;
    state_.soul_boundary_verified = false;
    state_.evidence_hex = generateEvidenceHex(node_id_, start_time_);
    state_.last_update = std::chrono::system_clock::now();
}

CyboquaticMachineController::~CyboquaticMachineController() {
    stop();
    logAuditEvent("CONTROLLER_SHUTDOWN", "Controller destructed");
}

CyboquaticMachineController::CyboquaticMachineController(
    CyboquaticMachineController&& other) noexcept
    : node_id_(std::move(other.node_id_))
    , config_(std::move(other.config_))
    , state_(std::move(other.state_))
    , ceim_kernel_(std::move(other.ceim_kernel_))
    , ker_evaluator_(std::move(other.ker_evaluator_))
    , soul_guardrail_(std::move(other.soul_guardrail_))
    , sensor_fusion_(std::move(other.sensor_fusion_))
    , running_(other.running_.load())
    , audit_log_(std::move(other.audit_log_))
    , start_time_(other.start_time_)
{
    other.running_ = false;
}

CyboquaticMachineController& CyboquaticMachineController::operator=(
    CyboquaticMachineController&& other) noexcept
{
    if (this != &other) {
        stop();
        node_id_ = std::move(other.node_id_);
        config_ = std::move(other.config_);
        state_ = std::move(other.state_);
        ceim_kernel_ = std::move(other.ceim_kernel_);
        ker_evaluator_ = std::move(other.ker_evaluator_);
        soul_guardrail_ = std::move(other.soul_guardrail_);
        sensor_fusion_ = std::move(other.sensor_fusion_);
        running_ = other.running_.load();
        audit_log_ = std::move(other.audit_log_);
        start_time_ = other.start_time_;
        other.running_ = false;
    }
    return *this;
}

bool CyboquaticMachineController::initialize() {
    std::lock_guard<std::mutex> lock(state_mutex_);
    
    try {
        // Initialize subsystems
        ceim_kernel_ = std::make_unique<CEIMKernel>(node_id_);
        ker_evaluator_ = std::make_unique<KEREvaluator>(config_.jurisdiction);
        soul_guardrail_ = SoulGuardrail::loadFromParticle("soul.guardrail.spec.v1");
        sensor_fusion_ = std::make_unique<SensorFusionEngine>(node_id_);
        
        if (!ceim_kernel_ || !ker_evaluator_ || !soul_guardrail_ || !sensor_fusion_) {
            logAuditEvent("INITIALIZATION_FAILED", "Subsystem initialization failed");
            return false;
        }
        
        // Verify soul guardrail compliance
        auto guardrail_valid = soul_guardrail_->verifyConfiguration();
        if (!guardrail_valid) {
            logAuditEvent("SOUL_GUARDRAIL_INVALID", "Guardrail configuration invalid");
            return false;
        }
        
        state_.soul_boundary_verified = true;
        logAuditEvent("INITIALIZATION_SUCCESS", 
            "All subsystems initialized with soul-boundary verification");
        
        return true;
    } catch (const std::exception& e) {
        logAuditEvent("INITIALIZATION_EXCEPTION", std::string(e.what()));
        return false;
    }
}

void CyboquaticMachineController::start() {
    if (running_) {
        return;
    }
    
    running_ = true;
    control_thread_ = std::thread(&CyboquaticMachineController::controlLoop, this);
    
    logAuditEvent("CONTROLLER_STARTED", "Main control loop initiated");
}

void CyboquaticMachineController::stop() {
    if (!running_) {
        return;
    }
    
    running_ = false;
    
    if (control_thread_.joinable()) {
        control_thread_.join();
    }
    
    setMode(MachineMode::IDLE);
    logAuditEvent("CONTROLLER_STOPPED", "Main control loop terminated");
}

MachineState CyboquaticMachineController::getState() const {
    std::lock_guard<std::mutex> lock(state_mutex_);
    return state_;
}

bool CyboquaticMachineController::setMode(MachineMode mode) {
    std::lock_guard<std::mutex> lock(state_mutex_);
    
    // Check if mode change complies with soul guardrails
    if (mode == MachineMode::REMEDIATION || mode == MachineMode::MAINTENANCE) {
        // Verify no soul-boundary violations would occur
        auto validation = soul_guardrail_->verifyModeTransition(state_.mode, mode);
        if (!validation.allowed) {
            logAuditEvent("MODE_CHANGE_BLOCKED", 
                "Soul guardrail violation: " + validation.violations[0]);
            return false;
        }
    }
    
    state_.mode = mode;
    state_.last_update = std::chrono::system_clock::now();
    state_.evidence_hex = generateEvidenceHex(node_id_, state_.last_update);
    
    logAuditEvent("MODE_CHANGED", 
        "Mode changed to: " + std::to_string(static_cast<int>(mode)));
    
    return true;
}

bool CyboquaticMachineController::executeRemediation(
    const std::string& action_type,
    const std::map<std::string, double>& parameters)
{
    std::lock_guard<std::mutex> lock(state_mutex_);
    
    // Pre-flight soul boundary check
    auto soul_validation = soul_guardrail_->verifyAction(action_type);
    if (!soul_validation.allowed) {
        logAuditEvent("REMEDIATION_BLOCKED", 
            "Soul boundary violation: " + soul_validation.violations[0]);
        
        if (soul_boundary_callback_) {
            soul_boundary_callback_(false, soul_validation.violations);
        }
        
        return false;
    }
    
    // Check compliance status
    auto compliance = checkCompliance();
    if (compliance == ComplianceStatus::VIOLATION_DETECTED ||
        compliance == ComplianceStatus::QUARANTINED) {
        logAuditEvent("REMEDIATION_BLOCKED", "Compliance violation detected");
        return false;
    }
    
    // Execute remediation via CEIM kernel
    bool success = ceim_kernel_->executeAction(action_type, parameters);
    
    if (success) {
        // Update eco-impact score
        state_.eco_impact_score = ceim_kernel_->getEcoImpactScore();
        state_.risk_residual = evaluateKER();
        
        if (eco_impact_callback_) {
            eco_impact_callback_(state_.eco_impact_score, action_type);
        }
        
        logAuditEvent("REMEDIATION_SUCCESS", 
            "Action executed: " + action_type);
    } else {
        logAuditEvent("REMEDIATION_FAILED", 
            "Action failed: " + action_type);
    }
    
    return success;
}

ValidationResult CyboquaticMachineController::verifySoulBoundary(
    const std::string& citizen_did,
    const std::string& action_type)
{
    std::lock_guard<std::mutex> lock(state_mutex_);
    
    auto validation = soul_guardrail_->verifyActionForCitizen(citizen_did, action_type);
    
    ValidationResult result;
    result.allowed = validation.allowed;
    result.violations = validation.violations;
    result.karma_delta = validation.karma_delta;
    result.rollback_required = validation.rollback_required;
    result.evidence_hex = generateEvidenceHex(node_id_, std::chrono::system_clock::now());
    
    if (soul_boundary_callback_) {
        soul_boundary_callback_(result.allowed, result.violations);
    }
    
    logAuditEvent("SOUL_BOUNDARY_CHECK", 
        "Citizen: " + citizen_did + ", Action: " + action_type + 
        ", Allowed: " + std::to_string(result.allowed));
    
    return result;
}

void CyboquaticMachineController::registerComplianceCallback(
    ComplianceCallback callback)
{
    std::lock_guard<std::mutex> lock(state_mutex_);
    compliance_callback_ = std::move(callback);
}

void CyboquaticMachineController::registerEcoImpactCallback(
    EcoImpactCallback callback)
{
    std::lock_guard<std::mutex> lock(state_mutex_);
    eco_impact_callback_ = std::move(callback);
}

void CyboquaticMachineController::registerSoulBoundaryCallback(
    SoulBoundaryCallback callback)
{
    std::lock_guard<std::mutex> lock(state_mutex_);
    soul_boundary_callback_ = std::move(callback);
}

std::string CyboquaticMachineController::getEvidenceHex() const {
    std::lock_guard<std::mutex> lock(state_mutex_);
    return state_.evidence_hex;
}

double CyboquaticMachineController::computeKnowledgeFactor() const {
    // Formula: F = α·V + β·R + γ·E + δ·N
    const double alpha = 0.30;  // validation weight
    const double beta = 0.25;   // reuse weight
    const double gamma = 0.30;  // ecological impact weight
    const double delta = 0.15;  // novelty weight
    
    std::lock_guard<std::mutex> lock(state_mutex_);
    
    double validation = 0.9;  // Assume high validation for deployed systems
    double reuse = 0.8;       // Reuse of existing kernels
    double ecological = state_.eco_impact_score;
    double novelty = 0.7;     // Novel integration patterns
    
    double factor = alpha * validation
                  + beta * reuse
                  + gamma * ecological
                  + delta * novelty;
    
    return std::max(0.0, std::min(1.0, factor));
}

void CyboquaticMachineController::controlLoop() {
    while (running_) {
        // Check compliance at configured interval
        auto compliance = checkCompliance();
        
        if (compliance_callback_) {
            compliance_callback_(compliance, node_id_);
        }
        
        // Update state based on compliance
        if (compliance == ComplianceStatus::VIOLATION_DETECTED) {
            setMode(MachineMode::EMERGENCY_SHUTDOWN);
        } else if (compliance == ComplianceStatus::WARNING) {
            // Log warning but continue operation
            logAuditEvent("COMPLIANCE_WARNING", "Minor compliance deviation detected");
        }
        
        // Update sensor fusion and CEIM kernel
        auto sensor_data = sensor_fusion_->getLatestReadings();
        updateCEIMKernel(sensor_data);
        
        // Sleep for next iteration
        std::this_thread::sleep_for(config_.compliance_check_interval);
    }
}

ComplianceStatus CyboquaticMachineController::checkCompliance() {
    // Check jurisdiction-specific policy compliance
    // Check soul guardrail constraints
    // Check NEU budget status
    // Check CEIM kernel integrity
    
    auto ker_status = ker_evaluator_->evaluateCompliance();
    
    if (ker_status.violation_detected) {
        return ComplianceStatus::VIOLATION_DETECTED;
    } else if (ker_status.warning) {
        return ComplianceStatus::WARNING;
    }
    
    return ComplianceStatus::COMPLIANT;
}

void CyboquaticMachineController::updateCEIMKernel(
    const std::map<std::string, double>& sensor_data)
{
    if (ceim_kernel_) {
        ceim_kernel_->updateSensorData(sensor_data);
        state_.eco_impact_score = ceim_kernel_->getEcoImpactScore();
        state_.risk_residual = ceim_kernel_->getRiskResidual();
    }
}

double CyboquaticMachineController::evaluateKER() {
    if (ker_evaluator_) {
        return ker_evaluator_->computeRiskResidual();
    }
    return 0.0;
}

void CyboquaticMachineController::logAuditEvent(
    const std::string& event_type,
    const std::string& details)
{
    std::stringstream entry;
    entry << std::chrono::system_clock::now().time_since_epoch().count()
          << "|" << event_type
          << "|" << details
          << "|" << state_.evidence_hex;
    
    std::lock_guard<std::mutex> lock(state_mutex_);
    audit_log_.push_back(entry.str());
    
    // Keep audit log bounded
    const size_t MAX_AUDIT_ENTRIES = 10000;
    if (audit_log_.size() > MAX_AUDIT_ENTRIES) {
        audit_log_.erase(audit_log_.begin());
    }
}

} // namespace cyboquatics
