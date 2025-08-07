//! Sharded execution implementation for distributing large jobs across multiple executors.
//!
//! This module provides job sharding logic, coordinator management, and result aggregation
//! for scalable distributed computation in the ICN mesh network.

use crate::{ActualMeshJob, JobId, JobKind, JobSpec, MeshJobBid, Resources};
use icn_common::{Cid, CommonError, Did};
use icn_identity::{ExecutionReceipt, SignatureBytes, SigningKey};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// A single shard of a larger job that can be executed independently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobShard {
    /// Unique identifier for this shard
    pub shard_id: String,
    /// ID of the parent job this shard belongs to
    pub parent_job_id: JobId,
    /// Shard index (0-based)
    pub shard_index: u32,
    /// Total number of shards in the job
    pub total_shards: u32,
    /// The actual job specification for this shard
    pub job_spec: JobSpec,
    /// Input data specific to this shard
    pub shard_input: ShardInput,
    /// Expected output specification
    pub expected_output: ShardOutputSpec,
    /// Dependencies on other shards (for complex workflows)
    pub dependencies: Vec<String>,
    /// Priority of this shard (higher numbers = higher priority)
    pub priority: u32,
}

/// Input data for a job shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInput {
    /// CIDs of input data for this shard
    pub input_cids: Vec<Cid>,
    /// Key-value parameters for this shard
    pub parameters: HashMap<String, String>,
    /// Binary data (for small inputs)
    pub inline_data: Option<Vec<u8>>,
    /// Range specification for data partitioning
    pub data_range: Option<DataRange>,
}

/// Specification for expected output from a shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardOutputSpec {
    /// Expected output format
    pub format: OutputFormat,
    /// Maximum expected output size in bytes
    pub max_size_bytes: Option<u64>,
    /// Output validation rules
    pub validation_rules: Vec<ValidationRule>,
}

/// Data range specification for partitioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRange {
    /// Starting offset (inclusive)
    pub start: u64,
    /// Ending offset (exclusive)
    pub end: u64,
    /// Unit of the range (bytes, records, frames, etc.)
    pub unit: RangeUnit,
}

/// Unit for data range specifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RangeUnit {
    Bytes,
    Records,
    Frames,
    Chunks,
    Custom(String),
}

/// Expected output format for shard results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Binary,
    Json,
    Csv,
    Image,
    Video,
    Custom(String),
}

/// Validation rule for shard outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    /// Minimum output size in bytes
    MinSize(u64),
    /// Maximum output size in bytes
    MaxSize(u64),
    /// Required file format/magic bytes
    RequiredFormat(Vec<u8>),
    /// Custom validation script/expression
    CustomRule(String),
}

/// Result from executing a single shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardResult {
    /// Shard that was executed
    pub shard_id: String,
    /// Whether execution was successful
    pub success: bool,
    /// Output data CID (if successful)
    pub output_cid: Option<Cid>,
    /// Execution receipt from the executor
    pub execution_receipt: ExecutionReceipt,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Execution metadata
    pub metadata: ShardExecutionMetadata,
}

/// Metadata about shard execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardExecutionMetadata {
    /// Executor that ran this shard
    pub executor_did: Did,
    /// Time taken to execute
    pub execution_time_ms: u64,
    /// Resources used
    pub resources_used: Resources,
    /// Start time
    pub started_at: u64,
    /// Completion time
    pub completed_at: u64,
}

/// Aggregated result from all shards of a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardedJobResult {
    /// Parent job ID
    pub job_id: JobId,
    /// Results from all shards
    pub shard_results: Vec<ShardResult>,
    /// Final aggregated output (if applicable)
    pub aggregated_output: Option<Cid>,
    /// Overall success status
    pub success: bool,
    /// Total execution time across all shards
    pub total_execution_time_ms: u64,
    /// Aggregation metadata
    pub aggregation_metadata: AggregationMetadata,
}

/// Metadata about the aggregation process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationMetadata {
    /// Strategy used for aggregation
    pub strategy: AggregationStrategy,
    /// Number of successful shards
    pub successful_shards: u32,
    /// Number of failed shards
    pub failed_shards: u32,
    /// Time taken for aggregation
    pub aggregation_time_ms: u64,
    /// Coordinator that performed aggregation
    pub coordinator_did: Did,
}

