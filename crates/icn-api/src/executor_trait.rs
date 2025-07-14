use async_trait::async_trait;
use icn_common::CommonError;
use serde::{Deserialize, Serialize};

/// Information about an executor's pending queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorQueueInfo {
    pub queued: usize,
    pub capacity: usize,
}

#[async_trait]
pub trait ExecutorIntrospectionApi {
    async fn get_executor_queue(&self, did: &str) -> Result<ExecutorQueueInfo, CommonError>;
}
