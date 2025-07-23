# ICN Core v0.2 – Cooperative Infrastructure in Active Development

> **⚠️ DEVELOPMENT STATUS: This project is actively under development and NOT production ready**

ICN Core is the **experimental** reference implementation of the InterCooperative Network (ICN) protocol, written in Rust with comprehensive frontend applications. It provides **early-stage** infrastructure for federations, cooperatives, and communities to coordinate democratically without relying on traditional centralized systems.

**Mission**: Replace every major function of the state and corporation with programmable, federated, democratic infrastructure—built for actual human needs and scalable solidarity.

**Current Status**: This is **heavy development software** with many core features stubbed or incomplete. While it demonstrates working P2P networking, cross-node job execution, and governance/economic systems, it should **NOT be used in production environments**.

## 🚧 Development Status Warning

**IMPORTANT**: ICN Core is under active development with many stub implementations and incomplete features. Key limitations include:

- **Stub services**: Many core services use placeholder implementations that need real functionality
- **Security**: Cryptographic implementations may not be production-hardened
- **Data persistence**: Storage backends may have bugs or incomplete features
- **Network stability**: P2P networking may be unreliable in some configurations
- **API stability**: APIs and data formats are subject to breaking changes

**Use only for:**
- Development and testing
- Research and experimentation
- Contributing to the project

**DO NOT use for:**
- Production applications
- Real economic transactions
- Critical governance decisions
- Any system where data loss or security issues would cause harm

---

## 🚀 Quick Start

### Try ICN in 10 Minutes (Development Only)
```bash
# Clone and build
git clone https://github.com/InterCooperative/icn-core
cd icn-core
just setup && just build

# Start a development node
cargo run --bin icn-node

# In another terminal, try the CLI
cargo run --bin icn-cli -- info
```

### Start Frontend Applications
```bash
# Setup frontend development environment
just setup-frontend

# Start all frontend apps
just dev-frontend

# Or start specific apps
just dev-web-ui     # Federation dashboard
just dev-explorer   # Network explorer  
just dev-agoranet   # Governance interface
just dev-wallet     # DID/key management
```

**Next Steps**: [Complete Getting Started Guide](docs/beginner/README.md)

### Documentation Hub 📚
📖 **[Documentation Index](DOCUMENTATION_INDEX.md)** - Quick navigation guide  
📚 **[Complete Documentation](docs/README.md)** - Full documentation directory  
📊 **[Project Status & Roadmap](PROJECT_STATUS_AND_ROADMAP.md)** - Consolidated status and development roadmap  
🏗️ **[Architecture Guide](docs/ARCHITECTURE.md)** - System design and component overview  
🔗 **[Complete API Reference](ICN_API_REFERENCE.md)** - All 60+ HTTP endpoints  

---

## 🎯 What ICN Provides (Current State)

### **Experimental Platform** (Under Heavy Development)
⚠️ **Multi-node P2P networking** with basic connectivity (advanced features need work)  
⚠️ **Job execution framework** with basic structure (scheduling algorithms stubbed)  
⚠️ **Governance foundations** with UI and data models (voting mechanisms incomplete)  
⚠️ **Economic concepts** with mana framework (transaction logic needs implementation)  
⚠️ **Identity architecture** with DID structures (key management needs security review)  
⚠️ **HTTP API scaffolding** with 60+ endpoint structures (many return mock data)  
⚠️ **Frontend applications** built and running (connected to stub backend services)  
⚠️ **Security frameworks** designed (cryptographic implementations may be incomplete)  

### **Development Capabilities (What's Demonstrable)**
- **Development Environment**: Multi-node devnet setup with containerized networking
- **API Framework**: HTTP server that handles requests and basic routing  
- **Database Integration**: Connection to multiple storage backends (data models may be incomplete)
- **Frontend Development**: Working UI applications with good UX (backend integration varies)
- **CLI Tooling**: Command-line interface for basic operations (some commands stubbed)
- **Configuration**: Service configuration system for development workflows

### **Architectural Frameworks**
ICN includes well-designed architectural foundations for:
- **Privacy Systems**: ZK proof frameworks designed (implementation completeness varies)
- **Economic Models**: Mana-based resource concepts (needs transaction logic)
- **Governance Models**: Proposal/voting structures (decision execution incomplete)
- **Reputation Thresholds**: Prove reputation levels without disclosing exact scores
- **Credential Disclosure**: Selective revelation of credential attributes
- **Batch Verification**: Efficient verification of multiple proofs

---

## 📦 Complete Architecture Overview

