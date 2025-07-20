# ICN Core v0.2 – Cooperative Infrastructure Engine

> **Building the infrastructure for a cooperative digital economy**

ICN Core is the reference implementation of the InterCooperative Network (ICN) protocol, written in Rust. It provides production-ready infrastructure for federations, cooperatives, and communities to coordinate democratically without relying on traditional centralized systems.

**Mission**: Replace every major function of the state and corporation with programmable, federated, democratic infrastructure—built for actual human needs and scalable solidarity.

---

## 🚀 Quick Start

### Try ICN in 10 Minutes
```bash
# Clone and build
git clone https://github.com/InterCooperative/icn-core
cd icn-core
just setup && just build

# Start a node
cargo run --bin icn-node

# In another terminal, try the CLI
cargo run --bin icn-cli -- info
```

**Next Steps**: [Complete Getting Started Guide](docs/beginner/README.md)

### Documentation Hub 📚
📖 **[Documentation Index](DOCUMENTATION_INDEX.md)** - Quick navigation guide  
📚 **[Complete Documentation](docs/README.md)** - Full documentation directory  
📊 **[Current Status](docs/status/STATUS.md)** - Implementation progress and capabilities  
🏗️ **[Architecture Guide](docs/ARCHITECTURE.md)** - System design and component overview  
🎨 **[Visual Editor Plan](docs/ccl/ccl_visual_editor_plan.md)** - CCL visual contract editor  

---

## 🎯 What ICN Provides

### **Production-Ready Platform** (77% Complete)
✅ **Multi-node P2P federation** with automatic peer discovery  
✅ **Cross-node job execution** with cryptographic verification  
✅ **Democratic governance** with programmable policies  
✅ **Economic coordination** with mana-based resource management  
✅ **Federated identity** with DID-based authentication  
✅ **HTTP API** with comprehensive REST endpoints  

### **Key Capabilities**
- **Shared Computing**: Mesh job execution across federation members
- **Democratic Governance**: Proposal creation, voting, and policy enforcement via CCL
- **Economic Enforcement**: Mana-based resource allocation preventing abuse
- **Federated Identity**: DID-based membership with cryptographic verification
- **Persistent History**: Content-addressed DAG storage for transparency
- **Developer Tools**: CLI, API, containerized testing environment

### **Cooperative Features**
ICN is specifically designed for cooperative organizations:
- **Programmable Bylaws**: Encode governance as executable CCL contracts
- **Member Management**: Democratic invitation, voting, and role assignment
- **Resource Sharing**: Pool computing, storage, and economic resources
- **Transparent Decisions**: All governance actions cryptographically recorded
- **Anti-Extraction**: Regenerating mana economy prevents speculation

---

## 📦 Architecture Overview

ICN Core is organized as a modular Rust workspace with clear responsibilities:

### **Core Infrastructure**
- **`icn-runtime`** – Node orchestration, WASM execution, and job management
- **`icn-common`** – Shared types, cryptographic primitives, and utilities
- **`icn-api`** – HTTP API definitions and external interfaces

### **Identity & Security**
- **`icn-identity`** – DID management, credential verification, and cryptographic operations
- **`icn-dag`** – Content-addressed storage with multiple backend options

### **Governance & Economics**
- **`icn-governance`** – Proposal engine, voting mechanisms, and policy execution
- **`icn-economics`** – Mana accounting and economic policy enforcement
- **`icn-reputation`** – Trust scoring and contribution tracking

### **Networking & Computation**
- **`icn-network`** – P2P networking with libp2p integration
- **`icn-mesh`** – Distributed job scheduling and execution

### **Language & Tools**
- **`icn-ccl`** – Cooperative Contract Language compiler
- **`icn-cli`** – Command-line interface for all operations
- **`icn-node`** – Main daemon binary with HTTP server

See [Architecture Documentation](docs/ARCHITECTURE.md) for detailed component relationships.

---

## 📱 Cross-Platform Applications

ICN provides comprehensive user interfaces across all major platforms. Built with modern technologies for maximum code sharing and native performance.

### **Mobile & Desktop Apps** (React Native + Expo + Tauri)
- **`wallet-ui`** 📱 – Secure DID and key management interface
  - Platforms: iOS, Android, Web, Desktop
  - Features: DID creation, private key storage, mana tracking, job submission
  - Technology: React Native + Tamagui + Expo + Tauri

- **`agoranet`** 🗳️ – Governance deliberation and voting platform  
  - Platforms: iOS, Android, Web, Desktop
  - Features: Proposal creation, community deliberation, voting interface
  - Technology: React Native + Tamagui + Expo + Tauri

### **Web Applications** (React + Vite + Tailwind)
- **`web-ui`** 🌐 – Federation administration dashboard
  - Platform: Web browser (PWA-enabled)
  - Features: Member management, system monitoring, network configuration
  - Technology: React + Vite + TypeScript + Tailwind CSS

