#![doc = include_str!("../README.md")]

#[cfg(feature = "async")]
use async_trait::async_trait;
use icn_common::Did;
#[cfg(test)]
use icn_identity::ExecutionReceipt;
use std::collections::HashMap;
use std::sync::Mutex;

#[cfg(feature = "persist-sled")]
pub mod sled_store;
#[cfg(feature = "persist-sled")]
pub use sled_store::SledReputationStore;
#[cfg(feature = "persist-sqlite")]
pub mod sqlite_store;
#[cfg(feature = "persist-sqlite")]
pub use sqlite_store::SqliteReputationStore;
#[cfg(feature = "persist-rocksdb")]
pub mod rocksdb_store;
#[cfg(feature = "persist-rocksdb")]
pub use rocksdb_store::RocksdbReputationStore;
pub mod metrics;

/// Store for retrieving and updating executor reputation scores.
pub trait ReputationStore: Send + Sync {
    /// Returns the numeric reputation score for the given executor DID.
    fn get_reputation(&self, did: &Did) -> u64;

    /// Updates reputation metrics for an executor.
    fn record_execution(&self, executor: &Did, success: bool, cpu_ms: u64);

    /// Records an attempt to verify a zero-knowledge proof.
    fn record_proof_attempt(&self, prover: &Did, success: bool);
}

impl std::fmt::Debug for dyn ReputationStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReputationStore")
    }
}

#[cfg(feature = "async")]
#[async_trait]
pub trait AsyncReputationStore: Send + Sync {
    async fn get_reputation(&self, did: &Did) -> u64;

    async fn record_execution(&self, executor: &Did, success: bool, cpu_ms: u64);

    async fn record_proof_attempt(&self, prover: &Did, success: bool);
}

#[cfg(feature = "async")]
pub struct CompatAsyncReputationStore<S> {
    inner: S,
}

#[cfg(feature = "async")]
impl<S> CompatAsyncReputationStore<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> S {
        self.inner
    }
}

#[cfg(feature = "async")]
#[async_trait]
impl<S> AsyncReputationStore for CompatAsyncReputationStore<S>
where
    S: ReputationStore + Send + Sync,
{
    async fn get_reputation(&self, did: &Did) -> u64 {
        self.inner.get_reputation(did)
    }

    async fn record_execution(&self, executor: &Did, success: bool, cpu_ms: u64) {
        self.inner.record_execution(executor, success, cpu_ms);
    }

    async fn record_proof_attempt(&self, prover: &Did, success: bool) {
        self.inner.record_proof_attempt(prover, success);
    }
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
        crate::metrics::EXECUTION_RECORDS.inc();
        let mut map = self.scores.lock().unwrap();
        let entry = map.entry(executor.clone()).or_insert(0);
        let base: i64 = if success { 1 } else { -1 };
        let delta: i64 = base + (cpu_ms / 1000) as i64;
        let updated = (*entry as i64) + delta;
        *entry = if updated < 0 { 0 } else { updated as u64 };
    }

    fn record_proof_attempt(&self, prover: &Did, success: bool) {
        crate::metrics::PROOF_ATTEMPTS.inc();
        let mut map = self.scores.lock().unwrap();
        let entry = map.entry(prover.clone()).or_insert(0);
        let delta: i64 = if success { 1 } else { -1 };
        let updated = (*entry as i64) + delta;
        *entry = if updated < 0 { 0 } else { updated as u64 };
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
            job_id: icn_common::Cid::new_v1_sha256(0x55, b"r"),
            executor_did: did.clone(),
            result_cid: icn_common::Cid::new_v1_sha256(0x55, b"r"),
            cpu_ms: 0,
            success: true,
            sig: icn_identity::SignatureBytes(vec![]),
        };
        store.record_execution(&receipt.executor_did, receipt.success, receipt.cpu_ms);
        assert_eq!(store.get_reputation(&did), 1);

        store.record_proof_attempt(&did, true);
        assert_eq!(store.get_reputation(&did), 2);

        store.record_proof_attempt(&did, false);
        assert_eq!(store.get_reputation(&did), 1);
    }
}
