# InterCooperative Network Mesh Job Execution Protocol
## Definitive Specification

---

## Executive Summary

The Mesh Job Execution Protocol defines how computational work is distributed, executed, and verified across the InterCooperative Network. Unlike traditional cloud computing that centralizes control and profit, ICN's mesh computing enables **democratic resource sharing** where any node—from mobile phones to server farms—can contribute compute power and earn mana.

Jobs are posted with specifications and budgets, executors bid based on their capabilities, work is sandboxed and verified, and payment is automatic upon successful completion. Every step creates immutable DAG records, ensuring accountability while maintaining privacy through zero-knowledge proofs where needed.

---

## 1. Core Design Principles

### 1.1 Inclusive Participation
- Any node can be an executor, from phones to data centers
- Jobs scale from microseconds to days
- Accessibility for resource-limited devices through job sharding

### 1.2 Trust Through Verification
- Deterministic execution with reproducible results
- Multi-validator sampling for critical jobs
- Cryptographic proofs of completion

### 1.3 Economic Fairness
- Transparent bidding process
- Automatic payment on completion
- Slashing for failures protects submitters

### 1.4 Privacy-Preserving
- Encrypted job data when needed
- Zero-knowledge execution proofs
- Optional private bidding

---

## 2. Job Specification Format

### 2.1 Job Structure

```rust
pub struct JobSpec {
    // Identity
    id: JobId,                         // Unique identifier
    submitter: DID,                    // Who submitted the job
    
    // Execution requirements
    runtime: Runtime,                  // WASM, Docker, Native
    code: CodeSpec,                    // What to execute
    input: InputSpec,                  // Input data
    
    // Resource requirements
    compute: ComputeRequirements,      // CPU, RAM, GPU needs
    storage: StorageRequirements,      // Disk space needs
    network: NetworkRequirements,      // Bandwidth needs
    
    // Economic parameters
    budget: Budget,                    // Max payment
    payment_type: PaymentType,         // Mana, tokens, or both
    
    // Execution parameters
    deadline: Epoch,                   // Must complete by
    redundancy: RedundancyLevel,       // How many validators
    privacy: PrivacyLevel,             // Public, private, or ZK
    
    // Quality requirements
    accuracy: Option<AccuracySpec>,    // For ML/statistical jobs
    determinism: bool,                 // Must be reproducible
}

pub struct ComputeRequirements {
    min_cpu_cores: u32,
    min_memory_mb: u64,
    gpu_required: Option<GpuSpec>,
    architecture: Option<Architecture>,  // x86_64, arm64, etc.
    
    // Estimated resource consumption
    estimated_cpu_hours: f64,
    estimated_memory_hours: f64,
    estimated_gpu_hours: Option<f64>,
}

pub enum Runtime {
    WASM {
        module: CID,                   // WASM module in DAG
        function: String,               // Entry point
        memory_pages: u32,              // Memory limit
    },
    Docker {
        image: String,                  // Docker image
        tag: String,                    // Version tag
        entrypoint: Vec<String>,        // Command to run
    },
    Native {
        binary: CID,                    // Binary in DAG
        args: Vec<String>,              // Arguments
        env: HashMap<String, String>,   // Environment variables
    },
    CCL {
        contract: ContractAddress,      // CCL contract
        function: String,               // Function to call
        args: Bytes,                    // Encoded arguments
    },
}

pub struct Budget {
    max_total: Mana,                   // Maximum total payment
    per_unit: Option<PricingModel>,    // Per-unit pricing
    
    // Incentive structure
    early_completion_bonus: Option<Mana>,
    quality_bonus: Option<QualityBonus>,
}

pub enum PricingModel {
    PerHour(Mana),                     // Per CPU/GPU hour
    PerOperation(Mana),                // Per operation completed
    PerByte(Mana),                     // Per byte processed
    Fixed(Mana),                       // Fixed price
}
```

### 2.2 Job Categories

