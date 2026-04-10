# Cyboquatic Ecosafety Spine v1
## 1. Scope and Objectives

This document freezes the Cyboquatic ecosafety spine grammar (“Spine v1”). It defines the normalized risk coordinate space, Lyapunov residual, KER governance metrics, and Atomic Ledger Notation (ALN) particles that constitute the non-negotiable ecosafety layer for all Cyboquatic workloads.

Spine v1 permits future work to add new risk coordinates, plane-specific kernels, and corridor bands, but does not permit changes to the residual form, safestep invariant, or KER aggregation semantics.

## 2. Core Types and Semantics

### 2.1 Normalized risk coordinate

A normalized risk coordinate is a dimensionless scalar in the closed interval [0, 1], denoted r_x. The mapping from physical units to r_x is defined by a corridor with three bands:

- safe: target or best-practice operating band;
- gold: acceptable operating band;
- hard: maximum permissible band, above which operation is prohibited.

For all coordinates, r_x = 0 represents the lowest risk state accessible to the system; r_x = 1 represents a hard failure or prohibited state. All risk coordinates used by controllers MUST be clamped into [0, 1] before aggregation.

### 2.2 Risk vector

A RiskVector is an ordered tuple of normalized risk coordinates covering the active planes of the ecosafety spine. Spine v1 fixes the following canonical planes:

- energy: r_energy
- hydraulics: r_hydraulics
- biology: r_biology
- carbon: r_carbon
- materials: r_materials
- biodiversity: r_biodiversity
- data quality: r_sigma (optional but recommended)

A concrete deployment MAY extend RiskVector with additional coordinates (e.g., pollutant-specific energy per mass, species-specific risk) as long as they are normalized into [0, 1] via declared corridors and recorded in ALN.

### 2.3 Lyapunov residual

The Lyapunov residual V_t is a scalar function of the current risk vector r(t) at discrete time t:

- V_t = Σ_j w_j r_j(t)^2

where:

- r_j(t) are the individual components of the RiskVector at time t;
- w_j ≥ 0 are fixed weights per coordinate that reflect the relative severity of each risk component.

Weights are parameters of the deployment and are stored in versioned ALN corridor tables. Spine v1 requires that all coordinates contributing to V_t have explicitly declared weights.

V_t is interpreted as a scalar potential measuring proximity to unsafe regions in the multi-plane risk space. Lower V_t indicates safer operation. V_t = 0 is only reachable if all r_j = 0.

### 2.4 Safestep invariant

Spine v1 enforces a discrete-time Lyapunov-type invariant over all actuation steps. For each proposed step from time t to t+1:

- V_{t+1} ≤ V_t + ε

where ε ≥ 0 is a small tolerance for numerical and modeling error. Deployments MAY choose ε = 0 for strict monotonic non-increase outside a defined safe interior. When the system is within a designated safe interior (e.g., all r_j below a conservative threshold), the residual may be allowed to fluctuate within a small bounded region while still satisfying safety requirements.

The ecosafety core defines a function safestep(prev, next, rv_next, weights) that classifies a step as:

- Accept: step is allowed; residual is non-increasing within tolerance;
- Derate: step is allowed but indicates approaching corridor edges;
- Stop: step is rejected; V_{t+1} exceeds the permitted bound or a hard corridor is breached.

Any actuation that would result in Stop MUST NOT reach physical actuators.

### 2.5 No action without risk estimate

Spine v1 codifies the principle “no action without a risk estimate” as a type-level rule:

- Every controller implementing SafeController MUST, for each proposed actuation, simultaneously produce:
  - an actuation proposal; and
  - a complete RiskVector and candidate Residual V_{t+1}.

The ecosafety kernel evaluates safestep before any actuation is applied. Controllers that do not furnish a RiskVector and Residual for each step MUST be rejected at compile-time or load-time.

## 3. KER Governance Triad

Spine v1 defines a governance triad KER over a rolling time window W:

