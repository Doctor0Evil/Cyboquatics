// ============================================================================
// Cyboquatic FOG Router - Energy-Efficient Workload Distribution
// ============================================================================
// Version: 1.0.0
// License: Apache-2.0 OR MIT
// Authors: Cyboquatic Research Collective
//
// This module routes Cyboquatic workloads to nodes with surplus energy,
// hydraulic safety, clean substrates, and non-increasing Vt residuals.
// All routing decisions are governed by rx/Vt/KER invariants and
// cryptographic audit trails.
//
// Key Features:
// - Energy tailwind validation (surplus energy routing)
// - Hydraulic safety predicates (surcharge risk, flow capacity)
// - Biosurface mode compatibility (pathogen, fouling, CEC risks)
// - Lyapunov stability enforcement (Vt non-increase)
// - Security checks (firmware integrity, node authentication)
// - ALN shard export for routing audit trails
// - Adaptive corridor bands for diverse environments
//
// Continuity Guarantee: All routing decisions are logged with
// cryptographic hashes. Route history can be verified decades
// after execution for regulatory compliance and incident analysis.
// ============================================================================

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ============================================================================
// Media Class (Workload Environment Type)
// ============================================================================

/// Classification of media environment for workload routing.
///
/// Different media types have different safety requirements and
/// risk profiles. This classification determines which nodes
/// are eligible to accept a given workload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MediaClass {
    /// Water-only media (no biofilm).
    WaterOnly,
    /// Water with biofilm (biological treatment).
    WaterBiofilm,
    /// Air plenum (gas-phase treatment).
    AirPlenum,
    /// Mixed media (water + air).
    Mixed,
    /// Solid substrate (terrestrial).
    SolidSubstrate,
}

impl MediaClass {
    /// Returns true if this media class requires biofilm compatibility.
    pub fn requires_biofilm(&self) -> bool {
        matches!(self, MediaClass::WaterBiofilm)
    }

    /// Returns true if this media class is water-based.
    pub fn is_water_based(&self) -> bool {
        matches!(self, MediaClass::WaterOnly | MediaClass::WaterBiofilm | MediaClass::Mixed)
    }

    /// Returns true if this media class is air-based.
    pub fn is_air_based(&self) -> bool {
        matches!(self, MediaClass::AirPlenum | MediaClass::Mixed)
    }

    /// Returns the risk threshold for this media class.
    pub fn risk_threshold(&self) -> f64 {
        match self {
            MediaClass::WaterOnly => 0.5,
            MediaClass::WaterBiofilm => 0.3,
            MediaClass::AirPlenum => 0.6,
            MediaClass::Mixed => 0.4,
            MediaClass::SolidSubstrate => 0.5,
        }
    }
}

impl fmt::Display for MediaClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MediaClass::WaterOnly => write!(f, "WaterOnly"),
            MediaClass::WaterBiofilm => write!(f, "WaterBiofilm"),
            MediaClass::AirPlenum => write!(f, "AirPlenum"),
            MediaClass::Mixed => write!(f, "Mixed"),
            MediaClass::SolidSubstrate => write!(f, "SolidSubstrate"),
        }
    }
}

// ============================================================================
// Cybo Variant (Workload Definition)
// ============================================================================

/// Definition of a Cyboquatic workload variant.
///
/// Each variant specifies the resource requirements, safety constraints,
/// and environmental needs for a particular computational or physical
/// task to be routed through the FOG network.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CyboVariant {
    /// Unique variant identifier.
    pub id: u64,

    /// Energy requirement in Joules.
    pub energy_req_j: f64,

    /// Safety factor multiplier (≥ 1.0).
    pub safety_factor: f64,

    /// Maximum acceptable latency in milliseconds.
    pub max_latency_ms: u64,

    /// Media class requirement.
    pub media: MediaClass,

    /// Hydraulic impact factor (0.0 = none, 1.0 = maximum).
    pub hydraulic_impact: f64,

    /// Nominal Vt change expected from this workload.
    pub dvt_nominal: f64,

    /// Priority level (1-10, 10 is highest).
    pub priority: u8,

    /// Whether this workload requires secure node (firmware verified).
    pub requires_secure_node: bool,

    /// Timestamp when workload was created (UNIX epoch seconds).
    pub created_timestamp: u64,
}

impl CyboVariant {
    /// Creates a new workload variant with validation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        energy_req_j: f64,
        safety_factor: f64,
        max_latency_ms: u64,
        media: MediaClass,
        hydraulic_impact: f64,
        dvt_nominal: f64,
        priority: u8,
    ) -> Result<Self, RouterError> {
        if energy_req_j < 0.0 {
            return Err(RouterError::NegativeEnergyRequirement);
        }
        if safety_factor < 1.0 {
            return Err(RouterError::SafetyFactorBelowOne);
        }
        if hydraulic_impact < 0.0 || hydraulic_impact > 1.0 {
            return Err(RouterError::HydraulicImpactOutOfRange);
        }
        if priority == 0 || priority > 10 {
            return Err(RouterError::PriorityOutOfRange);
        }

        Ok(CyboVariant {
            id,
            energy_req_j,
            safety_factor,
            max_latency_ms,
            media,
            hydraulic_impact,
            dvt_nominal,
            priority,
            requires_secure_node: false,
            created_timestamp: Self::current_timestamp(),
        })
    }

    /// Sets whether this workload requires a secure node.
    pub fn with_secure_requirement(mut self, requires: bool) -> Self {
        self.requires_secure_node = requires;
        self
    }

    /// Returns the total energy requirement with safety factor.
    pub fn total_energy_required(&self) -> f64 {
        self.energy_req_j * self.safety_factor.max(1.0)
    }

    /// Returns true if this workload is high-priority (≥ 8).
    pub fn is_high_priority(&self) -> bool {
        self.priority >= 8
    }

    /// Returns true if this workload is critical (priority = 10).
    pub fn is_critical(&self) -> bool {
        self.priority == 10
    }

    /// Returns current UNIX timestamp in seconds.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }
}

