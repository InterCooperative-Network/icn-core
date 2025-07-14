# ICN CLI (`icn-cli`)

This crate provides a command-line interface (CLI) for interacting with the InterCooperative Network (ICN).
It allows users and administrators to manage nodes, interact with the network, and perform administrative tasks.
The CLI aims for usability, discoverability, and scriptability.

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-cli` is the primary tool for users to interact with an ICN node from the command line. It connects to an ICN node (via the `icn-api` crate, which in turn might be an RPC client to a running `icn-node` instance) to perform actions.

## Current Functionality

*   **Node Information:**
    *   `icn-cli info`: Display basic information about the connected node (name, version, status).
    *   `icn-cli status [offline]`: Display detailed operational status of the node. The optional `offline` argument simulates a scenario where the node cannot be reached or reports as offline, allowing testing of error paths.
*   **DAG Operations:**
    *   `icn-cli dag put <DAG_BLOCK_JSON_STRING>`: Submits a new DagBlock to the node. The block data must be provided as a complete JSON string.
    *   `icn-cli dag get <CID_JSON_STRING>`: Retrieves a DagBlock from the node by its CID. The CID must be provided as a JSON string.
*   **Network Operations:**
    *   `icn-cli network discover-peers`: Query the connected node for peers. With the `with-libp2p` feature enabled the node will perform real discovery via libp2p.
    *   `icn-cli network send-message <PEER_ID> <MESSAGE_JSON>`: Send a `ProtocolMessage` (encoded as JSON) to a specified peer. Requires the node to run with libp2p networking.
    *   `icn-cli network peers`: Display this node's peer ID and the currently discovered peer list.
*   **Federation Operations:**
    *   `icn-cli federation init`: Initialize a new federation on this node.
    *   `icn-cli federation join <PEER_ID>`: Join a federation by adding the given peer.
    *   `icn-cli federation leave <PEER_ID>`: Leave a federation or remove the peer.
    *   `icn-cli federation list-peers`: List peers known to the node.
    *   `icn-cli federation status`: Display federation status including peer count.
    *   `icn-cli federation sync`: Synchronize federation state with peers.
*   **Identity Operations:**
    *   `icn-cli identity generate-proof <PROOF_REQUEST_JSON>`: Produce a zero-knowledge proof from the supplied request JSON.
    *   `icn-cli identity verify-proof <PROOF_JSON>`: Verify a proof and print whether it is valid.
*   **Monitoring:**
    *   `icn-cli monitor uptime`: Display node uptime using the metrics endpoint.
*   **Zero-Knowledge Operations:**
    *   `icn-cli zk generate-key`: Generate a Groth16 proving key and output the verifying key signature.
    *   `icn-cli zk analyze <CIRCUIT>`: Count constraints for a circuit.
    *   `icn-cli zk profile <CIRCUIT>`: Run Criterion benchmarks for a circuit.
*   **Miscellaneous:**
    *   `icn-cli hello`: A simple command to check if the CLI is responsive.
    *   `icn-cli help` or `icn-cli --help`: Displays usage information.
    *   `icn-cli wizard setup --config <FILE>`: Interactive node setup wizard.

### Example: Generate and Verify a Proof

```bash
# Generate a proof for a membership credential
cargo run -p icn-cli -- identity generate-proof '{"member_did":"did:example:123"}'
# => '{"proof":"base64string","backend":"Groth16"}'

# Verify the returned proof
cargo run -p icn-cli -- identity verify-proof '{"proof":"base64string","backend":"Groth16"}'
# => "verified: true"
```

### Example: Generate Groth16 Keys

```bash
# Generate a proving key and verifying key signature
cargo run -p icn-cli -- zk generate-key
# => '{"proving_key_path":"./groth16_proving_key.bin","verifying_key_signature_hex":"abc..."}'
```

### Example: Analyze a Circuit

```bash
cargo run -p icn-cli -- zk analyze age_over_18
# => 'constraints: 3'
```

### Example: Profile a Circuit

```bash
cargo run -p icn-cli -- zk profile age_over_18
# Runs `cargo bench` for the specified circuit
```

## Error Handling

The CLI aims to provide clear error messages and exit with appropriate status codes:
*   Successful commands generally exit with status code 0.
*   Commands that encounter an error (e.g., API error, network error, invalid input) will print an error message to `stderr` and exit with a non-zero status code (typically 1).
*   Errors from the underlying API or network services are propagated and displayed to the user in a readable format.

## Public API Style

As a CLI application, its "public API" is its command-line arguments, options, and output format (both `stdout` for data and `stderr` for errors).

*   Commands are structured hierarchically (e.g., `dag put`, `network discover-peers`).
*   Input for complex data structures (like DagBlocks or ProtocolMessages) is currently expected in JSON string format for simplicity in this development phase. Future versions may support file inputs or more structured argument parsing.
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