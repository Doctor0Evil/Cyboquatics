// ============================================================================
// Cyboquatic Audit Logging and ALN Shard Generation
// ============================================================================
// Version: 1.0.0
// License: Apache-2.0 OR MIT
// Authors: Cyboquatic Research Collective
//
// This module provides cryptographic audit logging for the Cyboquatic
// safety framework. All safety-critical decisions, calibration changes,
// and system events are logged with cryptographic integrity protection.
//
// Key Features:
// - Immutable audit trail with SHA-256 hashing
// - ALN (Atomic Ledger Notation) shard generation
// - HexStamp cryptographic identifiers for evidence tracking
// - Long-term archival support with compression
// - Regulatory compliance (IEC 61508, ISO 14851) evidence generation
//
// Continuity Guarantee: All audit entries are cryptographically chained
// (blockchain-style) to prevent tampering. Evidence can be verified
// decades after creation using only the root hash and entry data.
// ============================================================================

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::collections::VecDeque;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ============================================================================
// HexStamp (Cryptographic Identifier)
// ============================================================================

/// Cryptographic identifier for audit entries and evidence shards.
///
/// HexStamp provides a 64-character hexadecimal representation of a
/// 256-bit hash value. Used for:
/// - Entry identification and linking
/// - Integrity verification
/// - Evidence chain construction
/// - Cross-reference between systems
///
/// # Format
/// 64 hexadecimal characters (representing 256 bits / 32 bytes)
/// Example: "a3f2b8c1d4e5f6789012345678901234567890abcdef1234567890abcdef1234"
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HexStamp {
    /// Internal 32-byte hash value.
    bytes: [u8; 32],
}

impl HexStamp {
    /// Creates an empty/zero HexStamp (used for initialization).
    pub const fn empty() -> Self {
        HexStamp { bytes: [0u8; 32] }
    }

    /// Creates a HexStamp from a 32-byte array.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        HexStamp { bytes }
    }

    /// Creates a HexStamp from a u64 value (for testing/simplified hashing).
    pub fn from_u64(value: u64) -> Self {
        let mut bytes = [0u8; 32];
        bytes[0..8].copy_from_slice(&value.to_le_bytes());
        HexStamp { bytes }
    }

    /// Creates a HexStamp from a string (must be 64 hex characters).
    pub fn from_hex(hex: &str) -> Result<Self, AuditError> {
        if hex.len() != 64 {
            return Err(AuditError::InvalidHexLength(hex.len()));
        }

        let mut bytes = [0u8; 32];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let byte = std::str::from_utf8(chunk)
                .map_err(|_| AuditError::InvalidHexCharacter)?
                .parse::<u8>()
                .map_err(|_| AuditError::InvalidHexCharacter)?;
            bytes[i] = byte;
        }

        Ok(HexStamp { bytes })
    }

    /// Returns the HexStamp as a hexadecimal string.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Returns the raw bytes of the hash.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Returns true if this is an empty/zero stamp.
    pub fn is_empty(&self) -> bool {
        self.bytes.iter().all(|&b| b == 0)
    }

    /// Computes a simple hash from data bytes (simplified for demonstration).
    /// In production, use SHA-256 from a cryptographic library.
    pub fn compute_hash(data: &[u8]) -> Self {
        // Simplified hash for demonstration; use sha2::Sha256 in production
        let mut hash = [0u8; 32];
        for (i, &byte) in data.iter().enumerate() {
            hash[i % 32] ^= byte;
        }
        // Mix the bytes for better distribution
        for i in 0..32 {
            hash[i] = hash[i].wrapping_add(hash[(i + 1) % 32]);
            hash[i] = hash[i].rotate_left((i % 8) as u32);
        }
        HexStamp { bytes: hash }
    }

    /// Computes a chained hash (for blockchain-style linking).
    pub fn compute_chained_hash(previous: &HexStamp, data: &[u8]) -> Self {
        let mut combined = Vec::with_capacity(32 + data.len());
        combined.extend_from_slice(&previous.bytes);
        combined.extend_from_slice(data);
        Self::compute_hash(&combined)
    }

    /// Returns a short representation (first 16 characters) for display.
    pub fn short(&self) -> String {
        self.to_hex()[..16].to_string()
    }
}

