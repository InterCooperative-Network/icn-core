# ICN Core – Complete Project Context

## Purpose
`icn-core` is the **production-ready Rust workspace** for the InterCooperative Network (ICN). It provides **77-82% complete** deterministic libraries and comprehensive frontend applications for the federated infrastructure stack that supports cooperative, post‑capitalist coordination and enables autonomous federated systems without relying on traditional state or corporate structures.

**Current Status**: This is **not a prototype** - it's working infrastructure with real P2P networking, cross-node job execution, comprehensive governance/economic systems, and complete frontend applications.

## ICN Mission & Philosophy

**Replace every major function of the state and corporation with programmable, federated, democratic infrastructure—built for actual human needs and scalable solidarity.**

### Core Philosophical Principles
- **Anti-Capitalist Design**: Every choice prioritizes collective benefit over extraction and individual optimization
- **Nonviolent Infrastructure**: Replace systemic violence with cooperative coordination mechanisms
- **Revolutionary Pluralism**: Enable local autonomy within networked solidarity
- **Memetic Security**: Resistance to capture and cooptation through humorous and viral tactics
- **Regenerative Systems**: Ecological and social regeneration patterns embedded in design
- **Dignity & Autonomy**: Technology that enhances human agency rather than controlling it

### Strategic Vision
- **Systemic Sovereignty**: Fully autonomous federated systems independent of nation-state control
- **Consciousness Architecture**: Programmable layers of collective awareness and decision-making
- **Post-Capitalist Coordination**: Tools for economic organization beyond market mechanisms
- **Collective Liberation**: Technology infrastructure for universal human flourishing

## Current State: Production-Ready Infrastructure

### **What Works Today (Current Capabilities)**

#### **Multi-Node P2P Federation ✅**
- Real libp2p networking with Gossipsub messaging and Kademlia DHT
- Automatic peer discovery and federation bootstrap
- Cross-federation coordination and trust establishment
- 3+ node networks verified working with containerized devnet

#### **Cross-Node Job Execution ✅**
- Complete mesh job pipeline with bidding and executor selection
- WASM execution with security constraints and resource limits
- Cryptographic receipts with content-addressed storage
- Real networking verified across multiple nodes

#### **Democratic Governance ✅**
- Proposal creation with CCL contract compilation
- Voting mechanisms with quorum enforcement and signature verification
- Member management including invite/remove operations
- Policy execution that affects network parameters and behavior
- Delegation and revocation of voting power

#### **Economic Management ✅**
- Mana allocation and time-based regeneration
- Resource accounting for all operations with persistent transaction logs
- Multi-backend persistence (SQLite, PostgreSQL, RocksDB, Sled)
- Economic policy enforcement preventing resource abuse
- Token management and scoped resource allocation

#### **Identity & Security ✅**
- DID-based authentication with Ed25519 signatures and secure key management
- Complete credential lifecycle (issuance, verification, revocation)
- Zero-knowledge proof system for privacy-preserving operations
- Age verification, membership proofs, reputation thresholds

#### **Comprehensive API Ecosystem ✅**
- 60+ HTTP endpoints covering all functionality
- TypeScript SDK with automatic generation from Rust traits
- Authentication via API keys and bearer tokens
- Real-time WebSocket support (planned)
- Comprehensive error handling and monitoring

#### **Frontend Applications ✅**
- **4 complete applications** across Web/Mobile/Desktop platforms
- Cross-platform component library (Tamagui)
- TypeScript SDK for all frontend integrations
- Demo modes and production-ready interfaces

## Complete Architecture Principles

### **Technical Foundation (Production-Ready)**
- **Comprehensive Modularity**: 18-crate workspace with clear responsibilities and trait-based interfaces
- **Error-First Programming**: All crates return `Result<T, CommonError>` with detailed error variants
- **Deterministic Execution**: All core logic is predictable and verifiable across nodes
- **Multi-Backend Support**: Storage (PostgreSQL, RocksDB, SQLite, Sled) and networking options
- **WASM-First Contracts**: CCL compiles to WASM for deterministic, sandboxed policy execution

### **Identity & Federation (95% Complete)**
- **Federated Identity**: Complete DID lifecycle with Ed25519 cryptographic signatures
- **Scoped Federation**: Nodes interact via identity-scoped federation protocols
- **Zero-Knowledge Privacy**: Age verification, membership proofs, credential disclosure
- **Three-Tier Topology**: Cooperatives (economic units) → Communities (civic/social) → Federations (coordination layer)

### **Governance & Economics (82% Complete)**
- **Governance as Code**: All bylaws, voting mechanisms, and policies encoded in CCL
- **Purpose-Bound Economics**: Mana system for resource coordination, not financial speculation
- **Multi-Token Support**: Scoped tokens for specific capabilities and resource types
- **Event Sourcing**: Complete audit trails with tamper-evident history