```rust
pub enum JobCategory {
    // Computation-intensive
    Rendering {
        frames: Vec<FrameSpec>,
        format: RenderFormat,
        quality: QualityLevel,
    },
    
    MachineLearning {
        task: MLTask,                  // Training, inference, etc.
        model: ModelSpec,
        dataset: DatasetSpec,
        metrics: Vec<MetricType>,
    },
    
    Scientific {
        simulation: SimulationType,
        parameters: Parameters,
        iterations: u64,
    },
    
    // Data-intensive
    DataProcessing {
        input_format: DataFormat,
        output_format: DataFormat,
        transformation: TransformSpec,
    },
    
    MapReduce {
        mapper: CodeSpec,
        reducer: CodeSpec,
        partitions: u32,
    },
    
    // Network-intensive
    WebScraping {
        urls: Vec<Url>,
        selectors: Vec<Selector>,
        rate_limit: RateLimit,
    },
    
    Federation {
        task: FederationTask,          // Consensus, validation, etc.
        participants: Vec<DID>,
        threshold: f64,
    },
    
    // Lightweight
    Verification {
        proof: ProofSpec,
        public_inputs: Vec<Field>,
    },
    
    Microtask {
        operations: Vec<MicroOp>,
        max_duration_ms: u64,
    },
}
```

---

## 3. Bidding & Assignment

### 3.1 Bidding Process

```rust
pub struct Bid {
    bidder: DID,
    job_id: JobId,
    
    // Offered terms
    price: Mana,
    completion_time: Epoch,
    
    // Executor capabilities
    compute_score: ComputeScore,
    availability: AvailabilityWindow,
    
    // Reputation
    reputation_score: f64,
    completed_similar: u32,
    success_rate: f64,
    
    // Proof of capability
    capability_proof: CapabilityProof,
    
    // Bid validity
    valid_until: Epoch,
    signature: Signature,
}

pub struct CapabilityProof {
    // Prove you can execute the job
    resource_attestation: ResourceAttestation,
    benchmark_results: Option<BenchmarkResults>,
    past_execution_cids: Vec<CID>,     // Similar jobs completed
}

pub struct BiddingProtocol {
    pub fn submit_bid(job: &JobSpec, bid: Bid) -> Result<()> {
        // 1. Validate bid
        require(bid.job_id == job.id);
        require(bid.price <= job.budget.max_total);
        require(bid.completion_time <= job.deadline);
        
        // 2. Verify capability
        require(verify_capability(&bid.capability_proof, &job.compute)?);
        
        // 3. Check reputation threshold
        if let Some(min_rep) = job.min_reputation {
            require(bid.reputation_score >= min_rep);
        }
        
        // 4. Lock bid stake (prevent spam)
        let stake = bid.price / 20;  // 5% stake
        lock_mana(&bid.bidder, stake, bid.valid_until)?;
        
        // 5. Record bid in DAG
        let bid_cid = put_dag(BidRecord {
            bid: bid.clone(),
            timestamp: now(),
            job_cid: job.to_cid(),
        })?;
        
        emit BidSubmitted(job.id, bid.bidder, bid_cid);
        Ok(())
    }
}
```

### 3.2 Assignment Algorithm

```rust
pub struct AssignmentAlgorithm {
    pub fn select_executor(job: &JobSpec, bids: Vec<Bid>) -> Result<DID> {
        if bids.is_empty() {
            return Err(Error::NoBids);
        }
        
        // Score each bid
        let scored_bids: Vec<(Bid, f64)> = bids.into_iter()
            .map(|bid| {
                let score = calculate_bid_score(&bid, job);
                (bid, score)
            })
            .collect();
        
        // Select winner based on job requirements
        let winner = match job.selection_strategy {
            SelectionStrategy::LowestPrice => {
                scored_bids.into_iter()
                    .min_by_key(|(bid, _)| bid.price)
                    .map(|(bid, _)| bid)
            },
            
            SelectionStrategy::FastestDelivery => {
                scored_bids.into_iter()
                    .min_by_key(|(bid, _)| bid.completion_time)
                    .map(|(bid, _)| bid)
            },
            
            SelectionStrategy::BestScore => {
                scored_bids.into_iter()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .map(|(bid, _)| bid)
            },
            
            SelectionStrategy::Random => {
                // Weighted random based on scores
                weighted_random_selection(scored_bids)
            },
        };
        
        Ok(winner.unwrap().bidder)
    }
    
    fn calculate_bid_score(bid: &Bid, job: &JobSpec) -> f64 {
        let price_score = (job.budget.max_total as f64 - bid.price as f64) / 
                         job.budget.max_total as f64;
        
        let time_score = (job.deadline as f64 - bid.completion_time as f64) /
                        (job.deadline - current_epoch()) as f64;
        
        let reputation_score = bid.reputation_score;
        
        let capability_score = calculate_capability_match(
            &bid.compute_score,
            &job.compute
        );
        
        // Weighted combination
        price_score * 0.3 +
        time_score * 0.2 +
        reputation_score * 0.3 +
        capability_score * 0.2
    }
}
```

