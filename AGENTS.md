# AGENTS.md

## ICN Core – AI Agent & Contributor Guide

**Welcome, AI agent or human contributor! This file is your comprehensive guide to working on ICN Core, the production-ready infrastructure for a cooperative digital economy.**

---

## 🌐 **Current Reality: What We Have Built**

The InterCooperative Network (ICN) Core is **production-ready infrastructure** (77-82% complete) that provides **working federated cooperative coordination** without relying on corporate cloud platforms or speculative blockchain systems.

### **Current Status: Phase 5 (Operational Excellence)**
This is **not a prototype**—it's working infrastructure with:
- ✅ **Real P2P networking** with verified multi-node federation
- ✅ **Cross-node job execution** with cryptographic verification
- ✅ **Democratic governance** with CCL compilation and voting
- ✅ **Economic systems** with mana regeneration and reputation
- ✅ **Complete API ecosystem** with 60+ endpoints and TypeScript SDK
- ✅ **Frontend applications** across Web/Mobile/Desktop platforms
- ✅ **Zero-knowledge privacy** with credential proofs and selective disclosure

### **What Communities Can Do Today**
1. **Deploy federations** with real P2P networking and automatic peer discovery
2. **Submit mesh jobs** that execute across multiple nodes with bidding and selection
3. **Create proposals** using CCL that compile to WASM for democratic governance
4. **Manage resources** using regenerating mana with anti-extraction properties
5. **Issue credentials** with zero-knowledge proofs for privacy-preserving verification
6. **Use comprehensive APIs** through TypeScript SDK or direct HTTP endpoints
7. **Run complete applications** for federation management, governance, and identity

---

## 🏗️ **Complete Repository Architecture**

ICN Core is a **comprehensive monorepo** containing both deterministic Rust libraries and complete frontend applications across all platforms.

### **Backend Infrastructure (Rust)**

#### **Core Infrastructure (100% Complete)**
- **`icn-runtime`** – Node orchestration, WASM execution, job management
- **`icn-common`** – Shared types, cryptographic primitives, utilities
- **`icn-api`** – HTTP API definitions (60+ endpoints), TypeScript generation
- **`icn-protocol`** – P2P message formats and protocol definitions

#### **Identity & Security (95% Complete)**
- **`icn-identity`** – Complete DID lifecycle, credential verification, Ed25519 signatures
- **`icn-dag`** – Content-addressed storage (PostgreSQL, RocksDB, SQLite, Sled backends)
- **`icn-zk`** – Zero-knowledge circuits (age verification, membership proofs, reputation)

#### **Governance & Economics (82% Complete)**
- **`icn-governance`** – Proposal engine, voting mechanisms, CCL compilation
- **`icn-economics`** – Mana accounting, regeneration, economic policy enforcement
- **`icn-reputation`** – Trust scoring, contribution tracking, reputation algorithms
- **`icn-eventstore`** – Event sourcing utilities with JSON Lines format

#### **Networking & Computation (78% Complete)**
- **`icn-network`** – P2P networking with libp2p (Gossipsub, Kademlia DHT)
- **`icn-mesh`** – Distributed job scheduling, bidding, cross-node execution

#### **Developer Tools & SDKs (90% Complete)**
- **`icn-cli`** – Command-line interface for all operations
- **`icn-node`** – Main daemon binary with Axum HTTP server
- **`icn-sdk`** – High-level Rust SDK for HTTP API interactions
- **`icn-templates`** – Governance template management
- **`job-audit`** – Job auditing and compliance functionality

### **Frontend Applications**

#### **Cross-Platform Apps (React Native + Tamagui)**
- **`apps/wallet-ui`** (60% complete) – Secure DID and key management (iOS/Android/Web/Desktop)
- **`apps/agoranet`** (60% complete) – Governance deliberation platform (iOS/Android/Web/Desktop)

#### **Web Applications (React + TypeScript)**
- **`apps/web-ui`** (70% complete) – Federation administration dashboard with demo mode
- **`apps/explorer`** (65% complete) – DAG viewer and network browser with D3.js visualization

#### **Shared Frontend Infrastructure**
- **`packages/ui-kit`** (70% complete) – Cross-platform component library (Tamagui)
- **`packages/ts-sdk`** (80% complete) – TypeScript SDK with comprehensive API coverage
- **`packages/ccl-visual-editor`** (30% complete) – Visual contract editor (planned)

---

## 🎯 **Agent Authority & Current Focus**

### **Current Phase: Operational Excellence (Phase 5)**

**Key Insight**: The remaining 18-23% is primarily **configuration management** and **operational polish**, not missing core functionality. Production services exist and work—they need proper default configuration.

### **Immediate Priorities**
1. **Service Configuration**: Ensure production services are used by default
2. **Scale Testing**: Validate with 10+ node federations
3. **Frontend Completion**: Complete the 4 frontend applications
4. **Production Monitoring**: Add comprehensive observability
5. **Mobile Deployment**: Cross-platform app store deployment

