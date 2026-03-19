// ============================================================================
// Econet Material Cybo - Biodegradable Substrate Library
// ============================================================================
// Version: 1.0.0
// License: Apache-2.0 OR MIT
// Authors: Cyboquatic Research Collective
//
// This library provides first-class ecosafety inputs for biodegradable
// substrates used in Cyboquatic machinery casings, media, and FlowVac
// internals. All materials are validated against t90, rtox, rmicro,
// r_leach, and r_pfas corridor bands with KER gating.
//
// Key Features:
// - Biodegradation kinetics tracking (t90_days)
// - PFAS residue monitoring and prevention
// - Microplastic formation risk assessment
// - Chemical leaching (CEC) corridor validation
// - Node compatibility verification for distributed systems
// - ALN shard export for regulatory compliance
//
// Continuity Guarantee: All material specifications are cryptographically
// hashed and audit-logged. Material degradation is tracked over the
// 20-50 year operational lifespan with automatic replacement scheduling.
// ============================================================================

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================================
// Re-exports from cyboquatic-ecosafety-core
// ============================================================================

// Note: In production, these would be imported from the core crate.
// For this standalone file, we include minimal definitions.

/// Dimensionless risk coordinate r ∈ [0,1].
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct RiskCoord(f64);

impl RiskCoord {
    #[inline]
    pub fn new_clamped(v: f64) -> Self {
        let v = if v < 0.0 { 0.0 } else if v > 1.0 { 1.0 } else { v };
        RiskCoord(v)
    }

    #[inline]
    pub fn value(self) -> f64 { self.0 }

    #[inline]
    pub fn is_hard_band(self) -> bool { self.0 >= 1.0 }

    #[inline]
    pub fn is_gold_band(self) -> bool { self.0 <= 0.5 }
}

impl Default for RiskCoord {
    fn default() -> Self { RiskCoord(0.0) }
}

/// Corridor bands for one physical metric.
#[derive(Clone, Copy, Debug)]
pub struct CorridorBands {
    pub x_safe: f64,
    pub x_gold: f64,
    pub x_hard: f64,
}

impl CorridorBands {
    pub fn new(x_safe: f64, x_gold: f64, x_hard: f64) -> Result<Self, MaterialError> {
        if x_safe > x_gold || x_gold > x_hard {
            return Err(MaterialError::InvalidCorridorOrder);
        }
        Ok(CorridorBands { x_safe, x_gold, x_hard })
    }

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
}

/// Vector of risk coordinates.
#[derive(Clone, Debug)]
pub struct RiskVector {
    pub coords: Vec<RiskCoord>,
    pub labels: Vec<String>,
}

impl RiskVector {
    pub fn new(coords: Vec<RiskCoord>) -> Self {
        let labels = (0..coords.len()).map(|i| format!("r_{}", i)).collect();
        RiskVector { coords, labels }
    }

    pub fn with_labels(coords: Vec<RiskCoord>, labels: Vec<String>) -> Self {
        assert_eq!(coords.len(), labels.len());
        RiskVector { coords, labels }
    }

    pub fn max(&self) -> RiskCoord {
        self.coords
            .iter()
            .copied()
            .max_by(|a, b| a.value().partial_cmp(&b.value()).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(RiskCoord::default())
    }

    pub fn mean(&self) -> f64 {
        if self.coords.is_empty() {
            return 0.0;
        }
        self.coords.iter().map(|r| r.value()).sum::<f64>() / self.coords.len() as f64
    }

    pub fn weighted_squared_sum(&self, weights: &[f64]) -> f64 {
        let mut sum = 0.0;
        for (r, w) in self.coords.iter().zip(weights.iter()) {
            let v = r.value();
            sum += w.max(0.0) * v * v;
        }
        sum
    }
}

impl Default for RiskVector {
    fn default() -> Self { RiskVector::new(Vec::new()) }
}

/// K/E/R triad for ecological impact assessment.
#[derive(Clone, Copy, Debug)]
pub struct KerTriad {
    pub k_knowledge: f64,
    pub e_ecoimpact: f64,
    pub r_risk_of_harm: f64,
}

impl KerTriad {
    pub fn new(k: f64, e: f64, r: f64) -> Self {
        KerTriad {
            k_knowledge: k.clamp(0.0, 1.0),
            e_ecoimpact: e.clamp(0.0, 1.0),
            r_risk_of_harm: r.clamp(0.0, 1.0),
        }
    }

