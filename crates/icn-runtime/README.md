# ICN Runtime Crate

This crate provides the execution environment for InterCooperative Network (ICN) logic, possibly including WebAssembly (WASM) runtimes and host interaction capabilities.

## Purpose

The `icn-runtime` crate is responsible for:

*   **Execution Environment:** Defining and managing the environment where ICN's core logic or user-defined contracts/scripts run.
*   **WASM Runtime (if applicable):** If ICN uses WebAssembly for smart contracts or extensible logic, this crate would host and manage the WASM execution engine (e.g., Wasmer, Wasmtime).
*   **Host Functions:** Providing a set of functions (host calls) that WASM modules or other sandboxed code can use to interact with the ICN node's capabilities (e.g., accessing storage, sending network messages, interacting with ledgers).
*   **Sandboxing and Security:** Ensuring that executed code is properly isolated and cannot compromise the host node or the network.
*   **Metering and Resource Limits:** Potentially implementing mechanisms to measure and limit the computational resources (e.g., gas) consumed by executed code.

This crate is key to enabling safe and extensible functionality within ICN nodes.

## WASM Host Interaction

User code compiled to WebAssembly communicates with the runtime through host
functions exposed under the `icn` module. Each host call has a numeric constant
in `src/abi.rs` and is linked to the `RuntimeContext` when the module is
executed. A module imports the functions like so:

```wat
(import "icn" "host_submit_mesh_job" (func $submit (param i32 i32) (result i64)))
```

The guest allocates memory for the job JSON, then calls `$submit` with the
pointer and length of that buffer. The runtime reads the bytes from the module's
linear memory, deserializes the job, and returns the created job ID as a 64‑bit
value. Other host calls follow the same pointer/length convention for string
parameters.

Binding helpers can reduce boilerplate when exposing host calls. One possible
macro looks like this:

```rust
macro_rules! hostcall_str {
    ($linker:expr, $name:literal, $func:ident) => {
        $linker.func_wrap(
            "icn",
            $name,
            move |mut caller: wasmtime::Caller<'_, std::sync::Arc<RuntimeContext>>, ptr: u32, len: u32| {
                let memory = caller
                    .get_export("memory")
                    .unwrap()
                    .into_memory()
                    .unwrap();
                let mut buf = vec![0u8; len as usize];
                memory.read(&caller, ptr as usize, &mut buf).unwrap();
                let arg = String::from_utf8(buf).unwrap();
                let handle = tokio::runtime::Handle::current();
                handle.block_on($func(caller.data(), &arg)).unwrap();
            },
        )
    };
}
```

This pattern ensures memory is safely copied from the guest and allows host
functions like `host_submit_mesh_job` to remain concise.

## Public API Style

The API style emphasizes:

*   **Security:** Robust sandboxing and controlled access to host capabilities.
*   **Performance:** Efficient execution of runtime logic, especially for WASM.
*   **Modularity:** Clear separation between the runtime environment and the code being executed.
*   **Well-Defined Interface:** A stable and clear set of host functions for guest code.

### Governance

The runtime exposes host calls for managing on-chain proposals. Voting can be
closed via `host_close_governance_proposal_voting`, returning the final
`ProposalStatus` as a string. Accepted proposals may then be executed with
`host_execute_governance_proposal`, which updates the stored proposal and member
set.

## Mana Regeneration

`RuntimeContext` can automatically replenish mana. Use
`spawn_mana_regenerator` to start a background task that credits every
account with a fixed amount on a configurable interval.

## DAG Storage

`RuntimeContext` selects a storage backend for receipts and other DAG data. When
compiled with the `async` feature, use `TokioFileDagStore`:

```rust
use icn_runtime::context::{RuntimeContext, StubMeshNetworkService, StubSigner};
use icn_common::Did;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "async")]
let dag_store = Arc::new(Mutex::new(icn_dag::TokioFileDagStore::new("./dag".into()).unwrap()));
#[cfg(not(feature = "async"))]
let dag_store = Arc::new(Mutex::new(icn_dag::FileDagStore::new("./dag".into()).unwrap()));

let ctx = RuntimeContext::new(
    Did::new("key", "node"),
    Arc::new(StubMeshNetworkService::new()),
    Arc::new(StubSigner::new()),
    Arc::new(icn_identity::KeyDidResolver),
    dag_store,
);
```

## WASM Execution Limits

`WasmExecutor` instances can be configured with a maximum linear memory size and
a fuel allowance. Fuel metering is enabled via Wasmtime and each instruction
consumes fuel. When a module exhausts its fuel or attempts to grow memory beyond
the configured limit, execution is aborted.

## Error Types

`CommonError` is used for all runtime failures.
* `MissingOrInvalidReceipt` – receipt missing or failed validation.
* `UnknownJob` – referenced job ID does not exist.
* `ExecutionTimeout` – executor failed to produce a receipt in time.
* `ProcessingFailure` – job failed during runtime processing.
* `Serialization` – JSON or binary serialization failure.
* `InvalidSpec` – job specification failed validation.
* `PermissionDenied` – caller lacked permission for the operation.
* `InvalidJobState` – operation not allowed in the job's current state.
* `Internal` – generic internal runtime error.
* `HostAbi` – catch-all for unmapped host errors.
* `Economic` – insufficient mana or other economic constraint.
* `NotImplemented` – feature is not yet implemented.
* `DagOperationFailed` – failure while anchoring data to the DAG.
* `SignatureError` – invalid or unverifiable signature.
* `CryptoError` – general cryptography failure.
* `WasmExecutionError` – error during WASM execution.
* `ResourceLimitExceeded` – operation exceeded configured limits.
* `InvalidSystemApiCall` – guest attempted an unsupported host call.

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 