### **You Are Empowered To:**
- **Complete configuration management** for production service defaults
- **Enhance frontend applications** with missing features and polish
- **Improve API endpoints** and TypeScript SDK coverage
- **Add production monitoring** and operational excellence tools
- **Optimize performance** for multi-node federation scenarios
- **Enhance security** and privacy features
- **Improve developer experience** and documentation

---

## 🚀 **Working Production Systems**

### **1. Multi-Node P2P Federation**
```
Real libp2p Networking ✅
├── Gossipsub messaging
├── Kademlia DHT peer discovery
├── Automatic federation bootstrap
└── Cross-federation coordination

Current: 3+ node networks verified
Goal: Scale to 10+ node federations
```

### **2. Cross-Node Mesh Computing**
```
Complete Job Pipeline ✅
├── Job submission (CLI/API/Web UI)
├── Network-wide bid collection
├── Reputation-based executor selection
├── WASM execution with security constraints
├── Cryptographic receipt generation
└── DAG anchoring and reputation updates

Current: Real cross-node execution working
Goal: Enhanced performance and monitoring
```

### **3. Democratic Governance System**
```
CCL-Powered Governance ✅
├── Proposal creation with CCL compilation
├── WASM policy execution
├── Voting with quorum enforcement
├── Delegation and revocation
├── Policy implementation
└── Complete audit trails

Current: 95% complete CCL system
Goal: Enhanced governance templates
```

### **4. Economic Resource Management**
```
Mana-Based Economics ✅
├── Time-based mana regeneration
├── Reputation-influenced rates
├── Resource accounting and enforcement
├── Multi-backend persistence
├── Token management system
└── Anti-extraction mechanisms

Current: Working across multiple backends
Goal: Enhanced economic policies
```

### **5. Comprehensive API Ecosystem**
```
Production HTTP API ✅
├── 60+ endpoints across all domains
├── TypeScript SDK with type safety
├── Authentication and authorization
├── Comprehensive error handling
├── Prometheus metrics integration
└── Real-time WebSocket support (planned)

Current: Most endpoints implemented
Goal: Complete TypeScript SDK coverage
```

### **6. Zero-Knowledge Privacy System**
```
ZK Credential Proofs ✅
├── Age verification circuits
├── Membership proof generation
├── Reputation threshold proofs
├── Selective credential disclosure
├── Batch verification
└── Privacy-preserving operations

Current: Core circuits implemented
Goal: Expanded proof system
```

---

## 💻 **Development Environment & Workflow**

### **Complete Stack Development**
```bash
# Full environment setup
just setup-all              # Backend + Frontend environment
just validate-all-stack     # Complete validation
just build-all-stack        # Build everything

# Multi-node testing
just devnet                 # 3-node containerized federation
just health-check           # Federation health validation
```

### **Backend Development (Rust)**
```bash
# Core development cycle
just setup && just build    # Setup and build all crates
just test                   # Comprehensive test suite
just lint                   # Code quality checks
just validate               # Full validation
```

### **Frontend Development**
```bash
# Frontend development
just setup-frontend         # Node.js, pnpm, dependencies
just dev-frontend          # All apps simultaneously
just dev-web-ui            # Admin dashboard
just dev-explorer          # Network explorer
just dev-wallet            # DID/key management
just dev-agoranet          # Governance interface

# Cross-platform
just dev-mobile            # React Native (iOS/Android)
just dev-desktop           # Tauri desktop apps
```

---

## 🔧 **Current Configuration Challenge**

### **Production Services Available**
| Component | Stub Service | Production Service | Status |
|-----------|--------------|-------------------|---------|
| **Mesh Networking** | `StubMeshNetworkService` | `DefaultMeshNetworkService` | ✅ Ready |
| **Cryptographic Signing** | `StubSigner` | `Ed25519Signer` | ✅ Ready |
| **DAG Storage** | `StubDagStore` | PostgreSQL/RocksDB/SQLite/Sled | ✅ Ready |
| **P2P Networking** | N/A | `LibP2pNetworkService` | ✅ In Use |
| **Governance** | N/A | `GovernanceModule` | ✅ In Use |

### **Key Challenge**: Service Selection
Production services exist but some contexts default to stub services. The solution is configuration management, not implementing missing features.

```rust
// Current: Production services available but need configuration
#[cfg(feature = "enable-libp2p")]
let mesh_network_service = Arc::new(DefaultMeshNetworkService::new(libp2p_service));

let signer = Arc::new(Ed25519Signer::new(private_key));
let dag_store = select_dag_store(&config.storage); // Multiple backends available
```

---

## 📱 **Frontend Application Status**

### **Web UI (70% Complete)**
- ✅ **Demo mode** with comprehensive feature showcase
- ✅ **Federation management** with peer coordination
- ✅ **Governance interface** with proposal and voting
- 🚧 **Advanced monitoring** and analytics
- 🚧 **Production deployment** configuration

