# ICN Core API Reference

> **Comprehensive documentation for all public APIs, functions, and components in the InterCooperative Network (ICN) core workspace.**

This document provides detailed API documentation for all public interfaces in the `icn-core` workspace, including examples and usage instructions for each component.

---

## Table of Contents

1. [Overview](#overview)
2. [Core Foundation APIs](#core-foundation-apis)
3. [Identity and Cryptography APIs](#identity-and-cryptography-apis)
4. [Mesh Computing APIs](#mesh-computing-apis)
5. [Economic System APIs](#economic-system-apis)
6. [Runtime and Host ABI](#runtime-and-host-abi)
7. [Network Protocol APIs](#network-protocol-apis)
8. [External API Interfaces](#external-api-interfaces)
9. [Governance APIs](#governance-apis)
10. [DAG Storage APIs](#dag-storage-apis)
11. [Command Line Interface](#command-line-interface)
12. [Integration Examples](#integration-examples)

---

## Overview

The ICN core system consists of multiple crates that work together to provide a complete decentralized computing and governance platform. Each crate has specific responsibilities and exposes public APIs for external consumption.

### Crate Dependencies

```
icn-common (foundation)
├── icn-protocol (message definitions)
├── icn-identity (DID/crypto management)
├── icn-dag (content-addressed storage)
└── icn-economics (mana/resource management)
    ├── icn-mesh (job/bid/execution)
    ├── icn-governance (proposals/voting)
    ├── icn-reputation (scoring/validation)
    └── icn-network (p2p communication)
        └── icn-runtime (orchestration)
            ├── icn-api (external interfaces)
            ├── icn-cli (command interface)
            └── icn-node (binary/server)
```

---

## Core Foundation APIs

### `icn-common` - Foundation Types and Utilities

The `icn-common` crate provides fundamental types and utilities used across all other crates.

#### Core Constants

```rust
pub const ICN_CORE_VERSION: &str = "0.1.0-dev-functional";
```

#### Core Types

##### Did (Decentralized Identifier)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Did {
    pub method: String,      // e.g., "key", "web", "peer"
    pub id_string: String,   // method-specific identifier
    pub path: Option<String>,
    pub query: Option<String>,
    pub fragment: Option<String>,
}
```

**Usage Example:**
```rust
use icn_common::Did;
use std::str::FromStr;

// Create a DID from string
let did = Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8")?;

// Create a DID programmatically
let did = Did::new("key", "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8");

// Convert to string
println!("DID: {}", did);
```

##### Cid (Content Identifier)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cid {
    pub version: u64,        // CIDv0 or CIDv1
    pub codec: u64,          // Multicodec for content type
    pub hash_alg: u64,       // Hash algorithm (e.g., SHA-256)
    pub hash_bytes: Vec<u8>, // Raw hash bytes
}
```

**Usage Example:**
```rust
use icn_common::Cid;

// Create a CID using SHA-256
let cid = Cid::new_v1_sha256(0x55, b"hello world");

// Convert to string
let cid_string = cid.to_string();

// Parse from string
let parsed_cid = icn_common::parse_cid_from_string(&cid_string)?;
```

##### DagBlock (DAG Block)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagBlock {
    pub cid: Cid,
    pub data: Vec<u8>,
    pub links: Vec<DagLink>,
    pub timestamp: u64,
    pub author_did: Did,
    pub signature: Option<SignatureBytes>,
    pub scope: Option<NodeScope>,
}
```

**Usage Example:**
```rust
use icn_common::{DagBlock, DagLink, compute_merkle_cid, verify_block_integrity};

// Create a DAG block
let link = DagLink {
    cid: child_cid,
    name: "child".to_string(),
    size: 100,
};

let cid = compute_merkle_cid(
    0x55,
    b"block data",
    &[link.clone()],
    timestamp,
    &author_did,
    &None,
    &None,
);

let block = DagBlock {
    cid,
    data: b"block data".to_vec(),
    links: vec![link],
    timestamp,
    author_did,
    signature: None,
    scope: None,
};

// Verify block integrity
verify_block_integrity(&block)?;
```

#### Error Handling

```rust
#[derive(Debug, Error, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum CommonError {
    #[error("Invalid input: {0}")]
    InvalidInputError(String),
    
    #[error("Cryptography error: {0}")]
    CryptoError(String),
    
    #[error("Policy denied: {0}")]
    PolicyDenied(String),
    
    // ... many other error variants
}
```

#### Time Providers

```rust
pub trait TimeProvider: Send + Sync {
    fn unix_seconds(&self) -> u64;
}

pub struct SystemTimeProvider;
pub struct FixedTimeProvider(pub u64);
```

**Usage Example:**
```rust
use icn_common::{TimeProvider, SystemTimeProvider, FixedTimeProvider};

// Use system time
let time_provider = SystemTimeProvider;
let current_time = time_provider.unix_seconds();

// Use fixed time for testing
let fixed_time = FixedTimeProvider::new(1234567890);
let test_time = fixed_time.unix_seconds();
```

#### Signable Trait

```rust
pub trait Signable {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError>;
    fn sign(&self, key: &ed25519_dalek::SigningKey) -> Result<SignatureBytes, CommonError>;
    fn verify(&self, signature: &SignatureBytes, key: &ed25519_dalek::VerifyingKey) -> Result<(), CommonError>;
}
```

---

## Identity and Cryptography APIs

### `icn-identity` - DID Management and Cryptography

#### Key Generation

```rust
pub fn generate_ed25519_keypair() -> (SigningKey, VerifyingKey);
```

**Usage Example:**
```rust
use icn_identity::generate_ed25519_keypair;

let (signing_key, verifying_key) = generate_ed25519_keypair();
```

#### DID Methods

##### DID:key

```rust
pub fn did_key_from_verifying_key(pk: &VerifyingKey) -> String;
pub fn verifying_key_from_did_key(did: &Did) -> Result<VerifyingKey, CommonError>;
```

**Usage Example:**
```rust
use icn_identity::{generate_ed25519_keypair, did_key_from_verifying_key, verifying_key_from_did_key};
use std::str::FromStr;

let (sk, pk) = generate_ed25519_keypair();
let did_string = did_key_from_verifying_key(&pk);
let did = Did::from_str(&did_string)?;
let recovered_pk = verifying_key_from_did_key(&did)?;
```

##### DID:web

```rust
pub fn did_web_from_parts(domain: &str, path: &[&str]) -> Result<String, CommonError>;
pub fn parse_did_web(did: &Did) -> Result<(String, Vec<String>), CommonError>;
```

**Usage Example:**
```rust
use icn_identity::{did_web_from_parts, parse_did_web};

let did_string = did_web_from_parts("example.com", &["user", "alice"])?;
// Results in: "did:web:example.com:user:alice"

let did = Did::from_str(&did_string)?;
let (domain, path) = parse_did_web(&did)?;
```

##### DID:peer

```rust
pub fn did_peer_from_verifying_key(pk: &VerifyingKey) -> String;
pub fn verifying_key_from_did_peer(did: &Did) -> Result<VerifyingKey, CommonError>;
```

#### Key Storage and Rotation

```rust
pub trait KeyStorage: Send + Sync {
    fn get_signing_key(&self, did: &Did) -> Option<&SigningKey>;
    fn store_signing_key(&mut self, did: Did, key: SigningKey);
}

pub trait KeyRotation: Send + Sync {
    fn rotate_ed25519(&mut self, did: &Did) -> Result<Did, CommonError>;
}

pub struct InMemoryKeyStore {
    keys: HashMap<Did, SigningKey>,
}
```

**Usage Example:**
```rust
use icn_identity::{InMemoryKeyStore, KeyStorage, KeyRotation};

let mut store = InMemoryKeyStore::default();
store.store_signing_key(did.clone(), signing_key);

let retrieved_key = store.get_signing_key(&did);
let new_did = store.rotate_ed25519(&did)?;
```

#### DID Resolution

```rust
pub trait DidResolver: Send + Sync {
    fn resolve(&self, did: &Did) -> Result<VerifyingKey, CommonError>;
}

pub struct KeyDidResolver;
pub struct PeerDidResolver;
pub struct WebDidResolver {
    keys: HashMap<String, VerifyingKey>,
}
```

**Usage Example:**
```rust
use icn_identity::{KeyDidResolver, WebDidResolver, DidResolver};

// Resolve did:key
let resolver = KeyDidResolver;
let verifying_key = resolver.resolve(&did)?;

// Resolve did:web with custom resolver
let mut web_resolver = WebDidResolver::new();
web_resolver.insert(did_string, verifying_key);
let resolved_key = web_resolver.resolve(&did)?;
```

#### Execution Receipts

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub job_id: Cid,
    pub executor_did: Did,
    pub result_cid: Cid,
    pub cpu_ms: u64,
    pub success: bool,
    pub sig: SignatureBytes,
}
```

**Usage Example:**
```rust
use icn_identity::ExecutionReceipt;

let receipt = ExecutionReceipt {
    job_id: job_cid,
    executor_did: executor_did.clone(),
    result_cid: result_cid,
    cpu_ms: 1000,
    success: true,
    sig: SignatureBytes(vec![]),
};

// Sign the receipt
let signed_receipt = receipt.sign_with_key(&signing_key)?;

// Verify the receipt
signed_receipt.verify_against_key(&verifying_key)?;
signed_receipt.verify_against_did(&executor_did)?;
```

#### Membership and Permissions

```rust
pub trait MembershipResolver: Send + Sync {
    fn is_member(&self, did: &Did, scope: &NodeScope) -> bool;
}

pub struct MembershipPolicyEnforcer<R: MembershipResolver> {
    resolver: R,
}
```

**Usage Example:**
```rust
use icn_identity::{InMemoryMembershipResolver, MembershipPolicyEnforcer};

let mut resolver = InMemoryMembershipResolver::new();
resolver.add_member(scope.clone(), did.clone());

let enforcer = MembershipPolicyEnforcer::new(resolver);
enforcer.check_permission(&did, &scope)?;
```

---

## Mesh Computing APIs

### `icn-mesh` - Job Orchestration and Execution

#### Core Types

##### JobId

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub Cid);
```

##### JobSpec and JobKind

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum JobKind {
    Echo { payload: String },
    CclWasm,
    #[default]
    GenericPlaceholder,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobSpec {
    pub kind: JobKind,
    pub inputs: Vec<Cid>,
    pub outputs: Vec<String>,
    pub required_resources: Resources,
}
```

##### MeshJob

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
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

**Usage Example:**
```rust
use icn_mesh::{ActualMeshJob, JobSpec, JobKind, Resources};

let job_spec = JobSpec {
    kind: JobKind::Echo { payload: "hello".to_string() },
    inputs: vec![],
    outputs: vec!["result".to_string()],
    required_resources: Resources { cpu_cores: 1, memory_mb: 512 },
};

let job = ActualMeshJob {
    id: JobId::from(job_cid),
    manifest_cid: manifest_cid,
    spec: job_spec,
    creator_did: creator_did,
    cost_mana: 100,
    max_execution_wait_ms: Some(30000),
    signature: SignatureBytes(vec![]),
};

// Sign the job
let signed_job = job.sign(&signing_key)?;

// Verify the job signature
signed_job.verify_signature(&verifying_key)?;
```

#### Bidding System

##### MeshJobBid

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshJobBid {
    pub job_id: JobId,
    pub executor_did: Did,
    pub price_mana: u64,
    pub resources: Resources,
    pub signature: SignatureBytes,
}
```

**Usage Example:**
```rust
use icn_mesh::MeshJobBid;

let bid = MeshJobBid {
    job_id: job_id.clone(),
    executor_did: executor_did.clone(),
    price_mana: 50,
    resources: Resources { cpu_cores: 2, memory_mb: 1024 },
    signature: SignatureBytes(vec![]),
};

// Sign the bid
let signed_bid = bid.sign(&executor_signing_key)?;

// Verify the bid
signed_bid.verify_signature(&executor_verifying_key)?;
```

#### Executor Selection

```rust
pub struct SelectionPolicy {
    pub weight_price: f64,
    pub weight_reputation: f64,
    pub weight_resources: f64,
}

pub fn select_executor(
    job_id: &JobId,
    job_spec: &JobSpec,
    bids: Vec<MeshJobBid>,
    policy: &SelectionPolicy,
    reputation_store: &dyn icn_reputation::ReputationStore,
    mana_ledger: &dyn icn_economics::ManaLedger,
) -> Option<Did>;
```

**Usage Example:**
```rust
use icn_mesh::{select_executor, SelectionPolicy};

let policy = SelectionPolicy {
    weight_price: 1.0,
    weight_reputation: 50.0,
    weight_resources: 1.0,
};

let selected_executor = select_executor(
    &job_id,
    &job_spec,
    bids,
    &policy,
    &reputation_store,
    &mana_ledger,
);

if let Some(executor_did) = selected_executor {
    println!("Selected executor: {}", executor_did);
}
```

#### Protocol Messages

##### Job Announcement

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshJobAnnounce {
    pub job_id: JobId,
    pub manifest_cid: Cid,
    pub creator_did: Did,
    pub cost_mana: u64,
    pub job_kind: JobKind,
    pub required_resources: Resources,
}
```

##### Bid Submission

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshBidSubmit {
    pub bid: MeshJobBid,
    pub signature: SignatureBytes,
}
```

##### Job Assignment

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAssignmentNotice {
    pub job_id: JobId,
    pub executor_did: Did,
    pub signature: SignatureBytes,
    pub manifest_cid: Option<Cid>,
}
```

---

## Economic System APIs

### `icn-economics` - Mana Management and Economic Policies

#### ManaLedger Trait

```rust
pub trait ManaLedger: Send + Sync {
    fn get_balance(&self, did: &Did) -> u64;
    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    fn spend(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    fn credit(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    fn credit_all(&self, amount: u64) -> Result<(), CommonError>;
    fn all_accounts(&self) -> Vec<Did>;
}
```

#### Ledger Implementations

```rust
pub struct FileManaLedger;           // JSON file-based ledger
pub struct SledManaLedger;           // Sled embedded DB ledger  
pub struct RocksdbManaLedger;        // RocksDB ledger
pub struct SqliteManaLedger;         // SQLite ledger
```

**Usage Example:**
```rust
use icn_economics::{ManaLedger, FileManaLedger};

let ledger = FileManaLedger::new("mana.json")?;

// Set initial balance
ledger.set_balance(&did, 1000)?;

// Check balance
let balance = ledger.get_balance(&did);
println!("Balance: {}", balance);

// Spend mana
ledger.spend(&did, 100)?;

// Credit mana
ledger.credit(&did, 50)?;
```

#### Repository and Policy Enforcement

```rust
pub struct ManaRepositoryAdapter<L: ManaLedger> {
    ledger: L,
}

pub struct ResourcePolicyEnforcer<L: ManaLedger> {
    adapter: ManaRepositoryAdapter<L>,
}
```

**Usage Example:**
```rust
use icn_economics::{ManaRepositoryAdapter, ResourcePolicyEnforcer};

let adapter = ManaRepositoryAdapter::new(ledger);
let enforcer = ResourcePolicyEnforcer::new(adapter);

// This will enforce spending limits and policies
enforcer.spend_mana(&did, amount)?;
```

#### High-Level Functions

```rust
pub fn charge_mana<L: ManaLedger>(ledger: L, did: &Did, amount: u64) -> Result<(), CommonError>;
pub fn credit_mana<L: ManaLedger>(ledger: L, did: &Did, amount: u64) -> Result<(), CommonError>;

pub fn credit_by_reputation(
    ledger: &dyn ManaLedger,
    reputation_store: &dyn icn_reputation::ReputationStore,
    base_amount: u64,
) -> Result<(), CommonError>;

pub fn price_by_reputation(base_price: u64, reputation: u64) -> u64;
```

**Usage Example:**
```rust
use icn_economics::{charge_mana, credit_mana, price_by_reputation};

// Charge mana with policy enforcement
charge_mana(ledger, &did, 100)?;

// Credit mana
credit_mana(ledger, &did, 50)?;

// Calculate price based on reputation
let adjusted_price = price_by_reputation(100, reputation_score);
```

---

## Runtime and Host ABI

### `icn-runtime` - Runtime Context and Host ABI

#### RuntimeContext

```rust
pub struct RuntimeContext {
    pub current_identity: Did,
    pub pending_mesh_jobs: Arc<tokio::sync::Mutex<Vec<ActualMeshJob>>>,
    // ... other fields
}
```

#### Host ABI Functions

##### Job Management

```rust
pub async fn host_submit_mesh_job(
    ctx: &Arc<RuntimeContext>,
    job_json: &str,
) -> Result<JobId, HostAbiError>;

pub async fn host_get_pending_mesh_jobs(
    ctx: &RuntimeContext,
) -> Result<Vec<ActualMeshJob>, HostAbiError>;
```

**Usage Example:**
```rust
use icn_runtime::{host_submit_mesh_job, host_get_pending_mesh_jobs};

let job_json = serde_json::to_string(&job)?;
let job_id = host_submit_mesh_job(&ctx, &job_json).await?;

let pending_jobs = host_get_pending_mesh_jobs(&ctx).await?;
```

##### Account Management

```rust
pub async fn host_account_get_mana(
    ctx: &RuntimeContext,
    account_id_str: &str,
) -> Result<u64, HostAbiError>;

pub async fn host_account_spend_mana(
    ctx: &RuntimeContext,
    account_id_str: &str,
    amount: u64,
) -> Result<(), HostAbiError>;

pub async fn host_account_credit_mana(
    ctx: &RuntimeContext,
    account_id_str: &str,
    amount: u64,
) -> Result<(), HostAbiError>;
```

**Usage Example:**
```rust
use icn_runtime::{host_account_get_mana, host_account_spend_mana};

let balance = host_account_get_mana(&ctx, &did.to_string()).await?;
host_account_spend_mana(&ctx, &did.to_string(), 100).await?;
```

##### Receipt Anchoring

```rust
pub async fn host_anchor_receipt(
    ctx: &RuntimeContext,
    receipt_json: &str,
    reputation_updater: &ReputationUpdater,
) -> Result<Cid, HostAbiError>;
```

**Usage Example:**
```rust
use icn_runtime::{host_anchor_receipt, ReputationUpdater};

let receipt_json = serde_json::to_string(&receipt)?;
let updater = ReputationUpdater::new();
let anchored_cid = host_anchor_receipt(&ctx, &receipt_json, &updater).await?;
```

#### WASM Integration

```rust
pub fn wasm_host_submit_mesh_job(
    caller: wasmtime::Caller<'_, Arc<RuntimeContext>>,
    ptr: u32, len: u32,
    out_ptr: u32, out_len: u32,
) -> u32;

pub fn wasm_host_account_get_mana(
    caller: wasmtime::Caller<'_, Arc<RuntimeContext>>,
    ptr: u32, len: u32,
) -> u64;
```

---

## Network Protocol APIs

### `icn-protocol` - Message Definitions and Protocol Constants

#### Protocol Message Envelope

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub version: u32,
    pub payload: MessagePayload,
    pub sender: Did,
    pub recipient: Option<Did>,
    pub timestamp: u64,
    pub signature: SignatureBytes,
}
```

#### Message Payload Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    // Mesh Computing Messages
    MeshJobAnnouncement(MeshJobAnnouncementMessage),
    MeshBidSubmission(MeshBidSubmissionMessage),
    MeshJobAssignment(MeshJobAssignmentMessage),
    MeshReceiptSubmission(MeshReceiptSubmissionMessage),
    
    // DAG and Storage Messages
    DagBlockAnnouncement(DagBlockAnnouncementMessage),
    DagBlockRequest(DagBlockRequestMessage),
    DagBlockResponse(DagBlockResponseMessage),
    
    // Governance Messages
    GovernanceProposalAnnouncement(GovernanceProposalMessage),
    GovernanceVoteAnnouncement(GovernanceVoteMessage),
    GovernanceStateSyncRequest(GovernanceStateSyncRequestMessage),
    
    // Federation Management
    FederationJoinRequest(FederationJoinRequestMessage),
    FederationJoinResponse(FederationJoinResponseMessage),
    FederationSyncRequest(FederationSyncRequestMessage),
    
    // Network Management
    GossipMessage(GossipMessage),
    HeartbeatMessage(HeartbeatMessage),
    PeerDiscoveryMessage(PeerDiscoveryMessage),
}
```

**Usage Example:**
```rust
use icn_protocol::{ProtocolMessage, MessagePayload, HeartbeatMessage, NodeStatus};

let heartbeat = HeartbeatMessage {
    sequence: 1,
    sent_at: current_time,
    node_status: NodeStatus {
        is_online: true,
        peer_count: 5,
        block_height: 1000,
        version: "1.0.0".to_string(),
        available_resources: Resources::default(),
    },
};

let message = ProtocolMessage::new(
    MessagePayload::HeartbeatMessage(heartbeat),
    sender_did,
    None, // broadcast
);
```

#### Resource and Capability Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub storage_mb: u32,
    pub max_execution_time_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub compute_resources: ResourceRequirements,
    pub supported_job_kinds: Vec<JobKind>,
    pub network_bandwidth_mbps: u32,
    pub storage_capacity_gb: u64,
    pub uptime_percentage: f32,
}
```

---

## External API Interfaces

### `icn-api` - External Service APIs

#### Node Information

```rust
pub fn get_node_info() -> Result<NodeInfo, CommonError>;
pub fn get_node_status(is_simulated_online: bool) -> Result<NodeStatus, CommonError>;
```

**Usage Example:**
```rust
use icn_api::{get_node_info, get_node_status};

let info = get_node_info()?;
println!("Node: {} v{}", info.name, info.version);

let status = get_node_status(true)?;
println!("Online: {}, Peers: {}", status.is_online, status.peer_count);
```

#### DAG Operations

```rust
pub async fn submit_dag_block(
    storage: Arc<tokio::sync::Mutex<dyn StorageService<DagBlock> + Send>>,
    block_data_json: String,
    policy_enforcer: Option<Arc<dyn ScopedPolicyEnforcer>>,
    actor: Did,
) -> Result<Cid, CommonError>;

pub async fn retrieve_dag_block(
    storage: Arc<tokio::sync::Mutex<dyn StorageService<DagBlock> + Send>>,
    cid_json: String,
) -> Result<Option<DagBlock>, CommonError>;

pub async fn query_data(
    storage: Arc<tokio::sync::Mutex<dyn StorageService<DagBlock> + Send>>,
    cid_json: String,
) -> Result<Option<DagBlock>, CommonError>;
```

**Usage Example:**
```rust
use icn_api::{submit_dag_block, retrieve_dag_block};

// Submit a block
let block_json = serde_json::to_string(&block)?;
let cid = submit_dag_block(storage.clone(), block_json, None, actor_did).await?;

// Retrieve a block
let cid_json = serde_json::to_string(&cid)?;
let retrieved_block = retrieve_dag_block(storage, cid_json).await?;
```

#### Transaction Handling

```rust
pub fn submit_transaction(tx_json: String) -> Result<String, CommonError>;
```

**Usage Example:**
```rust
use icn_api::submit_transaction;

let transaction_json = serde_json::to_string(&transaction)?;
let tx_id = submit_transaction(transaction_json)?;
```

#### Network Operations

```rust
pub async fn discover_peers_api(
    bootstrap_nodes_str: Vec<String>,
) -> Result<Vec<PeerId>, CommonError>;

pub async fn send_network_message_api(
    peer_id_str: String,
    message_json: String,
) -> Result<(), CommonError>;
```

**Usage Example:**
```rust
use icn_api::{discover_peers_api, send_network_message_api};

let peers = discover_peers_api(bootstrap_nodes).await?;

let message_json = serde_json::to_string(&protocol_message)?;
send_network_message_api(peer_id.to_string(), message_json).await?;
```

#### HTTP Client Functions

```rust
pub async fn http_get_local_peer_id(api_url: &str) -> Result<String, CommonError>;
pub async fn http_get_peer_list(api_url: &str) -> Result<Vec<String>, CommonError>;
```

**Usage Example:**
```rust
use icn_api::{http_get_local_peer_id, http_get_peer_list};

let peer_id = http_get_local_peer_id("http://localhost:8080").await?;
let peers = http_get_peer_list("http://localhost:8080").await?;
```

---

## Governance APIs

### `icn-governance` - Proposals and Voting

(Note: Based on the imports I can see the governance functionality exists but wasn't fully explored in the initial scan)

#### Basic Governance Types

```rust
pub struct ProposalId(pub String);
pub struct Proposal { /* ... */ }

pub enum VoteOption {
    Yes,
    No,
    Abstain,
}
```

#### Governance API Trait

```rust
pub trait GovernanceApi {
    fn submit_proposal(&self, request: SubmitProposalRequest) -> Result<ProposalId, CommonError>;
    fn cast_vote(&self, request: CastVoteRequest) -> Result<(), CommonError>;
    fn get_proposal(&self, id: ProposalId) -> Result<Option<Proposal>, CommonError>;
    fn list_proposals(&self) -> Result<Vec<Proposal>, CommonError>;
}
```

---

## Integration Examples

### Complete Mesh Job Workflow

```rust
use icn_core::*;
use std::sync::Arc;

async fn complete_job_workflow() -> Result<(), CommonError> {
    // 1. Setup identity
    let (signing_key, verifying_key) = icn_identity::generate_ed25519_keypair();
    let did_string = icn_identity::did_key_from_verifying_key(&verifying_key);
    let creator_did = Did::from_str(&did_string)?;
    
    // 2. Create job specification
    let job_spec = icn_mesh::JobSpec {
        kind: icn_mesh::JobKind::Echo { payload: "Hello ICN!".to_string() },
        inputs: vec![],
        outputs: vec!["result".to_string()],
        required_resources: icn_mesh::Resources {
            cpu_cores: 1,
            memory_mb: 512,
        },
    };
    
    // 3. Create and sign mesh job
    let job = icn_mesh::ActualMeshJob {
        id: icn_mesh::JobId::from(job_cid),
        manifest_cid: manifest_cid,
        spec: job_spec,
        creator_did: creator_did.clone(),
        cost_mana: 100,
        max_execution_wait_ms: Some(30000),
        signature: icn_identity::SignatureBytes(vec![]),
    };
    
    let signed_job = job.sign(&signing_key)?;
    
    // 4. Submit via runtime
    let job_json = serde_json::to_string(&signed_job)?;
    let job_id = icn_runtime::host_submit_mesh_job(&runtime_ctx, &job_json).await?;
    
    // 5. Create and submit bid (from executor perspective)
    let bid = icn_mesh::MeshJobBid {
        job_id: job_id.clone(),
        executor_did: executor_did.clone(),
        price_mana: 80,
        resources: icn_mesh::Resources { cpu_cores: 2, memory_mb: 1024 },
        signature: icn_identity::SignatureBytes(vec![]),
    };
    
    let signed_bid = bid.sign(&executor_signing_key)?;
    
    // 6. Executor selection
    let selected = icn_mesh::select_executor(
        &job_id,
        &signed_job.spec,
        vec![signed_bid],
        &icn_mesh::SelectionPolicy::default(),
        &reputation_store,
        &mana_ledger,
    );
    
    // 7. Create execution receipt
    let receipt = icn_identity::ExecutionReceipt {
        job_id: job_id.0,
        executor_did: executor_did.clone(),
        result_cid: result_cid,
        cpu_ms: 1500,
        success: true,
        sig: icn_identity::SignatureBytes(vec![]),
    };
    
    let signed_receipt = receipt.sign_with_key(&executor_signing_key)?;
    
    // 8. Anchor receipt
    let receipt_json = serde_json::to_string(&signed_receipt)?;
    let updater = icn_runtime::ReputationUpdater::new();
    let anchored_cid = icn_runtime::host_anchor_receipt(
        &runtime_ctx,
        &receipt_json,
        &updater
    ).await?;
    
    println!("Job completed! Receipt anchored at: {}", anchored_cid);
    Ok(())
}
```

### Setting Up Storage and Economics

```rust
use icn_core::*;

async fn setup_node_infrastructure() -> Result<(), CommonError> {
    // 1. Setup DAG storage
    let storage = Arc::new(tokio::sync::Mutex::new(
        icn_dag::InMemoryDagStore::new()
    ));
    
    // 2. Setup mana ledger
    let ledger = icn_economics::FileManaLedger::new("node_mana.json")?;
    
    // 3. Setup initial mana for accounts
    let alice_did = Did::from_str("did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8")?;
    ledger.set_balance(&alice_did, 1000)?;
    
    // 4. Setup reputation store
    let reputation_store = icn_reputation::InMemoryReputationStore::new();
    reputation_store.set_score(alice_did.clone(), 5);
    
    // 5. Test economic operations
    let adapter = icn_economics::ManaRepositoryAdapter::new(ledger);
    let enforcer = icn_economics::ResourcePolicyEnforcer::new(adapter);
    
    // This will enforce policies
    enforcer.spend_mana(&alice_did, 100)?;
    
    println!("Node infrastructure setup complete!");
    Ok(())
}
```

### API Server Integration

```rust
use icn_api::*;
use warp::Filter;

async fn start_api_server() {
    let storage = Arc::new(tokio::sync::Mutex::new(
        icn_dag::InMemoryDagStore::new()
    ));
    
    // GET /node/info
    let info = warp::path!("node" / "info")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&get_node_info().unwrap())
        });
    
    // GET /node/status  
    let status = warp::path!("node" / "status")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&get_node_status(true).unwrap())
        });
    
    // POST /dag/submit
    let submit_block = warp::path!("dag" / "submit")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_storage(storage.clone()))
        .and_then(|block_json: String, storage| async move {
            let actor_did = Did::from_str("did:key:test").unwrap();
            match submit_dag_block(storage, block_json, None, actor_did).await {
                Ok(cid) => Ok(warp::reply::json(&cid)),
                Err(e) => Err(warp::reject::custom(ApiError(e))),
            }
        });
    
    let routes = info.or(status).or(submit_block);
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

