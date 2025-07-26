# ICN Core Beginner Quickstart

This guide helps you get started with the InterCooperative Network core repository. ICN Core is **advanced development software** with substantial working implementations across cooperative infrastructure domains.

## 🚧 What You're Getting Into

ICN Core is **NOT production-ready** but has significantly more functional code than typical early-stage projects:
- ✅ **CCL WASM Compilation**: Full pipeline working
- ✅ **Multi-Backend Storage**: PostgreSQL, RocksDB, SQLite, Sled operational
- ✅ **P2P Networking**: libp2p with gossipsub and Kademlia DHT working
- ✅ **Governance & Economics**: Voting, mana ledgers, resource tokens functional
- ⚠️ **Security Review Needed**: Cryptographic implementations need production audit
- ⚠️ **Scale Testing Required**: Works in development, needs production-scale validation

**Current Focus**: Security hardening, production readiness, and operational excellence.

## Quick Start (10 Minutes)

### 1. Clone and Setup
```bash
git clone https://github.com/InterCooperative-Network/icn-core
cd icn-core
just setup                    # install dependencies and tools
```

### 2. Verify Current State
```bash
just test-quick               # run basic tests (no persistence features)
just build                   # build with default features
```

### 3. Explore the Federation
```bash
just devnet                  # start 3-node test federation (if build successful)
```

## Understanding the Project

### Key Resources
- **[CONTRIBUTING.md](../../CONTRIBUTING.md)** - How to contribute effectively
- **[PROJECT_STATUS_AND_ROADMAP.md](../../PROJECT_STATUS_AND_ROADMAP.md)** - Current progress and priorities
- **[CONTEXT.md](../../CONTEXT.md)** - Complete project vision and philosophy
- **[docs/rfc/README.md](../rfc/README.md)** - Design decisions and open questions

### Stay Updated
- **Monthly Status Updates**: Follow GitHub Discussions for progress reports
- **RFC Process**: Participate in major design discussions
- **Communication Process**: See [docs/COMMUNICATION_PROCESS.md](../COMMUNICATION_PROCESS.md)

## Next Steps

### For Developers
1. **Read [Developer Guide](../DEVELOPER_GUIDE.md)** - Complete development workflow
2. **Review [Architecture](../ARCHITECTURE.md)** - System design and components
3. **Check [API Reference](../../ICN_API_REFERENCE.md)** - 60+ HTTP endpoints

### For Contributors
1. **Review [Current "Good First Issues"](../../CONTRIBUTING.md#current-good-first-issues)**
2. **Choose your focus area**: Security, Frontend, API, Testing
3. **Join RFC discussions** for major design decisions

### For Cooperatives
1. **Read [ICN Feature Overview](../ICN_FEATURE_OVERVIEW.md)** - Complete capabilities
2. **Explore [Frontend Apps](../../apps/)** - Web UI, Explorer, Wallet, AgoraNet
3. **Review [Deployment Guide](../deployment/deployment-guide.md)** - Federation setup

## Getting Help

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Community discussion and questions
- **Documentation**: Comprehensive guides in `docs/` directory
- **Communication Process**: Regular updates and community engagement

Remember: This is advanced development infrastructure for cooperative communities. Every contribution helps build the foundation of a cooperative digital economy!
