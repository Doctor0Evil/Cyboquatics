// brain_identity_residual.hpp
// Numeric-only mirror of Rust brain-identity residual for long-horizon simulation.
// Owns no corridors; they are passed in from Rust/ALN.
// Rust/ALN remains the single source of truth for corridors and admissibility.

#pragma once

#include <array>
#include <cmath>
#include <cstdint>

namespace bi_ecosafety {

using scalar = float;
using step_index = uint32_t;
using identity_id = std::array<uint8_t, 32>;
using hex_stamp = std::array<uint8_t, 32>;

enum class NeurorightsStatus : uint8_t {
    Active = 0,
    Restricted = 1,
    Suspended = 2
};

enum class EvidenceMode : uint8_t {
    Redacted = 0,
    HashOnly = 1,
    FullTrace = 2
};

enum class BiSafeStepDecision : uint8_t {
    Accept = 0,
    Derate = 1,
    Stop = 2,
    StopKarmaViolation = 3
};

struct BrainIdentityState {
    identity_id brainidentityid;
    hex_stamp hexstamp;
    scalar ecoimpactscore;
    NeurorightsStatus neurorights_status;
    scalar karma_floor;
    uint8_t data_sensitivity_level;
    EvidenceMode evidence_mode;
    scalar rsoul_residual;
    scalar social_exposure_coord;
};

struct BiRiskVector {
    scalar r_energy;
    scalar r_hydraulic;
    scalar r_biology;
    scalar r_carbon;
    scalar r_materials;
    scalar r_neurorights;
    scalar r_soul;
    scalar r_social;
    scalar r_ecoimpact;
};

struct BiLyapunovWeights {
    scalar w_energy;
    scalar w_hydraulic;
    scalar w_biology;
    scalar w_carbon;
    scalar w_materials;
    scalar w_neurorights;
    scalar w_soul;
    scalar w_social;
    scalar w_ecoimpact;
};

struct BiResidual {
    scalar vt;
};

inline scalar neurorights_to_risk(NeurorightsStatus status) {
    switch (status) {
        case NeurorightsStatus::Active: return 0.0f;
        case NeurorightsStatus::Restricted: return 0.5f;
        case NeurorightsStatus::Suspended: return 1.0f;
        default: return 1.0f;
    }
}

inline scalar clamp(scalar v, scalar min, scalar max) {
    return v < min ? min : (v > max ? max : v);
}

inline BiRiskVector build_bi_risk_vector(
    const BiRiskVector& physical,
    const BrainIdentityState& bi
) {
    BiRiskVector rv;
    rv.r_energy = physical.r_energy;
    rv.r_hydraulic = physical.r_hydraulic;
    rv.r_biology = physical.r_biology;
    rv.r_carbon = physical.r_carbon;
    rv.r_materials = physical.r_materials;
    rv.r_neurorights = neurorights_to_risk(bi.neurorights_status);
    rv.r_soul = clamp(bi.rsoul_residual, 0.0f, 1.0f);
    rv.r_social = clamp(bi.social_exposure_coord, 0.0f, 1.0f);
    rv.r_ecoimpact = clamp(bi.ecoimpactscore, 0.0f, 1.0f);
    return rv;
}

inline BiResidual compute_bi_vt(const BiRiskVector& rv, const BiLyapunovWeights& w) {
    BiResidual res;
    res.vt =
        w.w_energy * rv.r_energy * rv.r_energy +
        w.w_hydraulic * rv.r_hydraulic * rv.r_hydraulic +
        w.w_biology * rv.r_biology * rv.r_biology +
        w.w_carbon * rv.r_carbon * rv.r_carbon +
        w.w_materials * rv.r_materials * rv.r_materials +
        w.w_neurorights * rv.r_neurorights * rv.r_neurorights +
        w.w_soul * rv.r_soul * rv.r_soul +
        w.w_social * rv.r_social * rv.r_social +
        w.w_ecoimpact * rv.r_ecoimpact * rv.r_ecoimpact;
    return res;
}

struct BiSafeStepConfig {
    scalar epsilon;
    bool enforce_karma_nonslash;
};

inline BiSafeStepDecision bi_safestep_numeric(
    BiResidual prev_residual,
    BiResidual next_residual,
    const BiRiskVector& rv_next,
    scalar prev_karma,
    scalar proposed_karma,
    const BiSafeStepConfig& cfg
) {
    if (cfg.enforce_karma_nonslash && proposed_karma < prev_karma) {
        return BiSafeStepDecision::StopKarmaViolation;
    }

    const scalar hard_threshold = 1.0f;
    bool any_hard =
        rv_next.r_energy >= hard_threshold ||
        rv_next.r_hydraulic >= hard_threshold ||
        rv_next.r_biology >= hard_threshold ||
        rv_next.r_carbon >= hard_threshold ||
        rv_next.r_materials >= hard_threshold ||
        rv_next.r_neurorights >= hard_threshold ||
        rv_next.r_soul >= hard_threshold ||
        rv_next.r_social >= hard_threshold ||
        rv_next.r_ecoimpact >= hard_threshold;

    if (any_hard) {
        return BiSafeStepDecision::Stop;
    }

    if (next_residual.vt <= prev_residual.vt + cfg.epsilon) {
        return BiSafeStepDecision::Accept;
    } else {
        return BiSafeStepDecision::Derate;
    }
}

struct BiKerWindow {
    step_index steps;
    step_index safe_steps;
    scalar max_r;
    bool karma_preserved;