ICN Core is organized as a comprehensive platform with backend infrastructure, frontend applications, and developer tools:

### **Backend Infrastructure (Rust)**

#### **Core Infrastructure (100% Complete)**
- **`icn-runtime`** – Node orchestration, WASM execution, and job management
- **`icn-common`** – Shared types, cryptographic primitives, and utilities
- **`icn-api`** – HTTP API definitions with 60+ endpoints and TypeScript generation
- **`icn-protocol`** – P2P message formats and protocol definitions

#### **Identity & Security (95% Complete)**
- **`icn-identity`** – DID management, credential verification, Ed25519 signatures
- **`icn-dag`** – Content-addressed storage (PostgreSQL, RocksDB, SQLite, Sled backends)
- **`icn-zk`** – Zero-knowledge circuits for privacy-preserving credential proofs

#### **Governance & Economics (Early Development)**
- **`icn-governance`** – Proposal structures, basic voting UI (voting algorithms need work)
- **`icn-economics`** – Mana concepts, basic accounting framework (transaction logic incomplete)
- **`icn-reputation`** – Trust scoring foundations (algorithms may be stubbed)
- **`icn-eventstore`** – Event sourcing scaffolding (persistence may be incomplete)

#### **Networking & Computation (Basic Functionality)**
- **`icn-network`** – P2P networking basics with libp2p (advanced features need work)
- **`icn-mesh`** – Job scheduling framework (bidding algorithms and execution largely stubbed)

#### **Developer Tools & SDKs (Scaffolding Complete)**
- **`icn-cli`** – Command-line interface structure (some commands return mock data)
- **`icn-node`** – Main daemon binary with HTTP server (backend services may be stubbed)
- **`icn-sdk`** – HTTP client SDK (connects to potentially stubbed endpoints)
- **`icn-templates`** – Template management system (functionality may be incomplete)
- **`job-audit`** – Job auditing framework (compliance logic needs implementation)

### **Frontend Applications**

#### **Cross-Platform Applications (React Native + Tamagui)**

**`apps/wallet-ui`** 📱 – **Secure DID and Key Management**  
- **Platforms**: iOS, Android, Web, Desktop (via Tauri)
- **Features**: DID creation, private key storage, mana tracking, credential management
- **Technology**: React Native + Tamagui + Expo + Tauri
- **Status**: 60% complete

**`apps/agoranet`** 🗳️ – **Governance Deliberation Platform**  
- **Platforms**: iOS, Android, Web, Desktop (via Tauri)
- **Features**: Proposal creation, community deliberation, voting interface, CCL editing
- **Technology**: React Native + Tamagui + Expo + Tauri
- **Status**: 60% complete

#### **Web Applications (React + TypeScript)**

**`apps/web-ui`** 🌐 – **Federation Administration Dashboard**  
- **Platform**: Web browser (PWA-enabled)
- **Features**: Federation management, member administration, system monitoring
- **Technology**: React + Vite + TypeScript + Tailwind CSS
- **Status**: 70% complete

**`apps/explorer`** 🔍 – **DAG Viewer and Network Activity Browser**  
- **Platform**: Web browser (PWA-enabled)
- **Features**: DAG visualization, job tracking, network analytics, receipt browsing
- **Technology**: React + Vite + D3.js + Tailwind CSS
- **Status**: 65% complete

#### **Shared Frontend Infrastructure**

**`packages/ui-kit`** 🎨 – **Cross-Platform Component Library**  
- **Technology**: Tamagui (universal design system)
- **Status**: 70% complete

**`packages/ts-sdk`** 🛠️ – **TypeScript SDK**  
- **Features**: Type-safe API client, comprehensive endpoint coverage
- **Status**: 80% complete

**`packages/ccl-visual-editor`** ✨ – **Visual Contract Editor**  
- **Features**: Drag-and-drop governance contract creation
- **Status**: 30% complete (planned)

### **Platform Support Matrix**

| Application | Web | iOS | Android | Desktop | Technology |
|-------------|-----|-----|---------|---------|------------|
| **Wallet UI** | ✅ | ✅ | ✅ | ✅ | React Native + Tauri |
| **AgoraNet** | ✅ | ✅ | ✅ | ✅ | React Native + Tauri |
| **Web UI** | ✅ | 📱 | 📱 | 🔄 | React + Vite |
| **Explorer** | ✅ | 📱 | 📱 | 🔄 | React + D3.js |

*Legend: ✅ Native app • 📱 Responsive web • 🔄 Future support*

---