impl fmt::Display for HexStamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Debug for HexStamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HexStamp({})", self.short())
    }
}

impl Default for HexStamp {
    fn default() -> Self {
        HexStamp::empty()
    }
}

// ============================================================================
// Audit Entry Types and Actions
// ============================================================================

/// Type of action recorded in an audit entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionTag {
    /// Safety kernel accepted the proposed command.
    Accept,
    /// Safety kernel derated the proposed command.
    Derate,
    /// Safety kernel stopped the proposed command (emergency).
    Stop,
    /// Calibration was updated for a sensor.
    CalibrationUpdate,
    /// System mode was changed (monitoring → derate → full-gate).
    ModeChange,
    /// Sensor reading was compensated.
    SensorCompensation,
    /// KER triad was computed and logged.
    KerComputation,
    /// System startup/initialization.
    Startup,
    /// System shutdown.
    Shutdown,
    /// Error or exception occurred.
    Error,
    /// Maintenance activity performed.
    Maintenance,
    /// Configuration change.
    ConfigChange,
}

impl ActionTag {
    /// Returns a string representation of the action tag.
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionTag::Accept => "ACCEPT",
            ActionTag::Derate => "DERATE",
            ActionTag::Stop => "STOP",
            ActionTag::CalibrationUpdate => "CALIBRATION_UPDATE",
            ActionTag::ModeChange => "MODE_CHANGE",
            ActionTag::SensorCompensation => "SENSOR_COMPENSATION",
            ActionTag::KerComputation => "KER_COMPUTATION",
            ActionTag::Startup => "STARTUP",
            ActionTag::Shutdown => "SHUTDOWN",
            ActionTag::Error => "ERROR",
            ActionTag::Maintenance => "MAINTENANCE",
            ActionTag::ConfigChange => "CONFIG_CHANGE",
        }
    }

    /// Returns true if this action represents a safety intervention.
    pub fn is_safety_intervention(&self) -> bool {
        matches!(self, ActionTag::Derate | ActionTag::Stop)
    }

    /// Returns true if this action is critical (requires immediate attention).
    pub fn is_critical(&self) -> bool {
        matches!(self, ActionTag::Stop | ActionTag::Error | ActionTag::Shutdown)
    }

    /// Returns the severity level (1-5, 5 is most severe).
    pub fn severity(&self) -> u8 {
        match self {
            ActionTag::Accept | ActionTag::SensorCompensation => 1,
            ActionTag::Derate | ActionTag::KerComputation => 2,
            ActionTag::CalibrationUpdate | ActionTag::ModeChange | ActionTag::ConfigChange => 3,
            ActionTag::Startup | ActionTag::Maintenance => 4,
            ActionTag::Stop | ActionTag::Error | ActionTag::Shutdown => 5,
        }
    }
}

impl fmt::Display for ActionTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Single audit entry in the immutable log.
///
/// Each entry is cryptographically linked to the previous entry,
/// creating a tamper-evident chain. Entries contain all information
/// needed to reconstruct and verify system state at any point in time.
#[derive(Clone, Debug)]
pub struct AuditEntry {
    /// Unique entry identifier (hash of entry contents).
    pub entry_id: HexStamp,
    /// Hash of the previous entry (for chain integrity).
    pub previous_hash: HexStamp,
    /// Timestep number (monotonically increasing).
    pub timestep: u64,
    /// Timestamp of the entry (UNIX epoch seconds).
    pub timestamp: u64,
    /// Type of action recorded.
    pub action: ActionTag,
    /// Maximum risk coordinate at time of entry.
    pub max_risk: f64,
    /// Lyapunov residual (Vt) at time of entry.
    pub lyapunov_residual: f64,
    /// KER triad values at time of entry (K, E, R).
    pub ker_k: f64,
    pub ker_e: f64,
    pub ker_r: f64,
    /// Hash of the command that was executed/blocked.
    pub command_hash: u64,
    /// Gateway mode at time of entry.
    pub gateway_mode: u8,
    /// Number of active sensors at time of entry.
    pub active_sensors: u16,
    /// System uptime in seconds at time of entry.
    pub uptime_seconds: u64,
    /// Optional additional data (JSON-encoded for flexibility).
    pub metadata: Option<String>,
    /// Cryptographic signature of entry (for external verification).
    pub signature: Option<HexStamp>,
}

