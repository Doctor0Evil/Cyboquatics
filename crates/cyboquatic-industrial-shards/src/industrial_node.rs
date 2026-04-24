//! Industrial node shard types matching CyboquaticIndustrialEcosafety2026v1.aln

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Node type enumeration matching ALN schema
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CyboNodeType {
    MarModule,
    FogDesiccator,
    AirGlobe,
    Cain,
    CanalPurifier,
    Other,
}

impl Default for CyboNodeType {
    fn default() -> Self {
        CyboNodeType::Other
    }
}

/// Medium enumeration matching ALN schema
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Medium {
    Water,
    Air,
    Fog,
    Mixed,
}

impl Default for Medium {
    fn default() -> Self {
        Medium::Mixed
    }
}

/// Lane enumeration for governance
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Lane {
    Research,
    Experimental,
    Production,
}

impl Default for Lane {
    fn default() -> Self {
        Lane::Research
    }
}

/// Security response capability
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecurityResponseCap {
    Low,
    Medium,
    High,
}

impl Default for SecurityResponseCap {
    fn default() -> Self {
        SecurityResponseCap::Low
    }
}

/// Fog routing mode
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FogRoutingMode {
    Direct,
    Desiccate,
    Bypass,
    Shutdown,
}

impl Default for FogRoutingMode {
    fn default() -> Self {
        FogRoutingMode::Direct
    }
}

/// Shard header for provenance tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardHeader {
    pub schema_version: String,
    pub generated_at: DateTime<Utc>,
    pub aln_source: String,
}

impl Default for ShardHeader {
    fn default() -> Self {
        ShardHeader {
            schema_version: "1.0.0".to_string(),
            generated_at: Utc::now(),
            aln_source: "qpudatashards/particles/CyboquaticIndustrialEcosafety2026v1.aln".to_string(),
        }
    }
}

/// Main industrial node shard matching CyboquaticIndustrialEcosafety2026v1.aln row CyboNode
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CyboNodeShard {
    /// Header with provenance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<ShardHeader>,

    // Node identity
    pub nodeid: String,
    pub nodetype: CyboNodeType,
    pub medium: Medium,
    pub region: String,
    pub site: String,
    pub lat: f64,
    pub lon: f64,

    // Time window
    #[serde(with = "chrono::serde::ts_seconds_option", default)]
    pub twindowstart: Option<DateTime<Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option", default)]
    pub twindowend: Option<DateTime<Utc>>,

    // CEIM/CPVM eco-benefit
    pub mcapturedkg: f64,
    pub membodiedkg: f64,
    pub mpowerkgco2: f64,
    pub mrefkg: f64,
    pub ecobraw: f64,

    // KER governance
    pub kknowledge: f64,
    pub eecoimpact: f64,
    pub rriskofharm: f64,

    // Normalized risk coordinates (planes)
    pub renergy: f64,
    pub rhydraulics: f64,
    pub rbiology: f64,
    pub rcarbon: f64,
    pub rmaterials: f64,

    // Expanded b/material coords (optional, default 0.0)
    #[serde(default)]
    pub rpfas: f64,
    #[serde(default)]
    pub recoli: f64,
    #[serde(default)]
    pub rnutrient: f64,
    #[serde(default)]
    pub rtds: f64,
    #[serde(default)]
    pub rsat: f64,
    #[serde(default)]
    pub rt90soil: f64,
    #[serde(default)]
    pub rtoxsoil: f64,
    #[serde(default)]
    pub rmicrosoil: f64,
    #[serde(default)]
    pub rt90aquatic: f64,
    #[serde(default)]
    pub rtoxaquatic: f64,
    #[serde(default)]
    pub rmicroaquatic: f64,

    // Lyapunov residual
    pub wenergy: f64,
    pub whydraulics: f64,
    pub wbiology: f64,
    pub wcarbon: f64,
    pub wmaterials: f64,
    pub vresidual: f64,
    pub vresidualmax: f64,

    // Sensor trust (Multonry Dt)
    pub dttrust: f64,
    pub badj: f64,
    pub kadj: f64,
    pub eadj: f64,

    // Safety contracts and lane governance
    pub corridorpresent: bool,
    pub safestepok: bool,
    pub lane: Lane,

    // Kernel and corridor provenance
    pub riskkernelversion: String,
    pub corridortableid: String,
    pub ceimkernelversion: String,
    pub cpvmkernelversion: String,

    // Evidence and signatures
    pub evidencehex: String,
    pub signinghex: String,
    #[serde(default)]
    pub researchhex: String,

    // Operational config
    #[serde(default)]
    pub securityresponsecap: SecurityResponseCap,
    #[serde(default)]
    pub fogroutingmode: FogRoutingMode,
}

impl Default for CyboNodeShard {
    fn default() -> Self {
        CyboNodeShard {
            header: Some(ShardHeader::default()),
            nodeid: String::new(),
            nodetype: CyboNodeType::default(),
            medium: Medium::default(),
            region: String::new(),
            site: String::new(),
            lat: 0.0,
            lon: 0.0,
            twindowstart: None,
            twindowend: None,
            mcapturedkg: 0.0,
            membodiedkg: 0.0,
            mpowerkgco2: 0.0,
            mrefkg: 1.0,
            ecobraw: 0.0,
            kknowledge: 0.0,
            eecoimpact: 0.0,
            rriskofharm: 1.0,
            renergy: 0.0,
            rhydraulics: 0.0,
            rbiology: 0.0,
            rcarbon: 0.0,
            rmaterials: 0.0,
            rpfas: 0.0,
            recoli: 0.0,
            rnutrient: 0.0,
            rtds: 0.0,
            rsat: 0.0,
            rt90soil: 0.0,
            rtoxsoil: 0.0,
            rmicrosoil: 0.0,
            rt90aquatic: 0.0,
            rtoxaquatic: 0.0,
            rmicroaquatic: 0.0,
            wenergy: 0.2,
            whydraulics: 0.2,
            wbiology: 0.2,
            wcarbon: 0.2,
            wmaterials: 0.2,
            vresidual: 0.0,
            vresidualmax: 1.0,
            dttrust: 1.0,
            badj: 1.0,
            kadj: 1.0,
            eadj: 1.0,
            corridorpresent: false,
            safestepok: false,
            lane: Lane::default(),
            riskkernelversion: String::new(),
            corridortableid: String::new(),
            ceimkernelversion: String::new(),
            cpvmkernelversion: String::new(),
            evidencehex: String::new(),
            signinghex: String::new(),
            researchhex: String::new(),
            securityresponsecap: SecurityResponseCap::default(),
            fogroutingmode: FogRoutingMode::default(),
        }
    }
}

impl CyboNodeShard {
    /// Create a new shard with minimal required fields
    pub fn new(nodeid: impl Into<String>, nodetype: CyboNodeType, medium: Medium) -> Self {
        CyboNodeShard {
            nodeid: nodeid.into(),
            nodetype,
            medium,
            ..Default::default()
        }
    }

    /// Check if this shard is in a production-eligible lane
    pub fn is_production_lane(&self) -> bool {
        matches!(self.lane, Lane::Production)
    }

    /// Check if this shard meets basic admissibility requirements
    pub fn is_admissible(&self) -> bool {
        self.corridorpresent 
            && self.vresidual <= self.vresidualmax 
            && self.rriskofharm <= 0.13
    }
}