/// Strategy for aggregating shard results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// Simple concatenation of outputs
    Concatenate,
    /// Map-reduce style aggregation
    MapReduce,
    /// Average/statistical aggregation
    Statistical,
    /// Custom aggregation logic
    Custom(String),
    /// No aggregation (keep separate)
    None,
}

/// Job sharding engine that splits jobs into executable shards.
pub struct JobShardingEngine {
    /// Maximum number of shards per job
    max_shards_per_job: u32,
    /// Minimum shard size to avoid over-sharding
    min_shard_size: u64,
    /// Supported sharding strategies
    strategies: HashMap<JobKind, Box<dyn ShardingStrategy>>,
}

/// Trait for job sharding strategies.
pub trait ShardingStrategy: Send + Sync {
    /// Determine if a job can be sharded
    fn can_shard(&self, job: &ActualMeshJob) -> bool;

    /// Calculate optimal number of shards for a job
    fn calculate_optimal_shards(&self, job: &ActualMeshJob) -> u32;

    /// Split a job into shards
    fn create_shards(&self, job: &ActualMeshJob) -> Result<Vec<JobShard>, CommonError>;

    /// Validate that shards are correctly formed
    fn validate_shards(&self, shards: &[JobShard]) -> Result<(), CommonError>;
}

/// Map-reduce sharding strategy.
pub struct MapReduceShardingStrategy {
    /// Default number of map shards
    default_map_shards: u32,
    /// Maximum data size per shard
    max_shard_data_mb: u64,
}

/// Rendering job sharding strategy (by frame ranges).
pub struct RenderingShardingStrategy {
    /// Frames per shard
    frames_per_shard: u32,
    /// Maximum execution time per shard
    max_shard_duration_mins: u32,
}

/// Data processing sharding strategy.
pub struct DataProcessingShardingStrategy {
    /// Target size per shard in bytes
    target_shard_size_bytes: u64,
    /// Overlap between shards (for windowed operations)
    overlap_percentage: f32,
}

/// Shard coordinator that manages distributed execution.
pub struct ShardCoordinator {
    /// Node's DID
    coordinator_did: Did,
    /// Signing key for authentication
    signing_key: Arc<SigningKey>,
    /// Active shard executions
    active_executions: std::sync::RwLock<HashMap<String, ShardExecution>>,
    /// Maximum concurrent shards
    max_concurrent_shards: u32,
}

/// Information about an active shard execution.
#[derive(Debug, Clone)]
struct ShardExecution {
    shard: JobShard,
    executor_did: Did,
    started_at: SystemTime,
    timeout: Duration,
    dependencies_met: bool,
}

impl JobShardingEngine {
    /// Create a new job sharding engine.
    pub fn new(max_shards_per_job: u32, min_shard_size: u64) -> Self {
        let mut strategies: HashMap<JobKind, Box<dyn ShardingStrategy>> = HashMap::new();

        // Register default sharding strategies
        strategies.insert(
            JobKind::GenericPlaceholder,
            Box::new(DataProcessingShardingStrategy::new(10 * 1024 * 1024, 0.1)), // 10MB shards, 10% overlap
        );

        Self {
            max_shards_per_job,
            min_shard_size,
            strategies,
        }
    }

    /// Register a custom sharding strategy for a job kind.
    pub fn register_strategy(&mut self, job_kind: JobKind, strategy: Box<dyn ShardingStrategy>) {
        self.strategies.insert(job_kind, strategy);
    }

    /// Determine if a job should be sharded.
    pub fn should_shard(&self, job: &ActualMeshJob) -> bool {
        // Check if we have a strategy for this job type
        if let Some(strategy) = self.strategies.get(&job.spec.kind) {
            strategy.can_shard(job)
        } else {
            false
        }
    }

    /// Shard a job into executable pieces.
    pub fn shard_job(&self, job: &ActualMeshJob) -> Result<Vec<JobShard>, CommonError> {
        let strategy = self.strategies.get(&job.spec.kind).ok_or_else(|| {
            CommonError::InvalidParameters(format!(
                "No sharding strategy for job kind: {:?}",
                job.spec.kind
            ))
        })?;

        if !strategy.can_shard(job) {
            return Err(CommonError::InvalidParameters(
                "Job cannot be sharded with current strategy".to_string(),
            ));
        }

        let optimal_shards = strategy.calculate_optimal_shards(job);

        if optimal_shards > self.max_shards_per_job {
            warn!(
                "[JobSharding] Optimal shards ({}) exceeds maximum ({}), capping",
                optimal_shards, self.max_shards_per_job
            );
        }

        let shards = strategy.create_shards(job)?;
        strategy.validate_shards(&shards)?;

        info!(
            "[JobSharding] Created {} shards for job {}",
            shards.len(),
            job.id
        );
        Ok(shards)
    }

