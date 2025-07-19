# ICN Core Documentation

> **InterCooperative Network (ICN)**: Production-ready infrastructure for a cooperative digital economy

## 🌟 **Start Here: Welcome to ICN**

**New to ICN?** Begin with our comprehensive introduction that explains the vision, principles, and transformative potential of cooperative digital infrastructure:

### **📖 [Welcome to the InterCooperative Network](INTRODUCTION.md)**

*A book-like introduction that serves as both welcome and manifesto—explaining why ICN exists, what it enables, and how you can participate in building the cooperative digital economy.*

**This is your essential starting point** whether you're a community organizer, developer, technologist, or movement builder.

---

## 🚀 Quick Navigation

### **For New Users**
- **[📖 Introduction & Manifesto](INTRODUCTION.md)** - *Start here: The vision and reality of cooperative infrastructure*
- **[⚡ Getting Started](guides/getting-started.md)** - *Run your first ICN node in 10 minutes*
- **[🎯 Core Concepts](guides/concepts.md)** - *Understand the fundamental principles*
- **[🏗️ Architecture Overview](guides/architecture.md)** - *See how everything fits together*

### **For Developers**
- **[💻 Developer Guide](guides/development.md)** - *Complete development environment setup*
- **[🔌 API Reference](api/)** - *HTTP endpoints, Host ABI, and protocol documentation*
- **[📚 Crate Documentation](crates/)** - *Implementation details for each component*
- **[🧪 Examples & Tutorials](examples/)** - *Practical code examples and patterns*

### **For Operators**
- **[🚀 Deployment Guide](guides/deployment.md)** - *Production deployment and operations*
- **[⚙️ Configuration](guides/configuration.md)** - *Network, storage, and runtime configuration*
- **[📊 Monitoring](guides/monitoring.md)** - *Metrics, logging, and observability*
- **[🔧 Troubleshooting](guides/troubleshooting.md)** - *Common issues and solutions*

---

## 📊 **Current Status: 77% Complete & Production-Ready**

ICN Core is **remarkably mature infrastructure**, not a prototype:

### ✅ **Production-Ready Components**
- **🏗️ Runtime Engine** - WASM execution with comprehensive security
- **🌐 P2P Networking** - Real libp2p-based mesh networking  
- **💼 Distributed Computing** - Job bidding, execution, and verification
- **🏛️ Democratic Governance** - Proposal creation and transparent voting
- **💰 Economic Systems** - Mana generation, spending, and reputation
- **🔒 Security Layer** - Ed25519 signatures and trust validation
- **📦 DAG Storage** - Content-addressed storage with integrity

### 🔧 **In Development**
- Additional storage backends and optimizations
- Enhanced federation management tools
- Advanced governance policy frameworks
- Extended mutual aid capabilities

**[View Complete Status →](STATUS.md)**

---

## 🏗️ **Core Architecture**

ICN consists of **18 specialized Rust crates** that work together to provide cooperative infrastructure:

### **📁 Foundation Layer**
- **[`icn-common`](crates/icn-common.md)** - *Shared types, utilities, and error handling*

### **🎛️ Orchestration Layer**  
- **[`icn-runtime`](crates/icn-runtime.md)** - *WASM execution, state management, and Host ABI*

### **🌐 Application Layer**
- **[`icn-mesh`](crates/icn-mesh.md)** - *Distributed computing and job orchestration*
- **[`icn-governance`](crates/icn-governance.md)** - *Democratic governance and proposals*
- **[`icn-economics`](crates/icn-economics.md)** - *Mana system and economic enforcement*

### **🔧 Infrastructure Layer**
- **[`icn-identity`](crates/icn-identity.md)** - *DID management and credential verification*
- **[`icn-dag`](crates/icn-dag.md)** - *Content-addressed storage and integrity*
- **[`icn-network`](crates/icn-network.md)** - *P2P networking and message routing*

### **🔌 Interface Layer**
- **[`icn-api`](crates/icn-api.md)** - *External interfaces and service traits*
- **[`icn-cli`](crates/icn-cli.md)** - *Command-line interface*
- **[`icn-node`](crates/icn-node.md)** - *Main node binary and HTTP server*

**[View All Crates →](crates/)**

---

## 🎯 **Documentation by Role**

### **👥 Community Organizers**
*You don't need to be technical to benefit from ICN*

