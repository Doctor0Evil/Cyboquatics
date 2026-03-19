// ============================================================================
// Cyboquatic Kernel - C++ Hydraulics and Transport Numerics
// ============================================================================
// Version: 1.0.0
// License: Apache-2.0 OR MIT
// Authors: Cyboquatic Research Collective
//
// This header defines the numerical computation interface for Cyboquatic
// hydraulic and transport modeling. All safety logic (corridor bands,
// risk coordinates, Lyapunov residuals) is defined in Rust/ALN. This
// C++ kernel only performs numerics and emits data shards for Rust
// safety evaluation.
//
// Key Design Principles:
// - No safety logic in C++ (all gates in Rust/ALN)
// - Mirror Rust corridor structures for consistency
// - Emit structured data for Rust kernel evaluation
// - Support 20-50 year numerical stability
// - ISO/IEC compliance for scientific computing
//
// Continuity Guarantee: All numerical outputs are cryptographically
// hashed and linked to Rust safety evaluations. Model error is bounded
// and tracked over the system lifespan.
// ============================================================================

#ifndef CYBOQUATIC_KERNEL_HPP
#define CYBOQUATIC_KERNEL_HPP

#include <vector>
#include <string>
#include <cstdint>
#include <cmath>
#include <optional>
#include <chrono>
#include <functional>

// ============================================================================
// Version Information
// ============================================================================

namespace cyboquatic {
namespace kernel {

/// Kernel version major.
constexpr int VERSION_MAJOR = 1;

/// Kernel version minor.
constexpr int VERSION_MINOR = 0;

/// Kernel version patch.
constexpr int VERSION_PATCH = 0;

/// Kernel version string.
inline const char* version_string() {
    return "1.0.0";
}

} // namespace kernel
} // namespace cyboquatic

// ============================================================================
// Reach State (Hydraulic System State)
// ============================================================================

namespace cyboquatic {

/// Complete state of a hydraulic reach segment.
///
/// This structure captures all measurable physical properties of a
/// single reach segment in the Cyboquatic hydraulic network. Values
/// are computed by numerical integration and passed to Rust for
/// safety evaluation.
///
/// # Continuity Note
/// For 20-50 year operation, all state values should be validated
/// against physical bounds before use. NaN and infinity values must
/// be detected and handled gracefully.
struct ReachState {
    /// Flow rate in cubic meters per second.
    double q_m3s;

    /// Cross-sectional area in square meters.
    double area_m2;

    /// Concentration of target substance (kg per m³).
    double c_kg_per_m3;

    /// Temperature in degrees Celsius.
    double temp_c;

    /// Time to 90% biodegradation (days).
    double t90_days;

    /// Water depth in meters.
    double depth_m;

    /// Velocity in meters per second.
    double velocity_mps;

    /// Hydraulic radius in meters.
    double hydraulic_radius_m;

    /// Manning's roughness coefficient.
    double manning_n;

    /// Bed slope (dimensionless).
    double bed_slope;

    /// Dissolved oxygen concentration (mg/L).
    double do_mg_per_l;

    /// pH value (0-14).
    double ph;

    /// Oxidation-reduction potential (mV).
    double orp_mv;

    /// Turbidity (NTU).
    double turbidity_ntu;

    /// Timestamp of state (UNIX epoch seconds).
    uint64_t timestamp;

    /// Reach segment identifier.
    uint64_t reach_id;

    /// Upstream reach identifier (0 if none).
    uint64_t upstream_id;

    /// Downstream reach identifier (0 if none).
    uint64_t downstream_id;

    /// Default constructor with zero initialization.
    ReachState()
        : q_m3s(0.0)
        , area_m2(0.0)
        , c_kg_per_m3(0.0)
        , temp_c(25.0)
        , t90_days(120.0)
        , depth_m(0.0)
        , velocity_mps(0.0)
        , hydraulic_radius_m(0.0)
        , manning_n(0.03)
        , bed_slope(0.001)
        , do_mg_per_l(8.0)
        , ph(7.0)
        , orp_mv(0.0)
        , turbidity_ntu(0.0)
        , timestamp(0)
        , reach_id(0)
        , upstream_id(0)
        , downstream_id(0)
    {}

