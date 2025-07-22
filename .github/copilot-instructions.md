# ICN Core - AI Coding Assistant Instructions

> **Active development** cooperative digital economy infrastructure built in Rust with comprehensive frontend applications

## Project Overview

ICN Core is an **ambitious distributed platform** being built to enable federations and cooperatives to coordinate democratically without centralized systems. It aims to provide mesh computing, governance, economics, and identity management across a federated P2P network.

**Architecture**: Rust backend (20+ crates) + React/React Native frontends (4 apps) + TypeScript SDK
**Status**: **Heavy development** - many core features are stubbed and need implementation

## Critical System Flows

### Mesh Job Pipeline (Primary Workflow)
1. **Submission**: `host_submit_mesh_job` via `icn-runtime/src/abi.rs` → validates DID + mana → adds to `RuntimeContext::pending_mesh_jobs`
2. **Bidding**: `JobManager` announces → executors submit bids via `icn-mesh` selection algorithms
3. **Assignment**: `select_executor` uses reputation/price/resources → job state transitions to `Assigned`
4. **Execution**: Assigned executor runs job → produces signed `ExecutionReceipt` 
5. **Anchoring**: `host_anchor_receipt` validates → stores in DAG via `icn-dag` → updates reputation

**Critical**: All operations must validate mana balances and reputation scores. No bypassing economic constraints.

## Core Architecture

### Backend Dependency Flow
```
icn-common (foundation)
├── icn-protocol (wire formats)
├── icn-identity (DID/credentials) 
├── icn-dag (content-addressed storage)
└── icn-economics (mana/policies)
    ├── icn-mesh (jobs/bidding)
    ├── icn-governance (proposals/voting)
    └── icn-network (libp2p P2P)
        └── icn-runtime (orchestration)
            └── icn-node (HTTP server)
```

### Key Crate Responsibilities
- **`icn-runtime`**: WASM execution, Host ABI functions (`host_*`), job orchestration
- **`icn-mesh`**: Distributed job lifecycle, executor selection, bidding algorithms  
- **`icn-economics`**: Mana accounting, resource policies, economic invariants
- **`icn-governance`**: CCL compilation, proposal/voting, democratic decision-making
- **`icn-identity`**: DID management, credential verification, execution receipts
- **`icn-dag`**: Immutable content addressing, receipt anchoring, persistent storage

### Current Focus: Implementing Stubbed Services
**Issue**: Many services are stubbed placeholders that need real implementations
```rust
// CURRENT ISSUE: Many stub implementations need to be replaced
let mesh_network_service = Arc::new(StubMeshNetworkService::new());

// GOAL: Implement real service logic
pub struct DefaultMeshNetworkService {
    // Real implementation with P2P networking, job distribution, etc.
}

impl MeshNetworkService for DefaultMeshNetworkService {
    async fn submit_job(&self, job: MeshJob) -> Result<JobId, MeshError> {
        // TODO: Implement real job submission logic
        // - Validate job specification
        // - Broadcast to network
        // - Handle bidding process
        // - Return job ID
        todo!("Implement real mesh job submission")
    }
}
```

## Development Workflow

### Essential Commands (via `justfile`)
```bash
just setup              # Full dev environment setup
just test               # Run test suite (default features)
just test-all-features  # Full test suite (includes RocksDB) 
just validate           # Format + lint + test
just devnet             # Launch containerized federation
just dev-frontend       # Start all frontend apps
just docs               # Generate documentation
```

### Frontend Development
```bash
just setup-frontend     # Node.js + pnpm setup
just dev-wallet        # DID/key management app
just dev-web-ui        # Federation dashboard
just dev-agoranet      # Governance interface
just dev-explorer      # Network/DAG explorer
```

### Agent Development Cycle
**For every change, follow this pattern:**
1. **Identify stub implementations** - look for `todo!()` macros and `Stub*` services
2. **Implement core functionality** - replace stubs with working logic
3. **Test thoroughly** - `cargo test -p affected-crate` 
4. **Update docs immediately** - never leave documentation stale
5. **Commit with clear messages** - explain what was implemented and why

### Finding Work to Do
```bash
# Find stub implementations and TODOs
grep -r "todo!()" crates/
grep -r "Stub" crates/ --include="*.rs"
grep -r "unimplemented!()" crates/

# Look for placeholder implementations
grep -r "NotImplemented" crates/
grep -r "placeholder" crates/
```

### Commit Standards
```bash
git commit -m "[crate-name] Implement [feature/service]

Replaced stub implementation with working logic:
- Added real P2P networking integration
- Implemented bidding algorithm logic  
- Added proper error handling and validation
- Included comprehensive tests

Resolves: [specific TODO or stub functionality]
Tests: All existing tests pass + new integration tests"
```

## Code Patterns & Conventions

### Host ABI Integration
When adding runtime functionality, follow the established ABI pattern:
```rust
// 1. Define ABI index in icn-runtime/src/abi.rs
pub const ABI_HOST_NEW_FUNCTION: u32 = 27;

// 2. Implement in RuntimeContext 
impl RuntimeContext {
    pub fn handle_new_function(&mut self, params: Params) -> Result<Response> {
        // Validate mana/permissions first
        // Update state atomically
        // Log significant changes
    }
}
```

