# ICN Templates Crate

This crate packages common Cooperative Contract Language (CCL) patterns that cooperatives can use as starting points for their own bylaws.

Templates are provided as plain CCL source files and exposed through constants for programmatic access. Contracts can be compiled with the `icn-ccl` compiler and modified to suit local governance rules.

## Included Templates

- `simple_voting.ccl` – minimal majority voting procedure
- `treasury_rules.ccl` – example treasury withdrawal policy
- `rotating_stewards.ccl` – rotating single steward each cycle
- `rotating_council.ccl` – rotating council subset
- `rotating_assembly.ccl` – rotating meeting chair for assemblies

Use `icn_templates::SIMPLE_VOTING` or `icn_templates::TREASURY_RULES` to retrieve the source text.

```
use icn_templates::SIMPLE_VOTING;
let wasm = icn_ccl::compile_ccl_source_to_wasm(SIMPLE_VOTING)?;
```

Cooperatives are encouraged to copy these files and adapt them as needed.
