# ICN Core Workspace – Context & Rules of Engagement

> **Attention Contributor (Human or AI):** This file defines the specific rules and architectural context for the `icn-core` repository. It complements the global ICN Shared Contributor Rules. Familiarize yourself with both. CI will enforce these guidelines.

---

## 1 · Mission & Current State of `icn-core`

This repository, `icn-core`, is the **production-ready Rust monorepo for the InterCooperative Network's complete infrastructure**. It houses **77-82% complete** deterministic, runtime-critical libraries executed by every ICN node, along with comprehensive frontend applications and developer tools.

**Current Status: Production-Ready Foundation (Phase 5 Complete)**
- **Multi-node P2P federation** with automatic peer discovery ✅
- **Cross-node job execution** with cryptographic verification ✅  
- **Democratic governance** with programmable policies ✅
- **Economic coordination** with mana-based resource management ✅
- **Federated identity** with DID-based authentication ✅
- **Comprehensive HTTP API** with 60+ endpoints ✅
- **Frontend applications** for all major use cases ✅
- **Developer ecosystem** with CLI, SDK, and extensive tooling ✅

**Key Responsibilities of `icn-core`:**

### **Core Infrastructure (100% Complete)**
- **`icn-runtime`** – Node orchestration, WASM execution, and job management
- **`icn-common`** – Shared types, cryptographic primitives, and utilities  
- **`icn-api`** – HTTP API definitions and external interfaces
- **`icn-protocol`** – Message formats, protocol definitions, and serialization

### **Identity & Security (95% Complete)**
- **`icn-identity`** – DID management, credential verification, and cryptographic operations
- **`icn-dag`** – Content-addressed storage with multiple backend options (PostgreSQL, RocksDB, SQLite, Sled)
- **`icn-zk`** – Zero-knowledge circuits for privacy-preserving credential proofs

### **Governance & Economics (82% Complete)**  
- **`icn-governance`** – Proposal engine, voting mechanisms, and policy execution
- **`icn-economics`** – Mana accounting and economic policy enforcement  
- **`icn-reputation`** – Trust scoring and contribution tracking
- **`icn-eventstore`** – Event sourcing utilities for auditable systems

### **Networking & Computation (78% Complete)**
- **`icn-network`** – P2P networking with libp2p integration
- **`icn-mesh`** – Distributed job scheduling and execution

### **Language & Tools (95% Complete)**
- **`icn-ccl`** – Cooperative Contract Language compiler (standalone project)
- **`icn-cli`** – Command-line interface for all operations
- **`icn-node`** – Main daemon binary with HTTP server
- **`icn-sdk`** – High-level Rust SDK for HTTP API interactions
- **`icn-templates`** – Template management system
- **`job-audit`** – Job auditing functionality

### **Frontend Applications (70% Complete)**
- **`apps/web-ui`** – Federation administration dashboard (React + TypeScript)
- **`apps/explorer`** – DAG viewer and network activity browser  
- **`apps/agoranet`** – Governance deliberation and voting platform
- **`apps/wallet-ui`** – Secure DID and key management interface

### **Shared Frontend Infrastructure (80% Complete)**
- **`packages/ui-kit`** – Cross-platform component library (Tamagui)
- **`packages/ts-sdk`** – TypeScript SDK for all frontend applications
- **`packages/ccl-visual-editor`** – Visual contract editor (planned)

**Out of Scope for `icn-core`:**
- Deployment scripts and infrastructure configuration (belongs in `icn-infra`, `icn-devnet`)
- External project documentation (belongs in `icn-docs`)
- Production deployment configurations (belongs in `icn-infra`)

> **Current Reality:** ICN is a working platform with real P2P networking, cross-node job execution, and comprehensive governance/economic systems. This is **not a prototype** - it's production-ready infrastructure.

---

## 2 · Complete Workspace Layout

### **Backend Crates (`crates/`)**

