/**
 * Cyboquatic Android Controller Interface
 * 
 * Provides mobile monitoring and manual override capabilities for Cyboquatic
 * industrial machinery. Enforces client-side safety mirrors of the Rust ecosafety
 * spine, ensuring no command is sent without a corresponding risk estimate.
 * 
 * Safety Guarantees:
 * - Client-side Lyapunov validation before transmission
 * - Read-only mode enforced if KER metrics drop below thresholds
 * - Carbon-negative operation visualization and prioritization
 * - No actuation without risk vector (type-enforced via data classes)
 * 
 * @file CyboquaticController.kt
 * @destination cyboquatic-android-controller/src/CyboquaticController.kt
 * @language Kotlin/Android
 * @compatibility rust-core >= 1.0, aln-config >= 1.0
 */

package com.cyboquatic.controller

import android.util.Log
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.time.Instant
import java.util.UUID
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.sqrt

// ============================================================================
// CONSTANTS & CONFIGURATION (Mirrored from ALN Schema)
// ============================================================================

object CyboConfig {
    const val K_THRESHOLD_DEPLOY: Double = 0.90
    const val E_THRESHOLD_DEPLOY: Double = 0.90
    const val R_THRESHOLD_DEPLOY: Double = 0.13
    const val LYAPUNOV_EPSILON: Double = 0.001
    const val MAX_RISK_PLANES: Int = 8
    const val TAG: String = "CyboquaticController"
}

// ============================================================================
// DATA MODELS (Immutable State)
// ============================================================================

/**
 * Risk Plane Enumeration (Matches Rust RiskPlane)
 */
enum class RiskPlane(val index: Int, val label: String) {
    ENERGY(0, "Energy"),
    HYDRAULIC(1, "Hydraulic"),
    BIOLOGY(2, "Biology"),
    CARBON(3, "Carbon"),
    MATERIALS(4, "Materials"),
    THERMAL(5, "Thermal"),
    MECHANICAL(6, "Mechanical"),
    SENSOR_CALIBRATION(7, "SensorCalibration")
}

/**
 * Immutable Risk Vector (Mirrors Rust RiskVector)
 */
data class RiskVector(
    val coordinates: DoubleArray, // Size 8, values 0.0-1.0
    val timestamp: Long,
    val validated: Boolean = true
) {
    init {
        require(coordinates.size == CyboConfig.MAX_RISK_PLANES) { "RiskVector must have 8 planes" }
        require(coordinates.all { it in 0.0..1.0 }) { "Risk coordinates must be in [0.0, 1.0]" }
    }

    fun maxCoordinate(): Double = coordinates.maxOrNull() ?: 0.0

    fun lyapunovResidual(weights: DoubleArray): Double {
        require(weights.size == CyboConfig.MAX_RISK_PLANES)
        return coordinates.indices.sumOf { i -> weights[i] * coordinates[i].pow(2) }
    }
}

/**
 * KER Governance Metrics (Mirrors Rust KERMetrics)
 */
data class KERMetrics(
    val knowledgeFactor: Double,
    val ecoImpact: Double,
    val riskOfHarm: Double,
    val deployable: Boolean
) {
    init {
        require(knowledgeFactor in 0.0..1.0)
        require(ecoImpact in 0.0..1.0)
        require(riskOfHarm in 0.0..1.0)
    }

    companion object {
        fun fromValues(k: Double, e: Double, r: Double): KERMetrics {
            val deployable = (k >= CyboConfig.K_THRESHOLD_DEPLOY) &&
                    (e >= CyboConfig.E_THRESHOLD_DEPLOY) &&
                    (r <= CyboConfig.R_THRESHOLD_DEPLOY)
            return KERMetrics(k, e, r, deployable)
        }
    }
}

/**
 * System State Snapshot (Mirrors Rust SystemState)
 */
data class CyboquaticState(
    val nodeId: String,
    val currentVT: Double,
    val previousVT: Double,
    val currentRisk: RiskVector,
    val energySurplus: Double,
    val mode: OperatingMode,
    val kerMetrics: KERMetrics,
    val carbonState: CarbonState,
    val timestamp: Long
)

enum class OperatingMode {
    IDLE, NORMAL, ECO_RESTORATIVE, CARBON_NEGATIVE, MAINTENANCE, EMERGENCY
}

