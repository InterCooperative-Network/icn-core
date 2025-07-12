use icn_common::{compute_merkle_cid, Cid, CommonError, DagBlock, Did, NodeScope};
use serde::{Deserialize, Serialize};

/// Action recorded in the [`ResourceLedger`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceAction {
    Acquire,
    Consume,
}

/// Entry representing a resource event anchored in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLedgerEntry {
    pub did: Did,
    pub resource_id: String,
    pub action: ResourceAction,
    pub timestamp: u64,
    pub cid: Cid,
    pub scope: Option<NodeScope>,
}

/// Simple in-memory ledger of resource events.
#[derive(Debug, Default)]
pub struct ResourceLedger {
    entries: Vec<ResourceLedgerEntry>,
}

impl ResourceLedger {
    /// Create a new empty ledger.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Append an entry to the ledger.
    pub fn push(&mut self, entry: ResourceLedgerEntry) {
        self.entries.push(entry);
    }

    /// List all recorded entries.
    pub fn all(&self) -> Vec<ResourceLedgerEntry> {
        self.entries.clone()
    }
}

/// Record a resource event and anchor it in the DAG store.
pub async fn record_resource_event(
    dag_store: &mut dyn icn_dag::AsyncStorageService<DagBlock>,
    did: &Did,
    resource_id: String,
    action: ResourceAction,
    timestamp: u64,
    scope: Option<NodeScope>,
) -> Result<Cid, CommonError> {
    let entry_tmp = ResourceLedgerEntry {
        did: did.clone(),
        resource_id,
        action,
        timestamp,
        cid: Cid::default(),
        scope: scope.clone(),
    };
    let data = bincode::serialize(&entry_tmp)?;
    let cid = compute_merkle_cid(0x71, &data, &[], timestamp, did, &None, &scope);
    let entry = ResourceLedgerEntry {
        cid: cid.clone(),
        ..entry_tmp
    };
    let block = DagBlock {
        cid: cid.clone(),
        data,
        links: vec![],
        timestamp,
        author_did: did.clone(),
        signature: None,
        scope,
    };
    dag_store.put(&block).await?;
    Ok(cid)
}
