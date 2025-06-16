# ICN Node (`icn-node`)

This crate provides the main binary for running a long-lived InterCooperative Network (ICN) daemon.
It integrates various core components to operate a functional ICN node, handling initialization,
lifecycle, configuration, service hosting, and persistence.

## Purpose

The `icn-node` is intended to be the core daemon process that participants in the ICN will run.
It is responsible for:

*   Initializing and managing various ICN subsystems (e.g., identity, DAG storage, networking).
*   Exposing an API (likely via `icn-api` implementations like JSON-RPC or gRPC) for clients (including `icn-cli`) to interact with.
*   Participating in the P2P network, discovering peers, and exchanging messages.
*   Processing and validating data (e.g., DagBlocks, transactions).
*   Managing local data persistence.

## Current Functionality (Demonstration)

The `main.rs` in this crate currently serves as a demonstration of integrating and using the various library crates (`icn-common`, `icn-api`, `icn-dag`, `icn-network`). When run, it performs a sequence of operations:

1.  **Node Information & Status:** Calls `icn-api` functions (`get_node_info`, `get_node_status`) to retrieve and display node details. It demonstrates handling both successful calls and simulated error conditions (e.g., node offline).
2.  **DAG Operations:** Demonstrates submitting a sample `DagBlock` to the local DAG store (via `icn-api` which uses `icn-dag`) and then retrieving it.
3.  **Network Operations (Stubbed):**
    *   Initializes a `StubNetworkService` from the `icn-network` crate.
    *   Simulates peer discovery by calling `discover_peers`.
    *   Simulates submitting another `DagBlock` and then broadcasting its announcement using `broadcast_message`.
    *   Demonstrates sending a direct `RequestBlock` message to a (discovered) stubbed peer using `send_message`.

All operations show how results and errors from the API and underlying services are handled and printed to the console.

## Error Handling

The node executable aims to handle errors gracefully. Errors from API calls or internal services are caught, and informative messages are printed to `stderr`. In a production scenario, this would involve more robust logging and potentially different exit strategies or recovery mechanisms.

## Public API Style

As a daemon, `icn-node`'s primary external API will be through network protocols (e.g., RPC exposed by `icn-api`). Its command-line interface is minimal, primarily for startup and configuration.

The current `main()` function is for demonstration and testing the library integrations, not for long-running service yet.

## CLI Usage

The node accepts configuration via CLI flags or a TOML/YAML file. Provide the
file path with `--config <path>`. CLI flags override values found in the file.
Below is a minimal example in TOML (the same keys can be used in YAML):

```toml
node_name = "Local Node"
http_listen_addr = "127.0.0.1:8080"
storage_backend = "file"
storage_path = "./icn_data/node_store"
mana_ledger_path = "./mana_ledger.sled"
```

To start the node using this configuration:

```bash
./target/debug/icn-node --config ./node_config.toml
```

Any CLI option provided will override the value from the file.

## Contributing

Please refer to the main `CONTRIBUTING.md` in the root of the `icn-core` repository.

Future contributions to `icn-node` will focus on:
*   Implementing a proper main event loop (e.g., using an async runtime like Tokio).
*   Integrating a real RPC server.
*   Loading configurations from files.
*   Managing the lifecycle of different service components.
*   Robust logging and metrics.

## License

This crate is licensed under the Apache 2.0 license, as is the entire `icn-core` repository. 