    /// Constructor with minimal parameters.
    ReachState(double q, double area, double c, double temp, double t90)
        : q_m3s(q)
        , area_m2(area)
        , c_kg_per_m3(c)
        , temp_c(temp)
        , t90_days(t90)
        , depth_m(area > 0 ? std::sqrt(area) : 0.0)
        , velocity_mps(q / area)
        , hydraulic_radius_m(depth_m)
        , manning_n(0.03)
        , bed_slope(0.001)
        , do_mg_per_l(8.0)
        , ph(7.0)
        , orp_mv(0.0)
        , turbidity_ntu(0.0)
        , timestamp(0)
        , reach_id(0)
        , upstream_id(0)
        , downstream_id(0)
    {}

    /// Validates state values are within physical bounds.
    bool is_valid() const {
        if (std::isnan(q_m3s) || std::isinf(q_m3s) || q_m3s < 0.0) return false;
        if (std::isnan(area_m2) || std::isinf(area_m2) || area_m2 < 0.0) return false;
        if (std::isnan(c_kg_per_m3) || std::isinf(c_kg_per_m3) || c_kg_per_m3 < 0.0) return false;
        if (std::isnan(temp_c) || std::isinf(temp_c) || temp_c < -50.0 || temp_c > 150.0) return false;
        if (std::isnan(t90_days) || std::isinf(t90_days) || t90_days < 0.0) return false;
        if (std::isnan(depth_m) || std::isinf(depth_m) || depth_m < 0.0) return false;
        if (std::isnan(velocity_mps) || std::isinf(velocity_mps) || velocity_mps < 0.0) return false;
        if (std::isnan(ph) || std::isinf(ph) || ph < 0.0 || ph > 14.0) return false;
        if (std::isnan(do_mg_per_l) || std::isinf(do_mg_per_l) || do_mg_per_l < 0.0) return false;
        return true;
    }

    /// Computes velocity from flow and area.
    double compute_velocity() const {
        return area_m2 > 0.0 ? q_m3s / area_m2 : 0.0;
    }

    /// Computes hydraulic radius for circular channel.
    double compute_hydraulic_radius() const {
        if (depth_m <= 0.0) return 0.0;
        return depth_m / 4.0; // Simplified for circular
    }

    /// Computes Froude number (dimensionless).
    double froude_number() const {
        if (depth_m <= 0.0) return 0.0;
        const double g = 9.81; // m/s²
        return velocity_mps / std::sqrt(g * depth_m);
    }

    /// Returns true if flow is subcritical (Fr < 1).
    bool is_subcritical() const {
        return froude_number() < 1.0;
    }

    /// Returns true if flow is supercritical (Fr > 1).
    bool is_supercritical() const {
        return froude_number() > 1.0;
    }

    /// Computes Reynolds number (dimensionless).
    double reynolds_number() const {
        if (hydraulic_radius_m <= 0.0) return 0.0;
        const double nu = 1.0e-6; // Kinematic viscosity of water (m²/s)
        return (velocity_mps * hydraulic_radius_m) / nu;
    }

    /// Returns true if flow is turbulent (Re > 4000).
    bool is_turbulent() const {
        return reynolds_number() > 4000.0;
    }

    /// Computes travel time through reach segment (seconds).
    double travel_time_seconds(double length_m) const {
        if (velocity_mps <= 0.0) return 0.0;
        return length_m / velocity_mps;
    }

    /// Computes mass flow rate (kg/s).
    double mass_flow_rate() const {
        return q_m3s * c_kg_per_m3;
    }

    /// Computes volumetric load (m³/day).
    double volumetric_load_per_day() const {
        return q_m3s * 86400.0;
    }
};

} // namespace cyboquatic

// ============================================================================
// Kernel Corridor Bands (Safety Thresholds Mirror)
// ============================================================================

namespace cyboquatic {

/// Safety corridor bands mirroring Rust CorridorBands structure.
///
/// These corridors define the acceptable operational ranges for
/// hydraulic and water quality parameters. Values are computed
/// in C++ but validated against Rust-defined safety gates.
///
/// # Note
/// Corridor values should match the Rust cyboquatic-ecosafety-core
/// definitions for consistency. Changes must be synchronized.
struct KernelCorridors {
    /// Maximum flow rate before hard-band violation (m³/s).
    double q_hard_m3s;

    /// Safe concentration threshold (kg/m³).
    double c_safe_kg_per_m3;

    /// Hard-band concentration threshold (kg/m³).
    double c_hard_kg_per_m3;

    /// Gold-band t90 threshold (days).
    double t90_gold_days;

    /// Maximum t90 before hard-band violation (days).
    double t90_max_days;

    /// Safe temperature (°C).
    double temp_safe_c;

    /// Hard-band temperature (°C).
    double temp_hard_c;

