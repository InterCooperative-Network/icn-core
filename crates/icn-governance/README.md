# ICN Governance Crate

This crate defines the mechanisms for network governance within the InterCooperative Network (ICN).

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-governance` crate is responsible for:

*   **Proposal Systems:** Defining how proposals for network changes (e.g., protocol upgrades, parameter adjustments, funding) are submitted and managed.
*   **Voting Procedures:** Implementing the logic for how stakeholders vote on proposals, including vote counting and threshold determination.
*   **Quorum Logic:** Defining the requirements for a vote to be considered valid (e.g., minimum participation).
*   **Decision Execution:** Potentially interfacing with other crates to enact decisions once they are approved through governance.
*   **Role Management:** Managing roles and permissions related to governance participation.

This crate is essential for the decentralized control and evolution of the ICN.

## Public API Style

The API style emphasizes:

*   **Transparency:** Clear and auditable governance processes.
*   **Fairness:** Ensuring that voting and proposal mechanisms are equitable.
*   **Flexibility:** Allowing for different governance models or parameters to be configured.
*   **Interoperability:** Providing clear interfaces for other crates (e.g., `icn-cli`, `icn-node`) to interact with governance functions.
*   **Persistence:** With the `persist-sled` feature, proposals and votes are stored using `Sled`, enabling recovery across restarts.

## Federation Sync

When built with the `federation` feature, this crate exposes
`request_federation_sync`, an async helper that uses a provided
`NetworkService` to request state from another peer. It sends a
`FederationSyncRequest` message via the network layer and returns a
`CommonError` if the underlying send fails.
## DAO Reward Templates

Reusable CCL snippets for rewarding contributors live in `templates/`. Examples:
- `reward_member.ccl` — grant scoped tokens to a specific member.
- `community_bonus.ccl` — distribute a bonus to all members.
- `dao_reward_issuance.ccl` — issue DAO reward tokens to members.
- `contributor_recognition.ccl` — mint a recognition badge for a contributor.


## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 