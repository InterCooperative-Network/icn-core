# ICN Core Traits

Shared traits and interfaces for ICN Core components to break circular dependencies.

## Overview

This crate contains common abstractions that multiple ICN crates need to implement or depend on, without creating circular dependency chains. It provides a clean separation of interfaces from implementations.

## Features

- **Network Traits**: `NetworkService`, `NetworkServiceProvider` for P2P networking
- **Reputation Traits**: `ReputationStore`, `ReputationProvider` for reputation management  
- **Governance Traits**: `GovernanceProvider`, `ProposalProvider`, `VotingProvider` for democratic governance
- **Economic Traits**: `EconomicProvider`, `ManaProvider`, `ResourceProvider` for economic coordination
- **Mesh Traits**: `MeshProvider`, `JobProvider`, `ExecutorProvider` for distributed computation

## Usage

Add this crate as a dependency to use shared traits:

```toml
[dependencies]
icn-core-traits = { path = "../icn-core-traits" }
```

Then implement the relevant traits:

```rust
use icn_core_traits::{NetworkService, ReputationStore};

struct MyNetworkService;

#[async_trait]
impl NetworkService for MyNetworkService {
    // Implementation...
}
```

## Breaking Circular Dependencies

This crate allows dependent crates to reference shared interfaces without importing the full implementation crates:

- `icn-network` can use `ReputationStore` without depending on `icn-reputation`
- `icn-reputation` can use `NetworkService` without depending on `icn-network`
- All crates can depend on `icn-core-traits` without circular issues

## Architecture

```
icn-core-traits (shared interfaces)
     ↑                ↑
icn-network      icn-reputation
     ↑                ↑
icn-runtime (orchestrates implementations)
```