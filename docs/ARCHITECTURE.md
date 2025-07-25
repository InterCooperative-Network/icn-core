# ICN Core Architecture

> **⚠️ Development Status**: This document describes the intended architecture. Many components are currently stub implementations or incomplete features.

This document provides a comprehensive overview of the ICN Core architecture, including crate relationships, data flow patterns, and system design principles.

## Overview

The InterCooperative Network (ICN) Core is designed as a modular, distributed system that enables cooperative digital infrastructure. The architecture follows a layered approach with clear separation of concerns and well-defined interfaces between components.

**Important Note**: While the architecture is well-designed, many implementations are currently stubs or prototypes. Refer to individual crate documentation and look for `todo!()` macros to understand current implementation status.

## Architecture Principles

### 1. Layered Architecture
- **Foundation Layer**: Common utilities and types (`icn-common`)
- **Protocol Layer**: Message formats and network protocols (`icn-protocol`)
- **Domain Layer**: Business logic and core functionality
- **Service Layer**: API interfaces and runtime orchestration
- **Application Layer**: User interfaces and binaries

### 2. Modularity & Decoupling
- Each crate has a single, well-defined responsibility
- Interfaces are defined using traits for testability and flexibility
- Dependencies flow unidirectionally through the layers

### 3. Deterministic Execution
- All core runtime logic is deterministic and reproducible
- Random operations are abstracted through seeded generators
- Time-based operations use provided time sources

### 4. Async-First Design
- Network operations are asynchronous by default
- Storage operations support both sync and async interfaces
- Message passing uses async channels and streams

## Crate Dependency Graph

```
                    ┌─────────────────┐
                    │   icn-common    │
                    │ (foundation)    │
                    └─────────────────┘
                             │
                    ┌─────────────────┐
                    │  icn-protocol   │
                    │ (wire formats)  │
                    └─────────────────┘
                             │
              ┌──────────────┬──────────────┬──────────────┐
              │              │              │              │
     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
     │ icn-identity│ │   icn-dag   │ │  icn-crdt   │ │icn-economics│
     │ (DIDs, VCs) │ │ (storage)   │ │   (sync)    │ │ (mana, $)   │
     └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘
              │              │              │              │
              └──────────────┴──────────────┴──────────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
     │icn-governance│ │ icn-mesh    │ │icn-reputation│
     │ (proposals) │ │ (jobs)      │ │ (scoring)   │
     └─────────────┘ └─────────────┘ └─────────────┘
              │              │              │
              └──────────────┼──────────────┘
                             │
                    ┌─────────────────┐
                    │  icn-network    │
                    │ (P2P, libp2p)   │
                    └─────────────────┘
                             │
                    ┌─────────────────┐
                    │  icn-runtime    │
                    │ (orchestration) │
                    └─────────────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
     │   icn-api   │ │   icn-cli   │ │   icn-node  │
     │ (traits)    │ │ (command)   │ │ (binary)    │
     └─────────────┘ └─────────────┘ └─────────────┘
              │              │              │
              └──────────────┼──────────────┘
                             │
                    ┌─────────────────┐
                    │      icn-zk     │
                    │ (zero-knowledge)│
                    └─────────────────┘
```

## Core Components

### Foundation Layer

#### `icn-common`
**Purpose**: Shared types, utilities, and constants
**Key Types**:
- `Did` - Decentralized identifiers
- `Cid` - Content identifiers for DAG addressing
- `CommonError` - Unified error handling
- `Transaction` - Basic transaction structure
- `DagBlock` - Content-addressed data blocks

**Dependencies**: Minimal (serde, crypto primitives)

#### `icn-protocol`
**Purpose**: Wire format definitions and message types
**Key Types**:
- `ProtocolMessage` - All network message envelopes
- `MessagePayload` - Specific message type variants
- `FederationJoinRequest/Response` - Federation handshake
- Version negotiation and compatibility

**Dependencies**: `icn-common`

### Domain Layer

#### `icn-identity`
**Purpose**: Decentralized identity management
**Key Components**:
- DID document management and resolution
- Verifiable credential issuance and verification
- Cryptographic key management
- Execution receipt signing and verification

**Supported DID Methods**:
- `did:key` - Key-based identifiers
- `did:web` - Web-hosted DID documents
- `did:peer` - Peer-to-peer identifiers

**Dependencies**: `icn-common`, crypto libraries

#### `icn-dag`
**Purpose**: Content-addressed storage with DAG semantics
**Key Components**:
- Storage service trait abstraction
- Multiple backend implementations (memory, file, RocksDB, SQLite, Sled)
- Content addressing and integrity verification
- Pinning and TTL management
- Merkle root computation for synchronization

