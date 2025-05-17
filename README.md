# ICN Core

[![Rust CI](https://github.com/InterCooperative-Network/icn-core/actions/workflows/ci.yml/badge.svg)](https://github.com/InterCooperative-Network/icn-core/actions/workflows/ci.yml)

A monorepo of core Rust crates for the InterCooperative Network (ICN).

## Getting Started

```bash
# Clone
git clone git@github.com:InterCooperative-Network/icn-core.git
cd icn-core

# Build & test
cargo build
cargo test

```

## Crate Descriptions

This workspace is organized into several crates, each with a specific focus:

*   `icn-api`: Provides the primary API endpoints for interacting with ICN nodes, likely via JSON-RPC or gRPC.
*   `icn-cli`: A command-line interface for users and administrators to manage and interact with ICN nodes and the network.
*   `icn-common`: Contains common data structures, types, utilities, and error definitions shared across multiple ICN crates.
*   `icn-dag`: Implements or defines interfaces for content-addressed Directed Acyclic Graph (DAG) storage and manipulation, crucial for ICN's data model.
*   `icn-economics`: Handles the economic protocols of the ICN, including token models (e.g., Mana), ledger management, and transaction logic.
*   `icn-governance`: Defines the mechanisms for network governance, such as proposal systems, voting procedures, and quorum logic.
*   `icn-identity`: Manages decentralized identities (DIDs), verifiable credentials (VCs), and cryptographic operations for users and nodes.
*   `icn-mesh`: Focuses on job orchestration, scheduling, and execution within the ICN mesh network.
*   `icn-network`: Manages peer-to-peer networking aspects, likely using libp2p, including transport protocols and federation synchronization.
*   `icn-node`: The main binary for running a long-lived ICN daemon process.
*   `icn-protocol`: Defines core message formats, communication protocols, and potentially helpers for a domain-specific language like CCL (Cooperative Contract Language).
*   `icn-runtime`: Provides the execution environment for ICN logic, possibly including WebAssembly (WASM) runtimes and host interaction capabilities.

More detailed information can be found in the `README.md` file within each crate's directory.

## Contribution Guidelines

We welcome contributions to the ICN Core project! Please see our [Contributing Guidelines](CONTRIBUTING.md) for more information on how to get started, our coding conventions, and the pull request process.

All interactions within this project are governed by our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.

## Next Milestones

The following are high-level milestones for the development of ICN Core. These are subject to change and community input.

1.  **Foundation & Basic APIs (Q1-Q2 202X)**
    *   Solidify `icn-common` with core data types (DIDs, CIDs, robust errors).
    *   Implement initial JSON-RPC/gRPC skeletons in `icn-api` for basic node interaction (e.g., status, identity query).
    *   Develop basic DID management in `icn-identity` (generation, simple resolution).
    *   Set up basic node persistence for configuration and identity keys in `icn-node`.

2.  **Networking & Basic DAG (Q2-Q3 202X)**
    *   Wire up basic libp2p stack in `icn-network` (peer discovery, basic pub/sub or request-response).
    *   Implement initial in-memory `icn-dag` store with put/get operations for CIDs.
    *   `icn-node` can connect to a bootstrap peer and exchange basic messages.
    *   `icn-cli` can make basic API calls to a running node over the network (local for now).

3.  **Core Protocols - Governance & Economics (Q3-Q4 202X)**
    *   Define and implement basic proposal submission and voting logic in `icn-governance`.
    *   Define basic token structures and transfer logic in `icn-economics` (in-memory ledger initially).
    *   Integrate governance and economics actions into `icn-api` and `icn-node`.

4.  **Runtime & Mesh (Q4 202X - Q1 202Y)**
    *   Integrate a WASM runtime into `icn-runtime`.
    *   Define initial host functions for WASM contracts to interact with node services (DAG, identity, economics).
    *   Basic job definition and local execution PoC in `icn-mesh`.

5.  **Federation & Advanced Features (202Y Onwards)**
    *   Implement initial federation sync protocols in `icn-network` and `icn-protocol`.
    *   Advanced DAG features (e.g., selectors, sharding considerations).
    *   More sophisticated governance and economic models.
    *   Full `icn-mesh` job scheduling and distributed execution.

This roadmap will be refined into more detailed issues on GitHub. Community feedback is highly encouraged!
