# ICN SDK (`icn-sdk`)

This crate provides a high-level Rust SDK for interacting with InterCooperative Network (ICN) nodes via their HTTP API. It offers a convenient, type-safe interface for building applications that integrate with ICN.

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-sdk` crate is responsible for:

* **HTTP Client Interface**: Providing a high-level client for ICN node HTTP APIs
* **Type Safety**: Wrapping HTTP calls with Rust types for compile-time safety
* **Async Support**: Full async/await support for non-blocking operations  
* **Error Handling**: Consistent error handling using `reqwest::Error`
* **Developer Experience**: Simplified API for common ICN operations

## Key Features

### IcnClient
The main client for interacting with ICN nodes:

```rust
use icn_sdk::IcnClient;

let client = IcnClient::new("http://localhost:8080")?;
```

### Health and Status Monitoring
```rust
// Get node information
let info = client.info().await?;
println!("Node: {} v{}", info.name, info.version);

// Check node status
let status = client.status().await?;
println!("Node is {}", status.status);

// Health check
let health = client.health().await?;
println!("Health: {}", health.status);

// Readiness check
let ready = client.ready().await?;
println!("Ready: {}", ready.ready);
```

## API Coverage

### Node Management
- `info()` - Get node information
- `status()` - Get node status
- `health()` - Health check endpoint
- `ready()` - Readiness check endpoint
- `metrics()` - Prometheus metrics

### Mesh Computing
- `submit_mesh_job()` - Submit a job to the mesh network
- `list_mesh_jobs()` - List all mesh jobs
- `mesh_job(id)` - Get details of a specific job
- `submit_mesh_receipt()` - Submit execution receipt
- `mesh_stub_bid()` - Submit a bid for a job
- `mesh_stub_receipt()` - Submit receipt stub

### Governance
- `list_proposals()` - List governance proposals
- `proposal(id)` - Get proposal details
- `submit_proposal()` - Submit new proposal
- `cast_vote()` - Cast vote on proposal
- `delegate()` - Delegate voting power
- `revoke()` - Revoke delegation
- `close_vote()` - Close voting on proposal
- `execute()` - Execute approved proposal

### DAG Operations
- `dag_put()` - Store block in DAG
- `dag_get()` - Retrieve block from DAG
- `dag_meta()` - Get block metadata
- `dag_pin()` - Pin block to prevent garbage collection
- `dag_unpin()` - Unpin block
- `dag_prune()` - Prune expired blocks
- `dag_root()` - Get DAG root hash

### Network Operations
- `local_peer_id()` - Get local peer ID
- `connect_peer()` - Connect to remote peer
- `peers()` - List connected peers

### Economic Operations
- `account_mana(did)` - Get mana balance for account
- `submit_transaction()` - Submit economic transaction
- `token_classes()` - List token classes
- `create_token_class()` - Create new token class
- `mint_tokens()` - Mint tokens
- `transfer_tokens()` - Transfer tokens
- `burn_tokens()` - Burn tokens

### Identity and Security
- `keys()` - Get node keys
- `reputation(did)` - Get reputation score
- `revoke_credential()` - Revoke a credential by CID
- `verify_revocation()` - Verify a revocation proof

### Data Operations
- `data_query()` - Query stored data
- `upload_contract()` - Upload smart contract

### Federation Management
- `federation_peers()` - List federation peers
- `add_federation_peer()` - Add peer to federation
- `federation_join()` - Join federation
- `federation_leave()` - Leave federation
- `federation_status()` - Get federation status

## Usage Examples

### Basic Node Interaction
```rust
use icn_sdk::IcnClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = IcnClient::new("http://localhost:8080")?;
    
    // Get node info
    let info = client.info().await?;
    println!("Connected to {} v{}", info.name, info.version);
    
    // Check health
    let health = client.health().await?;
    println!("Node health: {}", health.status);
    
    Ok(())
}
```

### Mesh Job Management
```rust
use serde_json::json;

// Submit a job
let job_request = json!({
    "manifest_cid": "bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354",
    "cost_mana": 50
});

