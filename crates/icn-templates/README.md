# ICN Templates Crate

This crate packages common Cooperative Contract Language (CCL) patterns that cooperatives can use as starting points for their own bylaws.

Templates are provided as plain CCL source files and exposed through constants for programmatic access. Contracts can be compiled with the `icn-ccl` compiler and modified to suit local governance rules.

## Included Templates

- `simple_voting.ccl` – minimal majority voting procedure
- `treasury_rules.ccl` – example treasury withdrawal policy
- `federation_membership_proof.ccl` – verify federation membership via ZK proof

Use one of the constants such as `icn_templates::SIMPLE_VOTING` or `icn_templates::FEDERATION_MEMBERSHIP_PROOF` to retrieve the source text.

```
use icn_templates::SIMPLE_VOTING;
let wasm = icn_ccl::compile_ccl_source_to_wasm(SIMPLE_VOTING)?;
```

Cooperatives are encouraged to copy these files and adapt them as needed.
