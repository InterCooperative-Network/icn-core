# ICN Mesh Crate

This crate focuses on job orchestration, scheduling, and execution within the InterCooperative Network (ICN) mesh network.

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-mesh` crate is responsible for:

*   **Job Definition:** Defining how tasks or jobs are described and submitted to the mesh network.
*   **Resource Discovery:** Finding nodes within the mesh that have the capacity and capabilities to execute specific jobs.
*   **Scheduling:** Assigning jobs to appropriate nodes based on factors like load, locality, or specific requirements.
*   **Execution Management:** Overseeing the lifecycle of jobs, including starting, monitoring, and handling failures or results.
*   **Fault Tolerance:** Ensuring that jobs can be completed reliably even if some nodes in the mesh fail or become unavailable.

This crate enables distributed computation and service execution across the ICN.

## Public API Style

The API style emphasizes:

*   **Reliability:** Robust job execution and fault handling.
*   **Scalability:** Ability to manage a large number of jobs and nodes.
*   **Efficiency:** Optimal use of network resources for job execution.
*   **Extensibility:** Allowing different types of jobs and scheduling algorithms to be supported.

## Scoring Algorithm & Reputation

Job bids are evaluated by `select_executor`, which calls `score_bid` for each
bid. The score is a weighted combination of the bid price, the executor's
advertised performance, and a reputation value supplied by the reputation
module. The bid with the highest score is chosen as the executor.

The reputation module monitors execution receipts and updates each executor's
score over time. `select_executor` queries this module (via the
`ReputationExecutorSelector` helper) so that nodes with proven reliability are
favored in future selections.

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 