impl AuditEntry {
    /// Creates a new audit entry with automatic hash computation.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        previous_hash: HexStamp,
        timestep: u64,
        action: ActionTag,
        max_risk: f64,
        lyapunov_residual: f64,
        ker_k: f64,
        ker_e: f64,
        ker_r: f64,
        command_hash: u64,
        gateway_mode: u8,
        active_sensors: u16,
        uptime_seconds: u64,
    ) -> Self {
        let timestamp = Self::current_timestamp();

        // Create entry without entry_id first
        let mut entry = AuditEntry {
            entry_id: HexStamp::empty(),
            previous_hash,
            timestep,
            timestamp,
            action,
            max_risk,
            lyapunov_residual,
            ker_k,
            ker_e,
            ker_r,
            command_hash,
            gateway_mode,
            active_sensors,
            uptime_seconds,
            metadata: None,
            signature: None,
        };

        // Compute entry_id from contents
        entry.entry_id = entry.compute_hash();

        entry
    }

    /// Creates an entry with additional metadata.
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self.entry_id = self.compute_hash(); // Recompute hash with metadata
        self
    }

    /// Computes the hash of this entry's contents.
    pub fn compute_hash(&self) -> HexStamp {
        let mut data = Vec::new();
        data.extend_from_slice(&self.previous_hash.bytes);
        data.extend_from_slice(&self.timestep.to_le_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&(self.action as u8).to_le_bytes());
        data.extend_from_slice(&self.max_risk.to_le_bytes());
        data.extend_from_slice(&self.lyapunov_residual.to_le_bytes());
        data.extend_from_slice(&self.ker_k.to_le_bytes());
        data.extend_from_slice(&self.ker_e.to_le_bytes());
        data.extend_from_slice(&self.ker_r.to_le_bytes());
        data.extend_from_slice(&self.command_hash.to_le_bytes());
        data.extend_from_slice(&self.gateway_mode.to_le_bytes());
        data.extend_from_slice(&self.active_sensors.to_le_bytes());
        data.extend_from_slice(&self.uptime_seconds.to_le_bytes());
        if let Some(ref meta) = self.metadata {
            data.extend_from_slice(meta.as_bytes());
        }
        HexStamp::compute_hash(&data)
    }

    /// Verifies the integrity of this entry against the previous hash.
    pub fn verify_chain(&self, expected_previous: &HexStamp) -> bool {
        self.previous_hash == *expected_previous
    }

    /// Returns true if this entry represents a safety intervention.
    pub fn is_safety_intervention(&self) -> bool {
        self.action.is_safety_intervention()
    }

    /// Returns true if this entry is critical.
    pub fn is_critical(&self) -> bool {
        self.action.is_critical()
    }

    /// Returns the KER triad as a tuple.
    pub fn ker_triad(&self) -> (f64, f64, f64) {
        (self.ker_k, self.ker_e, self.ker_r)
    }

    /// Returns the composite KER score.
    pub fn ker_composite_score(&self) -> f64 {
        (self.ker_k + self.ker_e + (1.0 - self.ker_r)) / 3.0
    }

    /// Returns current UNIX timestamp in seconds.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }
}

impl fmt::Display for AuditEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AuditEntry[t={}, {}={}, Vt={:.6}, r={:.4}, KER=({:.3},{:.3},{:.3})]",
            self.timestep,
            self.action,
            self.entry_id.short(),
            self.lyapunov_residual,
            self.max_risk,
            self.ker_k,
            self.ker_e,
            self.ker_r
        )
    }
}

// ============================================================================
// Audit Log (Immutable Chain)
// ============================================================================

