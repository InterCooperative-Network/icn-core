//! Sled-backed implementation of `ReputationStore`.

#[cfg(feature = "persist-sled")]
use crate::ReputationStore;
#[cfg(feature = "persist-sled")]
use icn_common::{CommonError, Did};
#[cfg(feature = "persist-sled")]
use icn_identity::ExecutionReceipt;
#[cfg(feature = "persist-sled")]
use std::path::PathBuf;

#[cfg(feature = "persist-sled")]
use bincode;
#[cfg(feature = "persist-sled")]
use sled;

#[cfg(feature = "persist-sled")]
#[derive(Debug)]
pub struct SledReputationStore {
    tree: sled::Tree,
}

#[cfg(feature = "persist-sled")]
impl SledReputationStore {
    /// Opens or creates a sled database at the given path.
    pub fn new(path: PathBuf) -> Result<Self, CommonError> {
        let db = sled::open(path)
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open sled DB: {e}")))?;
        let tree = db
            .open_tree("reputation_scores")
            .map_err(|e| CommonError::DatabaseError(format!("Failed to open tree: {e}")))?;
        Ok(Self { tree })
    }

    fn read_score(&self, did: &Did) -> u64 {
        self.tree
            .get(did.to_string())
            .ok()
            .and_then(|opt| opt.and_then(|ivec| bincode::deserialize(&ivec).ok()))
            .unwrap_or(0)
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

    fn record_receipt(&self, receipt: &ExecutionReceipt) {
        let current = self.read_score(&receipt.executor_did);
        self.write_score(&receipt.executor_did, current + 1);
    }
}