- **`explorer`** 🔍 – DAG viewer and network activity browser
  - Platform: Web browser (PWA-enabled)  
  - Features: DAG visualization, job tracking, network analytics
  - Technology: React + Vite + D3.js + Tailwind CSS

### **Shared Infrastructure**
- **`ui-kit`** 🎨 – Cross-platform component library (Tamagui)
- **`ts-sdk`** 🛠️ – TypeScript SDK for all frontend applications

### **Development Commands**
```bash
# Frontend development setup
just setup-frontend
just install-frontend

# Start individual apps
just dev-wallet      # Wallet UI (all platforms)
just dev-agoranet    # Governance interface
just dev-web-ui      # Admin dashboard  
just dev-explorer    # Network explorer

# Mobile development
just dev-mobile      # React Native apps on iOS/Android

# Desktop development  
just dev-desktop     # Tauri desktop apps

# Build for production
just build-frontend  # Build all frontend apps
```

**Platform Support Matrix**:
| App | Web | iOS | Android | Desktop (Windows/Mac/Linux) |
|-----|-----|-----|---------|------------------------------|
| Wallet UI | ✅ | ✅ | ✅ | ✅ |
| AgoraNet | ✅ | ✅ | ✅ | ✅ |
| Web UI | ✅ | 📱 | 📱 | 🔄 |
| Explorer | ✅ | 📱 | 📱 | 🔄 |

*Legend: ✅ Native app • 📱 Responsive web • 🔄 Future support*

See individual app READMEs for detailed setup and deployment instructions.

---

## 🛠️ Development

### Prerequisites

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

### Development Workflow

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

# Federation testing
just devnet        # Start 3-node test federation
just test-e2e      # End-to-end testing
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

**Detailed Guides**:
- [Developer Setup](docs/DEVELOPER_GUIDE.md) - Complete development environment
- [Contributing](CONTRIBUTING.md) - Code standards and workflow
- [Testing Strategy](docs/DEVELOPER_GUIDE.md#testing-strategy) - Test patterns

---

## 🌐 Community & Support

### Getting Help
- **Documentation**: [docs/README.md](docs/README.md) - Comprehensive guides
- **API Reference**: [docs/API.md](docs/API.md) - HTTP endpoints
- **Troubleshooting**: [docs/troubleshooting.md](docs/troubleshooting.md) - Common issues
- **Discussions**: [GitHub Discussions](https://github.com/InterCooperative/icn-core/discussions)

### Contributing
We welcome contributions in multiple areas:
- **Code**: Rust implementation, tests, and documentation
- **Governance**: CCL policies and cooperative bylaws
- **Research**: Economic models and governance patterns
- **Community**: Education, organizing, and outreach

See [Contributing Guidelines](CONTRIBUTING.md) for detailed information.

### Resources
- **Website**: [intercooperative.network](https://intercooperative.network)
- **Code**: [GitHub Repository](https://github.com/InterCooperative/icn-core)
- **Status**: [Current Progress](STATUS.md)
- **License**: [Apache 2.0](LICENSE)

---

## 🎭 Project Status

**Version**: 0.2.0-beta  
**Phase**: Production Readiness (Phase 5)  
**Implementation**: 77% Complete  

### Current Focus
🔧 **Configuration Management** - Ensure production services are used by default  
🔧 **Scale Testing** - Validate with 10+ node federations  
🔧 **Operational Excellence** - Monitoring, alerting, and automation  

### Recent Achievements
✅ **Real P2P Networking** with libp2p integration  
✅ **Cross-Node Job Execution** verified working  
✅ **Production Security** with Ed25519 cryptography  
✅ **Federation Management** with trust bootstrapping  

See [Status Report](STATUS.md) for comprehensive progress details.

---

## 🔮 Roadmap

### Completed Phases
- **Phase 1-4**: Foundation, networking, mesh computing, HTTP API

### Current & Upcoming
- **Phase 5** (Current): Production readiness and configuration management
- **Phase 6** (Q2 2025): Advanced foundation (ZK proofs, liquid delegation)
- **Phase 7** (Q3 2025): Federation interoperability
- **Phase 8** (Q4 2025): Application layer (wallets, UIs)

**Cooperative Infrastructure Expansion**:
- Cooperative banking (mutual credit, time banking, local currencies)
- Mutual aid networks (emergency response, resource sharing)
- Supply chain cooperation (sourcing, quality assurance)
- Worker cooperative tools (profit sharing, democratic coordination)

See [Strategic Roadmap](ICN_ROADMAP_2025.md) for complete timeline.

---

**ICN Core is production-ready infrastructure for building the cooperative digital economy. [Get started today](docs/beginner/README.md).**
