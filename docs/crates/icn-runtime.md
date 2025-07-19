# ICN Runtime (`icn-runtime`) - Core Orchestration Layer

> **The central orchestration engine that provides WASM execution, state management, and service coordination for ICN nodes**

## Overview

The `icn-runtime` crate is the heart of every ICN node, providing the execution environment where WASM modules run and interact with ICN services. It manages all node state, coordinates between different services, and provides the Host ABI that WASM modules use to access network capabilities.

**Key Principle**: This crate provides deterministic, sandboxed execution with comprehensive resource management and security controls.

## Core Architecture

### 🏗️ RuntimeContext - Central State Manager

The `RuntimeContext` is the primary state management component that coordinates all ICN services:

```rust
pub struct RuntimeContext {
    pub current_identity: Did,
    pub mana_ledger: SimpleManaLedger,
    pub job_states: Arc<DashMap<JobId, JobState>>,
    pub governance_module: Arc<DagStoreMutexType<GovernanceModule>>,
    pub mesh_network_service: Arc<MeshNetworkServiceType>,
    pub signer: Arc<dyn Signer>,
    pub dag_store: Arc<DagStoreMutexType<DagStorageService>>,
    pub reputation_store: Arc<dyn ReputationStore>,
    // ... additional services
}
```

### 📋 Environment Types

ICN Runtime supports three distinct environment configurations:

#### Production Environment
- **All services must be production-ready**
- **Real libp2p networking**
- **Persistent storage backends**
- **Ed25519 cryptographic signing**
- **Comprehensive validation**

```rust
let ctx = RuntimeContextBuilder::new(EnvironmentType::Production)
    .with_identity(production_did)
    .with_network_service(libp2p_service)
    .with_signer(ed25519_signer)
    .with_dag_store(postgres_store)
    .with_mana_ledger(persistent_ledger)
    .build()?;
```

#### Development Environment
- **Mixed services allowed**
- **Optional real networking**
- **Optional persistent storage**
- **Real signer required**

```rust
let ctx = RuntimeContextBuilder::new(EnvironmentType::Development)
    .with_identity(dev_did)
    .with_signer(ed25519_signer)
    .with_mana_ledger(file_ledger)
    .build()?; // Network and DAG store will use stubs if not provided
```

#### Testing Environment
- **All stub services**
- **In-memory storage**
- **Deterministic execution**
- **Fast test execution**

```rust
let ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
    .with_identity(test_did)
    .with_initial_mana(1000)
    .build()?;
```

## 🔌 Host ABI - WASM Interface

The Host ABI provides WASM modules with access to ICN services through numbered function indices:

### Economic Functions
```rust
const ABI_HOST_ACCOUNT_GET_MANA: u32 = 10;
const ABI_HOST_ACCOUNT_SPEND_MANA: u32 = 11;
const ABI_HOST_ACCOUNT_CREDIT_MANA: u32 = 12;
```

**Usage Example:**
```rust
// Get current mana balance
pub async fn host_account_get_mana(
    ctx: &RuntimeContext,
    account_id_str: &str,
) -> Result<u64, HostAbiError>

// Spend mana for operations
pub async fn host_account_spend_mana(
    ctx: &RuntimeContext,
    account_id_str: &str,
    amount: u64,
) -> Result<(), HostAbiError>
```

### Mesh Computing Functions
```rust
const ABI_HOST_SUBMIT_MESH_JOB: u32 = 16;
const ABI_HOST_GET_PENDING_MESH_JOBS: u32 = 22;
const ABI_HOST_ANCHOR_RECEIPT: u32 = 23;
```

**Job Submission:**
```rust
pub async fn host_submit_mesh_job(
    ctx: &Arc<RuntimeContext>,
    job_json: &str,
) -> Result<JobId, HostAbiError>
```

### Governance Functions
```rust
const ABI_HOST_CREATE_GOVERNANCE_PROPOSAL: u32 = 17;
const ABI_HOST_CAST_GOVERNANCE_VOTE: u32 = 19;
const ABI_HOST_EXECUTE_GOVERNANCE_PROPOSAL: u32 = 21;
```

**Governance Workflow:**
```rust
// Create proposal
let proposal_id = host_create_governance_proposal(ctx, &payload_json).await?;

// Cast vote
host_cast_governance_vote(ctx, &vote_payload).await?;

// Execute if accepted
if proposal_accepted {
    host_execute_governance_proposal(ctx, &proposal_id).await?;
}
```

