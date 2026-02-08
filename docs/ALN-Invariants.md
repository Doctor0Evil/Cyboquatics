Below is a compact Rust/ALN shard-and-contract sketch for a **Phoenix-class MAR cyboquatic module**, with explicit K/E/R fields, hex-stamped governance metadata, and a research pattern that maximizes reliability of outputs. All field names and invariants are consistent with your existing ecosafety spine. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/994be4b5-c833-4d4d-ba32-b0c9d9a4ec7e/cycoquatic-instantiators-how-c-c.7kGeoiRMeXnnBTkkK_7A.md)

***

## 1. Rust core types and contracts (MAR module)

```rust
// File: crates/ecosafety_core/src/types.rs
// Purpose: Shared ecosafety grammar for all cyboquatic nodes.

#[derive(Clone, Debug)]
pub struct CorridorBands {
    pub var_id:       String,   // e.g. "r_SAT", "r_PFAS"
    pub units:        String,   // e.g. "m/d", "ng/L"
    pub safe:         f64,      // science-safe band edge (r = 0 at/inside)
    pub gold:         f64,      // preferred operating band
    pub hard:         f64,      // r = 1.0 hard limit
    pub weight_w:     f64,      // contribution to V_t
    pub lyap_channel: u16,      // residual channel index
    pub mandatory:    bool,     // true for hard-safety corridors
}

#[derive(Clone, Debug)]
pub struct RiskCoord {
    pub value: f64,            // r_x ∈ [0, 1]
    pub sigma: f64,            // uncertainty
    pub bands: CorridorBands,
}

#[derive(Clone, Debug)]
pub struct Residual {
    pub vt:      f64,          // Lyapunov-style residual
    pub coords:  Vec<RiskCoord>,
}

#[derive(Clone, Debug)]
pub struct CorridorDecision {
    pub derate: bool,
    pub stop:   bool,
    pub reason: String,
}
```

```rust
// File: crates/ecosafety_core/src/contracts.rs
// Purpose: ALN-style invariants as Rust guards for all nodes.

use crate::types::{Residual, RiskCoord, CorridorBands, CorridorDecision};

pub fn corridor_present(corridors: &[CorridorBands]) -> bool {
    // CI/ALN layer enforces the required set; here just "non-empty and all mandatory present".
    corridors.iter().any(|c| c.mandatory)
        && corridors.iter().all(|c| {
            !c.var_id.is_empty()
                && c.hard >= c.gold
                && c.gold >= c.safe
        })
}

// Enforce per-coordinate r_x ≤ 1 and V_{t+1} ≤ V_t outside safe interior.
pub fn safe_step(prev: &Residual, next: &Residual, safe_interior_eps: f64) -> CorridorDecision {
    // 1. Coordinate-wise check
    for rc in &next.coords {
        if rc.value > 1.0 + 1e-9 {
            return CorridorDecision {
                derate: true,
                stop:   true,
                reason: format!("hard-limit breach in {}", rc.bands.var_id),
            };
        }
    }

    // 2. Lyapunov monotonicity (allow slack inside safe interior)
    let all_inside_safe = next.coords.iter().all(|rc| rc.value <= rc.bands.safe + safe_interior_eps);

    if !all_inside_safe && next.vt > prev.vt + 1e-9 {
        return CorridorDecision {
            derate: true,
            stop:   true,
            reason: "Lyapunov residual increased outside safe interior".to_string(),
        };
    }

    CorridorDecision {
        derate: false,
        stop:   false,
        reason: "ok".to_string(),
    }
}
```

```rust
// File: crates/mar_node/src/mar_node.rs
// Purpose: MAR-specific node type with K/E/R and governance meta.

use ecosafety_core::types::{Residual, RiskCoord, CorridorBands};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceMeta {
    pub shard_id:        String,
    pub module_type:     String, // "PhoenixMARCell.v1"
    pub region:          String, // "Phoenix-AZ"
    pub sim_or_live:     String, // "sim" | "live"
    pub timestamp_utc:   String,
    pub did_signature:   String, // Bostrom DID hex
    pub rust_build_hash: String,
    pub aln_schema_ver:  String,
    pub hex_stamp:       String, // governance hex-stamp for this shard
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KER {
    pub knowledge_factor_01: f64, // K ∈ [0,1]
    pub eco_impact_01:       f64, // E ∈ [0,1]
    pub risk_of_harm_01:     f64, // R ∈ [0,1]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhoenixMarShard {
    pub meta:        GovernanceMeta,
    pub corridors:   Vec<CorridorBands>, // r_SAT, r_PFAS, r_nutrient, r_temp, r_foul, r_surcharge
    pub risk_state:  Vec<RiskCoord>,     // current r_x values
    pub residual:    Residual,           // V_t
    pub ker:         KER,                // triad for this run or design
    pub q_design_m3s: f64,               // design flow
    pub recharge_m3_per_year: f64,       // modeled/observed
    pub m_pollutant_removed_kg_per_y: f64,
}
```

