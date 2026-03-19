/**
 * Cyboquatic Simulation Kernel (C++)
 * 
 * High-performance numerical worker mirroring Rust ecosafety logic.
 * Simulates hydraulics, contaminant transport, and substrate degradation
 * in 1D canal reaches. Treats corridor bands and Lyapunov semantics as
 * imported constants owned by Rust/ALN.
 * 
 * Safety Invariants:
 * - V_t_next <= V_t_prev + epsilon
 * - No risk coordinate r_x >= 1.0
 * - Carbon-negative operation prioritized in risk scoring
 * 
 * @file main.cpp
 * @destination cyboquatic-simulation-kernel/src/main.cpp
 */

#include <iostream>
#include <vector>
#include <array>
#include <cmath>
#include <optional>
#include <string>
#include <numeric>
#include <algorithm>
#include <stdexcept>

// ============================================================================
// CONSTANTS (Mirrored from Rust Core)
// ============================================================================

constexpr size_t MAX_RISK_PLANES = 8;
constexpr double DEFAULT_LYAPUNOV_EPSILON = 0.001;
constexpr double RISK_HARD_LIMIT = 1.0;
constexpr double DEFAULT_T90_MAX_DAYS = 180.0;

// ============================================================================
// RISK PLANE ENUMERATION
// ============================================================================

enum class RiskPlane : uint8_t {
    Energy = 0,
    Hydraulic = 1,
    Biology = 2,
    Carbon = 3,
    Materials = 4,
    Thermal = 5,
    Mechanical = 6,
    SensorCalibration = 7
};

constexpr const char* RISK_PLANE_NAMES[] = {
    "energy", "hydraulic", "biology", "carbon", 
    "materials", "thermal", "mechanical", "sensor_calibration"
};

// ============================================================================
// CORRIDOR NORMALIZATION
// ============================================================================

struct CorridorBands {
    double safe_upper;
    double gold_upper;
    double hard_limit;

    constexpr CorridorBands() 
        : safe_upper(0.30), gold_upper(0.70), hard_limit(1.00) {}

    constexpr CorridorBands(double s, double g, double h) 
        : safe_upper(s), gold_upper(g), hard_limit(h) {}

    /**
     * Normalizes raw measurement to risk coordinate r_x ∈ [0,1]
     * Mirrors Rust logic exactly for simulation fidelity.
     */
    double normalize(double raw_value, double ref_min, double ref_max) const {
        if (ref_max <= ref_min) return 1.0;
        
        double normalized = (raw_value - ref_min) / (ref_max - ref_min);
        double clamped = std::max(0.0, std::min(1.0, normalized));
        
        if (clamped < safe_upper) {
            return clamped * 0.5;
        } else if (clamped < gold_upper) {
            return 0.15 + (clamped - safe_upper) * 1.25;
        } else {
            return 0.65 + (clamped - gold_upper) * 1.75;
        }
    }

    bool corridor_ok(double risk_coord) const {
        return risk_coord < hard_limit;
    }
};

// ============================================================================
// RISK VECTOR & LYAPUNOV
// ============================================================================

struct RiskVector {
    std::array<double, MAX_RISK_PLANES> coordinates;
    uint64_t timestamp;
    bool validated;

    RiskVector() : coordinates{}, timestamp(0), validated(true) {
        coordinates.fill(0.0);
    }

    void set_coordinate(RiskPlane plane, double value) {
        size_t idx = static_cast<size_t>(plane);
        coordinates[idx] = std::max(0.0, std::min(1.0, value));
    }

    double get_coordinate(RiskPlane plane) const {
        return coordinates[static_cast<size_t>(plane)];
    }

    double max_coordinate() const {
        return *std::max_element(coordinates.begin(), coordinates.end());
    }

    /**
     * Computes quadratic Lyapunov residual V_t = Σ w_j r_j²
     */
    double lyapunov_residual(const std::array<double, MAX_RISK_PLANES>& weights) const {
        double v_t = 0.0;
        for (size_t i = 0; i < MAX_RISK_PLANES; ++i) {
            v_t += weights[i] * coordinates[i] * coordinates[i];
        }
        return v_t;
    }

    bool is_valid() const {
        if (!validated) return false;
        for (double c : coordinates) {
            if (c < 0.0 || c > 1.0) return false;
        }
        return true;
    }
};

// ============================================================================
// SIMULATION STATE
// ============================================================================

