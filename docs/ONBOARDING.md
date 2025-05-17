# Onboarding New Contributors to ICN Core

Welcome to the InterCooperative Network (ICN) Core project! We're excited to have you.
This guide will help you get your development environment set up and make your first contribution.

## Prerequisites

*   **Rust:** Ensure you have Rust installed. We recommend using `rustup` for managing Rust versions.
    *   Installation: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    *   This project currently uses the `stable` toolchain, as specified in `rust-toolchain.toml`.
*   **Git:** For version control.
*   **GitHub Account:** For fork/clone and submitting Pull Requests.

## 1. Fork and Clone the Repository

1.  **Fork:** Go to the [ICN Core GitHub repository](https://github.com/InterCooperative-Network/icn-core) (replace with actual URL) and click the "Fork" button.
2.  **Clone:** Clone your forked repository to your local machine:
    ```bash
    git clone git@github.com:YOUR_USERNAME/icn-core.git
    cd icn-core
    ```

## 2. Build the Project

Navigate to the root of the cloned `icn-core` directory and build the entire workspace:

```bash
cargo build
```

This will compile all crates in the monorepo.

## 3. Run Tests

After a successful build, run all tests to ensure everything is working correctly:

```bash
cargo test --all
```

## 4. Check Formatting and Linting

We use `rustfmt` for code formatting and `clippy` for linting. Please run these before committing your changes:

```bash
cargo fmt --all --check  # Check formatting
cargo clippy --all -- -D warnings # Run clippy, denying all warnings
```

To automatically format your code:
```bash
cargo fmt --all
```

## 5. Running Binaries

This workspace includes two binaries:

*   `icn-node`: The ICN daemon.
*   `icn-cli`: The command-line interface.

You can run them using `cargo run`:

```bash
# Run the ICN node (it will print status and then loop)
cd crates/icn-node
cargo run
cd ../.. # Back to root

# Run the ICN CLI with a command (e.g., info)
cd crates/icn-cli
cargo run -- info
cargo run -- status
cargo run -- status offline # Test status with simulated offline error
cd ../.. # Back to root
```

Or, after building with `cargo build` (or `cargo build --release` for an optimized build), you can find the executables in `target/debug/` (or `target/release/`) and run them directly:

```bash
./target/debug/icn-node
./target/debug/icn-cli info
```

## 6. Understanding the Codebase

*   Start by reading the main `README.md` for an overview of the project and crate structure.
*   Each crate in the `crates/` directory has its own `README.md` explaining its purpose and planned API.
*   The `lib.rs` (for library crates) or `main.rs` (for binary crates) in each crate's `src/` directory contains module-level documentation and `// TODO:` or `/// Planned:` comments indicating areas for development.

## 7. How to Add a New API Endpoint (Example Flow)

This is a simplified guide to adding a new query-type API endpoint:

1.  **Define Types in `icn-common` (`crates/icn-common/src/lib.rs`):**
    *   If your new endpoint returns a new data structure, define it here (e.g., `pub struct MyNewData { ... }`).
    *   If it can return specific new errors, add variants to the `CommonError` enum.

2.  **Implement Logic in `icn-api` (`crates/icn-api/src/lib.rs`):**
    *   Create a new public function, e.g., `pub fn get_my_new_data(...) -> Result<MyNewData, CommonError> { ... }`.
    *   This function will contain the core logic, potentially calling into other crates or accessing node state (in a real scenario).
    *   Add unit tests for this new function, covering success and error cases.

3.  **Expose via `icn-node` (Conceptual - `crates/icn-node/src/main.rs`):**
    *   In a real node with an RPC server, you would register your new API function with the server so it can be called remotely.
    *   For now, you can simulate its use by calling it directly in `icn-node`'s `main` function to test integration, similar to how `get_node_info` and `get_node_status` are called.

4.  **Add Command to `icn-cli` (`crates/icn-cli/src/main.rs`):**
    *   Add a new match arm in `main` for your new command (e.g., `"mynewdata"`).
    *   Call the `icn_api::get_my_new_data()` function.
    *   Print the results or errors nicely to the console.

5.  **Update Documentation:**
    *   Mention the new API endpoint in `icn-api/README.md` under "Planned Public API" (or move it to a "Implemented API" section).
    *   Add module/function documentation (`/// ...`).

## 8. Making a Contribution

1.  **Find an Issue:** Look for issues tagged "good first issue" or "help wanted" on GitHub.
2.  **Discuss:** If you plan to work on a larger feature, it's good to discuss it in the relevant issue first.
3.  **Branch:** Create a new branch for your changes: `git checkout -b your-feature-branch`.
4.  **Code:** Implement your changes, including tests and documentation.
5.  **Test & Lint:** Run `cargo test --all`, `cargo fmt --all --check`, and `cargo clippy --all -- -D warnings`.
6.  **Commit:** Write clear and concise commit messages.
7.  **Push:** Push your branch to your fork: `git push origin your-feature-branch`.
8.  **Pull Request:** Open a Pull Request (PR) against the `main` branch of the main `icn-core` repository.
    *   Provide a clear description of your changes in the PR.
    *   Link any relevant issues.

## Questions?

If you have any questions, don't hesitate to ask in the GitHub issues or (if available) the project's communication channels (e.g., Discord, Matrix).

Thank you for contributing to ICN Core! 