use serde::Deserialize;
use serde_json::Value;
use std::fs;

#[derive(Deserialize)]
struct KER {
    knowledge_k: f64,
    eco_impact_e: f64,
    risk_of_harm_r: f64,
}

#[derive(Deserialize)]
struct Residual {
    vt: f64,
    vt_prev: f64,
}

#[derive(Deserialize)]
struct Field {
    param_id: String,
    raw_value: f64,
    unit: String,
    rx_value: f64,
    corridor_id: String,
}

#[derive(Deserialize)]
struct Shard {
    shard_id: String,
    node_id: String,
    grammar_id: String,
    origin: String,
    did_author: String,
    timestamp_utc: String,
    ker: KER,
    residual: Residual,
    fields: Vec<Field>,
    labels: Option<Vec<String>>,
    hex_stamp: String,
}

struct ValidationConfig {
    expected_grammar_id: String,
    owner_did: String,
    forbidden_labels: Vec<String>,
}

fn compute_hex_stamp(core: &Value) -> String {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;

    let s = core.to_string();
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:x}", hash)
}

fn verify_hex_stamp(shard: &Shard, raw: &Value, cfg: &ValidationConfig) -> bool {
    // Build core subset from raw JSON
    let mut core = serde_json::Map::new();
    core.insert("grammar_id".into(), raw["grammar_id"].clone());
    core.insert("did_author".into(), raw["did_author"].clone());
    core.insert("ker".into(), raw["ker"].clone());
    core.insert("residual".into(), raw["residual"].clone());
    core.insert("fields".into(), raw["fields"].clone());

    let core_val = Value::Object(core);
    let expected = compute_hex_stamp(&core_val);
    expected == shard.hex_stamp
}

fn labels_ok(shard: &Shard, cfg: &ValidationConfig) -> bool {
    if let Some(labels) = &shard.labels {
        for lbl in labels {
            for forbidden in &cfg.forbidden_labels {
                if lbl.contains(forbidden) {
                    return false;
                }
            }
        }
    }
    true
}

fn check_origin_and_grammar(shard: &Shard, cfg: &ValidationConfig) -> bool {
    if shard.grammar_id == cfg.expected_grammar_id {
        if shard.did_author != cfg.owner_did {
            return false;
        }
        if shard.origin != "internal" {
            return false;
        }
    }
    true
}

fn validate_shard(shard: &Shard, raw: &Value, cfg: &ValidationConfig) -> bool {
    if !check_origin_and_grammar(shard, cfg) {
        eprintln!("Origin/grammar/DID mismatch.");
        return false;
    }
    if !labels_ok(shard, cfg) {
        eprintln!("Forbidden external branding label detected.");
        return false;
    }
    if !verify_hex_stamp(shard, raw, cfg) {
        eprintln!("Hex stamp verification failed.");
        return false;
    }
    true
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: cybo_ecosafety_validator <json_file> <expected_grammar_id> <owner_did>");
        std::process::exit(1);
    }

    let json_path        = &args[1];
    let expected_grammar = &args[2];
    let owner_did        = &args[3];

    let cfg = ValidationConfig {
        expected_grammar_id: expected_grammar.clone(),
        owner_did: owner_did.clone(),
        forbidden_labels: vec![
            "DigitalWaterTwin".into(),
            "Digital Water Twin".into(),
            "SmartWaterTwin".into(),
        ],
    };

    let data = fs::read_to_string(json_path)
        .expect("Cannot read JSON file");
    let raw: Value = serde_json::from_str(&data)
        .expect("Invalid JSON");
    let shard: Shard = serde_json::from_value(raw.clone())
        .expect("JSON does not match Shard schema");

    if !validate_shard(&shard, &raw, &cfg) {
        eprintln!("Shard validation FAILED for {}", json_path);
        std::process::exit(2);
    }

    println!("Shard validation OK for {}", json_path);
}
