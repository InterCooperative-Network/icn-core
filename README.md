# ICN Core v0.2 – Cooperative Infrastructure Engine (Beta)

`icn-core` is the reference implementation of the InterCooperative Network (ICN) protocol, written in Rust.
It provides the foundational crates for building ICN nodes, CLI tools, and other related infrastructure.

**ICN Mission**: Replace every major function of the state and corporation with programmable, federated, democratic infrastructure—built for actual human needs and scalable solidarity.

## 📚 Documentation

### Core Documentation
- [CONTEXT.md](CONTEXT.md) - Core philosophy and architectural principles
- [docs/ICN_FEATURE_OVERVIEW.md](docs/ICN_FEATURE_OVERVIEW.md) - Complete feature set (current and planned)
- [docs/ASYNC_OVERVIEW.md](docs/ASYNC_OVERVIEW.md) - Async APIs and concurrency model

### Comprehensive Guides
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - System architecture, crate relationships, and data flow
- [docs/DEVELOPER_GUIDE.md](docs/DEVELOPER_GUIDE.md) - Complete developer setup, workflows, and best practices
- [docs/SYSTEM_COMPLETENESS_ROADMAP.md](docs/SYSTEM_COMPLETENESS_ROADMAP.md) - Development roadmap and missing features
- [docs/GLOSSARY.md](docs/GLOSSARY.md) - Comprehensive glossary of ICN terms and concepts

### Quick Start
- [docs/beginner/README.md](docs/beginner/README.md) - Quickest setup steps
- [docs/ONBOARDING.md](docs/ONBOARDING.md) - Comprehensive walkthrough
- [icn-devnet/README.md](icn-devnet/README.md) - Multi-node federation testing

### API Documentation
- [docs/API.md](docs/API.md) - HTTP API endpoints and authentication
- [docs/troubleshooting.md](docs/troubleshooting.md) - Common issues and solutions

## Overview

The InterCooperative Network is a comprehensive platform for building federated, cooperative digital infrastructure. This repository contains the core building blocks for autonomous federated systems that enable cooperative coordination without relying on traditional state or corporate structures.

**Key Differentiators:**
- **100% Rust**: Memory-safe, performant foundation
- **Cooperative Virtual Machine (CoVM)**: WASM-first deterministic execution
- **DAG Ledger**: Content-addressed storage without blockchain complexity  
- **Scoped Identity**: DIDs with built-in federation support
- **Anti-Capitalist Economics**: Purpose-bound tokens that cannot be speculated on
- **Governance as Code**: Programmable bylaws via Cooperative Contract Language (CCL)

## 🤝 Value for Cooperatives & Communities

ICN is purpose-built as digital infrastructure for the solidarity economy. It provides cooperatives, mutual aid networks, and communities with technology designed around their actual needs:

### **🏛️ Democratic Governance Infrastructure**
- **Programmable Bylaws**: Encode cooperative bylaws as executable CCL code for consistent, transparent governance
- **Democratic Decision-Making**: Proposal lifecycle with configurable quorum, delegation, and consensus mechanisms
- **Member Management**: Built-in systems for adding/removing members with role-based permissions
- **Audit Transparency**: All governance decisions cryptographically signed and stored immutably

### **💰 Anti-Capitalist Economics**
- **Non-Speculative Resource System**: Mana tokens regenerate over time and cannot be abstracted or financialized
- **Purpose-Bound Value**: Tokens scoped to specific capabilities prevent extraction and speculation
- **Reputation-Based Allocation**: Higher contribution leads to lower costs and increased resource access
- **Cooperative Banking Ready**: Foundation for mutual credit, time banking, and local currencies

### **🌐 Federated Cooperation**
- **Local Autonomy**: Each cooperative maintains full governance independence via CCL
- **Federation Benefits**: Resource sharing and coordination across cooperatives without hierarchy
- **Scoped Identity**: DID-based identity with verifiable credentials for cross-cooperative trust
- **Network Solidarity**: Mutual aid and resource pooling at scale

