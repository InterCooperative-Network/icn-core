//! RocksDB-backed implementation of the `ReputationStore` trait.

use crate::ReputationStore;
use icn_common::{CommonError, Did};
use rocksdb::DB;
use std::path::PathBuf;

#[cfg(feature = "persist-rocksdb")]
#[derive(Debug)]
pub struct RocksdbReputationStore {
    db: DB,
}

#[cfg(feature = "persist-rocksdb")]
impl RocksdbReputationStore {
    /// Open or create a RocksDB database at `path` to store reputation scores.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let db = DB::open_default(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open rocksdb: {e}")))?;
        Ok(Self { db })
    }

    fn read_score(&self, did: &Did) -> u64 {
        if let Ok(Some(bytes)) = self.db.get(did.to_string()) {
            bincode::deserialize(&bytes).unwrap_or(0)
        } else {
            0
        }
    }

    fn write_score(&self, did: &Did, score: u64) {
        if let Ok(encoded) = bincode::serialize(&score) {
            let _ = self.db.put(did.to_string(), encoded);
            let _ = self.db.flush();
        }
    }
}

#[cfg(feature = "persist-rocksdb")]
impl ReputationStore for RocksdbReputationStore {
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
        let base: i64 = if success { 1 } else { -1 };
        let updated = (current as i64) + base;
        let new_score = if updated < 0 { 0 } else { updated as u64 };
        self.write_score(prover, new_score);
    }
}