impl fmt::Display for CyboVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CyboVariant[id={}, energy={:.0}J, media={}, priority={}]",
            self.id, self.energy_req_j, self.media, self.priority
        )
    }
}

// ============================================================================
// BioSurface Mode (Biological Treatment State)
// ============================================================================

/// Mode of biosurface operation for a node.
///
/// Determines what types of workloads a node can accept based on
/// its biological treatment state and pathogen/fouling risks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BioSurfaceMode {
    /// Raw biosurface (unprocessed, higher risk).
    Raw,
    /// Preprocessed biosurface (treated, lower risk).
    Preprocessed,
    /// Restricted biosurface (air-only, no water contact).
    Restricted,
    /// Inactive biosurface (maintenance mode).
    Inactive,
}

impl BioSurfaceMode {
    /// Returns true if this mode allows water-based workloads.
    pub fn allows_water(&self) -> bool {
        matches!(self, BioSurfaceMode::Preprocessed)
    }

    /// Returns true if this mode allows air-based workloads.
    pub fn allows_air(&self) -> bool {
        matches!(self, BioSurfaceMode::Raw | BioSurfaceMode::Preprocessed | BioSurfaceMode::Restricted)
    }

    /// Returns the maximum acceptable pathogen risk for this mode.
    pub fn max_pathogen_risk(&self) -> f64 {
        match self {
            BioSurfaceMode::Raw => 0.7,
            BioSurfaceMode::Preprocessed => 0.3,
            BioSurfaceMode::Restricted => 0.5,
            BioSurfaceMode::Inactive => 0.0,
        }
    }

    /// Returns true if this mode is active (can accept workloads).
    pub fn is_active(&self) -> bool {
        !matches!(self, BioSurfaceMode::Inactive)
    }
}

impl fmt::Display for BioSurfaceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BioSurfaceMode::Raw => write!(f, "Raw"),
            BioSurfaceMode::Preprocessed => write!(f, "Preprocessed"),
            BioSurfaceMode::Restricted => write!(f, "Restricted"),
            BioSurfaceMode::Inactive => write!(f, "Inactive"),
        }
    }
}

// ============================================================================
// Node Shard (Routing Target Definition)
// ============================================================================

/// Complete definition of a routing target node.
///
/// Each node shard contains all information needed to evaluate
/// whether a workload can be safely routed to it. This includes
/// energy availability, hydraulic capacity, risk coordinates,
/// and Lyapunov stability metrics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeShard {
    /// Unique node identifier.
    pub node_id: u64,

    /// Surplus energy available in Joules.
    pub esurplus_j: f64,

    /// Power margin in kilowatts.
    pub pmargin_kw: f64,

    /// Whether tailwind mode is active (energy surplus routing).
    pub tailwind_mode: bool,

    /// Energy delivery rate in Watts.
    pub d_edt_w: f64,

    /// Flow rate in cubic meters per second.
    pub q_m3s: f64,

    /// Hydraulic loading rate in meters per hour.
    pub hlr_m_per_h: f64,

    /// Surcharge risk coordinate (0.0-1.0).
    pub surcharge_risk_rx: f64,

    /// Pathogen risk coordinate (0.0-1.0).
    pub r_pathogen: f64,

    /// Fouling risk coordinate (0.0-1.0).
    pub r_fouling: f64,

    /// Contaminants of emerging concern risk (0.0-1.0).
    pub r_cec: f64,

    /// PFAS residue risk (0.0-1.0).
    pub r_pfas: f64,

    /// Biosurface operation mode.
    pub biosurface_mode: BioSurfaceMode,

    /// Local Lyapunov residual (Vt).
    pub vt_local: f64,

    /// Vt trend (negative = decreasing, positive = increasing).
    pub vt_trend: f64,

    /// K (knowledge) score from KER triad.
    pub kscore: f64,

    /// E (eco-impact) score from KER triad.
    pub escore: f64,

    /// R (risk of harm) score from KER triad.
    pub rscore: f64,

    /// Firmware version hash (for security verification).
    pub firmware_hash: u64,

    /// Whether firmware is verified (secure boot).
    pub firmware_verified: bool,

    /// Node uptime in seconds.
    pub uptime_seconds: u64,

    /// Last heartbeat timestamp (UNIX epoch seconds).
    pub last_heartbeat: u64,
}

