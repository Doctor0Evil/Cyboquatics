// Filename: opt-cyboquatic-kernel/src/node_kernel.rs

use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub enum MediaClass {
    WaterOnly,
    WaterBiofilm,
    AirPlenum,
}

#[derive(Clone, Copy, Debug)]
pub struct CyboWorkload {
    pub id: u64,
    /// Joules required over the execution horizon
    pub energy_req_j: f64,
    /// Safety factor multiplier ≥ 1
    pub safety_factor: f64,
    /// Max acceptable latency in ms
    pub max_latency_ms: u64,
    /// Operating media
    pub media: MediaClass,
    /// Normalized hydraulic corridor usage 0–1
    pub hydraulic_impact: f64,
    /// Expected Lyapunov delta if applied at a neutral node
    pub dvt_nominal: f64,
}

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
    pub d_edt_w: f64,

    // Hydraulics plane
    pub q_m3s: f64,
    pub hlr_m_per_h: f64,
    /// 0–1 normalized surcharge risk
    pub surcharge_risk_rx: f64,

    // Biology / substrate plane
    pub r_pathogen: f64,
    pub r_fouling: f64,
    pub r_cec: f64,
    pub bio_surface_mode: BioSurfaceMode,

    // Global residual & KER
    pub vt_local: f64,
    pub vt_trend: f64,
    pub k_score: f64,
    pub e_score: f64,
    pub r_score: f64,
}

#[derive(Clone, Copy, Debug)]
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

// ---------- ENERGY PREDICATE: tailwindvalid ----------

fn tailwind_valid(node: &NodeShard, wl: &CyboWorkload) -> bool {
    if !node.tailwind_mode {
        return false;
    }

    let safety = wl.safety_factor.max(1.0);
    let required = wl.energy_req_j * safety;

    // Require strictly positive surplus and non‑collapsing power margin
    node.e_surplus_j > required &&
        node.p_margin_kw > 0.0 &&
        node.d_edt_w >= 0.0
}

// ---------- BIOLOGICAL PREDICATE: biosurface_ok ----------

fn biosurface_ok(node: &NodeShard, wl: &CyboWorkload) -> bool {
    // Never allow bio‑contact on restricted surfaces
    if let BioSurfaceMode::Restricted = node.bio_surface_mode {
        return matches!(wl.media, MediaClass::AirPlenum);
    }

    // Gold‑corridor thresholds (tighter than legal)
    let r_thresh = 0.5_f64;

    match wl.media {
        MediaClass::AirPlenum => {
            // Air‑only ops allowed when pathogen risk is low
            node.r_pathogen <= r_thresh
        }
        MediaClass::WaterOnly | MediaClass::WaterBiofilm => {
            // Require preprocessed surface & all risks within corridor
            matches!(node.bio_surface_mode, BioSurfaceMode::Preprocessed)
                && node.r_pathogen <= r_thresh
                && node.r_fouling <= r_thresh
                && node.r_cec <= r_thresh
        }
    }
}

// ---------- HYDRAULIC PREDICATE: hydraulic_ok ----------

fn hydraulic_ok(node: &NodeShard, wl: &CyboWorkload) -> bool {
    let impact = wl.hydraulic_impact.max(0.0);
    let rx = node.surcharge_risk_rx.max(0.0);

    // Forbid crossing normalized corridor closure (1.0)
    let predicted_rx = rx + impact;
    predicted_rx < 1.0
}

// ---------- LYAPUNOV PREDICATE: vt_non_increase ----------

fn lyapunov_ok(node: &NodeShard, wl: &CyboWorkload, ctx: &RoutingContext) -> bool {
    let dv_local = wl.dvt_nominal;
    let vt_next_est = ctx.vt_global + dv_local;

    // Require non‑increase globally and compatibility with local trend
    vt_next_est <= ctx.vt_global_next_max && (dv_local + node.vt_trend) <= 0.0
}

// ---------- COMPOSITE ROUTING RULE ----------

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

// ---------- EXAMPLE MAIN (TEST HARNESS) ----------

fn main() {
    let ctx = RoutingContext {
        vt_global: 1.0,
        vt_global_next_max: 1.0,
        now: Instant::now(),
    };

    let node = NodeShard {
        e_surplus_j: 5_000.0,
        p_margin_kw: 3.5,
        tailwind_mode: true,
        d_edt_w: 10.0,

        q_m3s: 0.2,
        hlr_m_per_h: 5.0,
        surcharge_risk_rx: 0.2,

        r_pathogen: 0.1,
        r_fouling: 0.3,
        r_cec: 0.2,
        bio_surface_mode: BioSurfaceMode::Preprocessed,

        vt_local: 0.9,
        vt_trend: -0.01,
        k_score: 0.93,
        e_score: 0.90,
        r_score: 0.14,
    };

    let wl = CyboWorkload {
        id: 42,
        energy_req_j: 500.0,
        safety_factor: 1.5,
        max_latency_ms: 200,
        media: MediaClass::WaterOnly,
        hydraulic_impact: 0.1,
        dvt_nominal: -0.001,
    };

    let decision = route_workload(&wl, &node, &ctx);
    println!("Routing decision for {} -> {:?}", wl.id, decision);
}
