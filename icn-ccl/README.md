# ICN CCL Crate

This crate provides the compiler for the Cooperative Contract Language (CCL) used in the InterCooperative Network (ICN). It translates CCL source into WebAssembly (WASM) modules and produces metadata describing the contract.

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

While minimal, these stages are enough to compile small example contracts.

## Basic Usage

Programmatic compilation is available via `compile_ccl_source_to_wasm`:

```rust
use icn_ccl::compile_ccl_source_to_wasm;

let source = "fn get_cost() -> Mana { return 10; }";
let (wasm, meta) = compile_ccl_source_to_wasm(source)?;
// `wasm` holds the compiled module and `meta` describes exports and other info
```

CLI-oriented helpers live in the `cli` module for tools like `icn-cli` to compile `.ccl` files from disk.

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