---

## 4. Execution Environment

### 4.1 Sandboxing

```rust
pub struct ExecutionSandbox {
    runtime: Runtime,
    resource_limits: ResourceLimits,
    network_policy: NetworkPolicy,
    filesystem_policy: FilesystemPolicy,
}

pub struct ResourceLimits {
    max_cpu_time: Duration,
    max_memory: ByteSize,
    max_disk_io: ByteSize,
    max_network_io: ByteSize,
    max_processes: u32,
    max_file_descriptors: u32,
}

impl ExecutionSandbox {
    pub fn create_wasm_sandbox(job: &JobSpec) -> Result<WasmSandbox> {
        let sandbox = WasmSandbox {
            memory_limit: job.compute.min_memory_mb * 1_048_576,
            instruction_limit: job.compute.estimated_cpu_hours * 3_600_000_000_000, // ~1THz
            
            // Restricted imports
            allowed_imports: vec![
                "env.memory",
                "icn.get_input",
                "icn.put_output",
                "icn.log",
            ],
            
            // No system access
            allow_network: false,
            allow_filesystem: false,
            allow_random: false,  // Deterministic only
        };
        
        Ok(sandbox)
    }
    
    pub fn create_docker_sandbox(job: &JobSpec) -> Result<DockerSandbox> {
        let sandbox = DockerSandbox {
            // Resource limits
            cpu_shares: job.compute.min_cpu_cores * 1024,
            memory_limit: format!("{}m", job.compute.min_memory_mb),
            
            // Security options
            readonly_rootfs: true,
            no_new_privileges: true,
            drop_capabilities: vec!["ALL"],
            
            // Network isolation
            network_mode: match job.network {
                Some(_) => "bridge",
                None => "none",
            },
            
            // User namespace
            user: "nobody:nogroup",
        };
        
        Ok(sandbox)
    }
}
```

### 4.2 Execution Monitoring

```rust
pub struct ExecutionMonitor {
    job_id: JobId,
    executor: DID,
    start_time: Timestamp,
    
    pub fn monitor_execution(&mut self) -> Result<ExecutionStatus> {
        loop {
            let metrics = self.collect_metrics()?;
            
            // Check resource usage
            if metrics.cpu_usage > self.limits.max_cpu_time {
                return Ok(ExecutionStatus::Failed(FailureReason::CpuTimeout));
            }
            
            if metrics.memory_usage > self.limits.max_memory {
                return Ok(ExecutionStatus::Failed(FailureReason::OutOfMemory));
            }
            
            // Check progress
            if let Some(progress) = self.get_progress()? {
                self.report_progress(progress)?;
                
                if progress.completed {
                    return Ok(ExecutionStatus::Completed);
                }
            }
            
            // Check deadline
            if current_time() > self.deadline {
                return Ok(ExecutionStatus::Failed(FailureReason::DeadlineExceeded));
            }
            
            sleep(Duration::from_secs(1));
        }
    }
    
    fn collect_metrics(&self) -> Result<ExecutionMetrics> {
        ExecutionMetrics {
            cpu_usage: read_cpu_time(self.process_id)?,
            memory_usage: read_memory_usage(self.process_id)?,
            disk_io: read_disk_io(self.process_id)?,
            network_io: read_network_io(self.process_id)?,
        }
    }
}
```

---

## 5. Result Validation

### 5.1 Deterministic Validation

