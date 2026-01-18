#pragma once

struct QuantumReflectionResidual {
    double delta_mass;    // kg or equivalent, should be ≈ 0
    double delta_energy;  // kWh, should be ≤ 0 for net-safe
};

inline bool satisfies_quantum_reflection(const QuantumReflectionResidual& r,
                                         double mass_tol,
                                         double energy_tol)
{
    const bool mass_ok   = std::abs(r.delta_mass)   <= mass_tol;
    const bool energy_ok = r.delta_energy           <= energy_tol;
    return mass_ok && energy_ok;
}
