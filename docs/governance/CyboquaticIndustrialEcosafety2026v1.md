# Cyboquatic Industrial Ecosafety Shard Family Governance

**Family ID:** `CyboquaticIndustrialEcosafety2026v1`  
**Status:** ACTIVE  
**Lane Bands:** RESEARCH, EXPERIMENTAL (PILOT), PRODUCTION  
**Governing Spec:** `ToolchainPolicyRustOnOss2026v1`

---

## 1. Shard Family Overview

This shard family governs **Cyboquatic industrial machinery nodes** including:

- **MAR Modules** – Modular aquatic remediation units
- **Fog Desiccators** – Airborne particulate capture systems
- **Air Globes** – Atmospheric monitoring and purification
- **CAIN Units** – Carbon absorption and integration nodes
- **Canal Purifiers** – Waterway treatment systems

All nodes operate under the **Rx–Vt–KER** ecosafety grammar with mandatory corridor validation and lane-gated actuation.

---

## 2. ALN Schema Reference

**Schema File:** `qpudatashards/particles/CyboquaticIndustrialEcosafety2026v1.aln`

The schema defines the following field groups:

| Group | Fields | Purpose |
|-------|--------|---------|
| **Identity** | `nodeid`, `nodetype`, `medium`, `region`, `site`, `lat`, `lon` | Node identification and geolocation |
| **Temporal** | `twindowstart`, `twindowend` | Operational time window |
| **CEIM Masses** | `mcapturedkg`, `membodiedkg`, `mpowerkgco2`, `mrefkg`, `ecobraw` | Carbon and material mass balance |
| **KER Core** | `kknowledge`, `eecoimpact`, `rriskofharm` | Knowledge, Eco-impact, Risk scores |
| **Risk Planes** | `renergy`, `rhydraulics`, `rbiology`, `rcarbon`, `rmaterials` | Primary risk coordinates |
| **Extended Risks** | `rpfas`, `recoli`, `rnutrient`, `rtds`, `rsat`, soil/aquatic toxins | Secondary risk metrics |
| **Lyapunov Weights** | `wenergy`, `whydraulics`, `wbiology`, `wcarbon`, `wmaterials` | Hazard ordering weights |
| **Residual** | `vresidual`, `vresidualmax` | Computed residual and threshold |
| **Trust & Adjustment** | `dttrust`, `badj`, `kadj`, `eadj` | Dynamic trust and calibration factors |
| **Corridor & Lane** | `corridorpresent`, `safestepok`, `lane` | Safety corridor and operational lane |
| **Security** | `securityresponsecap`, `fogroutingmode` | Response capability and routing mode |
| **Provenance** | `riskkernelversion`, `corridortableid`, `ceimkernelversion`, `cpvmkernelversion`, `evidencehex`, `signinghex` | Versioning and cryptographic proof |

---

## 3. Lane Semantics and KER Thresholds

### 3.1 RESEARCH Lane

- **Purpose:** Early-stage development, simulation, and algorithm exploration
- **Actuation:** ❌ **DISABLED** – Diagnostics only
- **Corridor Requirement:** Not required (but recommended for future promotion)
- **KER Guidelines:**
  - K: Any (typically < 0.90)
  - E: Any (typically < 0.90)
  - R: Any (typically > 0.15)

### 3.2 EXPERIMENTAL (PILOT) Lane

- **Purpose:** Field trials, limited deployment with human oversight
- **Actuation:** ⚠️ **LIMITED** – Requires explicit approval and monitoring
- **Corridor Requirement:** ✅ **REQUIRED**
- **KER Thresholds:**
  - K ≥ 0.90
  - E ≥ 0.90
  - R ≤ 0.15

### 3.3 PRODUCTION Lane

- **Purpose:** Full operational deployment with autonomous actuation
- **Actuation:** ✅ **ENABLED** – Subject to continuous corridor validation
- **Corridor Requirement:** ✅ **REQUIRED** with valid `corridortableid`
- **KER Thresholds:**
  - K ≥ 0.94
  - E ≥ 0.91
  - R ≤ 0.13
- **Additional Requirements:**
  - `vresidual ≤ vresidualmax`
  - `safestepok = true`
  - `securityresponsecap = MEDIUM` or `HIGH`