- **[📖 Introduction](INTRODUCTION.md)** - Understand the cooperative vision
- **[⚡ Getting Started](guides/getting-started.md)** - See ICN in action
- **[🤝 Community Use Cases](guides/community-cases.md)** - Real-world applications
- **[🎯 Governance Guide](guides/governance.md)** - Democratic decision-making

### **💻 Application Developers**  
*Build cooperative applications on ICN infrastructure*

- **[💻 Developer Guide](guides/development.md)** - Development environment setup
- **[🔌 API Documentation](api/)** - Complete API reference
- **[🧩 Integration Patterns](guides/integration.md)** - Common development patterns
- **[🧪 Examples](examples/)** - Working code examples

### **🔧 Infrastructure Operators**
*Deploy and manage ICN infrastructure*

- **[🚀 Deployment Guide](guides/deployment.md)** - Production deployment
- **[⚙️ Configuration Reference](guides/configuration.md)** - All configuration options
- **[📊 Operations Manual](guides/operations.md)** - Monitoring and maintenance
- **[🆘 Incident Response](guides/incident-response.md)** - Handle operational issues

### **🏗️ Core Contributors**
*Contribute to ICN Core development*

- **[🛠️ Contributing Guide](../CONTRIBUTING.md)** - Code standards and workflow
- **[📐 Architecture Deep Dive](guides/architecture-detailed.md)** - Internal system design
- **[🧪 Testing Guide](guides/testing.md)** - Test frameworks and patterns
- **[📝 Documentation System](DOCUMENTATION_SYSTEM.md)** - Maintain documentation

---

## 🔧 **Working with the Code**

### **Quick Commands**
```bash
# Get started in 3 commands
git clone https://github.com/InterCooperative/icn-core
cd icn-core
just setup && just build && just run

# Development workflow
just test      # Run all tests
just lint      # Check code quality  
just docs      # Generate documentation
just check     # Quick compile check
```

### **Key Resources**
- **[📦 Cargo Workspace](../Cargo.toml)** - *18 crates working together*
- **[⚙️ Just Commands](../justfile)** - *Common development tasks*
- **[🔧 CI Pipeline](../.github/workflows/)** - *Automated testing and validation*
- **[🧪 Integration Tests](../tests/)** - *End-to-end system testing*

---

## 🌍 **Join the Movement**

### **Community & Support**
- **[💬 Community Forum](https://community.intercooperative.network)** - Connect with other communities
- **[💭 Development Chat](https://chat.intercooperative.network)** - Real-time development discussion
- **[📅 Monthly Calls](https://calendar.intercooperative.network)** - Regular community meetings
- **[🐛 Issue Tracker](https://github.com/InterCooperative/icn-core/issues)** - Report bugs and request features

### **Contributing**
- **[🤝 Contributing Guide](../CONTRIBUTING.md)** - How to contribute code
- **[📝 Documentation](DOCUMENTATION_SYSTEM.md)** - Help improve documentation
- **[🧪 Testing](guides/testing.md)** - Add test coverage
- **[🌐 Translation](guides/translation.md)** - Help translate documentation

### **Stay Connected**
- **[📰 Blog](https://blog.intercooperative.network)** - Project updates and insights
- **[🐦 Social Media](https://social.intercooperative.network)** - Follow development progress
- **[📧 Newsletter](https://newsletter.intercooperative.network)** - Monthly project updates

---

## 🔮 **What's Next**

The foundation is built. Now we're scaling the cooperative digital economy:

### **Short Term (1-3 months)**
- Complete documentation for remaining crates
- Enhanced federation management tools
- Advanced governance policy frameworks
- Production deployment improvements

### **Medium Term (3-6 months)**
- Multi-language client SDKs
- Advanced economic modeling tools
- Enhanced mutual aid capabilities
- Cross-federation coordination protocols

### **Long Term (6+ months)**
- AI-powered resource optimization
- Advanced privacy-preserving features
- Integration with external economic systems
- Global federation coordination infrastructure

**[View Full Roadmap →](ROADMAP.md)**

---

## 💡 **Need Help?**

- **📖 Start with the [Introduction](INTRODUCTION.md)** for the big picture
- **⚡ Try the [Getting Started Guide](guides/getting-started.md)** for hands-on experience
- **🔍 Search the [API Documentation](api/)** for technical details
- **💬 Ask in [Community Forum](https://community.intercooperative.network)** for support
- **🐛 [Report Issues](https://github.com/InterCooperative/icn-core/issues)** for bugs or feature requests

---

*Building the infrastructure for a cooperative digital economy, one community at a time.* 