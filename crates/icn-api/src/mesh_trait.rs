use async_trait::async_trait;
use icn_common::{CommonError, Did};
use icn_mesh::JobId;

/// Information about jobs queued for a specific executor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutorQueueInfo {
    /// Executor DID the queue belongs to.
    pub executor: Did,
    /// Jobs currently awaiting execution by this executor.
    pub jobs: Vec<JobId>,
}

/// Mesh network related API operations.
#[async_trait]
pub trait MeshApi {
    /// Retrieve the current queue of jobs for the local executor.
    async fn executor_queue(&self) -> Result<ExecutorQueueInfo, CommonError>;
}