---

## 4. Admissibility Validation

A shard is **admissible** if and only if all of the following conditions are met:

1. **Corridor Present:** `corridorpresent = true` (except RESEARCH lane)
2. **Residual Within Bounds:** `vresidual ≤ vresidualmax`
3. **Lane KER Thresholds Met:** K, E, R satisfy lane-specific thresholds
4. **Safe Step OK:** `safestepok = true` (for actuation in PILOT/PRODUCTION)
5. **Valid Signature:** `signinghex` is non-empty and verifiable

**Validation Function:** `cyboquatic_industrial_shards::validate_admissibility()`

### Error Types

```rust
pub enum AdmissibilityError {
    CorridorMissing,
    ResidualExceeded { actual: f64, max: f64 },
    KerThresholdViolated { k: f64, e: f64, r: f64, lane: Lane },
    SafeStepNotOk,
    InvalidSignature,
}
```

---

## 5. Lane-Gated Controller Behavior

The `LaneGatedController<S, C>` wrapper enforces:

| Lane | `try_propose_step()` Behavior |
|------|-------------------------------|
| **RESEARCH** | Always returns `Err(LaneDoesNotPermitActuation)` – diagnostics only |
| **EXPERIMENTAL** | Returns command only if corridor present and KER thresholds met |
| **PRODUCTION** | Returns command if all admissibility checks pass |

**Rule:** "No corridor, no build; no lane, no actuation."

---

## 6. CI and Validation Pipeline

All changes to industrial crates trigger the following CI checks:

1. **Build Verification:** `cargo build -p cyboquatic-industrial-*`
2. **Schema Alignment:** ALN file presence and format validation
3. **Admissibility Tests:** Integration tests with fixture shards
4. **Corridor Checks:** Production fixtures must have `corridorpresent = true`
5. **Unsafe Code Audit:** Core and shards must be `#![forbid(unsafe_code)]`
6. **KER Summary:** Generate K/E/R trends for industrial plane

**Workflow:** `.github/workflows/ecosafety-industrial-ci.yml`

---

## 7. Toolchain Policy

All development and CI **MUST** use the Rust toolchain hosted on `/mnt/oss`:

- `RUSTUP_HOME = /mnt/oss/rustup`
- `CARGO_HOME = /mnt/oss/cargo`

**Wrapper Script:** `workspace/.tools/env-ecosafety.sh`

**Prohibited:**
- Installing Rust to root filesystem
- Using system `rustc` or `cargo` for this workspace
- Modifying `/usr` or `/usr/local` for Rust tooling

---

## 8. Research Turn Integration

When commits touch industrial shard or lane-gate code, a `ResponseShardEcoTurn` is generated:

```json
{
  "topic": "cyboquatic-industrial-shards: lane gate validation",
  "ker_impact": {
    "k_delta": 0.01,
    "e_delta": 0.00,
    "r_delta": -0.01
  },
  "summary": "Added integration tests for lane-gated actuation"
}
```

This ensures "just by researching it" eco-impact visibility in the coding loop.

---

## 9. Scores for This Family

| Metric | Value | Justification |
|--------|-------|---------------|
| **Knowledge-Factor (K)** | ≈ 0.95 | Directly grounded in existing ecosafety spine, lanes, and ALN patterns |
| **Eco-Impact (E)** | ≈ 0.91 | Enables governed, carbon-aware industrial automation |
| **Risk-of-Harm (R)** | ≈ 0.12 | Residual risk bounded by corridor calibration and CI gates |

---

## 10. Related Documents

- `ToolchainPolicyRustOnOss2026v1` – Rust-on-mnt/oss requirement spec
- `CyboquaticIndustrialEcosafety2026v1.aln` – ALN schema definition
- `ecosafety-core` – Shared ecosafety math and types
- `cyboquatic-industrial-ecosafety-core` – Industrial controller traits
- `cyboquatic-industrial-sim` – C FFI simulation bridge

---

**Last Updated:** 2026-01-01  
**Maintainer:** Ecosafety Governance Board  
**Audit Trail:** All changes tracked via Git + ALN evidence shards