- Knowledge factor K: fraction of actuation steps in W that satisfy the safestep invariant and corridor constraints (i.e., steps classified as Accept or Derate).
- Eco-impact E: an eco-impact score derived from the complement of the worst normalized risk coordinate over W, or from a domain-specific eco-metric normalized to [0, 1]. A simple default is E = 1 − R, where R is as defined below.
- Risk of harm R: the maximum normalized risk coordinate observed in W.

Values are in [0, 1]. Lower R is better, higher K and E are better.

Spine v1 establishes the following reference bands:

- Research band (Phoenix 2026 reference): K ≈ 0.94, E ≈ 0.90–0.91, R ≈ 0.11–0.14.
- Production gate: K ≥ 0.90, E ≥ 0.90, R ≤ 0.13.

Any crate, controller, or deployment lane used for production MUST demonstrate KER in or above these gates over defined qualification scenarios. Artifacts that fail these criteria MUST be restricted to research-only lanes and prevented from influencing physical actuators.

## 4. Materials Plane v1

The materials plane aggregates substrate and hardware material risks into a single coordinate r_materials ∈ [0, 1]. It is derived from sub-metrics including, but not limited to:

- t90: time to 90% degradation under standard conditions;
- r_tox: normalized toxicity of breakdown products;
- r_micro: normalized micro-residue concentration;
- r_CEC: normalized leachate cation exchange capacity;
- r_PFAS: normalized residual PFAS or other persistent pollutants.

Material kinetics are captured in MaterialKinetics structs populated from laboratory and simulation data. Material risks are summarized in MaterialRisks and then aggregated into r_materials via a monotone function (e.g., weighted quadratic).

The trait AntSafeSubstrateCorridorOk (or equivalent) defines a hard gate:

- If any sub-risk exceeds its hard corridor band, or t90 exceeds its maximum allowed value, instantiation of the material in a Cyboquatic node MUST be rejected at compile/load time. Such materials remain research-only.

## 5. ALN Particles and Shard Surfaces

Spine v1 standardizes the following ALN particles:

### 5.1 ecosafety.corridors.v2

Captures corridor definitions for each risk coordinate:

- var_id: string (coordinate identifier)
- safe: f64 (safe-band threshold)
- gold: f64 (gold-band threshold)
- hard: f64 (hard-band threshold)
- weight: f64 (Lyapunov weight w_j)
- lyap_channel: string (plane/channel identifier, e.g., "energy", "hydraulics")
- mandatory: bool (if true, coordinate must be present for deployment)

### 5.2 ecosafety.riskvector.v2

Records normalized risk vector and associated metrics per step:

- r_energy, r_hydraulics, r_biology, r_carbon, r_materials, r_biodiversity, r_sigma: f64
- V_t: f64 (residual at current step)
- k_metric: f64 (current K estimate over window)
- e_metric: f64 (current E estimate over window)
- r_metric: f64 (current R estimate over window)

### 5.3 qpudatashard templates

Deployment- and workload-specific shards MUST extend ecosafety.riskvector.v2 with:

- node identifiers and geostamps;
- raw physical telemetry used to compute each risk coordinate;
- spec hash and hexstamp binding the shard to the Spine v1 spec and corridor sets.

qpudatashards MUST be RFC4180-compliant CSV or ALN-compatible tabular formats and MUST be used as the authoritative ledger for ecosafety evidence.

## 6. Versioning and Governance

Spine v1 is identified by:

- spec_id: "Cyboquatic.Ecosafety.Spine.v1"
- spec_hash: <to be computed>
- version: 1.0.0

Any change to:

- the form of V_t;
- the safestep invariant;
- the semantics of K, E, or R;
- the ALN particle definitions referenced above

MUST result in a new spec_id and spec_hash (e.g., Spine v2). Plane-specific kernels, corridor values, and additional coordinates MAY evolve under Spine v1, provided they are defined in ALN and their outputs remain normalized risk coordinates aggregated by the same V_t and KER definitions.

All controller and workload crates participating in Cyboquatic ecosafety MUST depend on the Spine v1 core crate or a later spine version and MUST NOT introduce alternate residual definitions or bypass safestep.
