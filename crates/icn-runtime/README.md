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

### Governance

The runtime exposes host calls for managing on-chain proposals. Voting can be
closed via `host_close_governance_proposal_voting`, returning the final
`ProposalStatus` as a string. Accepted proposals may then be executed with
`host_execute_governance_proposal`, which updates the stored proposal and member
set.

## Error Types

`MeshJobError` enumerates failures that can occur while processing mesh jobs.
All variants can be converted from `HostAbiError` or `MeshNetworkError`.

* `Network` – issues communicating over the mesh network.
* `NoSuitableExecutor` – no executor met the policy for a job.
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