# ICN Core Documentation Index

> **Quick Navigation Guide for ICN Core's Comprehensive Documentation**  
> **Status**: Production-ready infrastructure (77-82% complete)

---

## 🚀 **Getting Started**

### **Quick Start (10 Minutes)**
- **[README.md](README.md)** - Complete project overview and quick start guide
- **[Quick Start Commands](#quick-commands)** - Essential commands to get running

### **For New Users**
- **[docs/beginner/README.md](docs/beginner/README.md)** - Beginner-friendly introduction
- **[CONTEXT.md](CONTEXT.md)** - Complete project context and philosophical foundation
- **[docs/INTRODUCTION.md](docs/INTRODUCTION.md)** - Vision, principles, and capabilities

### **For Developers**
- **[docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md)** - Complete development setup and workflow
- **[ICN_API_REFERENCE.md](ICN_API_REFERENCE.md)** - All 60+ HTTP endpoints
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design and component overview

---

## 📊 **Current Status & Planning**

### **Implementation Progress**
- **[docs/status/STATUS.md](docs/status/STATUS.md)** - Overall progress (77-82% complete)
- **[docs/status/ICN_IMPLEMENTATION_STATUS_MATRIX.md](docs/status/ICN_IMPLEMENTATION_STATUS_MATRIX.md)** - Detailed completion matrix
- **[docs/status/ICN_CORE_CURRENT_STATE_2025.md](docs/status/ICN_CORE_CURRENT_STATE_2025.md)** - Current state report

### **Strategic Planning**
- **[docs/planning/ICN_ROADMAP_2025.md](docs/planning/ICN_ROADMAP_2025.md)** - Complete strategic roadmap
- **[docs/planning/ICN_NEXT_STEPS_SUMMARY.md](docs/planning/ICN_NEXT_STEPS_SUMMARY.md)** - Immediate next steps
- **[docs/planning/ROADMAP_SUMMARY.md](docs/planning/ROADMAP_SUMMARY.md)** - Quick roadmap overview
- **[docs/SYSTEM_COMPLETENESS_ROADMAP.md](docs/SYSTEM_COMPLETENESS_ROADMAP.md)** - System completion planning

### **Phase Progress**
- **[docs/phases/PHASE_5_COMPLETE_IMPLEMENTATION.md](docs/phases/PHASE_5_COMPLETE_IMPLEMENTATION.md)** - Phase 5 achievements

---

## 🏗️ **Technical Architecture**

### **Core Architecture**
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design and component relationships
- **[docs/ASYNC_OVERVIEW.md](docs/ASYNC_OVERVIEW.md)** - Async/await patterns and design

### **API & Integration**
- **[ICN_API_REFERENCE.md](ICN_API_REFERENCE.md)** - Complete HTTP API (60+ endpoints)
- **[docs/API.md](docs/API.md)** - API design principles and patterns
- **[crates/icn-api/README.md](crates/icn-api/README.md)** - API implementation details

### **Backend Crates**

#### **Core Infrastructure (100% Complete)**
- **[crates/icn-runtime/README.md](crates/icn-runtime/README.md)** - Node orchestration and WASM execution
- **[crates/icn-common/README.md](crates/icn-common/README.md)** - Shared types and utilities
- **[crates/icn-api/README.md](crates/icn-api/README.md)** - HTTP API definitions (60+ endpoints)
- **[crates/icn-protocol/README.md](crates/icn-protocol/README.md)** - P2P message formats

#### **Identity & Security (95% Complete)**
- **[crates/icn-identity/README.md](crates/icn-identity/README.md)** - DID management and credentials
- **[crates/icn-dag/README.md](crates/icn-dag/README.md)** - Content-addressed storage
- **[crates/icn-zk/README.md](crates/icn-zk/README.md)** - Zero-knowledge circuits

#### **Governance & Economics (82% Complete)**
- **[crates/icn-governance/README.md](crates/icn-governance/README.md)** - Proposal engine and voting
- **[crates/icn-economics/README.md](crates/icn-economics/README.md)** - Mana and economic policy
- **[docs/economic-automation.md](docs/economic-automation.md)** - Economic automation engine overview
- **[crates/icn-reputation/README.md](crates/icn-reputation/README.md)** - Trust scoring
- **[crates/icn-eventstore/README.md](crates/icn-eventstore/README.md)** - Event sourcing utilities

#### **Networking & Computation (78% Complete)**
- **[crates/icn-network/README.md](crates/icn-network/README.md)** - P2P networking with libp2p
- **[crates/icn-mesh/README.md](crates/icn-mesh/README.md)** - Distributed job execution

#### **Developer Tools & SDKs (90% Complete)**
- **[crates/icn-cli/README.md](crates/icn-cli/README.md)** - Command-line interface
- **[crates/icn-node/README.md](crates/icn-node/README.md)** - Main daemon binary
- **[crates/icn-sdk/README.md](crates/icn-sdk/README.md)** - High-level Rust SDK
- **[crates/icn-templates/README.md](crates/icn-templates/README.md)** - Template management
- **[crates/job-audit/README.md](crates/job-audit/README.md)** - Job auditing

---

## 📱 **Frontend Applications**

### **Cross-Platform Apps (React Native + Tamagui)**
- **[apps/wallet-ui/README.md](apps/wallet-ui/README.md)** - Secure DID and key management (iOS/Android/Web/Desktop)
- **[apps/agoranet/README.md](apps/agoranet/README.md)** - Governance deliberation platform (iOS/Android/Web/Desktop)

### **Web Applications (React + TypeScript)**
- **[apps/web-ui/README.md](apps/web-ui/README.md)** - Federation administration dashboard
- **[apps/explorer/README.md](apps/explorer/README.md)** - DAG viewer and network browser

### **Shared Frontend Infrastructure**
- **[packages/ui-kit/README.md](packages/ui-kit/README.md)** - Cross-platform component library
- **[packages/ts-sdk/README.md](packages/ts-sdk/README.md)** - TypeScript SDK
- **[packages/ccl-visual-editor/README.md](packages/ccl-visual-editor/README.md)** - Visual contract editor

---

## 🗳️ **Governance & CCL**

### **Cooperative Contract Language**
- **[icn-ccl/README.md](icn-ccl/README.md)** - CCL language overview
- **[icn-ccl/CCL_FEATURE_ANALYSIS.md](icn-ccl/CCL_FEATURE_ANALYSIS.md)** - Feature analysis (95% complete)
- **[icn-ccl/feature_roadmap.md](icn-ccl/feature_roadmap.md)** - CCL development roadmap
- **[icn-ccl/working_features_demo.rs](icn-ccl/working_features_demo.rs)** - Working features demonstration

### **Governance Implementation**
- **[docs/ccl/ccl_wasm_tasks.md](docs/ccl/ccl_wasm_tasks.md)** - CCL WASM backend implementation
- **[docs/ccl/ccl_visual_editor_plan.md](docs/ccl/ccl_visual_editor_plan.md)** - Visual editor plan
- **[docs/CCL_INTEGRATION_SUMMARY.md](docs/CCL_INTEGRATION_SUMMARY.md)** - CCL integration status

---

## 🏛️ **Cooperative Infrastructure**

### **Cooperative Features**
- **[docs/COOPERATIVE_ROADMAP.md](docs/COOPERATIVE_ROADMAP.md)** - Cooperative infrastructure implementation
- **[docs/ICN_FEATURE_OVERVIEW.md](docs/ICN_FEATURE_OVERVIEW.md)** - Complete feature overview

### **Philosophy & Vision**
- **[docs/INTRODUCTION.md](docs/INTRODUCTION.md)** - ICN vision and capabilities
- **[CONTEXT.md](CONTEXT.md)** - Philosophical foundation and complete context

---

## 🛠️ **Development & Operations**

### **Development Workflow**
- **[docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md)** - Complete development guide
- **[justfile](justfile)** - All development commands (backend + frontend)
- **[.cursor/rules/](/.cursor/rules/)** - Cursor AI agent rules and guidelines

### **Testing & Deployment**
- **[icn-devnet/README.md](icn-devnet/README.md)** - Containerized development network
- **[docs/troubleshooting.md](docs/troubleshooting.md)** - Common issues and solutions

### **Security & Validation**
- **[docs/SECURITY.md](docs/SECURITY.md)** - Security patterns and validation
- **[docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - Comprehensive troubleshooting

---

## 🌐 **Community & Contributing**

### **Getting Involved**
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contribution guidelines
- **[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)** - Community standards
- **[AGENTS.md](AGENTS.md)** - AI agent collaboration guidelines

### **Community Resources**
- **GitHub Discussions**: [Community discussions](https://github.com/InterCooperative/icn-core/discussions)
- **Issues**: [Bug reports and feature requests](https://github.com/InterCooperative/icn-core/issues)
- **Website**: [intercooperative.network](https://intercooperative.network)

---

## 📈 **Monitoring & Analytics**

### **Performance & Metrics**
- **[docs/metrics/](docs/metrics/)** - Performance monitoring and metrics
- **[docs/observability/](docs/observability/)** - System observability patterns

---

## 🔗 **External Resources**

### **Related Projects**
- **ICN CCL**: [Standalone CCL compiler](https://github.com/InterCooperative/icn-ccl)
- **ICN Documentation**: [Complete documentation site](https://github.com/InterCooperative/icn-docs)

### **Academic & Research**
- **[docs/research/](docs/research/)** - Research papers and academic work
- **[docs/specifications/](docs/specifications/)** - Technical specifications

---

## ⚡ **Quick Commands**

### **Backend Development**
```bash
# Setup and build
just setup && just build

# Run tests and validation
just test
just validate

# Start development federation
just devnet
```

### **Frontend Development**
```bash
# Setup frontend environment
just setup-frontend

# Start all frontend apps
just dev-frontend

# Start specific apps
just dev-web-ui     # Admin dashboard
just dev-explorer   # Network explorer
just dev-wallet     # Wallet app
just dev-agoranet   # Governance app
```

### **Complete Stack**
```bash
# Full environment setup
just setup-all

# Complete validation
just validate-all-stack

# Build everything
just build-all-stack
```

---

## 📍 **Finding What You Need**

| If you want to... | Start here... |
|-------------------|---------------|
| **Understand ICN's vision** | [CONTEXT.md](CONTEXT.md) → [docs/INTRODUCTION.md](docs/INTRODUCTION.md) |
| **Get ICN running quickly** | [README.md](README.md) → [Quick Commands](#quick-commands) |
| **Learn to develop on ICN** | [docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md) → [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) |
| **Use the HTTP API** | [ICN_API_REFERENCE.md](ICN_API_REFERENCE.md) → [crates/icn-api/README.md](crates/icn-api/README.md) |
| **Work on frontend apps** | [apps/web-ui/README.md](apps/web-ui/README.md) → [packages/ts-sdk/README.md](packages/ts-sdk/README.md) |
| **Write governance contracts** | [icn-ccl/README.md](icn-ccl/README.md) → [docs/ccl/](docs/ccl/) |
| **Understand current progress** | [docs/status/STATUS.md](docs/status/STATUS.md) → [docs/planning/](docs/planning/) |
| **Contribute to ICN** | [CONTRIBUTING.md](CONTRIBUTING.md) → [docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md) |
| **Deploy ICN** | [icn-devnet/README.md](icn-devnet/README.md) → [docs/deployment/](docs/deployment/) |
| **Troubleshoot issues** | [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) → [docs/troubleshooting.md](docs/troubleshooting.md) |

---

**ICN Core is production-ready infrastructure for building the cooperative digital economy. This documentation covers the complete system - use the index above to navigate efficiently.** 