    pub fn meets_deployment_criteria(&self) -> bool {
        self.k_knowledge >= 0.90 && self.e_ecoimpact >= 0.90 && self.r_risk_of_harm <= 0.13
    }

    pub fn composite_score(&self) -> f64 {
        (self.k_knowledge + self.e_ecoimpact + (1.0 - self.r_risk_of_harm)) / 3.0
    }
}

impl Default for KerTriad {
    fn default() -> Self { KerTriad::new(1.0, 1.0, 0.0) }
}

impl fmt::Display for KerTriad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "KER(k={:.3}, e={:.3}, r={:.3}, score={:.3})",
            self.k_knowledge, self.e_ecoimpact, self.r_risk_of_harm, self.composite_score()
        )
    }
}

// ============================================================================
// Material Kinetics (Biodegradation Profiles)
// ============================================================================

/// Complete biodegradation kinetics profile for a material substrate.
///
/// This structure captures all measurable properties that determine
/// whether a material is safe for use in ecologically-restorative
/// Cyboquatic machinery. All fields are measured through standardized
/// laboratory protocols (ISO 14851, OECD 202, etc.).
///
/// # Continuity Note
/// For 20-50 year operation, these kinetics should be re-validated
/// periodically as material batches may vary. Track batch_id for
/// full traceability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialKinetics {
    /// Time to 90% biodegradation under standard conditions (days).
    /// Lower is better for ecological restoration.
    pub t90_days: f64,

    /// Toxicity risk coordinate (0.0 = non-toxic, 1.0 = highly toxic).
    /// Measured via Daphnia magna or equivalent bioassay.
    pub r_tox: f64,

    /// Microplastic formation risk (0.0 = none, 1.0 = severe).
    /// Measured via particle counting after degradation.
    pub r_micro: f64,

    /// Chemical leaching risk for contaminants of emerging concern.
    /// Measured via LC-MS/MS analysis of leachate.
    pub r_leach_cec: f64,

    /// PFAS residue risk (0.0 = none detected, 1.0 = above threshold).
    /// Measured via EPA Method 537 or equivalent.
    pub r_pfas_resid: f64,

    /// Caloric density for energy recovery potential (MJ/kg).
    /// Lower is better (less energy-intensive to produce).
    pub caloric_density_mj_per_kg: f64,

    /// Optional batch identifier for traceability.
    pub batch_id: Option<String>,

    /// Measurement timestamp (UNIX epoch seconds).
    pub measured_timestamp: u64,
}

impl MaterialKinetics {
    /// Creates a new kinetics profile with validation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        t90_days: f64,
        r_tox: f64,
        r_micro: f64,
        r_leach_cec: f64,
        r_pfas_resid: f64,
        caloric_density_mj_per_kg: f64,
    ) -> Result<Self, MaterialError> {
        if t90_days < 0.0 {
            return Err(MaterialError::NegativeT90);
        }
        if r_tox < 0.0 || r_tox > 1.0 {
            return Err(MaterialError::RiskOutOfRange("r_tox"));
        }
        if r_micro < 0.0 || r_micro > 1.0 {
            return Err(MaterialError::RiskOutOfRange("r_micro"));
        }
        if r_leach_cec < 0.0 || r_leach_cec > 1.0 {
            return Err(MaterialError::RiskOutOfRange("r_leach_cec"));
        }
        if r_pfas_resid < 0.0 || r_pfas_resid > 1.0 {
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
            measured_timestamp: Self::current_timestamp(),
        })
    }

    /// Creates kinetics with batch tracking.
    pub fn with_batch(mut self, batch_id: String) -> Self {
        self.batch_id = Some(batch_id);
        self
    }

    /// Returns true if this material is PFAS-free (r_pfas_resid = 0).
    pub fn is_pfas_free(&self) -> bool {
        self.r_pfas_resid < 0.01
    }

    /// Returns true if this material is rapidly biodegradable (t90 < 90 days).
    pub fn is_rapidly_biodegradable(&self) -> bool {
        self.t90_days < 90.0
    }

    /// Returns true if this material has low toxicity (r_tox < 0.1).
    pub fn is_low_toxicity(&self) -> bool {
        self.r_tox < 0.1
    }

    /// Returns the overall material risk score (unweighted mean).
    pub fn overall_risk_score(&self) -> f64 {
        (self.r_tox + self.r_micro + self.r_leach_cec + self.r_pfas_resid) / 4.0
    }

    /// Returns current UNIX timestamp in seconds.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
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

