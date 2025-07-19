# ICN Mesh (`icn-mesh`) - Distributed Computing Engine

> **A comprehensive distributed computing platform that enables secure, federated job execution across cooperative networks**

## Overview

The `icn-mesh` crate implements ICN's distributed computing capabilities, providing a complete job orchestration system where nodes can submit work to be executed across a network of trusted executors. It combines economic incentives, reputation systems, and cryptographic security to create a robust mesh computing platform.

**Key Principle**: All job execution is cryptographically verified, economically incentivized, and reputation-based to ensure reliable distributed computing.

## Core Architecture

### 🆔 Job Identification
```rust
/// Unique identifier for mesh jobs, wrapping a CID for type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub Cid);
```

**Features:**
- **Type Safety**: Prevents CID/JobId confusion
- **Content Addressing**: Deterministic identification
- **Display**: Human-readable string representation
- **Serialization**: Full serde support

### 💼 Job Types

#### ActualMeshJob - Network Job Format
```rust
pub struct ActualMeshJob {
    pub id: JobId,
    pub manifest_cid: Cid,
    pub spec: JobSpec,
    pub creator_did: Did,
    pub cost_mana: u64,
    pub max_execution_wait_ms: Option<u64>,
    pub signature: SignatureBytes,
}
```

**Purpose**: The format used for network transmission and immediate processing.

**Security Features:**
- **Cryptographic Signing**: Ed25519 signatures over canonical bytes
- **Tamper Detection**: Signature verification prevents modification
- **Identity Binding**: Tied to creator's DID

#### Job - DAG Storage Format
```rust
pub struct Job {
    pub id: JobId,
    pub manifest_cid: Cid,
    pub spec_bytes: Vec<u8>,           // Binary-encoded JobSpec
    pub spec_json: Option<String>,     // Deprecated JSON format
    pub submitter_did: Did,
    pub cost_mana: u64,
    pub submitted_at: u64,
    pub status: JobLifecycleStatus,
    pub resource_requirements: Resources,
}
```

**Purpose**: Persistent storage format in the DAG for lifecycle tracking.

**Backward Compatibility:**
- **Binary Encoding**: Primary format using bincode
- **JSON Fallback**: Legacy support for existing jobs
- **Decode Method**: `job.decode_spec()` handles both formats

### 📋 Job Specifications

#### JobSpec - Execution Requirements
```rust
pub struct JobSpec {
    pub kind: JobKind,
    pub inputs: Vec<Cid>,
    pub outputs: Vec<String>,
    pub required_resources: Resources,
    pub required_capabilities: Vec<String>,
    pub required_trust_scope: Option<String>,
    pub min_executor_reputation: Option<u64>,
    pub allowed_federations: Vec<String>,
}
```

#### JobKind - Job Types
```rust
pub enum JobKind {
    /// Simple echo job for testing
    Echo { payload: String },
    
    /// Execute CCL WASM module (auto-executed by runtime)
    CclWasm,
    
    /// Generic placeholder for extension
    GenericPlaceholder,
}
```

**CCL WASM Integration:**
- **Auto-Execution**: Runtime immediately executes CclWasm jobs
- **DAG Loading**: WASM bytes loaded from manifest_cid
- **Built-in Executor**: Uses runtime's WASM execution engine

#### Resources - Computational Requirements
```rust
pub struct Resources {
    pub cpu_cores: u32,      // Number of CPU cores
    pub memory_mb: u32,      // Memory in megabytes
    pub storage_mb: u32,     // Storage in megabytes
}
```

## 💰 Bidding System

### Bid Structure
```rust
pub struct MeshJobBid {
    pub job_id: JobId,
    pub executor_did: Did,
    pub price_mana: u64,
    pub resources: Resources,
    pub executor_capabilities: Vec<String>,
    pub executor_federations: Vec<String>,
    pub executor_trust_scope: Option<String>,
    pub signature: SignatureBytes,
}
```

### Bid Submission
```rust
pub struct MeshBidSubmit {
    pub bid: MeshJobBid,
    pub signature: SignatureBytes,  // Additional signature for transmission
}
```

**Double Signing:**
1. **Bid Signature**: Signs the bid content itself
2. **Submission Signature**: Signs the submission for network transmission

### Bid Scoring Algorithm
The `select_executor` function uses a sophisticated scoring system:

```rust
pub struct SelectionPolicy {
    pub weight_price: f64,       // 1.0 - Inverse price weight
    pub weight_reputation: f64,  // 50.0 - Reputation importance  
    pub weight_resources: f64,   // 1.0 - Resource capacity weight
    pub weight_latency: f64,     // 1.0 - Network latency weight
}
```

