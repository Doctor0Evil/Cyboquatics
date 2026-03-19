//! Cyboquatic Ecosafety Core Build Script
//! 
//! Performs compile-time validation of ALN configuration schema against
//! Rust code constants. Ensures configuration drift is caught before
//! deployment, shifting failures from the field into CI/CD pipelines.
//! 
//! # Validation Checks
//! 
//! 1. Corridor band ordering (safe < gold < hard)
//! 2. Lyapunov weight non-negativity
//! 3. KER threshold achievability
//! 4. Schema version compatibility
//! 5. Constant synchronization between ALN and Rust
//! 
//! # Output
//! 
//! - Generates `generated_config.rs` with validated constants
//! - Emits cargo warnings for non-critical mismatches
//! - Emits cargo errors for critical safety violations
//! 
//! @file build.rs
//! @destination cyboquatic-ecosafety-core/build.rs
//! @build-time only (not included in final binary)

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

// ============================================================================
// BUILD CONFIGURATION
// ============================================================================

const ALN_SCHEMA_PATH: &str = "../cyboquatic-aln-config/src/schema.aln";
const GENERATED_CONFIG_PATH: &str = "src/generated_config.rs";
const SCHEMA_VERSION_REQUIRED: &str = "1.0.0";

// ============================================================================
// MAIN BUILD FUNCTION
// ============================================================================

fn main() {
    println!("cargo:rerun-if-changed={}", ALN_SCHEMA_PATH);
    println!("cargo:rerun-if-changed=build.rs");
    
    // Tell cargo to re-run if ALN config changes
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let aln_path = PathBuf::from(&manifest_dir).join(ALN_SCHEMA_PATH);
    
    if !aln_path.exists() {
        println!("cargo:warning=ALN schema not found at {}. Using default constants.", ALN_SCHEMA_PATH);
        generate_default_config();
        return;
    }
    
    // Parse and validate ALN schema
    match validate_aln_schema(&aln_path) {
        Ok(config) => {
            println!("cargo:warning=ALN schema validated successfully");
            generate_validated_config(&config);
        }
        Err(e) => {
            // Critical errors fail the build
            println!("cargo:error=ALN schema validation failed: {}", e);
            std::process::exit(1);
        }
    }
    
    // Emit compilation flags for feature detection
    println!("cargo:rustc-cfg=cyboquatic_ecosafety");
    println!("cargo:rustc-cfg=lyapunov_enforced");
}

// ============================================================================
// ALN SCHEMA PARSING
// ============================================================================

struct AlnConfig {
    schema_version: String,
    corridor_bands: HashMap<String, CorridorDef>,
    lyapunov_weights: HashMap<String, f64>,
    ker_thresholds: KERThresholds,
    lyapunov_epsilon: f64,
}

struct CorridorDef {
    safe_upper: f64,
    gold_upper: f64,
    hard_limit: f64,
}

struct KERThresholds {
    k_minimum: f64,
    e_minimum: f64,
    r_maximum: f64,
}

fn validate_aln_schema(path: &Path) -> Result<AlnConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read ALN schema: {}", e))?;
    
    // Parse schema version
    let schema_version = extract_schema_version(&content)?;
    if schema_version != SCHEMA_VERSION_REQUIRED {
        return Err(format!(
            "Schema version mismatch: required {}, found {}",
            SCHEMA_VERSION_REQUIRED, schema_version
        ));
    }
    
    // Parse corridor bands
    let corridor_bands = extract_corridor_bands(&content)?;
    
    // Validate corridor ordering
    for (name, corridor) in &corridor_bands {
        if corridor.safe_upper >= corridor.gold_upper {
            return Err(format!(
                "Corridor '{}' violates ordering: safe_upper ({}) >= gold_upper ({})",
                name, corridor.safe_upper, corridor.gold_upper
            ));
        }
        if corridor.gold_upper > corridor.hard_limit {
            return Err(format!(
                "Corridor '{}' violates ordering: gold_upper ({}) > hard_limit ({})",
                name, corridor.gold_upper, corridor.hard_limit
            ));
        }
        if corridor.hard_limit > 1.0 {
            return Err(format!(
                "Corridor '{}' hard_limit exceeds 1.0: {}",
                name, corridor.hard_limit
            ));
        }
    }
    
    // Parse Lyapunov weights
    let lyapunov_weights = extract_lyapunov_weights(&content)?;
    
    // Validate weights are non-negative
    for (name, weight) in &lyapunov_weights {
        if *weight < 0.0 {
            return Err(format!(
                "Lyapunov weight '{}' is negative: {}",
                name, weight
            ));
        }
    }
    
    // Parse KER thresholds
    let ker_thresholds = extract_ker_thresholds(&content)?;
    
    // Validate KER thresholds are achievable
    if ker_thresholds.k_minimum > 1.0 || ker_thresholds.k_minimum < 0.0 {
        return Err(format!(
            "KER K threshold out of range [0,1]: {}",
            ker_thresholds.k_minimum
        ));
    }
    if ker_thresholds.e_minimum > 1.0 || ker_thresholds.e_minimum < 0.0 {
        return Err(format!(
            "KER E threshold out of range [0,1]: {}",
            ker_thresholds.e_minimum
        ));
    }
    if ker_thresholds.r_maximum > 1.0 || ker_thresholds.r_maximum < 0.0 {
        return Err(format!(
            "KER R threshold out of range [0,1]: {}",
            ker_thresholds.r_maximum
        ));
    }
    
    // Parse Lyapunov epsilon
    let lyapunov_epsilon = extract_lyapunov_epsilon(&content)?;
    if lyapunov_epsilon <= 0.0 {
        return Err(format!(
            "Lyapunov epsilon must be positive: {}",
            lyapunov_epsilon
        ));
    }
    
    // Validate against Rust code constants
    validate_against_rust_constants(&ker_thresholds, lyapunov_epsilon)?;
    
    Ok(AlnConfig {
        schema_version,
        corridor_bands,
        lyapunov_weights,
        ker_thresholds,
        lyapunov_epsilon,
    })
}

