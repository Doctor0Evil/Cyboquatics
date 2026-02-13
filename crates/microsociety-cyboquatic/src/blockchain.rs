use crate::episode::EpisodeLog;
use serde::{Deserialize, Serialize};

/// Minimal hash-linkable metadata for an Episode knowledge-object.
/// The actual Googolswarm integration is handled by your existing stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeAnchor {
    pub episode_id: String,
    pub hash: String,
    pub length: usize,
}

pub fn compute_episode_anchor(episode_id: &str, log: &EpisodeLog) -> EpisodeAnchor {
    // Use your existing Googolswarm hash-linking implementation here.
    // Placeholder: use a simple stable hash like SHA2 via an external crate,
    // avoiding any blacklisted algorithms.
    let json = serde_json::to_vec(log).expect("serialize episode log");
    let digest = sha2::Sha256::digest(&json);
    let hash = hex::encode(digest);

    EpisodeAnchor {
        episode_id: episode_id.to_string(),
        hash,
        length: log.ticks.len(),
    }
}