enum class CarbonState {
    NEGATIVE, NEUTRAL, POSITIVE
}

// ============================================================================
// COMMAND STRUCTURES (Safety-Enforced)
// ============================================================================

/**
 * Manual Override Request
 * 
 * Requires a proposed risk vector to be submitted with any actuation command.
 * This enforces the "no action without risk estimate" rule at the API level.
 */
data class ManualOverrideRequest(
    val requestId: String,
    val command: ControlCommand,
    val proposedRiskVector: RiskVector,
    val justification: String,
    val operatorId: String
)

sealed class ControlCommand {
    data class SetMode(val targetMode: OperatingMode) : ControlCommand()
    data class AdjustParameter(val plane: RiskPlane, val delta: Double) : ControlCommand()
    object EmergencyStop : ControlCommand()
}

/**
 * Validation Result for Commands
 */
data class CommandValidation(
    val accepted: Boolean,
    val reason: String,
    val predictedVT: Double? = null
)

// ============================================================================
// CLIENT-SIDE SAFETY ENFORCER
// ============================================================================

/**
 * Client-side mirror of the Rust EcosafetyEnforcer.
 * Performs preliminary validation before sending commands to the machinery.
 * Note: Final enforcement happens on the Rust core; this is a UX safety layer.
 */
class ClientSafetyEnforcer(
    private val weights: DoubleArray,
    private val epsilon: Double = CyboConfig.LYAPUNOV_EPSILON
) {

    private var currentVT: Double = 0.0

    fun updateState(newVT: Double) {
        currentVT = newVT
    }

    /**
     * Validates a manual override request locally.
     * Returns CommandValidation indicating if it's safe to transmit.
     */
    fun validateRequest(request: ManualOverrideRequest, currentState: CyboquaticState): CommandValidation {
        // 1. Check Risk Vector Integrity
        if (!request.proposedRiskVector.validated) {
            return CommandValidation(false, "Risk vector validation failed")
        }

        // 2. Check Corridor Bounds (Hard Limit)
        val maxRisk = request.proposedRiskVector.maxCoordinate()
        if (maxRisk >= 1.0) {
            return CommandValidation(false, "Risk coordinate exceeds hard limit (1.0)")
        }

        // 3. Check Lyapunov Stability Invariant
        val proposedVT = request.proposedRiskVector.lyapunovResidual(weights)
        if (proposedVT > currentVT + epsilon) {
            return CommandValidation(
                false, 
                "Lyapunov violation: V_t would increase from $currentVT to $proposedVT",
                predictedVT = proposedVT
            )
        }

        // 4. Check Governance Thresholds (Prevent override if system unstable)
        if (!currentState.kerMetrics.deployable) {
            if (request.command !is ControlCommand.EmergencyStop) {
                return CommandValidation(false, "System not deployable (KER thresholds breached)")
            }
        }

        return CommandValidation(true, "Client-side validation passed", predictedVT = proposedVT)
    }
}

// ============================================================================
// VIEWMODEL (Business Logic)
// ============================================================================

/**
 * Main ViewModel for Cyboquatic Android Controller.
 * Handles state observation, command submission, and safety enforcement.
 */
