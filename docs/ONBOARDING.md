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

*   **`icn-node` (Demonstration):**
    The `icn-node` currently runs a sequence of demonstrations (API calls, DAG operations, stubbed network interactions) and then exits. It does not yet run as a persistent daemon.
    ```bash
    ./target/debug/icn-node
    ```
*   **`icn-cli`:**
    The CLI interacts with the (conceptual) node via the `icn-api`.
    ```bash
    # General help
    ./target/debug/icn-cli --help

    # Node info and status
    ./target/debug/icn-cli info
    ./target/debug/icn-cli status
    ./target/debug/icn-cli status offline # Test error case

    # DAG operations (requires valid JSON strings)
    # Example: submit a block (replace ... with actual valid JSON parts)
    ./target/debug/icn-cli dag put '{"cid":{"version":1,"codec":113,"hash_alg":18,"hash_bytes":[...]},"data":[...],"links":[]}'
    # Example: retrieve a block
    ./target/debug/icn-cli dag get '{"version":1,"codec":113,"hash_alg":18,"hash_bytes":[...]}'

    # Network operations (uses stubbed services via API)
    ./target/debug/icn-cli network discover-peers
    # Example: send a RequestBlock message (replace ... with actual valid JSON parts)
    ./target/debug/icn-cli network send-message mock_peer_1 '{"RequestBlock":{"version":1,"codec":112,"hash_alg":18,"hash_bytes":[100,97,116,97]}}'
    ```

## 4. Understanding the Codebase

*   **Workspace Root (`Cargo.toml`):** Defines the workspace members (all the crates).
*   **`crates/` directory:** Contains all individual library and binary crates.
    *   **`icn-common`**: Core data structures (CIDs, DIDs, `DagBlock`, `NodeStatus`, etc.) and the central `CommonError` enum used throughout the workspace.
    *   **`icn-api`**: Defines functions that act as the API layer for node interactions. Currently, these are direct function calls but are designed to be adaptable for RPC.
    *   **`icn-dag`**: Implements L1 DAG block storage (currently an in-memory `HashMap`).
    *   **`icn-network`**: Contains networking abstractions (`NetworkService` trait, `NetworkMessage` enum) and a `StubNetworkService` for testing.
    *   **`icn-identity`**: Placeholders for DID management and cryptographic functions.
    *   **`icn-node`**: The main binary executable that demonstrates integration of other crates.
    *   **`icn-cli`**: The command-line interface client.
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
2.  **Define API Function:** Add the function signature to `icn-api/src/lib.rs`. It should take necessary parameters and return `Result<ResponseType, CommonError>`.
3.  **Implement API Function Logic:** This might involve calling functions from other crates (e.g., `icn-dag`, `icn-identity`, `icn-network`). Handle their `Result`s appropriately.
4.  **Add Unit Tests:** In `icn-api/src/lib.rs` (within `#[cfg(test)] mod tests`), add tests for your new API function, covering both success and error cases.
5.  **Expose in `icn-cli` (if applicable):**
    *   Add a new command/subcommand in `icn-cli/src/main.rs`.
    *   Implement a handler function that parses arguments, calls the new API function from `icn-api`.
    *   Handle the `Result`, print output to `stdout` or errors to `stderr`, and manage exit codes.
    *   Update `print_usage()` and any relevant subcommand usage messages.
6.  **Demonstrate in `icn-node` (if applicable):** If it's a core function, add a call to it in `icn-node/src/main.rs` as part of its demonstration sequence, showing how to use it and handle its outcome.
7.  **Documentation:** Update relevant `README.md` files (for `icn-api`, `icn-cli`, etc.) and this `ONBOARDING.md` if the new endpoint is significant for developers.
8.  **Run Checks:** `cargo fmt --all`, `cargo clippy --all -- -D warnings`, `cargo test --all`.
9.  **Commit & Push:** Follow commit message guidelines (see `CONTRIBUTING.md`).

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

The immediate next steps focus on making the protocol stack more concrete:

*   **Real Persistence:** Implement file-based or SQLite backends for `icn-dag`.
*   **Real Networking:** Integrate `libp2p` into `icn-network` for actual P2P communication.
*   **Configuration:** Add configuration file support for `icn-node`.
*   **Identity Implementation:** Flesh out DID methods and cryptographic primitives in `icn-identity`.

Look for `TODO:` comments in the code and open GitHub issues for good places to start contributing.

(TODO: Add simple sequence diagrams for core flows like block storage and peer messaging once the APIs stabilize further.)

// TODO (#issue_url_for_multinode_docs): Create a new document (e.g., `docs/MULTI_NODE_GUIDE.md`) detailing 
// configuration, bootstrapping, and dev/test workflows for setting up and running multi-node local clusters 
// once real networking and persistence are implemented.

--- 
Thank you for your interest in contributing to ICN Core! 