    /// Safe pH minimum.
    double ph_safe_min;

    /// Safe pH maximum.
    double ph_safe_max;

    /// Hard-band pH minimum.
    double ph_hard_min;

    /// Hard-band pH maximum.
    double ph_hard_max;

    /// Minimum dissolved oxygen (mg/L).
    double do_min_mg_per_l;

    /// Maximum turbidity (NTU).
    double turbidity_max_ntu;

    /// Default constructor with conservative values.
    KernelCorridors()
        : q_hard_m3s(100.0)
        , c_safe_kg_per_m3(0.001)
        , c_hard_kg_per_m3(0.01)
        , t90_gold_days(120.0)
        , t90_max_days(180.0)
        , temp_safe_c(25.0)
        , temp_hard_c(40.0)
        , ph_safe_min(6.5)
        , ph_safe_max(8.5)
        , ph_hard_min(5.0)
        , ph_hard_max(10.0)
        , do_min_mg_per_l(5.0)
        , turbidity_max_ntu(50.0)
    {}

    /// Constructor with explicit parameters.
    KernelCorridors(
        double q_hard,
        double c_safe,
        double c_hard,
        double t90_gold,
        double t90_max
    )
        : q_hard_m3s(q_hard)
        , c_safe_kg_per_m3(c_safe)
        , c_hard_kg_per_m3(c_hard)
        , t90_gold_days(t90_gold)
        , t90_max_days(t90_max)
        , temp_safe_c(25.0)
        , temp_hard_c(40.0)
        , ph_safe_min(6.5)
        , ph_safe_max(8.5)
        , ph_hard_min(5.0)
        , ph_hard_max(10.0)
        , do_min_mg_per_l(5.0)
        , turbidity_max_ntu(50.0)
    {}

    /// Validates corridor bounds are properly ordered.
    bool is_valid() const {
        if (q_hard_m3s <= 0.0) return false;
        if (c_safe_kg_per_m3 < 0.0 || c_hard_kg_per_m3 < c_safe_kg_per_m3) return false;
        if (t90_gold_days < 0.0 || t90_max_days < t90_gold_days) return false;
        if (temp_safe_c < -50.0 || temp_hard_c < temp_safe_c) return false;
        if (ph_safe_min < 0.0 || ph_safe_max < ph_safe_min) return false;
        if (ph_hard_min < 0.0 || ph_hard_max < ph_hard_min) return false;
        if (do_min_mg_per_l < 0.0) return false;
        if (turbidity_max_ntu < 0.0) return false;
        return true;
    }

    /// Creates strict corridors for sensitive environments.
    static KernelCorridors strict() {
        KernelCorridors c;
        c.q_hard_m3s = 50.0;
        c.c_safe_kg_per_m3 = 0.0005;
        c.c_hard_kg_per_m3 = 0.005;
        c.t90_gold_days = 60.0;
        c.t90_max_days = 90.0;
        c.temp_safe_c = 20.0;
        c.temp_hard_c = 30.0;
        c.ph_safe_min = 7.0;
        c.ph_safe_max = 8.0;
        c.do_min_mg_per_l = 7.0;
        c.turbidity_max_ntu = 10.0;
        return c;
    }

    /// Creates relaxed corridors for industrial environments.
    static KernelCorridors relaxed() {
        KernelCorridors c;
        c.q_hard_m3s = 200.0;
        c.c_safe_kg_per_m3 = 0.005;
        c.c_hard_kg_per_m3 = 0.05;
        c.t90_gold_days = 180.0;
        c.t90_max_days = 365.0;
        c.temp_safe_c = 30.0;
        c.temp_hard_c = 50.0;
        c.ph_safe_min = 6.0;
        c.ph_safe_max = 9.0;
        c.do_min_mg_per_l = 3.0;
        c.turbidity_max_ntu = 100.0;
        return c;
    }
};

} // namespace cyboquatic

// ============================================================================
// Kernel Risk Coordinates (Computed from State + Corridors)
// ============================================================================

namespace cyboquatic {

/// Computed risk coordinates for kernel evaluation.
///
/// These coordinates are derived by normalizing state values against
/// corridor bands. Each coordinate represents a different dimension
/// of hydraulic/water quality risk.
///
/// # Note
/// Risk computation logic must mirror Rust RiskCoord normalization
/// for consistency across the safety framework.
struct KernelRisks {
    /// Flow rate risk coordinate (0.0-1.0).
    double r_q;

    /// Concentration risk coordinate (0.0-1.0).
    double r_c;

