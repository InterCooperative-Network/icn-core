# Contributing to ICN Core

**Welcome to the InterCooperative Network (ICN) Core!** We're excited that you want to contribute to **experimental infrastructure** for the cooperative digital economy.

---

## 🚧 **Current Project State**

ICN Core is **under heavy development** with **many stub implementations and incomplete features**. This is **experimental software** with:

- ⚠️ **P2P networking demonstrations** (some features stubbed)
- ⚠️ **Job execution prototypes** (security and reliability need work)
- ⚠️ **Governance foundations** (voting mechanisms partially implemented)
- ⚠️ **API scaffolding** (many endpoints return mock data)
- ⚠️ **Frontend applications** (connected to stub backends)
- ⚠️ **Cryptographic frameworks** (implementations may be incomplete)

**Key Focus**: The primary work involves **replacing stub implementations with real functionality** and **completing TODO items** throughout the codebase.

---

## 🚀 **Quick Start**

### **1. Get Up and Running (10 Minutes)**
```bash
# Clone and setup
git clone https://github.com/InterCooperative/icn-core
cd icn-core
just setup-all         # Backend + Frontend environment

# Verify what currently works (expect some failures)
just validate-all-stack # Validation may show stub implementations
just devnet            # Start 3-node federation (may have limitations)
```

### **2. Choose Your Area**
- **Backend (Rust)**: Replace stub implementations, complete TODO items, fix failing tests
- **Frontend (React/React Native)**: Connect to real backends, replace mock data
- **Security**: Implement proper cryptographic operations, harden key management  
- **Documentation**: Update docs to reflect actual implementation status
- **Testing**: Add tests for new implementations, improve coverage

### **3. Essential Reading**
- **[README.md](README.md)** - Complete project overview
- **[CONTEXT.md](CONTEXT.md)** - Philosophical foundation and current state
- **[docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md)** - Development workflow
- **[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)** - Complete navigation

---

## 🎭 **Current Phase Priorities**

### **Phase 5: Operational Excellence**

#### **High Priority: Configuration Management**
- Ensure production services are used by default (not stubs)
- Improve service selection and configuration logic
- Enhance deployment and operational readiness

#### **High Priority: Frontend Application Completion**
- **Web UI** (70% → 100%): Advanced monitoring, production deployment
- **Explorer** (65% → 100%): Real-time updates, advanced queries
- **Wallet** (60% → 100%): Credential management, cross-platform deployment
- **AgoraNet** (60% → 100%): Advanced governance, mobile optimization

#### **High Priority: Scale Testing**
- Test 10+ node federations (currently verified: 3+ nodes)
- Performance optimization for larger networks
- Stress testing and reliability improvements

#### **Medium Priority: API & SDK Enhancement**
- Complete TypeScript SDK coverage
- WebSocket support for real-time events
- Enhanced error handling and documentation

---

## 💻 **Development Areas**

### **Backend Development (Rust)**

#### **Current Completion Status**
| Area | Completion | Focus |
|------|------------|-------|
| **Core Infrastructure** | 100% | Polish and optimization |
| **Identity & Security** | 95% | ZK proof system expansion |
| **Governance & Economics** | 82% | Advanced governance patterns |
| **Networking & Computation** | 78% | Performance and scale testing |
| **Developer Tools** | 90% | Enhanced developer experience |

#### **Key Backend Opportunities**
- **Configuration Management**: Default to production services
- **Performance Optimization**: Multi-node federation efficiency
- **Monitoring**: Comprehensive observability and alerting
- **Security**: Enhanced audit trails and compliance features
- **API Enhancement**: Complete missing endpoints

### **Frontend Development**

#### **Technology Stack**
- **Cross-Platform**: React Native + Tamagui (iOS/Android/Web/Desktop)
- **Web-Only**: React + Vite + TypeScript + Tailwind CSS
- **Shared**: TypeScript SDK, UI component library

#### **Application Status**
- **`apps/web-ui`** - Federation administration dashboard
- **`apps/explorer`** - DAG viewer and network browser  
- **`apps/agoranet`** - Governance deliberation platform
- **`apps/wallet-ui`** - Secure DID and key management

#### **Key Frontend Opportunities**
- **Feature Completion**: Finish missing functionality in each app
- **Real-Time Updates**: WebSocket integration for live data
- **Mobile Experience**: Native iOS/Android deployment
- **TypeScript SDK**: Enhanced API coverage and error handling
- **Design System**: Complete UI component library

---

## 🛠️ **Development Workflow**

### **Setup Process**
```bash
# Complete environment setup
just setup-all              # Rust + Node.js + dependencies

# Backend development
just test                    # Run all tests
just lint                    # Code quality checks
just validate                # Complete validation

# Frontend development  
just dev-frontend            # All apps simultaneously
just dev-web-ui             # Specific app development
```

### **Development Commands**
```bash
# Backend
just build                   # Build all crates
just devnet                  # Multi-node test federation
just health-check            # Federation health validation

# Frontend
just setup-frontend          # Frontend environment setup
just dev-mobile             # React Native development
just dev-desktop            # Tauri desktop apps
just build-frontend         # Production builds

# Complete Stack
just validate-all-stack     # Full validation
just build-all-stack        # Build everything
```

