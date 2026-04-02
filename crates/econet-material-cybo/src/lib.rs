//! EcoNet Material Cybo – biodegradable, non-toxic substrates
//! for Cyboquatic industrial machinery.
//!
//! This crate encodes AntSafeSubstrate and CyboNodeCompatible traits
//! with Phoenix-anchored corridors (t90, rtox, rmicro, caloric density)
//! and KER governance consistent with the Cyboquatic ecosafety spine.[file:11][file:22]

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use core::fmt;
use core::time::Duration;

// For timestamping and registry HashMap, gate std with a feature if desired.
#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::time::SystemTime;

/// Dimensionless risk coordinate r ∈ [0,1].[file:22]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct RiskCoord(f64);

impl RiskCoord {
    /// Creates a new clamped coordinate in [0,1].
    #[inline]
    pub fn new_clamped(v: f64) -> Self {
        let v = if v < 0.0 {
            0.0
        } else if v > 1.0 {
            1.0
        } else {
            v
        };
        RiskCoord(v)
    }

    /// Returns the underlying scalar value.
    #[inline]
    pub fn value(self) -> f64 {
        self.0
    }

    /// Returns true if this coordinate is in the hard band (violation).
    #[inline]
    pub fn is_hard_band(self) -> bool {
        self.0 >= 1.0
    }

    /// Returns true if this coordinate is in the safe+gold region.
    #[inline]
    pub fn is_gold_band(self) -> bool {
        self.0 <= 0.5
    }
}

impl Default for RiskCoord {
    fn default() -> Self {
        RiskCoord(0.0)
    }
}

/// Corridor bands for one physical metric, with a safe/gold/hard split.[file:22]
#[derive(Clone, Copy, Debug)]
pub struct CorridorBands {
    /// Lower safe bound.
    pub x_safe: f64,
    /// Gold threshold (end of safe, start of gold).
    pub x_gold: f64,
    /// Hard upper bound (violation above this).
    pub x_hard: f64,
}

impl CorridorBands {
    /// Creates new corridor bands with ordering validation.
    pub fn new(x_safe: f64, x_gold: f64, x_hard: f64) -> Result<Self, MaterialError> {
        if x_safe > x_gold || x_gold > x_hard {
            return Err(MaterialError::InvalidCorridorOrder);
        }
        Ok(CorridorBands {
            x_safe,
            x_gold,
            x_hard,
        })
    }

    /// Normalizes a raw value into a RiskCoord using safe/gold/hard bands.[file:22]
    pub fn normalize(&self, x: f64) -> RiskCoord {
        if x <= self.x_safe {
            return RiskCoord::new_clamped(0.0);
        }
        if x >= self.x_hard {
            return RiskCoord::new_clamped(1.0);
        }
        if x <= self.x_gold {
            let num = x - self.x_safe;
            let den = (self.x_gold - self.x_safe).max(f64::EPSILON);
            return RiskCoord::new_clamped(num / den * 0.5);
        }
        let num = x - self.x_gold;
        let den = (self.x_hard - self.x_gold).max(f64::EPSILON);
        RiskCoord::new_clamped(0.5 + num / den * 0.5)
    }

    /// Returns true if the value is within the hard band (not a violation).
    pub fn within_hard(&self, x: f64) -> bool {
        x >= self.x_safe && x <= self.x_hard
    }
}

/// Vector of risk coordinates with optional labels.[file:22]
#[derive(Clone, Debug)]
pub struct RiskVector {
    /// Risk coordinates.
    pub coords: Vec<RiskCoord>,
    /// Human-readable labels, same length as coords.
    pub labels: Vec<String>,
}

impl RiskVector {
    /// Creates a new unlabeled vector (r_0, r_1, ...).
    pub fn new(coords: Vec<RiskCoord>) -> Self {
        let labels = (0..coords.len())
            .map(|i| alloc::format!("r_{}", i))
            .collect();
        RiskVector { coords, labels }
    }

    /// Creates a labeled vector; lengths must match.
    pub fn with_labels(coords: Vec<RiskCoord>, labels: Vec<String>) -> Self {
        assert_eq!(coords.len(), labels.len());
        RiskVector { coords, labels }
    }