### **Storage & Execution (85% Complete)**
- **Multi-Backend DAG**: Content-addressed storage with PostgreSQL, RocksDB, SQLite, Sled support
- **WASM Runtime**: Deterministic execution with host ABI and comprehensive sandboxing
- **Receipt Anchoring**: All significant actions emit signed execution receipts
- **Real Networking**: libp2p with Gossipsub and Kademlia DHT for P2P communication

## Complete Crate Ecosystem

### **Core Infrastructure (100% Complete)**
- **`icn-common`** – Shared types, error handling, cryptographic primitives (DIDs, CIDs, signatures)
- **`icn-runtime`** – Node orchestration, WASM execution environment, job management, Host-ABI
- **`icn-api`** – HTTP API definitions, TypeScript generation, comprehensive DTOs (60+ endpoints)
- **`icn-protocol`** – P2P message formats, protocol definitions, serialization standards

### **Identity & Security (95% Complete)**
- **`icn-identity`** – Complete DID management, credential lifecycle, Ed25519 signatures, ZK proofs
- **`icn-dag`** – Content-addressed storage with PostgreSQL, RocksDB, SQLite, Sled backends
- **`icn-zk`** – Zero-knowledge circuits (age verification, membership, reputation thresholds)

### **Governance & Economics (82% Complete)**
- **`icn-governance`** – Proposal engine, voting mechanisms, CCL compilation, policy execution
- **`icn-economics`** – Mana accounting, regeneration, economic policy enforcement, token management
- **`icn-reputation`** – Trust scoring, contribution tracking, reputation algorithms
- **`icn-eventstore`** – Event sourcing utilities, JSON Lines format, complete audit trails

### **Networking & Computation (78% Complete)**
- **`icn-network`** – P2P networking with libp2p (Gossipsub, Kademlia DHT, peer discovery)
- **`icn-mesh`** – Distributed job scheduling, bidding, executor selection, cross-node execution

### **Developer Tools & SDKs (90% Complete)**
- **`icn-cli`** – Command-line interface (federation management, job submission, governance)
- **`icn-node`** – Main daemon binary with Axum HTTP server and comprehensive authentication
- **`icn-sdk`** – High-level Rust SDK for HTTP API interactions with type safety
- **`icn-templates`** – Governance template management and CCL pattern library
- **`job-audit`** – Job auditing, compliance tracking, audit trail management

### **Frontend Applications**

#### **Cross-Platform Applications (React Native + Tamagui)**
- **`apps/wallet-ui`** (60% complete) – Secure DID and key management (iOS/Android/Web/Desktop)
- **`apps/agoranet`** (60% complete) – Governance deliberation and voting platform (iOS/Android/Web/Desktop)

#### **Web Applications (React + TypeScript)**
- **`apps/web-ui`** (70% complete) – Federation administration dashboard with demo mode
- **`apps/explorer`** (65% complete) – DAG viewer and network activity browser with D3.js visualization

#### **Shared Frontend Infrastructure**
- **`packages/ui-kit`** (70% complete) – Cross-platform component library with Tamagui
- **`packages/ts-sdk`** (80% complete) – TypeScript SDK with comprehensive API coverage
- **`packages/ccl-visual-editor`** (30% complete) – Visual contract editor (planned)

## System Flow: Complete Production Pipeline

### **1. Federation Bootstrap & Networking**
```
Node Startup → libp2p Service → Automatic Peer Discovery → Federation Join
     ↓
Multi-node Network (Gossipsub + Kademlia DHT)
     ↓
Cross-federation coordination and trust establishment
```

### **2. Mesh Job Execution Pipeline**
```
Job Submission (CLI/API/Web UI) → Mana Validation → Network Announcement
     ↓
Bid Collection → Reputation-Based Executor Selection → Job Assignment
     ↓
WASM Execution → Receipt Generation → DAG Anchoring → Reputation Update
```

### **3. Governance & Decision Making**
```
Proposal Creation (Web UI/CLI) → CCL Compilation → Network Broadcast
     ↓
Deliberation Phase → Voting Collection → Quorum Check
     ↓
Execution (if passed) → Parameter Updates → DAG Anchoring → Audit Trail
```

### **4. Frontend Integration**
```
Web/Mobile UI → TypeScript SDK → HTTP API (60+ endpoints) → Node Runtime
     ↓
Real-time updates via WebSocket connections (planned)
```

### **5. Zero-Knowledge Privacy**
```
Credential Issuance → ZK Circuit Selection → Proof Generation → Verification
     ↓
Age verification, membership proofs, reputation thresholds
```

## Complete Development Infrastructure

### **Backend Development (Rust)**
```bash
# Setup and validation
just setup && just build
just test           # Comprehensive test suite
just lint           # Clippy + formatting
just validate       # Complete validation

# Multi-node testing
just devnet         # 3-node containerized federation
just health-check   # Federation health validation
```

### **Frontend Development**
```bash
# Frontend environment setup
just setup-frontend
just install-frontend

# Development servers
just dev-frontend   # All apps simultaneously
just dev-web-ui     # Federation dashboard
just dev-explorer   # Network explorer
just dev-agoranet   # Governance interface
just dev-wallet     # DID/key management

# Cross-platform
just dev-mobile     # React Native (iOS/Android)
just dev-desktop    # Tauri desktop apps
```

