# ICN Reputation Crate

This crate provides reputation tracking utilities for the InterCooperative Network (ICN).
It defines the `ReputationStore` trait used by the mesh scheduling logic and a simple
in-memory implementation useful for testing.

`record_proof_attempt` is provided to track zero-knowledge proof verifications.
`host_verify_zk_proof` and `host_verify_zk_revocation_proof` call this method to
reward or penalize provers.

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.