class CyboquaticViewModel(
    private val safetyEnforcer: ClientSafetyEnforcer,
    private val repository: CyboquaticRepository // Abstracted network/IPC layer
) {
    private val _state = MutableStateFlow<CyboquaticState?>(null)
    val state: StateFlow<CyboquaticState?> = _state.asStateFlow()

    private val _validationStatus = MutableStateFlow<CommandValidation?>(null)
    val validationStatus: StateFlow<CommandValidation?> = _validationStatus.asStateFlow()

    private val commandMutex = Mutex()

    /**
     * Updates internal state from remote machinery
     */
    fun onStateUpdate(newState: CyboquaticState) {
        _state.value = newState
        safetyEnforcer.updateState(newState.currentVT)
    }

    /**
     * Submits a manual override request with safety checks.
     */
    suspend fun submitOverride(request: ManualOverrideRequest) {
        commandMutex.withLock {
            val currentState = _state.value ?: run {
                _validationStatus.value = CommandValidation(false, "No current state available")
                return
            }

            // 1. Client-Side Validation
            val validation = safetyEnforcer.validateRequest(request, currentState)
            _validationStatus.value = validation

            if (!validation.accepted) {
                Log.w(CyboConfig.TAG, "Command rejected locally: ${validation.reason}")
                return
            }

            // 2. Transmit to Rust Core (Final enforcement happens there)
            try {
                repository.sendCommand(request)
                Log.i(CyboConfig.TAG, "Command transmitted: ${request.requestId}")
            } catch (e: Exception) {
                Log.e(CyboConfig.TAG, "Transmission failed", e)
                _validationStatus.value = CommandValidation(false, "Transmission error: ${e.message}")
            }
        }
    }

    /**
     * Generates a safe parameter adjustment request.
     */
    fun createAdjustmentRequest(plane: RiskPlane, delta: Double, operatorId: String): ManualOverrideRequest {
        val currentState = _state.value ?: throw IllegalStateException("No state available")
        
        // Construct proposed risk vector (simple mirror for adjustment)
        // In production, this would come from a local simulation preview
        val proposedCoords = currentState.currentRisk.coordinates.copyOf()
        // Apply delta safely (clamped)
        val idx = plane.index
        proposedCoords[idx] = (proposedCoords[idx] + delta).coerceIn(0.0, 1.0)

        return ManualOverrideRequest(
            requestId = UUID.randomUUID().toString(),
            command = ControlCommand.AdjustParameter(plane, delta),
            proposedRiskVector = RiskVector(proposedCoords, System.currentTimeMillis()),
            justification = "Manual adjustment via Android Controller",
            operatorId = operatorId
        )
    }

    /**
     * Checks if UI should be locked (Read-Only Mode)
     */
    fun isReadOnlyMode(): Boolean {
        val currentState = _state.value ?: return true
        // Lock if KER metrics are critical or mode is Emergency
        return !currentState.kerMetrics.deployable || currentState.mode == OperatingMode.EMERGENCY
    }
}

// ============================================================================
// REPOSITORY INTERFACE (Network/IPC Abstraction)
// ============================================================================

interface CyboquaticRepository {
    /**
     * Sends a validated command to the Rust core.
     * Throws exception on transmission failure.
     */
    suspend fun sendCommand(request: ManualOverrideRequest)

    /**
     * Observes state updates from the machinery.
     */
    fun observeState(): Flow<CyboquaticState>
}

// ============================================================================
// UI STATE UTILITIES (For Compose/XML Rendering)
// ============================================================================

object UiUtils {

    /**
     * Returns a color code for risk levels (Green/Yellow/Red)
     */
    fun getRiskColor(riskValue: Double): Int {
        return when {
            riskValue < 0.30 -> 0xFF00FF00.toInt() // Green
            riskValue < 0.70 -> 0xFFFFFF00.toInt() // Yellow
            else -> 0xFFFF0000.toInt()             // Red
        }
    }

    /**
     * Formats carbon state for display
     */
    fun formatCarbonState(state: CarbonState): String {
        return when (state) {
            CarbonState.NEGATIVE -> "Carbon Negative (Restorative)"
            CarbonState.NEUTRAL -> "Carbon Neutral"
            CarbonState.POSITIVE -> "Carbon Positive (Warning)"
        }
    }

    /**
     * Calculates overall system health score (0-100)
     */
    fun calculateHealthScore(metrics: KERMetrics): Int {
        val kScore = metrics.knowledgeFactor * 40
        val eScore = metrics.ecoImpact * 40
        val rScore = (1.0 - metrics.riskOfHarm) * 20
        return (kScore + eScore + rScore).toInt()
    }
}

// ============================================================================
// LOGGING & AUDIT (Client-Side)
// ============================================================================

object ClientAuditLog {
    private val logs = mutableListOf<String>()

    fun log(event: String, details: String) {
        val timestamp = Instant.now().toString()
        val entry = "[$timestamp] $event: $details"
        logs.add(entry)
        Log.d(CyboConfig.TAG, entry)
        // In production: Persist to secure local storage
    }

    fun logCommandSubmission(request: ManualOverrideRequest, validation: CommandValidation) {
        log(
            "COMMAND_SUBMISSION",
            "ID=${request.requestId}, Accepted=${validation.accepted}, Reason=${validation.reason}"
        )
    }
}