### **Explorer (65% Complete)**
- ✅ **DAG visualization** with D3.js
- ✅ **Job tracking** and progress monitoring
- ✅ **Network analytics** and peer status
- 🚧 **Real-time updates** via WebSocket
- 🚧 **Advanced query** capabilities

### **Wallet UI (60% Complete)**
- ✅ **DID creation** and management
- ✅ **Private key storage** with security
- ✅ **Mana tracking** and transactions
- 🚧 **Credential management** interface
- 🚧 **Cross-platform deployment**

### **AgoraNet (60% Complete)**
- ✅ **Proposal creation** with CCL editing
- ✅ **Voting interface** with delegation
- ✅ **Community deliberation** tools
- 🚧 **Advanced governance** patterns
- 🚧 **Mobile optimization**

---

## 🛡️ **Security & Production Guidelines**

### **Production-Ready Security**
- ✅ **Ed25519 signatures** for all cryptographic operations
- ✅ **Multi-backend storage** with encryption at rest
- ✅ **API authentication** with keys and bearer tokens
- ✅ **Rate limiting** and abuse prevention
- ✅ **ZK proofs** for privacy-preserving operations
- ✅ **Comprehensive audit trails** with event sourcing

### **Critical Security Invariants**
- All economic transactions must be mana-enforced
- All governance actions must be cryptographically verified
- All network messages must be signed and authenticated
- All DAG operations must maintain content-addressing integrity
- All ZK proofs must be properly verified before acceptance

---

## 🎯 **Agent Task Categories**

### **1. Configuration Management (High Priority)**
```rust
// Example: Ensure production services by default
fn create_default_runtime_context() -> RuntimeContext {
    RuntimeContext {
        mesh_network_service: Arc::new(DefaultMeshNetworkService::new()),
        signer: Arc::new(Ed25519Signer::new()),
        dag_store: Arc::new(PostgresDagStore::new()), // Not stub!
        // ... other production services
    }
}
```

### **2. Frontend Application Enhancement**
- Complete missing features in Web UI dashboard
- Add real-time updates to Explorer
- Enhance mobile experience for Wallet and AgoraNet
- Improve TypeScript SDK coverage and error handling

### **3. API & Integration Improvements**
- Complete implementation of remaining endpoints
- Enhance TypeScript SDK with full type coverage
- Add WebSocket support for real-time events
- Improve API documentation and examples

### **4. Scale Testing & Performance**
- Test 10+ node federation scenarios
- Optimize mesh job execution performance
- Enhance P2P networking efficiency
- Add comprehensive performance monitoring

### **5. Production Operations**
- Add Prometheus metrics and Grafana dashboards
- Implement health checks and alerting
- Create deployment automation
- Enhance security monitoring and logging

---

## 📚 **Essential Reading for Agents**

### **Start Here (Updated Documentation)**
1. **[README.md](README.md)** – Complete project overview (77-82% complete status)
2. **[CONTEXT.md](CONTEXT.md)** – Full project context and philosophical foundation
3. **[ICN_API_REFERENCE.md](ICN_API_REFERENCE.md)** – All 60+ HTTP endpoints
4. **[docs/status/STATUS.md](docs/status/STATUS.md)** – Current implementation status

### **Architecture & Development**
5. **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** – System design and components
6. **[docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md)** – Complete development workflow
7. **[.cursor/rules/](/.cursor/rules/)** – Comprehensive development rules
8. **[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)** – Complete navigation guide

### **Frontend Development**
9. **[apps/web-ui/README.md](apps/web-ui/README.md)** – Federation dashboard
10. **[packages/ts-sdk/README.md](packages/ts-sdk/README.md)** – TypeScript SDK
11. **[packages/ui-kit/README.md](packages/ui-kit/README.md)** – Component library

---

## 🌟 **Vision Alignment**

You're working on **production-ready infrastructure** that's already changing how communities coordinate. ICN Core isn't a future vision—it's working technology that enables:

- **Democratic governance** without centralized control
- **Economic coordination** without extraction
- **Resource sharing** across federation boundaries
- **Privacy preservation** through zero-knowledge proofs
- **Sovereign infrastructure** owned by communities

### **Current Impact**
- Communities can deploy **real federations** today
- Cooperatives can encode **bylaws as executable policy**
- Members can participate in **cryptographically verified governance**
- Resources can be shared using **regenerating mana economics**
- Privacy can be preserved through **zero-knowledge credentials**

### **Your Contribution**
Every improvement you make to ICN Core directly enhances the infrastructure that cooperatives and communities use to coordinate democratically. You're not building software—you're building the foundation of a **cooperative digital economy**.

---

**Thank you for contributing to production-ready cooperative infrastructure. Together, we're creating the tools that communities use today to govern themselves and coordinate resources without extraction or centralized control.**
