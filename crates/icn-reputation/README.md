# ICN Reputation Crate

This crate provides reputation tracking utilities for the InterCooperative Network (ICN).
It defines the `ReputationStore` trait used by the mesh scheduling logic and a simple
in-memory implementation useful for testing.

`ReputationStore` now exposes `record_proof_attempt` allowing runtimes to
increase or decrease reputation based on zero-knowledge proof verification
results. Successful proofs increment the prover's score while invalid proofs
decrement it, with scores never dropping below zero.

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.