    /// Maximum coordinate.
    pub fn max(&self) -> RiskCoord {
        self.coords
            .iter()
            .copied()
            .max_by(|a, b| {
                a.value()
                    .partial_cmp(&b.value())
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .unwrap_or(RiskCoord::default())
    }

    /// Simple mean of coordinates.
    pub fn mean(&self) -> f64 {
        if self.coords.is_empty() {
            return 0.0;
        }
        self.coords.iter().map(|r| r.value()).sum::<f64>() / self.coords.len() as f64
    }
}

impl Default for RiskVector {
    fn default() -> Self {
        RiskVector::new(Vec::new())
    }
}

/// K/E/R triad for ecological impact assessment.[file:22]
#[derive(Clone, Copy, Debug)]
pub struct KerTriad {
    /// Fraction of steps that are Lyapunov- and corridor-safe.
    pub k_knowledge: f64,
    /// Eco-impact: 1 - max risk (or similar eco-benefit metric).
    pub e_ecoimpact: f64,
    /// Max risk-of-harm across coordinates.
    pub r_risk_of_harm: f64,
}

impl KerTriad {
    /// Creates a clamped KER triad.
    pub fn new(k: f64, e: f64, r: f64) -> Self {
        KerTriad {
            k_knowledge: k.clamp(0.0, 1.0),
            e_ecoimpact: e.clamp(0.0, 1.0),
            r_risk_of_harm: r.clamp(0.0, 1.0),
        }
    }

    /// Returns true if this triad meets Cyboquatic deployment thresholds.[file:22]
    pub fn meets_deployment_criteria(&self) -> bool {
        self.k_knowledge >= 0.90 && self.e_ecoimpact >= 0.90 && self.r_risk_of_harm <= 0.13
    }

    /// Composite score combining knowledge, eco-impact, and inverse risk.
    pub fn composite_score(&self) -> f64 {
        (self.k_knowledge + self.e_ecoimpact + (1.0 - self.r_risk_of_harm)) / 3.0
    }
}

impl Default for KerTriad {
    fn default() -> Self {
        KerTriad::new(1.0, 1.0, 0.0)
    }
}

impl fmt::Display for KerTriad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "KER(k={:.3}, e={:.3}, r={:.3}, score={:.3})",
            self.k_knowledge,
            self.e_ecoimpact,
            self.r_risk_of_harm,
            self.composite_score()
        )
    }
}

// ============================================================================
// Material Kinetics (Biodegradation Profiles)
// ============================================================================

/// Complete biodegradation kinetics profile for a material substrate.[file:22]
///
/// All fields are measured via standardized protocols (ISO 14851, OECD 202,
/// EPA 537, etc.) under Phoenix-class conditions.[file:11][file:19]
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialKinetics {
    /// Time to 90% biodegradation (days). Lower is better.
    pub t90_days: f64,
    /// Toxicity risk coordinate (0.0 = non-toxic, 1.0 = highly toxic).
    pub r_tox: f64,
    /// Microplastic formation risk (0.0 = none, 1.0 = severe).
    pub r_micro: f64,
    /// Leaching risk for contaminants of emerging concern.
    pub r_leach_cec: f64,
    /// PFAS residue risk (0.0 = none detected, 1.0 = above threshold).
    pub r_pfas_resid: f64,
    /// Caloric density (MJ/kg), used for baiting/energy-risk surrogate.
    pub caloric_density_mj_per_kg: f64,
    /// Optional batch identifier.
    pub batch_id: Option<String>,
    /// Measurement timestamp (UNIX epoch seconds).
    pub measured_timestamp: u64,
}

impl MaterialKinetics {
    /// Creates a new kinetics profile with validation.[file:22]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        t90_days: f64,
        r_tox: f64,
        r_micro: f64,
        r_leach_cec: f64,
        r_pfas_resid: f64,
        caloric_density_mj_per_kg: f64,
        measured_timestamp: u64,
    ) -> Result<Self, MaterialError> {
        if t90_days < 0.0 {
            return Err(MaterialError::NegativeT90);
        }
        if !(0.0..=1.0).contains(&r_tox) {
            return Err(MaterialError::RiskOutOfRange("r_tox"));
        }
        if !(0.0..=1.0).contains(&r_micro) {
            return Err(MaterialError::RiskOutOfRange("r_micro"));
        }
        if !(0.0..=1.0).contains(&r_leach_cec) {
            return Err(MaterialError::RiskOutOfRange("r_leach_cec"));
        }
        if !(0.0..=1.0).contains(&r_pfas_resid) {
            return Err(MaterialError::RiskOutOfRange("r_pfas_resid"));
        }
        if caloric_density_mj_per_kg < 0.0 {
            return Err(MaterialError::NegativeCaloricDensity);
        }

        Ok(MaterialKinetics {
            t90_days,
            r_tox,
            r_micro,
            r_leach_cec,
            r_pfas_resid,
            caloric_density_mj_per_kg,
            batch_id: None,
            measured_timestamp,
        })
    }

    /// Sets batch identifier.
    pub fn with_batch(mut self, batch_id: String) -> Self {
        self.batch_id = Some(batch_id);
        self
    }

    /// Returns true if PFAS-free (within numerical tolerance).
    pub fn is_pfas_free(&self) -> bool {
        self.r_pfas_resid < 0.01
    }

    /// Returns true if rapidly biodegradable (t90 < 90 days).
    pub fn is_rapidly_biodegradable(&self) -> bool {
        self.t90_days < 90.0
    }

    /// Returns true if low toxicity (r_tox < 0.1).
    pub fn is_low_toxicity(&self) -> bool {
        self.r_tox < 0.1
    }

    /// Simple average of core risk metrics.
    pub fn overall_risk_score(&self) -> f64 {
        (self.r_tox + self.r_micro + self.r_leach_cec + self.r_pfas_resid) / 4.0
    }
}