    /// Biodegradation time risk coordinate (0.0-1.0).
    double r_t90;

    /// Temperature risk coordinate (0.0-1.0).
    double r_temp;

    /// pH risk coordinate (0.0-1.0).
    double r_ph;

    /// Dissolved oxygen risk coordinate (0.0-1.0).
    double r_do;

    /// Turbidity risk coordinate (0.0-1.0).
    double r_turbidity;

    /// Default constructor with zero risks.
    KernelRisks()
        : r_q(0.0)
        , r_c(0.0)
        , r_t90(0.0)
        , r_temp(0.0)
        , r_ph(0.0)
        , r_do(0.0)
        , r_turbidity(0.0)
    {}

    /// Returns the maximum risk coordinate.
    double max_risk() const {
        double max = r_q;
        if (r_c > max) max = r_c;
        if (r_t90 > max) max = r_t90;
        if (r_temp > max) max = r_temp;
        if (r_ph > max) max = r_ph;
        if (r_do > max) max = r_do;
        if (r_turbidity > max) max = r_turbidity;
        return max;
    }

    /// Returns the mean risk coordinate.
    double mean_risk() const {
        return (r_q + r_c + r_t90 + r_temp + r_ph + r_do + r_turbidity) / 7.0;
    }

    /// Returns true if all risks are acceptable (< 1.0).
    bool all_acceptable() const {
        return max_risk() < 1.0;
    }

    /// Returns true if any risk is in hard-band (>= 1.0).
    bool any_hard_band() const {
        return max_risk() >= 1.0;
    }

    /// Computes weighted sum of squared risks (Vt component).
    double weighted_squared_sum(const std::vector<double>& weights) const {
        if (weights.size() < 7) return 0.0;
        double sum = 0.0;
        sum += std::max(0.0, weights[0]) * r_q * r_q;
        sum += std::max(0.0, weights[1]) * r_c * r_c;
        sum += std::max(0.0, weights[2]) * r_t90 * r_t90;
        sum += std::max(0.0, weights[3]) * r_temp * r_temp;
        sum += std::max(0.0, weights[4]) * r_ph * r_ph;
        sum += std::max(0.0, weights[5]) * r_do * r_do;
        sum += std::max(0.0, weights[6]) * r_turbidity * r_turbidity;
        return sum;
    }
};

} // namespace cyboquatic

// ============================================================================
// Step Result (Kernel Computation Output)
// ============================================================================

namespace cyboquatic {

/// Result of a single kernel computation step.
///
/// This structure contains the updated state, computed risks, and
/// Lyapunov residual for Rust safety evaluation.
struct StepResult {
    /// Updated reach state after computation.
    ReachState state;

    /// Computed risk coordinates.
    KernelRisks risks;

    /// Lyapunov residual (Vt).
    double vt;

    /// Timestep number.
    uint64_t timestep;

    /// Computation time in microseconds.
    uint64_t computation_time_us;

    /// Whether computation was successful.
    bool success;

    /// Error message (if computation failed).
    std::string error_message;

    /// Cryptographic hash of result (for audit integrity).
    uint64_t result_hash;

    /// Default constructor.
    StepResult()
        : vt(0.0)
        , timestep(0)
        , computation_time_us(0)
        , success(false)
        , result_hash(0)
    {}

    /// Validates result is consistent.
    bool is_valid() const {
        return success && state.is_valid() && risks.all_acceptable();
    }
};

} // namespace cyboquatic

// ============================================================================
// Kernel Functions (Numerical Computation Interface)
// ============================================================================

