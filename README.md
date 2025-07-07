# ICN Core

`icn-core` is the reference implementation of the InterCooperative Network (ICN) protocol, written in Rust.
It provides the foundational crates for building ICN nodes, CLI tools, and other related infrastructure.

**ICN Mission**: Replace every major function of the state and corporation with programmable, federated, democratic infrastructure—built for actual human needs and scalable solidarity.

For full architecture, philosophy, and comprehensive feature overview, see:
- [CONTEXT.md](CONTEXT.md) - Core philosophy and architectural principles
- [docs/ICN_FEATURE_OVERVIEW.md](docs/ICN_FEATURE_OVERVIEW.md) - Complete feature set (current and planned)

## Overview

The InterCooperative Network is a comprehensive platform for building federated, cooperative digital infrastructure. This repository contains the core building blocks for autonomous federated systems that enable cooperative coordination without relying on traditional state or corporate structures.

**Key Differentiators:**
- **100% Rust**: Memory-safe, performant foundation
- **Cooperative Virtual Machine (CoVM)**: WASM-first deterministic execution
- **DAG Ledger**: Content-addressed storage without blockchain complexity  
- **Scoped Identity**: DIDs with built-in federation support
- **Anti-Capitalist Economics**: Purpose-bound tokens that cannot be speculated on
- **Governance as Code**: Programmable bylaws via Cooperative Contract Language (CCL)

## Current Project Status (Production-Ready Foundation)

The project has achieved a significant milestone with a production-ready foundation featuring:

### **✅ Core Infrastructure Complete**
- **Modular Crate Structure**: Well-defined crates for all major domains
- **Real Protocol Data Models**: DIDs, CIDs, DagBlocks, governance primitives
- **Multiple Storage Backends**: SQLite, RocksDB, Sled, File-based persistence
- **API Layer**: Comprehensive REST endpoints with authentication and TLS
- **P2P Mesh Networking**: libp2p integration with Kademlia peer discovery
- **Production Security**: Ed25519 cryptographic signing with memory protection
- **Comprehensive Error Handling**: Robust error propagation and user feedback

### **✅ Governance & Economics**
- **CCL Compiler**: Complete Cooperative Contract Language toolchain
- **Governance Engine**: Proposals, voting, quorum, and policy execution with persistence
- **Mana System**: Regenerating resource tokens with multiple ledger backends
- **Identity Management**: DID-based identity with verifiable credentials
- **Reputation System**: Contribution tracking and trust metrics with persistence
- **Federation Sync**: Cross-federation governance synchronization

### **✅ Distributed Computing**
- **Mesh Job Execution**: WASM-sandboxed distributed computation
- **Execution Receipts**: Cryptographically signed proof of work
- **Load Balancing**: Intelligent job routing based on capacity and reputation
- **Multi-Node Federation**: Production-ready cluster coordination
- **Resource Management**: Mana-based economic enforcement

### **✅ Developer Experience**
- **Comprehensive CLI**: Full node management with federation commands
- **HTTP API**: Machine-readable endpoints with authentication and TLS
- **Containerized Devnet**: Multi-node federation testing environment
- **Rich Documentation**: Onboarding guides, API docs, governance examples
- **Monitoring**: Prometheus metrics and audit logging

### **📊 Implementation Status**
- **Foundation**: 8/9 features complete (89%)
- **Governance**: 7/11 features complete (64%)
- **Economics**: 6/12 features complete (50%)
- **Computing**: 6/11 features complete (55%)
- **Security**: 5/8 features complete (63%)
- **Overall**: 65/104 total features complete (63%)

See [docs/ICN_FEATURE_OVERVIEW.md](docs/ICN_FEATURE_OVERVIEW.md) for the complete feature breakdown and roadmap.

### Rust Toolchain

This repository is pinned to the nightly Rust toolchain via `rust-toolchain.toml`.
Install it with:

```bash
rustup toolchain install nightly
rustup override set nightly
```

## Getting Started

