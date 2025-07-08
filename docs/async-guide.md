# Async Network and Storage Overview

Most operations that involve I/O in `icn-core` use Rust's `async`/`await` model. This applies to networking, the HTTP API, and all persistent storage layers. A Tokio runtime is used by the binaries and tests.

## Async Storage Interfaces

All storage traits expose asynchronous methods. The `icn-dag` crate defines `AsyncStorageService` and provides `TokioFileDagStore` when compiled with the `async` feature. `icn-runtime` and `icn-node` depend on these traits for DAG persistence and mana ledgers.

## Crates with Async APIs

* **`icn-network`** – peer discovery, messaging, and libp2p services built on Tokio.
* **`icn-api`** – HTTP client utilities implemented with `reqwest` and async handlers.
* **`icn-dag`** – asynchronous storage backends via the `async` feature (`TokioFileDagStore`).
* **`icn-runtime`** – runtime context and host calls that operate on async storage and network traits.
* **`icn-governance`** – `request_federation_sync` helper uses async network operations.
* **`icn-cli`** and **`icn-node`** – binaries execute within a Tokio runtime and interact with async APIs.

Some older synchronous helpers remain for file-based storage (`FileDagStore`) or simple test utilities, but new code should prefer the async interfaces.
