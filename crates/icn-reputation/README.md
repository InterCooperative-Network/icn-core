# ICN Reputation Crate

This crate provides reputation tracking utilities for the InterCooperative Network (ICN).
It defines the `ReputationStore` trait used by the mesh scheduling logic and ships with
both in-memory and persistent backends.

## Scoring Formula

Mesh executor bids are scored using the formula implemented in `icn-mesh`:

```text
score = w_price * price_score + w_rep * reputation_score + w_res * resource_score
```

Where:

* `price_score` = `1000 / bid.price_mana`
* `reputation_score` = value returned by `ReputationStore::get_reputation`
* `resource_score` = CPU cores + (memory_mb / 1024)

The default weights are `w_price = 1.0`, `w_rep = 50.0` and `w_res = 1.0`.

Executors gain reputation each time a valid execution receipt is recorded.

## Available Backends

* `InMemoryReputationStore` – fast, non-persistent store ideal for tests.
* `SledReputationStore` – persistent backend backed by a [`sled`](https://github.com/spacejam/sled) database (enabled by the `persist-sled` feature).
