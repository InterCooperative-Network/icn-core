# Integration Tests

This directory contains integration scenarios that exercise multiple ICN nodes and
related tooling. Tests rely on the federation devnet provided in `icn-devnet`.

## Running Tests

Start by ensuring Docker is available. Most scenarios will automatically launch
the devnet using `ensure_devnet`. Simply run all tests:

```bash
cargo test --all-features --workspace -p icn-integration-tests
```

Each test will spin up the devnet if it is not already running and tear it down
at the end.

### Network Resilience

`network_resilience.rs` contains cases that stop and start nodes to verify
operation recovery. The `test_long_partition_circuit_breaker` scenario simulates
a longer network partition, confirms that circuit breakers open after repeated
failures, that exponential backoff occurs during retries, and that normal
operation resumes once the node is restarted.

These tests may take several minutes because they wait for retry backoff and
network convergence. Use `ICN_DEVNET_RUNNING=1` to run them against an existing
deployment.