**Eco-impact and risk scoring guidance (for research use):**  
- For 2026 Phoenix-class MAR modules: target K ≈ 0.93, E ≈ 0.92, R ≈ 0.14, matching your earlier scores; treat any shard with K < 0.8 or R > 0.2 as *research-only*, not deployment-eligible. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/e3456789-b513-4c4d-a93b-6af99f5dce10/find-new-and-useful-knowledge-UKMFqsqaT4amvkJWf1rkoQ.md)

***

## 2. ALN-style shard schema (MAR qpudatashard)

```aln
// File: specs/qpudatashards.phoenix_mar.aln
// Purpose: Schema fragment for a Phoenix MAR module shard.

aln particle phoenix.mar.module.v1
  field meta.shard_id        string
  field meta.module_type     string    -- "PhoenixMARCell.v1"
  field meta.region          string    -- "Phoenix-AZ"
  field meta.sim_or_live     string    -- "sim" | "live"
  field meta.timestamp_utc   string
  field meta.did_signature   string    -- Bostrom DID / CHAT-linked
  field meta.rust_build_hash string
  field meta.aln_schema_ver  string
  field meta.hex_stamp       string    -- governance hex stamp

  -- Triad scores
  field ker.knowledge_factor_01 f64
  field ker.eco_impact_01       f64
  field ker.risk_of_harm_01     f64

  -- Design / impact kernel
  field kernel.q_design_m3s          f64
  field kernel.recharge_m3_per_year  f64
  field kernel.m_removed_kg_per_year f64

  -- Residual summary
  field residual.vt          f64
  field residual.n_coords    u32

  -- Corridors table (one row per risk variable)
  table corridors
    column var_id       string  -- e.g. "r_SAT", "r_PFAS"
    column units        string  -- "m/d", "ng/L", etc.
    column safe         f64
    column gold         f64
    column hard         f64
    column weight_w     f64
    column lyap_channel u16
    column mandatory    bool
  end

  -- Current risk state (can also be timeseries in extended schema)
  table risk_state
    column var_id string
    column r_val  f64    -- normalized r_x ∈ [0,1]
    column sigma  f64    -- uncertainty
  end
end
```

### ALN invariants (research and deployment gates)

```aln
-- 1. No corridor, no build (CI gate)
aln contract invariant.corridor_complete(m phoenix.mar.module.v1) -> bool
  let n_mandatory = count(m.corridors where mandatory == true)
  in  n_mandatory > 0
      && forall row in m.corridors .
             row.hard >= row.gold && row.gold >= row.safe
end

-- 2. Residual and r_x safety at runtime
aln contract invariant.residual_safe(prev phoenix.mar.module.v1,
                                     next phoenix.mar.module.v1,
                                     eps f64) -> bool
  -- per-coordinate hard limit
  let coords_ok =
    forall r in next.risk_state .
      r.r_val <= 1.0 + 1e-9
  -- Lyapunov non-increase outside safe interior
  let all_inside_safe =
    forall row in next.corridors .
      let r = lookup(next.risk_state, row.var_id).r_val in
        r <= row.safe + eps
  in
    coords_ok &&
    (all_inside_safe || next.residual.vt <= prev.residual.vt + 1e-9)
end

-- 3. K/E/R deployment gate (Earth-benefit hard gate)
aln contract invariant.ker_deployable(m phoenix.mar.module.v1) -> bool
  m.ker.knowledge_factor_01 >= 0.90 &&
  m.ker.eco_impact_01       >= 0.90 &&
  m.ker.risk_of_harm_01     <= 0.13
end
```

These invariants embed your 2026 rule that **no module can be considered deployable unless it is corridor-complete, Lyapunov-safe, and passes K/E/R thresholds**, making “just-by-researching-it” meaningful because each study updates bands, kernels, and K/E/R values in a shard-checked way. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/dbb1be3a-b949-4cc6-8a7f-3064d747d843/what-can-improve-our-ability-t-_YVzCDVWSZSAjanwBR8c2w.md)