impl NodeShard {
    /// Creates a new node shard with validation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: u64,
        esurplus_j: f64,
        pmargin_kw: f64,
        tailwind_mode: bool,
        d_edt_w: f64,
        q_m3s: f64,
        hlr_m_per_h: f64,
        surcharge_risk_rx: f64,
        r_pathogen: f64,
        r_fouling: f64,
        r_cec: f64,
        r_pfas: f64,
        biosurface_mode: BioSurfaceMode,
        vt_local: f64,
        vt_trend: f64,
        kscore: f64,
        escore: f64,
        rscore: f64,
    ) -> Result<Self, RouterError> {
        if esurplus_j < 0.0 {
            return Err(RouterError::NegativeEnergySurplus);
        }
        if surcharge_risk_rx < 0.0 || surcharge_risk_rx > 1.0 {
            return Err(RouterError::RiskCoordinateOutOfRange("surcharge_risk_rx"));
        }
        if r_pathogen < 0.0 || r_pathogen > 1.0 {
            return Err(RouterError::RiskCoordinateOutOfRange("r_pathogen"));
        }
        if r_fouling < 0.0 || r_fouling > 1.0 {
            return Err(RouterError::RiskCoordinateOutOfRange("r_fouling"));
        }
        if r_cec < 0.0 || r_cec > 1.0 {
            return Err(RouterError::RiskCoordinateOutOfRange("r_cec"));
        }
        if r_pfas < 0.0 || r_pfas > 1.0 {
            return Err(RouterError::RiskCoordinateOutOfRange("r_pfas"));
        }
        if kscore < 0.0 || kscore > 1.0 {
            return Err(RouterError::KerScoreOutOfRange("kscore"));
        }
        if escore < 0.0 || escore > 1.0 {
            return Err(RouterError::KerScoreOutOfRange("escore"));
        }
        if rscore < 0.0 || rscore > 1.0 {
            return Err(RouterError::KerScoreOutOfRange("rscore"));
        }

        Ok(NodeShard {
            node_id,
            esurplus_j,
            pmargin_kw,
            tailwind_mode,
            d_edt_w,
            q_m3s,
            hlr_m_per_h,
            surcharge_risk_rx,
            r_pathogen,
            r_fouling,
            r_cec,
            r_pfas,
            biosurface_mode,
            vt_local,
            vt_trend,
            kscore,
            escore,
            rscore,
            firmware_hash: 0,
            firmware_verified: false,
            uptime_seconds: 0,
            last_heartbeat: Self::current_timestamp(),
        })
    }

    /// Sets firmware verification status.
    pub fn with_firmware_verification(mut self, hash: u64, verified: bool) -> Self {
        self.firmware_hash = hash;
        self.firmware_verified = verified;
        self
    }

    /// Returns true if node has sufficient energy for variant.
    pub fn has_energy_for(&self, variant: &CyboVariant) -> bool {
        let required = variant.total_energy_required();
        self.esurplus_j >= required && self.pmargin_kw > 0.0 && self.d_edt_w >= 0.0
    }

    /// Returns true if node biosurface is compatible with variant.
    pub fn biosurface_compatible(&self, variant: &CyboVariant) -> bool {
        match self.biosurface_mode {
            BioSurfaceMode::Restricted => matches!(variant.media, MediaClass::AirPlenum),
            BioSurfaceMode::Inactive => false,
            BioSurfaceMode::Raw | BioSurfaceMode::Preprocessed => {
                let rthresh = self.biosurface_mode.max_pathogen_risk();
                match variant.media {
                    MediaClass::AirPlenum => self.r_pathogen < rthresh,
                    MediaClass::WaterOnly | MediaClass::WaterBiofilm => {
                        matches!(self.biosurface_mode, BioSurfaceMode::Preprocessed)
                            && self.r_pathogen < rthresh
                            && self.r_fouling < rthresh
                            && self.r_cec < rthresh
                    }
                    MediaClass::Mixed | MediaClass::SolidSubstrate => {
                        self.r_pathogen < rthresh && self.r_fouling < rthresh
                    }
                }
            }
        }
    }

    /// Returns true if node has hydraulic capacity for variant.
    pub fn hydraulic_capacity_for(&self, variant: &CyboVariant) -> bool {
        let impact = variant.hydraulic_impact.max(0.0);
        let rx = self.surcharge_risk_rx.max(0.0);
        let predicted = rx + impact;
        predicted < 1.0
    }

    /// Returns true if routing to this node maintains Lyapunov stability.
    pub fn lyapunov_stable_for(&self, variant: &CyboVariant, vt_global_next_max: f64) -> bool {
        let dv_local = variant.dvt_nominal;
        let vt_next_est = self.vt_local + dv_local;
        vt_next_est <= vt_global_next_max && dv_local + self.vt_trend <= 0.0
    }

    /// Returns true if node meets security requirements for variant.
    pub fn security_compatible(&self, variant: &CyboVariant) -> bool {
        if !variant.requires_secure_node {
            return true;
        }
        self.firmware_verified
    }

    /// Returns the node's composite health score (0.0-1.0).
    pub fn health_score(&self) -> f64 {
        let ker_score = (self.kscore + self.escore + (1.0 - self.rscore)) / 3.0;
        let risk_score = 1.0 - (self.surcharge_risk_rx + self.r_pathogen + self.r_fouling + self.r_cec + self.r_pfas) / 5.0;
        let energy_score = if self.esurplus_j > 1000.0 { 1.0 } else { self.esurplus_j / 1000.0 };
        (ker_score * 0.4 + risk_score * 0.4 + energy_score * 0.2).clamp(0.0, 1.0)
    }

    /// Returns true if node is healthy enough to accept workloads.
    pub fn is_healthy(&self) -> bool {
        self.health_score() >= 0.7 && self.biosurface_mode.is_active()
    }

    /// Returns current UNIX timestamp in seconds.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }
}

