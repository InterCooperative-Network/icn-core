# ICN Core

`icn-core` is the reference implementation of the InterCooperative Network (ICN) protocol, written in Rust.
It provides the foundational crates for building ICN nodes, CLI tools, and other related infrastructure.

## Overview

The InterCooperative Network is envisioned as a decentralized network fostering collaboration and resource sharing. This repository contains the core building blocks for such a network.

## Current Project Status (MVP - Functional Protocol Stack)

The project has achieved a significant milestone, delivering an MVP with a functional, albeit stubbed, protocol stack. Key features include:

*   **Modular Crate Structure:** Well-defined crates for common types (`icn-common`), API definitions (`icn-api`), DAG L1 logic (`icn-dag`), identity placeholders (`icn-identity`), networking abstractions (`icn-network`), a node runner (`icn-node`), a CLI (`icn-cli`), and the Cooperative Contract Language compiler (`icn-ccl`, located at the repository root outside `crates/`).
*   **Real Protocol Data Models:** Core data types like DIDs, CIDs, DagBlocks, Transactions, and NodeStatus are defined in `icn-common` and utilize `serde` for serialization.
*   **In-Memory DAG Store:** `icn-dag` provides a basic in-memory L1 DAG block store (`put_block`, `get_block`).
*   **API Layer:** `icn-api` exposes functions for node interaction (info, status) and DAG operations (submit, retrieve blocks).
*   **Node & CLI Prototypes:**
    *   `icn-node`: A binary that demonstrates the integration of API, DAG, and network components. It shows how to get node status, submit/retrieve DAG blocks, and perform stubbed network operations like peer discovery and message broadcasting/sending.
    *   `icn-cli`: A command-line tool to interact with the node via the API. It supports commands for node info/status, DAG put/get, and now includes **stubbed network operations** (`network discover-peers`, `network send-message`).
*   **Stubbed Networking Layer:** `icn-network` defines a `NetworkService` trait, `NetworkMessage` enum (now serializable), and a `StubNetworkService` for simulated P2P interactions.
*   **Refined Error Handling:** Comprehensive error handling is implemented across all layers. Functions return `Result<T, CommonError>`, using specific error variants defined in `icn-common`. The CLI and Node applications now handle these errors more gracefully, providing better user feedback and exiting with appropriate status codes.
*   **Repository Hygiene:** Includes `LICENSE` (Apache 2.0), `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, `.editorconfig`, `rust-toolchain.toml`, issue templates, and a `CHANGELOG.md`.

*   **CI & Dependabot:** Basic CI pipeline (`ci.yml`) for formatting, linting, testing, and docs. Dependabot is set up for Cargo dependency updates.
*   **Basic Documentation:** READMEs for each crate, module-level documentation, and an initial `docs/ONBOARDING.md`.

### Rust Toolchain

This repository is pinned to the nightly Rust toolchain via `rust-toolchain.toml`.
Install it with:

```bash
rustup toolchain install nightly
rustup override set nightly
```

## Getting Started

Refer to `docs/ONBOARDING.md` for detailed instructions on prerequisites, setup, building, testing, and running the components.

### Quick CLI Examples:

```bash
# Build all crates (from the icn-core workspace root)
cargo build

# Run the CLI (examples)
./target/debug/icn-cli info
./target/debug/icn-cli status
./target/debug/icn-cli status offline # Test error path

# DAG operations (requires valid JSON for DagBlock and Cid)
./target/debug/icn-cli dag put '{"cid":{"version":1,"codec":113,"hash_alg":18,"hash_bytes":[...]},"data":[...],"links":[]}'
./target/debug/icn-cli dag get '{"version":1,"codec":113,"hash_alg":18,"hash_bytes":[...]}'

# Network operations (stubbed)
./target/debug/icn-cli network discover-peers
./target/debug/icn-cli network send-message mock_peer_1 '{"RequestBlock":{"version":1,"codec":112,"hash_alg":18,"hash_bytes":[100,97,116,97]}}'
```

## Error Handling Philosophy

This project prioritizes robust and clear error handling to improve developer experience and system reliability:

1.  **No Panics in Libraries:** Library crates (`icn-common`, `icn-api`, `icn-dag`, `icn-network`, etc.) should avoid `panic!` for recoverable errors. Instead, they return `Result<T, CommonError>`.
2.  **Specific Error Variants:** The `icn_common::CommonError` enum defines a comprehensive set of error variants (e.g., `StorageError`, `BlockNotFound`, `NetworkConnectionError`, `PeerNotFound`, `SerializationError`, `InvalidInputError`). This allows calling code to match on specific error types and handle them appropriately.
3.  **Clear Error Messages:** Error variants include a `String` payload to provide contextual information about the error.
4.  **Graceful Handling in Binaries:** Executables (`icn-node`, `icn-cli`) catch these `Result`s, print user-friendly error messages (typically to `stderr`), and exit with non-zero status codes when an operation fails.
5.  **Propagation:** Errors are propagated up the call stack, often wrapped with additional context at each layer (e.g., API layer might wrap a `StorageError` from `icn-dag`).

This approach ensures that errors are not silently ignored and that developers using or contributing to the codebase can understand and react to issues effectively.

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