impl Default for MaterialKinetics {
    fn default() -> Self {
        MaterialKinetics {
            t90_days: 120.0,
            r_tox: 0.05,
            r_micro: 0.02,
            r_leach_cec: 0.05,
            r_pfas_resid: 0.01,
            caloric_density_mj_per_kg: 0.15,
            batch_id: None,
            measured_timestamp: 0,
        }
    }
}

impl fmt::Display for MaterialKinetics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MaterialKinetics[t90={:.1}d, tox={:.3}, micro={:.3}, leach={:.3}, pfas={:.3}]",
            self.t90_days, self.r_tox, self.r_micro, self.r_leach_cec, self.r_pfas_resid
        )
    }
}

// ============================================================================
// Material Corridor Bands (Safety Thresholds)
// ============================================================================

/// Safety corridors for material properties under a deployment regime.[file:22]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialCorridors {
    /// Max acceptable t90 (hard band).
    pub t90_max_days: f64,
    /// Gold-band t90 (optimal biodegradation).
    pub t90_gold_days: f64,
    /// Gold-band toxicity upper bound.
    pub r_tox_gold_max: f64,
    /// Max microplastic risk.
    pub r_micro_max: f64,
    /// Max leachate risk.
    pub r_leach_max: f64,
    /// Max PFAS risk.
    pub r_pfas_max: f64,
    /// Max caloric density (MJ/kg).
    pub caloric_density_max: f64,
}

impl Default for MaterialCorridors {
    fn default() -> Self {
        MaterialCorridors {
            t90_max_days: 180.0,
            t90_gold_days: 120.0,
            r_tox_gold_max: 0.10,
            r_micro_max: 0.05,
            r_leach_max: 0.10,
            r_pfas_max: 0.10,
            caloric_density_max: 0.30,
        }
    }
}

impl MaterialCorridors {
    /// Creates custom corridors with validation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        t90_max_days: f64,
        t90_gold_days: f64,
        r_tox_gold_max: f64,
        r_micro_max: f64,
        r_leach_max: f64,
        r_pfas_max: f64,
        caloric_density_max: f64,
    ) -> Result<Self, MaterialError> {
        if t90_gold_days > t90_max_days {
            return Err(MaterialError::InvalidCorridorOrder);
        }
        if !(0.0..=1.0).contains(&r_tox_gold_max) {
            return Err(MaterialError::RiskOutOfRange("r_tox_gold_max"));
        }
        if !(0.0..=1.0).contains(&r_micro_max) {
            return Err(MaterialError::RiskOutOfRange("r_micro_max"));
        }
        if !(0.0..=1.0).contains(&r_leach_max) {
            return Err(MaterialError::RiskOutOfRange("r_leach_max"));
        }
        if !(0.0..=1.0).contains(&r_pfas_max) {
            return Err(MaterialError::RiskOutOfRange("r_pfas_max"));
        }

        Ok(MaterialCorridors {
            t90_max_days,
            t90_gold_days,
            r_tox_gold_max,
            r_micro_max,
            r_leach_max,
            r_pfas_max,
            caloric_density_max,
        })
    }

    /// Strict corridors for sensitive environments (marine, drinking water).
    pub fn strict() -> Self {
        MaterialCorridors {
            t90_max_days: 90.0,
            t90_gold_days: 60.0,
            r_tox_gold_max: 0.05,
            r_micro_max: 0.02,
            r_leach_max: 0.05,
            r_pfas_max: 0.05,
            caloric_density_max: 0.20,
        }
    }

    /// Relaxed corridors for industrial environments.
    pub fn relaxed() -> Self {
        MaterialCorridors {
            t90_max_days: 365.0,
            t90_gold_days: 180.0,
            r_tox_gold_max: 0.20,
            r_micro_max: 0.10,
            r_leach_max: 0.20,
            r_pfas_max: 0.15,
            caloric_density_max: 0.50,
        }
    }

    /// Returns t90 corridor bands for normalization.
    pub fn t90_bands(&self) -> CorridorBands {
        CorridorBands {
            x_safe: 0.0,
            x_gold: self.t90_gold_days,
            x_hard: self.t90_max_days,
        }
    }
}