namespace cyboquatic {
namespace kernel {

/// Normalizes a value linearly across corridor bands.
///
/// # Parameters
/// - x: Value to normalize
/// - x_safe: Safe band upper bound
/// - x_gold: Gold band upper bound
/// - x_hard: Hard band upper bound
///
/// # Returns
/// Normalized risk coordinate in [0, 1]
///
/// # Note
/// This function must mirror Rust CorridorBands::normalize()
double normalize_linear(double x, double x_safe, double x_gold, double x_hard);

/// Clamps a value to [0, 1] range.
///
/// # Parameters
/// - x: Value to clamp
///
/// # Returns
/// Clamped value in [0, 1]
double clamp01(double x);

/// Computes risk coordinates from state and corridors.
///
/// # Parameters
/// - state: Current reach state
/// - corridors: Safety corridor bands
///
/// # Returns
/// Computed risk coordinates
KernelRisks compute_risks(const ReachState& state, const KernelCorridors& corridors);

/// Computes Lyapunov residual from risks and weights.
///
/// # Parameters
/// - risks: Risk coordinates
/// - weights: Risk weights (must have at least 7 elements)
///
/// # Returns
/// Lyapunov residual (Vt)
double compute_vt(const KernelRisks& risks, const std::vector<double>& weights);

/// Advances reach state by one timestep.
///
/// # Parameters
/// - s0: Initial state
/// - corridors: Safety corridor bands
/// - weights: Risk weights
/// - dt_seconds: Timestep duration
///
/// # Returns
/// Step result with updated state and risks
StepResult advance_reach(
    const ReachState& s0,
    const KernelCorridors& corridors,
    const std::vector<double>& weights,
    double dt_seconds
);

/// Computes mass balance for reach segment.
///
/// # Parameters
/// - state: Current state
/// - inflow_conc: Inflow concentration (kg/m³)
/// - reaction_rate: First-order reaction rate (1/day)
/// - dt_seconds: Timestep duration
///
/// # Returns
/// Updated concentration (kg/m³)
double compute_mass_balance(
    const ReachState& state,
    double inflow_conc,
    double reaction_rate,
    double dt_seconds
);

/// Computes temperature adjustment for reaction rates.
///
/// # Parameters
/// - base_rate: Reaction rate at reference temperature
/// - temp_c: Current temperature (°C)
/// - ref_temp_c: Reference temperature (°C)
/// - theta: Temperature coefficient (typically 1.047)
///
/// # Returns
/// Adjusted reaction rate
double temperature_adjustment(
    double base_rate,
    double temp_c,
    double ref_temp_c,
    double theta
);

/// Computes dissolved oxygen saturation concentration.
///
/// # Parameters
/// - temp_c: Temperature (°C)
/// - pressure_atm: Atmospheric pressure (atm)
///
/// # Returns
/// DO saturation concentration (mg/L)
double do_saturation(double temp_c, double pressure_atm);

/// Computes reaeration rate (O'Connor-Dobbins).
///
/// # Parameters
/// - velocity_mps: Flow velocity (m/s)
/// - depth_m: Water depth (m)
///
/// # Returns
/// Reaeration rate (1/day)
double reaeration_rate_od(double velocity_mps, double depth_m);

/// Computes sediment oxygen demand.
///
/// # Parameters
/// - temp_c: Temperature (°C)
/// - base_sod: Base SOD at 20°C (g/m²/day)
///
/// # Returns
/// Temperature-adjusted SOD (g/m²/day)
double sediment_oxygen_demand(double temp_c, double base_sod);

/// Generates a simple hash for result integrity.
///
/// # Parameters
/// - data: Data to hash
/// - length: Data length in bytes
///
/// # Returns
/// 64-bit hash value
uint64_t compute_hash(const void* data, size_t length);

} // namespace kernel
} // namespace cyboquatic

// ============================================================================
// CSV Shard Export (ALN-Compatible Output)
// ============================================================================

namespace cyboquatic {
namespace io {

/// Writes a step result to CSV format for ALN shard compatibility.
///
/// # Parameters
/// - filepath: Output file path
/// - t_hours: Time in hours
/// - state: Reach state
/// - risks: Risk coordinates
/// - vt: Lyapunov residual
/// - hexstamp: Cryptographic hash string
///
/// # Returns
/// true if write successful, false otherwise
bool write_shard_row_csv(
    const std::string& filepath,
    double t_hours,
    const ReachState& state,
    const KernelRisks& risks,
    double vt,
    const std::string& hexstamp
);

/// Writes multiple step results to CSV file.
///
/// # Parameters
/// - filepath: Output file path
/// - results: Vector of step results
/// - include_header: Whether to write CSV header
///
/// # Returns
/// Number of rows written
size_t write_shard_batch_csv(
    const std::string& filepath,
    const std::vector<StepResult>& results,
    bool include_header
);

/// Reads step results from CSV file.
///
/// # Parameters
/// - filepath: Input file path
///
/// # Returns
/// Vector of step results (empty on error)
std::vector<StepResult> read_shard_batch_csv(const std::string& filepath);

/// Validates CSV file format.
///
/// # Parameters
/// - filepath: File path to validate
///
/// # Returns
/// true if valid, false otherwise
bool validate_csv_format(const std::string& filepath);

} // namespace io
} // namespace cyboquatic

// ============================================================================
// Configuration and Utilities
// ============================================================================

namespace cyboquatic {
namespace config {

/// Kernel configuration parameters.
struct KernelConfig {
    /// Timestep duration (seconds).
    double dt_seconds;