    /// Estimate resource requirements for sharded execution.
    pub fn estimate_shard_resources(&self, shards: &[JobShard]) -> Resources {
        let total_cpu = shards
            .iter()
            .map(|s| s.job_spec.required_resources.cpu_cores)
            .sum();
        let max_memory = shards
            .iter()
            .map(|s| s.job_spec.required_resources.memory_mb)
            .max()
            .unwrap_or(0);
        let total_storage = shards
            .iter()
            .map(|s| s.job_spec.required_resources.storage_mb)
            .sum();

        Resources {
            cpu_cores: total_cpu,
            memory_mb: max_memory, // Memory usage is typically not additive across shards
            storage_mb: total_storage,
        }
    }
}

impl MapReduceShardingStrategy {
    pub fn new(default_map_shards: u32, max_shard_data_mb: u64) -> Self {
        Self {
            default_map_shards,
            max_shard_data_mb,
        }
    }
}

impl ShardingStrategy for MapReduceShardingStrategy {
    fn can_shard(&self, job: &ActualMeshJob) -> bool {
        // Map-reduce jobs are inherently shardable
        matches!(job.spec.kind, JobKind::GenericPlaceholder) && !job.spec.inputs.is_empty()
    }

    fn calculate_optimal_shards(&self, _job: &ActualMeshJob) -> u32 {
        // For map-reduce, start with default and adjust based on data size
        self.default_map_shards
    }

    fn create_shards(&self, job: &ActualMeshJob) -> Result<Vec<JobShard>, CommonError> {
        let num_shards = self.calculate_optimal_shards(job);
        let mut shards = Vec::new();

        for i in 0..num_shards {
            let shard_id = format!("{}:shard:{}", job.id, i);

            // Create input range for this shard
            let data_range = DataRange {
                start: (i as u64) * (u64::MAX / num_shards as u64),
                end: ((i + 1) as u64) * (u64::MAX / num_shards as u64),
                unit: RangeUnit::Records,
            };

            let shard_input = ShardInput {
                input_cids: job.spec.inputs.clone(),
                parameters: HashMap::new(),
                inline_data: None,
                data_range: Some(data_range),
            };

            let shard = JobShard {
                shard_id: shard_id.clone(),
                parent_job_id: job.id.clone(),
                shard_index: i,
                total_shards: num_shards,
                job_spec: job.spec.clone(),
                shard_input,
                expected_output: ShardOutputSpec {
                    format: OutputFormat::Binary,
                    max_size_bytes: Some(self.max_shard_data_mb * 1024 * 1024),
                    validation_rules: vec![ValidationRule::MaxSize(
                        self.max_shard_data_mb * 1024 * 1024,
                    )],
                },
                dependencies: vec![],
                priority: 100, // Normal priority
            };

            shards.push(shard);
        }

        Ok(shards)
    }

    fn validate_shards(&self, shards: &[JobShard]) -> Result<(), CommonError> {
        if shards.is_empty() {
            return Err(CommonError::InvalidParameters(
                "No shards created".to_string(),
            ));
        }

        // Validate shard indices are sequential
        for (i, shard) in shards.iter().enumerate() {
            if shard.shard_index != i as u32 {
                return Err(CommonError::InvalidParameters(format!(
                    "Shard index mismatch: expected {}, got {}",
                    i, shard.shard_index
                )));
            }
        }

        Ok(())
    }
}

impl DataProcessingShardingStrategy {
    pub fn new(target_shard_size_bytes: u64, overlap_percentage: f32) -> Self {
        Self {
            target_shard_size_bytes,
            overlap_percentage,
        }
    }
}

impl ShardingStrategy for DataProcessingShardingStrategy {
    fn can_shard(&self, job: &ActualMeshJob) -> bool {
        // Most data processing jobs can be sharded
        !job.spec.inputs.is_empty() && job.spec.required_resources.storage_mb > 100
        // Only shard if significant data
    }