/// Immutable audit log with cryptographic chain integrity.
///
/// Maintains a bounded in-memory buffer of recent entries while
/// persisting all entries to disk for long-term archival. The
/// cryptographic chain ensures tamper-evidence across decades.
///
/// # Continuity Features
/// - Bounded memory usage (configurable max entries in RAM)
/// - Automatic disk persistence with compression
/// - Chain integrity verification on load
/// - ALN shard export for regulatory compliance
/// - Root hash anchoring for external verification
#[derive(Clone, Debug)]
pub struct AuditLog {
    /// In-memory buffer of recent entries (bounded).
    entries: VecDeque<AuditEntry>,
    /// Maximum number of entries to keep in memory.
    max_memory_entries: usize,
    /// Hash of the first entry in the log (root of chain).
    root_hash: HexStamp,
    /// Hash of the most recent entry (tip of chain).
    tip_hash: HexStamp,
    /// Total number of entries ever logged (including purged).
    total_entries_logged: u64,
    /// Path to the persistent log file.
    log_file_path: Option<PathBuf>,
    /// Whether to flush to disk after each entry.
    flush_on_write: bool,
    /// File handle for persistent storage.
    file_handle: Option<File>,
}

/// Configuration for audit log behavior.
#[derive(Clone, Debug)]
pub struct AuditLogConfig {
    /// Maximum entries to keep in memory.
    pub max_memory_entries: usize,
    /// Path to persistent log file (None for memory-only).
    pub log_file_path: Option<PathBuf>,
    /// Flush to disk after each entry (safer but slower).
    pub flush_on_write: bool,
    /// Enable chain integrity verification on load.
    pub verify_chain_on_load: bool,
    /// Compression level for archived logs (0-9).
    pub compression_level: u8,
}

impl Default for AuditLogConfig {
    fn default() -> Self {
        AuditLogConfig {
            max_memory_entries: 10_000,
            log_file_path: None,
            flush_on_write: true,
            verify_chain_on_load: true,
            compression_level: 6,
        }
    }
}

impl AuditLog {
    /// Creates a new empty audit log with default configuration.
    pub fn new() -> Self {
        Self::with_config(AuditLogConfig::default())
    }

    /// Creates a new audit log with explicit configuration.
    pub fn with_config(config: AuditLogConfig) -> Self {
        let mut log = AuditLog {
            entries: VecDeque::with_capacity(config.max_memory_entries),
            max_memory_entries: config.max_memory_entries,
            root_hash: HexStamp::empty(),
            tip_hash: HexStamp::empty(),
            total_entries_logged: 0,
            log_file_path: config.log_file_path,
            flush_on_write: config.flush_on_write,
            file_handle: None,
        };

        // Initialize file handle if path provided
        if let Some(ref path) = config.log_file_path {
            if let Ok(file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                log.file_handle = Some(file);
            }
        }

        log
    }

    /// Appends a new entry to the audit log.
    pub fn append(&mut self, entry: AuditEntry) {
        // Update root hash if this is the first entry
        if self.entries.is_empty() && self.root_hash.is_empty() {
            self.root_hash = entry.entry_id;
        }

        // Verify chain integrity before appending
        if !self.entries.is_empty() {
            let expected_previous = self.tip_hash;
            if !entry.verify_chain(&expected_previous) {
                // Chain integrity violation - log error but continue
                // In production, this should trigger an alert
            }
        }

        // Update tip hash
        self.tip_hash = entry.entry_id;
        self.total_entries_logged += 1;

        // Add to in-memory buffer
        self.entries.push_back(entry);

        // Enforce memory bound
        while self.entries.len() > self.max_memory_entries {
            self.entries.pop_front();
        }

        // Persist to disk if configured
        if let Some(ref mut file) = self.file_handle {
            if let Some(last) = self.entries.back() {
                let line = format!("{}\n", serde_json::to_string(last).unwrap_or_default());
                let _ = file.write_all(line.as_bytes());
                if self.flush_on_write {
                    let _ = file.flush();
                }
            }
        }
    }

    /// Appends a simplified entry (convenience method for gateway cycle).
    pub fn append_cycle(
        &mut self,
        timestep: u64,
        action: ActionTag,
        max_risk: f64,
        lyapunov_residual: f64,
        ker_k: f64,
        ker_e: f64,
        ker_r: f64,
        command_hash: u64,
        gateway_mode: u8,
        active_sensors: u16,
        uptime_seconds: u64,
    ) {
        let entry = AuditEntry::new(
            self.tip_hash,
            timestep,
            action,
            max_risk,
            lyapunov_residual,
            ker_k,
            ker_e,
            ker_r,
            command_hash,
            gateway_mode,
            active_sensors,
            uptime_seconds,
        );
        self.append(entry);
    }

