# ICN CLI (`icn-cli`)

This crate provides a command-line interface (CLI) for interacting with the InterCooperative Network (ICN).
It allows users and administrators to manage nodes, interact with the network, and perform administrative tasks.
The CLI aims for usability, discoverability, and scriptability.

## Purpose

The `icn-cli` is the primary tool for users to interact with an ICN node from the command line. It connects to an ICN node (via the `icn-api` crate, which in turn might be an RPC client to a running `icn-node` instance) to perform actions.

## Current Functionality

*   **Node Information:**
    *   `icn-cli info`: Display basic information about the connected node (name, version, status).
    *   `icn-cli status [offline]`: Display detailed operational status of the node. The optional `offline` argument simulates a scenario where the node cannot be reached or reports as offline, allowing testing of error paths.
*   **DAG Operations:**
    *   `icn-cli dag put <DAG_BLOCK_JSON_STRING>`: Submits a new DagBlock to the node. The block data must be provided as a complete JSON string.
    *   `icn-cli dag get <CID_JSON_STRING>`: Retrieves a DagBlock from the node by its CID. The CID must be provided as a JSON string.
*   **Network Operations (Stubbed):**
    *   `icn-cli network discover-peers`: Simulates peer discovery through the connected node. (Currently uses a stubbed network service).
    *   `icn-cli network send-message <PEER_ID> <MESSAGE_JSON>`: Simulates sending a `NetworkMessage` to a specified peer. The peer ID is a string, and the message is a JSON representation of a `NetworkMessage` variant (e.g., `RequestBlock`, `AnnounceBlock`). (Currently uses a stubbed network service).
*   **Miscellaneous:**
    *   `icn-cli hello`: A simple command to check if the CLI is responsive.
    *   `icn-cli help` or `icn-cli --help`: Displays usage information.

## Error Handling

The CLI aims to provide clear error messages and exit with appropriate status codes:
*   Successful commands generally exit with status code 0.
*   Commands that encounter an error (e.g., API error, network error, invalid input) will print an error message to `stderr` and exit with a non-zero status code (typically 1).
*   Errors from the underlying API or network services are propagated and displayed to the user in a readable format.

## Public API Style

As a CLI application, its "public API" is its command-line arguments, options, and output format (both `stdout` for data and `stderr` for errors).

*   Commands are structured hierarchically (e.g., `dag put`, `network discover-peers`).
*   Input for complex data structures (like DagBlocks or NetworkMessages) is currently expected in JSON string format for simplicity in this development phase. Future versions may support file inputs or more structured argument parsing.
*   Output is human-readable. Successful data retrieval is printed to `stdout`. Errors and verbose logging (if any) go to `stderr`.

## Contributing

Please refer to the main `CONTRIBUTING.md` in the root of the `icn-core` repository.

Specific contributions to `icn-cli` could include:
*   Adding new commands for upcoming features.
*   Improving argument parsing and validation.
*   Enhancing output formatting (e.g., table views, alternative output formats like JSON for scriptability).
*   Adding shell completion scripts.

## License

This crate is licensed under the Apache 2.0 license, as is the entire `icn-core` repository.