    /// Maximum simulation time (seconds).
    double max_time_seconds;

    /// Lyapunov epsilon tolerance.
    double eps_vt;

    /// Enable numerical damping.
    bool enable_damping;

    /// Damping coefficient.
    double damping_coeff;

    /// Enable adaptive timestep.
    bool enable_adaptive_dt;

    /// Minimum timestep (seconds).
    double min_dt_seconds;

    /// Maximum timestep (seconds).
    double max_dt_seconds;

    /// Enable result hashing.
    bool enable_hashing;

    /// Default configuration.
    KernelConfig()
        : dt_seconds(60.0)
        , max_time_seconds(86400.0)
        , eps_vt(0.001)
        , enable_damping(true)
        , damping_coeff(0.1)
        , enable_adaptive_dt(false)
        , min_dt_seconds(1.0)
        , max_dt_seconds(300.0)
        , enable_hashing(true)
    {}

    /// Validates configuration parameters.
    bool is_valid() const {
        if (dt_seconds <= 0.0) return false;
        if (max_time_seconds <= 0.0) return false;
        if (eps_vt < 0.0) return false;
        if (damping_coeff < 0.0 || damping_coeff > 1.0) return false;
        if (min_dt_seconds <= 0.0) return false;
        if (max_dt_seconds < min_dt_seconds) return false;
        return true;
    }

    /// Creates configuration for high-precision simulation.
    static KernelConfig high_precision() {
        KernelConfig c;
        c.dt_seconds = 10.0;
        c.min_dt_seconds = 1.0;
        c.max_dt_seconds = 60.0;
        c.enable_adaptive_dt = true;
        c.enable_damping = true;
        c.damping_coeff = 0.05;
        return c;
    }

    /// Creates configuration for long-term simulation.
    static KernelConfig long_term() {
        KernelConfig c;
        c.dt_seconds = 300.0;
        c.max_time_seconds = 31536000.0; // 1 year
        c.min_dt_seconds = 60.0;
        c.max_dt_seconds = 600.0;
        c.enable_adaptive_dt = true;
        c.enable_damping = true;
        c.damping_coeff = 0.2;
        return c;
    }
};

/// Returns current timestamp in microseconds.
uint64_t current_time_micros();

/// Returns current UNIX timestamp in seconds.
uint64_t current_timestamp_unix();

} // namespace config
} // namespace cyboquatic

// ============================================================================
// Error Handling
// ============================================================================

namespace cyboquatic {

/// Kernel error codes.
enum class KernelError {
    Success = 0,
    InvalidState = 1,
    InvalidCorridors = 2,
    NumericalInstability = 3,
    ConvergenceFailure = 4,
    OutOfBounds = 5,
    NaN_Detected = 6,
    Infinity_Detected = 7,
    TimestepTooSmall = 8,
    TimestepTooLarge = 9,
    MassBalanceError = 10,
    IOError = 11,
    ConfigurationError = 12,
    Unknown = 99
};

/// Converts error code to string.
inline const char* kernel_error_string(KernelError err) {
    switch (err) {
        case KernelError::Success: return "Success";
        case KernelError::InvalidState: return "Invalid State";
        case KernelError::InvalidCorridors: return "Invalid Corridors";
        case KernelError::NumericalInstability: return "Numerical Instability";
        case KernelError::ConvergenceFailure: return "Convergence Failure";
        case KernelError::OutOfBounds: return "Out of Bounds";
        case KernelError::NaN_Detected: return "NaN Detected";
        case KernelError::Infinity_Detected: return "Infinity Detected";
        case KernelError::TimestepTooSmall: return "Timestep Too Small";
        case KernelError::TimestepTooLarge: return "Timestep Too Large";
        case KernelError::MassBalanceError: return "Mass Balance Error";
        case KernelError::IOError: return "I/O Error";
        case KernelError::ConfigurationError: return "Configuration Error";
        case KernelError::Unknown: return "Unknown Error";
        default: return "Unknown Error";
    }
}

/// Kernel error result type.
struct KernelResult {
    KernelError error;
    std::string message;

    KernelResult() : error(KernelError::Success) {}
    KernelResult(KernelError e) : error(e) {}
    KernelResult(KernelError e, const std::string& msg) : error(e), message(msg) {}

    bool is_success() const { return error == KernelError::Success; }
    bool is_error() const { return error != KernelError::Success; }
};

} // namespace cyboquatic

#endif // CYBOQUATIC_KERNEL_HPP
