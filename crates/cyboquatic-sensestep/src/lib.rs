#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Normalized risk coordinates coming from the ecosafety grammar.
/// Each component should already live in [0,1] and be RoH-bounded upstream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskCoords {
    pub r_env: f32,
    pub r_hydro: f32,
    pub r_contaminants: f32,
    pub r_ops: f32,
}

/// Lyapunov residual V_t for the ecosafety kernel (same scalar used by `safestep`).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LyapunovResidual {
    pub v: f32,
}

/// Multonry trust scalar D_t in [0,1], global scalar for sensing apparatus health.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TrustScalar {
    pub d: f32,
}

impl TrustScalar {
    pub fn clamped(self) -> Self {
        Self { d: self.d.clamp(0.0, 1.0) }
    }
}

/// Tier‑1 sensor channel families for MAR / wetland corridors.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SensorFamily {
    PfasProxy,
    Pharmaceuticals,
    Nutrients,
    Hlr,
    Temperature,
    Fouling,
}

/// Logical identity of a sensor channel on the vault / wetland grid.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorChannelId {
    pub family: SensorFamily,
    pub logical_name: String, // e.g. "MAR01_HLR_A", "SAT_PFAS_03"
}

/// One raw reading + metadata for a sensor channel at time t.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorSample {
    pub channel: SensorChannelId,
    pub timestamp: DateTime<Utc>,
    pub value: f32,
    /// Optional reference value from LC‑MS / precision instrument if available this tick.
    pub reference_value: Option<f32>,
}

/// Bundle of all sensor readings in a tick.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorSnapshot {
    pub samples: Vec<SensorSample>,
}

/// Empirical calibration stats for a channel (populated from qpudatashards).
/// These are the “grounded priors” from bench/field calibration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationProfile {
    pub channel: SensorChannelId,
    /// Systematic offset estimate (sensor − reference).
    pub bias: f32,
    /// Noise variance around bias.
    pub noise_var: f32,
    /// Drift rate per day (or per 1e3 ticks) in absolute units.
    pub drift_rate: f32,
    /// Hard corridor limits for physically plausible readings.
    pub min_corridor: f32,
    pub max_corridor: f32,
}

/// Aggregated disagreement between redundant sensors of same family.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisagreementStats {
    pub channel: SensorChannelId,
    pub peer_count: u8,
    pub mean_abs_delta: f32,
}

/// Trust adjustment decision per channel.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelDecision {
    pub channel: SensorChannelId,
    pub degrade_trust: bool,
    pub ignore_channel: bool,
    pub require_recalibration: bool,
    pub sensor_fault: bool,
    /// Local trust score for this channel in [0,1].
    pub channel_trust: f32,
    /// Short, machine‑parsable reason tags.
    pub reasons: Vec<String>, // e.g. ["out_of_corridor", "step_jump", "ref_mismatch"]
}

/// Whole‑step decision log returned to controllers and archived as qpudatashard payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SenseDecision {
    pub global_trust_before: TrustScalar,
    pub global_trust_after: TrustScalar,
    pub channel_decisions: Vec<ChannelDecision>,
}

/// Output from `sensestep` into the `safestep` ecosafety kernel.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SenseStepOutput {
    pub rx_new: RiskCoords,
    pub lyapunov_new: LyapunovResidual,
    pub trust_new: TrustScalar,
    pub decision_log: SenseDecision,
}

/// Hard policy constants (can be later moved into ALN particles / site profiles).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensePolicy {
    /// Minimum acceptable global D_t to forward coords into `safestep`.
    pub d_min: f32,            // e.g. 0.90
    /// Thresholds for anomaly detection.
    pub step_jump_sigma: f32,  // e.g. 5.0
    pub ref_mismatch_sigma: f32,
    pub disagreement_sigma: f32,
}

/// All inputs `sensestep` needs for a tick.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SenseStepInput {
    pub prev_snapshot: SensorSnapshot,
    pub curr_snapshot: SensorSnapshot,
    pub prev_risk: RiskCoords,
    pub prev_lyapunov: LyapunovResidual,
    pub prev_trust: TrustScalar,
    /// Per‑channel calibration profiles, from qpudatashard corpus.
    pub calibration_profiles: Vec<CalibrationProfile>,
    /// Cross‑sensor disagreement stats for this tick (computed upstream or in helper).
    pub disagreement: Vec<DisagreementStats>,
    /// Site‑specific policy thresholds.
    pub policy: SensePolicy,
}

/// Main ecosafety pre‑gate.
pub fn sensestep(input: SenseStepInput) -> SenseStepOutput {
    let mut channel_decisions = Vec::new();
    let mut trust_delta = 0.0_f32;

    for curr in &input.curr_snapshot.samples {
        let prev_opt = input.prev_snapshot.samples.iter()
            .find(|s| s.channel.logical_name == curr.channel.logical_name);

        let calib_opt = input.calibration_profiles.iter()
            .find(|c| c.channel.logical_name == curr.channel.logical_name);

        let disagree_opt = input.disagreement.iter()
            .find(|d| d.channel.logical_name == curr.channel.logical_name);

        let (decision, dt_local) = evaluate_channel(
            curr,
            prev_opt,
            calib_opt,
            disagree_opt,
            &input.policy,
        );

        trust_delta += dt_local;
        channel_decisions.push(decision);
    }

    // Aggregate global trust delta (clamp small noise, normalize).
    let trust_after = TrustScalar {
        d: (input.prev_trust.d + trust_delta).clamp(0.0, 1.0),
    }
    .clamped();

    // Compute new risk coordinates and Lyapunov residual on *filtered* data only.
    let filtered_snapshot = filter_snapshot(&input.curr_snapshot, &channel_decisions);
    let rx_new = update_risk_coords(&input.prev_risk, &filtered_snapshot);
    let lyapunov_new = update_lyapunov(&input.prev_lyapunov, &rx_new);

    let decision_log = SenseDecision {
        global_trust_before: input.prev_trust,
        global_trust_after: trust_after,
        channel_decisions,
    };

    SenseStepOutput {
        rx_new,
        lyapunov_new,
        trust_new: trust_after,
        decision_log,
    }
}

