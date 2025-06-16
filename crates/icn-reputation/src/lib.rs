#![doc = include_str!("../README.md")]

use icn_common::Did;
use icn_identity::ExecutionReceipt;
use std::collections::HashMap;
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
