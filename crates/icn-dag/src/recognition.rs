use crate::{DagBlock, StorageService};
use icn_common::TimeProvider;
use icn_common::{compute_merkle_cid, CommonError, Did, NodeScope, SystemTimeProvider};
use serde::{Deserialize, Serialize};

/// Record describing a contributor action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionRecord {
    pub contributor: Did,
    pub message: String,
    pub timestamp: u64,
}

/// Append a contribution record to the DAG and return its CID.
pub fn log_contribution(
    store: &mut dyn StorageService<DagBlock>,
    contributor: Did,
    message: String,
    scope: Option<NodeScope>,
) -> Result<icn_common::Cid, CommonError> {
    let record = ContributionRecord {
        contributor: contributor.clone(),
        message,
        timestamp: SystemTimeProvider.unix_seconds(),
    };
    let data =
        serde_json::to_vec(&record).map_err(|e| CommonError::SerializationError(e.to_string()))?;
    let cid = compute_merkle_cid(
        0x71,
        &data,
        &[],
        record.timestamp,
        &contributor,
        &None,
        &scope,
    );
    let block = DagBlock {
        cid: cid.clone(),
        data,
        links: vec![],
        timestamp: record.timestamp,
        author_did: contributor,
        signature: None,
        scope,
    };
    store.put(&block)?;
    Ok(cid)
}