impl fmt::Display for MaterialCorridors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MaterialCorridors[t90_max={:.0}d, t90_gold={:.0}d, tox_max={:.2}, pfas_max={:.2}]",
            self.t90_max_days, self.t90_gold_days, self.r_tox_gold_max, self.r_pfas_max
        )
    }
}

// ============================================================================
// Material Risk Coordinates
// ============================================================================

/// Computed risk coordinates for a material substrate.[file:22]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialRisks {
    /// Risk for biodegradation time.
    pub r_t90: RiskCoord,
    /// Risk for toxicity.
    pub r_tox: RiskCoord,
    /// Risk for microplastic formation.
    pub r_micro: RiskCoord,
    /// Risk for chemical leaching.
    pub r_leach_cec: RiskCoord,
    /// Risk for PFAS residue.
    pub r_pfas_resid: RiskCoord,
}

impl MaterialRisks {
    /// Computes risks from kinetics and corridors.
    pub fn from_kinetics(k: &MaterialKinetics, c: &MaterialCorridors) -> Self {
        let t_corr = c.t90_bands();
        let r_t90 = t_corr.normalize(k.t90_days);

        let r_tox = RiskCoord::new_clamped(k.r_tox / c.r_tox_gold_max);
        let r_micro = RiskCoord::new_clamped(k.r_micro / c.r_micro_max);
        let r_leach_cec = RiskCoord::new_clamped(k.r_leach_cec / c.r_leach_max);
        let r_pfas_resid = RiskCoord::new_clamped(k.r_pfas_resid / c.r_pfas_max);

        MaterialRisks {
            r_t90,
            r_tox,
            r_micro,
            r_leach_cec,
            r_pfas_resid,
        }
    }

    /// Converts to a RiskVector for ecosafety kernels.
    pub fn to_vector(&self) -> RiskVector {
        RiskVector::with_labels(
            vec![
                self.r_t90,
                self.r_tox,
                self.r_micro,
                self.r_leach_cec,
                self.r_pfas_resid,
            ],
            vec![
                "r_t90".to_string(),
                "r_tox".to_string(),
                "r_micro".to_string(),
                "r_leach_cec".to_string(),
                "r_pfas_resid".to_string(),
            ],
        )
    }

    /// Eco-impact score: 1 - weighted average risk (higher is better).
    pub fn ecoimpact_score(&self, weights: &[f64; 5]) -> f64 {
        let risks = [
            self.r_t90.value(),
            self.r_tox.value(),
            self.r_micro.value(),
            self.r_leach_cec.value(),
            self.r_pfas_resid.value(),
        ];
        let mut s = 0.0;
        let mut wsum = 0.0;
        for (r, w) in risks.iter().zip(weights.iter()) {
            let w = w.max(0.0);
            s += w * r;
            wsum += w;
        }
        if wsum == 0.0 {
            1.0
        } else {
            let r_bar = s / wsum;
            (1.0 - r_bar).max(0.0)
        }
    }

    /// Computes KER triad (K=1 for static material score, E from ecoimpact, R from max risk).
    pub fn ker(&self, weights: &[f64; 5]) -> KerTriad {
        let e = self.ecoimpact_score(weights);
        let max_r = self.to_vector().max().value();
        KerTriad::new(1.0, e, max_r)
    }

    /// Maximum risk coordinate.
    pub fn max_risk(&self) -> RiskCoord {
        [self.r_t90, self.r_tox, self.r_micro, self.r_leach_cec, self.r_pfas_resid]
            .iter()
            .copied()
            .max_by(|a, b| {
                a.value()
                    .partial_cmp(&b.value())
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .unwrap_or(RiskCoord::default())
    }

    /// True if all risks are within acceptable bands (no hard violation).
    pub fn all_acceptable(&self) -> bool {
        !self.max_risk().is_hard_band()
    }
}

impl Default for MaterialRisks {
    fn default() -> Self {
        MaterialRisks {
            r_t90: RiskCoord::default(),
            r_tox: RiskCoord::default(),
            r_micro: RiskCoord::default(),
            r_leach_cec: RiskCoord::default(),
            r_pfas_resid: RiskCoord::default(),
        }
    }
}

impl fmt::Display for MaterialRisks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MaterialRisks[t90={:.3}, tox={:.3}, micro={:.3}, leach={:.3}, pfas={:.3}, max={:.3}]",
            self.r_t90.value(),
            self.r_tox.value(),
            self.r_micro.value(),
            self.r_leach_cec.value(),
            self.r_pfas_resid.value(),
            self.max_risk().value()
        )
    }
}