### **Complete Stack**
```bash
# Full environment
just setup-all
just validate-all-stack
just build-all-stack
```

## Production Services Architecture

### **Service Configuration Status**
| Component | Stub Service | Production Service | Status |
|-----------|--------------|-------------------|---------|
| **Mesh Networking** | `StubMeshNetworkService` | `DefaultMeshNetworkService` | ✅ Ready |
| **Cryptographic Signing** | `StubSigner` | `Ed25519Signer` | ✅ Ready |
| **DAG Storage** | `StubDagStore` | PostgreSQL/RocksDB/SQLite/Sled | ✅ Ready |
| **P2P Networking** | N/A | `LibP2pNetworkService` | ✅ In Use |
| **Governance** | N/A | `GovernanceModule` | ✅ In Use |
| **Reputation** | N/A | `ReputationStore` | ✅ In Use |

### **Multi-Backend Storage Support**
- **PostgreSQL** - Production database backend for scalable deployments
- **RocksDB** - High-performance embedded database for single-node setups
- **SQLite** - Single-file deployment for development and small federations
- **Sled** - Pure Rust embedded database for cross-platform compatibility

## Current Phase: Operational Excellence (Phase 5)

### **Key Finding: Configuration Management, Not Missing Features**
The remaining 18-23% completion is primarily **configuration management** and **operational polish**, not missing core functionality. Production services exist and work - they need proper default configuration.

### **Immediate Priorities**
- [ ] Complete service configuration management (ensure production services by default)
- [ ] Scale testing to 10+ node federations
- [ ] Production monitoring and alerting integration
- [ ] Complete frontend application development
- [ ] Cross-platform mobile app deployment

### **What's Actually Working**
- Real P2P federation with 3+ nodes verified
- Cross-node mesh computing with job execution and cryptographic receipts
- Complete governance system with proposals, voting, and policy execution
- Economic resource management with mana regeneration and reputation
- Identity management with DID authentication and ZK proofs
- Comprehensive APIs with authentication and monitoring
- Frontend applications with demo modes and production interfaces

## Development & Governance Guidelines

### **Technical Standards (Production Quality)**
- Use canonical data types from `icn-common` and API contracts from `icn-api`
- Maintain deterministic logic; avoid wall-clock time or unseeded randomness
- Comprehensive testing: unit, integration, and E2E tests required
- Security-first: All economic/governance logic requires adversarial testing
- Multi-backend support: Storage and networking allow multiple implementations

### **Cooperative Values Integration**
- **Design for Mutual Aid**: Prioritize collective benefit over individual optimization
- **Ensure Participatory Governance**: Governance mechanisms remain accessible and democratic
- **Prevent Centralization**: Avoid single points of failure or control
- **Support Local Autonomy**: Enable communities to govern themselves within federation protocols

### **Production Deployment Considerations**
- All services production-ready with comprehensive error handling
- Multi-backend storage for different deployment scenarios
- Authentication and authorization for all endpoints
- Monitoring and metrics for operational visibility
- Cross-platform frontend applications for user accessibility

---

## Cooperative Infrastructure Vision (Implementation Roadmap)

### **Currently Implemented Cooperative Features**
- **Democratic Governance**: Proposal creation, voting, delegation, policy execution
- **Member Management**: DID-based identity, credential verification, trust relationships
- **Resource Sharing**: Mesh computing, mana-based resource allocation
- **Transparent Operations**: Complete audit trails, DAG-anchored history
- **Federated Coordination**: Cross-federation trust and coordination protocols

### **Next Phase Cooperative Features (Phase 6-8)**
- **Cooperative Banking**: Mutual credit, time banking, democratic loans
- **Mutual Aid Networks**: Emergency response, resource sharing, skill matching
- **Supply Chain Cooperation**: Sourcing, quality assurance, fair trade coordination
- **Worker Cooperative Tools**: Profit sharing, democratic workplace coordination
- **Climate Action**: Carbon credits, renewable energy sharing, sustainability metrics

### **Advanced Democracy & Justice (Future)**
- **Liquid Democracy**: Enhanced delegation with revocable trust chains
- **Consensus Building**: Tools for achieving agreement beyond majority voting
- **Participatory Budgeting**: Multi-round democratic resource allocation
- **Transformative Justice**: Community-based accountability and healing processes

---

## Next Steps for Contributors

1. **Understand Current State**: This is production-ready infrastructure, not early-stage development
2. **Read Implementation Status**: Check `docs/status/STATUS.md` for current 77-82% completion details
3. **Choose Your Area**: Backend (Rust), Frontend (React/React Native), or Governance (CCL)
4. **Start with Working Systems**: Build on existing production-ready capabilities
5. **Think Production Quality**: All contributions should meet production standards

**ICN Core is building the infrastructure for a cooperative digital civilization. Every contribution helps replace centralized systems with democratic, federated alternatives.**