```rust
pub struct ResultValidation {
    pub fn validate_result(
        job: &JobSpec,
        result: &ExecutionResult,
        validators: Vec<ValidatorId>
    ) -> Result<ValidationOutcome> {
        
        match job.redundancy {
            RedundancyLevel::None => {
                // Trust single executor
                Ok(ValidationOutcome::Accepted)
            },
            
            RedundancyLevel::Sampling(n) => {
                // Random validators re-execute samples
                let samples = select_random_samples(&result.output, n);
                let validations = validators.iter()
                    .map(|v| v.validate_sample(&samples))
                    .collect::<Vec<_>>();
                
                if validations.iter().filter(|v| v.is_valid()).count() >= n * 2 / 3 {
                    Ok(ValidationOutcome::Accepted)
                } else {
                    Ok(ValidationOutcome::Rejected)
                }
            },
            
            RedundancyLevel::Full(n) => {
                // Multiple executors run full job
                let executions = run_redundant_executions(&job, n)?;
                
                // Compare results
                if all_results_match(&executions) {
                    Ok(ValidationOutcome::Accepted)
                } else {
                    // Majority wins
                    let majority = find_majority_result(&executions);
                    Ok(ValidationOutcome::AcceptedWithDispute(majority))
                }
            },
            
            RedundancyLevel::ZeroKnowledge => {
                // Verify ZK proof without seeing data
                let proof = result.zk_proof.as_ref()
                    .ok_or(Error::MissingProof)?;
                
                if verify_zk_execution_proof(proof, &job.spec_commitment) {
                    Ok(ValidationOutcome::Accepted)
                } else {
                    Ok(ValidationOutcome::Rejected)
                }
            },
        }
    }
}

pub struct ExecutionResult {
    job_id: JobId,
    executor: DID,
    
    // Output data
    output: OutputData,
    output_cid: CID,               // Stored in DAG
    
    // Execution proof
    execution_trace: ExecutionTrace,
    resource_usage: ResourceUsage,
    
    // Optional proofs
    determinism_proof: Option<DeterminismProof>,
    zk_proof: Option<ZKExecutionProof>,
    
    // Metadata
    start_time: Timestamp,
    end_time: Timestamp,
    exit_code: i32,
}
```

### 5.2 Dispute Resolution

```rust
pub struct DisputeProtocol {
    pub fn raise_dispute(
        job_id: JobId,
        disputer: DID,
        evidence: DisputeEvidence
    ) -> Result<DisputeId> {
        // 1. Verify disputer has standing
        require(is_job_participant(&disputer, &job_id) || 
                is_validator(&disputer));
        
        // 2. Lock dispute stake
        let stake = get_job_value(&job_id) / 10;  // 10% of job value
        lock_mana(&disputer, stake, current_epoch() + DISPUTE_PERIOD)?;
        
        // 3. Create dispute
        let dispute = Dispute {
            id: generate_id(),
            job_id,
            disputer,
            disputed_result: get_job_result(&job_id)?,
            evidence,
            status: DisputeStatus::Open,
        };
        
        // 4. Trigger re-execution by validators
        let validators = select_dispute_validators(5);
        for validator in &validators {
            schedule_validation(validator, &job_id)?;
        }
        
        // 5. Record in DAG
        let dispute_cid = put_dag(&dispute)?;
        emit DisputeRaised(dispute.id, job_id, dispute_cid);
        
        Ok(dispute.id)
    }
    
    pub fn resolve_dispute(dispute_id: DisputeId) -> Result<DisputeResolution> {
        let dispute = get_dispute(&dispute_id)?;
        let validations = get_validation_results(&dispute.job_id)?;
        
        // Majority decision
        let (valid, invalid): (Vec<_>, Vec<_>) = validations.into_iter()
            .partition(|v| v.is_valid);
        
        let resolution = if valid.len() > invalid.len() {
            // Original result stands
            DisputeResolution::Rejected {
                reason: "Majority validators confirm original result",
                penalty: dispute.stake,  // Disputer loses stake
            }
        } else {
            // Original result overturned
            DisputeResolution::Accepted {
                new_result: compute_consensus_result(&valid),
                compensation: dispute.stake * 2,  // Disputer rewarded
                executor_penalty: get_job_value(&dispute.job_id),
            }
        };
        
        apply_resolution(&resolution)?;
        emit DisputeResolved(dispute_id, resolution);
        
        Ok(resolution)
    }
}
```

