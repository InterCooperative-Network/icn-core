# Developer Onboarding Guide for ICN Core

Welcome to the InterCooperative Network (ICN) Core project! This guide will help you get set up, understand the codebase, and start contributing.

## 1. Prerequisites

*   **Rust:** Install the latest stable Rust toolchain. You can get it from [rustup.rs](https://rustup.rs/).
    *   Run `rustup update stable` to ensure you have the most recent version.
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
    The `icn-node` now runs as a persistent HTTP server exposing API endpoints. It manages DAG storage and governance state.
    To run the node (defaults to in-memory storage and listens on `127.0.0.1:7845`):
    ```bash
    cargo run -p icn-node
    ```
    You can specify the listen address and storage backend:
    ```bash
    cargo run -p icn-node -- --listen-addr 0.0.0.0:8000 --storage-backend file --storage-path ./my_node_data
    ```
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
      Requires a JSON string representing the `DagBlock`.
      Example `DagBlock` JSON (note: CIDs are complex; this is illustrative):
      ```json
      {
        "cid": {
          "version": 1,
          "codec": 85, // dag-cbor
          "hash_alg": 18, // sha2-256
          "hash_bytes": [72,101,108,108,111,87,111,114,108,100,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0] // Example hash
        },
        "data": [72,101,108,108,111,32,87,111,114,108,100,33], // "Hello World!" in bytes
        "links": []
      }
      ```
      CLI command (pass the JSON as a single string argument):
      ```sh
      cargo run -p icn-cli -- dag put '{ "cid": { "version": 1, "codec": 85, "hash_alg": 18, "hash_bytes": [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32] }, "data": [104,101,108,108,111], "links": [] }'
      ```

   *   **Retrieve a DAG Block (`dag get`):**
      Requires a JSON string representing the `Cid`.
      Example `Cid` JSON (matching the one above):
      ```json
      {
        "version": 1,
        "codec": 85,
        "hash_alg": 18,
        "hash_bytes": [72,101,108,108,111,87,111,114,108,100,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
      }
      ```
      CLI command:
      ```sh
      cargo run -p icn-cli -- dag get '{ "version": 1, "codec": 85, "hash_alg": 18, "hash_bytes": [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32] }'
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
        "duration_secs": 604800 
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

(Note: The old network examples are removed as they were stubbed and not part of the core HTTP API requirements in the prompt.)

## 4. Understanding the Codebase

*   **Workspace Root (`Cargo.toml`):** Defines the workspace members (all the crates).
*   **`crates/` directory:** Contains all individual library and binary crates.
    *   **`icn-common`**: Core data structures (CIDs, DIDs, `DagBlock`, `NodeStatus`, etc.) and the central `CommonError` enum used throughout the workspace.
    *   **`icn-api`**: Defines functions that act as the API layer for node interactions. Currently, these are direct function calls but are designed to be adaptable for RPC.
    *   **`icn-dag`**: Implements L1 DAG block storage (currently an in-memory `HashMap`).
    *   **`icn-network`**: Contains networking abstractions (`NetworkService` trait, `NetworkMessage` enum) and a `StubNetworkService` for testing.
    *   **`icn-identity`**: Placeholders for DID management and cryptographic functions.
    *   **`icn-node`**: The main binary executable that runs a persistent HTTP server for the ICN node API.
    *   **`icn-cli`**: The command-line interface client that interacts with `icn-node` via HTTP.
    *   Other crates (`icn-economics`, `icn-governance`, etc.) are placeholders for future development.
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
9.  **Create a Pull Request:** Target the `main` branch of the upstream repository. Clearly describe your changes and link the relevant issue.

## 7. Next Steps for the Project (and areas for contribution)

The system now has a foundational HTTP API and CLI client.
Immediate next steps from the original prompt include:

*   **Persistence Backends:** While `icn-node` supports a file backend for `DagStorageService`, implementing a `sled` backend is a next step. `GovernanceModule` currently uses in-memory storage; this needs a pluggable persistence strategy similar to `DagStorageService`.
*   **Real Networking (Libp2p):** The current focus was on the HTTP API. Integrating `libp2p` for true P2P federation remains a larger goal.
*   **Configuration:** Advanced configuration file support for `icn-node` (beyond CLI args).
*   **Identity Implementation:** Further flesh out DID methods and cryptographic primitives in `icn-identity`.
*   **Testing:** Enhance test coverage, especially integration tests for the node-cli interaction and endpoint tests for `icn-node`.

Look for `TODO:` comments in the code and open GitHub issues for good places to start contributing.

(TODO: Add simple sequence diagrams for core flows like block storage and peer messaging once the APIs stabilize further.)

// TODO (#issue_url_for_multinode_docs): Create a new document (e.g., `docs/MULTI_NODE_GUIDE.md`) detailing 
// configuration, bootstrapping, and dev/test workflows for setting up and running multi-node local clusters 
// once real networking and persistence are implemented.

--- 
Thank you for your interest in contributing to ICN Core! 