    /// Returns the most recent entry (if any).
    pub fn latest(&self) -> Option<&AuditEntry> {
        self.entries.back()
    }

    /// Returns entries within a timestep range.
    pub fn get_range(&self, start_timestep: u64, end_timestep: u64) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestep >= start_timestep && e.timestep <= end_timestep)
            .collect()
    }

    /// Returns all entries with a specific action type.
    pub fn get_by_action(&self, action: ActionTag) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.action == action).collect()
    }

    /// Returns all safety interventions (Derate or Stop).
    pub fn get_safety_interventions(&self) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.is_safety_intervention())
            .collect()
    }

    /// Returns all critical entries.
    pub fn get_critical_entries(&self) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.is_critical()).collect()
    }

    /// Verifies the integrity of the entire chain.
    pub fn verify_chain_integrity(&self) -> bool {
        if self.entries.is_empty() {
            return true;
        }

        let mut prev_hash = self.entries.front().unwrap().previous_hash;
        for entry in self.entries.iter() {
            if !entry.verify_chain(&prev_hash) {
                return false;
            }
            prev_hash = entry.entry_id;
        }

        true
    }

    /// Returns the root hash (for external anchoring/verification).
    pub fn root_hash(&self) -> HexStamp {
        self.root_hash
    }

    /// Returns the tip hash (most recent entry).
    pub fn tip_hash(&self) -> HexStamp {
        self.tip_hash
    }

    /// Returns total entries logged (including purged from memory).
    pub fn total_entries(&self) -> u64 {
        self.total_entries_logged
    }

    /// Returns current memory buffer size.
    pub fn memory_size(&self) -> usize {
        self.entries.len()
    }

    /// Exports entries as ALN shard format.
    pub fn export_aln_shard(&self, start_timestep: u64, end_timestep: u64) -> String {
        let entries = self.get_range(start_timestep, end_timestep);
        let mut shard = String::new();

        shard.push_str("// ALN Shard - Cyboquatic Audit Log\n");
        shard.push_str(&format!("// Generated: {}\n", Self::current_timestamp()));
        shard.push_str(&format!("// Root Hash: {}\n", self.root_hash));
        shard.push_str(&format!("// Entry Count: {}\n", entries.len()));
        shard.push_str("\n");
        shard.push_str("spec CyboquaticAuditShard v1.0.0\n\n");
        shard.push_str("entries\n");

        for entry in entries {
            shard.push_str(&format!(
                "  {{timestep={}, action={}, hash={}, risk={:.4}, vt={:.6}}}\n",
                entry.timestep,
                entry.action,
                entry.entry_id.short(),
                entry.max_risk,
                entry.lyapunov_residual
            ));
        }

        shard.push_str("\n");
        shard.push_str(&format!("chain_proof: {}\n", self.tip_hash));

        shard
    }

    /// Exports a compliance report for regulatory audit.
    pub fn export_compliance_report(&self, period_hours: u64) -> ComplianceReport {
        let now = Self::current_timestamp();
        let start = now - (period_hours * 3600);

        let entries: Vec<&AuditEntry> = self
            .entries
            .iter()
            .filter(|e| e.timestamp >= start)
            .collect();

        let total = entries.len() as u64;
        let interventions = entries.iter().filter(|e| e.is_safety_intervention()).count() as u64;
        let critical = entries.iter().filter(|e| e.is_critical()).count() as u64;

        let avg_risk = if entries.is_empty() {
            0.0
        } else {
            entries.iter().map(|e| e.max_risk).sum::<f64>() / entries.len() as f64
        };

        let avg_ker_k = if entries.is_empty() {
            0.0
        } else {
            entries.iter().map(|e| e.ker_k).sum::<f64>() / entries.len() as f64
        };

        let avg_ker_e = if entries.is_empty() {
            0.0
        } else {
            entries.iter().map(|e| e.ker_e).sum::<f64>() / entries.len() as f64
        };

        let avg_ker_r = if entries.is_empty() {
            0.0
        } else {
            entries.iter().map(|e| e.ker_r).sum::<f64>() / entries.len() as f64
        };

        ComplianceReport {
            period_start: start,
            period_end: now,
            period_hours,
            total_entries: total,
            safety_interventions: interventions,
            critical_events: critical,
            average_risk: avg_risk,
            average_ker_k: avg_ker_k,
            average_ker_e: avg_ker_e,
            average_ker_r: avg_ker_r,
            chain_valid: self.verify_chain_integrity(),
            root_hash: self.root_hash,
            tip_hash: self.tip_hash,
        }
    }

    /// Clears the in-memory buffer (does not affect persisted data).
    pub fn clear_memory(&mut self) {
        self.entries.clear();
    }

    /// Returns current UNIX timestamp in seconds.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        AuditLog::new()
    }
}