**Storage Backends**:
- **Memory**: In-memory storage for testing
- **File**: Simple file-based storage
- **RocksDB**: High-performance embedded database
- **SQLite**: Lightweight SQL database
- **Sled**: Pure Rust embedded database

**Dependencies**: `icn-common`, `icn-identity`

#### `icn-crdt`
**Purpose**: CRDT-based real-time synchronization modules
**Key Components**:
- G-Counter, PN-Counter, OR-Set, LWW-Register
- Vector clocks and gossip-based replication

**Integration Points**:
- DAG storage for operation logs
- Network layer for state exchange
- Runtime host functions for contract access

**Dependencies**: `icn-common`, `icn-dag`, `icn-network`

#### `icn-economics`
**Purpose**: Economic policy and mana management
**Key Components**:
- Mana account management and regeneration
- Economic policy enforcement
- Resource usage tracking
- Transaction validation and processing

**Mana System**:
- Regenerating capacity credits
- Policy-driven regeneration rates
- Reputation-influenced allocation
- Anti-speculation mechanisms

**Dependencies**: `icn-common`, `icn-identity`, `icn-dag`

#### `icn-governance`
**Purpose**: Proposal and voting mechanisms
**Key Components**:
- Proposal lifecycle management
- Voting systems and tallying
- Quorum and threshold enforcement
- Parameter management and updates

**Governance Features**:
- Multiple voting methods (simple majority, supermajority, etc.)
- Proposal types (parameter changes, upgrades, etc.)
- Persistent storage of decisions
- Federation synchronization
- Advanced democracy primitives (ranked choice, quadratic, delegated)

**Governance Workflow**:
1. Proposal submission and validation
2. Community review period
3. Voting period with chosen method
4. Challenge window for disputes
5. On-chain execution and federation sync

**Dependencies**: `icn-common`, `icn-identity`, `icn-economics`, `icn-dag`

#### `icn-mesh`
**Purpose**: Distributed job execution
**Key Components**:
- Job specification and lifecycle management
- Executor selection and bidding
- Resource requirement matching
- Execution monitoring and results

**Job Execution Flow**:
1. Job submission with resource requirements
2. Executor discovery and bidding
3. Bid evaluation and executor selection
4. Job execution with monitoring
5. Result collection and verification

**Dependencies**: `icn-common`, `icn-identity`, `icn-economics`, `icn-reputation`

#### `icn-reputation`
**Purpose**: Trust and reputation scoring
**Key Components**:
- Reputation score calculation
- Historical performance tracking
- Contribution measurement
- Sybil resistance mechanisms

**Dependencies**: `icn-common`, `icn-identity`, `icn-dag`

### Infrastructure Layer

#### `icn-network`
**Purpose**: P2P networking and communication
**Key Components**:
- NetworkService trait abstraction
- libp2p integration and configuration
- Peer discovery and routing
- Message broadcasting and direct communication

**Network Features**:
- Kademlia DHT for peer discovery
- Gossipsub for message broadcasting
- Multiple transport protocols (TCP, QUIC)
- NAT traversal and connection management

**Dependencies**: `icn-common`, `icn-protocol`, libp2p

#### `icn-runtime`
**Purpose**: System orchestration and WASM execution
**Key Components**:
- RuntimeContext for global state management
- Host ABI for WASM module interaction
- Job execution and lifecycle management
- Resource monitoring and limiting
- Intelligent load balancing and predictive capacity planning
- Automatic recovery and resilience monitoring

**WASM Integration**:
- Sandboxed execution environment
- Resource limits (CPU, memory, time)
- Host function exposure
- Deterministic execution guarantees

**Dependencies**: All domain crates, WASM runtime

**Runtime Initialization Steps**:
1. `NodeConfig` is loaded and environment overrides are applied.
2. A `ServiceConfig` is constructed with networking, storage and signing services.
3. `RuntimeContext::from_service_config` initializes the execution monitor logger and builds the cross-component coordinator.
4. Executor capabilities and federation memberships from `NodeConfig` populate runtime parameters.
5. Job and executor managers are started to process work.

### Service Layer

#### `icn-api`
**Purpose**: External API interfaces and contracts
**Key Components**:
- HTTP API trait definitions
- Request/response DTOs
- Service abstractions
- Client helper functions

**API Categories**:
- Node management (info, status, health)
- DAG operations (put, get, pin, prune)
- Mesh job management
- Governance operations
- Identity and credential management

**Dependencies**: Domain crates for types

