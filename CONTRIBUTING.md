# Contributing to ICN Core

**Welcome to the InterCooperative Network (ICN) Core!** We're excited that you want to contribute to **experimental infrastructure** for the cooperative digital economy.

---

## 🚧 **Current Project State**

ICN Core is **advanced development software** with **substantial working implementations** across core domains. While **NOT production-ready**, it has significantly more functional code than typical early-stage projects. This is **experimental cooperative infrastructure** with:

- ✅ **CCL WASM Compilation**: Full pipeline from CCL source to WASM execution
- ✅ **Multi-Backend Storage**: PostgreSQL, RocksDB, SQLite, Sled all operational  
- ✅ **P2P Networking**: libp2p with gossipsub and Kademlia DHT working
- ✅ **Governance Workflows**: Proposals, ranked choice voting, budget allocation functional
- ✅ **Economic Systems**: Mana ledgers, resource tokens, mutual credit working
- ✅ **Mesh Computing**: End-to-end job submission, bidding, execution, receipt generation
- ✅ **Identity Management**: DID creation, credential verification, signing operational
- ⚠️ **Security Review**: Cryptographic implementations need production-level audit
- ⚠️ **Scale Testing**: Works in development, needs validation at production scale

**Key Focus**: The primary work involves **security hardening, production readiness, and operational excellence** rather than building missing core functionality.

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

### **Phase 6: Production Readiness (Current Focus)**

#### **High Priority: Security Hardening**
- Professional cryptographic security audit of all implementations
- Attack vector analysis and penetration testing  
- Key management improvements and HSM integration
- Enhanced access control and authorization systems

#### **High Priority: Operational Excellence**
- Comprehensive monitoring and alerting (Prometheus/Grafana)
- Error recovery and fault tolerance improvements
- Scale testing with 10+ node federations
- Performance optimization for production workloads

#### **High Priority: Frontend Application Completion**
- **Web UI** (80% → 100%): Advanced monitoring, production deployment
- **Explorer** (75% → 100%): Real-time updates, advanced queries  
- **Wallet** (70% → 100%): Credential management, cross-platform deployment
- **AgoraNet** (70% → 100%): Advanced governance, mobile optimization

#### **Medium Priority: API & SDK Enhancement**
- Complete TypeScript SDK coverage for all 60+ endpoints
- WebSocket support for real-time events
- Enhanced error handling and user-friendly messages
- Comprehensive API documentation with examples

---

## 💻 **Development Areas**

### **Backend Development (Rust)**

#### **Current Completion Status**
| Area | Completion | Focus |
|------|------------|-------|
| **Core Infrastructure** | 100% | Polish and optimization |
| **Identity & Security** | 95% | Security audit and hardening |
| **Governance & Economics** | 85% | Advanced governance patterns |
| **Networking & Computation** | 80% | Performance and scale testing |
| **Developer Tools** | 90% | Enhanced developer experience |

#### **Key Backend Opportunities**
- **Security Hardening**: Professional cryptographic audit and production-level security
- **Performance Optimization**: Multi-node federation efficiency and scale testing
- **Monitoring & Observability**: Comprehensive metrics, alerting, and operational tools
- **Production Readiness**: Deployment automation, error recovery, and operational procedures
- **API Enhancement**: Complete missing endpoints and improve error handling

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

### **RFC Process for Major Changes**
For significant design decisions, protocol changes, or cross-cutting concerns:

1. **Create RFC**: Use the template in `docs/rfc/rfc-template.md`
2. **Community Discussion**: Open GitHub issue linking to RFC for feedback
3. **Technical Review**: Maintainer and expert review process
4. **Implementation**: Accepted RFCs guide development work

See **[docs/rfc/README.md](docs/rfc/README.md)** for complete RFC process details.

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

### **Security & Production Readiness**
- **Security Audit**: Review cryptographic implementations for production readiness
- **Monitoring**: Enhance Prometheus metrics collection and Grafana dashboards
- **Error Recovery**: Improve fault tolerance and recovery procedures
- **Performance Testing**: Benchmarking and optimization for production workloads

### **Frontend Applications**
- **Web UI Features**: Complete missing monitoring and analytics features
- **Explorer Enhancements**: Real-time DAG updates and advanced query capabilities
- **Wallet Features**: Complete credential management and cross-platform deployment
- **AgoraNet Polish**: Enhanced governance workflows and mobile experience

### **API & SDK**
- **TypeScript Coverage**: Complete SDK coverage for all 60+ endpoints
- **WebSocket Support**: Real-time event streaming implementation
- **Error Handling**: Enhanced error types and user-friendly messages
- **Documentation**: API examples and comprehensive integration guides

### **Testing & Quality**
- **Scale Testing**: 10+ node federation validation and stress testing
- **Security Testing**: Enhanced audit and compliance testing frameworks
- **Frontend Testing**: Component and integration test coverage
- **Performance Testing**: Benchmarking and optimization validation

---

## 📚 **Learning Resources**

### **Project Understanding**
- **[README.md](README.md)** - Project overview and capabilities
- **[CONTEXT.md](CONTEXT.md)** - Philosophical foundation and complete context
- **[PROJECT_STATUS_AND_ROADMAP.md](PROJECT_STATUS_AND_ROADMAP.md)** - Current progress and development roadmap
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

### **Stay Updated**
- **Monthly Status Updates**: Follow GitHub Discussions for regular progress updates
- **Quarterly Roadmap Reviews**: Major roadmap and priority updates
- **RFC Process**: Participate in design discussions for major changes
- **Documentation**: Check [COMMUNICATION_PROCESS.md](docs/COMMUNICATION_PROCESS.md) for update schedule

See **[docs/COMMUNICATION_PROCESS.md](docs/COMMUNICATION_PROCESS.md)** for complete communication guidelines and update schedule.

---

**Thank you for contributing to experimental cooperative infrastructure! Together, we're building the foundation of a cooperative digital economy - help us make it real.** 