/// Evaluate a single channel against calibration, history, reference, and peers.
fn evaluate_channel(
    curr: &SensorSample,
    prev_opt: Option<&SensorSample>,
    calib_opt: Option<&CalibrationProfile>,
    disagree_opt: Option<&DisagreementStats>,
    policy: &SensePolicy,
) -> (ChannelDecision, f32) {
    let mut decision = ChannelDecision {
        channel: curr.channel.clone(),
        degrade_trust: false,
        ignore_channel: false,
        require_recalibration: false,
        sensor_fault: false,
        channel_trust: 1.0,
        reasons: Vec::new(),
    };

    let mut dt = 0.0_f32; // delta contribution to global D_t

    // 1. Corridor check (hard physical plausibility).
    if let Some(calib) = calib_opt {
        if curr.value < calib.min_corridor || curr.value > calib.max_corridor {
            decision.degrade_trust = true;
            decision.ignore_channel = true;
            decision.require_recalibration = true;
            decision.sensor_fault = true;
            decision.channel_trust = 0.0;
            decision.reasons.push("out_of_corridor".into());
            dt -= 0.05;
            return (decision, dt);
        }
    }

    // 2. Step jump check vs previous reading.
    if let (Some(prev), Some(calib)) = (prev_opt, calib_opt) {
        let delta = curr.value - prev.value;
        let sigma = calib.noise_var.sqrt().max(1e-6);
        if (delta / sigma).abs() > policy.step_jump_sigma {
            decision.degrade_trust = true;
            decision.ignore_channel = false; // soft-fault, still usable but down-weighted.
            decision.reasons.push("step_jump".into());
            decision.channel_trust *= 0.7;
            dt -= 0.02;
        }
    }

    // 3. Reference instrument mismatch (LC-MS, high-grade flowmeter, etc.).
    if let (Some(ref_val), Some(calib)) = (curr.reference_value, calib_opt) {
        let err = (curr.value - ref_val) - calib.bias;
        let sigma = calib.noise_var.sqrt().max(1e-6);
        if (err / sigma).abs() > policy.ref_mismatch_sigma {
            decision.degrade_trust = true;
            decision.require_recalibration = true;
            decision.reasons.push("ref_mismatch".into());
            decision.channel_trust *= 0.5;
            dt -= 0.03;
        }
    }

    // 4. Cross-sensor disagreement within family / zone.
    if let Some(dis) = disagree_opt {
        // Treat dis.mean_abs_delta vs expected noise as another sigma-test.
        let sigma_eff = 1.0_f32; // TODO: derive from family-level calibration corpus.
        if (dis.mean_abs_delta / sigma_eff) > policy.disagreement_sigma {
            decision.degrade_trust = true;
            decision.reasons.push("peer_disagreement".into());
            decision.channel_trust *= 0.6;
            dt -= 0.02;
        }
    }

    // 5. Drift indicator: handled upstream by updating CalibrationProfile; here we only
    //    mark recalibration if long-term trust collapses.
    if let Some(calib) = calib_opt {
        if calib.drift_rate.abs() > 0.0 && decision.degrade_trust {
            decision.require_recalibration = true;
            decision.reasons.push("drift_suspected".into());
        }
    }

    // 6. Reward stable, corridor-respecting operation over time.
    if !decision.degrade_trust {
        decision.reasons.push("stable".into());
        dt += 0.001; // very slow recovery toward baseline.
    }

    (decision, dt)
}

/// Drop/retain samples based on `ignore_channel` flag so `safestep` never sees quarantined data.
fn filter_snapshot(snapshot: &SensorSnapshot, decisions: &[ChannelDecision]) -> SensorSnapshot {
    let mut out = Vec::new();
    for sample in &snapshot.samples {
        if let Some(dec) = decisions
            .iter()
            .find(|d| d.channel.logical_name == sample.channel.logical_name)
        {
            if dec.ignore_channel || dec.sensor_fault {
                continue;
            }
        }
        out.push(sample.clone());
    }
    SensorSnapshot { samples: out }
}

// These are stubs; in your ecosafety stack they should call the same kernels as `safestep`.

fn update_risk_coords(prev: &RiskCoords, snapshot: &SensorSnapshot) -> RiskCoords {
    // Example: adjust r_contaminants, r_hydro based on filtered Tier‑1 channels.
    // Implementation is domain-specific and should be shared with `safestep`.
    let mut r = prev.clone();
    // ... fill in with your existing MAR / wetland risk algebra ...
    r
}

fn update_lyapunov(prev: &LyapunovResidual, rx_new: &RiskCoords) -> LyapunovResidual {
    // Example: V_t = max(0, V_{t-1} + Φ(r_x_new) - Φ(r_x_prev)).
    LyapunovResidual { v: prev.v } // placeholder: wire into ecosafety Lyapunov kernel.
}

/// Helper predicate for controllers: may we forward r_x^{new} into `safestep`?
pub fn can_forward_to_safestep(
    output: &SenseStepOutput,
    policy: &SensePolicy,
) -> bool {
    let fault_present = output
        .decision_log
        .channel_decisions
        .iter()
        .any(|c| c.sensor_fault);

    !fault_present && output.trust_new.d >= policy.d_min
}