#### `icn-zk`
**Purpose**: Zero-knowledge proof circuits
**Key Components**:
- Age verification circuits
- Membership proof circuits
- Reputation threshold proofs
- Range proof circuits
- Batch verification support

**Circuit Types**:
- `AgeOver18Circuit` - Age verification without revealing birth date
- `MembershipCircuit` - Membership proof
- `ReputationCircuit` - Reputation threshold proof
- `BalanceRangeCircuit` - Balance range proof
- `AgeRepMembershipCircuit` - Composite circuit

**Dependencies**: arkworks ecosystem, `icn-common`

### Application Layer

#### `icn-cli`
**Purpose**: Command-line interface
**Key Components**:
- Command parsing and validation
- HTTP client for API communication
- User interaction and feedback
- Configuration management

**Command Categories**:
- Node operations (info, status)
- DAG operations (put, get)
- Mesh job management
- Governance operations
- Network management

**Dependencies**: `icn-api`, `icn-common`, HTTP client

#### `icn-node`
**Purpose**: Main node daemon
**Key Components**:
- HTTP server implementation
- Service orchestration
- Configuration management
- Startup and shutdown procedures

**Server Features**:
- REST API endpoints
- Authentication and authorization
- TLS/HTTPS support
- Metrics and monitoring
- Graceful shutdown

**Dependencies**: All crates

## Developer Tooling Architecture

> **Development Status**: Tooling features are usable but still evolving.

### Components
- **`ccl-lsp`** – Language Server Protocol implementation for CCL
- **`ccl-debug`** – Source-level WASM debugger
- **`ccl-package`** – Package manager for reusable governance modules

### Tooling Flow

```
IDE/Editor → ccl-lsp → ccl-debug → ccl-package
```

## Data Flow Patterns

### 1. Job Submission Flow

```
CLI/API → icn-node → icn-runtime → icn-mesh
                                      ↓
                                 icn-economics (mana check)
                                      ↓
                                 icn-network (broadcast)
                                      ↓
                                 Executor Selection
                                      ↓
                                 Job Execution
                                      ↓
                                 icn-identity (receipt signing)
                                      ↓
                                 icn-dag (receipt storage)
                                      ↓
                                 icn-reputation (score update)
```

### 2. Governance Proposal Flow

```
CLI/API → icn-node → icn-governance → icn-dag (proposal storage)
                                          ↓
                                     Voting Period
                                          ↓
                                     Vote Collection
                                          ↓
                                     Quorum Check
                                          ↓
                                     Policy Update
                                          ↓
                                     icn-network (federation sync)
```

### 3. Identity Resolution Flow

```
Request → icn-identity → DID Resolution
                              ↓
                         Method Handler
                              ↓
                         Public Key Retrieval
                              ↓
                         Signature Verification
                              ↓
                         Credential Validation
```

### 4. DAG Storage Flow

```
Data → icn-dag → Storage Backend → Persistence
                      ↓
                 CID Generation
                      ↓
                 Metadata Storage
                      ↓
                 Index Updates
                      ↓
                 Sync Root Computation
```

## Network Architecture

### P2P Network Topology

```
     ┌─────────────┐
     │    Node A   │
     │  (Bootstrap)│
     └─────────────┘
          │     │
          │     │
    ┌─────────────┐   ┌─────────────┐
    │    Node B   │───│    Node C   │
    │  (Worker)   │   │  (Worker)   │
    └─────────────┘   └─────────────┘
          │               │
          └───────────────┘
```

### Federation Architecture

```
Federation A           Federation B
┌─────────────┐       ┌─────────────┐
│    Node A1  │       │    Node B1  │
│    Node A2  │═══════│    Node B2  │
│    Node A3  │       │    Node B3  │
└─────────────┘       └─────────────┘
      │                       │
      │                       │
┌─────────────┐       ┌─────────────┐
│    Node A4  │       │    Node B4  │
│    Node A5  │       │    Node B5  │
└─────────────┘       └─────────────┘
```

### Federation DAG Synchronization Protocol

This protocol exchanges DAG roots between federated nodes and transfers missing
blocks. CRDT merges ensure deterministic convergence.

```text
Node A DAG ── Sync Request ──> Node B DAG
         └── Missing Blocks <──┘
                 ▲
                 └─ CRDT Merge
```

## Security Architecture

### Trust Boundaries

```
┌─────────────────────────────────────────┐
│              Trusted Zone               │
│  ┌─────────────────────────────────────┐│
│  │           ICN Runtime             ││
│  │  ┌─────────────────────────────────┐││
│  │  │          WASM Sandbox         │││
│  │  │        (Untrusted Code)       │││
│  │  └─────────────────────────────────┘││
│  └─────────────────────────────────────┘│
└─────────────────────────────────────────┘
```