    BiKerWindow() : steps(0), safe_steps(0), max_r(0.0f), karma_preserved(true) {}

    void update(const BiRiskVector& rv, BiSafeStepDecision decision, bool karma_ok) {
        steps += 1;
        if (decision == BiSafeStepDecision::Accept) {
            safe_steps += 1;
        }
        karma_preserved = karma_preserved && karma_ok;

        scalar current_max = max_r;
        current_max = current_max > rv.r_energy ? current_max : rv.r_energy;
        current_max = current_max > rv.r_hydraulic ? current_max : rv.r_hydraulic;
        current_max = current_max > rv.r_biology ? current_max : rv.r_biology;
        current_max = current_max > rv.r_carbon ? current_max : rv.r_carbon;
        current_max = current_max > rv.r_materials ? current_max : rv.r_materials;
        current_max = current_max > rv.r_neurorights ? current_max : rv.r_neurorights;
        current_max = current_max > rv.r_soul ? current_max : rv.r_soul;
        current_max = current_max > rv.r_social ? current_max : rv.r_social;
        current_max = current_max > rv.r_ecoimpact ? current_max : rv.r_ecoimpact;
        max_r = current_max;
    }

    scalar k() const {
        return steps == 0 ? 1.0f : static_cast<scalar>(safe_steps) / static_cast<scalar>(steps);
    }

    scalar r() const {
        return max_r;
    }

    scalar e() const {
        return 1.0f - max_r;
    }

    bool bi_ker_deployable() const {
        return k() >= 0.90f && e() >= 0.90f && r() <= 0.13f && karma_preserved;
    }
};

struct BiAuditRecord {
    step_index step;
    identity_id brainidentityid;
    hex_stamp hexstamp;
    scalar vt_previous;
    scalar vt_current;
    scalar vt_delta;
    BiSafeStepDecision decision;
    scalar karma_floor_before;
    scalar karma_floor_after;
    bool ker_deployable;
    uint64_t timestamp_unix;
};

struct BiSimulationReport {
    step_index total_steps;
    step_index completed_steps;
    scalar avg_vt_physical;
    scalar avg_vt_bi;
    scalar final_karma_floor;
    uint32_t karma_violations_detected;
    uint32_t karma_violations_enforced;
    uint32_t vt_violations;
    uint32_t hard_violations;
    uint32_t ker_violations;
    uint32_t accept_count;
    uint32_t derate_count;
    uint32_t stop_count;
    scalar final_k_score;
    scalar final_e_score;
    scalar final_r_score;
    bool final_deployable;
    bool invariant_held;
};

inline BiLyapunovWeights default_bi_weights() {
    BiLyapunovWeights w;
    w.w_energy = 0.15f;
    w.w_hydraulic = 0.10f;
    w.w_biology = 0.10f;
    w.w_carbon = 0.15f;
    w.w_materials = 0.10f;
    w.w_neurorights = 0.15f;
    w.w_soul = 0.10f;
    w.w_social = 0.08f;
    w.w_ecoimpact = 0.07f;
    return w;
}

inline BiRiskVector default_physical_risk_vector() {
    BiRiskVector rv;
    rv.r_energy = 0.30f;
    rv.r_hydraulic = 0.20f;
    rv.r_biology = 0.25f;
    rv.r_carbon = 0.35f;
    rv.r_materials = 0.28f;
    rv.r_neurorights = 0.0f;
    rv.r_soul = 0.0f;
    rv.r_social = 0.0f;
    rv.r_ecoimpact = 0.0f;
    return rv;
}

inline BrainIdentityState default_bi_state(
    const identity_id& id,
    const hex_stamp& stamp,
    scalar initial_karma
) {
    BrainIdentityState state;
    state.brainidentityid = id;
    state.hexstamp = stamp;
    state.ecoimpactscore = 0.0f;
    state.neurorights_status = NeurorightsStatus::Active;
    state.karma_floor = initial_karma;
    state.data_sensitivity_level = 1;
    state.evidence_mode = EvidenceMode::Redacted;
    state.rsoul_residual = 0.0f;
    state.social_exposure_coord = 0.0f;
    return state;
}

} // namespace bi_ecosafety