// ============================================================================
// Substrate Specification
// ============================================================================

/// Application type for substrate deployment.[file:22]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubstrateApplication {
    /// External casing/housing.
    Casing,
    /// Internal flow media.
    Media,
    /// FlowVac internal components.
    FlowVacInternals,
    /// Structural supports.
    Structural,
    /// Sealing/gasket materials.
    Sealing,
    /// Other/custom application.
    Other,
}

impl fmt::Display for SubstrateApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubstrateApplication::Casing => write!(f, "Casing"),
            SubstrateApplication::Media => write!(f, "Media"),
            SubstrateApplication::FlowVacInternals => write!(f, "FlowVacInternals"),
            SubstrateApplication::Structural => write!(f, "Structural"),
            SubstrateApplication::Sealing => write!(f, "Sealing"),
            SubstrateApplication::Other => write!(f, "Other"),
        }
    }
}

/// Deployment environment for substrate.[file:22]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeploymentEnvironment {
    /// Freshwater (rivers, lakes, municipal water).
    Freshwater,
    /// Marine (ocean, saltwater).
    Marine,
    /// Wastewater treatment.
    Wastewater,
    /// Industrial process water.
    Industrial,
    /// Drinking water (most stringent).
    DrinkingWater,
    /// Soil/terrestrial.
    Terrestrial,
}

impl fmt::Display for DeploymentEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeploymentEnvironment::Freshwater => write!(f, "Freshwater"),
            DeploymentEnvironment::Marine => write!(f, "Marine"),
            DeploymentEnvironment::Wastewater => write!(f, "Wastewater"),
            DeploymentEnvironment::Industrial => write!(f, "Industrial"),
            DeploymentEnvironment::DrinkingWater => write!(f, "DrinkingWater"),
            DeploymentEnvironment::Terrestrial => write!(f, "Terrestrial"),
        }
    }
}

/// Complete substrate specification for Cyboquatic machinery.[file:22]
#[derive(Clone, Debug)]
pub struct SubstrateSpec {
    /// Unique substrate identifier.
    pub id: String,
    /// Material name/description.
    pub name: String,
    /// Manufacturer/supplier information.
    pub manufacturer: String,
    /// Biodegradation kinetics profile.
    pub kinetics: MaterialKinetics,
    /// Safety corridor bands for validation.
    pub corridors: MaterialCorridors,
    /// Whether substrate introduces PFAS mass.
    pub pfas_back_leach: bool,
    /// Whether substrate introduces nutrient mass.
    pub nutrient_back_leach: bool,
    /// Intended application (casing, media, internals, etc.).
    pub application: SubstrateApplication,
    /// Deployment environment.
    pub environment: DeploymentEnvironment,
    /// Installation timestamp (UNIX epoch seconds).
    pub installation_timestamp: Option<u64>,
    /// Expected replacement timestamp (UNIX epoch seconds).
    pub replacement_due_timestamp: Option<u64>,
    /// Cryptographic hash of specification (for audit integrity).
    pub spec_hash: String,
}