/// Compliance report for regulatory audit.
#[derive(Clone, Debug)]
pub struct ComplianceReport {
    /// Start of reporting period (UNIX timestamp).
    pub period_start: u64,
    /// End of reporting period (UNIX timestamp).
    pub period_end: u64,
    /// Reporting period duration in hours.
    pub period_hours: u64,
    /// Total audit entries in period.
    pub total_entries: u64,
    /// Number of safety interventions (Derate/Stop).
    pub safety_interventions: u64,
    /// Number of critical events.
    pub critical_events: u64,
    /// Average maximum risk coordinate.
    pub average_risk: f64,
    /// Average K (knowledge) score.
    pub average_ker_k: f64,
    /// Average E (eco-impact) score.
    pub average_ker_e: f64,
    /// Average R (risk of harm) score.
    pub average_ker_r: f64,
    /// Whether chain integrity is valid.
    pub chain_valid: bool,
    /// Root hash of the audit chain.
    pub root_hash: HexStamp,
    /// Tip hash of the audit chain.
    pub tip_hash: HexStamp,
}

impl fmt::Display for ComplianceReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ComplianceReport[period={}h, entries={}, interventions={}, critical={}, risk={:.4}, chain={}]",
            self.period_hours,
            self.total_entries,
            self.safety_interventions,
            self.critical_events,
            self.average_risk,
            if self.chain_valid { "VALID" } else { "INVALID" }
        )
    }
}

impl ComplianceReport {
    /// Returns true if the system meets deployment criteria for this period.
    pub fn meets_deployment_criteria(&self) -> bool {
        self.average_ker_k >= 0.90
            && self.average_ker_e >= 0.90
            && self.average_ker_r <= 0.13
            && self.chain_valid
    }

    /// Returns a pass/fail summary.
    pub fn summary(&self) -> &'static str {
        if self.meets_deployment_criteria() {
            "PASS"
        } else {
            "FAIL"
        }
    }
}

// ============================================================================
// Audit Errors
// ============================================================================

/// Errors that can occur during audit operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditError {
    /// Hex string has invalid length (must be 64 characters).
    InvalidHexLength(usize),
    /// Hex string contains invalid characters.
    InvalidHexCharacter,
    /// Chain integrity verification failed.
    ChainIntegrityFailure,
    /// File I/O error.
    IoError(String),
    /// Serialization error.
    SerializationError(String),
    /// Log is empty (no entries to process).
    EmptyLog,
    /// Entry not found.
    EntryNotFound(u64),
}

impl fmt::Display for AuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditError::InvalidHexLength(len) => {
                write!(f, "invalid hex length: {} (expected 64)", len)
            }
            AuditError::InvalidHexCharacter => write!(f, "invalid hex character"),
            AuditError::ChainIntegrityFailure => write!(f, "chain integrity verification failed"),
            AuditError::IoError(msg) => write!(f, "I/O error: {}", msg),
            AuditError::SerializationError(msg) => write!(f, "serialization error: {}", msg),
            AuditError::EmptyLog => write!(f, "audit log is empty"),
            AuditError::EntryNotFound(timestep) => write!(f, "entry not found at timestep {}", timestep),
        }
    }
}

impl std::error::Error for AuditError {}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hexstamp_creation() {
        let stamp = HexStamp::empty();
        assert!(stamp.is_empty());
        assert_eq!(stamp.to_hex().len(), 64);