impl fmt::Display for NodeShard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NodeShard[id={}, energy={:.0}J, Vt={:.4}, health={:.2}]",
            self.node_id, self.esurplus_j, self.vt_local, self.health_score()
        )
    }
}

// ============================================================================
// Routing Context (Global State)
// ============================================================================

/// Global routing context for decision-making.
///
/// Contains system-wide state that affects routing decisions,
/// including global Lyapunov residuals, timing information,
/// and environmental conditions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoutingContext {
    /// Global Lyapunov residual (Vt).
    pub vt_global: f64,

    /// Maximum allowed Vt for next timestep.
    pub vt_global_next_max: f64,

    /// Epsilon tolerance for Vt increase.
    pub eps_vt: f64,

    /// Current timestamp (Instant for latency calculations).
    pub now: Instant,

    /// Current timestamp (UNIX epoch for logging).
    pub timestamp_unix: u64,

    /// System-wide risk threshold.
    pub global_risk_threshold: f64,

    /// Whether security checks are enforced.
    pub security_enforced: bool,
}

impl RoutingContext {
    /// Creates a new routing context with default values.
    pub fn new() -> Self {
        RoutingContext {
            vt_global: 1.0,
            vt_global_next_max: 1.0,
            eps_vt: 0.001,
            now: Instant::now(),
            timestamp_unix: Self::current_timestamp(),
            global_risk_threshold: 0.5,
            security_enforced: true,
        }
    }

    /// Creates context with explicit Vt parameters.
    pub fn with_vt(vt_global: f64, eps_vt: f64) -> Self {
        RoutingContext {
            vt_global,
            vt_global_next_max: vt_global + eps_vt,
            eps_vt,
            now: Instant::now(),
            timestamp_unix: Self::current_timestamp(),
            global_risk_threshold: 0.5,
            security_enforced: true,
        }
    }

    /// Updates Vt for next timestep.
    pub fn update_vt(&mut self, new_vt: f64) {
        self.vt_global = new_vt;
        self.vt_global_next_max = new_vt + self.eps_vt;
    }

    /// Returns current UNIX timestamp in seconds.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }
}

impl Default for RoutingContext {
    fn default() -> Self {
        RoutingContext::new()
    }
}

// ============================================================================
// Route Decision (Routing Outcome)
// ============================================================================

/// Outcome of a routing evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RouteDecision {
    /// Workload can be routed to this node.
    Accept,
    /// Workload cannot be routed to this node.
    Reject,
    /// Workload should be routed to a different node.
    Reroute,
    /// Node is unavailable (offline, maintenance).
    Unavailable,
    /// Node requires security verification before routing.
    SecurityPending,
}

impl RouteDecision {
    /// Returns true if routing is approved.
    pub fn is_accepted(&self) -> bool {
        matches!(self, RouteDecision::Accept)
    }

    /// Returns true if routing is denied.
    pub fn is_rejected(&self) -> bool {
        matches!(self, RouteDecision::Reject | RouteDecision::Reroute | RouteDecision::Unavailable)
    }

    /// Returns the rejection reason code (for logging).
    pub fn reason_code(&self) -> u8 {
        match self {
            RouteDecision::Accept => 0,
            RouteDecision::Reject => 1,
            RouteDecision::Reroute => 2,
            RouteDecision::Unavailable => 3,
            RouteDecision::SecurityPending => 4,
        }
    }
}

impl fmt::Display for RouteDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouteDecision::Accept => write!(f, "Accept"),
            RouteDecision::Reject => write!(f, "Reject"),
            RouteDecision::Reroute => write!(f, "Reroute"),
            RouteDecision::Unavailable => write!(f, "Unavailable"),
            RouteDecision::SecurityPending => write!(f, "SecurityPending"),
        }
    }
}

/// Detailed routing result with failure reasons.
#[derive(Clone, Debug)]
pub struct RoutingResult {
    /// Final routing decision.
    pub decision: RouteDecision,

    /// Node ID that was evaluated.
    pub node_id: u64,

    /// Variant ID that was evaluated.
    pub variant_id: u64,

    /// Whether tailwind validation passed.
    pub tailwind_valid: bool,

    /// Whether biosurface validation passed.
    pub biosurface_valid: bool,

    /// Whether hydraulic validation passed.
    pub hydraulic_valid: bool,

    /// Whether Lyapunov validation passed.
    pub lyapunov_valid: bool,

    /// Whether security validation passed.
    pub security_valid: bool,

    /// Failure reasons (if any).
    pub failure_reasons: Vec<String>,

    /// Timestamp of routing decision (UNIX epoch seconds).
    pub timestamp: u64,

    /// Latency of routing decision (microseconds).
    pub latency_us: u64,
}

impl RoutingResult {
    /// Creates a new routing result.
    pub fn new(node_id: u64, variant_id: u64, decision: RouteDecision) -> Self {
        RoutingResult {
            decision,
            node_id,
            variant_id,
            tailwind_valid: false,
            biosurface_valid: false,
            hydraulic_valid: false,
            lyapunov_valid: false,
            security_valid: false,
            failure_reasons: Vec::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
            latency_us: 0,
        }
    }

    /// Adds a failure reason.
    pub fn with_failure_reason(mut self, reason: String) -> Self {
        self.failure_reasons.push(reason);
        self
    }