---

## 6. Payment & Settlement

### 6.1 Payment Flow

```rust
pub struct PaymentProtocol {
    pub fn process_payment(
        job: &JobSpec,
        result: &ExecutionResult,
        validation: &ValidationOutcome
    ) -> Result<PaymentReceipt> {
        
        match validation {
            ValidationOutcome::Accepted => {
                // Calculate payment
                let base_payment = calculate_base_payment(job, result)?;
                let bonuses = calculate_bonuses(job, result)?;
                let total = base_payment + bonuses;
                
                // Transfer payment
                transfer_mana(&job.submitter, &result.executor, total)?;
                
                // Update reputation
                increase_reputation(&result.executor, 0.01)?;
                
                // Create receipt
                let receipt = PaymentReceipt {
                    job_id: job.id,
                    executor: result.executor.clone(),
                    amount: total,
                    timestamp: now(),
                    status: PaymentStatus::Completed,
                };
                
                emit PaymentCompleted(job.id, result.executor, total);
                Ok(receipt)
            },
            
            ValidationOutcome::Rejected => {
                // Slash executor
                let stake = get_executor_stake(&job.id)?;
                slash_mana(&result.executor, stake)?;
                
                // Refund submitter
                refund_mana(&job.submitter, job.budget.max_total)?;
                
                // Decrease reputation
                decrease_reputation(&result.executor, 0.05)?;
                
                let receipt = PaymentReceipt {
                    job_id: job.id,
                    executor: result.executor.clone(),
                    amount: 0,
                    timestamp: now(),
                    status: PaymentStatus::Failed,
                };
                
                emit PaymentFailed(job.id, result.executor);
                Ok(receipt)
            },
            
            ValidationOutcome::AcceptedWithDispute(consensus) => {
                // Partial payment based on consensus
                let payment = calculate_disputed_payment(job, consensus)?;
                transfer_mana(&job.submitter, &result.executor, payment)?;
                
                // Small reputation penalty for dispute
                decrease_reputation(&result.executor, 0.01)?;
                
                let receipt = PaymentReceipt {
                    job_id: job.id,
                    executor: result.executor.clone(),
                    amount: payment,
                    timestamp: now(),
                    status: PaymentStatus::Disputed,
                };
                
                emit PaymentDisputed(job.id, result.executor, payment);
                Ok(receipt)
            },
        }
    }
}
```

### 6.2 Incentive Mechanisms

```rust
pub struct IncentiveMechanisms {
    // Early completion bonus
    pub fn calculate_early_bonus(job: &JobSpec, result: &ExecutionResult) -> Mana {
        if let Some(bonus) = job.budget.early_completion_bonus {
            let time_saved = job.deadline - result.end_time;
            let time_window = job.deadline - job.created_at;
            let bonus_rate = time_saved as f64 / time_window as f64;
            (bonus as f64 * bonus_rate) as Mana
        } else {
            0
        }
    }
    
    // Quality bonus for ML/statistical jobs
    pub fn calculate_quality_bonus(job: &JobSpec, result: &ExecutionResult) -> Mana {
        if let Some(accuracy_spec) = &job.accuracy {
            if let Some(achieved) = result.metrics.get("accuracy") {
                if achieved > &accuracy_spec.target {
                    let excess = achieved - accuracy_spec.target;
                    (accuracy_spec.bonus_per_percent * excess * 100.0) as Mana
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    }
    
    // Reputation multiplier
    pub fn apply_reputation_multiplier(base: Mana, reputation: f64) -> Mana {
        let multiplier = 1.0 + (reputation - 0.5).max(0.0) * 0.2;  // Up to 20% bonus
        (base as f64 * multiplier) as Mana
    }
}
```

---

## 7. Failure Handling & Retries

### 7.1 Failure Recovery

