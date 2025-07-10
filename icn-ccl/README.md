# ICN CCL Crate

This crate provides the compiler for the Cooperative Contract Language (CCL) used in the InterCooperative Network (ICN). It translates CCL source into WebAssembly (WASM) modules and produces metadata describing the contract.

See [CONTEXT.md](../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-ccl` crate is responsible for:

* Parsing and semantically analyzing CCL source files.
* Optimizing the contract's abstract syntax tree.
* Generating WASM bytecode and accompanying metadata.
* Offering helper functions consumed by CLI tools and other crates.

## Development Status

The compiler is functional and includes:

* A parser built with [Pest](https://pest.rs)
* A semantic analyzer with basic type checking
* A simple optimizer for constant folding
* A WASM backend built on `wasm-encoder`

These components allow compiling example contracts to WASM.

## Basic Usage

Programmatic compilation is available via `compile_ccl_source_to_wasm`:

```rust
use icn_ccl::compile_ccl_source_to_wasm;

let source = "fn get_cost() -> Mana { return 10; }";
let (wasm, meta) = compile_ccl_source_to_wasm(source)?;
// `wasm` holds the compiled module and `meta` describes exports and other info
```

CLI-oriented helpers live in the `cli` module for tools like `icn-cli` to compile `.ccl` files from disk.

Real-world sample contracts are available under [`examples/`](examples/) and demonstrate features like `rule` statements and `while` loops.

### End-to-End Workflow

1. **Write a CCL policy** – author a `.ccl` file describing the desired logic.
2. **Compile to WASM** – use `compile_ccl_source_to_wasm` or the CLI helpers to
   produce a `.wasm` module and metadata JSON.
3. **Store in the DAG** – anchor the compiled module as a `DagBlock` so it can
   be referenced by a mesh job's `manifest_cid`.
4. **Submit a mesh job** – create an `ActualMeshJob` whose `manifest_cid` points
   at the stored module and set `JobKind::CclWasm`.
5. **Execution** – an executor node loads the module via `icn-runtime` and runs
   its `run` export. The resulting `ExecutionReceipt` is anchored back to the
   DAG.

### Example: Compile and Submit with `icn-cli`

With a node running and exposing the HTTP API you can compile a contract and
submit it for execution using the CLI:

```bash
# Compile locally to `policy.wasm` and `policy.json`
cargo run -p icn-cli -- ccl compile policy.ccl

# Upload the compiled module and obtain its CID
cargo run -p icn-cli -- --api-url http://localhost:7845 compile-ccl policy.ccl

# Submit a mesh job referencing the returned CID
cargo run -p icn-cli -- --api-url http://localhost:7845 submit-job \
  '{"manifest_cid":"CID_FROM_UPLOAD","spec_json":{},"cost_mana":0}'
```

### Included Governance Examples

Several example contracts live in `tests/contracts/`:

* `proposal_flow.ccl` – illustrates proposal creation, voting and finalization.
* `voting_logic.ccl` – demonstrates an open/cast/close voting sequence.

These files can be compiled with `compile_ccl_file_to_wasm` and executed using
the `WasmExecutor` as shown in the integration tests.

Additional real‑world policies are provided in [`examples/`](examples/). The
`worker_time_tracking.ccl` file illustrates loops and array usage while
`community_energy_sharing.ccl` demonstrates `rule` actions such as `charge`.

## Mana Policies

The repository includes example economic logic that manipulates mana balances. A
full guide to implementing and deploying such policies is available in
[docs/mana_policies.md](../docs/mana_policies.md).

## Integration with Other Crates

* Compiled WASM is executed inside [`icn-runtime`](../crates/icn-runtime/README.md).
* Contract metadata relies on types from [`icn-common`](../crates/icn-common/README.md).
* Higher level protocols in [`icn-protocol`](../crates/icn-protocol/README.md) may reference CCL contracts.

## Roadmap & Issues

Future work focuses on expanding language features and improving optimization. For specific tasks, check the [open issues](https://github.com/InterCooperative/icn-core/issues?q=label%3Accl).

## Contributing

Contributions are welcome! Please see the root [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../LICENSE).
