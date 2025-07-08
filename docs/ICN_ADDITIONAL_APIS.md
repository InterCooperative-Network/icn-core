# ICN Core Additional APIs

> **Supplementary documentation for DAG storage, reputation system, network abstractions, and CLI interfaces in the ICN core workspace.**

This document covers the remaining public APIs and components not fully detailed in the main API reference.

---

## Table of Contents

1. [DAG Storage APIs](#dag-storage-apis)
2. [Reputation System APIs](#reputation-system-apis)
3. [Network Abstractions](#network-abstractions)
4. [CLI Interface APIs](#cli-interface-apis)
5. [Governance System APIs](#governance-system-apis)
6. [Node Binary APIs](#node-binary-apis)
7. [Metrics and Monitoring](#metrics-and-monitoring)
8. [Configuration Management](#configuration-management)

---

## DAG Storage APIs

### `icn-dag` - Content-Addressed Storage

#### StorageService Trait

```rust
pub trait StorageService<T>: Send + Sync {
    fn put(&mut self, item: &T) -> Result<(), CommonError>;
    fn get(&self, cid: &Cid) -> Result<Option<T>, CommonError>;
    fn delete(&mut self, cid: &Cid) -> Result<(), CommonError>;
    fn contains(&self, cid: &Cid) -> Result<bool, CommonError>;
}
```

#### Storage Implementations

##### InMemoryDagStore

```rust
pub struct InMemoryDagStore {
    blocks: HashMap<Cid, DagBlock>,
}

impl InMemoryDagStore {
    pub fn new() -> Self;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

**Usage Example:**
```rust
use icn_dag::{InMemoryDagStore, StorageService};

let mut store = InMemoryDagStore::new();

// Store a block
store.put(&dag_block)?;

// Retrieve a block
let retrieved = store.get(&cid)?;

// Check if block exists
let exists = store.contains(&cid)?;

// Delete a block
store.delete(&cid)?;
```

##### FileDagStore

```rust
pub struct FileDagStore {
    base_path: PathBuf,
}

impl FileDagStore {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self, CommonError>;
}
```

**Usage Example:**
```rust
use icn_dag::FileDagStore;

let store = FileDagStore::new("./dag_store")?;
store.put(&dag_block)?;
```

#### Async Storage Service

```rust
#[cfg(feature = "async")]
pub trait AsyncStorageService<T>: Send + Sync {
    async fn put(&mut self, item: &T) -> Result<(), CommonError>;
    async fn get(&self, cid: &Cid) -> Result<Option<T>, CommonError>;
    async fn delete(&mut self, cid: &Cid) -> Result<(), CommonError>;
    async fn contains(&self, cid: &Cid) -> Result<bool, CommonError>;
}
```

#### Content Addressing Utilities

```rust
pub fn compute_content_cid(data: &[u8]) -> Cid;
pub fn verify_content_integrity(cid: &Cid, data: &[u8]) -> bool;
```

**Usage Example:**
```rust
use icn_dag::{compute_content_cid, verify_content_integrity};

let data = b"hello world";
let cid = compute_content_cid(data);
let is_valid = verify_content_integrity(&cid, data);
```

---

## Reputation System APIs

### `icn-reputation` - Scoring and Validation

#### ReputationStore Trait

```rust
pub trait ReputationStore: Send + Sync {
    fn get_reputation(&self, did: &Did) -> u64;
    fn set_score(&self, did: Did, score: u64);
    fn record_execution(&self, executor: &Did, success: bool, cpu_ms: u64);
    fn update_reputation(&self, did: &Did, delta: i64);
}
```

#### InMemoryReputationStore

```rust
pub struct InMemoryReputationStore {
    scores: Arc<RwLock<HashMap<Did, u64>>>,
    execution_history: Arc<RwLock<HashMap<Did, Vec<ExecutionRecord>>>>,
}

impl InMemoryReputationStore {
    pub fn new() -> Self;
    pub fn get_all_scores(&self) -> HashMap<Did, u64>;
    pub fn get_execution_history(&self, did: &Did) -> Vec<ExecutionRecord>;
}
```

**Usage Example:**
```rust
use icn_reputation::{InMemoryReputationStore, ReputationStore};

let store = InMemoryReputationStore::new();

// Set initial reputation
store.set_score(did.clone(), 5);

// Record successful execution
store.record_execution(&did, true, 1000);

// Get current reputation
let reputation = store.get_reputation(&did);

// Update reputation manually
store.update_reputation(&did, 1); // increase by 1
```

#### Reputation Calculation

```rust
pub struct ReputationCalculator {
    success_weight: f64,
    failure_weight: f64,
    recency_factor: f64,
}

impl ReputationCalculator {
    pub fn new(success_weight: f64, failure_weight: f64, recency_factor: f64) -> Self;
    
    pub fn calculate_reputation(
        &self,
        execution_history: &[ExecutionRecord],
        base_score: u64,
    ) -> u64;
}
```

**Usage Example:**
```rust
use icn_reputation::ReputationCalculator;

let calculator = ReputationCalculator::new(1.0, -2.0, 0.95);
let history = store.get_execution_history(&did);
let new_reputation = calculator.calculate_reputation(&history, 5);
```

#### Execution Records

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub job_id: Cid,
    pub success: bool,
    pub cpu_ms: u64,
    pub timestamp: u64,
}
```

---

## Network Abstractions

### `icn-network` - P2P Communication

#### NetworkService Trait

```rust
pub trait NetworkService: Send + Sync {
    async fn discover_peers(&self, bootstrap: Option<String>) -> Result<Vec<PeerId>, CommonError>;
    async fn send_message(&self, peer: &PeerId, message: ProtocolMessage) -> Result<(), CommonError>;
    async fn broadcast_message(&self, message: ProtocolMessage) -> Result<(), CommonError>;
    async fn get_local_peer_id(&self) -> Result<PeerId, CommonError>;
    async fn get_connected_peers(&self) -> Result<Vec<PeerId>, CommonError>;
}
```

#### PeerId Type

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub String);

impl PeerId {
    pub fn new(id: String) -> Self;
    pub fn as_str(&self) -> &str;
}
```

#### StubNetworkService (Testing)

```rust
pub struct StubNetworkService {
    local_peer_id: PeerId,
    connected_peers: Arc<Mutex<Vec<PeerId>>>,
}

impl StubNetworkService {
    pub fn new(local_id: String) -> Self;
    pub fn add_peer(&self, peer: PeerId);
    pub fn remove_peer(&self, peer: &PeerId);
}
```

**Usage Example:**
```rust
use icn_network::{StubNetworkService, NetworkService, PeerId};

let service = StubNetworkService::new("local_peer_123".to_string());

// Add peers for testing
service.add_peer(PeerId::new("peer_456".to_string()));

// Discover peers
let peers = service.discover_peers(None).await?;

// Send message
let message = create_protocol_message();
service.send_message(&peer_id, message).await?;

// Broadcast message
service.broadcast_message(broadcast_message).await?;
```

#### LibP2pService (Production)

```rust
pub struct LibP2pService {
    swarm: Arc<Mutex<Swarm<Behaviour>>>,
    local_peer_id: PeerId,
}

impl LibP2pService {
    pub fn new(config: NetworkConfig) -> Result<Self, CommonError>;
    pub fn start_listening(&mut self, addr: Multiaddr) -> Result<(), CommonError>;
    pub fn dial_peer(&mut self, peer: &PeerId, addr: Multiaddr) -> Result<(), CommonError>;
}
```

#### Network Configuration

```rust
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub listen_addresses: Vec<String>,
    pub bootstrap_peers: Vec<String>,
    pub max_connections: usize,
    pub connection_timeout: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self;
}
```

**Usage Example:**
```rust
use icn_network::{LibP2pService, NetworkConfig};

let config = NetworkConfig {
    listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".to_string()],
    bootstrap_peers: vec![],
    max_connections: 50,
    connection_timeout: Duration::from_secs(30),
};

let mut service = LibP2pService::new(config)?;
service.start_listening("/ip4/127.0.0.1/tcp/8000".parse()?)?;
```

---

## CLI Interface APIs

### `icn-cli` - Command Line Interface

#### Command Structure

```rust
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Node(NodeCommands),
    Job(JobCommands),
    Account(AccountCommands),
    Governance(GovernanceCommands),
    Network(NetworkCommands),
}
```

#### Node Commands

```rust
#[derive(Debug, Subcommand)]
pub enum NodeCommands {
    Start {
        #[arg(long)]
        config: Option<PathBuf>,
    },
    Status {
        #[arg(long)]
        api_url: Option<String>,
    },
    Info {
        #[arg(long)]
        api_url: Option<String>,
    },
}
```

**Usage Example:**
```bash
# Start a node
icn-cli node start --config ./node-config.toml

# Get node status
icn-cli node status --api-url http://localhost:8080

# Get node info
icn-cli node info
```

#### Job Commands

```rust
#[derive(Debug, Subcommand)]
pub enum JobCommands {
    Submit {
        #[arg(long)]
        spec_file: PathBuf,
        #[arg(long)]
        cost: u64,
    },
    List {
        #[arg(long)]
        status: Option<String>,
    },
    Get {
        job_id: String,
    },
    Cancel {
        job_id: String,
    },
}
```

**Usage Example:**
```bash
# Submit a job
icn-cli job submit --spec-file job.json --cost 100

# List jobs
icn-cli job list --status pending

# Get specific job
icn-cli job get job_abc123

# Cancel a job
icn-cli job cancel job_abc123
```

#### Account Commands

```rust
#[derive(Debug, Subcommand)]
pub enum AccountCommands {
    Balance {
        did: String,
    },
    Transfer {
        from: String,
        to: String,
        amount: u64,
    },
    History {
        did: String,
        #[arg(long)]
        limit: Option<usize>,
    },
}
```

**Usage Example:**
```bash
# Check balance
icn-cli account balance did:key:z6Mk...

# Transfer mana
icn-cli account transfer did:key:alice did:key:bob 100

# View transaction history
icn-cli account history did:key:alice --limit 10
```

#### CLI Configuration

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CliConfig {
    pub default_api_url: String,
    pub default_identity: Option<String>,
    pub output_format: OutputFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Table,
    Yaml,
}
```

**Usage Example:**
```rust
use icn_cli::{CliConfig, OutputFormat};

let config = CliConfig {
    default_api_url: "http://localhost:8080".to_string(),
    default_identity: Some("did:key:z6Mk...".to_string()),
    output_format: OutputFormat::Table,
};
```

---

## Governance System APIs

### `icn-governance` - Extended Governance Features

#### Proposal Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    SystemParameterChange(String, String),
    NewMemberInvitation(Did),
    MemberRemoval(Did),
    SoftwareUpgrade(String),
    GenericText(String),
}
```

#### Governance Module

```rust
pub struct GovernanceModule {
    proposals: HashMap<ProposalId, Proposal>,
    votes: HashMap<ProposalId, HashMap<Did, Vote>>,
    members: HashSet<Did>,
}

impl GovernanceModule {
    pub fn new() -> Self;
    
    pub fn submit_proposal(
        &mut self,
        proposer: Did,
        proposal_type: ProposalType,
        description: String,
        duration_secs: u64,
        quorum: u32,
        threshold: f64,
    ) -> Result<ProposalId, CommonError>;
    
    pub fn cast_vote(
        &mut self,
        voter: Did,
        proposal_id: &ProposalId,
        vote: VoteOption,
    ) -> Result<(), CommonError>;
    
    pub fn get_proposal(&self, id: &ProposalId) -> Result<Option<Proposal>, CommonError>;
    pub fn list_proposals(&self) -> Result<Vec<Proposal>, CommonError>;
    pub fn close_voting(&mut self, proposal_id: &ProposalId) -> Result<VoteResult, CommonError>;
}
```

**Usage Example:**
```rust
use icn_governance::{GovernanceModule, ProposalType, VoteOption};

let mut gov = GovernanceModule::new();

// Submit a proposal
let proposal_id = gov.submit_proposal(
    proposer_did,
    ProposalType::SystemParameterChange("max_job_cost".to_string(), "1000".to_string()),
    "Increase maximum job cost limit".to_string(),
    86400, // 1 day
    3,     // quorum
    0.6,   // 60% threshold
)?;

// Cast a vote
gov.cast_vote(voter_did, &proposal_id, VoteOption::Yes)?;

// Close voting
let result = gov.close_voting(&proposal_id)?;
```

#### Vote and Proposal Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub proposer: Did,
    pub proposal_type: ProposalType,
    pub description: String,
    pub created_at: u64,
    pub voting_deadline: u64,
    pub quorum: u32,
    pub threshold: f64,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: Did,
    pub option: VoteOption,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Active,
    Passed,
    Failed,
    Cancelled,
}
```

---

## Node Binary APIs

### `icn-node` - Main Node Binary

#### Node Configuration

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    pub identity: IdentityConfig,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub economics: EconomicsConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityConfig {
    pub did_method: String,
    pub key_file: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}
```

**Usage Example:**
```rust
use icn_node::NodeConfig;

let config = NodeConfig {
    identity: IdentityConfig {
        did_method: "key".to_string(),
        key_file: Some("./identity.key".into()),
    },
    network: NetworkConfig::default(),
    storage: StorageConfig::default(),
    economics: EconomicsConfig::default(),
    api: ApiConfig {
        enabled: true,
        bind_address: "0.0.0.0".to_string(),
        port: 8080,
        cors_origins: vec!["*".to_string()],
    },
};
```

#### Node Startup

```rust
pub struct ICNNode {
    config: NodeConfig,
    runtime_context: Arc<RuntimeContext>,
    api_server: Option<ApiServer>,
}

impl ICNNode {
    pub fn new(config: NodeConfig) -> Result<Self, CommonError>;
    pub async fn start(&mut self) -> Result<(), CommonError>;
    pub async fn stop(&mut self) -> Result<(), CommonError>;
    pub fn get_runtime_context(&self) -> &Arc<RuntimeContext>;
}
```

**Usage Example:**
```rust
use icn_node::{ICNNode, NodeConfig};

let config = NodeConfig::load_from_file("node-config.toml")?;
let mut node = ICNNode::new(config)?;

// Start the node
node.start().await?;

// Node runs...

// Stop the node
node.stop().await?;
```

---

## Metrics and Monitoring

### `icn-runtime::metrics` - Performance Metrics

#### Metric Counters

```rust
use metrics::{counter, histogram, gauge};

pub static HOST_SUBMIT_MESH_JOB_CALLS: Lazy<Counter> = Lazy::new(|| {
    counter!("icn.runtime.host_submit_mesh_job.calls")
});

pub static HOST_ACCOUNT_GET_MANA_CALLS: Lazy<Counter> = Lazy::new(|| {
    counter!("icn.runtime.host_account_get_mana.calls")
});
```

#### Custom Metrics

```rust
pub struct ICNMetrics {
    pub jobs_submitted: Counter,
    pub jobs_completed: Counter,
    pub mana_transactions: Counter,
    pub network_messages: Counter,
}

impl ICNMetrics {
    pub fn new() -> Self;
    
    pub fn record_job_submitted(&self);
    pub fn record_job_completed(&self, success: bool, duration_ms: u64);
    pub fn record_mana_transaction(&self, amount: u64);
    pub fn record_network_message(&self, message_type: &str);
}
```

**Usage Example:**
```rust
use icn_runtime::metrics::ICNMetrics;

let metrics = ICNMetrics::new();

// Record metrics
metrics.record_job_submitted();
metrics.record_mana_transaction(100);
metrics.record_job_completed(true, 1500);
```

---

## Configuration Management

### Configuration Loading

```rust
pub trait ConfigLoader {
    type Config;
    
    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self::Config, CommonError>;
    fn load_from_env() -> Result<Self::Config, CommonError>;
    fn load_from_args(args: &[String]) -> Result<Self::Config, CommonError>;
}
```

### Environment Variables

```rust
pub struct EnvConfig {
    pub icn_node_identity: Option<String>,
    pub icn_api_port: Option<u16>,
    pub icn_storage_path: Option<PathBuf>,
    pub icn_log_level: Option<String>,
}

impl EnvConfig {
    pub fn load() -> Self;
    pub fn merge_with_file_config(self, file_config: NodeConfig) -> NodeConfig;
}
```

**Usage Example:**
```rust
use icn_node::{NodeConfig, EnvConfig};

// Load from file
let mut config = NodeConfig::load_from_file("config.toml")?;

// Override with environment variables
let env_config = EnvConfig::load();
config = env_config.merge_with_file_config(config);
```

### Configuration Validation

```rust
pub trait ValidateConfig {
    fn validate(&self) -> Result<(), Vec<String>>;
}

impl ValidateConfig for NodeConfig {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if self.api.port == 0 {
            errors.push("API port cannot be 0".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

---

## Integration Testing Utilities

### Test Harnesses

```rust
pub struct TestHarness {
    pub storage: Arc<tokio::sync::Mutex<InMemoryDagStore>>,
    pub ledger: Arc<InMemoryLedger>,
    pub reputation_store: Arc<InMemoryReputationStore>,
    pub network_service: Arc<StubNetworkService>,
    pub runtime_context: Arc<RuntimeContext>,
}

impl TestHarness {
    pub fn new() -> Self;
    pub async fn submit_test_job(&self) -> Result<JobId, CommonError>;
    pub async fn create_test_bid(&self, job_id: &JobId) -> Result<MeshJobBid, CommonError>;
    pub async fn complete_job_workflow(&self) -> Result<Cid, CommonError>;
}
```

**Usage Example:**
```rust
use icn_testing::TestHarness;

#[tokio::test]
async fn test_complete_workflow() {
    let harness = TestHarness::new();
    
    let job_id = harness.submit_test_job().await?;
    let bid = harness.create_test_bid(&job_id).await?;
    let receipt_cid = harness.complete_job_workflow().await?;
    
    assert!(!receipt_cid.to_string().is_empty());
}
```

---

This supplementary documentation completes the coverage of all major public APIs and components in the ICN core workspace. Together with the main API reference, it provides comprehensive guidance for developing with and extending the ICN system.