```rust
pub enum FailureReason {
    // Executor failures
    OutOfMemory,
    CpuTimeout,
    DiskFull,
    NetworkError,
    CrashOrPanic,
    
    // Job failures
    InvalidInput,
    UnsupportedOperation,
    DeterminismViolation,
    
    // System failures
    ValidatorUnavailable,
    DAGWriteFailure,
    ManaInsufficient,
}

pub struct FailureHandler {
    pub fn handle_failure(
        job: &JobSpec,
        failure: FailureReason,
        executor: DID
    ) -> Result<RecoveryAction> {
        
        match failure {
            // Retryable failures
            FailureReason::NetworkError |
            FailureReason::ValidatorUnavailable |
            FailureReason::DAGWriteFailure => {
                Ok(RecoveryAction::Retry {
                    delay: Duration::from_secs(60),
                    max_attempts: 3,
                })
            },
            
            // Re-assign to different executor
            FailureReason::OutOfMemory |
            FailureReason::CpuTimeout |
            FailureReason::DiskFull |
            FailureReason::CrashOrPanic => {
                // Slash partial stake
                slash_mana(&executor, get_stake(&job.id) / 2)?;
                
                Ok(RecoveryAction::Reassign {
                    exclude_executor: executor,
                    increase_requirements: true,
                })
            },
            
            // Job problem - refund
            FailureReason::InvalidInput |
            FailureReason::UnsupportedOperation => {
                refund_mana(&job.submitter, job.budget.max_total)?;
                
                Ok(RecoveryAction::Cancel {
                    reason: format!("Job cannot be executed: {:?}", failure),
                    refund: true,
                })
            },
            
            // Serious failure - investigate
            FailureReason::DeterminismViolation => {
                Ok(RecoveryAction::Investigate {
                    validators: select_validators(3),
                    freeze_executor: true,
                })
            },
            
            _ => Ok(RecoveryAction::Cancel {
                reason: format!("Unrecoverable failure: {:?}", failure),
                refund: false,
            })
        }
    }
}
```

### 7.2 Retry Logic

```rust
pub struct RetryManager {
    max_retries: u32,
    backoff_strategy: BackoffStrategy,
    
    pub fn retry_job(
        job: &JobSpec,
        attempt: u32,
        last_failure: FailureReason
    ) -> Result<()> {
        
        if attempt >= self.max_retries {
            return Err(Error::MaxRetriesExceeded);
        }
        
        // Calculate delay
        let delay = match self.backoff_strategy {
            BackoffStrategy::Fixed(d) => d,
            BackoffStrategy::Linear(base) => base * attempt,
            BackoffStrategy::Exponential(base) => base * 2_u32.pow(attempt),
            BackoffStrategy::Jittered(base) => {
                base * 2_u32.pow(attempt) + random_jitter()
            },
        };
        
        // Wait before retry
        sleep(delay);
        
        // Adjust job parameters based on failure
        let adjusted_job = adjust_job_spec(job, &last_failure)?;
        
        // Re-submit for bidding
        resubmit_job(&adjusted_job)?;
        
        emit JobRetried(job.id, attempt, last_failure);
        Ok(())
    }
    
    fn adjust_job_spec(job: &JobSpec, failure: &FailureReason) -> Result<JobSpec> {
        let mut adjusted = job.clone();
        
        match failure {
            FailureReason::OutOfMemory => {
                adjusted.compute.min_memory_mb *= 2;
            },
            FailureReason::CpuTimeout => {
                adjusted.compute.estimated_cpu_hours *= 1.5;
                adjusted.deadline += 3600;  // Add 1 hour
            },
            FailureReason::DiskFull => {
                adjusted.storage.min_disk_mb *= 2;
            },
            _ => {},
        }
        
        Ok(adjusted)
    }
}
```

---

## 8. Specialized Execution Modes

### 8.1 Sharded Execution