| Crate | Purpose | Completion | Key Features |
|-------|---------|------------|--------------|
| **`icn-runtime`** | Node orchestration, Host-ABI, job management | 95% | WASM execution, RuntimeContext, job orchestration |
| **`icn-common`** | Shared types, cryptographic primitives | 100% | DIDs, CIDs, error handling, utilities |
| **`icn-api`** | HTTP API definitions and external interfaces | 90% | 60+ endpoints, TypeScript generation, comprehensive DTOs |
| **`icn-protocol`** | Message formats and protocol definitions | 100% | P2P messages, serialization, wire formats |
| **`icn-identity`** | DID management, credential verification | 95% | Ed25519 signatures, credential lifecycle, ZK proofs |
| **`icn-dag`** | Content-addressed storage, multiple backends | 85% | PostgreSQL, RocksDB, SQLite, Sled backends |
| **`icn-governance`** | Proposal engine, voting, policy execution | 82% | CCL compilation, proposal lifecycle, voting |
| **`icn-economics`** | Mana accounting, economic policy enforcement | 75% | Regenerating mana, resource management, token system |
| **`icn-reputation`** | Trust scoring, contribution tracking | 80% | Reputation algorithms, scoring, validation |
| **`icn-network`** | P2P networking with libp2p integration | 85% | Gossipsub, Kademlia DHT, peer discovery |
| **`icn-mesh`** | Distributed job scheduling and execution | 78% | Job bidding, executor selection, cross-node execution |
| **`icn-eventstore`** | Event sourcing utilities | 90% | JSON Lines format, memory/file stores, audit trails |
| **`icn-zk`** | Zero-knowledge circuits for privacy | 85% | Age verification, membership proofs, reputation thresholds |
| **`icn-sdk`** | High-level Rust SDK for HTTP APIs | 85% | Type-safe client, comprehensive coverage, async support |
| **`icn-templates`** | Template management system | 70% | CCL templates, governance patterns |
| **`icn-cli`** | Command-line interface | 90% | All operations, federation management, job submission |
| **`icn-node`** | Main daemon binary with HTTP server | 88% | Axum server, authentication, comprehensive endpoints |
| **`job-audit`** | Job auditing functionality | 75% | Job tracking, audit trails, compliance |

### **Frontend Applications (`apps/`)**

| App | Purpose | Technology | Completion | Key Features |
|-----|---------|------------|------------|--------------|
| **`web-ui`** | Federation administration dashboard | React + TypeScript + Vite | 70% | Demo mode, federation management, governance UI |
| **`explorer`** | DAG viewer and network activity browser | React + TypeScript + D3.js | 65% | DAG visualization, job tracking, network analytics |
| **`agoranet`** | Governance deliberation and voting | React Native + Tamagui | 60% | Proposal creation, voting interface, deliberation tools |
| **`wallet-ui`** | Secure DID and key management | React Native + Tamagui | 60% | DID creation, key storage, mana tracking |

### **Shared Packages (`packages/`)**

| Package | Purpose | Technology | Completion |
|---------|---------|------------|------------|
| **`ui-kit`** | Cross-platform component library | Tamagui | 70% |
| **`ts-sdk`** | TypeScript SDK for frontends | TypeScript | 80% |
| **`ccl-visual-editor`** | Visual contract editor | React + TypeScript | 30% |

---

## 3 · System Flow: Complete ICN Pipeline

### **1. Federation Bootstrap & P2P Networking**
```
Node Startup → libp2p Service → Peer Discovery → Federation Join
     ↓
Multi-node Network with Gossipsub + Kademlia DHT
     ↓
Cross-federation coordination and trust establishment
```

### **2. Mesh Job Execution Pipeline** 
```
Job Submission (CLI/API) → Mana Validation → Network Announcement
     ↓
Bid Collection → Executor Selection → Job Assignment  
     ↓
WASM Execution → Receipt Generation → DAG Anchoring → Reputation Update
```

### **3. Governance & Decision Making**
```
Proposal Creation (CCL) → Compilation to WASM → Network Broadcast
     ↓
Deliberation Phase → Voting Collection → Quorum Check
     ↓  
Execution (if passed) → Parameter Updates → DAG Anchoring
```

### **4. Frontend Integration**
```
Web UI → TypeScript SDK → HTTP API → Node Runtime
     ↓
Real-time updates via planned WebSocket connections
```

---

## 4 · Development Workflow & Commands

### **Backend Development (Rust)**
```bash
# Setup and basic operations
just setup && just build
just test           # Run all tests
just lint           # Check code quality
just validate       # Complete validation suite

# Multi-node testing
just devnet         # 3-node containerized federation
just health-check   # Federation health validation
```