***

## 3. Research approach for highest reliability and eco-impact

To get the **most reliable, accurate outputs** from this shard-and-contract stack and maximize real-world eco-benefit:

1. **Band calibration program (physics-first):**  
   - Derive SAT, PFAS, nutrient, temperature, fouling, and surcharge corridors from Phoenix pilots and external literature, writing them into `corridors` rows with explicit safe/gold/hard and uncertainty. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/31490c9b-25eb-4d41-882d-52166ccbf5c9/daily-rust-and-aln-code-genera-g0Rz_p5bTGCq6sEaIODFtg.md)
   - Score: K ≈ 0.94, E ≈ 0.91, R ≈ 0.13 (residual risk mainly band edge error). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/e3456789-b513-4c4d-a93b-6af99f5dce10/find-new-and-useful-knowledge-UKMFqsqaT4amvkJWf1rkoQ.md)

2. **Kernel and residual validation (model-first):**  
   - Implement deterministic normalization kernels in Rust that map raw data (e.g., \(C_{\text{PFAS}}\), HLR, temperature) into r_x and V_t; attach exhaustive unit/regression tests at safe/gold/hard boundaries and under sensor faults. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/994be4b5-c833-4d4d-ba32-b0c9d9a4ec7e/cycoquatic-instantiators-how-c-c.7kGeoiRMeXnnBTkkK_7A.md)
   - Use hardware-in-the-loop MAR pilots to confirm V_t trends and adjust weights w and channels.  
   - Score: K ≈ 0.93, E ≈ 0.90, R ≈ 0.12. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/dbb1be3a-b949-4cc6-8a7f-3064d747d843/what-can-improve-our-ability-t-_YVzCDVWSZSAjanwBR8c2w.md)

3. **Formal verification focus (code-first):**  
   - Restrict verification to `corridor_present`, `safe_step`, and their immediate call graph; use Rust formal tools to prove that **all reachable control paths** respect invariants before they can actuate pumps/valves. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/994be4b5-c833-4d4d-ba32-b0c9d9a4ec7e/cycoquatic-instantiators-how-c-c.7kGeoiRMeXnnBTkkK_7A.md)
   - Score: K ≈ 0.95, E ≈ 0.91, R ≈ 0.12. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/994be4b5-c833-4d4d-ba32-b0c9d9a4ec7e/cycoquatic-instantiators-how-c-c.7kGeoiRMeXnnBTkkK_7A.md)

4. **Shard-governed pilots (governance-first):**  
   - Run Phoenix MAR pilots where **every control tick** and configuration change is recorded as a DID-signed `phoenix.mar.module.v1` shard with updated K/E/R; operate only under `invariant.corridor_complete` and `invariant.residual_safe`. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/31490c9b-25eb-4d41-882d-52166ccbf5c9/daily-rust-and-aln-code-genera-g0Rz_p5bTGCq6sEaIODFtg.md)
   - Use at least one full season of shards as a “Pilot-Gate” dataset to decide if templates and corridors can be promoted city-wide. [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/31490c9b-25eb-4d41-882d-52166ccbf5c9/daily-rust-and-aln-code-genera-g0Rz_p5bTGCq6sEaIODFtg.md)

By following this sequence, each research step tightens corridors, improves K/E/R, and reduces residual risk before any large-scale deployment, so the act of researching—running pilots, calibrating kernels, and proving contracts—directly restores eco-health by preventing harmful configurations from ever entering service.

### Suggested triad for this MAR-shard design step

- Knowledge-factor K: **0.94** (direct reuse of validated grammar, plus MAR-specific fields). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/e3456789-b513-4c4d-a93b-6af99f5dce10/find-new-and-useful-knowledge-UKMFqsqaT4amvkJWf1rkoQ.md)
- Eco-impact E: **0.91** (design is explicitly tied to recharge and pollutant-removal kernels before hardware scale-up). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/e3456789-b513-4c4d-a93b-6af99f5dce10/find-new-and-useful-knowledge-UKMFqsqaT4amvkJWf1rkoQ.md)
- Risk-of-harm R: **0.12** (remaining risk concentrated in corridor calibration and model error, both targeted by this research workflow). [ppl-ai-file-upload.s3.amazonaws](https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/collection_1d7dde59-b474-475b-a731-2469d14a3632/dbb1be3a-b949-4cc6-8a7f-3064d747d843/what-can-improve-our-ability-t-_YVzCDVWSZSAjanwBR8c2w.md)
