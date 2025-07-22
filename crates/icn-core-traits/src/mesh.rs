//! Mesh service traits and types

use crate::CoreTraitsError;
use async_trait::async_trait;
use icn_common::Did;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mesh provider trait for distributed job execution
#[async_trait]
pub trait MeshProvider: Send + Sync {
    /// Submit a job for distributed execution
    async fn submit_job(&self, job: JobSubmission) -> Result<String, CoreTraitsError>; // Returns job ID
    
    /// Get job status
    async fn get_job_status(&self, job_id: &str) -> Result<JobStatus, CoreTraitsError>;
    
    /// Cancel a job
    async fn cancel_job(&self, job_id: &str) -> Result<(), CoreTraitsError>;
    
    /// Get available executors
    async fn get_available_executors(&self) -> Result<Vec<ExecutorInfo>, CoreTraitsError>;
    
    /// Get mesh network statistics
    async fn get_mesh_stats(&self) -> Result<MeshStats, CoreTraitsError>;
}

/// Job provider trait for job management
#[async_trait]
pub trait JobProvider: Send + Sync {
    /// Create a new job
    async fn create_job(&self, submitter: &Did, job_spec: JobSpec) -> Result<String, CoreTraitsError>;
    
    /// Update job status
    async fn update_job_status(&self, job_id: &str, status: JobExecutionStatus) -> Result<(), CoreTraitsError>;
    
    /// Get job details
    async fn get_job(&self, job_id: &str) -> Result<Option<JobInfo>, CoreTraitsError>;
    
    /// List jobs by status
    async fn list_jobs_by_status(&self, status: JobExecutionStatus) -> Result<Vec<JobInfo>, CoreTraitsError>;
    
    /// Get job execution results
    async fn get_job_results(&self, job_id: &str) -> Result<Option<JobResults>, CoreTraitsError>;
}

/// Executor provider trait for executor management
#[async_trait]
pub trait ExecutorProvider: Send + Sync {
    /// Register an executor
    async fn register_executor(&self, executor: ExecutorRegistration) -> Result<String, CoreTraitsError>;
    
    /// Update executor status
    async fn update_executor_status(&self, executor_id: &str, status: ExecutorStatus) -> Result<(), CoreTraitsError>;
    
    /// Get executor capabilities
    async fn get_executor_capabilities(&self, executor_id: &str) -> Result<ExecutorCapabilities, CoreTraitsError>;
    
    /// Submit bid for a job
    async fn submit_bid(&self, executor_id: &str, job_id: &str, bid: JobBid) -> Result<(), CoreTraitsError>;
    
    /// Get executor performance metrics
    async fn get_executor_metrics(&self, executor_id: &str) -> Result<ExecutorMetrics, CoreTraitsError>;
}

/// Job submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSubmission {
    pub submitter: Did,
    pub job_spec: JobSpec,
    pub resource_requirements: ResourceRequirements,
    pub max_bid: Option<u64>,
    pub deadline: Option<u64>,
}

/// Job specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    pub job_type: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub environment: HashMap<String, String>,
    pub input_data: Option<Vec<u8>>,
    pub timeout: Option<u64>,
}

/// Resource requirements for job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: Option<u32>,
    pub memory_mb: Option<u64>,
    pub storage_mb: Option<u64>,
    pub network_bandwidth: Option<u64>,
    pub gpu_count: Option<u32>,
    pub custom_requirements: HashMap<String, String>,
}

/// Job execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobExecutionStatus {
    Pending,
    Bidding,
    Assigned,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// Job status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    pub job_id: String,
    pub status: JobExecutionStatus,
    pub assigned_executor: Option<String>,
    pub progress: f64, // 0.0 to 1.0
    pub estimated_completion: Option<u64>,
    pub error_message: Option<String>,
}

/// Job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub job_id: String,
    pub submitter: Did,
    pub job_spec: JobSpec,
    pub status: JobExecutionStatus,
    pub created_at: u64,
    pub assigned_executor: Option<String>,
    pub execution_cost: Option<u64>,
}

/// Job execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResults {
    pub job_id: String,
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub execution_time: u64,
    pub resource_usage: ResourceUsage,
    pub output_data: Option<Vec<u8>>,
}

/// Executor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorInfo {
    pub executor_id: String,
    pub owner: Did,
    pub status: ExecutorStatus,
    pub capabilities: ExecutorCapabilities,
    pub current_load: f64,
    pub reputation_score: u32,
}

/// Executor registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorRegistration {
    pub owner: Did,
    pub capabilities: ExecutorCapabilities,
    pub endpoint: String,
    pub public_key: Vec<u8>,
}

/// Executor status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutorStatus {
    Available,
    Busy,
    Offline,
    Maintenance,
}

/// Executor capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorCapabilities {
    pub max_cpu_cores: u32,
    pub max_memory_mb: u64,
    pub max_storage_mb: u64,
    pub supported_job_types: Vec<String>,
    pub gpu_count: u32,
    pub custom_capabilities: HashMap<String, String>,
}

/// Job bid from executor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobBid {
    pub executor_id: String,
    pub cost: u64,
    pub estimated_duration: u64,
    pub proposed_start_time: u64,
    pub quality_score: f64,
}

/// Executor performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorMetrics {
    pub total_jobs_completed: u64,
    pub success_rate: f64,
    pub average_execution_time: f64,
    pub average_cost: f64,
    pub uptime_percentage: f64,
    pub last_activity: u64,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub storage_usage: u64,
    pub network_usage: u64,
    pub execution_duration: u64,
}

/// Mesh network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshStats {
    pub total_executors: u64,
    pub available_executors: u64,
    pub total_jobs: u64,
    pub active_jobs: u64,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub average_job_duration: f64,
    pub network_utilization: f64,
}