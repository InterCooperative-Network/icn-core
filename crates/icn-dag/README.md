# ICN DAG Crate

This crate implements or defines interfaces for content-addressed Directed Acyclic Graph (DAG) storage and manipulation, crucial for the InterCooperative Network (ICN) data model.

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-dag` crate is responsible for:

*   **DAG Primitives:** Defining core DAG structures (nodes, links, blocks) and operations (put, get, traverse).
*   **Content Addressing:** Ensuring data integrity and retrievability using cryptographic hashes (e.g., CIDs - Content Identifiers).
*   **Storage Abstraction:** Potentially providing interfaces for various backing stores (e.g., in-memory, disk-based, distributed).
*   **Serialization Formats:** Defining how DAGs are serialized and deserialized (e.g., CBOR, IPLD codecs).

This forms a foundational layer for data representation and exchange within the ICN.

## Public API Style

The API style prioritizes:

*   **Composability:** Allowing different DAG-based data structures to be built on top.
*   **Performance:** Efficient handling of DAG operations, especially for large graphs.
*   **Flexibility:** Supporting different codecs and storage backends where appropriate.
*   **Pluggable Persistence:** Includes in-memory, file-based, and optional `sled` backends via the `persist-sled` feature. When enabled, `SledDagStore` provides durable storage on disk.

## Async Feature

Enable the `async` feature to use asynchronous storage via `TokioFileDagStore`:

```toml
[dependencies]
icn-dag = { path = "../icn-dag", features = ["async"] }
```

```rust
use icn_dag::{AsyncStorageService, TokioFileDagStore};
use tokio::sync::Mutex;
use std::path::PathBuf;

let store = TokioFileDagStore::new(PathBuf::from("./dag")).unwrap();
let dag_store = Mutex::new(store); // implement AsyncStorageService
```

## Running Persistence Tests

Integration tests for each storage backend are gated by their corresponding
`persist-*` feature. Enable the feature when running `cargo test`:

```bash
# sled backend (enabled by default)
cargo test -p icn-dag --features persist-sled --test sled_backend

# SQLite backend
cargo test -p icn-dag --no-default-features --features persist-sqlite \
  --test sqlite_backend

# RocksDB backend
cargo test -p icn-dag --no-default-features --features persist-rocksdb \
  --test rocks_backend
```

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 