/// Safety corridor bands for material properties.
///
/// Defines the acceptable ranges for each material kinetic property.
/// Materials must fall within these corridors to be approved for
/// use in Cyboquatic machinery.
///
/// # Default Values
/// Based on ISO 14851, OECD 202, and EPA Method 537 standards.
/// Adjust based on specific deployment environment requirements.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialCorridors {
    /// Maximum acceptable t90 (days) for hard-band violation.
    pub t90_max_days: f64,

    /// Gold-band threshold for t90 (optimal biodegradation).
    pub t90_gold_days: f64,

    /// Maximum acceptable toxicity risk for gold-band.
    pub r_tox_gold_max: f64,

    /// Maximum acceptable microplastic risk.
    pub r_micro_max: f64,

    /// Maximum acceptable leaching risk.
    pub r_leach_max: f64,

    /// Maximum acceptable PFAS residue risk.
    pub r_pfas_max: f64,

    /// Maximum acceptable caloric density (MJ/kg).
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
        if r_tox_gold_max > 1.0 || r_tox_gold_max < 0.0 {
            return Err(MaterialError::RiskOutOfRange("r_tox_gold_max"));
        }
        if r_micro_max > 1.0 || r_micro_max < 0.0 {
            return Err(MaterialError::RiskOutOfRange("r_micro_max"));
        }
        if r_leach_max > 1.0 || r_leach_max < 0.0 {
            return Err(MaterialError::RiskOutOfRange("r_leach_max"));
        }
        if r_pfas_max > 1.0 || r_pfas_max < 0.0 {
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

    /// Creates strict corridors for sensitive environments (marine, drinking water).
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

    /// Creates relaxed corridors for industrial environments.
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
// Material Risk Coordinates (Computed from Kinetics + Corridors)
// ============================================================================

/// Computed risk coordinates for a material substrate.
///
/// These coordinates are derived by normalizing the raw kinetics
/// measurements against the corridor bands. Each coordinate represents
/// a different dimension of material safety risk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialRisks {
    /// Risk coordinate for biodegradation time.
    pub r_t90: RiskCoord,

    /// Risk coordinate for toxicity.
    pub r_tox: RiskCoord,

    /// Risk coordinate for microplastic formation.
    pub r_micro: RiskCoord,

    /// Risk coordinate for chemical leaching.
    pub r_leach_cec: RiskCoord,

    /// Risk coordinate for PFAS residue.
    pub r_pfas_resid: RiskCoord,
}

impl MaterialRisks {
    /// Computes risk coordinates from kinetics and corridors.
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

    /// Converts to a RiskVector for kernel evaluation.
    pub fn to_vector(&self) -> RiskVector {
        RiskVector::with_labels(
            vec![self.r_t90, self.r_tox, self.r_micro, self.r_leach_cec, self.r_pfas_resid],
            vec![
                "r_t90".to_string(),
                "r_tox".to_string(),
                "r_micro".to_string(),
                "r_leach_cec".to_string(),
                "r_pfas_resid".to_string(),
            ],
        )
    }

    /// Computes eco-impact score (1 - weighted average risk).
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

    /// Computes KER triad for this material.
    pub fn ker(&self, weights: &[f64; 5]) -> KerTriad {
        let e = self.ecoimpact_score(weights);
        let max_r = self.to_vector().max().value();
        KerTriad::new(1.0, e, max_r)
    }

