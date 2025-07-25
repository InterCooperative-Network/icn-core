# CCL Language Server Protocol

> **⚠️ Development Status**: The CCL LSP is functional but under active development. Expect breaking changes.

This document describes how to use the Cooperative Contract Language (CCL) language server, its architecture, and tips for debugging contract development.

## Usage

Run the language server from the `icn-ccl` crate:

```bash
cargo run -p icn-ccl --bin ccl_lsp
```

Configure your editor to connect to the server on the provided port. The server offers syntax highlighting, diagnostics, and contract completion suggestions.

## Architecture Overview

The LSP communicates with the CCL compiler to parse and analyze source files. Diagnostics are generated from the semantic analyzer and returned through standard LSP messages. See `icn-ccl/src/lsp/` for implementation details.

## Debugging Tips

* Run the server with `RUST_LOG=debug` to see internal messages.
* Use the `--stdio` flag to connect via standard input/output.
* If diagnostics seem incorrect, run `cargo test -p icn-ccl tests/test_developer_tooling.rs` to verify expected behavior.

Development contributions are welcome as we stabilize the tooling.