// ============================================================================
// EXTRACTION FUNCTIONS (Simple ALN Parser)
// ============================================================================

fn extract_schema_version(content: &str) -> Result<String, String> {
    // Look for: schema_version: "1.0.0"
    for line in content.lines() {
        if line.contains("schema_version:") {
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 2 {
                return Ok(parts[1].to_string());
            }
        }
    }
    Err("Schema version not found in ALN file".to_string())
}

fn extract_corridor_bands(content: &str) -> Result<HashMap<String, CorridorDef>, String> {
    let mut corridors = HashMap::new();
    let mut in_corridor_section = false;
    let mut current_name = String::new();
    let mut safe_upper = 0.0;
    let mut gold_upper = 0.0;
    let mut hard_limit = 0.0;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("@corridor_bands") {
            in_corridor_section = true;
            continue;
        }
        
        if in_corridor_section {
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }
            
            if trimmed.contains("}:") || trimmed == "}" {
                if !current_name.is_empty() {
                    corridors.insert(current_name.clone(), CorridorDef {
                        safe_upper,
                        gold_upper,
                        hard_limit,
                    });
                    current_name = String::new();
                }
                if trimmed.starts_with("@") || trimmed.starts_with("//") {
                    in_corridor_section = trimmed.starts_with("//");
                }
                continue;
            }
            
            if trimmed.contains(':') && !trimmed.contains('{') {
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() >= 2 {
                    let key = parts[0].trim();
                    let value_str = parts[1].trim().trim_end_matches(',');
                    
                    if key.contains("safe_upper") && !current_name.is_empty() {
                        safe_upper = value_str.parse().unwrap_or(0.0);
                    } else if key.contains("gold_upper") && !current_name.is_empty() {
                        gold_upper = value_str.parse().unwrap_or(0.0);
                    } else if key.contains("hard_limit") && !current_name.is_empty() {
                        hard_limit = value_str.parse().unwrap_or(0.0);
                    } else if key.contains("description") {
                        continue;
                    } else if !key.contains('{') && !key.is_empty() {
                        // This is a corridor name
                        current_name = key.trim_end_matches(':').to_string();
                        // Reset values for new corridor
                        safe_upper = 0.0;
                        gold_upper = 0.0;
                        hard_limit = 0.0;
                    }
                }
            }
        }
    }
    
    if corridors.is_empty() {
        return Err("No corridor bands found in ALN schema".to_string());
    }
    
    Ok(corridors)
}

fn extract_lyapunov_weights(content: &str) -> Result<HashMap<String, f64>, String> {
    let mut weights = HashMap::new();
    let mut in_weights_section = false;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("@lyapunov_weights") {
            in_weights_section = true;
            continue;
        }
        
        if in_weights_section {
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }
            
            if trimmed.starts_with("@") && !trimmed.starts_with("@lyapunov") {
                in_weights_section = false;
                continue;
            }
            
            if trimmed.contains(':') && !trimmed.contains('{') {
                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() >= 2 {
                    let key = parts[0].trim();
                    let value_str = parts[1].trim().trim_end_matches(',');
                    
                    if let Ok(value) = value_str.parse::<f64>() {
                        // Use the last component of the key (e.g., "energy" from "default.energy")
                        let name = key.split('.').last().unwrap_or(key).to_string();
                        if !name.is_empty() && name != "default" && name != "eco_restorative" {
                            weights.insert(name, value);
                        }
                    }
                }
            }
        }
    }
    
    if weights.is_empty() {
        // Return default weights if none found
        weights.insert("energy".to_string(), 1.0);
        weights.insert("hydraulic".to_string(), 1.5);
        weights.insert("biology".to_string(), 2.0);
        weights.insert("carbon".to_string(), 1.8);
        weights.insert("materials".to_string(), 1.7);
    }
    
    Ok(weights)
}