### **Frontend Development**
```bash
# Setup frontend environment  
just setup-frontend
just install-frontend

# Development servers
just dev-frontend   # All apps
just dev-web-ui     # Admin dashboard
just dev-explorer   # Network explorer
just dev-agoranet   # Governance interface
just dev-wallet     # DID/key management

# Cross-platform development
just dev-mobile     # React Native (iOS/Android)
just dev-desktop    # Tauri desktop apps
```

### **Complete Stack Development**
```bash
# Full environment setup
just setup-all

# Complete validation
just validate-all-stack

# Build everything
just build-all-stack
```

---

## 5 · API Architecture & Capabilities

### **HTTP API Status: 60+ Endpoints (vs 37 documented)**

**Production-Ready Endpoints:**
- **Governance:** `/governance/*` (7 endpoints) - Proposals, voting, delegation
- **Identity:** `/identity/*` (8 endpoints) - Credentials, ZK proofs, verification  
- **Federation:** `/federation/*` (6 endpoints) - Peer management, trust, status
- **Mesh Computing:** `/mesh/*` (12 endpoints) - Job lifecycle, progress, streaming
- **Cooperative:** `/cooperative/*` (7 endpoints) - Registry, trust, capabilities
- **DAG Storage:** `/dag/*` (8 endpoints) - Content addressing, pinning, pruning
- **Account/Mana:** `/account/*` (3 endpoints) - Balance, transactions
- **System:** `/info`, `/status`, `/health`, `/metrics` (4 endpoints)
- **Network:** `/network/*` (3 endpoints) - Peer management, connections
- **Circuits:** `/circuits/*` (3 endpoints) - ZK circuit registration

### **TypeScript SDK Generation**
- Automatic TypeScript type generation from Rust API traits
- Comprehensive client SDK with full type safety
- WebSocket support planned for real-time updates
- Error handling and authentication built-in

---

## 6 · Key Implementation Insights

### **Production Services vs Stubs**
```rust
// Current: Production services available but need configuration
#[cfg(feature = "enable-libp2p")]
let mesh_network_service = Arc::new(DefaultMeshNetworkService::new(libp2p_service));

let signer = Arc::new(Ed25519Signer::new(private_key));
let dag_store = select_dag_store(&config.storage); // PostgreSQL/RocksDB/SQLite/Sled
```

### **Multi-Backend Storage Support**
- **PostgreSQL** - Production database backend
- **RocksDB** - High-performance embedded database  
- **SQLite** - Single-file deployment
- **Sled** - Pure Rust embedded database
- **Memory** - Development/testing only

### **Comprehensive Feature Coverage**
- **Zero-Knowledge Proofs:** Age verification, reputation thresholds, membership proofs
- **Event Sourcing:** Complete audit trails with JSON Lines format
- **Economic Systems:** Mana regeneration, token management, resource policies
- **Federated Identity:** DID lifecycle, credential management, trust attestation

---

## 7 · Coding & Review Guidelines

### **Architecture Patterns (Strictly Enforced)**
- **Trait-based interfaces:** All services implement traits defined in `icn-api`
- **Deterministic execution:** Core logic must be predictable across nodes
- **Async-first:** All I/O operations use `async/await`
- **Multi-backend support:** Storage and networking allow multiple implementations

### **Quality Standards**
- **Comprehensive testing:** Unit, integration, and E2E tests required
- **Security-first:** All economic/governance logic requires adversarial testing
- **Performance monitoring:** Prometheus metrics for all critical paths
- **Cross-platform support:** Frontend apps target Web/iOS/Android/Desktop

### **Documentation Requirements**
- **Public APIs:** Comprehensive rustdoc with examples
- **Architecture decisions:** Update .md files in `docs/`
- **API changes:** Update TypeScript SDK generation
- **Frontend changes:** Update component documentation

---

## 8 · Current Phase: Operational Excellence

### **Immediate Priorities (Phase 5)**
- [ ] Complete service configuration management
- [ ] Scale testing to 10+ node federations  
- [ ] Production monitoring and alerting
- [ ] Complete frontend application development

### **Next Phase (Phase 6): Advanced Features**
- [ ] ZK proof system expansion
- [ ] Advanced governance patterns
- [ ] Cooperative banking features
- [ ] Cross-federation protocols

---

**Remember: ICN is production-ready infrastructure for building the cooperative digital economy. Approach development with production quality and real-world use cases in mind.** 