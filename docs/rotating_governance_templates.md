# Rotating Governance Templates

This guide explains how to adopt the rotating steward, council, and assembly templates provided in `icn-ccl` and `icn-templates`.

## Overview

Three example contracts illustrate common cooperative structures where leadership responsibilities rotate on a fixed cycle:

- **Rotating Stewards** – single steward position changes each cycle.
- **Rotating Council** – small council membership rotates through a list of members.
- **Rotating Assembly** – meeting chair rotates among all members.

The source files live under [`icn-ccl/examples/`](../icn-ccl/examples/) and are mirrored in the `icn-templates` crate for programmatic use.

## Using the Templates

1. Copy the desired `.ccl` file into your project or reference the constant from `icn-templates`.
2. Edit member identifiers and cycle lengths to match your cooperative rules.
3. Compile the contract with `icn-cli ccl compile <file>` or using `icn_ccl::compile_ccl_source_to_wasm` in code.
4. Upload the resulting WASM module to your node and submit a proposal referencing it.

## Using the CLI Wizard

The `wizard cooperative` command in `icn-cli` generates a starter file
interactively. It asks for your cooperative name and which rotating pattern you
want. The wizard then writes `<name>_governance.ccl` into the specified
directory, ready for customization and compilation.

The same flow appears in the web UI under the **Formation** page for users who
prefer a graphical interface.

See [governance-pattern-library.md](governance-pattern-library.md) for the full
list of available patterns.