Refer to `docs/ONBOARDING.md` for detailed instructions on prerequisites, setup, building, testing, and running the components. The latest API documentation is available at [https://intercooperative.network/docs](https://intercooperative.network/docs).

### Quick CLI Examples:

```bash
# Build with libp2p support and the default `sled` persistence backend
cargo build --features with-libp2p

# Build using the SQLite backend
cargo build --no-default-features --features "with-libp2p persist-sqlite"

# Build using the RocksDB backend
cargo build --no-default-features --features "with-libp2p persist-rocksdb"

# Start a node with persistent storage, P2P, and TLS enabled
./target/debug/icn-node \
  --enable-p2p \
  --p2p-listen-addr /ip4/0.0.0.0/tcp/4001 \
  --storage-backend sqlite \
  --storage-path ./icn_data/node1.sqlite \
  --mana-ledger-backend sled \
  --mana-ledger-path ./icn_data/mana1.sled \
  --auth-token "secure-api-token" \
  --tls-cert-path ./certs/server.crt \
  --tls-key-path ./certs/server.key

# In a second terminal start another node connecting to the first
./target/debug/icn-node \
  --enable-p2p \
  --p2p-listen-addr /ip4/0.0.0.0/tcp/4002 \
  --bootstrap-peers /ip4/127.0.0.1/tcp/4001/p2p/<PEER_ID> \
  --storage-backend sqlite \
  --storage-path ./icn_data/node2.sqlite \
  --mana-ledger-backend sled \
  --mana-ledger-path ./icn_data/mana2.sled

# Interact with a node via the CLI
./target/debug/icn-cli info
./target/debug/icn-cli status
./target/debug/icn-cli governance propose "Increase mesh job timeout to 300 seconds"
./target/debug/icn-cli mesh submit-job echo-job.json

# Federation management commands
./target/debug/icn-cli federation join <PEER_ID>
./target/debug/icn-cli federation status
./target/debug/icn-cli federation leave <PEER_ID>
./target/debug/icn-cli federation list-peers
```

### Justfile Commands

Common development tasks are defined in a `justfile` at the repository root. Install [just](https://github.com/casey/just) and run:

```bash
just format   # check formatting
just lint     # run clippy
just test     # execute all tests
just build    # build all crates
just devnet   # launch the containerized federation devnet
icn-devnet/launch_federation.sh # build and test the federation containers
```

Before running `just format` or `cargo fmt`, make sure the `rustfmt` component is installed:

```bash
rustup component add rustfmt
```

### Enabling Peer Discovery and Persistent Storage

1. **Compile with libp2p support** using `cargo build --features with-libp2p`.
2. Start each node with `--enable-p2p` and a unique `--p2p-listen-addr`.
3. Provide known peers via `--bootstrap-peers` to join an existing mesh.
4. Use `--storage-backend sqlite|sled|rocksdb|file` with a dedicated `--storage-path` to persist DAG blocks and governance state across restarts.
5. Configure `--mana-ledger-backend sled|sqlite|rocksdb|file` with `--mana-ledger-path` for persistent mana accounting.
6. Optional: Enable API security with `--auth-token` and TLS with `--tls-cert-path` and `--tls-key-path`.

For multi-node testing instructions, see [Libp2p Integration Tests](MULTI_NODE_GUIDE.md#libp2p-integration-tests).

### Metrics & Observability

ICN crates expose Prometheus metrics using the `prometheus-client` crate. When
running `icn-node` with the embedded HTTP server enabled, scrape metrics at the
`/metrics` endpoint. Counters such as `host_submit_mesh_job_calls` and gauges for
network latency are collected automatically.

## ICN Philosophy & Design Principles

### **Core Values**
- **Anti-Capitalist Design**: Every choice prioritizes collective benefit over extraction
- **Nonviolent Infrastructure**: Replace systemic violence with cooperative coordination
- **Revolutionary Pluralism**: Enable local autonomy within networked solidarity
- **Dignity & Autonomy**: Technology that enhances human agency

### **Technical Principles**
- **Deterministic Execution**: All core logic is predictable and verifiable
- **Purpose-Bound Economics**: Tokens scoped to specific capabilities, no speculation
- **Governance as Code**: Bylaws and policies encoded in CCL and executed automatically
- **Federated Autonomy**: Local control with voluntary federation protocols

### **Strategic Vision**
- **Systemic Sovereignty**: Autonomous systems independent of state control
- **Post-Capitalist Coordination**: Tools for economic organization beyond markets
- **Consciousness Architecture**: Programmable collective decision-making
- **Collective Liberation**: Infrastructure for universal human flourishing

## Error Handling Philosophy

This project prioritizes robust and clear error handling to improve developer experience and system reliability:

1. **No Panics in Libraries**: Library crates avoid `panic!` for recoverable errors, returning `Result<T, CommonError>` instead.
2. **Specific Error Variants**: The `icn_common::CommonError` enum defines comprehensive error types with contextual information.
3. **Clear Error Messages**: Error variants include detailed context for debugging and user feedback.
4. **Graceful Handling in Binaries**: Executables catch errors, provide user-friendly messages, and exit with appropriate status codes.
5. **Error Propagation**: Errors are propagated up the call stack with additional context at each layer.

This approach ensures errors are not silently ignored and provides clear debugging information for developers.

## Crate Descriptions

This workspace is organized into several crates, each with a specific focus:

### **Core Infrastructure**
- **`icn-common`**: Shared data structures, types, utilities, and error definitions
- **`icn-runtime`**: WASM execution environment, job orchestration, and host ABI
- **`icn-api`**: API endpoints, DTOs, and service traits for external interfaces

### **Identity & Security**  
- **`icn-identity`**: Decentralized identity (DIDs), verifiable credentials, and cryptographic operations
- **`icn-dag`**: Content-addressed DAG storage, manipulation, and receipt anchoring

### **Governance & Economics**
- **`icn-governance`**: Proposal systems, voting mechanisms, and policy execution
- **`icn-economics`**: Mana accounting, scoped token management, and economic policies
- **`icn-reputation`**: Reputation scoring, contribution tracking, and trust metrics

### **Networking & Computation**
- **`icn-network`**: P2P networking with libp2p, peer discovery, and federation sync
- **`icn-mesh`**: Distributed job orchestration, scheduling, and execution management

### **Language & Protocol**
- **[`icn-ccl`](icn-ccl/README.md)**: Cooperative Contract Language compiler producing WASM modules
- **`icn-protocol`**: Core message formats, communication protocols, and serialization

### **User Interfaces**
- **`icn-cli`**: Command-line interface for users and administrators
- **`icn-node`**: Main binary for running ICN daemon processes with HTTP API

More detailed information can be found in the `README.md` file within each crate's directory.

## Further Reading

### **Architecture & Design**
* [Complete Feature Overview](docs/ICN_FEATURE_OVERVIEW.md) – comprehensive feature breakdown
* [Context & Philosophy](CONTEXT.md) – core principles and architectural vision
* [Development Workflow](docs/ONBOARDING.md) – getting started guide
* [Multi-Node Setup](MULTI_NODE_GUIDE.md) – federation deployment guide

### **Governance & Economics**
* [RFC Index](icn-docs/rfcs/README.md) – design proposals and specifications
* [RFC 0010: ICN Governance Core](icn-docs/rfcs/0010-governance-core.md) – governance framework
* [Mana Policies](docs/mana_policies.md) – economic policy examples
* [CCL Examples](icn-ccl/tests/contracts/) – governance contract templates

### **Development Resources**
* Crate documentation: [icn-common](crates/icn-common/README.md), [icn-dag](crates/icn-dag/README.md), [icn-identity](crates/icn-identity/README.md), [icn-mesh](crates/icn-mesh/README.md), [icn-governance](crates/icn-governance/README.md), [icn-runtime](crates/icn-runtime/README.md), [icn-network](crates/icn-network/README.md)
* [API Documentation](docs/API.md) – HTTP endpoints and programmatic interfaces
* [Deployment Guide](docs/deployment-guide.md) – production deployment instructions (see [circuit breaker & retry settings](docs/deployment-guide.md#circuit-breaker-and-retry))

## Community & Contribution

We welcome contributions to build infrastructure for cooperative digital civilization! ICN is developed by a global community of developers, cooperatives, and researchers committed to human flourishing.

### **How to Contribute**
- **Code**: Rust, JavaScript, Python, documentation improvements
- **Governance**: CCL policies, governance templates, cooperative bylaws
- **Research**: Academic papers, case studies, economic analysis
- **Community**: Outreach, education, organizing, cooperative onboarding

See our [Contributing Guidelines](CONTRIBUTING.md) for detailed information on coding conventions, the pull request process, and community standards.

All interactions are governed by our [Code of Conduct](CODE_OF_CONDUCT.md).

### **Community Resources**
- **Discussions**: [GitHub Discussions](https://github.com/InterCooperative/icn-core/discussions)
- **Issues**: [Issue Tracker](https://github.com/InterCooperative/icn-core/issues)
- **Security**: [Security Policy](SECURITY.md)
- **Website**: [intercooperative.network](https://intercooperative.network)

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.

## Development Progress & Roadmap

### **Completed Phases**
1. **Phase 1 – libp2p Integration**: Real networking with mesh peer discovery
2. **Phase 2A – Multi-Node CLI**: Bootstrap peer connection and federation
3. **Phase 2B – Cross-Node Mesh Jobs**: Distributed execution with cryptographic receipts
4. **Phase 3 – HTTP Gateway**: Complete REST API for all functionality
5. **Phase 4 – Federation Devnet**: Containerized multi-node testing environment

### **Current Phase 5: Production Readiness (Q1 2025)**
- ✅ Real networking and persistent storage
- ✅ Governance system with comprehensive voting
- ✅ Ed25519 cryptographic signing with memory protection
- ✅ API authentication and TLS support
- ✅ Federation management and synchronization
- ✅ Comprehensive monitoring and observability
- ✅ Multi-node federation testing at scale
- 🚧 Performance benchmarking and optimization

### **Upcoming Phases**
- **Phase 6**: Advanced governance, zero-knowledge proofs, CCL IDE support
- **Phase 7**: Interfederation protocol, cross-chain bridges, standards development
- **Phase 8**: AgoraNet platform, mobile apps, cooperative banking
- **Phase 9**: Machine learning integration, edge computing, supply chain management
- **Phase 10**: Post-capitalist coordination tools, systemic transformation

See [docs/ICN_FEATURE_OVERVIEW.md](docs/ICN_FEATURE_OVERVIEW.md) for the complete roadmap and [ICN_ROADMAP_2025.md](ICN_ROADMAP_2025.md) for detailed planning.

### **Running the Devnet**
```bash
# Launch containerized three-node federation
icn-devnet/launch_federation.sh

# Manual Docker build (requires 64 MiB stack)
export RUST_MIN_STACK=67108864
docker build -f icn-devnet/Dockerfile .
```

Future development and outstanding tasks are tracked on the [issue tracker](https://github.com/InterCooperative/icn-core/issues). Community feedback and contributions are always welcome!

---

**ICN is more than technology—it's a movement toward cooperative digital civilization. Join us in building infrastructure for a more just and sustainable world.**