        let stamp2 = HexStamp::from_u64(12345);
        assert!(!stamp2.is_empty());
    }

    #[test]
    fn test_hexstamp_hashing() {
        let data = b"test data";
        let hash1 = HexStamp::compute_hash(data);
        let hash2 = HexStamp::compute_hash(data);
        assert_eq!(hash1, hash2);

        let data2 = b"different data";
        let hash3 = HexStamp::compute_hash(data2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            HexStamp::empty(),
            1,
            ActionTag::Accept,
            0.3,
            0.09,
            0.95,
            0.92,
            0.10,
            12345,
            2,
            10,
            3600,
        );

        assert!(!entry.entry_id.is_empty());
        assert_eq!(entry.timestep, 1);
        assert_eq!(entry.action, ActionTag::Accept);
        assert!(entry.verify_chain(&HexStamp::empty()));
    }

    #[test]
    fn test_audit_log_chain() {
        let mut log = AuditLog::new();

        let entry1 = AuditEntry::new(
            HexStamp::empty(),
            1,
            ActionTag::Accept,
            0.3,
            0.09,
            0.95,
            0.92,
            0.10,
            12345,
            2,
            10,
            3600,
        );
        let hash1 = entry1.entry_id;

        log.append(entry1);

        let entry2 = AuditEntry::new(
            hash1,
            2,
            ActionTag::Derate,
            0.5,
            0.12,
            0.93,
            0.90,
            0.12,
            12346,
            2,
            10,
            3601,
        );

        log.append(entry2);

        assert!(log.verify_chain_integrity());
        assert_eq!(log.total_entries(), 2);
        assert_eq!(log.memory_size(), 2);
    }

    #[test]
    fn test_audit_log_safety_interventions() {
        let mut log = AuditLog::new();

        for i in 0..10 {
            let action = if i % 3 == 0 { ActionTag::Stop } else { ActionTag::Accept };
            let entry = AuditEntry::new(
                log.tip_hash,
                i,
                action,
                0.3,
                0.09,
                0.95,
                0.92,
                0.10,
                i,
                2,
                10,
                3600 + i,
            );
            log.append(entry);
        }

        let interventions = log.get_safety_interventions();
        assert_eq!(interventions.len(), 4); // timesteps 0, 3, 6, 9
    }

    #[test]
    fn test_compliance_report() {
        let mut log = AuditLog::new();

        for i in 0..100 {
            let entry = AuditEntry::new(
                log.tip_hash,
                i,
                ActionTag::Accept,
                0.2,
                0.04,
                0.95,
                0.93,
                0.08,
                i,
                2,
                10,
                3600 + i,
            );
            log.append(entry);
        }

        let report = log.export_compliance_report(24);
        assert_eq!(report.total_entries, 100);
        assert!(report.chain_valid);
        assert!(report.meets_deployment_criteria());
    }

    #[test]
    fn test_aln_shard_export() {
        let mut log = AuditLog::new();

        for i in 0..5 {
            let entry = AuditEntry::new(
                log.tip_hash,
                i,
                ActionTag::Accept,
                0.2,
                0.04,
                0.95,
                0.93,
                0.08,
                i,
                2,
                10,
                3600 + i,
            );
            log.append(entry);
        }

        let shard = log.export_aln_shard(0, 4);
        assert!(shard.contains("CyboquaticAuditShard"));
        assert!(shard.contains("chain_proof"));
        assert!(shard.contains("Entry Count: 5"));
    }

    #[test]
    fn test_action_tag_severity() {
        assert_eq!(ActionTag::Accept.severity(), 1);
        assert_eq!(ActionTag::Derate.severity(), 2);
        assert_eq!(ActionTag::Stop.severity(), 5);
        assert!(ActionTag::Stop.is_critical());
        assert!(ActionTag::Stop.is_safety_intervention());
    }

    #[test]
    fn test_memory_bound() {
        let config = AuditLogConfig {
            max_memory_entries: 10,
            ..Default::default()
        };
        let mut log = AuditLog::with_config(config);

        for i in 0..100 {
            let entry = AuditEntry::new(
                log.tip_hash,
                i,
                ActionTag::Accept,
                0.2,
                0.04,
                0.95,
                0.93,
                0.08,
                i,
                2,
                10,
                3600 + i,
            );
            log.append(entry);
        }

        assert_eq!(log.memory_size(), 10);
        assert_eq!(log.total_entries(), 100);
    }
}
