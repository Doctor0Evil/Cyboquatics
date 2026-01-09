from math import pow, max

struct FogNodeState:
    var Cin_mg_L: Float64
    var Cout_mg_L: Float64
    var Q_L_min: Float64
    var dt_s: Float64

struct FogImpactConfig:
    var Cref_mg_L: Float64
    var hazard_weight: Float64
    var karma_per_kg: Float64

struct FogImpactResult:
    var mass_avoided_kg: Float64
    var node_impact_K: Float64

fn computeFogImpact(s: FogNodeState, cfg: FogImpactConfig) -> FogImpactResult:
    if cfg.Cref_mg_L <= 0.0:
        raise Error("Cref_mg_L must be positive.")
    if s.dt_s <= 0.0 or s.Q_L_min < 0.0:
        raise Error("Invalid timestep or flow.")
    let deltaC_mg_L = s.Cin_mg_L - s.Cout_mg_L
    if deltaC_mg_L <= 0.0:
        return FogImpactResult(0.0, 0.0)
    # Convert to SI units for mass calculation
    let Q_m3_s = (s.Q_L_min / 1000.0) / 60.0
    let Cin_kg_m3 = s.Cin_mg_L / 1.0e6
    let Cout_kg_m3 = s.Cout_mg_L / 1.0e6
    let deltaC_kg_m3 = Cin_kg_m3 - Cout_kg_m3
    let mass_avoided_kg = deltaC_kg_m3 * Q_m3_s * s.dt_s
    # Normalized risk unit for Karma
    let risk_unit = deltaC_mg_L / cfg.Cref_mg_L
    let K_node = cfg.hazard_weight * risk_unit * mass_avoided_kg * cfg.karma_per_kg
    return FogImpactResult(mass_avoided_kg, K_node)

fn main():
    let s = FogNodeState(300.0, 60.0, 20.0, 28800.0)
    let cfg = FogImpactConfig(100.0, 1.2, 6.7e5)
    let result = computeFogImpact(s, cfg)
    print("Mass avoided (kg):", result.mass_avoided_kg)
    print("Node impact Karma:", result.node_impact_K)
