# Async APIs and Concurrency in ICN Core

The ICN Core workspace favors asynchronous code for any operation that touches the network or persistent storage. Tokio is the default runtime and every crate that performs I/O exposes `async` functions so callers can compose them with `async`/`await`.

## Key Points

- **Network and storage interactions are async.** Services in `icn-network`, the HTTP APIs in `icn-api`, and the CLI HTTP client all use async functions.
- **Storage interfaces are async.** `icn-dag` defines `AsyncStorageService` and async backends such as `TokioFileDagStore`. The older `StorageService` trait remains for limited synchronous environments.
- **Tokio runtime.** All nodes and services run on the Tokio runtime for cooperative scheduling.
- **Crates with async APIs:**
  - `icn-api` – async RPC helpers and network calls
  - `icn-cli` – async HTTP requests
  - `icn-dag` – async storage trait and backends when built with the `async` feature
  - `icn-network` – async peer‑to‑peer networking service
  - `icn-runtime` – async runtime context interacting with storage and networking
  - `icn-governance` – async federation sync helpers
  - `icn-node` – daemon uses Tokio for its main loop
- **Remaining sync code:**
  - `icn-dag` provides synchronous storage (`StorageService` and `FileDagStore`) when the `async` feature is disabled. Most other crates rely exclusively on async traits.

When adding new network or persistence features, prefer async traits and functions so that nodes remain responsive and can integrate with the Tokio runtime.