fn with_storage(
    storage: Arc<tokio::sync::Mutex<dyn StorageService<DagBlock> + Send>>
) -> impl Filter<Extract = (Arc<tokio::sync::Mutex<dyn StorageService<DagBlock> + Send>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || storage.clone())
}
```

---

## Error Handling Patterns

All ICN APIs use the `CommonError` type for error handling. Here are common patterns:

```rust
use icn_common::CommonError;

// Handle specific error types
match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(CommonError::InvalidInputError(msg)) => eprintln!("Invalid input: {}", msg),
    Err(CommonError::PolicyDenied(msg)) => eprintln!("Policy denied: {}", msg),
    Err(CommonError::InsufficientMana) => eprintln!("Not enough mana"),
    Err(e) => eprintln!("Other error: {:?}", e),
}

// Convert errors for API responses
fn api_handler() -> Result<ApiResponse, ApiError> {
    let result = some_icn_operation()?;
    Ok(ApiResponse::success(result))
}
```

---

## Testing Patterns

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_job_submission() {
        let ctx = create_test_runtime_context();
        let job_json = create_test_job_json();
        
        let result = host_submit_mesh_job(&ctx, &job_json).await;
        assert!(result.is_ok());
        
        let job_id = result.unwrap();
        assert!(!job_id.to_string().is_empty());
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_complete_workflow() {
    // Setup
    let storage = Arc::new(tokio::sync::Mutex::new(InMemoryDagStore::new()));
    let ledger = create_test_ledger();
    let reputation_store = create_test_reputation_store();
    
    // Execute workflow
    let result = complete_job_workflow().await;
    assert!(result.is_ok());
    
    // Verify state
    let final_balance = ledger.get_balance(&test_did);
    assert_eq!(final_balance, expected_balance);
}
```

---

This comprehensive API reference covers all the major public APIs, functions, and components in the ICN core workspace. Each section includes detailed type definitions, usage examples, and integration patterns to help developers understand and use the ICN system effectively.

For the most up-to-date API documentation, refer to the generated rustdoc documentation by running:

```bash
cargo doc --open --no-deps
```