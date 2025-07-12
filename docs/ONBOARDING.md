# Developer Onboarding Guide for ICN Core

Welcome to the InterCooperative Network (ICN) Core project! This guide will help you get set up, understand the codebase, and start contributing.

If you just want the minimal steps, start with [beginner/README.md](beginner/README.md) first.

Before jumping into the setup steps below, please read [CONTEXT.md](../CONTEXT.md) at the repository root. It explains the project's goals, rules, and terminology that all contributors should understand.

## 1. Prerequisites

*   **Rust:** Install the stable Rust toolchain using [rustup.rs](https://rustup.rs/).
    *   Run `rustup toolchain install stable` if you don't already have it.
    *   Set the override for this repository with `rustup override set stable`.
    *   The project uses the `stable` channel, as defined in `rust-toolchain.toml`.
*   **Git:** For version control.
*   **EditorConfig Plugin:** (Recommended) For your IDE/editor to maintain consistent coding styles across the project (uses `.editorconfig`).
*   **Basic Familiarity:** With Rust programming, `cargo`, and decentralized systems concepts (DIDs, CIDs, P2P) will be helpful.

## 2. Project Setup

1.  **Clone the Repository:**
    ```bash
    git clone <repository-url> # Replace with the actual URL
    cd icn-core
    ```
2.  **Initial Build & Test:**
    ```bash
    cargo build
    cargo test --all # Run tests for all crates in the workspace
    ```
    This will download all dependencies and compile the project. If all tests pass, your setup is correct.
3.  **Install Toolchain Components:**
    ```bash
    rustup component add clippy rustfmt
    ```
    These components are required for linting and formatting checks that run in CI.
4.  **Install `just` (optional but recommended):**
    ```bash
    cargo install just
    just --list    # Show available development commands
    ```
    The `just` command runner provides shortcuts for formatting, linting, testing, and devnet tasks defined in the repository's `justfile`.
5.  **Install WebAssembly tooling:**
    ```bash
    rustup target add wasm32-unknown-unknown
    cargo install wasm-tools --locked
    ```
    ICN crates compile to WebAssembly for deterministic sandboxing. The `wasm32-unknown-unknown` target and `wasm-tools` utilities are required for building and running tests that involve the runtime or CCL compiler.
6.  **Install Git hooks with `pre-commit`:**
    ```bash
    pre-commit install
    ```
    This sets up formatting and linting checks that run automatically before each commit.

## 3. Building and Running Components

### 3.1. Building

*   **Build all crates (debug mode):**
    ```bash
    cargo build
    ```
*   **Build all crates (release mode):**
    ```bash
    cargo build --release
    ```
*   **Build a specific crate (e.g., `icn-cli`):**
    ```bash
    cargo build -p icn-cli
    ```

Binaries will be placed in the `target/debug/` or `target/release/` directory at the workspace root.

### 3.2. Testing

*   **Run all tests in the workspace:**
    ```bash
    cargo test --all
    ```
*   **Run tests for a specific crate (e.g., `icn-common`):**
    ```bash
    cargo test -p icn-common
    ```
*   **Run a specific test function:**
    ```bash
    cargo test -p <crate_name> --test <test_module_name> <test_function_name>
    # Or just part of the name
    cargo test <partial_test_name>
    ```
*   **Run persistence tests for DAG backends:**
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

### 3.3. Linting & Formatting

The project uses `cargo fmt` for code formatting and `cargo clippy` for linting. These are checked in CI.

*   **Check formatting:**
    ```bash
    cargo fmt --all --check
    ```
*   **Apply formatting:**
    ```bash
    cargo fmt --all
    ```
*   **Run Clippy (strict, deny warnings):**
    ```bash
    cargo clippy --all -- -D warnings
    ```

### 3.4. Running Binaries

*   **`icn-node` (HTTP Server):**
    The `icn-node` runs as a persistent HTTP server exposing API endpoints. It manages DAG storage, governance state, mana accounting, and P2P networking.
    To run the node (defaults to in-memory storage and listens on `127.0.0.1:7845`):
    ```bash
    cargo run -p icn-node
    ```
    
    **Common Configuration Options:**
    ```bash
    # Basic node with persistent storage
    cargo run -p icn-node -- --http-listen-addr 0.0.0.0:8000 \
        --storage-backend sqlite --storage-path ./my_node_data.sqlite \
        --mana-ledger-backend sled --mana-ledger-path ./ledger.sled
    
    # Node with P2P networking enabled
    cargo run -p icn-node -- --enable-p2p \
        --p2p-listen-addr /ip4/0.0.0.0/tcp/4001 \
        --storage-backend sqlite --storage-path ./node.sqlite
    
    # Production node with authentication and TLS
    cargo run -p icn-node -- --http-listen-addr 0.0.0.0:8443 \
        --storage-backend sqlite --storage-path ./node.sqlite \
        --mana-ledger-backend sled --mana-ledger-path ./mana.sled \
        --api-key "secure-api-key" \
        --auth-token "bearer-token" \
        --tls-cert-path ./certs/server.crt \
        --tls-key-path ./certs/server.key
    ```
    
    **Available Storage Backends:**
    - `--storage-backend sqlite|sled|rocksdb|file` (for DAG storage)
    - `--mana-ledger-backend sled|sqlite|rocksdb|file` (for mana accounting)
    
    **Security Options:**
    - `--api-key` - Require x-api-key header for authentication
    - `--auth-token` - Require Bearer token authentication
    - `--tls-cert-path` / `--tls-key-path` - Enable HTTPS-only mode
    
    **P2P Networking:**
    - `--enable-p2p` - Enable libp2p networking
    - `--p2p-listen-addr` - P2P listening address
    - `--bootstrap-peers` - Connect to existing federation
    
    The server will print a message indicating it's listening and then run until stopped (e.g., with Ctrl+C).

*   **`icn-cli` (HTTP Client):**
    The CLI now interacts with an `icn-node` instance exclusively via its HTTP API.
    You must specify the API URL of the target node if it's not running on the default `http://127.0.0.1:7845`.

    **General help:**
    ```bash
    cargo run -p icn-cli -- --help
    ```

    **Node info and status (assuming node is running):**
    ```bash
    cargo run -p icn-cli -- info
    # If node is on a different address:
    cargo run -p icn-cli -- --api-url http://some.other.host:8000 info

    cargo run -p icn-cli -- status
    ```
    (The old `status offline` example is removed as status is now a direct reflection of the running node via HTTP).

    See the new section below for detailed command examples.

### 3.5. Example CLI Usage (Interacting with `icn-node` HTTP API)

This section provides examples for all major `icn-cli` commands. Ensure an `icn-node` instance is running and accessible at the specified `--api-url` (defaults to `http://127.0.0.1:7845`).

**1. Node Information:**
   ```sh
   cargo run -p icn-cli -- info
   # With a custom API URL:
   cargo run -p icn-cli -- --api-url http://localhost:7845 info
   ```

**2. Node Status:**
   ```sh
   cargo run -p icn-cli -- status
   ```

**3. DAG Operations:**

   *   **Store a DAG Block (`dag put`):**
      Provide JSON containing the block data. The endpoint returns the CID as a base32 string.
      ```sh
      cargo run -p icn-cli -- dag put '{ "data": [104,101,108,108,111] }'
      # => "bafy...cid-string"
      ```

   *   **Retrieve a DAG Block (`dag get`):**
      Send the CID string returned from `dag put`.
      ```sh
      cargo run -p icn-cli -- dag get '{ "cid": "bafy...cid-string" }'
      ```

**4. Governance Operations:**

   *   **Submit a Proposal (`governance submit`):**
      Requires a JSON string for `ApiSubmitProposalRequest`.
      Example `ApiSubmitProposalRequest` JSON:
      ```json
      {
        "proposer_did": "did:example:123",
        "proposal_type_json": { "GenericText": "My awesome proposal idea" },
        "description": "This proposal aims to do great things.",
        "duration_secs": 604800,
        "quorum": null,
        "threshold": null
      }
      ```
      CLI command:
      ```sh
      cargo run -p icn-cli -- governance submit '{ "proposer_did": "did:example:proposer123", "proposal_type_json": { "SystemParameterChange": ["max_users", "1000"] }, "description": "Increase max users to 1000", "duration_secs": 86400 }'
      ```

   *   **Cast a Vote (`governance vote`):**
      Requires a JSON string for `ApiCastVoteRequest`.
      Example `ApiCastVoteRequest` JSON (replace `proposal_id_value` with an actual ID from `governance proposals`):
      ```json
      {
        "voter_did": "did:example:456",
        "proposal_id": "proposal_id_value",
        "vote_option": "yes"
      }
      ```
      CLI command:
      ```sh
      cargo run -p icn-cli -- governance vote '{ "voter_did": "did:example:voter456", "proposal_id": "did:example:proposer123:Increase ma:1678886400", "vote_option": "yes" }'
      ```
      (Note: The `proposal_id` must be an existing ID. You can get valid IDs from the `governance proposals` command.)

   *   **List Proposals (`governance proposals`):**
      ```sh
      cargo run -p icn-cli -- governance proposals
      ```

   *   **Get a Specific Proposal (`governance proposal <id>`):**
      Replace `<proposal_id_from_list>` with an ID obtained from `governance proposals`.
      ```sh
      cargo run -p icn-cli -- governance proposal <proposal_id_from_list>
      # Example:
      cargo run -p icn-cli -- governance proposal "did:example:proposer123:Increase ma:1678886400"
      ```

**5. Federation Management:**

   The ICN CLI provides comprehensive federation management commands:

   *   **Join a Federation (`federation join <peer_id>`):**
      ```sh
      cargo run -p icn-cli -- federation join "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."
      ```

   *   **Get Federation Status (`federation status`):**
      ```sh
      cargo run -p icn-cli -- federation status
      ```

   *   **Leave a Federation (`federation leave <peer_id>`):**
      ```sh
      cargo run -p icn-cli -- federation leave "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."
      ```

   *   **List Federation Peers (`federation list-peers`):**
      ```sh
      cargo run -p icn-cli -- federation list-peers
      ```

**6. Network Operations:**

   *   **Discover Peers (`network discover-peers`):**
      ```sh
      cargo run -p icn-cli -- network discover-peers
      ```

   *   **List Connected Peers (`network peers`):**
      ```sh
      cargo run -p icn-cli -- network peers
      ```

   *   **Send Direct Message (`network send-message`):**
      ```sh
      cargo run -p icn-cli -- network send-message <peer_id> '{"message": "hello"}'
      ```

(Note: P2P networking features require the node to be started with `--enable-p2p`.)

### 3.6. Example `icn-node` Configuration with TLS and API Keys

`icn-node` reads its settings from a TOML file or environment variables. Below is
an example configuration that enables an API key, uses persistent storage, and
expects traffic to be terminated via a TLS reverse proxy (such as Nginx or
Caddy).

```toml
# node_config.toml
node_name = "Federation Node"
http_listen_addr = "0.0.0.0:7845"        # HTTP address
storage_backend = "sqlite"                # DAG storage backend
storage_path = "./icn_data/node.sqlite"   # DAG storage path
mana_ledger_backend = "sled"              # Mana accounting backend
mana_ledger_path = "./icn_data/mana.sled" # Mana ledger path
api_key = "mysecretkey"                   # API key for authentication
auth_token = "bearer-token-here"          # Bearer token for auth
open_rate_limit = 60                      # Requests per minute without auth
enable_p2p = true                         # Enable P2P networking
p2p_listen_addr = "/ip4/0.0.0.0/tcp/4001" # P2P listening address
tls_cert_path = "./certs/server.crt"      # TLS certificate (optional)
tls_key_path = "./certs/server.key"       # TLS private key (optional)
```

Start the node with:

```bash
./target/debug/icn-node --config node_config.toml
```

For TLS, you can either:
1. **Use built-in TLS** by providing `tls_cert_path` and `tls_key_path` in the configuration
2. **Use a reverse proxy** on port `443` that forwards requests to the HTTP server

**Option 1: Built-in TLS (Recommended)**
```bash
./target/debug/icn-node --http-listen-addr 0.0.0.0:8443 \
  --tls-cert-path ./certs/server.crt \
  --tls-key-path ./certs/server.key \
  --api-key "mysecretkey"
```

**Option 2: Reverse Proxy (Nginx)**
```nginx
server {
    listen 443 ssl;
    ssl_certificate     /path/to/fullchain.pem;
    ssl_certificate_key /path/to/privkey.pem;
    location / {
        proxy_pass http://127.0.0.1:7845;
        proxy_set_header x-api-key mysecretkey;
        proxy_set_header Authorization "Bearer bearer-token-here";
    }
}
```

Both approaches secure the HTTP API with TLS and handle authentication appropriately.

### 3.7. Environment Variables

`icn-node` can read any configuration value from environment variables using the
`ICN_` prefix. Variable names mirror the keys found in the configuration file.
For example, `ICN_HTTP_LISTEN_ADDR` sets `http_listen_addr` and
`ICN_STORAGE_BACKEND` sets `storage_backend`. These environment values override
those loaded from a file but are overridden by CLI flags.

Example usage:

```bash
export ICN_HTTP_LISTEN_ADDR=0.0.0.0:9000
export ICN_STORAGE_BACKEND=sled
cargo run -p icn-node --config node_config.toml
```

To use RocksDB instead of sled, build `icn-node` with the `persist-rocksdb` feature
and set `storage_backend = "rocksdb"` in your configuration.

## 4. Understanding the Codebase

*   **Workspace Root (`Cargo.toml`):** Defines the workspace members (all the crates).
*   **`crates/` directory:** Contains all individual library and binary crates.
    *   **`icn-common`**: Core data structures (CIDs, DIDs, `DagBlock`, `NodeStatus`, etc.) and the central `CommonError` enum used throughout the workspace.
    *   **`icn-api`**: Defines HTTP API endpoints and request/response types for external consumption.
    *   **`icn-dag`**: Implements content-addressed DAG storage with multiple backend support (SQLite, Sled, RocksDB, File).
    *   **`icn-network`**: P2P networking via libp2p with Kademlia DHT and Gossipsub protocols.
    *   **`icn-identity`**: DID management, Ed25519 cryptographic signing, and execution receipt verification.
    *   **`icn-governance`**: Proposal creation, voting mechanisms, and governance state management.
    *   **`icn-economics`**: Mana accounting system with multiple ledger backends and resource policies.
    *   **`icn-mesh`**: Distributed mesh computing with job submission, bidding, and execution.
    *   **`icn-reputation`**: Reputation scoring and validation for network participants.
    *   **`icn-node`**: The main binary executable that runs a persistent HTTP server with P2P networking.
    *   **`icn-cli`**: The command-line interface client that interacts with `icn-node` via HTTP API.
*   **`docs/` directory:** Contains this onboarding guide and potentially other architectural documents.
*   **`.github/` directory:** CI workflows, issue templates, Dependabot configuration.

### Error Handling Approach

A key principle in `icn-core` is robust error handling:
*   Library functions return `Result<T, icn_common::CommonError>` instead of panicking on recoverable errors.
*   `CommonError` provides specific variants for different error conditions (e.g., `StorageError`, `PeerNotFound`, `DeserializationError`).
*   Binary crates (`icn-node`, `icn-cli`) handle these `Result`s, print informative error messages to `stderr`, and exit with non-zero status codes on failure.
This makes the system more predictable and easier to debug.

## 5. Example Workflow: Adding a New API Endpoint

1.  **Define Data Structures (if new):** If your endpoint uses new request/response types, define them in `icn-common/src/lib.rs` (ensure they derive `Serialize`, `Deserialize` if they cross API boundaries).
2.  **Implement in `icn-node`:**
    *   Add a new Axum handler function in `icn-node/src/main.rs`.
    *   This function will take `State<AppState>` and any necessary extractors (e.g., `Json<RequestBody>`).
    *   It should interact with `AppState.dag_storage` or `AppState.governance_module`.
    *   Return an Axum `impl IntoResponse` (typically `(StatusCode, Json<ResponseBody>)`).
    *   Add the new route to the Axum `Router`.
3.  **Add Unit/Integration Tests for the Node Endpoint:** Test the new handler, possibly using `axum::body::Body` and `tower::ServiceExt` or a test client.
4.  **Expose in `icn-cli`:**
    *   Add a new command/subcommand to the `clap` enums in `icn-cli/src/main.rs`.
    *   Implement a new async handler function in `icn-cli` that:
        *   Constructs the request path and body (if any).
        *   Uses the `reqwest::Client` (e.g., `get_request` or `post_request` helpers) to call the `icn-node` endpoint.
        *   Parses the response and prints it or an error message.
    *   Update the main command dispatch logic in `icn-cli`.
5.  **Documentation:** Update `README.md` for `icn-node` (documenting the new endpoint, request/response formats, error codes) and this `ONBOARDING.md` with CLI examples.
6.  **Run Checks:** `cargo fmt --all`, `cargo clippy --all -- -D warnings`, `cargo test --all`.
7.  **Commit & Push:** Follow commit message guidelines (see `CONTRIBUTING.md`).

## 6. Contribution Steps

1.  **Find/Create an Issue:** Look for existing issues labeled "good first issue" or "help wanted." If you have a new idea, create an issue to discuss it first.
2.  **Fork the Repository:** (If you are an external contributor).
3.  **Create a Branch:** `git checkout -b feature/my-new-feature` or `fix/some-bug`.
4.  **Implement Changes:** Write code, add tests, update documentation.
5.  **Test Thoroughly:** Ensure all tests pass, including any new ones you've added.
6.  **Format & Lint:** Run `cargo fmt --all` and `cargo clippy --all -- -D warnings`.
7.  **Commit Changes:** Use conventional commit messages.
8.  **Push to Your Fork/Branch.**
9.  **Create a Pull Request:** Target the `develop` branch of the upstream repository. Rebase your feature branch onto `develop` before opening the PR and clearly describe your changes with links to any relevant issues.

## 7. Next Steps for the Project (and areas for contribution)

The system now has a production-ready foundation with comprehensive features.
Current areas for contribution include:

*   **✅ Persistence Backends:** Multiple storage backends (SQLite, Sled, RocksDB, File) are implemented for both DAG and mana storage.
*   **✅ Networking (Libp2p):** Full libp2p support with Kademlia DHT and Gossipsub protocols is implemented.
*   **✅ Federation Management:** Complete federation join/leave/status functionality is available.
*   **✅ Security:** Ed25519 cryptographic signing, API authentication, and TLS support are implemented.
*   **✅ Configuration:** Comprehensive configuration file support is available.

**Active Development Areas:**
*   **Performance Optimization:** Benchmarking and performance improvements for large-scale federations.
*   **Advanced CCL Features:** Enhanced Cooperative Contract Language capabilities.
*   **Testing:** Expand test coverage, especially for federation scenarios and security edge cases.
*   **Documentation:** User guides for specific deployment scenarios and federation setup.
*   **Monitoring:** Enhanced metrics and observability features beyond basic Prometheus support.
*   **UI/UX:** Web interfaces for federation management and governance participation.

Look for `TODO:` comments in the code and open GitHub issues for good places to start contributing.

### Core Sequence Diagrams

Below are simplified sequence diagrams illustrating two fundamental flows in the system.

#### Block Storage

```text
Node Runtime -> DagStorageService: put(block)
DagStorageService -> StorageBackend: persist block
StorageBackend --> DagStorageService: CID
DagStorageService --> Node Runtime: CID
```

#### Peer Messaging

```text
Node A -> NetworkService: send_message(Node B, msg)
NetworkService -> Node B: deliver msg
Node B -> NetworkService: optional response
NetworkService -> Node A: response
```

## 8. Running the Federation Devnet

The repository includes a containerized devnet for quickly spinning up a three-node federation and testing Cooperative Contract Language (CCL) jobs.

1. **Build the workspace binaries (optional if using Docker caches):**
   ```bash
   cargo build --release
   ```
2. **Launch the federation:**
   ```bash
   cd icn-devnet
   ./launch_federation.sh
   ```
   The script checks prerequisites, starts Docker containers, waits for P2P convergence, and submits a test job. Typical output looks like:
   ```bash
   🚀 ICN Federation Devnet Launch Starting...
   ✅ Prerequisites checked
   ✅ Node A is healthy
   ✅ Node B is healthy
   ✅ Node C is healthy
   ✅ P2P network has converged
   ✅ Job submitted with ID: cidv1-85-20-abc123...
   🎉 ICN Federation is now running!
   ```
3. **Run with monitoring (Prometheus & Grafana):**
   ```bash
  cd icn-devnet
  docker-compose --profile monitoring up -d
  ```
  Prometheus will be available at <http://localhost:9090> and Grafana at
  <http://localhost:3000> (login `admin` / `icnfederation`).
   Alternatively, run the standalone monitoring stack:
   ```bash
   docker compose -f docker-compose-monitoring.yml up -d
   ```
4. **Submit your own job:**
   ```bash
   curl -X POST http://localhost:5001/mesh/submit \
     -H 'Content-Type: application/json' \
  -d '{"manifest_cid":"example_manifest","spec_bytes":"BASE64_SPEC","cost_mana":50}'
   ```
   The response contains `job_id`. You can query any node for its status:
   ```bash
   curl http://localhost:5002/mesh/jobs/JOB_ID
   ```
5. **Collect the execution receipt:** when the job completes, the status response includes a `result_cid`. Retrieve the receipt data from any node via:
   ```bash
   curl -X POST http://localhost:5003/dag/get \
     -H 'Content-Type: application/json' \
     -d '{"cid":"RESULT_CID"}'
   ```

## 9. Submitting a CCL WASM Job

The devnet can execute Cooperative Contract Language (CCL) policies compiled to WebAssembly.
Follow these steps to run your own module:

1. **Compile the contract:**
   ```bash
   icn-cli ccl compile ./my_policy.ccl ./my_policy.wasm
   ```
2. **Store the WASM in the DAG:** the returned CID is base32 and begins with `bafy`.
   ```bash
   curl -X POST http://localhost:5001/dag/put \
     --data-binary '@my_policy.wasm'
   # => "bafy...base32-cid"
   ```
3. **Generate the job spec:**
   ```bash
   generate_ccl_job_spec --wasm-cid bafy...base32-cid --output ccl_job_spec.json
   ```
4. **Submit the job:**
   ```bash
   curl -X POST http://localhost:5001/mesh/submit \
     -H 'Content-Type: application/json' \
     -d @ccl_job_spec.json
   ```

Refer to `MULTI_NODE_GUIDE.md` for more details on manual multi-node setups.

--- 
Thank you for your interest in contributing to ICN Core! 