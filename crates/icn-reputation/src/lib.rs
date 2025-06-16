#![doc = include_str!("../README.md")]

use icn_common::Did;
use icn_identity::ExecutionReceipt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

/// Store for retrieving and updating executor reputation scores.
///
/// # Examples
///
/// ```
/// use icn_reputation::{InMemoryReputationStore, ReputationStore};
/// use icn_identity::{generate_ed25519_keypair, did_key_from_verifying_key, ExecutionReceipt, SignatureBytes};
/// use icn_common::{Cid, Did};
/// use std::str::FromStr;
///
/// let store = InMemoryReputationStore::new();
/// let (_sk, vk) = generate_ed25519_keypair();
/// let did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
/// let receipt = ExecutionReceipt {
///     job_id: Cid::new_v1_dummy(0x55, 0x12, b"r"),
///     executor_did: did.clone(),
///     result_cid: Cid::new_v1_dummy(0x55, 0x12, b"r"),
///     cpu_ms: 0,
///     sig: SignatureBytes(vec![]),
/// };
/// store.record_receipt(&receipt);
/// assert_eq!(store.get_reputation(&did), 1);
/// ```
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

impl ReputationStore for InMemoryReputationStore {
    fn get_reputation(&self, did: &Did) -> u64 {
        *self.scores.lock().unwrap().get(did).unwrap_or(&0)
    }

    fn record_receipt(&self, receipt: &ExecutionReceipt) {
        let mut map = self.scores.lock().unwrap();
        let entry = map.entry(receipt.executor_did.clone()).or_insert(0);
        *entry += 1;
    }
}

#[cfg(feature = "persist-sled")]
/// Persistent reputation tracker backed by `sled`.
///
/// ```
/// use icn_reputation::{SledReputationStore, ReputationStore};
/// use icn_identity::{generate_ed25519_keypair, did_key_from_verifying_key, ExecutionReceipt, SignatureBytes};
/// use icn_common::{Cid, Did};
/// use std::str::FromStr;
/// use tempfile::TempDir;
///
/// let dir = TempDir::new().unwrap();
/// let store = SledReputationStore::new(dir.path().to_path_buf()).unwrap();
/// let (_sk, vk) = generate_ed25519_keypair();
/// let did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
/// let receipt = ExecutionReceipt {
///     job_id: Cid::new_v1_dummy(0x55, 0x12, b"r"),
///     executor_did: did.clone(),
///     result_cid: Cid::new_v1_dummy(0x55, 0x12, b"r"),
///     cpu_ms: 0,
///     sig: SignatureBytes(vec![]),
/// };
/// store.record_receipt(&receipt);
/// assert_eq!(store.get_reputation(&did), 1);
/// ```
pub struct SledReputationStore {
    tree: sled::Tree,
}

#[cfg(feature = "persist-sled")]
impl SledReputationStore {
    pub fn new(path: PathBuf) -> Result<Self, icn_common::CommonError> {
        let db = sled::open(path).map_err(|e| {
            icn_common::CommonError::DatabaseError(format!("Failed to open sled DB: {e}"))
        })?;
        let tree = db.open_tree("reputation_scores").map_err(|e| {
            icn_common::CommonError::DatabaseError(format!("Failed to open tree: {e}"))
        })?;
        Ok(Self { tree })
    }

    fn write_score(&self, did: &Did, score: u64) -> Result<(), icn_common::CommonError> {
        let encoded = bincode::serialize(&score).map_err(|e| {
            icn_common::CommonError::SerializationError(format!("Failed to serialize score: {e}"))
        })?;
        self.tree.insert(did.to_string(), encoded).map_err(|e| {
            icn_common::CommonError::DatabaseError(format!("Failed to store score: {e}"))
        })?;
        self.tree.flush().map_err(|e| {
            icn_common::CommonError::DatabaseError(format!("Failed to flush tree: {e}"))
        })?;
        Ok(())
    }

    fn read_score(&self, did: &Did) -> Result<u64, icn_common::CommonError> {
        if let Some(val) = self.tree.get(did.to_string()).map_err(|e| {
            icn_common::CommonError::DatabaseError(format!("Failed to read score: {e}"))
        })? {
            let score: u64 = bincode::deserialize(&val).map_err(|e| {
                icn_common::CommonError::DeserializationError(format!(
                    "Failed to deserialize score: {e}"
                ))
            })?;
            Ok(score)
        } else {
            Ok(0)
        }
    }
}

#[cfg(feature = "persist-sled")]
impl ReputationStore for SledReputationStore {
    fn get_reputation(&self, did: &Did) -> u64 {
        self.read_score(did).unwrap_or(0)
    }

    fn record_receipt(&self, receipt: &ExecutionReceipt) {
        let current = self.read_score(&receipt.executor_did).unwrap_or(0);
        let _ = self.write_score(&receipt.executor_did, current + 1);
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
            sig: icn_identity::SignatureBytes(vec![]),
        };
        store.record_receipt(&receipt);
        assert_eq!(store.get_reputation(&did), 1);
    }
}
