# Governance Template Onboarding

These templates provide starting points for common cooperative governance models. Each template lives in `icn-ccl/examples/` and can be compiled to WASM using `icn-cli`.

## Available Templates

- `rotating_stewards_template.ccl` – simple rotation schedule for steward roles
- `council_template.ccl` – council decisions based on majority vote
- `assembly_template.ccl` – assembly quorum checking

## Adoption Steps

1. **Review the Template**
   - Inspect the CCL source under `icn-ccl/examples/`.
2. **Compile with the CLI**
   - `cargo run -p icn-cli -- ccl compile icn-ccl/examples/rotating_stewards_template.ccl`
3. **Customize Parameters**
   - Edit the template to fit your cooperative's rules.
4. **Deploy as Governance Policy**
   - Upload the compiled WASM to your federation via proposal.
5. **Iterate and Test**
   - Use the governance pattern library for more examples and ideas.