### Zero-Knowledge Functions
```rust
const ABI_HOST_VERIFY_ZK_PROOF: u32 = 25;
const ABI_HOST_GENERATE_ZK_PROOF: u32 = 26;
```

### Reputation Functions
```rust
const ABI_HOST_GET_REPUTATION: u32 = 24;
```

## ⚙️ WASM Execution Engine

### WasmExecutor with Security Limits

The runtime provides a sophisticated WASM executor with comprehensive security controls:

```rust
pub struct WasmSecurityLimits {
    pub max_execution_time_secs: u64,  // 30 seconds default
    pub max_memory_pages: u32,          // 10 MB default (160 * 64KB)
    pub max_fuel: u64,                  // 1 million instructions
    pub max_stack_depth: u32,           // 1024 depth limit
    pub max_globals: u32,               // 100 globals limit
    pub max_functions: u32,             // 1000 functions limit
    pub max_tables: u32,                // 10 tables limit
    pub max_table_size: u32,            // 10000 table size limit
}
```

### Resource Limiting
```rust
pub struct ICNResourceLimiter {
    timeout: Duration,
    max_memory_pages: u32,
}

impl ResourceLimiter for ICNResourceLimiter {
    fn memory_growing(&mut self, current: usize, desired: usize, maximum: Option<usize>) -> bool {
        let pages = desired / (64 * 1024);
        if pages > self.max_memory_pages as usize {
            WASM_MEMORY_GROWTH_DENIED.inc();
            false
        } else {
            true
        }
    }
}
```

### Memory Management
Safe WASM memory operations with bounds checking:

```rust
// Read string from WASM memory
pub fn read_string(
    caller: &mut Caller<'_, Arc<RuntimeContext>>,
    ptr: u32,
    len: u32,
) -> Result<String, HostAbiError>

// Write string to WASM memory
pub fn write_string(
    caller: &mut Caller<'_, Arc<RuntimeContext>>,
    ptr: u32,
    data: &str,
) -> Result<(), HostAbiError>
```

## 🏭 Service Configuration System

### Type-Safe Service Selection
The runtime uses a sophisticated configuration system that prevents accidental use of stub services in production:

```rust
impl ServiceConfig {
    pub fn validate(&self) -> Result<(), CommonError> {
        match self.environment {
            ServiceEnvironment::Production => {
                // Ensure no stub services in production
                if let MeshNetworkServiceType::Stub(_) = &*self.mesh_network_service {
                    return Err(CommonError::InternalError(
                        "Stub mesh network service cannot be used in production".to_string(),
                    ));
                }
                // Additional production validations...
            }
        }
    }
}
```

### Service Types

#### Mesh Network Services
- **Production**: `DefaultMeshNetworkService` with real libp2p
- **Testing**: `StubMeshNetworkService` for isolated testing

```rust
pub enum MeshNetworkServiceType {
    Stub(StubMeshNetworkService),
    Default(DefaultMeshNetworkService),
}
```

#### Cryptographic Signers
```rust
// Production signer with Ed25519
pub struct Ed25519Signer {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    did: Did,
}

// Secure key storage with encryption
pub struct HsmKeyStore {
    encrypted_keys: HashMap<String, Vec<u8>>,
    master_key: [u8; 32],
}
```

#### DAG Storage Services
- **PostgreSQL**: Scalable database backend
- **RocksDB**: High-performance embedded database
- **SQLite**: Lightweight embedded database
- **Sled**: Pure Rust embedded database
- **StubDagStore**: In-memory testing storage

### Configuration Examples

#### Production Configuration
```rust
let config = ServiceConfig::production(
    node_did,
    network_service,     // Real libp2p
    ed25519_signer,      // Real cryptography
    did_resolver,        // Real DID resolution
    postgres_dag_store,  // Persistent storage
    sled_mana_ledger,    // Persistent mana ledger
    reputation_store,    // Persistent reputation
    policy_enforcer,     // Optional governance
)?;

let ctx = RuntimeContext::from_service_config(config)?;
```

#### Development Configuration
```rust
let config = ServiceConfig::development(
    dev_did,
    ed25519_signer,      // Real signer required
    file_mana_ledger,    // File-based ledger
    Some(network_service), // Optional real networking
    Some(sqlite_store),    // Optional persistent storage
)?;
```

#### Testing Configuration
```rust
let config = ServiceConfig::testing(
    test_did,
    Some(1000), // Initial mana
)?;
```

## 📊 Job Lifecycle Management

### Complete Job Pipeline
The runtime manages the full lifecycle of mesh jobs:

