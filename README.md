# ICN Core

[![Rust CI](https://github.com/USERNAME/icn-core/actions/workflows/ci.yml/badge.svg)](https://github.com/USERNAME/icn-core/actions/workflows/ci.yml)

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
