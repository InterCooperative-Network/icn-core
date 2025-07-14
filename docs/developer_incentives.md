# Developer Incentives

This guide explains how to reward open source contributors using the governance templates shipped with `icn-core`.

## Usage

1. Pick a template under `crates/icn-governance/templates/`:
   - `dao_reward_issuance.ccl` issues transferable DAO reward tokens.
   - `contributor_recognition.ccl` mints a non-transferable recognition badge.
2. Compile the chosen template:
   ```bash
   icn-cli ccl compile dao_reward_issuance.ccl
   ```
3. Submit a proposal referencing the compiled module via your governance tooling.
4. Once approved, the runtime executes the policy and distributes tokens.

Modify these templates or mana costs to match your cooperative's incentive policy.