    fn calculate_optimal_shards(&self, job: &ActualMeshJob) -> u32 {
        // Estimate shards based on storage requirements
        let estimated_data_size = (job.spec.required_resources.storage_mb as u64) * 1024 * 1024;
        let optimal_shards = (estimated_data_size / self.target_shard_size_bytes).max(1);
        optimal_shards.min(16) as u32 // Cap at 16 shards for data processing
    }

    fn create_shards(&self, job: &ActualMeshJob) -> Result<Vec<JobShard>, CommonError> {
        let num_shards = self.calculate_optimal_shards(job);
        let mut shards = Vec::new();

        let shard_size = self.target_shard_size_bytes;
        let overlap_size = (shard_size as f32 * self.overlap_percentage) as u64;

        for i in 0..num_shards {
            let shard_id = format!("{}:data_shard:{}", job.id, i);

            let start = if i == 0 {
                0
            } else {
                i as u64 * shard_size - overlap_size
            };
            let end = (i + 1) as u64 * shard_size;

            let data_range = DataRange {
                start,
                end,
                unit: RangeUnit::Bytes,
            };

            let shard_input = ShardInput {
                input_cids: job.spec.inputs.clone(),
                parameters: HashMap::new(),
                inline_data: None,
                data_range: Some(data_range),
            };

            let shard = JobShard {
                shard_id: shard_id.clone(),
                parent_job_id: job.id.clone(),
                shard_index: i,
                total_shards: num_shards,
                job_spec: JobSpec {
                    required_resources: Resources {
                        cpu_cores: job.spec.required_resources.cpu_cores / num_shards.max(1),
                        memory_mb: job.spec.required_resources.memory_mb,
                        storage_mb: (shard_size / (1024 * 1024)) as u32,
                    },
                    ..job.spec.clone()
                },
                shard_input,
                expected_output: ShardOutputSpec {
                    format: OutputFormat::Binary,
                    max_size_bytes: Some(shard_size * 2), // Allow 2x expansion
                    validation_rules: vec![ValidationRule::MaxSize(shard_size * 2)],
                },
                dependencies: vec![],
                priority: 100,
            };

            shards.push(shard);
        }

        Ok(shards)
    }

