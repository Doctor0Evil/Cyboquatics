#![forbid(unsafe_code)]

use std::time::Instant;
use cyboquatic_ecosafety_core::{CorridorDecision, EcoSafetyKernel, KerWindow, RiskCoord, RiskVector};

#[derive(Clone, Copy, Debug)]
pub enum MediaClass {
    WaterOnly,
    WaterBiofilm,
    AirPlenum,
}

/// Typed workload descriptor (energy + hydraulic + expected ΔV_t). [file:21]
#[derive(Clone, Copy, Debug)]
pub struct CyboWorkload {
    pub id: u64,
    pub energy_req_j: f64,
    pub safety_factor: f64,
    pub max_latency_ms: u64,
    pub media: MediaClass,
    /// Fraction of remaining hydraulic corridor margin consumed (0–1).
    pub hydraulic_impact: f64,
    /// Expected incremental Lyapunov delta if routed to a neutral node.
    pub dv_t_nominal: f64,
}

/// Node shard view used by router (energy, hydraulics, biology, V_t, KER). [file:21]
#[derive(Clone, Copy, Debug)]
pub enum BioSurfaceMode {
    Raw,
    Preprocessed,
    Restricted,
}

#[derive(Clone, Copy, Debug)]
pub struct NodeShard {
    // Energy plane
    pub e_surplus_j: f64,
    pub p_margin_kw: f64,
    pub tailwind_mode: bool,
    pub dE_dt_w: f64,
    // Hydraulics plane
    pub q_m3s: f64,
    pub hlr_m_per_h: f64,
    pub surcharge_risk_rx: f64, // 0–1
    // Biology plane
    pub r_pathogen: f64,
    pub r_fouling: f64,
    pub r_cec: f64,
    pub biosurface_mode: BioSurfaceMode,
    // Lyapunov / KER
    pub vt_local: f64,
    pub vt_trend: f64,
    pub k_score: f64,
    pub e_score: f64,
    pub r_score: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteDecision {
    Accept,
    Reject,
    Reroute,
}

#[derive(Clone, Copy, Debug)]
pub struct RoutingContext {
    pub vt_global: f64,
    pub vt_global_next_max: f64,
    pub now: Instant,
}

// Energy predicate: only route if true surplus remains. [file:21]
fn tailwind_valid(node: &NodeShard, wl: &CyboWorkload) -> bool {
    if !node.tailwind_mode {
        return false;
    }
    let required = wl.energy_req_j * wl.safety_factor.max(1.0);
    node.e_surplus_j > required && node.p_margin_kw > 0.0 && node.dE_dt_w <= 0.0
}

// Biological substrate predicate: only preprocessed, low-risk surfaces get biocontact. [file:21]
fn biosurface_ok(node: &NodeShard, wl: &CyboWorkload) -> bool {
    match node.biosurface_mode {
        BioSurfaceMode::Restricted => matches!(wl.media, MediaClass::AirPlenum),
        BioSurfaceMode::Raw | BioSurfaceMode::Preprocessed => {
            let r_thresh = 0.5; // stricter than legal limit band
            match wl.media {
                MediaClass::AirPlenum => node.r_pathogen <= r_thresh,
                MediaClass::WaterOnly | MediaClass::WaterBiofilm => {
                    matches!(node.biosurface_mode, BioSurfaceMode::Preprocessed)
                        && node.r_pathogen <= r_thresh
                        && node.r_fouling <= r_thresh
                        && node.r_cec <= r_thresh
                }
            }
        }
    }
}

// Hydraulics predicate: do not exceed surcharge corridor. [file:21]
fn hydraulic_ok(node: &NodeShard, wl: &CyboWorkload) -> bool {
    let impact = wl.hydraulic_impact.max(0.0);
    let rx = node.surcharge_risk_rx.max(0.0);
    let predicted = rx + impact;
    predicted < 1.0
}

// Lyapunov predicate: keep V_t non-increasing within configured bound. [file:3][file:21]
fn lyapunov_ok(node: &NodeShard, wl: &CyboWorkload, ctx: &RoutingContext) -> bool {
    let dv_local = wl.dv_t_nominal;
    let vt_next_est = ctx.vt_global + dv_local;
    vt_next_est <= ctx.vt_global_next_max && dv_local + node.vt_trend <= 0.0
}

/// Composite routing rule used by FOG router. [file:21]
pub fn route_workload(
    wl: &CyboWorkload,
    node: &NodeShard,
    ctx: &RoutingContext,
) -> RouteDecision {
    if !tailwind_valid(node, wl) {
        return RouteDecision::Reroute;
    }
    if !biosurface_ok(node, wl) {
        return RouteDecision::Reroute;
    }
    if !hydraulic_ok(node, wl) {
        return RouteDecision::Reroute;
    }
    if !lyapunov_ok(node, wl, ctx) {
        return RouteDecision::Reject;
    }
    RouteDecision::Accept
}

/// Example: integrate routing decision with ecosafety kernel for a node. [file:3][file:21]
pub fn routed_step_with_ecosafety<S, A, C>(
    controller: &mut C,
    state: &mut S,
    kernel: &mut EcoSafetyKernel,
    ker_window: &mut KerWindow,
    dt: std::time::Duration,
    apply: impl Fn(&mut S, &A),
    node_shard: &NodeShard,
    wl: &CyboWorkload,
    ctx: &RoutingContext,
) -> (RouteDecision, CorridorDecision)
where
    C: cyboquatic_ecosafety_core::SafeController<S, A>,
{
    let route = route_workload(wl, node_shard, ctx);
    if route != RouteDecision::Accept {
        return (route, CorridorDecision::Stop);
    }

    let prev_vt = kernel.residual.vt;
    let (act, risks) = controller.propose_step(state, dt);
    let decision = kernel.safestep(prev_vt, &risks);
    let lyap_safe = matches!(decision, CorridorDecision::Ok);
    ker_window.record_step(lyap_safe, &risks);

    if decision == CorridorDecision::Ok {
        apply(state, &act);
    }

    (route, decision)
}
