#![no_std]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]

extern crate alloc;

use cyboquatic_ecosafety_core::{RiskCoord, Scalar, Residual, RiskVector, LyapunovWeights};

pub type BrainIdentityId = [u8; 32];
pub type HexStamp = [u8; 32];

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NeurorightsStatus {
    Active = 0,
    Restricted = 1,
    Suspended = 2,
}

impl NeurorightsStatus {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(NeurorightsStatus::Active),
            1 => Some(NeurorightsStatus::Restricted),
            2 => Some(NeurorightsStatus::Suspended),
            _ => None,
        }
    }

    pub fn risk_coord(self) -> RiskCoord {
        match self {
            NeurorightsStatus::Active => RiskCoord::new_clamped(0.0),
            NeurorightsStatus::Restricted => RiskCoord::new_clamped(0.5),
            NeurorightsStatus::Suspended => RiskCoord::new_clamped(1.0),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EvidenceMode {
    Redacted = 0,
    HashOnly = 1,
    FullTrace = 2,
}

impl EvidenceMode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(EvidenceMode::Redacted),
            1 => Some(EvidenceMode::HashOnly),
            2 => Some(EvidenceMode::FullTrace),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BrainIdentityShard {
    pub brainidentityid: BrainIdentityId,
    pub hexstamp: HexStamp,
    pub ecoimpactscore: Scalar,
    pub neurorights_status: NeurorightsStatus,
    pub karma_floor: Scalar,
    pub data_sensitivity_level: u8,
    pub evidence_mode: EvidenceMode,
    pub rsoul_residual: Scalar,
    pub social_exposure_coord: Scalar,
}

impl BrainIdentityShard {
    pub fn new(
        brainidentityid: BrainIdentityId,
        hexstamp: HexStamp,
        karma_floor: Scalar,
    ) -> Self {
        BrainIdentityShard {
            brainidentityid,
            hexstamp,
            ecoimpactscore: 0.0,
            neurorights_status: NeurorightsStatus::Active,
            karma_floor,
            data_sensitivity_level: 1,
            evidence_mode: EvidenceMode::Redacted,
            rsoul_residual: 0.0,
            social_exposure_coord: 0.0,
        }
    }

    pub fn update_ecoimpactscore(&mut self, score: Scalar) {
        self.ecoimpactscore = if score < 0.0 { 0.0 } else if score > 1.0 { 1.0 } else { score };
    }

    pub fn update_neurorights(&mut self, status: NeurorightsStatus) {
        self.neurorights_status = status;
    }

    pub fn try_set_karma_floor(&mut self, new_floor: Scalar) -> bool {
        if new_floor >= self.karma_floor {
            self.karma_floor = new_floor;
            true
        } else {
            false
        }
    }

    pub fn update_rsoul(&mut self, residual: Scalar) {
        self.rsoul_residual = if residual < 0.0 { 0.0 } else if residual > 1.0 { 1.0 } else { residual };
    }

    pub fn update_social_exposure(&mut self, coord: Scalar) {
        self.social_exposure_coord = if coord < 0.0 { 0.0 } else if coord > 1.0 { 1.0 } else { coord };
    }

    pub fn r_neurorights(&self) -> RiskCoord {
        self.neurorights_status.risk_coord()
    }

    pub fn r_soul(&self) -> RiskCoord {
        RiskCoord::new_clamped(self.rsoul_residual)
    }

    pub fn r_social(&self) -> RiskCoord {
        RiskCoord::new_clamped(self.social_exposure_coord)
    }

    pub fn r_ecoimpact(&self) -> RiskCoord {
        RiskCoord::new_clamped(self.ecoimpactscore)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BiLyapunovWeights {
    pub w_energy: Scalar,
    pub w_hydraulic: Scalar,
    pub w_biology: Scalar,
    pub w_carbon: Scalar,
    pub w_materials: Scalar,
    pub w_neurorights: Scalar,
    pub w_soul: Scalar,
    pub w_social: Scalar,
    pub w_ecoimpact: Scalar,
}

impl Default for BiLyapunovWeights {
    fn default() -> Self {
        BiLyapunovWeights {
            w_energy: 0.15,
            w_hydraulic: 0.10,
            w_biology: 0.10,
            w_carbon: 0.15,
            w_materials: 0.10,
            w_neurorights: 0.15,
            w_soul: 0.10,
            w_social: 0.08,
            w_ecoimpact: 0.07,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct BiRiskVector {
    pub r_energy: RiskCoord,
    pub r_hydraulic: RiskCoord,
    pub r_biology: RiskCoord,
    pub r_carbon: RiskCoord,
    pub r_materials: RiskCoord,
    pub r_neurorights: RiskCoord,
    pub r_soul: RiskCoord,
    pub r_social: RiskCoord,
    pub r_ecoimpact: RiskCoord,
}

impl BiRiskVector {
    pub fn from_physical_and_bi(
        physical: &RiskVector,
        bi: &BrainIdentityShard,
    ) -> Self {
        BiRiskVector {
            r_energy: physical.r_energy,
            r_hydraulic: physical.r_hydraulic,
            r_biology: physical.r_biology,
            r_carbon: physical.r_carbon,
            r_materials: physical.r_materials,
            r_neurorights: bi.r_neurorights(),
            r_soul: bi.r_soul(),
            r_social: bi.r_social(),
            r_ecoimpact: bi.r_ecoimpact(),
        }
    }

    pub fn any_hard_violation(&self) -> bool {
        self.r_energy.value() >= 1.0 ||
        self.r_hydraulic.value() >= 1.0 ||
        self.r_biology.value() >= 1.0 ||
        self.r_carbon.value() >= 1.0 ||
        self.r_materials.value() >= 1.0 ||
        self.r_neurorights.value() >= 1.0 ||
        self.r_soul.value() >= 1.0 ||
        self.r_social.value() >= 1.0 ||
        self.r_ecoimpact.value() >= 1.0
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct BiResidual {
    pub vt: Scalar,
}

impl BiResidual {
    pub fn compute(rv: &BiRiskVector, w: &BiLyapunovWeights) -> Self {
        let vt =
            w.w_energy * rv.r_energy.value().powi(2) +
            w.w_hydraulic * rv.r_hydraulic.value().powi(2) +
            w.w_biology * rv.r_biology.value().powi(2) +
            w.w_carbon * rv.r_carbon.value().powi(2) +
            w.w_materials * rv.r_materials.value().powi(2) +
            w.w_neurorights * rv.r_neurorights.value().powi(2) +
            w.w_soul * rv.r_soul.value().powi(2) +
            w.w_social * rv.r_social.value().powi(2) +
            w.w_ecoimpact * rv.r_ecoimpact.value().powi(2);
        BiResidual { vt }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BiSafeStepDecision {
    Accept,
    Derate,
    Stop,
    StopKarmaViolation,
}

pub struct BiSafeStepConfig {
    pub epsilon: Scalar,
    pub enforce_karma_nonslash: bool,
}

pub fn bi_safestep(
    prev_residual: BiResidual,
    next_residual: BiResidual,
    rv_next: &BiRiskVector,
    prev_karma: Scalar,
    proposed_karma: Scalar,
    cfg: &BiSafeStepConfig,
) -> BiSafeStepDecision {
    if cfg.enforce_karma_nonslash && proposed_karma < prev_karma {
        return BiSafeStepDecision::StopKarmaViolation;
    }

    if rv_next.any_hard_violation() {
        return BiSafeStepDecision::Stop;
    }

    if next_residual.vt <= prev_residual.vt + cfg.epsilon {
        BiSafeStepDecision::Accept
    } else {
        BiSafeStepDecision::Derate
    }
}

pub trait BiSafeController {
    type State;
    type Actuation;

    fn propose_step_with_bi(
        &mut self,
        state: &Self::State,
        bi_shard: &BrainIdentityShard,
        prev_residual: BiResidual,
        weights: &BiLyapunovWeights,
    ) -> (Self::Actuation, BiRiskVector, BiResidual, Scalar);
}

#[derive(Copy, Clone, Debug)]
pub struct BiKerWindow {
    pub steps: u32,
    pub safe_steps: u32,
    pub max_r: Scalar,
    pub karma_preserved: bool,
}

impl Default for BiKerWindow {
    fn default() -> Self {
        BiKerWindow {
            steps: 0,
            safe_steps: 0,
            max_r: 0.0,
            karma_preserved: true,
        }
    }
}

impl BiKerWindow {
    pub fn update(&mut self, rv: &BiRiskVector, decision: BiSafeStepDecision, karma_ok: bool) {
        self.steps += 1;
        if matches!(decision, BiSafeStepDecision::Accept) {
            self.safe_steps += 1;
        }
        self.karma_preserved &= karma_ok;

        let mut max_r = self.max_r;
        max_r = max_r.max(rv.r_energy.value());
        max_r = max_r.max(rv.r_hydraulic.value());
        max_r = max_r.max(rv.r_biology.value());
        max_r = max_r.max(rv.r_carbon.value());
        max_r = max_r.max(rv.r_materials.value());
        max_r = max_r.max(rv.r_neurorights.value());
        max_r = max_r.max(rv.r_soul.value());
        max_r = max_r.max(rv.r_social.value());
        max_r = max_r.max(rv.r_ecoimpact.value());
        self.max_r = max_r;
    }

    pub fn k(&self) -> Scalar {
        if self.steps == 0 { 1.0 } else { self.safe_steps as Scalar / self.steps as Scalar }
    }

    pub fn r(&self) -> Scalar {
        self.max_r
    }

    pub fn e(&self) -> Scalar {
        1.0 - self.max_r
    }

    pub fn bi_ker_deployable(&self) -> bool {
        self.k() >= 0.90 && self.e() >= 0.90 && self.r() <= 0.13 && self.karma_preserved
    }
}