## 🛠️ Complete Development Environment

### **Prerequisites**

#### **Backend Development (Rust)**
- **Rust** (stable toolchain via `rust-toolchain.toml`)
- **Just** command runner
- **Git** with pre-commit hooks

#### **Frontend Development**
- **Node.js** 18+ ([Download](https://nodejs.org/))
- **pnpm** 8+ (package manager)
- **iOS Development** (macOS only): Xcode and iOS Simulator
- **Android Development**: Android Studio and Android SDK
- **Desktop Development**: Rust toolchain (for Tauri)

### **Development Workflow**

#### **Complete Stack Development**
```bash
# Setup entire development environment (Rust + Frontend)
just setup-all

# Full stack validation
just validate-all-stack

# Build everything (backend + frontend)
just build-all-stack
```

#### **Backend Development (Rust)**
```bash
# Setup Rust development environment
just setup

# Core development commands
just test          # Run all tests
just lint          # Check code quality  
just build         # Build all crates

# Multi-node federation testing
just devnet        # Start 3-node test federation
just health-check  # Federation health validation
```

#### **Frontend Development**
```bash
# Setup frontend development environment
just setup-frontend

# Start all frontend apps
just dev-frontend

# Start specific apps
just dev-wallet    # Cross-platform wallet
just dev-agoranet  # Governance interface
just dev-web-ui    # Admin dashboard
just dev-explorer  # Network explorer

# Platform-specific development
just dev-mobile    # Mobile apps (iOS/Android)
just dev-desktop   # Desktop apps (Tauri)

# Frontend validation
just lint-frontend
just test-frontend
just build-frontend
```

---

## 🚧 Development API & Integration

### **HTTP API Framework (60+ Endpoint Structures)**

ICN provides a well-structured REST API framework covering intended functionality:

- **Governance** (8 endpoints): Proposal structures, voting UI (backend logic incomplete)
- **Identity** (11 endpoints): Credential frameworks, DID structures (security needs review)
- **Mesh Computing** (12 endpoints): Job interfaces, status tracking (execution may be stubbed)
- **Federation** (8 endpoints): Peer structures, trust frameworks (algorithms need work)
- **Cooperative** (7 endpoints): Registry concepts, discovery frameworks (features incomplete)
- **Storage** (8 endpoints): DAG interfaces, content addressing (persistence may be limited)
- **Economics** (6 endpoints): Mana, transactions, reputation
- **System** (5+ endpoints): Health, metrics, monitoring

**Complete API Documentation**: [ICN_API_REFERENCE.md](ICN_API_REFERENCE.md)

### **TypeScript SDK**

Automatically generated TypeScript client with full type safety:

```typescript
import { ICNClient } from '@icn/client-sdk';

const client = new ICNClient({
  baseUrl: 'http://localhost:7845',
  apiKey: 'your-api-key'
});

// Type-safe API calls
const proposals = await client.governance.listProposals();
const jobResult = await client.mesh.submitJob({
  job_spec: {
    image: 'python:3.9',
    command: ['python', '-c', 'print("Hello, ICN!")'],
    resources: { cpu_cores: 1, memory_mb: 512, storage_mb: 1024 }
  },
  submitter_did: 'did:key:submitter',
  max_cost: 1000
});
```

### **WebSocket Support**

Real-time event subscriptions (planned):
- Proposal status changes
- Job progress updates  
- Federation peer events
- Mana balance changes
- Network events

---

## 📊 Current Implementation Status

### **Overall Progress: Heavy Development - Many Stubs Remain**

| Domain | Working | Partial | Stub/TODO | Progress |
|--------|---------|---------|-----------|----------|
| **Foundation** | 3/9 | 4/9 | 2/9 | **~40%** |
| **Mesh Computing** | 2/9 | 3/9 | 4/9 | **~30%** |
| **Governance** | 2/11 | 4/11 | 5/11 | **~25%** |
| **Economics** | 1/12 | 3/12 | 8/12 | **~20%** |
| **Security** | 2/9 | 3/9 | 4/9 | **~30%** |
| **Networking** | 3/8 | 2/8 | 3/8 | **~40%** |
| **Storage** | 2/7 | 2/7 | 3/7 | **~35%** |
| **Frontend** | 4/4 | 0/4 | 0/4 | **UI: 100%** |

### **Development Features ⚠️**

- **P2P networking basics** with libp2p integration (advanced features need work)
- **Job execution framework** with basic structure (scheduling algorithms stubbed)
- **Governance foundations** with UI and data models (voting mechanisms incomplete)
- **Economic concepts** with mana framework (transaction logic needs implementation)
- **Identity architecture** with DID structures (security implementations need review)
- **API scaffolding** with comprehensive endpoint structures (many return mock data)
- **Frontend applications** fully built and functional (connected to potentially stubbed backends)
- **Development environment** with good tooling and testing setup

---

## 🔨 Current Phase: Core Implementation

**Phase**: 3 (Core Development) - **Heavy Development**  
**Focus**: Replace stub implementations, complete TODO items, implement real functionality

### **Key Reality: Implementation, Not Just Configuration**
The remaining work involves significant **stub replacement** and **feature implementation**, not just configuration. While the architecture is well-designed, many services need real implementations.

### **Current Development Status**
- **Basic federation networking** with limited functionality (needs algorithm work)
- **Job execution framework** with stub scheduling (bidding and execution need implementation)
- **Governance UI and structures** with incomplete voting (decision logic needs work)
- **Economic framework** with basic concepts (transaction processing incomplete)
- **Excellent development tooling** and documentation (this part is genuinely good)

### **Immediate Development Priorities**
- [ ] Replace stub implementations with real algorithms
- [ ] Complete voting and governance decision mechanisms  
- [ ] Implement proper economic transaction processing
- [ ] Security review and hardening of cryptographic operations
- [ ] Add comprehensive testing for new implementations

---

## 🔮 Strategic Roadmap

### **Completed Foundation Work**
- **Phase 1-2**: Architecture design, crate structure, development environment ✅
- **Phase 3** (Current): Core implementation and stub replacement

### **Upcoming Development Phases**
- **Phase 6** (Q2 2025): Advanced governance patterns, ZK proof expansion
- **Phase 7** (Q3 2025): Cross-federation protocols, cooperative banking
- **Phase 8** (Q4 2025-Q2 2026): Application ecosystem, mainstream adoption

### **Cooperative Infrastructure Vision**
- **Cooperative Banking**: Mutual credit, time banking, democratic loans
- **Mutual Aid Networks**: Emergency response, resource sharing, skill matching
- **Supply Chain Cooperation**: Sourcing, quality assurance, fair trade
- **Worker Cooperative Tools**: Profit sharing, democratic coordination
- **Climate Action**: Carbon credits, renewable energy sharing, sustainability metrics

**Complete Roadmap**: [PROJECT_STATUS_AND_ROADMAP.md](PROJECT_STATUS_AND_ROADMAP.md)

---

## 🌍 Community & Support

### Getting Help
- **Documentation**: [docs/README.md](docs/README.md) - Comprehensive guides
- **API Reference**: [ICN_API_REFERENCE.md](ICN_API_REFERENCE.md) - All HTTP endpoints
- **Troubleshooting**: [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) - Common issues
- **Discussions**: [GitHub Discussions](https://github.com/InterCooperative/icn-core/discussions)

### Contributing
We welcome contributions across multiple areas:
- **Backend Development**: Rust implementation, tests, and optimization
- **Frontend Development**: React/React Native applications and components
- **Governance**: CCL policies and cooperative bylaws
- **Research**: Economic models and governance patterns
- **Community**: Education, organizing, and outreach

**Contributing Guide**: [CONTRIBUTING.md](CONTRIBUTING.md)

### Resources
- **Website**: [intercooperative.network](https://intercooperative.network)
- **Repository**: [GitHub](https://github.com/InterCooperative/icn-core)
- **Status Dashboard**: [Project Status & Roadmap](PROJECT_STATUS_AND_ROADMAP.md)
- **License**: [Apache 2.0](LICENSE)

---

## 🔧 Development Recognition

**ICN Core represents a significant architectural achievement for cooperative digital infrastructure.** 

Key architectural accomplishments:
- **Well-designed P2P networking** with solid libp2p integration foundation
- **Thoughtful job execution framework** with good architectural patterns (needs algorithm implementation)
- **Comprehensive governance structure** with extensible policy framework (voting logic needs completion)
- **Complete API design** with 60+ well-structured endpoints (backend implementations vary)
- **Excellent frontend applications** for Web, iOS, Android, and Desktop (great UX, some stub backends)
- **Privacy-ready architecture** with ZK framework designed (implementation completeness varies)
- **Outstanding developer experience** with excellent tooling and documentation

This is **experimental infrastructure with great potential** - the architecture is solid, but significant implementation work remains.

---

**ICN Core: Experimental cooperative infrastructure under active development. [Help us build it](CONTRIBUTING.md).**
