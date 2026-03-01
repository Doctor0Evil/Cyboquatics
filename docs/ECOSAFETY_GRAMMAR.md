# Cyboquatic Ecosafety Grammar (Drain Focus)

This document defines the machine-checkable ecosafety grammar for cyboquatic drain systems.

## Concepts

- **Risk coordinates** `r_j ∈ [0,1]` for FOG, BOD/COD, nutrients, microplastics, PFAS, pathogens, deforestation, sewer blockage, energy.
- **Corridor bands** `[safe, gold, hard]` per metric, aligned with UWWTD, UNEP PFAS, microplastic guidance, and EUDR deforestation status.
- **Residuals**:
  - Lyapunov-style eco-risk residual `V_t = Σ_j w_j r_{j,t}`.
  - Uncertainty residual `U_t = Σ_j w_j σ_{j,t}`.

## Invariants

- **No corridor, no build**: deployment forbidden if required corridors are missing or invalid.
- **Hard-band guard**: if any `r_j ≥ 1`, the node must derate or stop.
- **Non-worsening outside safe interior**:
  - If `V_t > ε`, any accepted control step must satisfy `V_{t+1} ≤ V_t` and `U_{t+1} ≤ U_t`.

These invariants are enforced by `ecosafety-core` and used by `cyboquatic-drain-node` to gate control decisions and shard generation.