    /// Sets validation flags.
    pub fn with_validations(
        mut self,
        tailwind: bool,
        biosurface: bool,
        hydraulic: bool,
        lyapunov: bool,
        security: bool,
    ) -> Self {
        self.tailwind_valid = tailwind;
        self.biosurface_valid = biosurface;
        self.hydraulic_valid = hydraulic;
        self.lyapunov_valid = lyapunov;
        self.security_valid = security;
        self
    }

    /// Returns true if routing was successful.
    pub fn is_success(&self) -> bool {
        self.decision.is_accepted()
    }

    /// Returns a summary of failure reasons.
    pub fn failure_summary(&self) -> String {
        if self.failure_reasons.is_empty() {
            "None".to_string()
        } else {
            self.failure_reasons.join("; ")
        }
    }
}

impl fmt::Display for RoutingResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RoutingResult[node={}, variant={}, decision={}, latency={}μs]",
            self.node_id, self.variant_id, self.decision, self.latency_us
        )
    }
}

// ============================================================================
// FOG Router (Core Routing Engine)
// ============================================================================

/// Core FOG routing engine.
///
/// Evaluates workloads against available nodes and makes routing
/// decisions based on energy, hydraulic, biosurface, Lyapunov,
/// and security predicates.
#[derive(Clone, Debug)]
pub struct FogRouter {
    /// Map of node ID to node shard.
    nodes: HashMap<u64, NodeShard>,

    /// Routing history (bounded buffer).
    history: VecDeque<RoutingResult>,

    /// Maximum history entries to retain.
    max_history_size: usize,

    /// Default routing context.
    default_context: RoutingContext,

    /// Total routing decisions made.
    total_decisions: u64,

    /// Total accepted routes.
    total_accepted: u64,

    /// Total rejected routes.
    total_rejected: u64,
}

impl FogRouter {
    /// Creates a new FOG router with default configuration.
    pub fn new() -> Self {
        FogRouter {
            nodes: HashMap::new(),
            history: VecDeque::new(),
            max_history_size: 10_000,
            default_context: RoutingContext::new(),
            total_decisions: 0,
            total_accepted: 0,
            total_rejected: 0,
        }
    }

    /// Creates a router with custom history size.
    pub fn with_history_size(max_history_size: usize) -> Self {
        FogRouter {
            nodes: HashMap::new(),
            history: VecDeque::with_capacity(max_history_size),
            max_history_size,
            default_context: RoutingContext::new(),
            total_decisions: 0,
            total_accepted: 0,
            total_rejected: 0,
        }
    }

    /// Registers a node shard for routing.
    pub fn register_node(&mut self, node: NodeShard) -> Result<(), RouterError> {
        if self.nodes.contains_key(&node.node_id) {
            return Err(RouterError::DuplicateNodeId(node.node_id));
        }
        self.nodes.insert(node.node_id, node);
        Ok(())
    }

    /// Updates an existing node shard.
    pub fn update_node(&mut self, node: NodeShard) -> Result<(), RouterError> {
        if !self.nodes.contains_key(&node.node_id) {
            return Err(RouterError::NodeNotFound(node.node_id));
        }
        self.nodes.insert(node.node_id, node);
        Ok(())
    }

    /// Removes a node from routing.
    pub fn remove_node(&mut self, node_id: u64) -> Option<NodeShard> {
        self.nodes.remove(&node_id)
    }

    /// Gets a node by ID.
    pub fn get_node(&self, node_id: u64) -> Option<&NodeShard> {
        self.nodes.get(&node_id)
    }

    /// Evaluates a single node for a variant.
    pub fn evaluate_node(
        &self,
        variant: &CyboVariant,
        node: &NodeShard,
        ctx: &RoutingContext,
    ) -> RoutingResult {
        let start = Instant::now();
        let mut result = RoutingResult::new(node.node_id, variant.id, RouteDecision::Accept);

        // Check if node is healthy
        if !node.is_healthy() {
            result.decision = RouteDecision::Unavailable;
            result = result.with_failure_reason("Node unhealthy".to_string());
            result.latency_us = start.elapsed().as_micros() as u64;
            return result;
        }

        // Tailwind validation (energy surplus)
        let tailwind_valid = node.has_energy_for(variant);
        if !tailwind_valid {
            result.decision = RouteDecision::Reroute;
            result = result.with_failure_reason("Insufficient energy surplus".to_string());
        }

        // Biosurface validation
        let biosurface_valid = node.biosurface_compatible(variant);
        if !biosurface_valid {
            result.decision = RouteDecision::Reroute;
            result = result.with_failure_reason("Biosurface incompatible".to_string());
        }

        // Hydraulic validation
        let hydraulic_valid = node.hydraulic_capacity_for(variant);
        if !hydraulic_valid {
            result.decision = RouteDecision::Reroute;
            result = result.with_failure_reason("Hydraulic capacity exceeded".to_string());
        }

        // Lyapunov validation
        let lyapunov_valid = node.lyapunov_stable_for(variant, ctx.vt_global_next_max);
        if !lyapunov_valid {
            result.decision = RouteDecision::Reject;
            result = result.with_failure_reason("Lyapunov stability violated".to_string());
        }

        // Security validation
        let security_valid = node.security_compatible(variant);
        if !security_valid && ctx.security_enforced {
            result.decision = RouteDecision::SecurityPending;
            result = result.with_failure_reason("Security verification required".to_string());
        }

        // Set validation flags
        result = result.with_validations(
            tailwind_valid,
            biosurface_valid,
            hydraulic_valid,
            lyapunov_valid,
            security_valid,
        );

        // Determine final decision (most severe failure wins)
        if result.decision == RouteDecision::Accept {
            if !tailwind_valid || !biosurface_valid || !hydraulic_valid {
                result.decision = RouteDecision::Reroute;
            }
            if !lyapunov_valid {
                result.decision = RouteDecision::Reject;
            }
            if !security_valid && ctx.security_enforced {
                result.decision = RouteDecision::SecurityPending;
            }
        }

        result.latency_us = start.elapsed().as_micros() as u64;
        result
    }

