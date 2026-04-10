# Cyboquatic Materials Plane v1
## 1. Scope and Objectives

This document freezes the Cyboquatic materials plane grammar (“Materials Plane v1”) as a client of the ecosafety spine (Spine v1). It defines how biodegradable substrates and structural materials are modeled, normalized, and aggregated into a single materials risk coordinate r_materials ∈ [0, 1] that feeds the Lyapunov residual V_t and KER governance.

Materials Plane v1 permits future work to add new sub-metrics and kinetics models, but does not permit changes to the normalization and aggregation semantics that guarantee Lyapunov monotonicity and hard gating of unsafe substrates.

## 2. Physical and Risk Coordinates

### 2.1 Physical metrics

Each candidate material or substrate is characterized by a set of physical and ecological metrics, including:

- t_90: time to 90% mass loss under specified standard conditions;
- mass_loss_curve: time series of residual mass fraction S(t);
- tox_index: ecotoxicity index of leachate (e.g., LC50, NOEC);
- micro_residue: micro-residue count or concentration after standard exposure;
- leach_cec: cation exchange capacity of leachate;
- pfas_residue: residual PFAS or analogous persistent pollutant concentration;
- caloric_density: embedded energy per unit mass of substrate.

These metrics are obtained from laboratory tests, long-horizon simulations, and field measurements under controlled protocols.

### 2.2 Normalized sub-risk coordinates

Materials Plane v1 defines the following normalized sub-risk coordinates, all in [0, 1]:

- r_t90: risk induced by degradation time;
- r_tox: risk induced by leachate toxicity;
- r_micro: risk induced by micro-residue generation;
- r_leach_cec: risk induced by leachate cation exchange capacity;
- r_pfas: risk induced by persistent pollutant residue;
- r_caloric: risk induced by embedded energy and life-cycle emissions.

Each coordinate is produced by a corridor-based normalization kernel:

- r_sub = f_sub(x; safe, gold, hard)

where x is the physical metric, and safe, gold, hard are corridor thresholds stored in ecosafety.corridors.v2. All such kernels MUST be monotone non-decreasing in the harmful direction: higher x (worse outcome) MUST NOT reduce r_sub.

### 2.3 Composite materials risk

The composite materials risk coordinate r_materials is a monotone aggregation of the sub-risks:

- r_materials = g(r_t90, r_tox, r_micro, r_leach_cec, r_pfas, r_caloric)

Materials Plane v1 adopts a weighted quadratic aggregation:

- r_materials^2 = w_t90 r_t90^2 + w_tox r_tox^2
                  + w_micro r_micro^2 + w_leach r_leach_cec^2
                  + w_pfas r_pfas^2 + w_caloric r_caloric^2

where all weights w_* ≥ 0 and are stored in corridor tables. The composite coordinate is then:

- r_materials = sqrt(r_materials^2)

This function is monotone non-decreasing in each sub-coordinate, ensuring that worsening any sub-risk cannot lower r_materials or V_t.

## 3. MaterialKinetics and MaterialRisks

### 3.1 MaterialKinetics

MaterialKinetics captures the time evolution of material degradation and leachate properties:

- kinetic_model: model identifier (e.g., first_order, arrhenius);
- k_rate: decay rate constant(s);
- t_90: computed time to 90% degradation;
- env_params: environmental parameters (temperature, pH, moisture, flow regime);
- leach_profiles: time series of leachate composition, toxicity, and micro-residue;
- caloric_density: embedded energy per unit mass.

This struct is populated from standard tests and simulations. For first-order decay:

- dS/dt = −k S

t_90 is derived analytically or numerically and then used as input to the t_90 corridor.

### 3.2 MaterialRisks

MaterialRisks encapsulates the normalized risk coordinates derived from MaterialKinetics and associated lab data:

- r_t90: RiskCoord
- r_tox: RiskCoord
- r_micro: RiskCoord
- r_leach_cec: RiskCoord
- r_pfas: RiskCoord
- r_caloric: RiskCoord
- r_materials: RiskCoord
- corridor_ok: bool

corridor_ok is true if and only if each sub-risk is below its hard band and all mandatory corridors are defined. If corridor_ok is false, the material MUST be treated as non-deployable.

## 4. AntSafeSubstrateCorridorOk Trait

Materials Plane v1 defines a trait AntSafeSubstrateCorridorOk (or equivalently named guard) that is applied to substrate and material definitions:

- A substrate type S satisfies AntSafeSubstrateCorridorOk if:
  - its MaterialKinetics and MaterialRisks have been computed under the current corridor set; and
  - corridor_ok is true; and
  - r_materials is strictly below the hard band for the materials plane.

Controllers and runtime configuration loaders MUST only instantiate substrates that implement AntSafeSubstrateCorridorOk. Failure to satisfy this trait MUST result in a compile-time or load-time error. This shifts failure modes from field operations into laboratory and CI stages.

## 5. Integration with the Ecosafety Spine

### 5.1 RiskVector integration

Spine v1 RiskVector includes a dedicated r_materials coordinate. MaterialRisks.r_materials SHALL be used as the source of this coordinate. For composite hardware, r_materials MAY be computed from a mix of MaterialRisks for each component, using a weighted aggregation over mass fraction or surface contribution.

### 5.2 Lyapunov residual contribution

The global residual remains:

- V_t = Σ_j w_j r_j^2

Materials Plane v1 adds the term:

- w_materials r_materials^2

with w_materials ≥ 0 declared in ecosafety.corridors.v2. Because r_materials is monotone in each sub-risk and r_j ∈ [0, 1], the Lyapunov proofs for monotone non-increase and boundedness remain valid when materials are included.

### 5.3 KER governance

The impact of materials on KER is explicit:

- K is unchanged in definition but may decrease if more steps are rejected due to materials-plane violations.
- R increases when r_materials approaches its hard band, directly reflecting slow degradation, high toxicity, or high micro-residue risk.
- E (e.g., 1 − R) decreases when r_materials or any other coordinate approaches 1, making high-risk material selections visibly degrade eco-impact.

CICD gates MUST enforce:

- no corridor, no build: if any required material corridor is missing, the build fails;
- violated corridor → derate/stop: if r_materials or its sub-risks breach hard bands, safestep MUST return Stop and kerdeployable MUST be false for that configuration.

## 6. ALN Representation

Materials Plane v1 requires ALN particles for material kinetics and risks. An example:

- ecosafety.material_kinetics.v1
- ecosafety.material_risks.v1

These shards bind MaterialKinetics and MaterialRisks to spec hashes, corridor sets, and hexstamps, and serve as evidence for material approval decisions. qpudatashards for runtime nodes MUST reference the material IDs and their associated r_materials values.

## 7. Versioning

Materials Plane v1 is identified by:

- spec_id: "Cyboquatic.Ecosafety.MaterialsPlane.v1"
- spec_hash: <to be computed>
- version: 1.0.0

Changes to aggregation semantics (e.g., different g()) or the definition of r_materials require a new materials plane version. Adding new sub-risks or refining corridor values is allowed within Materials Plane v1, provided monotonicity and Lyapunov properties are preserved.
