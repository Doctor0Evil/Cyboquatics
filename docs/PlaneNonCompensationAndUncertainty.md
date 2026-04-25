# Plane Non-Compensation and Uncertainty Invariants

This document describes three additions to the ecosafety grammar:

1. Per-plane weights and non-compensation flags (PlaneWeightsShard).
2. KER-level non-compensation rules for carbon and biodiversity.
3. Per-plane uncertainty channels and data-quality invariants for K and E.

## PlaneWeightsShard

`PlaneWeightsShard2026v1.csv` and `ecosafety.plane_weights.v1` define
per-plane Lyapunov weights plus flags indicating which planes are
non-offsettable in K/E scoring.

Typical setting for a water basin:

- Carbon and biodiversity marked as non-offsettable.
- Hydraulics, materials, and data-quality weighted but compensable.

## Non-Compensation Invariants

`ecosafety.ker_non_compensation.v1` encodes the rule:

- If carbon or biodiversity risk is already above its soft band and moves
  higher in a proposed step, then:
  - Residual risk R must not decrease.
  - Eco-impact E must not increase.

This prevents trading increased carbon or biodiversity harm for
improvements in other planes.

## Uncertainty Channels

`ecosafety.plane_uncertainty.v1` introduces:

- Per-plane uncertainty coordinates (e.g. `r_carbon_unc`).
- Production-lane caps loaded from `PlaneWeightsShard`.
- A global rule that high uncertainty caps K and E.

This makes epistemic uncertainty a first-class limiter on claimed
eco-impact.

## Data-Quality Invariants

`ecosafety.data_quality_invariants.v1` enforces:

- If `r_calib` or `r_sigma` worsen, then:
  - K_next ≤ K_prev
  - E_next ≤ E_prev
  - R_next ≥ R_prev

Higher ingest error or sensor uncertainty can never improve knowledge
or eco-impact scores and can never hide risk.

The helper header `ker_plane_adjust.hpp` provides a C++ reference
implementation suitable for CI and firmware integration.
