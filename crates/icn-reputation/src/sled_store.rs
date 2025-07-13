//! Sled-backed implementation of the [`ReputationStore`] trait.

use crate::ReputationStore;
use icn_common::{CommonError, Did};
use std::path::PathBuf;

#[cfg(feature = "persist-sled")]
use bincode;
#[cfg(feature = "persist-sled")]
use sled;

/// Persistent sled-backed reputation store.
#[cfg(feature = "persist-sled")]
pub struct SledReputationStore {
    tree: sled::Tree,
}

#[cfg(feature = "persist-sled")]
impl SledReputationStore {
    /// Opens or creates a sled-backed store at the given path.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let db = sled::open(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sled DB: {e}")))?;
        let tree = db
            .open_tree("reputation_v1")
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open tree: {e}")))?;
        Ok(Self { tree })
    }

    fn read_score(&self, did: &Did) -> u64 {
        if let Ok(Some(bytes)) = self.tree.get(did.to_string()) {
            bincode::deserialize(&bytes).unwrap_or(0)
        } else {
            0
        }
    }

    fn write_score(&self, did: &Did, score: u64) {
        if let Ok(encoded) = bincode::serialize(&score) {
            let _ = self.tree.insert(did.to_string(), encoded);
            let _ = self.tree.flush();
        }
    }
}

#[cfg(feature = "persist-sled")]
impl ReputationStore for SledReputationStore {
    fn get_reputation(&self, did: &Did) -> u64 {
        self.read_score(did)
    }

    fn record_execution(&self, executor: &Did, success: bool, cpu_ms: u64) {
        let current = self.read_score(executor);
        let base: i64 = if success { 1 } else { -1 };
        let delta: i64 = base + (cpu_ms / 1000) as i64;
        let updated = (current as i64) + delta;
        let new_score = if updated < 0 { 0 } else { updated as u64 };
        self.write_score(executor, new_score);
    }

    fn record_proof_attempt(&self, prover: &Did, success: bool) {
        let current = self.read_score(prover);
        let delta: i64 = if success { 1 } else { -1 };
        let updated = (current as i64) + delta;
        let new_score = if updated < 0 { 0 } else { updated as u64 };
        self.write_score(prover, new_score);
    }
}
