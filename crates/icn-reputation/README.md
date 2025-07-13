# ICN Reputation Crate

This crate provides reputation tracking utilities for the InterCooperative Network (ICN).
It defines the `ReputationStore` trait used by the mesh scheduling logic and a simple
in-memory implementation useful for testing.

`ReputationStore` records both execution results and zero-knowledge proof
attempts. Successful proofs increase a prover's score while invalid proofs
reduce it.

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.