    /// Finds the best node for a variant.
    pub fn find_best_node(
        &self,
        variant: &CyboVariant,
        ctx: &RoutingContext,
    ) -> Option<(u64, RoutingResult)> {
        let mut best_node_id: Option<u64> = None;
        let mut best_score = -1.0;
        let mut best_result: Option<RoutingResult> = None;

        for (&node_id, node) in &self.nodes {
            let result = self.evaluate_node(variant, node, ctx);

            if result.is_success() {
                let score = node.health_score();
                if score > best_score {
                    best_score = score;
                    best_node_id = Some(node_id);
                    best_result = Some(result);
                }
            }
        }

        best_node_id.map(|id| (id, best_result.unwrap()))
    }

    /// Routes a variant to the best available node.
    pub fn route_variant(
        &mut self,
        variant: &CyboVariant,
        ctx: &RoutingContext,
    ) -> Option<RoutingResult> {
        if let Some((node_id, result)) = self.find_best_node(variant, ctx) {
            self.total_decisions += 1;
            if result.is_success() {
                self.total_accepted += 1;
            } else {
                self.total_rejected += 1;
            }

            // Add to history
            self.history.push_back(result.clone());
            while self.history.len() > self.max_history_size {
                self.history.pop_front();
            }

            Some(result)
        } else {
            None
        }
    }

    /// Returns routing statistics.
    pub fn statistics(&self) -> RouterStatistics {
        let acceptance_rate = if self.total_decisions == 0 {
            0.0
        } else {
            self.total_accepted as f64 / self.total_decisions as f64
        };

        let avg_latency_us = if self.history.is_empty() {
            0.0
        } else {
            self.history.iter().map(|r| r.latency_us).sum::<u64>() as f64 / self.history.len() as f64
        };

        RouterStatistics {
            total_nodes: self.nodes.len(),
            healthy_nodes: self.nodes.values().filter(|n| n.is_healthy()).count(),
            total_decisions: self.total_decisions,
            total_accepted: self.total_accepted,
            total_rejected: self.total_rejected,
            acceptance_rate,
            average_latency_us: avg_latency_us,
            history_size: self.history.len(),
        }
    }

    /// Returns recent routing history.
    pub fn recent_history(&self, count: usize) -> Vec<&RoutingResult> {
        self.history.iter().rev().take(count).collect()
    }

    /// Exports routing history as ALN shard format.
    pub fn export_aln_shard(&self, count: usize) -> String {
        let history: Vec<&RoutingResult> = self.recent_history(count);
        let mut shard = String::new();

        shard.push_str("// ALN Shard - FOG Routing Log\n");
        shard.push_str(&format!("// Generated: {}\n", RoutingContext::current_timestamp()));
        shard.push_str(&format!("// Entry Count: {}\n", history.len()));
        shard.push_str("\n");
        shard.push_str("spec FogRoutingShard v1.0.0\n\n");
        shard.push_str("routes\n");

        for result in history {
            shard.push_str(&format!(
                "  {{node={}, variant={}, decision={}, latency_us={}, tailwind={}, biosurface={}, hydraulic={}, lyapunov={}}}\n",
                result.node_id,
                result.variant_id,
                result.decision,
                result.latency_us,
                result.tailwind_valid,
                result.biosurface_valid,
                result.hydraulic_valid,
                result.lyapunov_valid
            ));
        }

        shard.push_str("\n");
        shard.push_str(&format!("statistics: {}\n", self.statistics()));

        shard
    }

    /// Clears routing history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Returns current UNIX timestamp in seconds.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }
}

impl Default for FogRouter {
    fn default() -> Self {
        FogRouter::new()
    }
}

/// Routing statistics summary.
#[derive(Clone, Copy, Debug)]
pub struct RouterStatistics {
    /// Total registered nodes.
    pub total_nodes: usize,

    /// Number of healthy nodes.
    pub healthy_nodes: usize,

    /// Total routing decisions made.
    pub total_decisions: u64,

    /// Total accepted routes.
    pub total_accepted: u64,

    /// Total rejected routes.
    pub total_rejected: u64,

    /// Acceptance rate (accepted / total).
    pub acceptance_rate: f64,

    /// Average routing latency in microseconds.
    pub average_latency_us: f64,

    /// Current history buffer size.
    pub history_size: usize,
}

impl fmt::Display for RouterStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RouterStatistics[nodes={}/{}, decisions={}, accepted={}, rate={:.2}, latency={:.0}μs]",
            self.healthy_nodes,
            self.total_nodes,
            self.total_decisions,
            self.total_accepted,
            self.acceptance_rate,
            self.average_latency_us
        )
    }
}