```rust
pub struct ShardedExecution {
    pub fn shard_job(job: &JobSpec) -> Result<Vec<JobShard>> {
        match &job.category {
            JobCategory::MapReduce { partitions, .. } => {
                // Natural sharding for map-reduce
                create_map_reduce_shards(job, *partitions)
            },
            
            JobCategory::Rendering { frames, .. } => {
                // Shard by frame ranges
                create_frame_shards(job, frames)
            },
            
            JobCategory::DataProcessing { .. } => {
                // Shard by data chunks
                create_data_shards(job)
            },
            
            _ => Err(Error::NotShardable),
        }
    }
    
    pub fn coordinate_shards(
        shards: Vec<JobShard>,
        coordinators: Vec<DID>
    ) -> Result<ShardedResult> {
        // Submit shards in parallel
        let shard_futures: Vec<_> = shards.iter()
            .map(|shard| submit_shard(shard))
            .collect();
        
        // Wait for completion
        let shard_results = join_all(shard_futures).await?;
        
        // Combine results
        let combined = combine_shard_results(shard_results)?;
        
        Ok(combined)
    }
}
```

### 8.2 Private Execution

```rust
pub struct PrivateExecution {
    pub fn execute_private_job(
        job: &PrivateJobSpec,
        executor: DID
    ) -> Result<PrivateResult> {
        
        // 1. Establish secure channel
        let channel = establish_secure_channel(&job.submitter, &executor)?;
        
        // 2. Transfer encrypted input
        let encrypted_input = encrypt_with_shared_key(
            &job.input,
            &channel.shared_key
        );
        channel.send(encrypted_input)?;
        
        // 3. Execute in TEE if available
        let result = if has_tee_support() {
            execute_in_tee(&job.code, &encrypted_input)?
        } else {
            // Execute with homomorphic encryption
            execute_homomorphic(&job.code, &encrypted_input)?
        };
        
        // 4. Generate ZK proof of correct execution
        let proof = generate_execution_proof(
            &job.code,
            &result.commitment,
            &result.trace
        )?;
        
        // 5. Return encrypted result with proof
        Ok(PrivateResult {
            encrypted_output: result.encrypted_output,
            proof,
            executor_attestation: sign(&result.commitment, &executor.key),
        })
    }
}
```

### 8.3 Federated Learning

```rust
pub struct FederatedLearning {
    pub fn coordinate_training(
        model: ModelSpec,
        participants: Vec<DID>,
        rounds: u32
    ) -> Result<TrainedModel> {
        
        let mut global_model = initialize_model(&model)?;
        
        for round in 0..rounds {
            // 1. Distribute current model
            let model_cid = put_dag(&global_model)?;
            
            // 2. Local training jobs
            let local_jobs: Vec<_> = participants.iter()
                .map(|p| create_local_training_job(p, &model_cid))
                .collect();
            
            // 3. Execute training
            let updates = execute_parallel_jobs(local_jobs).await?;
            
            // 4. Aggregate updates (no raw data shared)
            let aggregated = federated_averaging(updates)?;
            
            // 5. Update global model
            global_model = apply_updates(&global_model, &aggregated)?;
            
            // 6. Validate improvement
            if !has_improved(&global_model, round) {
                break;
            }
            
            emit FederatedRoundComplete(round, participants.len());
        }
        
        Ok(global_model)
    }
}
```

---

## 9. Monitoring & Observability

### 9.1 Job Metrics

```rust
pub struct JobMetrics {
    // Submission metrics
    jobs_submitted: Counter,
    jobs_by_category: HashMap<JobCategory, Counter>,
    average_budget: Gauge,
    
    // Execution metrics
    jobs_completed: Counter,
    jobs_failed: Counter,
    average_execution_time: Histogram,
    resource_utilization: Gauge,
    
    // Economic metrics
    total_mana_spent: Counter,
    average_price_per_cpu_hour: Gauge,
    
    // Quality metrics
    validation_success_rate: Gauge,
    dispute_rate: Gauge,
    average_reputation_change: Gauge,
}

pub struct ExecutorMetrics {
    // Capacity metrics
    available_cpu_cores: Gauge,
    available_memory_gb: Gauge,
    available_gpu_units: Gauge,
    
    // Performance metrics
    jobs_executed: Counter,
    success_rate: Gauge,
    average_completion_time: Histogram,
    
    // Economic metrics
    mana_earned: Counter,
    reputation_score: Gauge,
    stake_locked: Gauge,
}
```