**Scoring Formula:**
```rust
score = (weight_price / normalized_price) + 
        (weight_reputation * normalized_reputation) + 
        (weight_resources * normalized_resources) + 
        (weight_latency / normalized_latency)
```

**Selection Process:**
1. **Filter Bids**: Remove bids that don't meet requirements
2. **Capability Checking**: Verify executor capabilities
3. **Federation Constraints**: Ensure federation membership
4. **Trust Validation**: Check trust scope compatibility
5. **Score Calculation**: Apply weighted scoring algorithm
6. **Best Selection**: Choose highest-scoring valid bid

## 🔄 Job Lifecycle Management

### Lifecycle Status
```rust
pub enum JobLifecycleStatus {
    Submitted,      // Job received, awaiting bids
    BiddingOpen,    // Collecting bids from executors
    BiddingClosed,  // Bidding complete, selecting executor
    Assigned,       // Executor selected and notified
    Executing,      // Job running on executor
    Completed,      // Job finished successfully
    Failed,         // Job execution failed
    Cancelled,      // Job cancelled before completion
}
```

**Status Transitions:**
```
Submitted → BiddingOpen → BiddingClosed → Assigned → Executing → Completed/Failed
                                                                     ↓
                                                                 Cancelled
```

### Complete Lifecycle Tracking
```rust
pub struct JobLifecycle {
    pub job: Job,
    pub bids: Vec<JobBid>,
    pub assignment: Option<JobAssignment>,
    pub receipt: Option<JobReceipt>,
    pub checkpoints: Vec<JobCheckpoint>,
    pub partial_outputs: Vec<PartialOutputReceipt>,
}
```

**Lifecycle Methods:**
- **`current_status()`**: Derive status from available lifecycle events
- **`is_long_running()`**: Check if job has checkpoints/partial outputs
- **`latest_checkpoint()`**: Get most recent execution checkpoint
- **`current_progress()`**: Calculate completion percentage

### DAG-Based Storage
All lifecycle events are stored as separate DAG blocks:

1. **Job Submission**: Initial job block
2. **Bid Collection**: Individual bid blocks
3. **Assignment**: Executor selection block
4. **Execution Progress**: Checkpoint blocks
5. **Partial Outputs**: Intermediate result blocks
6. **Final Receipt**: Completion block

## 🏃‍♂️ Long-Running Job Support

### Job Checkpoints
```rust
pub struct JobCheckpoint {
    pub job_id: JobId,
    pub checkpoint_id: String,
    pub timestamp: u64,
    pub stage: String,
    pub progress_percent: f32,
    pub execution_state: Vec<u8>,
    pub intermediate_data_cid: Option<Cid>,
    pub executor_did: Did,
    pub signature: SignatureBytes,
}
```

**Features:**
- **Resumability**: Save execution state for recovery
- **Progress Tracking**: Percentage completion monitoring
- **State Persistence**: Serialized execution context
- **Intermediate Results**: CID references to partial data
- **Cryptographic Integrity**: Signed by executor

### Partial Output Receipts
```rust
pub struct PartialOutputReceipt {
    pub job_id: JobId,
    pub output_id: String,
    pub stage: String,
    pub timestamp: u64,
    pub output_cid: Cid,
    pub output_size: u64,
    pub output_format: Option<String>,
    pub executor_did: Did,
    pub signature: SignatureBytes,
}
```

**Use Cases:**
- **Streaming Results**: Large jobs producing incremental outputs
- **Pipeline Processing**: Multi-stage job execution
- **Quality Assurance**: Intermediate verification points
- **Early Access**: Use partial results before job completion

### Progress Reporting
```rust
pub struct ProgressReport {
    pub job_id: JobId,
    pub current_stage: String,
    pub progress_percent: f32,
    pub eta_seconds: Option<u64>,
    pub message: String,
    pub timestamp: u64,
    pub executor_did: Did,
    pub completed_stages: Vec<String>,
    pub remaining_stages: Vec<String>,
}
```

## 🤝 Mutual Aid Integration

### Aid Request System
```rust
pub struct AidRequest {
    pub id: String,
    pub requester: Did,
    pub tags: Vec<String>,    // Resource tags like "food", "shelter", "compute"
}

pub struct AidJobTemplate {
    pub tags: Vec<String>,    // Tags this template can handle
    pub job: JobSpec,         // Job to execute when matched
}
```

**Aid Matching:**
```rust
// Match requests with templates by tag overlap
pub fn match_aid_requests(
    requests: &[AidRequest],
    templates: &[AidJobTemplate],
) -> Vec<(&AidRequest, &AidJobTemplate)>

// Integration with DAG-stored mutual aid registry
pub fn match_registry_requests<S: StorageService<DagBlock>>(
    registry: &MutualAidRegistry<S>,
    templates: &[AidJobTemplate],
) -> Result<Vec<(AidResource, &AidJobTemplate)>, CommonError>
```