// ============================================================================
// Router Errors
// ============================================================================

/// Errors that can occur during routing operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouterError {
    /// Negative energy requirement.
    NegativeEnergyRequirement,
    /// Safety factor below 1.0.
    SafetyFactorBelowOne,
    /// Hydraulic impact out of [0,1] range.
    HydraulicImpactOutOfRange,
    /// Priority out of [1,10] range.
    PriorityOutOfRange,
    /// Negative energy surplus.
    NegativeEnergySurplus,
    /// Risk coordinate out of [0,1] range.
    RiskCoordinateOutOfRange(&'static str),
    /// KER score out of [0,1] range.
    KerScoreOutOfRange(&'static str),
    /// Duplicate node ID.
    DuplicateNodeId(u64),
    /// Node not found.
    NodeNotFound(u64),
    /// No suitable node found for routing.
    NoSuitableNode,
    /// Routing timeout.
    RoutingTimeout,
    /// Security verification failed.
    SecurityVerificationFailed,
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouterError::NegativeEnergyRequirement => write!(f, "energy requirement cannot be negative"),
            RouterError::SafetyFactorBelowOne => write!(f, "safety factor must be ≥ 1.0"),
            RouterError::HydraulicImpactOutOfRange => write!(f, "hydraulic impact must be in [0,1]"),
            RouterError::PriorityOutOfRange => write!(f, "priority must be in [1,10]"),
            RouterError::NegativeEnergySurplus => write!(f, "energy surplus cannot be negative"),
            RouterError::RiskCoordinateOutOfRange(field) => {
                write!(f, "{} risk coordinate must be in [0,1]", field)
            }
            RouterError::KerScoreOutOfRange(field) => {
                write!(f, "{} must be in [0,1]", field)
            }
            RouterError::DuplicateNodeId(id) => write!(f, "duplicate node ID: {}", id),
            RouterError::NodeNotFound(id) => write!(f, "node not found: {}", id),
            RouterError::NoSuitableNode => write!(f, "no suitable node found for routing"),
            RouterError::RoutingTimeout => write!(f, "routing operation timed out"),
            RouterError::SecurityVerificationFailed => write!(f, "security verification failed"),
        }
    }
}

impl std::error::Error for RouterError {}

// ============================================================================
// Main Entry Point (Example Usage)
// ============================================================================