### **Testing Strategy**
- **Unit Tests**: Individual component validation
- **Integration Tests**: Cross-crate interactions
- **End-to-End Tests**: Complete workflow validation
- **Frontend Tests**: Component and integration testing
- **Scale Tests**: Multi-node federation validation

---

## 📋 **Contribution Guidelines**

### **Code Standards**
- **Rust**: Follow existing patterns, comprehensive testing, rustdoc
- **Frontend**: TypeScript, component documentation, responsive design
- **Security**: All economic/governance changes need security review
- **Performance**: Consider multi-node federation impact

### **Pull Request Process**
1. **Fork and branch** from `main` (we use trunk-based development)
2. **Make focused changes** that clearly advance project goals
3. **Add comprehensive tests** for any changes
4. **Update documentation** for public API changes
5. **Run validation** with `just validate-all-stack`
6. **Create clear PR description** linking to any issues

### **Commit Message Format**
```
[area] Brief description

More detailed description if needed:
- What changed
- Why it changed
- Any breaking changes

Closes #issue-number
```

Examples:
- `[icn-api] Add WebSocket endpoints for real-time events`
- `[web-ui] Complete federation monitoring dashboard`
- `[config] Default to production services instead of stubs`

---

## 🎯 **Current "Good First Issues"**

### **Configuration & Operations**
- **Service Defaults**: Update contexts to use production services by default
- **Health Checks**: Add comprehensive health monitoring endpoints
- **Metrics**: Enhance Prometheus metrics collection and Grafana dashboards
- **Deployment**: Improve containerized deployment configurations

### **Frontend Applications**
- **Web UI Features**: Add missing monitoring and analytics features
- **Explorer Enhancements**: Real-time DAG updates and advanced queries
- **Wallet Features**: Complete credential management interface
- **AgoraNet Polish**: Enhanced governance workflow and mobile experience

### **API & SDK**
- **TypeScript Coverage**: Complete SDK coverage for all 60+ endpoints
- **WebSocket Support**: Real-time event streaming implementation
- **Error Handling**: Enhanced error types and user-friendly messages
- **Documentation**: API examples and comprehensive guides

### **Testing & Quality**
- **Scale Testing**: 10+ node federation validation scripts
- **Performance Testing**: Benchmarking and optimization
- **Security Testing**: Enhanced audit and compliance features
- **Frontend Testing**: Component and integration test coverage

---

## 📚 **Learning Resources**

### **Project Understanding**
- **[README.md](README.md)** - Project overview and capabilities
- **[CONTEXT.md](CONTEXT.md)** - Philosophical foundation and complete context
- **[docs/status/STATUS.md](docs/status/STATUS.md)** - Current progress (77-82% complete)
- **[ICN_API_REFERENCE.md](ICN_API_REFERENCE.md)** - All 60+ HTTP endpoints

### **Development Guides**
- **[docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md)** - Complete development workflow
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design and components
- **[.cursor/rules/](/.cursor/rules/)** - Comprehensive development rules
- **[justfile](justfile)** - All available development commands

### **Specific Areas**
- **[apps/web-ui/README.md](apps/web-ui/README.md)** - Frontend application guides
- **[packages/ts-sdk/README.md](packages/ts-sdk/README.md)** - TypeScript SDK
- **[crates/icn-api/README.md](crates/icn-api/README.md)** - HTTP API implementation

---

## 🌟 **Impact & Vision**

### **What Your Contribution Enables**
- **Real Federations**: Communities using ICN for actual coordination today
- **Democratic Governance**: Cryptographically verified democratic processes
- **Economic Cooperation**: Mana-based resource sharing without extraction
- **Privacy Preservation**: Zero-knowledge proofs for sensitive operations
- **Sovereign Infrastructure**: Community-owned and controlled technology

### **Current Production Use**
ICN Core is **working infrastructure** that enables:
- Multi-node federations with real P2P networking
- Cross-node job execution with verified results
- Democratic governance with programmable policies
- Economic coordination with regenerating mana
- Privacy-preserving credential systems

### **Your Role**
You're contributing to **production infrastructure** that cooperatives and communities use today. Every improvement you make directly enhances the tools that enable democratic coordination and resource sharing without centralized control.

---

## 🤝 **Community & Support**

### **Getting Help**
- **GitHub Discussions**: [Community discussions](https://github.com/InterCooperative/icn-core/discussions)
- **Issues**: [Bug reports and feature requests](https://github.com/InterCooperative/icn-core/issues)
- **Documentation**: Comprehensive guides in `docs/` directory

### **Communication Guidelines**
- **Be Respectful**: Follow our [Code of Conduct](CODE_OF_CONDUCT.md)
- **Be Specific**: Provide clear descriptions and examples
- **Be Collaborative**: ICN is built by and for the cooperative community

---

**Thank you for contributing to experimental cooperative infrastructure! Together, we're building the foundation of a cooperative digital economy - help us make it real.** 