**Community Features:**
- **Tag-Based Matching**: Flexible resource categorization
- **Template System**: Reusable job patterns for common aid
- **Registry Integration**: Persistent aid request storage
- **Automatic Execution**: Aid jobs triggered by matching

## 🛡️ Security & Trust

### Federation Security
```rust
// Job specification security fields
pub struct JobSpec {
    pub required_trust_scope: Option<String>,      // Trust requirements
    pub min_executor_reputation: Option<u64>,      // Reputation threshold
    pub allowed_federations: Vec<String>,          // Federation whitelist
    // ...
}

// Bid security fields  
pub struct MeshJobBid {
    pub executor_capabilities: Vec<String>,        // Advertised capabilities
    pub executor_federations: Vec<String>,         // Federation memberships
    pub executor_trust_scope: Option<String>,      // Trust scope coverage
    // ...
}
```

### Cryptographic Verification
All major operations include cryptographic signatures:

```rust
// Job signing
let signed_job = job.sign(&creator_signing_key)?;
signed_job.verify_signature(&creator_verifying_key)?;

// Bid signing
let signed_bid = bid.sign(&executor_signing_key)?;
signed_bid.verify_signature(&executor_verifying_key)?;

// Assignment signing  
let signed_assignment = assignment.sign(&manager_signing_key)?;
signed_assignment.verify_signature(&manager_verifying_key)?;
```

### Capability Checking
```rust
// Executor must provide all required capabilities
let has_capabilities = job_spec.required_capabilities
    .iter()
    .all(|req| bid.executor_capabilities.contains(req));

// Federation membership validation
let in_allowed_federation = job_spec.allowed_federations.is_empty() || 
    job_spec.allowed_federations
        .iter()
        .any(|fed| bid.executor_federations.contains(fed));
```

## 📊 Metrics & Monitoring

### Core Metrics
```rust
// Job submission metrics
pub static JOBS_SUBMITTED_TOTAL: Counter;
pub static JOBS_COMPLETED_TOTAL: Counter;
pub static JOBS_FAILED_TOTAL: Counter;
pub static JOBS_ASSIGNED_TOTAL: Counter;

// Bidding metrics
pub static BIDS_RECEIVED_TOTAL: Counter;
pub static SELECT_EXECUTOR_CALLS: Counter;

// State tracking gauges
pub static PENDING_JOBS_GAUGE: Gauge<i64>;
pub static JOBS_BIDDING_GAUGE: Gauge<i64>;
pub static JOBS_EXECUTING_GAUGE: Gauge<i64>;

// Performance histograms
pub static JOB_PROCESS_TIME: Histogram;      // Assignment to completion
pub static JOB_ASSIGNMENT_TIME: Histogram;   // Submission to assignment
```

### Usage Monitoring
```rust
// Track executor selection performance
SELECT_EXECUTOR_CALLS.inc();

// Monitor job pipeline
JOBS_SUBMITTED_TOTAL.inc();
PENDING_JOBS_GAUGE.inc();

// Measure job processing time
let start = Instant::now();
// ... job processing ...
JOB_PROCESS_TIME.observe(start.elapsed().as_secs_f64());
```

## 🔧 Integration Patterns

### Runtime Integration
```rust
// Job submission through runtime
impl RuntimeContext {
    pub async fn handle_submit_job(
        &self,
        manifest_cid: Cid,
        spec_bytes: Vec<u8>,
        cost_mana: u64,
    ) -> Result<JobId, HostAbiError> {
        // 1. Decode and validate job spec
        let job_spec: JobSpec = bincode::deserialize(&spec_bytes)?;
        
        // 2. Apply reputation-based pricing
        let reputation = self.reputation_store.get_reputation(&self.current_identity);
        let adjusted_cost = icn_economics::price_by_reputation(cost_mana, reputation);
        
        // 3. Spend mana
        self.spend_mana(&self.current_identity, adjusted_cost).await?;
        
        // 4. Generate job ID and create DAG entry
        let job_id = generate_job_id(&manifest_cid, &spec_bytes, &self.current_identity);
        
        // 5. Auto-execute CCL WASM jobs
        if job_spec.kind.is_ccl_wasm() {
            return self.execute_ccl_wasm_job(job_id, manifest_cid).await;
        }
        
        // 6. Queue for distributed execution
        self.queue_mesh_job(job_id, job_spec).await?;
        
        Ok(job_id)
    }
}
```