impl SubstrateSpec {
    /// Creates a new substrate specification (hash computed later).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        name: String,
        manufacturer: String,
        kinetics: MaterialKinetics,
        corridors: MaterialCorridors,
        application: SubstrateApplication,
        environment: DeploymentEnvironment,
    ) -> Self {
        let spec = SubstrateSpec {
            id,
            name,
            manufacturer,
            kinetics,
            corridors,
            pfas_back_leach: false,
            nutrient_back_leach: false,
            application,
            environment,
            installation_timestamp: None,
            replacement_due_timestamp: None,
            spec_hash: String::new(),
        };
        spec.with_hash()
    }

    /// Computes and sets a specification hash.
    ///
    /// NOTE: In production, replace with a quantum-safe hash. For now this
    /// is just a placeholder concatenation-based hex string.[file:22]
    pub fn with_hash(mut self) -> Self {
        let data = alloc::format!(
            "{}|{}|{}|{:.2}|{:.2}|{:.2}|{:.2}|{:.2}",
            self.id,
            self.name,
            self.manufacturer,
            self.kinetics.t90_days,
            self.kinetics.r_tox,
            self.kinetics.r_micro,
            self.kinetics.r_leach_cec,
            self.kinetics.r_pfas_resid
        );
        // Simple XOR-based toy hash; replace in production.
        let mut acc: u64 = 0;
        for b in data.as_bytes() {
            acc = acc.wrapping_mul(131) ^ (*b as u64);
        }
        self.spec_hash = alloc::format!("{:016x}", acc);
        self
    }

    /// Computes risk coordinates for this substrate.
    pub fn risks(&self) -> MaterialRisks {
        MaterialRisks::from_kinetics(&self.kinetics, &self.corridors)
    }

    /// Computes KER triad for this substrate.
    pub fn ker(&self, weights: &[f64; 5]) -> KerTriad {
        self.risks().ker(weights)
    }

    /// Default equal weights for risk planes.
    pub fn default_weights() -> [f64; 5] {
        [0.2, 0.2, 0.2, 0.2, 0.2]
    }

    /// Strict weights prioritizing PFAS and toxicity.
    pub fn strict_weights() -> [f64; 5] {
        [0.1, 0.3, 0.1, 0.2, 0.3]
    }

    /// Sets installation and replacement timestamps (20–50 year planning).
    #[cfg(feature = "std")]
    pub fn with_lifecycle(mut self, installation: u64, lifetime_days: u64) -> Self {
        self.installation_timestamp = Some(installation);
        self.replacement_due_timestamp = Some(installation + lifetime_days * 24 * 3600);
        self.with_hash()
    }

    /// Returns true if replacement is due (requires std).
    #[cfg(feature = "std")]
    pub fn replacement_due(&self) -> bool {
        if let Some(due) = self.replacement_due_timestamp {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs();
            now >= due
        } else {
            false
        }
    }

    /// Days until replacement due (requires std).
    #[cfg(feature = "std")]
    pub fn days_until_replacement(&self) -> Option<i64> {
        self.replacement_due_timestamp.map(|due| {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs();
            ((due as i64) - (now as i64)) / (24 * 3600)
        })
    }

    /// Validates substrate for a specific environment and KER gates.[file:22]
    pub fn validate_for_environment(&self, env: DeploymentEnvironment) -> Result<(), MaterialError> {
        if self.environment != env {
            return Err(MaterialError::EnvironmentMismatch {
                expected: env,
                actual: self.environment,
            });
        }

        if !self.corridor_ok() {
            return Err(MaterialError::CorridorValidationFailed);
        }

        if !self.node_compatible() {
            return Err(MaterialError::NodeCompatibilityFailed);
        }

        let ker = self.ker(&Self::default_weights());
        if !ker.meets_deployment_criteria() {
            return Err(MaterialError::KerThresholdFailed(ker));
        }

        Ok(())
    }

    /// Exports substrate spec as an ALN shard string.[file:22]
    pub fn export_aln_shard(&self) -> String {
        let risks = self.risks();
        let ker = self.ker(&Self::default_weights());

        alloc::format!(
r#"// ALN Shard - Material Specification
spec MaterialCyboShard v1.0.0

node_id: {id}
material_name: {name}
manufacturer: {man}
application: {app}
environment: {env}

kinetics
  t90_days: {t90:.2}
  r_tox: {rt:.4}
  r_micro: {rm:.4}
  r_leach_cec: {rl:.4}
  r_pfas_resid: {rp:.4}
  caloric_density: {cd:.4}

risks
  r_t90: {r_t90:.4}
  r_tox: {r_tox:.4}
  r_micro: {r_micro:.4}
  r_leach_cec: {r_leach:.4}
  r_pfas_resid: {r_pfas:.4}

ker_triad
  k_knowledge: {kk:.4}
  e_ecoimpact: {ke:.4}
  r_risk_of_harm: {kr:.4}
  composite_score: {kc:.4}

validation
  corridor_ok: {cok}
  node_compatible: {nok}
  deployment_allowed: {dok}
  pfas_free: {pfas}

spec_hash: {hash}
"#,
            id = self.id,
            name = self.name,
            man = self.manufacturer,
            app = self.application,
            env = self.environment,
            t90 = self.kinetics.t90_days,
            rt = self.kinetics.r_tox,
            rm = self.kinetics.r_micro,
            rl = self.kinetics.r_leach_cec,
            rp = self.kinetics.r_pfas_resid,
            cd = self.kinetics.caloric_density_mj_per_kg,
            r_t90 = risks.r_t90.value(),
            r_tox = risks.r_tox.value(),
            r_micro = risks.r_micro.value(),
            r_leach = risks.r_leach_cec.value(),
            r_pfas = risks.r_pfas_resid.value(),
            kk = ker.k_knowledge,
            ke = ker.e_ecoimpact,
            kr = ker.r_risk_of_harm,
            kc = ker.composite_score(),
            cok = self.corridor_ok(),
            nok = self.node_compatible(),
            dok = self.deployment_allowed(&Self::default_weights()),
            pfas = self.is_pfas_free(),
            hash = self.spec_hash
        )
    }
}