```rust
// 1. Job Submission with Validation
pub async fn handle_submit_job(
    self: &Arc<Self>,
    manifest_cid: Cid,
    spec_bytes: Vec<u8>,
    cost_mana: u64,
) -> Result<JobId, HostAbiError> {
    // Parse and validate job spec
    let job_spec: JobSpec = bincode::deserialize(&spec_bytes)?;
    
    // Apply reputation-based pricing
    let reputation = self.reputation_store.get_reputation(&self.current_identity);
    let adjusted_cost = icn_economics::price_by_reputation(cost_mana, reputation);
    
    // Spend mana
    self.spend_mana(&self.current_identity, adjusted_cost).await?;
    
    // Generate deterministic job ID
    let job_id = generate_job_id(&manifest_cid, &spec_bytes, &self.current_identity);
    
    // Create DAG entry and manage lifecycle
    // ...
}
```

### Job Execution with Trust Validation
```rust
pub async fn execute_job(
    &self,
    job: &ActualMeshJob,
    executor_did: &Did,
    creator_did: &Did,
) -> Result<IdentityExecutionReceipt, CommonError> {
    // Trust validation for secure execution
    if let Some(required_scope) = &job.spec.required_trust_scope {
        let trust_context = TrustContext::from_scope(required_scope)?;
        let validation_result = self.trust_engine
            .lock()
            .await
            .validate_trust(creator_did, executor_did, &trust_context)
            .await?;
            
        if validation_result == TrustValidationResult::Denied {
            return Err(CommonError::PermissionDenied(
                "Executor not trusted for this job scope".to_string()
            ));
        }
    }
    
    // Execute with security limits
    let executor = WasmExecutor::new(self.clone(), security_limits)?;
    executor.execute_job(job).await
}
```

## 🗳️ Governance Integration

### Proposal Management
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProposalPayload {
    pub proposal_type_str: String,
    pub type_specific_payload: Vec<u8>,
    pub description: String,
    pub duration_secs: u64,
    pub quorum: Option<usize>,
    pub threshold: Option<f32>,
    pub body: Option<Vec<u8>>,
}

impl RuntimeContext {
    pub async fn create_governance_proposal(
        &self,
        payload: CreateProposalPayload,
    ) -> Result<String, HostAbiError> {
        // Validate proposal
        // Spend mana for proposal creation
        // Create proposal in governance module
        // Broadcast to network
        // Return proposal ID
    }
}
```

### Voting System
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastVotePayload {
    pub proposal_id_str: String,
    pub vote_option_str: String,
}

impl RuntimeContext {
    pub async fn cast_governance_vote(
        &self,
        payload: CastVotePayload,
    ) -> Result<(), HostAbiError> {
        // Validate voter eligibility
        // Spend mana for voting
        // Cast vote
        // Broadcast vote to network
    }
}
```

## 📈 Metrics and Monitoring

### Performance Metrics
```rust
// Job metrics
static JOBS_SUBMITTED: Lazy<Counter> = Lazy::new(Counter::default);
static JOBS_ACTIVE_GAUGE: Lazy<Gauge<f64>> = Lazy::new(Gauge::default);
static JOBS_COMPLETED: Lazy<Counter> = Lazy::new(Counter::default);
static JOBS_FAILED: Lazy<Counter> = Lazy::new(Counter::default);

// Host ABI metrics
static HOST_SUBMIT_MESH_JOB_CALLS: Lazy<Counter> = Lazy::new(Counter::default);
static HOST_ACCOUNT_GET_MANA_CALLS: Lazy<Counter> = Lazy::new(Counter::default);

// WASM security metrics
static WASM_MEMORY_GROWTH_DENIED: Lazy<Counter> = Lazy::new(Counter::default);
static WASM_TABLE_GROWTH_DENIED: Lazy<Counter> = Lazy::new(Counter::default);
```

### Health Monitoring
```rust
impl RuntimeContext {
    pub fn validate_production_services(&self) -> Result<(), CommonError> {
        // Check if using stub services in production
        if let MeshNetworkServiceType::Stub(_) = &*self.mesh_network_service {
            return Err(CommonError::InternalError(
                "❌ PRODUCTION ERROR: Stub mesh network service detected".to_string()
            ));
        }
        // Additional production validations...
    }
}
```

## 🔧 Configuration Management

### Environment Configuration
```rust
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub identity: IdentityConfig,
    pub storage: StorageConfig,
    pub network: NetworkConfig,
    pub governance: GovernanceConfig,
    pub parameters: RuntimeParametersConfig,
}

impl RuntimeConfig {
    pub fn production() -> Self {
        RuntimeConfig {
            identity: IdentityConfig::with_encryption(),
            storage: StorageConfig::persistent(),
            network: NetworkConfig::libp2p(),
            governance: GovernanceConfig::enabled(),
            parameters: RuntimeParametersConfig::production_defaults(),
        }
    }
}
```

