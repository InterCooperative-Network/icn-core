# ICN Core

`icn-core` is the reference implementation of the InterCooperative Network (ICN) protocol, written in Rust.
It provides the foundational crates for building ICN nodes, CLI tools, and other related infrastructure.

## Overview

The InterCooperative Network is envisioned as a decentralized network fostering collaboration and resource sharing. This repository contains the core building blocks for such a network.

## Current Project Status (MVP - Functional Protocol Stack)

The project has achieved a significant milestone, delivering an MVP with a functional protocol stack powered by a real libp2p mesh. Key features include:

*   **Modular Crate Structure:** Well-defined crates for common types (`icn-common`), API definitions (`icn-api`), DAG L1 logic (`icn-dag`), an initial identity module (`icn-identity`), networking abstractions (`icn-network`), a node runner (`icn-node`), a CLI (`icn-cli`), and the Cooperative Contract Language compiler (`icn-ccl`, located at the repository root outside `crates/`).
*   **Real Protocol Data Models:** Core data types like DIDs, CIDs, DagBlocks, Transactions, and NodeStatus are defined in `icn-common` and utilize `serde` for serialization.
*   **In-Memory DAG Store:** `icn-dag` provides an in-memory DAG store implementing the `StorageService` trait. Interact with DAG data through a chosen `StorageService` implementation.
*   **API Layer:** `icn-api` exposes functions for node interaction (info, status) and DAG operations (submit, retrieve blocks).
*   **Node & CLI Prototypes:**
    *   `icn-node`: A binary that demonstrates the integration of API, DAG, and networking components. When compiled with `with-libp2p` and started with `--enable-p2p`, it joins the libp2p mesh, discovers peers, and exchanges messages via gossipsub.
    *   `icn-cli`: A command-line tool to interact with the node via the API for DAG and governance operations.
*   **P2P Mesh Networking:** `icn-network` integrates libp2p with Kademlia peer discovery and Gossipsub for reliable message propagation.
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

Refer to `docs/ONBOARDING.md` for detailed instructions on prerequisites, setup, building, testing, and running the components. The latest API documentation is available at [https://intercooperative.network/docs/icn-core](https://intercooperative.network/docs/icn-core).

### Quick CLI Examples:

```bash
# Build with libp2p support and the default `sled` persistence backend
cargo build --features with-libp2p

# Build using the SQLite backend
cargo build --no-default-features --features "with-libp2p persist-sqlite"

# Start a node with persistent storage and P2P enabled
./target/debug/icn-node \
  --enable-p2p \
  --p2p-listen-addr /ip4/0.0.0.0/tcp/4001 \
  --storage-backend sqlite \
  --storage-path ./icn_data/node1.sqlite

# In a second terminal start another node connecting to the first
./target/debug/icn-node \
  --enable-p2p \
  --p2p-listen-addr /ip4/0.0.0.0/tcp/4002 \
  --bootstrap-peers /ip4/127.0.0.1/tcp/4001/p2p/<PEER_ID> \
  --storage-backend sqlite \
  --storage-path ./icn_data/node2.sqlite

# Interact with a node via the CLI
./target/debug/icn-cli info
./target/debug/icn-cli status
./target/debug/icn-cli federation join peer1
./target/debug/icn-cli federation status
```

### Justfile Commands

Common development tasks are defined in a `justfile` at the repository root. Install [just](https://github.com/casey/just) and run:

```bash
just format   # check formatting
just lint     # run clippy
just test     # execute all tests
just build    # build all crates
just devnet   # launch the containerized federation devnet
```


### Enabling Peer Discovery and Persistent Storage

1. **Compile with libp2p support** using `cargo build --features with-libp2p`.
   For the SQLite backend add `--features persist-sqlite` (optionally with
   `--no-default-features`).
2. Start each node with `--enable-p2p` and a unique `--p2p-listen-addr`.
3. Provide known peers via `--bootstrap-peers` to join an existing mesh.
4. Use `--storage-backend sqlite` or `file` with a dedicated `--storage-path` to
   persist DAG blocks and governance state across restarts.

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
*   [`icn-ccl`](icn-ccl/README.md): Implements the Cooperative Contract Language compiler, producing WASM modules for the runtime. 
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

## Further Reading

* [RFC Index](icn-docs/rfcs/README.md) – notably [RFC 0010: ICN Governance & Deliberation Core](icn-docs/rfcs/0010-governance-core.md)
* Crate documentation:
  * [icn-common](crates/icn-common/README.md)
  * [icn-dag](crates/icn-dag/README.md)
  * [icn-identity](crates/icn-identity/README.md)
  * [icn-mesh](crates/icn-mesh/README.md)
  * [icn-governance](crates/icn-governance/README.md)
  * [icn-runtime](crates/icn-runtime/README.md)
  * [icn-network](crates/icn-network/README.md)

## Contribution Guidelines

We welcome contributions to the ICN Core project! Please see our [Contributing Guidelines](CONTRIBUTING.md) for more information on how to get started, our coding conventions, and the pull request process.

All interactions within this project are governed by our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.

## Completed Phases & Next Steps

Development has progressed through several major phases:

1. **Phase&nbsp;1 – libp2p Integration**: real networking replaced the early stubs and `RuntimeContext` gained methods for joining the mesh.
2. **Phase&nbsp;2A – Multi‑Node CLI**: nodes can be launched with libp2p enabled and discovered via bootstrap peers.
3. **Phase&nbsp;2B – Cross‑Node Mesh Jobs**: distributed job execution is verified with cryptographically signed receipts.
4. **Phase&nbsp;3 – HTTP Gateway**: all runtime functionality is accessible over REST endpoints.
5. **Phase&nbsp;4 – Federation Devnet**: containerized devnet demonstrating a three‑node federation.

Future planning and outstanding tasks are tracked on the
[issue tracker](https://github.com/InterCooperative/icn-core/issues).
Community feedback and contributions are always welcome!