### Cryptographic Layers

1. **Identity Layer**: DID-based authentication
2. **Message Layer**: Signed protocol messages
3. **Storage Layer**: Content-addressed integrity
4. **Transport Layer**: TLS/HTTPS encryption
5. **Zero-Knowledge Layer**: Privacy-preserving proofs

## Performance Considerations

### Concurrency Model

- **Async Runtime**: Tokio-based async execution
- **Message Passing**: Channel-based communication
- **Shared State**: Arc + Mutex for thread safety
- **Batch Processing**: Efficient batch operations

### Storage Optimization

- **Content Addressing**: Deduplication through CIDs
- **Caching**: LRU caches for frequently accessed data
- **Indexing**: Efficient lookups and queries
- **Compression**: Optional compression for large blocks

### Network Optimization

- **Connection Pooling**: Reuse of network connections
- **Batch Verification**: Efficient cryptographic operations
- **Circuit Breakers**: Fault tolerance and recovery
- **Rate Limiting**: Protection against abuse

## Testing Architecture

### Test Hierarchy

```
┌─────────────────────────────────────────┐
│           Integration Tests             │
│  ┌─────────────────────────────────────┐│
│  │           Unit Tests              ││
│  │  ┌─────────────────────────────────┐││
│  │  │        Property Tests         │││
│  │  └─────────────────────────────────┘││
│  └─────────────────────────────────────┘│
└─────────────────────────────────────────┘
```

### Test Strategy

- **Unit Tests**: Individual component testing
- **Integration Tests**: Cross-component interaction
- **Property Tests**: Invariant validation
- **End-to-End Tests**: Full system validation
- **Performance Tests**: Load and stress testing

## Deployment Architecture

### Single Node Deployment

```
┌─────────────────────────────────────────┐
│                Node                     │
│  ┌─────────────┐  ┌─────────────────────┐│
│  │  ICN Node   │  │     Storage         ││
│  │  (Binary)   │  │  ┌─────────────────┐││
│  │             │  │  │   DAG Store     │││
│  │             │  │  │   Mana Ledger   │││
│  │             │  │  │   Governance    │││
│  │             │  │  └─────────────────┘││
│  └─────────────┘  └─────────────────────┘│
└─────────────────────────────────────────┘
```

### Multi-Node Federation

```
┌─────────────────────────────────────────┐
│            Load Balancer                │
└─────────────────────────────────────────┘
                      │
    ┌─────────────────┼─────────────────┐
    │                 │                 │
┌─────────┐      ┌─────────┐      ┌─────────┐
│ Node 1  │      │ Node 2  │      │ Node 3  │
│         │      │         │      │         │
│ Storage │      │ Storage │      │ Storage │
└─────────┘      └─────────┘      └─────────┘
```

## Configuration Architecture

### Configuration Hierarchy

```
Command Line Args
       ↓
Environment Variables
       ↓
Configuration Files
       ↓
Default Values
```

### Configuration Categories

- **Network**: P2P settings, bootstrap peers
- **Storage**: Backend selection, paths
- **Identity**: Key management, DID configuration
- **Economics**: Mana policies, regeneration rates
- **Governance**: Voting parameters, thresholds
- **Security**: TLS certificates, API keys

## Monitoring and Observability

### Metrics Collection

```
ICN Components → Prometheus Client → Metrics Endpoint
                                            ↓
                                    Prometheus Server
                                            ↓
                                      Grafana Dashboard
```

### Logging Architecture

```
Components → Structured Logging → Log Aggregation → Analysis
                                        ↓
                                   Audit Trail
```

### Health Checks

- **Liveness**: Process health and responsiveness
- **Readiness**: Service availability and dependencies
- **Startup**: Initialization progress and completion

## Future Architecture Considerations

### Planned Enhancements

1. **Horizontal Scaling**: Improved federation protocols
2. **Edge Computing**: Lightweight node variants
3. **Mobile Support**: Resource-constrained deployments
4. **Quantum Resistance**: Post-quantum cryptography
5. **Machine Learning**: Intelligent resource allocation

### Scalability Improvements

- **Sharding**: Distributed storage partitioning
- **Caching**: Multi-level caching strategies
- **Compression**: Advanced compression algorithms
- **Optimization**: Performance profiling and tuning

This architecture provides a solid foundation for cooperative digital infrastructure while maintaining flexibility for future enhancements and scaling requirements. 