### **⚡ Shared Resource Computing**
- **Democratic Resource Allocation**: Reputation-based job selection rewards reliable contributors
- **Economic Enforcement**: Mana-based payment prevents resource abuse while ensuring access
- **Transparent Execution**: Cryptographic receipts provide verifiable proof of work completion
- **Community Infrastructure**: Cooperatives can pool computational resources instead of relying on cloud monopolies

### **🚀 Planned Cooperative Infrastructure**
The ICN roadmap includes comprehensive cooperative-specific features:
- **Cooperative Banking**: Mutual credit systems, time banking, local currencies, democratic loans
- **Mutual Aid Networks**: Emergency response coordination, resource sharing, community support
- **Supply Chain Cooperation**: Product sourcing, bulk purchasing, quality assurance
- **Worker Cooperative Tools**: Profit sharing, democratic workplace coordination, labor scheduling
- **Consumer Cooperative Features**: Patronage dividends, member benefits, purchasing coordination
- **Housing Justice**: Maintenance coordination, occupancy planning, eviction defense
- **Educational Cooperation**: Skill sharing, learning resources, knowledge management
- **Climate Action**: Carbon credit trading, renewable energy sharing, impact tracking
- **Transformative Justice**: Mediation workflows, restorative justice, community healing

See the complete [Cooperative Infrastructure overview](docs/ICN_FEATURE_OVERVIEW.md#11-🤝-cooperative-infrastructure-new) for detailed feature descriptions.

## Current Project Status (Production-Ready Foundation)

The project has achieved a significant milestone with a production-ready foundation featuring:

### **✅ Core Infrastructure Complete**
- **Modular Crate Structure**: Well-defined crates for all major domains
- **Real Protocol Data Models**: DIDs, CIDs, DagBlocks, governance primitives
- **Multiple Storage Backends**: SQLite, RocksDB, Sled, File-based persistence
- **Event Sourcing**: Append-only event log via `icn-eventstore` for replayable state
- **API Layer**: Comprehensive REST endpoints with authentication and TLS
- **P2P Mesh Networking**: real libp2p networking stack with Kademlia peer discovery
- **PostgresDagStore**: scalable PostgreSQL backend for DAG storage
- **WASM ResourceLimiter**: caps CPU and memory usage for mesh jobs
- **Production Security**: `Ed25519Signer` with memory protection, encrypted key files and optional HSM integration
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

### **🎯 Next Development Phase**

The project is ready for the next phase of development. See [docs/SYSTEM_COMPLETENESS_ROADMAP.md](docs/SYSTEM_COMPLETENESS_ROADMAP.md) for a comprehensive analysis of remaining work, including:

#### Critical Path (Must Have)
1. **Zero-Knowledge Credential Disclosure** - Privacy-preserving cooperative membership
2. **Scoped Token Economy** - Core cooperative resource sharing
3. **Federation Sync Protocol Hardening** - Multi-node federation reliability

#### High Value (Should Have)
4. **Dynamic Governance Policies** - Truly programmable cooperative governance
5. **Web UI / Wallet / Explorer Suite** - Critical for user adoption
6. **Federation Bootstrap CLI/UX** - Cooperative formation tools

The roadmap includes detailed action items, success metrics, and implementation priorities for transforming ICN from a solid foundation into a complete cooperative digital infrastructure platform.

### Rust Toolchain

This repository uses the stable Rust toolchain via `rust-toolchain.toml`.
Install it with:

```bash
rustup toolchain install stable
rustup override set stable
```

## Getting Started

Start with `docs/beginner/README.md` for the quickest setup steps. Then see `docs/ONBOARDING.md` for a comprehensive walkthrough. The latest API documentation is available at [https://intercooperative.network/docs](https://intercooperative.network/docs).

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
  --governance-db-path ./icn_data/gov.sqlite \
  --node-did-path ./icn_data/node1.did \
  --node-private-key-path ./icn_data/node1.key \
  --http-listen-addr 127.0.0.1:7845 \
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
./target/debug/icn-cli compile-ccl examples/simple.ccl
./target/debug/icn-cli submit-job examples/echo-job.json
./target/debug/icn-cli job-status <JOB_CID>
./target/debug/icn-cli governance propose "Increase mesh job timeout to 300 seconds"
./target/debug/icn-cli network discover-peers

# Federation management commands
./target/debug/icn-cli federation join <PEER_ID>
./target/debug/icn-cli federation status
./target/debug/icn-cli federation leave <PEER_ID>
./target/debug/icn-cli federation list-peers
```

## Production vs Test Modes

`icn-node` runs with production services by default. Use `--test-mode` or set
`ICN_TEST_MODE=true` to launch with stub networking and in-memory storage.
Persistent DAG paths and the signing key are configured via CLI or environment
variables:

```bash
ICN_STORAGE_PATH=./icn_data/dag.sqlite \
ICN_NODE_DID_PATH=./icn_data/node.did \
ICN_NODE_PRIVATE_KEY_PATH=./icn_data/node.key \
icn-node --storage-backend sqlite
```

## 🌐 Quick Start: Devnet Testing

For the fastest way to test ICN features, use the containerized devnet that launches a 3-node federation:

```bash
# Launch complete devnet (from project root)
just devnet

# Or manually:
cd icn-devnet && ./launch_federation.sh
```

This starts three nodes with automatic peer discovery and mana initialization. Each node gets **1000 mana** on startup.

### Test Job Submission

```bash
# Submit a mesh job to the devnet
curl -X POST http://localhost:5001/mesh/submit \
  -H "Content-Type: application/json" \
  -H "X-API-Key: devnet-a-key" \
  -d '{
    "manifest_cid": "bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354",
    "spec_bytes": "BASE64_SPEC",
    "spec_json": null,
    "cost_mana": 50
  }'

# Expected response:
# {"job_id":"bafkrfz2acgrvhdag6q2rs7h5buh2i6omqqhffnrvatziwrlrnx3elqyp"}

# Check job status
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/mesh/jobs

# Access the different nodes:
# Node A (bootstrap): http://localhost:5001
# Node B (worker): http://localhost:5002  
# Node C (worker): http://localhost:5003
```

**Important Notes:**
- Use `X-API-Key` header (not `Authorization: Bearer`)
- Jobs may show "failed - no bids" in single-node testing (expected behavior)
- Mana is automatically refunded when jobs fail
- See [icn-devnet/README.md](icn-devnet/README.md) for complete documentation

### Recent Fix: Mana Initialization ✅

A critical bug was recently fixed where nodes failed to initialize mana accounts, causing `"Account not found"` errors during job submission. This issue is now resolved - you should see `✅ Node initialized with 1000 mana` in startup logs.

If you encounter mana-related errors:
```bash
# Check node startup logs for mana initialization
docker-compose logs icn-node-a | grep -i "mana\|initialized"

# Should show: "✅ Node initialized with 1000 mana"
```

### Architecture Overview

The following text diagrams from [docs/ONBOARDING.md](docs/ONBOARDING.md) illustrate how core components interact.

#### Block Storage

```text
Node Runtime -> DagStorageService: put(block)
DagStorageService -> StorageBackend: persist block
StorageBackend --> DagStorageService: CID
DagStorageService --> Node Runtime: CID
```

#### Peer Messaging

```text
Node A -> NetworkService: send_message(Node B, msg)
NetworkService -> Node B: deliver msg
Node B -> NetworkService: optional response
NetworkService -> Node A: response
```

### Justfile Commands

Common development tasks are defined in a `justfile` at the repository root. Install [just](https://github.com/casey/just) and run:

```bash
just format   # check formatting
just lint     # run clippy
just test     # execute all tests
just build    # build all crates
just devnet   # launch the containerized federation devnet
just health-check # run federation health checks
just status   # query node status
just logs     # show recent devnet logs
just metrics  # fetch node metrics
just docs     # build documentation
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

To spin up the full Prometheus and Grafana stack locally run:

```bash
docker compose -f docker-compose-monitoring.yml up -d
```

Grafana will be available at <http://localhost:3000> (login `admin`/`admin`) and Prometheus at <http://localhost:9090>.

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
- **`icn-common`** – Shared types and error utilities used across crates
- **`icn-runtime`** – Host runtime with WASM execution and job orchestration
- **`icn-api`** – HTTP/gRPC API definitions and client helpers

### **Identity & Security**
- **`icn-identity`** – DID management and credential verification
- **`icn-dag`** – Content-addressed storage and receipt anchoring

### **Governance & Economics**
- **`icn-governance`** – Proposals, voting, and policy execution
- **`icn-economics`** – Mana accounting and economic policies
- **`icn-reputation`** – Contribution tracking and trust metrics

### **Networking & Computation**
- **`icn-network`** – libp2p peer discovery and federation sync
- **`icn-mesh`** – Distributed job scheduling and execution

### **Language & Protocol**
- **[`icn-ccl`](icn-ccl/README.md)** – Cooperative Contract Language compiler
- **`icn-protocol`** – Core message formats and serialization

### **User Interfaces**
- **`icn-cli`** – Command-line interface for operators
- **`icn-node`** – Main daemon binary exposing the HTTP API

More detailed information can be found in the `README.md` file within each crate's directory.

## Further Reading

### **Architecture & Design**
* [Beginner Quickstart](docs/beginner/README.md) – minimal setup steps
* [Complete Feature Overview](docs/ICN_FEATURE_OVERVIEW.md) – comprehensive feature breakdown
* [Context & Philosophy](CONTEXT.md) – core principles and architectural vision
* [Development Workflow](docs/ONBOARDING.md) – full onboarding guide
* [Async Overview](docs/ASYNC_OVERVIEW.md) – networking and storage concurrency
* [Multi-Node Setup](MULTI_NODE_GUIDE.md) – federation deployment guide

### **Governance & Economics**
* [RFC Index](icn-docs/rfcs/README.md) – design proposals and specifications
* [RFC 0010: ICN Governance Core](icn-docs/rfcs/0010-governance-core.md) – governance framework
* [Mana Policies](docs/mana_policies.md) – economic policy examples
* [Resource Tokens](docs/resource_tokens.md) – token classes and mana integration
* [CCL Language Reference](docs/CCL_LANGUAGE_REFERENCE.md) – full syntax guide
* [CCL Examples](icn-ccl/examples/) – governance contract templates
* [Governance Onboarding Guide](docs/governance_onboarding.md) – using templates
* [Contract Creation Guide](docs/howto-create-contract.md) – compile and submit CCL jobs

### **Development Resources**
* Crate documentation: [icn-common](crates/icn-common/README.md), [icn-dag](crates/icn-dag/README.md), [icn-identity](crates/icn-identity/README.md), [icn-mesh](crates/icn-mesh/README.md), [icn-governance](crates/icn-governance/README.md), [icn-runtime](crates/icn-runtime/README.md), [icn-network](crates/icn-network/README.md), [icn-templates](crates/icn-templates/README.md)
* [Rust API Documentation](https://intercooperative-network.github.io/icn-core/) – automatically built by [docs.yml](.github/workflows/docs.yml)
* Build docs locally with `cargo doc --workspace --no-deps` (or `just docs`) and open them from `target/doc`
* [API Documentation](docs/API.md) – HTTP endpoints and programmatic interfaces
* [Deployment Guide](docs/deployment-guide.md) – production deployment instructions (see [circuit breaker & retry settings](docs/deployment-guide.md#circuit-breaker-and-retry))
* [Troubleshooting Guide](docs/TROUBLESHOOTING.md) – common issues and solutions
* [Custom Circuit Development](docs/zk_circuit_development.md) – implement and profile new zero-knowledge circuits

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
- ✅ **Critical Mana Initialization Fix**: Resolved account not found errors
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

# Or use just command
just devnet

# Manual Docker build (requires 64 MiB stack)
export RUST_MIN_STACK=67108864
docker build -f icn-devnet/Dockerfile .
```

#### 10-Node Devnet
For larger scale testing you can spin up a ten node federation with load generation:

```bash
scripts/run_10node_devnet.sh --start-only   # launch containers
scripts/run_10node_devnet.sh --jobs-only    # submit sample jobs
scripts/run_10node_devnet.sh --stop-only    # tear down
```

Future development and outstanding tasks are tracked on the [issue tracker](https://github.com/InterCooperative/icn-core/issues). Community feedback and contributions are always welcome!

---

**ICN is more than technology—it's a movement toward cooperative digital civilization. Join us in building infrastructure for a more just and sustainable world.**