### Parameter Management
```rust
// Runtime parameters are anchored to DAG for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterUpdate {
    pub name: String,
    pub value: String,
    pub timestamp: u64,
    pub signer: Did,
}

impl RuntimeContext {
    pub async fn update_parameter(
        &self,
        name: &str,
        value: &str,
    ) -> Result<(), HostAbiError> {
        // Update in-memory parameter
        self.parameters.insert(name.to_string(), value.to_string());
        
        // Create audit record
        let update = ParameterUpdate {
            name: name.to_string(),
            value: value.to_string(),
            timestamp: self.time_provider.unix_seconds(),
            signer: self.current_identity.clone(),
        };
        
        // Anchor to DAG for transparency
        self.anchor_parameter_update(update).await?;
        
        Ok(())
    }
}
```

## 🧪 Testing Support

### Test Utilities
```rust
// Create test context with predefined mana
let ctx = RuntimeContext::new_testing(
    Did::from_str("did:key:zTest...")?,
    Some(1000), // Initial mana
)?;

// Test with custom system info
let system_info = FixedSystemInfoProvider::new(4, 8192); // 4 cores, 8GB
let ctx = RuntimeContext::new_with_system_info(
    test_did,
    system_info,
    Some(500),
)?;

// Test WASM host functions
let result = host_submit_mesh_job(&ctx, &job_json).await?;
assert!(result.is_ok());
```

### Mock Services
```rust
// Stub network service for testing
pub struct StubMeshNetworkService {
    announced_jobs: Arc<Mutex<Vec<ActualMeshJob>>>,
    proposals: Arc<Mutex<Vec<Vec<u8>>>>,
    votes: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl MeshNetworkService for StubMeshNetworkService {
    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        self.announced_jobs.lock().unwrap().push(job.clone());
        Ok(())
    }
}
```

## 📚 Integration Patterns

### WASM Module Integration
```rust
// WASM modules import ICN functions
(import "icn" "host_submit_mesh_job" (func $submit (param i32 i32) (result i64)))
(import "icn" "host_account_get_mana" (func $get_mana (param i32 i32) (result i64)))
(import "icn" "host_cast_governance_vote" (func $vote (param i32 i32) (result i32)))

// Memory management for WASM
let job_json = serde_json::to_string(&job)?;
let ptr = allocate_in_wasm_memory(&job_json);
let result = call_wasm_function("host_submit_mesh_job", ptr, job_json.len());
```

### Service Integration
```rust
// Runtime coordinates between all services
impl RuntimeContext {
    pub async fn process_mesh_job(&self, job: ActualMeshJob) -> Result<(), CommonError> {
        // 1. Validate with governance
        self.governance_module.lock().await.validate_job(&job)?;
        
        // 2. Check mana and reputation
        let cost = job.cost_mana;
        let reputation = self.reputation_store.get_reputation(&job.creator_did);
        let adjusted_cost = icn_economics::price_by_reputation(cost, reputation);
        self.spend_mana(&job.creator_did, adjusted_cost).await?;
        
        // 3. Execute via WASM
        let receipt = self.execute_job(&job).await?;
        
        // 4. Anchor receipt to DAG
        let cid = self.anchor_receipt(&receipt).await?;
        
        // 5. Update reputation
        self.reputation_store.update_reputation(&job.creator_did, &receipt);
        
        // 6. Broadcast completion
        self.mesh_network_service.announce_completion(&receipt).await?;
        
        Ok(())
    }
}
```

## 🔮 Future Development

### Planned Enhancements
- **Multi-Language Support**: Support for languages beyond WASM
- **Advanced Checkpointing**: Resume execution from arbitrary points
- **Resource Prediction**: ML-based resource usage prediction
- **Hot Reloading**: Update runtime without restart
- **Advanced Trust Models**: More sophisticated trust policies

### Performance Optimizations
- **JIT Compilation**: Faster WASM execution
- **Memory Pooling**: Reduce allocation overhead
- **Batch Processing**: Process multiple jobs efficiently
- **Parallel Execution**: Execute multiple jobs concurrently

---

**The `icn-runtime` crate provides the secure, scalable execution environment that enables ICN's vision of cooperative digital infrastructure. Every design decision prioritizes security, determinism, and verifiable execution.** 