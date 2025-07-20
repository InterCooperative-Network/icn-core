use async_trait::async_trait;
use icn_common::{CommonError, Did};
use icn_mesh::{JobCheckpoint, JobId, PartialOutputReceipt, ProgressReport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about jobs queued for a specific executor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutorQueueInfo {
    /// Executor DID the queue belongs to.
    pub executor: Did,
    /// Jobs currently awaiting execution by this executor.
    pub jobs: Vec<JobId>,
}

/// Response containing job progress information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgressResponse {
    /// The job ID this progress relates to.
    pub job_id: JobId,
    /// Current progress report.
    pub progress: Option<ProgressReport>,
    /// Available checkpoints for this job.
    pub checkpoints: Vec<JobCheckpoint>,
    /// Partial outputs produced so far.
    pub partial_outputs: Vec<PartialOutputReceipt>,
    /// Whether the job is currently running.
    pub is_running: bool,
    /// Timestamp of this response.
    pub timestamp: u64,
}

/// Streaming data chunk for job output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStreamChunk {
    /// The job ID this chunk relates to.
    pub job_id: JobId,
    /// Sequence number of this chunk.
    pub sequence: u64,
    /// The stage that produced this chunk.
    pub stage: String,
    /// Raw data bytes.
    pub data: Vec<u8>,
    /// MIME type of the data.
    pub content_type: Option<String>,
    /// Whether this is the final chunk.
    pub is_final: bool,
    /// Timestamp when this chunk was produced.
    pub timestamp: u64,
}

/// Metrics for job execution and mesh operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshMetrics {
    /// Total number of jobs processed.
    pub total_jobs: u64,
    /// Number of currently running jobs.
    pub running_jobs: u64,
    /// Number of jobs with checkpoints (long-running).
    pub long_running_jobs: u64,
    /// Number of completed jobs.
    pub completed_jobs: u64,
    /// Number of failed jobs.
    pub failed_jobs: u64,
    /// Average job execution time in seconds.
    pub avg_execution_time_secs: f64,
    /// Additional custom metrics.
    pub custom_metrics: HashMap<String, f64>,
}

/// Mesh network related API operations.
#[async_trait]
pub trait MeshApi {
    /// Retrieve the current queue of jobs for the local executor.
    async fn executor_queue(&self) -> Result<ExecutorQueueInfo, CommonError>;

    /// Get progress information for a specific job.
    /// Returns current progress, checkpoints, and partial outputs.
    async fn get_job_progress(&self, job_id: &JobId) -> Result<JobProgressResponse, CommonError>;

    /// Get streaming output chunks for a job.
    /// Returns available output chunks in order.
    async fn get_job_stream(
        &self,
        job_id: &JobId,
        from_sequence: Option<u64>,
    ) -> Result<Vec<JobStreamChunk>, CommonError>;

    /// Get the latest streaming chunk for a job.
    /// Useful for real-time monitoring.
    async fn get_latest_job_chunk(
        &self,
        job_id: &JobId,
    ) -> Result<Option<JobStreamChunk>, CommonError>;

    /// Cancel a running job.
    /// This will attempt to stop execution and clean up resources.
    async fn cancel_job(&self, job_id: &JobId) -> Result<bool, CommonError>;

    /// Resume a job from its latest checkpoint.
    /// Returns whether the job was successfully resumed.
    async fn resume_job(&self, job_id: &JobId) -> Result<bool, CommonError>;

    /// Get mesh execution metrics.
    /// Returns Prometheus-compatible metrics about job execution.
    async fn get_mesh_metrics(&self) -> Result<MeshMetrics, CommonError>;

    /// Get detailed metrics for a specific job.
    /// Includes timing, resource usage, and checkpoint information.
    async fn get_job_metrics(&self, job_id: &JobId) -> Result<HashMap<String, f64>, CommonError>;
}