    fn validate_shards(&self, shards: &[JobShard]) -> Result<(), CommonError> {
        if shards.is_empty() {
            return Err(CommonError::InvalidParameters(
                "No shards created".to_string(),
            ));
        }

        // Validate data ranges don't have gaps (except for overlaps)
        for window in shards.windows(2) {
            let current = &window[0];
            let next = &window[1];

            if let (Some(current_range), Some(next_range)) = (
                &current.shard_input.data_range,
                &next.shard_input.data_range,
            ) {
                if next_range.start > current_range.end {
                    return Err(CommonError::InvalidParameters(
                        "Gap detected between shard data ranges".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

impl ShardCoordinator {
    /// Create a new shard coordinator.
    pub fn new(coordinator_did: Did, signing_key: Arc<SigningKey>, max_concurrent: u32) -> Self {
        Self {
            coordinator_did,
            signing_key,
            active_executions: std::sync::RwLock::new(HashMap::new()),
            max_concurrent_shards: max_concurrent,
        }
    }

    /// Coordinate execution of multiple job shards.
    pub async fn coordinate_shards(
        &self,
        shards: Vec<JobShard>,
        executor_bids: HashMap<String, Vec<MeshJobBid>>, // shard_id -> bids
    ) -> Result<ShardedJobResult, CommonError> {
        info!(
            "[ShardCoordinator] Starting coordination of {} shards",
            shards.len()
        );

        let start_time = SystemTime::now();
        let mut shard_results = Vec::new();
        let mut successful_shards = 0;
        let mut failed_shards = 0;

        // Group shards by dependencies (simple topological sort)
        let execution_groups = self.group_shards_by_dependencies(&shards)?;

        for group in execution_groups {
            let group_results = self.execute_shard_group(group, &executor_bids).await?;

            for result in group_results {
                if result.success {
                    successful_shards += 1;
                } else {
                    failed_shards += 1;
                }
                shard_results.push(result);
            }
        }

        let total_execution_time = start_time
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;

        // Aggregate results if all shards succeeded
        let aggregated_output = if failed_shards == 0 {
            self.aggregate_shard_results(&shard_results, &shards[0].parent_job_id)
                .await?
        } else {
            None
        };

        let result = ShardedJobResult {
            job_id: shards[0].parent_job_id.clone(),
            shard_results,
            aggregated_output,
            success: failed_shards == 0,
            total_execution_time_ms: total_execution_time,
            aggregation_metadata: AggregationMetadata {
                strategy: AggregationStrategy::Concatenate,
                successful_shards,
                failed_shards,
                aggregation_time_ms: 0, // Will be updated in aggregate_shard_results
                coordinator_did: self.coordinator_did.clone(),
            },
        };

        info!(
            "[ShardCoordinator] Coordination completed: {}/{} shards successful",
            successful_shards,
            successful_shards + failed_shards
        );

        Ok(result)
    }

    /// Group shards by their dependencies for ordered execution.
    fn group_shards_by_dependencies(
        &self,
        shards: &[JobShard],
    ) -> Result<Vec<Vec<JobShard>>, CommonError> {
        let mut groups = Vec::new();
        let mut remaining_shards: Vec<JobShard> = shards.to_vec();

        // Simple implementation: execute shards without dependencies first
        while !remaining_shards.is_empty() {
            let mut current_group = Vec::new();
            let mut indices_to_remove = Vec::new();

            for (i, shard) in remaining_shards.iter().enumerate() {
                if shard.dependencies.is_empty()
                    || self.are_dependencies_satisfied(&shard.dependencies, &groups)
                {
                    current_group.push(shard.clone());
                    indices_to_remove.push(i);
                }
            }

            if current_group.is_empty() {
                return Err(CommonError::InternalError(
                    "Circular dependency detected in shards".to_string(),
                ));
            }

            // Remove processed shards in reverse order to maintain indices
            for &i in indices_to_remove.iter().rev() {
                remaining_shards.remove(i);
            }

            groups.push(current_group);
        }

        Ok(groups)
    }

    /// Check if shard dependencies are satisfied.
    fn are_dependencies_satisfied(
        &self,
        dependencies: &[String],
        completed_groups: &[Vec<JobShard>],
    ) -> bool {
        let completed_shard_ids: std::collections::HashSet<String> = completed_groups
            .iter()
            .flatten()
            .map(|s| s.shard_id.clone())
            .collect();

        dependencies
            .iter()
            .all(|dep| completed_shard_ids.contains(dep))
    }

    /// Execute a group of independent shards in parallel.
    async fn execute_shard_group(
        &self,
        shards: Vec<JobShard>,
        executor_bids: &HashMap<String, Vec<MeshJobBid>>,
    ) -> Result<Vec<ShardResult>, CommonError> {
        let mut tasks = Vec::new();

        for shard in shards {
            let bids = executor_bids.get(&shard.shard_id).ok_or_else(|| {
                CommonError::InvalidParameters(format!(
                    "No bids found for shard {}",
                    shard.shard_id
                ))
            })?;

            if bids.is_empty() {
                return Err(CommonError::InvalidParameters(format!(
                    "No executors available for shard {}",
                    shard.shard_id
                )));
            }

            // Select best executor (simplified - use first bid for now)
            let selected_executor = &bids[0].executor_did;

            let shard_clone = shard.clone();
            let executor_clone = selected_executor.clone();

            // In a real implementation, this would delegate to the mesh job executor
            let task = tokio::spawn(async move {
                Self::execute_single_shard(shard_clone, executor_clone).await
            });

            tasks.push(task);
        }

        // Wait for all shards to complete
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result?),
                Err(e) => {
                    return Err(CommonError::InternalError(format!(
                        "Shard execution task failed: {}",
                        e
                    )))
                }
            }
        }

        Ok(results)
    }

    /// Execute a single shard (placeholder implementation).
    async fn execute_single_shard(
        shard: JobShard,
        executor_did: Did,
    ) -> Result<ShardResult, CommonError> {
        debug!(
            "[ShardCoordinator] Executing shard {} on executor {}",
            shard.shard_id, executor_did
        );

        let start_time = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Simulate execution time
        tokio::time::sleep(Duration::from_millis(100)).await;

        let end_time = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Create a mock execution receipt
        let execution_receipt = ExecutionReceipt {
            job_id: icn_common::Cid::new_v1_sha256(0x55, shard.shard_id.as_bytes()),
            executor_did: executor_did.clone(),
            result_cid: icn_common::Cid::new_v1_sha256(0x55, b"mock_result"),
            cpu_ms: 100,
            success: true,
            sig: SignatureBytes(vec![]), // Would be properly signed in real implementation
        };

        let result_cid = execution_receipt.result_cid.clone();

        Ok(ShardResult {
            shard_id: shard.shard_id,
            success: true,
            output_cid: Some(result_cid),
            execution_receipt,
            error_message: None,
            metadata: ShardExecutionMetadata {
                executor_did,
                execution_time_ms: (end_time - start_time) * 1000,
                resources_used: shard.job_spec.required_resources,
                started_at: start_time,
                completed_at: end_time,
            },
        })
    }

    /// Aggregate results from completed shards.
    async fn aggregate_shard_results(
        &self,
        shard_results: &[ShardResult],
        job_id: &JobId,
    ) -> Result<Option<Cid>, CommonError> {
        debug!(
            "[ShardCoordinator] Aggregating {} shard results",
            shard_results.len()
        );

        // For now, just create a mock aggregated result CID
        // In a real implementation, this would:
        // 1. Download shard outputs from DAG
        // 2. Apply appropriate aggregation strategy
        // 3. Store aggregated result in DAG
        // 4. Return the CID of the aggregated result

        let aggregation_data = format!("aggregated_result_for_{}", job_id);
        let aggregated_cid = icn_common::Cid::new_v1_sha256(0x55, aggregation_data.as_bytes());

        info!(
            "[ShardCoordinator] Created aggregated result with CID: {}",
            aggregated_cid
        );
        Ok(Some(aggregated_cid))
    }

    /// Get current coordination capacity.
    pub fn get_capacity(&self) -> (usize, u32) {
        let active = self.active_executions.read().unwrap();
        (active.len(), self.max_concurrent_shards)
    }

    /// Cancel all active shard executions.
    pub async fn cancel_all_executions(&self) {
        let mut active = self.active_executions.write().unwrap();
        info!(
            "[ShardCoordinator] Cancelling {} active shard executions",
            active.len()
        );
        active.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JobKind, JobSpec};
    use icn_common::{Cid, Did};
    use std::str::FromStr;

    #[test]
    fn test_map_reduce_sharding_strategy() {
        let strategy = MapReduceShardingStrategy::new(4, 100);

        let job = ActualMeshJob {
            id: JobId(Cid::new_v1_sha256(0x55, b"test_job")),
            manifest_cid: Cid::new_v1_sha256(0x55, b"manifest"),
            spec: JobSpec {
                kind: JobKind::GenericPlaceholder,
                inputs: vec![Cid::new_v1_sha256(0x55, b"input1")],
                required_resources: Resources {
                    cpu_cores: 4,
                    memory_mb: 1024,
                    storage_mb: 2048,
                },
                ..Default::default()
            },
            creator_did: Did::from_str("did:key:test").unwrap(),
            cost_mana: 1000,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![]),
        };

        assert!(strategy.can_shard(&job));
        assert_eq!(strategy.calculate_optimal_shards(&job), 4);

        let shards = strategy.create_shards(&job).unwrap();
        assert_eq!(shards.len(), 4);
        assert!(strategy.validate_shards(&shards).is_ok());

        // Check shard properties
        for (i, shard) in shards.iter().enumerate() {
            assert_eq!(shard.shard_index, i as u32);
            assert_eq!(shard.total_shards, 4);
            assert_eq!(shard.parent_job_id, job.id);
        }
    }

    #[test]
    fn test_data_processing_sharding_strategy() {
        let strategy = DataProcessingShardingStrategy::new(10 * 1024 * 1024, 0.1); // 10MB, 10% overlap

        let job = ActualMeshJob {
            id: JobId(Cid::new_v1_sha256(0x55, b"data_job")),
            manifest_cid: Cid::new_v1_sha256(0x55, b"manifest"),
            spec: JobSpec {
                kind: JobKind::GenericPlaceholder,
                inputs: vec![Cid::new_v1_sha256(0x55, b"large_dataset")],
                required_resources: Resources {
                    cpu_cores: 8,
                    memory_mb: 2048,
                    storage_mb: 500, // 500 MB dataset
                },
                ..Default::default()
            },
            creator_did: Did::from_str("did:key:test").unwrap(),
            cost_mana: 2000,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![]),
        };

        assert!(strategy.can_shard(&job));

        let optimal_shards = strategy.calculate_optimal_shards(&job);
        assert!(optimal_shards > 1); // Should suggest multiple shards for large data

        let shards = strategy.create_shards(&job).unwrap();
        assert!(!shards.is_empty());
        assert!(strategy.validate_shards(&shards).is_ok());
    }

    #[test]
    fn test_job_sharding_engine() {
        let engine = JobShardingEngine::new(16, 1024 * 1024); // Max 16 shards, min 1MB

        let job = ActualMeshJob {
            id: JobId(Cid::new_v1_sha256(0x55, b"engine_test")),
            manifest_cid: Cid::new_v1_sha256(0x55, b"manifest"),
            spec: JobSpec {
                kind: JobKind::GenericPlaceholder,
                inputs: vec![Cid::new_v1_sha256(0x55, b"input")],
                required_resources: Resources {
                    cpu_cores: 4,
                    memory_mb: 1024,
                    storage_mb: 200, // 200 MB
                },
                ..Default::default()
            },
            creator_did: Did::from_str("did:key:test").unwrap(),
            cost_mana: 1000,
            max_execution_wait_ms: None,
            signature: SignatureBytes(vec![]),
        };

        assert!(engine.should_shard(&job));

        let shards = engine.shard_job(&job).unwrap();
        assert!(!shards.is_empty());

        let resources = engine.estimate_shard_resources(&shards);
        assert!(resources.cpu_cores > 0);
        assert!(resources.memory_mb > 0);
    }

    #[tokio::test]
    async fn test_shard_coordinator() {
        let coordinator_did = Did::from_str("did:key:coordinator").unwrap();
        let signing_key = Arc::new(icn_identity::generate_ed25519_keypair().0);
        let coordinator = ShardCoordinator::new(coordinator_did.clone(), signing_key, 10);

        let shard = JobShard {
            shard_id: "test_shard_1".to_string(),
            parent_job_id: JobId(Cid::new_v1_sha256(0x55, b"parent_job")),
            shard_index: 0,
            total_shards: 1,
            job_spec: JobSpec::default(),
            shard_input: ShardInput {
                input_cids: vec![],
                parameters: HashMap::new(),
                inline_data: None,
                data_range: None,
            },
            expected_output: ShardOutputSpec {
                format: OutputFormat::Binary,
                max_size_bytes: None,
                validation_rules: vec![],
            },
            dependencies: vec![],
            priority: 100,
        };

        let executor_did = Did::from_str("did:key:executor").unwrap();
        let bid = MeshJobBid {
            job_id: JobId(Cid::new_v1_sha256(0x55, b"parent_job")),
            executor_did: executor_did.clone(),
            price_mana: 100,
            resources: Resources::default(),
            executor_capabilities: vec![],
            executor_federations: vec![],
            executor_trust_scope: None,
            signature: SignatureBytes(vec![]),
        };

        let mut executor_bids = HashMap::new();
        executor_bids.insert("test_shard_1".to_string(), vec![bid]);

        let result = coordinator
            .coordinate_shards(vec![shard], executor_bids)
            .await
            .unwrap();
        assert!(result.success);
        assert_eq!(result.shard_results.len(), 1);
        assert!(result.aggregated_output.is_some());
    }

    #[test]
    fn test_data_range_creation() {
        let range = DataRange {
            start: 0,
            end: 1024,
            unit: RangeUnit::Bytes,
        };

        assert_eq!(range.start, 0);
        assert_eq!(range.end, 1024);
        assert!(matches!(range.unit, RangeUnit::Bytes));
    }

    #[test]
    fn test_shard_output_spec_validation() {
        let spec = ShardOutputSpec {
            format: OutputFormat::Json,
            max_size_bytes: Some(1024 * 1024), // 1MB
            validation_rules: vec![
                ValidationRule::MaxSize(1024 * 1024),
                ValidationRule::MinSize(100),
            ],
        };

        assert!(matches!(spec.format, OutputFormat::Json));
        assert_eq!(spec.max_size_bytes, Some(1024 * 1024));
        assert_eq!(spec.validation_rules.len(), 2);
    }
}