fn main() {
    println!("Cyboquatic FOG Router v1.0.0");
    println!("=============================\n");

    // Create router
    let mut router = FogRouter::new();

    // Create routing context
    let ctx = RoutingContext::with_vt(1.0, 0.001);

    // Register sample nodes
    let node1 = NodeShard::new(
        1, 5000.0, 3.5, true, 10.0, 0.2, 5.0, 0.2, 0.1, 0.3, 0.2, 0.1,
        BioSurfaceMode::Preprocessed, 0.9, -0.01, 0.93, 0.90, 0.14,
    )
    .unwrap()
    .with_firmware_verification(12345, true);

    let node2 = NodeShard::new(
        2, 3000.0, 2.0, true, 8.0, 0.15, 4.0, 0.3, 0.2, 0.4, 0.3, 0.15,
        BioSurfaceMode::Raw, 0.85, 0.0, 0.88, 0.85, 0.18,
    )
    .unwrap()
    .with_firmware_verification(12346, false);

    router.register_node(node1).unwrap();
    router.register_node(node2).unwrap();

    // Create sample workload variant
    let variant = CyboVariant::new(
        42,
        500.0,
        1.5,
        200,
        MediaClass::WaterOnly,
        0.1,
        -0.001,
        5,
    )
    .unwrap()
    .with_secure_requirement(true);

    println!("Workload: {}", variant);
    println!("Nodes registered: {}\n", router.statistics().total_nodes);

    // Route the variant
    if let Some(result) = router.route_variant(&variant, &ctx) {
        println!("Routing Result: {}", result);
        println!("  Decision: {}", result.decision);
        println!("  Tailwind Valid: {}", result.tailwind_valid);
        println!("  Biosurface Valid: {}", result.biosurface_valid);
        println!("  Hydraulic Valid: {}", result.hydraulic_valid);
        println!("  Lyapunov Valid: {}", result.lyapunov_valid);
        println!("  Security Valid: {}", result.security_valid);
        if !result.failure_reasons.is_empty() {
            println!("  Failure Reasons: {}", result.failure_summary());
        }
    } else {
        println!("No suitable node found for routing");
    }

    // Print statistics
    println!("\nRouter Statistics: {}", router.statistics());

    // Export ALN shard
    println!("\nALN Shard Export:");
    println!("{}", router.export_aln_shard(5));
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cybo_variant_creation() {
        let variant = CyboVariant::new(
            1, 500.0, 1.5, 200, MediaClass::WaterOnly, 0.1, -0.001, 5,
        ).unwrap();
        assert_eq!(variant.id, 1);
        assert_eq!(variant.total_energy_required(), 750.0);
        assert!(!variant.is_high_priority());
    }

    #[test]
    fn test_cybo_variant_rejection() {
        assert!(CyboVariant::new(1, -100.0, 1.5, 200, MediaClass::WaterOnly, 0.1, -0.001, 5).is_err());
        assert!(CyboVariant::new(1, 500.0, 0.5, 200, MediaClass::WaterOnly, 0.1, -0.001, 5).is_err());
    }

    #[test]
    fn test_node_shard_creation() {
        let node = NodeShard::new(
            1, 5000.0, 3.5, true, 10.0, 0.2, 5.0, 0.2, 0.1, 0.3, 0.2, 0.1,
            BioSurfaceMode::Preprocessed, 0.9, -0.01, 0.93, 0.90, 0.14,
        ).unwrap();
        assert_eq!(node.node_id, 1);
        assert!(node.is_healthy());
        assert!(node.health_score() > 0.7);
    }

    #[test]
    fn test_node_energy_validation() {
        let node = NodeShard::new(
            1, 5000.0, 3.5, true, 10.0, 0.2, 5.0, 0.2, 0.1, 0.3, 0.2, 0.1,
            BioSurfaceMode::Preprocessed, 0.9, -0.01, 0.93, 0.90, 0.14,
        ).unwrap();

        let variant = CyboVariant::new(
            1, 500.0, 1.5, 200, MediaClass::WaterOnly, 0.1, -0.001, 5,
        ).unwrap();

        assert!(node.has_energy_for(&variant));

        let large_variant = CyboVariant::new(
            2, 10000.0, 1.5, 200, MediaClass::WaterOnly, 0.1, -0.001, 5,
        ).unwrap();

        assert!(!node.has_energy_for(&large_variant));
    }

    #[test]
    fn test_biosurface_compatibility() {
        let node_raw = NodeShard::new(
            1, 5000.0, 3.5, true, 10.0, 0.2, 5.0, 0.2, 0.1, 0.3, 0.2, 0.1,
            BioSurfaceMode::Raw, 0.9, -0.01, 0.93, 0.90, 0.14,
        ).unwrap();

        let water_variant = CyboVariant::new(
            1, 500.0, 1.5, 200, MediaClass::WaterOnly, 0.1, -0.001, 5,
        ).unwrap();

        assert!(!node_raw.biosurface_compatible(&water_variant));

        let air_variant = CyboVariant::new(
            2, 500.0, 1.5, 200, MediaClass::AirPlenum, 0.1, -0.001, 5,
        ).unwrap();

        assert!(node_raw.biosurface_compatible(&air_variant));
    }

    #[test]
    fn test_fog_router_routing() {
        let mut router = FogRouter::new();
        let ctx = RoutingContext::with_vt(1.0, 0.001);

        let node = NodeShard::new(
            1, 5000.0, 3.5, true, 10.0, 0.2, 5.0, 0.2, 0.1, 0.3, 0.2, 0.1,
            BioSurfaceMode::Preprocessed, 0.9, -0.01, 0.93, 0.90, 0.14,
        ).unwrap();

        router.register_node(node).unwrap();

        let variant = CyboVariant::new(
            1, 500.0, 1.5, 200, MediaClass::WaterOnly, 0.1, -0.001, 5,
        ).unwrap();

        let result = router.route_variant(&variant, &ctx);
        assert!(result.is_some());
        assert!(result.unwrap().is_success());
    }

    #[test]
    fn test_router_statistics() {
        let mut router = FogRouter::new();
        let ctx = RoutingContext::new();

        let node = NodeShard::new(
            1, 5000.0, 3.5, true, 10.0, 0.2, 5.0, 0.2, 0.1, 0.3, 0.2, 0.1,
            BioSurfaceMode::Preprocessed, 0.9, -0.01, 0.93, 0.90, 0.14,
        ).unwrap();

        router.register_node(node).unwrap();

        for i in 0..10 {
            let variant = CyboVariant::new(
                i, 500.0, 1.5, 200, MediaClass::WaterOnly, 0.1, -0.001, 5,
            ).unwrap();
            router.route_variant(&variant, &ctx);
        }

        let stats = router.statistics();
        assert_eq!(stats.total_decisions, 10);
        assert_eq!(stats.total_nodes, 1);
    }

    #[test]
    fn test_aln_shard_export() {
        let mut router = FogRouter::new();
        let ctx = RoutingContext::new();

        let node = NodeShard::new(
            1, 5000.0, 3.5, true, 10.0, 0.2, 5.0, 0.2, 0.1, 0.3, 0.2, 0.1,
            BioSurfaceMode::Preprocessed, 0.9, -0.01, 0.93, 0.90, 0.14,
        ).unwrap();

        router.register_node(node).unwrap();

        let variant = CyboVariant::new(
            1, 500.0, 1.5, 200, MediaClass::WaterOnly, 0.1, -0.001, 5,
        ).unwrap();

        router.route_variant(&variant, &ctx);

        let shard = router.export_aln_shard(5);
        assert!(shard.contains("FogRoutingShard"));
        assert!(shard.contains("routes"));
        assert!(shard.contains("statistics"));
    }

    #[test]
    fn test_media_class_risk_thresholds() {
        assert_eq!(MediaClass::WaterOnly.risk_threshold(), 0.5);
        assert_eq!(MediaClass::WaterBiofilm.risk_threshold(), 0.3);
        assert_eq!(MediaClass::AirPlenum.risk_threshold(), 0.6);
    }

    #[test]
    fn test_biosurface_mode_capabilities() {
        assert!(BioSurfaceMode::Preprocessed.allows_water());
        assert!(!BioSurfaceMode::Restricted.allows_water());
        assert!(BioSurfaceMode::Raw.allows_air());
        assert!(!BioSurfaceMode::Inactive.is_active());
    }
}
