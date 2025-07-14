# Cooperative Formation Onboarding

This guide explains how to bootstrap a new cooperative using the governance templates provided with ICN Core.

## 1. Choose a Governance Template

Templates live under [`icn-ccl/examples/`](../icn-ccl/examples/):

- `rotating_stewards.ccl` – weekly steward rotation
- `cooperative_council.ccl` – small council voting
- `general_assembly.ccl` – one-member-one-vote assembly

Review these examples and decide which pattern best fits your community. You can modify the files to suit local bylaws.

## 2. Run the Formation Wizard

The CLI includes a simple wizard to copy one of these templates into your project:

```bash
icn-cli wizard cooperative-formation
```

The wizard prompts for a template and writes `governance.ccl` in the current directory. Edit this file and compile it with `icn-cli ccl compile` when ready.

## 3. Next Steps

After compiling your contract you can upload the WASM module to an ICN node and begin proposing actions governed by your new policy.