struct SimulationState {
    double current_v_t;
    double previous_v_t;
    RiskVector current_risk;
    double energy_surplus;
    uint64_t timestamp;
    bool valid;

    SimulationState() 
        : current_v_t(0.0), previous_v_t(0.0), energy_surplus(0.0), 
          timestamp(0), valid(true) {}
};

// ============================================================================
// PHYSICS KERNEL (Hydraulics & Contaminants)
// ============================================================================

class PhysicsKernel {
public:
    /**
     * Advances flow and contaminant concentrations (PFAS/CEC) 
     * with explicit advection-reaction steps.
     * 
     * @param flow_rate m³/s
     * @param contaminant_mass kg
     * @param volume m³
     * @param dt time step seconds
     * @return updated contaminant concentration kg/m³
     */
    static double advance_contaminant(double flow_rate, double contaminant_mass, 
                                      double volume, double dt) {
        if (volume <= 0.0) return 0.0;
        
        // Simple advection-reaction model
        double concentration = contaminant_mass / volume;
        double outflow_mass = concentration * flow_rate * dt;
        double reaction_decay = concentration * 0.01 * dt; // First order decay
        
        double new_mass = std::max(0.0, contaminant_mass - outflow_mass - reaction_decay);
        return new_mass / volume;
    }

    /**
     * Computes substrate mass loss based on t90 kinetics and fluid state.
     * 
     * @param current_mass kg
     * @param t90_days days to 90% degradation
     * @param temperature Celsius
     * @param dt days
     * @return remaining mass kg
     */
    static double advance_substrate(double current_mass, double t90_days, 
                                    double temperature, double dt) {
        if (t90_days <= 0.0) return current_mass;
        
        // Arrhenius-like temperature adjustment (simplified)
        double temp_factor = 1.0 + 0.05 * (temperature - 20.0);
        double decay_rate = -std::log(0.1) / (t90_days * temp_factor);
        
        double remaining = current_mass * std::exp(-decay_rate * dt);
        return std::max(0.0, remaining);
    }
};

// ============================================================================
// ECOSAFETY ENFORCER (C++ Mirror)
// ============================================================================

class EcosafetyEnforcer {
private:
    std::array<double, MAX_RISK_PLANES> weights;
    double epsilon;
    std::array<CorridorBands, MAX_RISK_PLANES> corridors;
    double current_v_t;

public:
    EcosafetyEnforcer() : epsilon(DEFAULT_LYAPUNOV_EPSILON), current_v_t(0.0) {
        weights.fill(1.0);
        corridors.fill(CorridorBands());
    }

    void set_weight(RiskPlane plane, double weight) {
        weights[static_cast<size_t>(plane)] = std::max(0.0, weight);
    }

    /**
     * Enforces Lyapunov stability invariant on simulation step.
     * 
     * @return std::nullopt if safety invariant violated
     */
    std::optional<RiskVector> enforce_step(const RiskVector& proposed_risk) {
        if (!proposed_risk.is_valid()) {
            return std::nullopt;
        }

        // Check corridor bounds
        for (size_t i = 0; i < MAX_RISK_PLANES; ++i) {
            double coord = proposed_risk.coordinates[i];
            if (!corridors[i].corridor_ok(coord)) {
                std::cerr << "Corridor violation in plane " << i << std::endl;
                return std::nullopt;
            }
        }

        // Check Lyapunov invariant
        double proposed_v_t = proposed_risk.lyapunov_residual(weights);
        if (proposed_v_t > current_v_t + epsilon) {
            std::cerr << "Lyapunov violation: V_t " << current_v_t 
                      << " -> " << proposed_v_t << std::endl;
            return std::nullopt;
        }

        current_v_t = proposed_v_t;
        return proposed_risk;
    }

    double current_lyapunov() const { return current_v_t; }
};

// ============================================================================
// SIMULATION KERNEL MAIN
// ============================================================================

class CyboquaticSimulationKernel {
private:
    EcosafetyEnforcer enforcer;
    SimulationState state;
    double channel_length;
    double channel_area;

public:
    CyboquaticSimulationKernel(double length, double area) 
        : channel_length(length), channel_area(area) {}

