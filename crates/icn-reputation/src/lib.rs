#![doc = include_str!("../README.md")]

#[cfg(feature = "persist-sled")]
use icn_common::CommonError;
use icn_common::Did;
use icn_identity::ExecutionReceipt;
use std::collections::HashMap;
#[cfg(feature = "persist-sled")]
use std::path::PathBuf;
use std::sync::Mutex;

/// Store for retrieving and updating executor reputation scores.
pub trait ReputationStore: Send + Sync {
    /// Returns the numeric reputation score for the given executor DID.
    fn get_reputation(&self, did: &Did) -> i64;

    /// Updates reputation metrics using an execution receipt and whether it
    /// represents a successful execution.
    fn record_receipt(&self, receipt: &ExecutionReceipt, success: bool);
}

/// Simple in-memory reputation tracker for tests.
#[derive(Default)]
pub struct InMemoryReputationStore {
    scores: Mutex<HashMap<Did, i64>>,
}

impl InMemoryReputationStore {
    /// Creates a new empty reputation store.
    pub fn new() -> Self {
        Self {
            scores: Mutex::new(HashMap::new()),
        }
    }

    /// Sets the reputation score for a specific executor.
    pub fn set_score(&self, did: Did, score: i64) {
        self.scores.lock().unwrap().insert(did, score);
    }
}

impl ReputationStore for InMemoryReputationStore {
    fn get_reputation(&self, did: &Did) -> i64 {
        *self.scores.lock().unwrap().get(did).unwrap_or(&0)
    }

    fn record_receipt(&self, receipt: &ExecutionReceipt, success: bool) {
        let mut map = self.scores.lock().unwrap();
        let entry = map.entry(receipt.executor_did.clone()).or_insert(0);
        let base = if success { 1 } else { -1 };
        let delta = base + (receipt.cpu_ms / 1000) as i64;
        *entry += delta;
    }
}

#[cfg(feature = "persist-sled")]
/// Persistent reputation store backed by sled.
pub struct SledReputationStore {
    db: sled::Db,
    tree_name: String,
}

#[cfg(feature = "persist-sled")]
impl SledReputationStore {
    /// Creates a new sled-backed reputation store at the given path.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let db = sled::open(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sled DB: {e}")))?;
        Ok(Self {
            db,
            tree_name: "reputation_v1".into(),
        })
    }

    fn tree(&self) -> Result<sled::Tree, CommonError> {
        self.db
            .open_tree(&self.tree_name)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open tree: {e}")))
    }
}

#[cfg(feature = "persist-sled")]
impl ReputationStore for SledReputationStore {
    fn get_reputation(&self, did: &Did) -> i64 {
        self.tree()
            .and_then(|tree| {
                Ok(
                    match tree.get(did.to_string()).map_err(|e| {
                        CommonError::DatabaseError(format!("Failed to read reputation: {e}"))
                    })? {
                        Some(val) => bincode::deserialize(&val).map_err(|e| {
                            CommonError::DeserializationError(format!(
                                "Failed to deserialize score: {e}"
                            ))
                        })?,
                        None => 0,
                    },
                )
            })
            .unwrap_or(0)
    }

    fn record_receipt(&self, receipt: &ExecutionReceipt, success: bool) {
        if let Ok(tree) = self.tree() {
            let current = self.get_reputation(&receipt.executor_did);
            let base = if success { 1 } else { -1 };
            let delta = base + (receipt.cpu_ms / 1000) as i64;
            let new_score = current + delta;
            if let Ok(encoded) = bincode::serialize(&new_score) {
                let _ = tree.insert(receipt.executor_did.to_string(), encoded);
                let _ = tree.flush();
            }
        }
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
            sig: icn_identity::SignatureBytes(vec![]),
        };
        store.record_receipt(&receipt, true);
        assert_eq!(store.get_reputation(&did), 1);
        store.record_receipt(&receipt, false);
        assert_eq!(store.get_reputation(&did), 0);
    }

    #[cfg(feature = "persist-sled")]
    #[test]
    fn sled_persistence() {
        let dir = tempdir().unwrap();
        let store = SledReputationStore::new(dir.path().to_path_buf()).unwrap();
        let (_sk, vk) = generate_ed25519_keypair();
        let did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
        let receipt = ExecutionReceipt {
            job_id: icn_common::Cid::new_v1_dummy(0x55, 0x12, b"r"),
            executor_did: did.clone(),
            result_cid: icn_common::Cid::new_v1_dummy(0x55, 0x12, b"r"),
            cpu_ms: 1000,
            sig: icn_identity::SignatureBytes(vec![]),
        };
        store.record_receipt(&receipt, true);
        assert_eq!(store.get_reputation(&did), 2);
        drop(store);
        let store2 = SledReputationStore::new(dir.path().to_path_buf()).unwrap();
        assert_eq!(store2.get_reputation(&did), 2);
    }
}
