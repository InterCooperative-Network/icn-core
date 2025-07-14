# Cooperative Governance Onboarding

This short guide walks cooperatives through setting up their first CCL-based governance policy using the provided templates.

## 1. Review the Templates

Browse [`icn-ccl/examples/governance_templates/`](../icn-ccl/examples/governance_templates/) for common patterns:

- `simple_voting.ccl` – minimal majority voting
- `treasury_rules.ccl` – example spending policy
- `rotating_stewards.ccl` – single steward rotates each cycle
- `rotating_council.ccl` – council membership rotates
- `rotating_assembly.ccl` – meeting chair rotates among members

Copy the template that best matches your cooperative structure and customize member identifiers or parameters as needed.

## 2. Compile the Contract

Use `icn-cli` to compile your CCL file to WASM:

```bash
icn-cli ccl compile my_policy.ccl -o my_policy.wasm
```

## 3. Upload and Propose

1. Upload the compiled module to your node's DAG service:
   ```bash
   curl -X POST http://localhost:5001/dag/put \
     --data-binary '@my_policy.wasm'
   ```
2. Generate a job spec and submit the proposal:
   ```bash
   generate_ccl_job_spec --wasm-cid <cid> --output policy_job.json
   curl -X POST http://localhost:5001/governance/propose \
     -H 'Content-Type: application/json' \
     -d @policy_job.json
   ```

For additional patterns see [governance-pattern-library.md](governance-pattern-library.md).