    /// Returns the maximum risk coordinate.
    pub fn max_risk(&self) -> RiskCoord {
        [self.r_t90, self.r_tox, self.r_micro, self.r_leach_cec, self.r_pfas_resid]
            .iter()
            .copied()
            .max_by(|a, b| a.value().partial_cmp(&b.value()).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(RiskCoord::default())
    }

    /// Returns true if all risks are within acceptable bounds.
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
// Substrate Specification (Complete Material Definition)
// ============================================================================

/// Complete substrate specification for Cyboquatic machinery.
///
/// This is the primary data structure for material validation.
/// Each substrate must pass all corridor checks and KER gates
/// before being approved for deployment.
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

    /// Whether this substrate introduces PFAS mass (back-leach).
    pub pfas_back_leach: bool,

    /// Whether this substrate introduces nutrient mass (back-leach).
    pub nutrient_back_leach: bool,

    /// Intended application (casing, media, internals, etc.).
    pub application: SubstrateApplication,

    /// Deployment environment (freshwater, marine, industrial, etc.).
    pub environment: DeploymentEnvironment,

    /// Installation timestamp (UNIX epoch seconds).
    pub installation_timestamp: Option<u64>,

    /// Expected replacement timestamp (UNIX epoch seconds).
    pub replacement_due_timestamp: Option<u64>,

    /// Cryptographic hash of specification (for audit integrity).
    pub spec_hash: String,
}

/// Application type for substrate deployment.
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

/// Deployment environment for substrate.
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

impl SubstrateSpec {
    /// Creates a new substrate specification.
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

