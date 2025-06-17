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

## Public API Style

The API style emphasizes:

*   **Security:** Robust sandboxing and controlled access to host capabilities.
*   **Performance:** Efficient execution of runtime logic, especially for WASM.
*   **Modularity:** Clear separation between the runtime environment and the code being executed.
*   **Well-Defined Interface:** A stable and clear set of host functions for guest code.

## Error Handling

`MeshJobError` is used throughout the runtime to represent failures while processing mesh jobs. The following variants may be returned:

* `Network` – underlying mesh network failures.
* `NoSuitableExecutor` – executor selection could not find a valid candidate.
* `MissingOrInvalidReceipt` – receipt was not produced or failed validation.
* `UnknownJob` – the referenced job does not exist in runtime state.
* `ExecutionTimeout` – executor failed to provide a receipt in time.
* `ProcessingFailure` – generic failure while handling a job.
* `Serialization` – failed to encode or decode job data.
* `InvalidSpec` – job specification or parameters are invalid.
* `PermissionDenied` – caller lacks permission for the operation.
* `InvalidJobState` – operation attempted on a job in the wrong state.
* `Internal` – unexpected internal runtime error.
* `HostAbi` – wrapper for a lower level `HostAbiError`.
* `Economic` – issues related to mana or payment logic.
* `NotImplemented` – feature is not yet implemented.
* `DagOperationFailed` – failure while interacting with the DAG store.
* `InvalidSignature` – signature verification failed for a receipt or message.
* `CryptoError` – other cryptographic failure.
* `WasmExecutionError` – execution engine reported an error.
* `ResourceLimitExceeded` – operation exceeded allowed resources.
* `InvalidSystemApiCall` – attempted to call an invalid system API.

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 