let job_response = client.submit_mesh_job(&job_request).await?;
println!("Job submitted: {}", job_response);

// List all jobs
let jobs = client.list_mesh_jobs().await?;
println!("Current jobs: {}", jobs);
```

### Governance Participation
```rust
use serde_json::json;

// Submit a proposal
let proposal = json!({
    "title": "Increase mesh job timeout",
    "description": "Increase timeout from 60 to 120 seconds",
    "changes": [{
        "parameter": "mesh_job_timeout",
        "new_value": "120"
    }]
});

let response = client.submit_proposal(&proposal).await?;
println!("Proposal submitted: {}", response);

// Cast a vote
let vote = json!({
    "proposal_id": "proposal_123",
    "vote": "approve"
});

let response = client.cast_vote(&vote).await?;
println!("Vote cast: {}", response);
```

### DAG Operations
```rust
use serde_json::json;

// Store data in DAG
let block = json!({
    "data": "Hello, ICN!",
    "timestamp": 1234567890
});

let response = client.dag_put(&block).await?;
println!("Block stored: {}", response);

// Retrieve data
let cid = json!("bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354");
let block = client.dag_get(&cid).await?;
println!("Retrieved block: {}", block);
```

### Economic Operations
```rust
// Check mana balance
let balance = client.account_mana("did:key:alice").await?;
println!("Mana balance: {}", balance);

// Transfer tokens
let transfer = json!({
    "from": "did:key:alice",
    "to": "did:key:bob",
    "amount": 100,
    "token_class": "local.food.token"
});

let response = client.transfer_tokens(&transfer).await?;
println!("Transfer result: {}", response);
```

### Federation Management
```rust
// Check federation status
let status = client.federation_status().await?;
println!("Federation status: {}", status);

// Join a federation
let join_request = json!({
    "peer_id": "12D3KooWExample",
    "federation_id": "food-coop-federation"
});

let response = client.federation_join(&join_request).await?;
println!("Federation join result: {}", response);
```

## Error Handling

All methods return `Result<T, reqwest::Error>` for consistent error handling:

```rust
match client.info().await {
    Ok(info) => println!("Node: {}", info.name),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Configuration

The client can be configured with different base URLs for different environments:

```rust
// Local development
let client = IcnClient::new("http://localhost:8080")?;

// Remote node
let client = IcnClient::new("https://node.icn.example.com")?;

// Custom port
let client = IcnClient::new("http://localhost:3000")?;
```

## Authentication

The SDK uses the underlying `reqwest` client which can be configured with authentication:

```rust
use reqwest::Client;

let http_client = Client::builder()
    .default_headers(/* auth headers */)
    .build()?;

// Custom client creation would need to be exposed by the SDK
```

## Response Types

The SDK provides structured response types for health and readiness checks:

```rust
pub struct HealthStatus {
    pub status: String,
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub checks: HealthChecks,
}

pub struct ReadinessStatus {
    pub ready: bool,
    pub timestamp: u64,
    pub checks: ReadinessChecks,
}
```

Most other endpoints return `serde_json::Value` for flexibility.

## Testing

```bash
# Run all tests
cargo test -p icn-sdk

# Test with a running node
cargo test -p icn-sdk --features integration
```

## Integration with ICN Ecosystem

The SDK is designed to work with:

* **ICN Node**: Primary target for HTTP API calls
* **ICN CLI**: Can be used to build alternative CLI tools
* **Web Applications**: JavaScript/TypeScript bindings via WASM
* **Monitoring Tools**: Prometheus metrics integration
* **Custom Applications**: Any Rust application needing ICN integration

## Performance Considerations

* **Connection Pooling**: Uses `reqwest::Client` with connection pooling
* **Async Operations**: All operations are async for non-blocking I/O
* **Batch Operations**: Consider batching requests for better performance
* **Caching**: Implement client-side caching for frequently accessed data

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

**Areas for contribution:**
- Additional API endpoint wrappers
- Strongly-typed request/response structures
- Authentication support
- WebAssembly bindings
- Example applications
- Performance optimizations

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 