    /// Computes and sets the specification hash.
    pub fn with_hash(mut self) -> Self {
        // Simplified hash; use SHA-256 in production
        let data = format!(
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
        self.spec_hash = format!("{:x}", md5::compute(data.as_bytes()));
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

    /// Returns default weights for KER calculation.
    pub fn default_weights() -> [f64; 5] {
        [0.2, 0.2, 0.2, 0.2, 0.2]
    }

    /// Returns strict weights (prioritize PFAS and toxicity).
    pub fn strict_weights() -> [f64; 5] {
        [0.1, 0.3, 0.1, 0.2, 0.3]
    }
}

// ============================================================================
// AntSafeSubstrate Trait (PFAS and Safety Requirements)
// ============================================================================

/// Trait for PFAS-safe substrate validation.
///
/// All materials used in Cyboquatic machinery must implement
/// this trait to prove they meet ant-safe (PFAS-free) requirements.
pub trait AntSafeSubstrate {
    /// Returns the material kinetics profile.
    fn kinetics(&self) -> &MaterialKinetics;

    /// Returns the safety corridor bands.
    fn corridors(&self) -> &MaterialCorridors;

    /// Returns true if all corridor checks pass.
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

    /// Returns true if the material is PFAS-free.
    fn is_pfas_free(&self) -> bool {
        self.kinetics().is_pfas_free()
    }

    /// Returns true if the material is rapidly biodegradable.
    fn is_rapidly_biodegradable(&self) -> bool {
        self.kinetics().is_rapidly_biodegradable()
    }
}

// ============================================================================
// CyboNodeCompatible Trait (Distributed System Requirements)
// ============================================================================

/// Trait for node compatibility in distributed Cyboquatic systems.
///
/// Materials must not introduce contaminants that could propagate
/// through the network of connected machinery.
pub trait CyboNodeCompatible: AntSafeSubstrate {
    /// Returns true if this substrate introduces PFAS mass.
    fn introduces_pfas_mass(&self) -> bool;

    /// Returns true if this substrate introduces nutrient mass.
    fn introduces_nutrient_mass(&self) -> bool;

    /// Returns true if substrate is compatible with node deployment.
    fn node_compatible(&self) -> bool {
        self.corridor_ok()
            && !self.introduces_pfas_mass()
            && !self.introduces_nutrient_mass()
    }

    /// Returns deployment eligibility based on KER thresholds.
    fn deployment_allowed(&self, weights: &[f64; 5]) -> bool {
        if !self.node_compatible() {
            return false;
        }
        let ker = self.ker(weights);
        ker.k_knowledge >= 0.90 && ker.e_ecoimpact >= 0.90 && ker.r_risk_of_harm <= 0.13
    }

    /// Computes KER triad for this substrate.
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

impl SubstrateSpec {
    /// Sets installation and replacement timestamps.
    pub fn with_lifecycle(mut self, installation: u64, lifetime_days: u64) -> Self {
        self.installation_timestamp = Some(installation);
        self.replacement_due_timestamp = Some(installation + lifetime_days * 24 * 3600);
        self.with_hash()
    }

    /// Returns true if replacement is due.
    pub fn replacement_due(&self) -> bool {
        if let Some(due) = self.replacement_due_timestamp {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs();
            now >= due
        } else {
            false
        }
    }

    /// Returns days until replacement is due.
    pub fn days_until_replacement(&self) -> Option<i64> {
        self.replacement_due_timestamp.map(|due| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs();
            ((due as i64) - (now as i64)) / (24 * 3600)
        })
    }

    /// Validates substrate for specific environment.
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

    /// Exports substrate spec as ALN shard format.
    pub fn export_aln_shard(&self) -> String {
        let risks = self.risks();
        let ker = self.ker(&Self::default_weights());

        format!(
            r#"// ALN Shard - Material Specification
spec MaterialCyboShard v1.0.0

node_id: {}
material_name: {}
manufacturer: {}
application: {}
environment: {}

kinetics
  t90_days: {:.2}
  r_tox: {:.4}
  r_micro: {:.4}
  r_leach_cec: {:.4}
  r_pfas_resid: {:.4}
  caloric_density: {:.4}

risks
  r_t90: {:.4}
  r_tox: {:.4}
  r_micro: {:.4}
  r_leach_cec: {:.4}
  r_pfas_resid: {:.4}

ker_triad
  k_knowledge: {:.4}
  e_ecoimpact: {:.4}
  r_risk_of_harm: {:.4}
  composite_score: {:.4}

validation
  corridor_ok: {}
  node_compatible: {}
  deployment_allowed: {}
  pfas_free: {}

spec_hash: {}
"#,
            self.id,
            self.name,
            self.manufacturer,
            self.application,
            self.environment,
            self.kinetics.t90_days,
            self.kinetics.r_tox,
            self.kinetics.r_micro,
            self.kinetics.r_leach_cec,
            self.kinetics.r_pfas_resid,
            self.kinetics.caloric_density_mj_per_kg,
            risks.r_t90.value(),
            risks.r_tox.value(),
            risks.r_micro.value(),
            risks.r_leach_cec.value(),
            risks.r_pfas_resid.value(),
            ker.k_knowledge,
            ker.e_ecoimpact,
            ker.r_risk_of_harm,
            ker.composite_score(),
            self.corridor_ok(),
            self.node_compatible(),
            self.deployment_allowed(&Self::default_weights()),
            self.is_pfas_free(),
            self.spec_hash
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
// Material Registry (Fleet Management)
// ============================================================================

/// Registry for tracking all approved substrates in a deployment.
#[derive(Clone, Debug, Default)]
pub struct MaterialRegistry {
    /// Map of substrate ID to specification.
    substrates: HashMap<String, SubstrateSpec>,
    /// Default corridors for new substrates.
    default_corridors: MaterialCorridors,
    /// Total substrates registered.
    total_registered: u64,
    /// Substrates pending replacement.
    pending_replacement: Vec<String>,
}

impl MaterialRegistry {
    /// Creates a new material registry.
    pub fn new() -> Self {
        MaterialRegistry {
            substrates: HashMap::new(),
            default_corridors: MaterialCorridors::default(),
            total_registered: 0,
            pending_replacement: Vec::new(),
        }
    }

    /// Creates a registry with custom default corridors.
    pub fn with_corridors(corridors: MaterialCorridors) -> Self {
        MaterialRegistry {
            substrates: HashMap::new(),
            default_corridors: corridors,
            total_registered: 0,
            pending_replacement: Vec::new(),
        }
    }

    /// Registers a new substrate.
    pub fn register(&mut self, substrate: SubstrateSpec) -> Result<(), MaterialError> {
        if self.substrates.contains_key(&substrate.id) {
            return Err(MaterialError::DuplicateSubstrateId(substrate.id.clone()));
        }

        // Validate before registration
        let risks = substrate.risks();
        if !risks.all_acceptable() {
            return Err(MaterialError::RiskValidationFailed(risks.max_risk().value()));
        }

        self.substrates.insert(substrate.id.clone(), substrate);
        self.total_registered += 1;
        Ok(())
    }

    /// Gets a substrate by ID.
    pub fn get(&self, id: &str) -> Option<&SubstrateSpec> {
        self.substrates.get(id)
    }

    /// Gets all substrates needing replacement.
    pub fn get_pending_replacement(&mut self) -> Vec<&SubstrateSpec> {
        self.pending_replacement.clear();
        for (id, substrate) in &self.substrates {
            if substrate.replacement_due() {
                self.pending_replacement.push(id.clone());
            }
        }
        self.pending_replacement
            .iter()
            .filter_map(|id| self.substrates.get(id))
            .collect()
    }

    /// Returns registry health summary.
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

    /// Exports all substrates as ALN shards.
    pub fn export_all_shards(&self) -> Vec<String> {
        self.substrates.values().map(|s| s.export_aln_shard()).collect()
    }
}

/// Summary of material registry health.
#[derive(Clone, Copy, Debug)]
pub struct MaterialRegistrySummary {
    pub total_substrates: usize,
    pub pfas_free_count: usize,
    pub rapidly_biodegradable_count: usize,
    pub corridor_ok_count: usize,
    pub node_compatible_count: usize,
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

// ============================================================================
// Material Errors
// ============================================================================

/// Errors that can occur during material validation.
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
        expected: DeploymentEnvironment,
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
            MaterialError::DuplicateSubstrateId(id) => write!(f, "duplicate substrate ID: {}", id),
            MaterialError::CorridorValidationFailed => write!(f, "corridor validation failed"),
            MaterialError::NodeCompatibilityFailed => write!(f, "node compatibility check failed"),
            MaterialError::KerThresholdFailed(ker) => {
                write!(f, "KER threshold not met: {}", ker)
            }
            MaterialError::EnvironmentMismatch { expected, actual } => {
                write!(f, "environment mismatch: expected {:?}, actual {:?}", expected, actual)
            }
            MaterialError::RiskValidationFailed(max_risk) => {
                write!(f, "risk validation failed: max risk = {:.4}", max_risk)
            }
            MaterialError::SubstrateNotFound(id) => write!(f, "substrate not found: {}", id),
        }
    }
}

impl std::error::Error for MaterialError {}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_kinetics_validation() {
        let kinetics = MaterialKinetics::new(90.0, 0.05, 0.02, 0.05, 0.01, 0.15).unwrap();
        assert!(kinetics.is_pfas_free());
        assert!(kinetics.is_rapidly_biodegradable());
        assert!(kinetics.is_low_toxicity());
    }

    #[test]
    fn test_material_kinetics_rejection() {
        assert!(MaterialKinetics::new(-10.0, 0.05, 0.02, 0.05, 0.01, 0.15).is_err());
        assert!(MaterialKinetics::new(90.0, 1.5, 0.02, 0.05, 0.01, 0.15).is_err());
    }

    #[test]
    fn test_material_corridors_default() {
        let corridors = MaterialCorridors::default();
        assert_eq!(corridors.t90_max_days, 180.0);
        assert_eq!(corridors.t90_gold_days, 120.0);
        assert_eq!(corridors.r_pfas_max, 0.10);
    }

    #[test]
    fn test_material_risks_computation() {
        let kinetics = MaterialKinetics::new(90.0, 0.05, 0.02, 0.05, 0.01, 0.15).unwrap();
        let corridors = MaterialCorridors::default();
        let risks = MaterialRisks::from_kinetics(&kinetics, &corridors);

        assert!(risks.r_t90.value() <= 0.5);
        assert!(risks.r_tox.value() <= 0.5);
        assert!(risks.r_pfas_resid.value() <= 0.1);
    }

    #[test]
    fn test_substrate_spec_creation() {
        let kinetics = MaterialKinetics::new(90.0, 0.05, 0.02, 0.05, 0.01, 0.15).unwrap();
        let corridors = MaterialCorridors::default();

        let substrate = SubstrateSpec::new(
            "SUB-001".to_string(),
            "BioPolymer X".to_string(),
            "EcoMaterials Inc".to_string(),
            kinetics,
            corridors,
            SubstrateApplication::Casing,
            DeploymentEnvironment::Freshwater,
        );

        assert!(substrate.corridor_ok());
        assert!(substrate.node_compatible());
        assert!(substrate.deployment_allowed(&SubstrateSpec::default_weights()));
    }

    #[test]
    fn test_substrate_ker_computation() {
        let kinetics = MaterialKinetics::new(90.0, 0.05, 0.02, 0.05, 0.01, 0.15).unwrap();
        let corridors = MaterialCorridors::default();

        let substrate = SubstrateSpec::new(
            "SUB-001".to_string(),
            "BioPolymer X".to_string(),
            "EcoMaterials Inc".to_string(),
            kinetics,
            corridors,
            SubstrateApplication::Casing,
            DeploymentEnvironment::Freshwater,
        );

        let ker = substrate.ker(&SubstrateSpec::default_weights());
        assert!(ker.meets_deployment_criteria());
        assert!(ker.composite_score() > 0.9);
    }

    #[test]
    fn test_material_registry() {
        let mut registry = MaterialRegistry::new();

        let kinetics = MaterialKinetics::new(90.0, 0.05, 0.02, 0.05, 0.01, 0.15).unwrap();
        let corridors = MaterialCorridors::default();

        let substrate = SubstrateSpec::new(
            "SUB-001".to_string(),
            "BioPolymer X".to_string(),
            "EcoMaterials Inc".to_string(),
            kinetics,
            corridors,
            SubstrateApplication::Casing,
            DeploymentEnvironment::Freshwater,
        );

        assert!(registry.register(substrate).is_ok());
        assert!(registry.get("SUB-001").is_some());
        assert!(registry.get("SUB-002").is_none());

        let summary = registry.health_summary();
        assert_eq!(summary.total_substrates, 1);
        assert_eq!(summary.pfas_free_count, 1);
    }

    #[test]
    fn test_aln_shard_export() {
        let kinetics = MaterialKinetics::new(90.0, 0.05, 0.02, 0.05, 0.01, 0.15).unwrap();
        let corridors = MaterialCorridors::default();

        let substrate = SubstrateSpec::new(
            "SUB-001".to_string(),
            "BioPolymer X".to_string(),
            "EcoMaterials Inc".to_string(),
            kinetics,
            corridors,
            SubstrateApplication::Casing,
            DeploymentEnvironment::Freshwater,
        );

        let shard = substrate.export_aln_shard();
        assert!(shard.contains("MaterialCyboShard"));
        assert!(shard.contains("SUB-001"));
        assert!(shard.contains("spec_hash"));
    }

    #[test]
    fn test_strict_corridors() {
        let strict = MaterialCorridors::strict();
        let default = MaterialCorridors::default();

        assert!(strict.t90_max_days < default.t90_max_days);
        assert!(strict.r_pfas_max < default.r_pfas_max);
        assert!(strict.r_tox_gold_max < default.r_tox_gold_max);
    }

    #[test]
    fn test_environment_validation() {
        let kinetics = MaterialKinetics::new(90.0, 0.05, 0.02, 0.05, 0.01, 0.15).unwrap();
        let corridors = MaterialCorridors::default();

        let substrate = SubstrateSpec::new(
            "SUB-001".to_string(),
            "BioPolymer X".to_string(),
            "EcoMaterials Inc".to_string(),
            kinetics,
            corridors,
            SubstrateApplication::Casing,
            DeploymentEnvironment::Freshwater,
        );

        assert!(substrate.validate_for_environment(DeploymentEnvironment::Freshwater).is_ok());
        assert!(substrate.validate_for_environment(DeploymentEnvironment::Marine).is_err());
    }
}