### Economic Validation Pattern
All resource operations must validate economic constraints:
```rust
// Check mana before any operation
let mana_required = calculate_operation_cost(&operation);
if !self.mana_account.has_sufficient_mana(mana_required) {
    return Err(InsufficientMana);
}

// Perform operation atomically
let result = self.perform_operation(operation)?;

// Charge mana only on success
self.mana_account.spend_mana(mana_required)?;
```

### Testing Patterns
- **Unit tests**: Test individual crate logic in isolation
- **Integration tests**: Cross-crate workflows in `tests/` directory
- **E2E tests**: Full node operations via `just devnet`
- **Deterministic**: Use fixed seeds for randomness in tests

### Error Handling
Use comprehensive error types from `icn-common`:
```rust
pub enum RuntimeError {
    InsufficientMana { required: u64, available: u64 },
    InvalidDid(String),
    JobExecutionFailed { job_id: String, reason: String },
    // ...
}
```

### Input Validation Pattern
Always validate inputs thoroughly:
```rust
pub fn validate_did(did: &str) -> Result<Did, ValidationError> {
    // Format validation
    if !did.starts_with("did:") {
        return Err(ValidationError::InvalidFormat("DID must start with 'did:'"));
    }
    
    // Length and character validation
    if did.len() < 10 || did.len() > 256 {
        return Err(ValidationError::InvalidLength);
    }
    
    // Parse and semantic validation
    let parsed = Did::parse(did)?;
    validate_did_semantic(&parsed)?;
    Ok(parsed)
}
```

## Reading Order for Complex Changes

For mesh/economics/runtime changes:
1. `crates/icn-runtime/src/abi.rs` (Host ABI surface)
2. `crates/icn-runtime/src/context.rs` (`RuntimeContext` state)
3. `crates/icn-mesh/src/lib.rs` (job lifecycle)
4. `crates/icn-economics/src/lib.rs` (mana validation)
5. Relevant integration tests

For governance changes:
1. `crates/icn-governance/src/lib.rs`
2. `icn-ccl/` (CCL compiler)
3. Frontend governance UI in `apps/agoranet/`

## Critical Invariants

### Economic
- **Never bypass mana checks** - all resource usage must be validated
- **Atomic state updates** - mana spending and operations must be atomic
- **Regeneration policies** - mana regeneration follows configured economic policies

### Technical  
- **Determinism required** - no wall-clock time, unseeded randomness, or unpredictable I/O
- **Cryptographic verification** - all receipts must be signed and verifiable
- **Content addressing** - DAG storage must maintain immutability guarantees

### Governance
- **Democratic processes** - proposals must follow proper lifecycle (creation → voting → execution)
- **CCL compilation** - governance policies compile to WASM for execution

## Common Integration Points

### Adding New API Endpoints
1. Define in `crates/icn-api/src/lib.rs`
2. Implement in `crates/icn-node/src/main.rs` 
3. Add to TypeScript SDK generation
4. Update `ICN_API_REFERENCE.md`

### Frontend State Management
- Use TypeScript SDK from `packages/ts-sdk/`
- Maintain consistent patterns across React/React Native apps
- Update shared UI components in `packages/ui-kit/`

### Storage Backend Integration
- Support multiple backends: Sled (default), RocksDB, PostgreSQL, SQLite
- Test with `just test-sled` / `just test-rocksdb` 
- Maintain backend abstraction in `icn-dag`

## Troubleshooting Common Issues

### "Insufficient Mana" Errors
```rust
// Add proper wait time calculation for regeneration
if account.mana_balance < estimated_cost {
    let wait_time = calculate_regeneration_time(
        estimated_cost - account.mana_balance, 
        account.regeneration_rate
    );
    return Err(RuntimeError::InsufficientMana { 
        required: estimated_cost,
        available: account.mana_balance,
        regeneration_wait: wait_time,
    });
}
```

### "No Valid Bids" for Jobs
```bash
# Diagnosis commands
icn-cli network peers --verbose
icn-cli job details --id <job-id>
icn-cli network executors --capabilities <required-caps>
```

### Peer Discovery Issues
```bash
# Check network configuration and connectivity
cat ~/.icn/network-config.toml
icn-cli network ping --peer <peer-id>
netstat -an | grep <icn-port>
```

## Documentation Requirements

- **Update docs immediately** with code changes
- **Include examples** in rustdoc for public APIs
- **Maintain API reference** for all HTTP endpoints  
- **Update architecture docs** for structural changes
- **Keep frontend docs** current with UI changes

Refer to existing `.cursor/rules/*.mdc` files for additional context on architecture, workflow, and troubleshooting patterns.

## Agent Authority & Focus

### Current Phase: Implementation and Development
**Remaining work** is primarily **implementing stubbed functionality** and building out core features that are currently placeholders.

### You Are Empowered To:
- **Implement stubbed services** with real functionality
- **Replace `todo!()` macros** with working code
- **Build out missing features** in existing crate structure
- **Improve API endpoints** and add missing functionality
- **Complete frontend applications** with real backend integration
- **Add comprehensive testing** for new implementations
- **Enhance developer experience** and tooling

### Quality Standards
**Every change must:**
- **Replace stubs with real implementations** not just different stubs
- Be thoroughly tested (`just validate`)
- Include updated documentation explaining the implementation
- Follow established architectural patterns
- Handle errors comprehensively
- Include clear commit messages explaining what was implemented
