# ICN API Crate

This crate provides the primary API endpoints for interacting with InterCooperative Network (ICN) nodes.

## Purpose

The `icn-api` crate defines the service interfaces, data structures for requests and responses, and potentially server/client implementations for ICN node APIs. This typically includes functionalities like querying node status, submitting transactions or messages, and interacting with specific ICN protocols (e.g., governance, economics).

## Public API Style

The API is designed to be accessible via common RPC mechanisms such as JSON-RPC or gRPC. The style emphasizes:

*   **Clarity:** Well-defined request and response types.
*   **Modularity:** Separating concerns for different aspects of the ICN functionality.
*   **Extensibility:** Allowing for new API endpoints and versions to be added as the ICN evolves.

Refer to the `lib.rs` documentation for specific API function signatures and data types.

### Example Usage

The crate exposes request/response structures for common operations:

```rust
use icn_api::transaction::{SubmitTransactionRequest, DataQueryRequest};
use icn_common::{Transaction, Cid};

let tx = Transaction { /* ... */ };
let submit = SubmitTransactionRequest { transaction: tx };
// Send `submit` to an ICN node via HTTP

let query = DataQueryRequest { cid: Cid::new_v1_dummy(0x71, 0x12, b"demo") };
// Send `query` to retrieve a `DagBlock`
```

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 