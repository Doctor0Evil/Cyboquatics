# Phoenix Cyboquatic Water-Reuse Pilot Framework

ecosafety-hex-stamp: 0xC1B0QUA7IC-PHX-ALN-2026-01-17
primary-bostrom-id: bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
alt-bostrom-id: bostrom1ldgmtf20d6604a24ztr0jxht7xt7az4jhkmsrc
linked-qpu-shard: qpudatashards.cyboquatic-phoenix.aln

## Scope

- District-scale cyboquatic module (~50,000 residents) integrated into Phoenix canals, sewers, and SAT recharge basins.
- Hard ecosafety corridors on hydraulics, treatment, SAT, energy, and governance.
- Formal Pilot-Gate protocol that must be passed before any replication or scale-up. [file:3]

## Design Envelope (Hydraulics)

- Design flow: Q_design = 0.29 m³/s (50,000 people × 500 L/(person·day)). [file:3]
- Target residence time: t_res = 1,200 s (20 min).
- Reactor volume: V_reactor = Q_design × t_res ≈ 348–377 m³ (e.g., 30 m × 4 m ID cylindrical vault, parallel trains for redundancy). [file:3]
- Energy recovery: H_turbine = 15 m, P ≈ 25.6 kW at η = 0.6. [file:3]

## Pilot-Gate Protocols (Narrative)

1. Hydraulic & Structural Gate
2. Treatment & SAT Gate
3. Fouling & O&M Gate
4. Social License & Governance Gate

Each gate is defined formally in `aln/pilot_gates.cyboquatic-phoenix.aln` and enforced in Rust verification modules under `src/cyboquatic/pilot_guard.rs`. [file:3]

## Risk Corridor & Residual

- Normalized corridor residual V(t) over key metrics: hydraulic headroom, sewer surcharge, BOD/TSS/N/P, CEC/PFAS, SAT performance, fouling rate, and social-license indicators.
- Scale-up is only permitted if V(t+1) ≤ V(t) over a full seasonal cycle and all Pilot-Gate predicates are satisfied. [file:3]

## Cross-Links

- `qpudatashards.cyboquatic-phoenix.aln`
- `docs/phoenix-cyboquatic-engine.md` (evidence-based framework and math spine).
- `src/cyboquatic/core.rs`, `src/cyboquatic/turbine.rs`, `src/cyboquatic/airfilter.rs`. [file:3]

## CI ecosafety checks for Phoenix cyboquatic pilot

The Phoenix cyboquatic pilot uses a hard ecosafety gate implemented in:

- `aln/pilot_gates.cyboquatic-phoenix.aln`
- `src/cyboquatic/pilot_guard.rs`
- `qpudatashards/cyboquatic-phoenix/*.csv` (telemetry + evidence)  

Before any deployment tag is allowed, CI runs a Rust test harness that:

1. Loads recent pilot metrics from `qpudatashards/cyboquatic-phoenix/`.
2. Maps them into `CyboquaticPilotMetrics`.
3. Applies `PilotCorridor::pilot_scale_up_ok(&metrics)` with corridor bounds that encode ADEQ / Clean Water Act limits, SAT constraints, and social-license thresholds. [file:3]

The CI job **must fail** if:

- Any metric violates its corridor (e.g., BOD/TSS/N/P, CEC/PFAS, SAT HLR, fouling rate, social trust, dashboard uptime), or
- `pilot_scale_up_ok` returns `false` for the current pilot state. [file:3]

### Example CI hook (Rust test)

```rust
// tests/pilot_scale_up_ci.rs

use cyboquatic::pilot_guard::{PilotCorridor};
use cyboquatic::types::CyboquaticPilotMetrics;

#[test]
fn phoenix_pilot_must_pass_scale_up_gate() {
    // 1. Load metrics from qpudatashards (CSV/ALN adapter not shown here).
    let m: CyboquaticPilotMetrics = load_phoenix_pilot_metrics()
        .expect("failed to load Phoenix pilot metrics");

    // 2. Corridor parameters (kept conservative; align with ALN schema).
    let corridor = PilotCorridor {
        bod_mg_l_max: 10.0,
        tss_mg_l_max: 10.0,
        n_total_mg_l_max: 5.0,
        p_total_mg_l_max: 0.5,
        cec_index_max: 0.3,
        pfbs_index_max: 0.3,
        sat_hlr_mday_min: 0.05,
        sat_hlr_mday_max: 0.30,
        fouling_rate_rel_min: -0.01,
        fouling_rate_rel_max: 0.00,
        social_trust_min: 0.75,
        violation_residual_max: 1.0,
    };

    assert!(
        corridor.pilot_scale_up_ok(&m),
        "Phoenix cyboquatic pilot failed ecosafety gate; deployment tag must not be created."
    );
}