### 9.2 Health Monitoring

```rust
pub struct HealthMonitor {
    pub fn check_job_health(job_id: JobId) -> HealthStatus {
        let job = get_job(&job_id)?;
        let executor = get_executor(&job_id)?;
        
        // Check progress
        let progress = get_job_progress(&job_id)?;
        if progress.last_update > Duration::from_secs(300) {
            return HealthStatus::Stalled;
        }
        
        // Check resource usage
        let usage = get_resource_usage(&job_id)?;
        if usage.memory > job.compute.min_memory_mb * 0.9 {
            return HealthStatus::Warning("High memory usage");
        }
        
        // Check deadline
        let time_remaining = job.deadline - current_epoch();
        let estimated_remaining = estimate_time_to_complete(&progress);
        if estimated_remaining > time_remaining {
            return HealthStatus::Warning("May miss deadline");
        }
        
        HealthStatus::Healthy
    }
}
```

---

## 10. Implementation Roadmap

### 10.1 Phase 1: Core Job System (Months 1-2)
- [ ] Job specification format
- [ ] Basic submission and bidding
- [ ] Simple WASM execution
- [ ] Direct payment flow

### 10.2 Phase 2: Sandboxing & Security (Months 3-4)
- [ ] WASM sandbox implementation
- [ ] Docker container support
- [ ] Resource monitoring
- [ ] Basic validation

### 10.3 Phase 3: Advanced Features (Months 5-6)
- [ ] Sharded execution
- [ ] Private/ZK execution
- [ ] Federated learning
- [ ] Dispute resolution

### 10.4 Phase 4: Production Hardening (Months 7-8)
- [ ] Performance optimization
- [ ] Comprehensive monitoring
- [ ] Failure recovery
- [ ] Load testing

---

## Appendix A: Job Specification Examples

```json
{
  "example_rendering_job": {
    "id": "job_001",
    "runtime": {
      "type": "Docker",
      "image": "blender/blender",
      "tag": "3.6",
      "entrypoint": ["blender", "-b", "scene.blend", "-f", "1:100"]
    },
    "compute": {
      "min_cpu_cores": 8,
      "min_memory_mb": 16384,
      "gpu_required": {
        "type": "NVIDIA",
        "min_vram_gb": 8
      }
    },
    "budget": {
      "max_total": 5000,
      "per_unit": {
        "type": "PerFrame",
        "amount": 50
      }
    },
    "deadline": 1735689600,
    "redundancy": "Sampling(3)"
  },
  
  "example_ml_training_job": {
    "id": "job_002",
    "runtime": {
      "type": "WASM",
      "module": "bafybeig...",
      "function": "train_model"
    },
    "compute": {
      "min_memory_mb": 32768,
      "gpu_required": {
        "type": "Any",
        "min_vram_gb": 16
      },
      "estimated_gpu_hours": 24
    },
    "privacy": "ZeroKnowledge",
    "accuracy": {
      "target": 0.95,
      "bonus_per_percent": 100
    }
  }
}
```

---

## Appendix B: Error Codes

| Code | Error | Recovery |
|------|-------|----------|
| J001 | InvalidJobSpec | Fix specification |
| J002 | InsufficientBudget | Increase budget |
| J003 | NoExecutorAvailable | Wait or adjust requirements |
| J004 | ExecutionTimeout | Retry with longer deadline |
| J005 | ValidationFailed | Check determinism |
| J006 | DisputeRaised | Await resolution |
| J007 | PaymentFailed | Check mana balance |
| J008 | SandboxViolation | Review security policy |
| J009 | ResourceExhausted | Increase limits |
| J010 | DeadlineExceeded | Extend deadline |

---

*This completes the Mesh Job Execution Protocol specification. The system enables democratic, distributed computation where any node can contribute resources and earn fair compensation.*

**Protocol Status**: DEFINITIVE  
**Dependencies**: DAG Protocol, Economic Protocol, CCL Protocol  
**Implementation Complexity**: High (sandboxing, validation, distributed execution)  
**Estimated Development**: 8 months for full implementation