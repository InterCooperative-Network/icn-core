#![doc = include_str!("../README.md")]

use icn_common::{CommonError, Did};
use icn_identity::ExecutionReceipt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// Store for retrieving and updating executor reputation scores.
pub trait ReputationStore: Send + Sync {
    /// Returns the numeric reputation score for the given executor DID.
    fn get_reputation(&self, did: &Did) -> u64;

    /// Updates reputation metrics using an execution receipt.
    fn record_receipt(&self, receipt: &ExecutionReceipt);
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

#[cfg(feature = "persist-sled")]
pub struct SledReputationStore {
    tree: sled::Tree,
}

#[cfg(feature = "persist-sled")]
impl SledReputationStore {
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let db = sled::open(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sled DB: {e}")))?;
        let tree = db
            .open_tree("reputation_v1")
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open tree: {e}")))?;
        Ok(Self { tree })
    }

    fn read_score(&self, did: &Did) -> Result<u64, CommonError> {
        if let Some(bytes) = self
            .tree
            .get(did.to_string())
            .map_err(|e| CommonError::DatabaseError(format!("Failed to read score: {e}")))?
        {
            Ok(bincode::deserialize(&bytes).map_err(|e| {
                CommonError::DeserializationError(format!("Failed to decode score: {e}"))
            })?)
        } else {
            Ok(0)
        }
    }

    fn write_score(&self, did: &Did, score: u64) -> Result<(), CommonError> {
        let bytes = bincode::serialize(&score)
            .map_err(|e| CommonError::SerializationError(format!("Failed to encode score: {e}")))?;
        self.tree
            .insert(did.to_string(), bytes)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to store score: {e}")))?;
        self.tree
            .flush()
            .map_err(|e| CommonError::DatabaseError(format!("Failed to flush: {e}")))?;
        Ok(())
    }
}

#[cfg(feature = "persist-sled")]
impl ReputationStore for SledReputationStore {
    fn get_reputation(&self, did: &Did) -> u64 {
        self.read_score(did).unwrap_or(0)
    }

    fn record_receipt(&self, receipt: &ExecutionReceipt) {
        let current = self.read_score(&receipt.executor_did).unwrap_or(0);
        let mut new_score = current;
        if receipt.success {
            new_score += 1 + receipt.cpu_ms / 100;
        } else if current > 0 {
            new_score = current.saturating_sub(1);
        }
        let _ = self.write_score(&receipt.executor_did, new_score);
    }
}
impl ReputationStore for InMemoryReputationStore {
    fn get_reputation(&self, did: &Did) -> u64 {
        *self.scores.lock().unwrap().get(did).unwrap_or(&0)
    }

    fn record_receipt(&self, receipt: &ExecutionReceipt) {
        let mut map = self.scores.lock().unwrap();
        let entry = map.entry(receipt.executor_did.clone()).or_insert(0);
        if receipt.success {
            *entry += 1 + receipt.cpu_ms / 100;
        } else {
            *entry = entry.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair};
    use std::str::FromStr;

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
        store.record_receipt(&receipt);
        assert_eq!(store.get_reputation(&did), 1);
    }
}

#[cfg(all(test, feature = "persist-sled"))]
mod sled_tests {
    use super::*;
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair};
    use std::str::FromStr;
    use tempfile::tempdir;

    #[test]
    fn sled_round_trip() {
        let dir = tempdir().unwrap();
        let store = SledReputationStore::new(dir.path().to_path_buf()).unwrap();
        let (_sk, vk) = generate_ed25519_keypair();
        let did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();

        let receipt = ExecutionReceipt {
            job_id: icn_common::Cid::new_v1_dummy(0x55, 0x12, b"r"),
            executor_did: did.clone(),
            result_cid: icn_common::Cid::new_v1_dummy(0x55, 0x12, b"r"),
            cpu_ms: 100,
            success: true,
            sig: icn_identity::SignatureBytes(vec![]),
        };
        store.record_receipt(&receipt);
        drop(store);

        let store2 = SledReputationStore::new(dir.path().to_path_buf()).unwrap();
        assert_eq!(store2.get_reputation(&did), 2); // 1 + cpu_ms/100
    }
}