### Executor Selection
```rust
// Select best executor from bids
pub fn select_executor(
    bids: &[MeshJobBid],
    job_spec: &JobSpec,
    reputation_store: &dyn ReputationStore,
    latency_store: &dyn LatencyStore,
    capability_checker: &dyn CapabilityChecker,
    policy: &SelectionPolicy,
) -> Option<&MeshJobBid> {
    let valid_bids: Vec<_> = bids
        .iter()
        .filter(|bid| validate_bid(bid, job_spec, capability_checker))
        .collect();
    
    if valid_bids.is_empty() {
        return None;
    }
    
    valid_bids
        .iter()
        .max_by(|a, b| {
            let score_a = score_bid(a, job_spec, reputation_store, latency_store, policy);
            let score_b = score_bid(b, job_spec, reputation_store, latency_store, policy);
            score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}
```

### Network Protocol Messages
```rust
// Job announcement
pub struct MeshJobAnnounce {
    pub job_id: JobId,
    pub manifest_cid: Cid,
    pub creator_did: Did,
    pub cost_mana: u64,
    pub job_kind: JobKind,
    pub required_resources: Resources,
}

// Assignment notification
pub struct JobAssignmentNotice {
    pub job_id: JobId,
    pub executor_did: Did,
    pub signature: SignatureBytes,
    pub manifest_cid: Option<Cid>,
}

// Receipt submission
pub struct SubmitReceiptMessage {
    pub receipt: ExecutionReceipt,
    pub signature: SignatureBytes,
}
```

## 🧪 Testing Support

### Test Utilities
```rust
// Create test job
let job = ActualMeshJob {
    id: JobId(Cid::new_v1_sha256(0x55, b"test_job")),
    manifest_cid: Cid::new_v1_sha256(0x55, b"manifest"),
    spec: JobSpec {
        kind: JobKind::Echo { payload: "hello".to_string() },
        ..Default::default()
    },
    creator_did: test_did,
    cost_mana: 100,
    max_execution_wait_ms: None,
    signature: SignatureBytes(vec![]),
};

// Sign and verify
let signed_job = job.sign(&signing_key)?;
assert!(signed_job.verify_signature(&verifying_key).is_ok());

// Test bid creation
let bid = MeshJobBid {
    job_id: job.id.clone(),
    executor_did: executor_did,
    price_mana: 50,
    resources: Resources {
        cpu_cores: 4,
        memory_mb: 8192,
        storage_mb: 1024,
    },
    executor_capabilities: vec!["wasm".to_string()],
    executor_federations: vec!["test-federation".to_string()],
    executor_trust_scope: Some("public".to_string()),
    signature: SignatureBytes(vec![]),
};
```

### Mock Services
```rust
// In-memory reputation store for testing
pub struct InMemoryReputationStore {
    scores: Mutex<HashMap<Did, u64>>,
}

// In-memory latency store for testing
pub struct InMemoryLatencyStore {
    latencies: Mutex<HashMap<Did, u64>>,
}

// No-op capability checker for testing
pub struct NoOpCapabilityChecker;

impl CapabilityChecker for NoOpCapabilityChecker {
    fn check_capability(&self, _did: &Did, _capability: &str) -> bool {
        true  // Allow all capabilities in tests
    }
}
```

## 📈 Performance Characteristics

### Scalability Features
- **Concurrent Bidding**: Multiple executors bid simultaneously
- **Efficient Selection**: O(n) bid scoring and selection
- **DAG Storage**: Content-addressed storage scales with network
- **Checkpoint Recovery**: Resume from failure points
- **Streaming Results**: Process large outputs incrementally

### Resource Management
- **Mana Economics**: Prevent resource abuse through economic incentives
- **Reputation Weighting**: Prioritize reliable executors
- **Resource Matching**: Ensure executors meet job requirements
- **Federation Constraints**: Control job execution scope

### Network Efficiency
- **Content Addressing**: Deduplication through CID-based storage
- **Lazy Loading**: Only load required job components
- **Signature Caching**: Reuse verified signatures where possible
- **Batch Operations**: Group related operations for efficiency

## 🔮 Future Development

### Planned Enhancements
- **Advanced Scheduling**: Machine learning-based executor selection
- **Resource Prediction**: Historical analysis for better matching
- **Cross-Federation Jobs**: Jobs spanning multiple federations
- **Payment Channels**: Micropayments for long-running jobs
- **Fault Tolerance**: Automatic failover and retry mechanisms

### Extension Points
- **Custom JobKind**: Add new job types beyond Echo and CclWasm
- **Selection Policies**: Implement custom executor selection algorithms
- **Capability Systems**: Define domain-specific capability frameworks
- **Aid Templates**: Create specialized mutual aid job patterns

---

**The `icn-mesh` crate provides a complete distributed computing platform that combines economic incentives, cryptographic security, and reputation systems to enable reliable cooperative computing across federated networks.** 