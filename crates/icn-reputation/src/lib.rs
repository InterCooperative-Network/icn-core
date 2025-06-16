#![doc = include_str!("../README.md")]

use icn_common::Did;
#[cfg(test)]
use icn_identity::ExecutionReceipt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// Store for retrieving and updating executor reputation scores.
pub trait ReputationStore: Send + Sync {
    /// Returns the numeric reputation score for the given executor DID.
    fn get_reputation(&self, did: &Did) -> u64;

    /// Updates reputation metrics for an executor.
    fn record_execution(&self, executor: &Did, success: bool, cpu_ms: u64);
}

/// Simple in-memory reputation tracker for tests.
#[derive(Default)]
pub struct InMemoryReputationStore {
    scores: Mutex<HashMap<Did, u64>>,
}

impl InMemoryReputationStore {
    /// Creates a new empty reputation store.
    pub fn new() -> Self {
        Self {
            scores: Mutex::new(HashMap::new()),
        }
    }

    /// Sets the reputation score for a specific executor.
    pub fn set_score(&self, did: Did, score: u64) {
        self.scores.lock().unwrap().insert(did, score);
    }
}

impl ReputationStore for InMemoryReputationStore {
    fn get_reputation(&self, did: &Did) -> u64 {
        *self.scores.lock().unwrap().get(did).unwrap_or(&0)
    }

    fn record_execution(&self, executor: &Did, success: bool, cpu_ms: u64) {
        let mut map = self.scores.lock().unwrap();
        let entry = map.entry(executor.clone()).or_insert(0);
        let base: i64 = if success { 1 } else { -1 };
        let delta: i64 = base + (cpu_ms / 1000) as i64;
        let updated = (*entry as i64) + delta;
        *entry = if updated < 0 { 0 } else { updated as u64 };
    }
}

/// Persistent sled-backed reputation store.
#[cfg(feature = "persist-sled")]
pub struct SledReputationStore {
    tree: sled::Tree,
}

#[cfg(feature = "persist-sled")]
impl SledReputationStore {
    /// Opens or creates a sled-backed store at the given path.
    pub fn new(path: PathBuf) -> Result<Self, icn_common::CommonError> {
        let db = sled::open(path).map_err(|e| {
            icn_common::CommonError::DatabaseError(format!("Failed to open sled DB: {e}"))
        })?;
        let tree = db.open_tree("reputation_v1").map_err(|e| {
            icn_common::CommonError::DatabaseError(format!("Failed to open tree: {e}"))
        })?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair};
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn reputation_updates() {
        let store = InMemoryReputationStore::new();
        let (_sk, vk) = generate_ed25519_keypair();
        let did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

        let receipt = ExecutionReceipt {
            job_id: icn_common::Cid::new_v1_dummy(0x55, 0x12, b"r"),
            executor_did: did.clone(),
            result_cid: icn_common::Cid::new_v1_dummy(0x55, 0x12, b"r"),
            cpu_ms: 0,
            success: true,
            sig: icn_identity::SignatureBytes(vec![]),
        };
        store.record_execution(&receipt.executor_did, receipt.success, receipt.cpu_ms);
        assert_eq!(store.get_reputation(&did), 1);
    }

    #[cfg(feature = "persist-sled")]
    #[test]
    fn sled_store_persists_scores() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rep.sled");
        let store = SledReputationStore::new(path.clone()).unwrap();
        let (_sk, vk) = generate_ed25519_keypair();
        let did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
        let receipt = ExecutionReceipt {
            job_id: icn_common::Cid::new_v1_dummy(0x55, 0x12, b"r"),
            executor_did: did.clone(),
            result_cid: icn_common::Cid::new_v1_dummy(0x55, 0x12, b"r"),
            cpu_ms: 1000,
            success: true,
            sig: icn_identity::SignatureBytes(vec![]),
        };
        store.record_execution(&receipt.executor_did, receipt.success, receipt.cpu_ms);
        assert_eq!(store.get_reputation(&did), 2);
        drop(store);
        let reopened = SledReputationStore::new(path).unwrap();
        assert_eq!(reopened.get_reputation(&did), 2);
    }
}
