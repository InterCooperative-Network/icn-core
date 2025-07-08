# ICN Node (`icn-node`)

This crate provides the main binary for running a long-lived InterCooperative Network (ICN) daemon.
It integrates various core components to operate a functional ICN node, handling initialization,
lifecycle, configuration, service hosting, and persistence.

See [CONTEXT.md](../CONTEXT.md) for ICN Core design philosophy and crate roles.
Async runtime considerations are documented in [async-guide](../docs/async-guide.md).

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
3.  **Network Operations:**
    *   When built with `with-libp2p`, the node spawns a real `Libp2pNetworkService` and discovers peers via bootstrap addresses.
    *   Demonstrates submitting another `DagBlock` and broadcasting its announcement using `broadcast_message`.
    *   Demonstrates sending a direct `RequestBlock` message to a discovered peer using `send_message`.

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
storage_backend = "rocksdb"
storage_path = "./icn_data/node_store"
mana_ledger_path = "./mana_ledger.sqlite"
node_did_path = "./icn_data/node_did.txt"
node_private_key_path = "./icn_data/node_sk.bs58"
```

To start the node using this configuration:

```bash
./target/debug/icn-node --config ./node_config.toml
```

Any CLI option provided will override the value from the file.

### Nested Configuration & Environment Variables

Configuration files may organize settings into sections. The following is
equivalent to the flat example above:

```toml
[storage]
backend = "rocksdb"
path = "./icn_data/node_store"

[http]
listen_addr = "127.0.0.1:8080"

[identity]
node_did_path = "./icn_data/node_did.txt"
node_private_key_path = "./icn_data/node_sk.bs58"
```

Every option can also be set via environment variables prefixed with `ICN_`.
For example:

```bash
ICN_HTTP_LISTEN_ADDR=0.0.0.0:9000 ICN_STORAGE_BACKEND=sqlite \
    ./target/debug/icn-node --config ./node_config.toml
```

Environment variables override file values but are in turn overridden by CLI
flags.

Useful CLI flags include:

* `--node-did-path <PATH>` – location to read/write the node DID string
* `--node-private-key-path <PATH>` – location to read/write the node private key
* `--storage-backend <memory|file|sqlite|sled|rocksdb>` – choose the DAG storage backend
* `--storage-path <PATH>` – directory for the file or SQLite backends
* `--mana-ledger-backend <file|sled|sqlite|rocksdb>` – choose ledger persistence
* `--mana-ledger-path <PATH>` – location of the mana ledger database
* `--governance-db-path <PATH>` – location to persist governance proposals and votes
* `--http-listen-addr <ADDR>` – HTTP server bind address (default `127.0.0.1:7845`)
* `--node-name <NAME>` – human-readable node name for logs
* `--listen-address <MULTIADDR>` – libp2p listen multiaddr (alias `--p2p-listen-addr`)
* `--bootstrap-peers <LIST>` – comma-separated list of bootstrap peer multiaddrs
* `--enable-p2p` – enable libp2p networking (requires `with-libp2p` feature)
* `--api-key <KEY>` – require this key via the `x-api-key` header for all requests
* `auth_token` / `auth_token_path` – set a bearer token string or file and require `Authorization: Bearer <token>`
* `--open-rate-limit <N>` – allowed unauthenticated requests per minute when no API key is set
* `--tls-cert-path <PATH>` – PEM certificate file to enable HTTPS
* `--tls-key-path <PATH>` – PEM private key file to enable HTTPS

### Authentication and TLS

If `auth_token` or `auth_token_path` is supplied, the server requires
`Authorization: Bearer <token>` on every request in addition to any `x-api-key`.
Providing both `tls_cert_path` and `tls_key_path` switches the HTTP server to
HTTPS mode using the given PEM files.

Supplying both TLS options makes the server listen on HTTPS instead of HTTP.

When the node starts, it will attempt to load its DID and private key from the
configured paths. If no key material exists, a new Ed25519 key pair is generated
and written to these files. Subsequent runs will reuse the persisted identity
allowing consistent node identification across restarts.

## Contributing

Please refer to the main `CONTRIBUTING.md` in the root of the `icn-core` repository.

Future contributions to `icn-node` will focus on:
*   Implementing a proper main event loop (e.g., using an async runtime like Tokio).
*   Integrating a real RPC server.
*   Loading configurations from files.
*   Managing the lifecycle of different service components.
*   Robust logging and metrics.

### Metrics Endpoint

The embedded HTTP server exposes `/metrics` for Prometheus scraping. It returns
runtime counters such as `host_submit_mesh_job_calls` in the standard text
format:

```bash
curl http://127.0.0.1:7845/metrics
```

## License

This crate is licensed under the Apache 2.0 license, as is the entire `icn-core` repository. 