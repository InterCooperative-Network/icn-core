use async_trait::async_trait;
use icn_common::{Cid, CommonError, DagSyncStatus};

/// API surface for DAG operations.
#[async_trait]
pub trait DagApi {
    /// Return the current root CID of the node's DAG.
    ///
    /// Clients can poll this value to detect when a peer has new history and
    /// needs state synchronization.
    async fn get_dag_root(&self) -> Result<Option<Cid>, CommonError>;

    /// Retrieve synchronization status for the local DAG.
    async fn get_dag_sync_status(&self) -> Result<DagSyncStatus, CommonError>;
}