impl fmt::Display for SubstrateSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ker = self.ker(&Self::default_weights());
        write!(
            f,
            "SubstrateSpec[id={}, name={}, {}, {}]",
            self.id, self.name, self.application, ker
        )
    }
}

// ============================================================================
// AntSafeSubstrate and CyboNodeCompatible
// ============================================================================

/// Trait for PFAS-safe substrate validation and corridor gates.[file:22]
pub trait AntSafeSubstrate {
    /// Returns material kinetics.
    fn kinetics(&self) -> &MaterialKinetics;
    /// Returns corridor bands.
    fn corridors(&self) -> &MaterialCorridors;

    /// True if all corridor checks pass.
    fn corridor_ok(&self) -> bool {
        let k = self.kinetics();
        let c = self.corridors();

        if k.t90_days > c.t90_max_days {
            return false;
        }
        if k.r_tox > c.r_tox_gold_max {
            return false;
        }
        if k.r_micro > c.r_micro_max {
            return false;
        }
        if k.r_leach_cec > c.r_leach_max {
            return false;
        }
        if k.r_pfas_resid > c.r_pfas_max {
            return false;
        }
        if k.caloric_density_mj_per_kg > c.caloric_density_max {
            return false;
        }
        true
    }

    /// True if PFAS-free.
    fn is_pfas_free(&self) -> bool {
        self.kinetics().is_pfas_free()
    }

    /// True if rapidly biodegradable.
    fn is_rapidly_biodegradable(&self) -> bool {
        self.kinetics().is_rapidly_biodegradable()
    }
}

/// Trait for node compatibility in distributed Cyboquatic systems.[file:22]
pub trait CyboNodeCompatible: AntSafeSubstrate {
    /// True if this substrate introduces PFAS mass.
    fn introduces_pfas_mass(&self) -> bool;

    /// True if this substrate introduces nutrient mass.
    fn introduces_nutrient_mass(&self) -> bool;

    /// Node-level compatibility (no back-leach, corridors respected).
    fn node_compatible(&self) -> bool {
        self.corridor_ok()
            && !self.introduces_pfas_mass()
            && !self.introduces_nutrient_mass()
    }

    /// Deployment eligibility based on KER thresholds.
    fn deployment_allowed(&self, weights: &[f64; 5]) -> bool {
        if !self.node_compatible() {
            return false;
        }
        let ker = self.ker(weights);
        ker.meets_deployment_criteria()
    }

    /// Computes KER triad.
    fn ker(&self, weights: &[f64; 5]) -> KerTriad {
        let risks = MaterialRisks::from_kinetics(self.kinetics(), self.corridors());
        risks.ker(weights)
    }
}

impl AntSafeSubstrate for SubstrateSpec {
    fn kinetics(&self) -> &MaterialKinetics {
        &self.kinetics
    }

    fn corridors(&self) -> &MaterialCorridors {
        &self.corridors
    }
}

impl CyboNodeCompatible for SubstrateSpec {
    fn introduces_pfas_mass(&self) -> bool {
        self.pfas_back_leach
    }

    fn introduces_nutrient_mass(&self) -> bool {
        self.nutrient_back_leach
    }
}

// ============================================================================
// Material Registry (Fleet Management)
// ============================================================================

