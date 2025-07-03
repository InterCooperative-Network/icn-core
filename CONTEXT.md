# ICN Core – Context Overview

## Purpose
`icn-core` is the authoritative Rust workspace for the InterCooperative Network (ICN). It provides deterministic libraries for the federated infrastructure stack that supports cooperative, post‑capitalist coordination.

## Architectural Principles
- **Strict Modularity:** The workspace is organized into discrete crates, each with a clear responsibility. Modules should minimize direct dependencies between domains.
- **Error-First Programming:** All crates return `Result<T, CommonError>` and avoid panics in library code. Error variants provide contextual messages for reliable handling.
- **Runtime-Based Execution:** The `icn-runtime` crate hosts WASM contracts and orchestrates mesh jobs through a host ABI. Deterministic execution ensures verifiable receipts.
- **Scoped Federation:** Nodes interact via identity-scoped federation protocols, using DIDs to define trust boundaries and access control.
- **Identity-Driven Design:** `icn-identity` manages DIDs, verifiable credentials, and signing utilities so every action is attributable.
- **DAG Ground Truth:** `icn-dag` anchors execution receipts and stores state in a content-addressed DAG, providing tamper-evident history.

## Crate Responsibilities
- **`icn-api`** – shared API traits and DTOs for node communication.
- **`icn-dag`** – DAG primitives, content addressing, and storage interfaces.
- **`icn-identity`** – decentralized identity management and cryptography.
- **`icn-mesh`** – job definition, bidding, and execution management in the mesh.
- **`icn-runtime`** – host runtime and WASM execution environment.
- **`icn-economics`** – mana accounting, ledger management, and incentives.
- **`icn-governance`** – proposal and voting engine.
- **`icn-network`** – P2P networking abstractions with libp2p support.
- **`icn-reputation`** – reputation scoring utilities.
- **`icn-cli`** & **`icn-node`** – command-line interface and node binary (HTTP server) built on the libraries above.

## Core System Patterns
- **DAG Anchoring:** All significant actions emit signed execution receipts that are stored in the DAG for auditability.
- **Scoped Operations:** Every operation is tied to a DID and governed by explicit policy. There are no unscoped actions or hardcoded IDs.
- **Decentralized Networking:** libp2p-based communication enables peer discovery and federation synchronization across nodes.
- **WASM-First Contracts:** Cooperative Contract Language (CCL) source compiles to WASM modules executed by the runtime, providing policy-driven behavior.

## Governance & Development Rules
- Use canonical data types from `icn-common` and API contracts from `icn-api`.
- Maintain deterministic logic; avoid wall-clock time or unseeded randomness.
- No hardcoded identifiers or manual cross-crate coupling.
- Contributions must include comprehensive tests and Rustdoc for public APIs.
- Follow the repository guidelines in `.cursor/rules` and run `just validate` before committing.

## Developer Experience Notes
- See `docs/ONBOARDING.md` for setup instructions and walkthroughs.
- The `justfile` provides common tasks (`just build`, `just test`, `just devnet`).
- The containerized devnet (`icn-devnet`) demonstrates a three-node federation with HTTP APIs for experimentation.
- Rich error messages, observability hooks, and CLI/HTTP tools enable rapid debugging and integration testing.