    /**
     * Executes a single simulation time step with safety enforcement.
     * 
     * @param dt Time step (days)
     * @param inflow_mass Contaminant inflow (kg)
     * @param substrate_mass Current substrate mass (kg)
     * @param temperature Celsius
     * @return true if step was safe and accepted
     */
    bool step(double dt, double inflow_mass, double substrate_mass, double temperature) {
        state.previous_v_t = state.current_v_t;
        state.timestamp++;

        // 1. Advance Physics
        double volume = channel_length * channel_area;
        double flow_rate = volume * 0.1; // Simplified flow assumption
        double contaminant_conc = PhysicsKernel::advance_contaminant(
            flow_rate, inflow_mass, volume, dt * 86400.0 // Convert days to seconds
        );
        
        double remaining_substrate = PhysicsKernel::advance_substrate(
            substrate_mass, DEFAULT_T90_MAX_DAYS, temperature, dt
        );

        // 2. Compute Risk Coordinates
        RiskVector proposed_risk;
        proposed_risk.timestamp = state.timestamp;

        // Hydraulic Risk (based on flow variance)
        proposed_risk.set_coordinate(RiskPlane::Hydraulic, 
            enforcer.corridors[static_cast<size_t>(RiskPlane::Hydraulic)]
            .normalize(flow_rate, 0.0, 100.0));

        // Biology Risk (based on contaminant concentration)
        proposed_risk.set_coordinate(RiskPlane::Biology, 
            enforcer.corridors[static_cast<size_t>(RiskPlane::Biology)]
            .normalize(contaminant_conc, 0.0, 0.001)); // kg/m³ threshold

        // Carbon Risk (based on substrate degradation sequestration)
        double carbon_sequestered = (substrate_mass - remaining_substrate) * 0.5; // kg CO2e
        double carbon_risk = std::max(0.0, 1.0 - (carbon_sequestered / 10.0)); // Negative ops reduce risk
        proposed_risk.set_coordinate(RiskPlane::Carbon, carbon_risk);

        // Materials Risk (based on substrate persistence)
        double material_risk = remaining_substrate / substrate_mass;
        proposed_risk.set_coordinate(RiskPlane::Materials, 
            enforcer.corridors[static_cast<size_t>(RiskPlane::Materials)]
            .normalize(material_risk, 0.0, 1.0));

        // Energy Risk (assume constant low energy for simulation)
        proposed_risk.set_coordinate(RiskPlane::Energy, 0.1);

        // 3. Enforce Safety
        auto result = enforcer.enforce_step(proposed_risk);
        
        if (result.has_value()) {
            state.current_risk = result.value();
            state.current_v_t = enforcer.current_lyapunov();
            state.energy_surplus -= 0.01; // Simulate energy cost
            return true;
        } else {
            state.valid = false;
            return false;
        }
    }

    void print_status() const {
        std::cout << "Step: " << state.timestamp 
                  << " | V_t: " << state.current_v_t 
                  << " | Max Risk: " << state.current_risk.max_coordinate()
                  << " | Valid: " << (state.valid ? "YES" : "NO")
                  << std::endl;
    }

    bool is_valid() const { return state.valid; }
};

// ============================================================================
// ENTRY POINT
// ============================================================================

int main(int argc, char* argv[]) {
    std::cout << "=== Cyboquatic Simulation Kernel (C++) ===" << std::endl;
    std::cout << "Mirroring Rust Ecosafety Logic for High-Throughput Numerics" << std::endl;
    std::cout << "------------------------------------------" << std::endl;

    // Initialize Kernel (100m channel, 10m² area)
    CyboquaticSimulationKernel kernel(100.0, 10.0);

    // Simulation Parameters
    double dt = 0.1; // days
    double initial_substrate = 50.0; // kg
    double initial_contaminant = 0.0001; // kg
    
    std::cout << "Starting Simulation Loop..." << std::endl;

    // Run 20 Simulation Steps
    for (int i = 0; i < 20; ++i) {
        // Inject slight contaminant pulse at step 5
        double inflow = (i == 5) ? 0.0005 : 0.0001;
        
        bool safe = kernel.step(dt, inflow, initial_substrate, 25.0);
        
        if (!safe) {
            std::cerr << "Simulation halted at step " << i << " due to safety violation." << std::endl;
            break;
        }

        if (i % 5 == 0) {
            kernel.print_status();
        }
    }

    std::cout << "------------------------------------------" << std::endl;
    if (kernel.is_valid()) {
        std::cout << "Simulation Completed Successfully." << std::endl;
        std::cout << "Final Lyapunov Residual: " << kernel.current_v_t << std::endl;
    } else {
        std::cout << "Simulation Terminated Early (Safety Gate)." << std::endl;
    }

    return kernel.is_valid() ? 0 : 1;
}