/// Errors that can occur during material validation.[file:22]
#[derive(Clone, Debug, PartialEq)]
pub enum MaterialError {
    /// Negative t90 value.
    NegativeT90,
    /// Negative caloric density.
    NegativeCaloricDensity,
    /// Risk coordinate out of [0,1] range.
    RiskOutOfRange(&'static str),
    /// Corridor bounds in invalid order.
    InvalidCorridorOrder,
    /// Duplicate substrate ID.
    DuplicateSubstrateId(String),
    /// Corridor validation failed.
    CorridorValidationFailed,
    /// Node compatibility check failed.
    NodeCompatibilityFailed,
    /// KER threshold not met.
    KerThresholdFailed(KerTriad),
    /// Environment mismatch.
    EnvironmentMismatch {
        /// Expected environment.
        expected: DeploymentEnvironment,
        /// Actual environment on spec.
        actual: DeploymentEnvironment,
    },
    /// Risk validation failed.
    RiskValidationFailed(f64),
    /// Substrate not found.
    SubstrateNotFound(String),
}

impl fmt::Display for MaterialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaterialError::NegativeT90 => write!(f, "t90 cannot be negative"),
            MaterialError::NegativeCaloricDensity => write!(f, "caloric density cannot be negative"),
            MaterialError::RiskOutOfRange(field) => {
                write!(f, "{} risk must be in [0,1] range", field)
            }
            MaterialError::InvalidCorridorOrder => {
                write!(f, "corridor bounds must satisfy safe <= gold <= hard")
            }
            MaterialError::DuplicateSubstrateId(id) => {
                write!(f, "duplicate substrate ID: {}", id)
            }
            MaterialError::CorridorValidationFailed => write!(f, "corridor validation failed"),
            MaterialError::NodeCompatibilityFailed => write!(f, "node compatibility check failed"),
            MaterialError::KerThresholdFailed(ker) => {
                write!(f, "KER threshold not met: {}", ker)
            }
            MaterialError::EnvironmentMismatch { expected, actual } => {
                write!(
                    f,
                    "environment mismatch: expected {:?}, actual {:?}",
                    expected, actual
                )
            }
            MaterialError::RiskValidationFailed(max_risk) => {
                write!(f, "risk validation failed: max risk = {:.4}", max_risk)
            }
            MaterialError::SubstrateNotFound(id) => {
                write!(f, "substrate not found: {}", id)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MaterialError {}

/// Summary of material registry health.[file:22]
#[derive(Clone, Copy, Debug)]
pub struct MaterialRegistrySummary {
    /// Total substrates.
    pub total_substrates: usize,
    /// PFAS-free substrates.
    pub pfas_free_count: usize,
    /// Rapidly biodegradable substrates.
    pub rapidly_biodegradable_count: usize,
    /// Substrates passing corridor checks.
    pub corridor_ok_count: usize,
    /// Substrates passing node compatibility.
    pub node_compatible_count: usize,
    /// Fraction node-compatible.
    pub compliance_rate: f64,
}

impl fmt::Display for MaterialRegistrySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MaterialRegistrySummary[total={}, pfas_free={}, biodegradable={}, compliant={}, rate={:.2}]",
            self.total_substrates,
            self.pfas_free_count,
            self.rapidly_biodegradable_count,
            self.node_compatible_count,
            self.compliance_rate
        )
    }
}

/// Registry for tracking approved substrates in a deployment.[file:22]
#[cfg(feature = "std")]
#[derive(Clone, Debug, Default)]
pub struct MaterialRegistry {
    /// Map of substrate ID to specification.
    substrates: HashMap<String, SubstrateSpec>,
    /// Default corridors for new substrates.
    default_corridors: MaterialCorridors,
}

#[cfg(feature = "std")]
impl MaterialRegistry {
    /// Creates a new registry with default corridors.
    pub fn new() -> Self {
        MaterialRegistry {
            substrates: HashMap::new(),
            default_corridors: MaterialCorridors::default(),
        }
    }

    /// Creates a registry with custom default corridors.
    pub fn with_corridors(corridors: MaterialCorridors) -> Self {
        MaterialRegistry {
            substrates: HashMap::new(),
            default_corridors: corridors,
        }
    }

    /// Registers a new substrate after risk validation.
    pub fn register(&mut self, substrate: SubstrateSpec) -> Result<(), MaterialError> {
        if self.substrates.contains_key(&substrate.id) {
            return Err(MaterialError::DuplicateSubstrateId(substrate.id.clone()));
        }

        let risks = substrate.risks();
        if !risks.all_acceptable() {
            return Err(MaterialError::RiskValidationFailed(
                risks.max_risk().value(),
            ));
        }

        self.substrates.insert(substrate.id.clone(), substrate);
        Ok(())
    }

    /// Gets a substrate by ID.
    pub fn get(&self, id: &str) -> Option<&SubstrateSpec> {
        self.substrates.get(id)
    }

    /// Summary of registry health.
    pub fn health_summary(&self) -> MaterialRegistrySummary {
        let total = self.substrates.len();
        let pfas_free = self.substrates.values().filter(|s| s.is_pfas_free()).count();
        let biodegradable = self
            .substrates
            .values()
            .filter(|s| s.is_rapidly_biodegradable())
            .count();
        let corridor_ok = self.substrates.values().filter(|s| s.corridor_ok()).count();
        let node_compatible = self
            .substrates
            .values()
            .filter(|s| s.node_compatible())
            .count();

        MaterialRegistrySummary {
            total_substrates: total,
            pfas_free_count: pfas_free,
            rapidly_biodegradable_count: biodegradable,
            corridor_ok_count: corridor_ok,
            node_compatible_count: node_compatible,
            compliance_rate: if total == 0 {
                1.0
            } else {
                node_compatible as f64 / total as f64
            },
        }
    }

    /// Exports all substrates as ALN shard strings.
    pub fn export_all_shards(&self) -> Vec<String> {
        self.substrates.values().map(|s| s.export_aln_shard()).collect()
    }
}