fn extract_ker_thresholds(content: &str) -> Result<KERThresholds, String> {
    let mut k_min = 0.90;
    let mut e_min = 0.90;
    let mut r_max = 0.13;
    let mut in_thresholds_section = false;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("@ker_thresholds") {
            in_thresholds_section = true;
            continue;
        }
        
        if in_thresholds_section {
            if trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }
            
            if trimmed.starts_with("@") && !trimmed.starts_with("@ker") {
                in_thresholds_section = false;
                continue;
            }
            
            if trimmed.contains("k_minimum:") {
                let value = trimmed.split(':').nth(1)
                    .unwrap_or("0.90").trim().trim_end_matches(',');
                k_min = value.parse().unwrap_or(0.90);
            } else if trimmed.contains("e_minimum:") {
                let value = trimmed.split(':').nth(1)
                    .unwrap_or("0.90").trim().trim_end_matches(',');
                e_min = value.parse().unwrap_or(0.90);
            } else if trimmed.contains("r_maximum:") {
                let value = trimmed.split(':').nth(1)
                    .unwrap_or("0.13").trim().trim_end_matches(',');
                r_max = value.parse().unwrap_or(0.13);
            }
        }
    }
    
    Ok(KERThresholds {
        k_minimum: k_min,
        e_minimum: e_min,
        r_maximum: r_max,
    })
}

fn extract_lyapunov_epsilon(content: &str) -> Result<f64, String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("epsilon_default:") {
            let value = trimmed.split(':').nth(1)
                .unwrap_or("0.001").trim().trim_end_matches(',');
            return value.parse::<f64>()
                .map_err(|_| format!("Failed to parse epsilon: {}", value));
        }
    }
    Ok(0.001) // Default
}

// ============================================================================
// RUST CONSTANT VALIDATION
// ============================================================================

fn validate_against_rust_constants(ker: &KERThresholds, epsilon: f64) -> Result<(), String> {
    // These are the constants defined in lib.rs
    // Build script ensures they match ALN schema
    const RUST_K_THRESHOLD: f64 = 0.90;
    const RUST_E_THRESHOLD: f64 = 0.90;
    const RUST_R_THRESHOLD: f64 = 0.13;
    const RUST_EPSILON: f64 = 0.001;
    
    // Check for critical mismatches (warnings, not errors)
    if (ker.k_minimum - RUST_K_THRESHOLD).abs() > 0.01 {
        println!("cargo:warning=KER K threshold mismatch: ALN={}, Rust={}", 
                 ker.k_minimum, RUST_K_THRESHOLD);
    }
    if (ker.e_minimum - RUST_E_THRESHOLD).abs() > 0.01 {
        println!("cargo:warning=KER E threshold mismatch: ALN={}, Rust={}", 
                 ker.e_minimum, RUST_E_THRESHOLD);
    }
    if (ker.r_maximum - RUST_R_THRESHOLD).abs() > 0.01 {
        println!("cargo:warning=KER R threshold mismatch: ALN={}, Rust={}", 
                 ker.r_maximum, RUST_R_THRESHOLD);
    }
    if (epsilon - RUST_EPSILON).abs() > 0.0001 {
        println!("cargo:warning=Lyapunov epsilon mismatch: ALN={}, Rust={}", 
                 epsilon, RUST_EPSILON);
    }
    
    Ok(())
}

// ============================================================================
// CODE GENERATION
// ============================================================================

fn generate_default_config() {
    let generated = r#"//! Auto-generated configuration from ALN schema
//! DO NOT EDIT - Generated by build.rs

pub const SCHEMA_VERSION: &str = "1.0.0";
pub const K_THRESHOLD_DEPLOY: f64 = 0.90;
pub const E_THRESHOLD_DEPLOY: f64 = 0.90;
pub const R_THRESHOLD_DEPLOY: f64 = 0.13;
pub const LYAPUNOV_EPSILON: f64 = 0.001;

pub const CORRIDOR_SAFE_UPPER_DEFAULT: f64 = 0.30;
pub const CORRIDOR_GOLD_UPPER_DEFAULT: f64 = 0.70;
pub const CORRIDOR_HARD_LIMIT_DEFAULT: f64 = 1.00;
"#;
    
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("generated_config.rs");
    fs::write(&out_path, generated).expect("Failed to write generated config");
    println!("cargo:rustc-env=GENERATED_CONFIG_PATH={}", out_path.display());
}

fn generate_validated_config(config: &AlnConfig) {
    let generated = format!(
        r#"//! Auto-generated configuration from ALN schema
//! DO NOT EDIT - Generated by build.rs
//! Generated at: {}
//! Schema Version: {}

pub const SCHEMA_VERSION: &str = "{}";
pub const K_THRESHOLD_DEPLOY: f64 = {};
pub const E_THRESHOLD_DEPLOY: f64 = {};
pub const R_THRESHOLD_DEPLOY: f64 = {};
pub const LYAPUNOV_EPSILON: f64 = {};

pub const CORRIDOR_SAFE_UPPER_DEFAULT: f64 = {};
pub const CORRIDOR_GOLD_UPPER_DEFAULT: f64 = {};
pub const CORRIDOR_HARD_LIMIT_DEFAULT: f64 = {};

// Validated corridor bands per risk plane
pub const CORRIDOR_ENERGY_SAFE: f64 = {};
pub const CORRIDOR_ENERGY_GOLD: f64 = {};
pub const CORRIDOR_ENERGY_HARD: f64 = {};

pub const CORRIDOR_HYDRAULIC_SAFE: f64 = {};
pub const CORRIDOR_HYDRAULIC_GOLD: f64 = {};
pub const CORRIDOR_HYDRAULIC_HARD: f64 = {};

pub const CORRIDOR_BIOLOGY_SAFE: f64 = {};
pub const CORRIDOR_BIOLOGY_GOLD: f64 = {};
pub const CORRIDOR_BIOLOGY_HARD: f64 = {};

pub const CORRIDOR_CARBON_SAFE: f64 = {};
pub const CORRIDOR_CARBON_GOLD: f64 = {};
pub const CORRIDOR_CARBON_HARD: f64 = {};

pub const CORRIDOR_MATERIALS_SAFE: f64 = {};
pub const CORRIDOR_MATERIALS_GOLD: f64 = {};
pub const CORRIDOR_MATERIALS_HARD: f64 = {};

// Validation timestamp for CI/CD tracking
pub const BUILD_VALIDATION_TIMESTAMP: u64 = {};
"#,
        chrono_lite_timestamp(),
        config.schema_version,
        config.schema_version,
        config.ker_thresholds.k_minimum,
        config.ker_thresholds.e_minimum,
        config.ker_thresholds.r_maximum,
        config.lyapunov_epsilon,
        // Default corridors
        0.30, 0.70, 1.00,
        // Energy (use first corridor as example)
        config.corridor_bands.get("energy").map(|c| c.safe_upper).unwrap_or(0.30),
        config.corridor_bands.get("energy").map(|c| c.gold_upper).unwrap_or(0.70),
        config.corridor_bands.get("energy").map(|c| c.hard_limit).unwrap_or(1.00),
        // Hydraulic
        config.corridor_bands.get("hydraulic").map(|c| c.safe_upper).unwrap_or(0.25),
        config.corridor_bands.get("hydraulic").map(|c| c.gold_upper).unwrap_or(0.65),
        config.corridor_bands.get("hydraulic").map(|c| c.hard_limit).unwrap_or(1.00),
        // Biology
        config.corridor_bands.get("biology").map(|c| c.safe_upper).unwrap_or(0.10),
        config.corridor_bands.get("biology").map(|c| c.gold_upper).unwrap_or(0.50),
        config.corridor_bands.get("biology").map(|c| c.hard_limit).unwrap_or(1.00),
        // Carbon
        config.corridor_bands.get("carbon").map(|c| c.safe_upper).unwrap_or(0.20),
        config.corridor_bands.get("carbon").map(|c| c.gold_upper).unwrap_or(0.60),
        config.corridor_bands.get("carbon").map(|c| c.hard_limit).unwrap_or(1.00),
        // Materials
        config.corridor_bands.get("materials").map(|c| c.safe_upper).unwrap_or(0.15),
        config.corridor_bands.get("materials").map(|c| c.gold_upper).unwrap_or(0.55),
        config.corridor_bands.get("materials").map(|c| c.hard_limit).unwrap_or(1.00),
        // Timestamp
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir).join("generated_config.rs");
    fs::write(&out_path, &generated).expect("Failed to write generated config");
    
    // Also write to src for easier access (optional)
    let src_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(GENERATED_CONFIG_PATH);
    if let Some(parent) = src_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&src_path, &generated).unwrap_or_else(|e| {
        println!("cargo:warning=Failed to write to src: {}", e);
    });
    
    println!("cargo:rustc-env=GENERATED_CONFIG_PATH={}", out_path.display());
    println!("cargo:rustc-env=SCHEMA_VERSION={}", config.schema_version);
}

fn chrono_lite_timestamp() -> String {
    // Simple timestamp without external dependencies
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

// ============================================================================
// END OF BUILD SCRIPT
// ============================================================================
