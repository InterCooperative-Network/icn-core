# ICN Economics Crate

> **⚠️ Development Status**: Economic protocols, mana systems, and ledger management contain stub implementations. Transaction logic and incentive mechanisms need development work.

This crate handles the economic protocols of the InterCooperative Network (ICN).

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-economics` crate is responsible for:

*   **Token Models:** Defining and managing the native digital assets of the ICN (e.g., Mana or other utility tokens).
*   **Ledger Management:** Implementing or interfacing with the distributed ledger that records transactions and account balances.
*   **Transaction Logic:** Defining the rules for valid transactions, including transfers, fees, and CCL contract interactions related to economic activity.
*   **Incentive Mechanisms:** Potentially including staking, rewards, and other economic incentives for network participation.

This crate is crucial for the sustainable operation and value exchange within the ICN.

## Public API Style

The API style emphasizes:

*   **Security:** Robustness against common financial vulnerabilities.
*   **Accuracy:** Precise and auditable tracking of economic states.
*   **Interoperability:** Clear interfaces for other crates (e.g., `icn-governance`, `icn-runtime`) to interact with economic functions.

### Mana Regeneration

All persistent ledger backends expose a bulk credit operation via
`ManaLedger::credit_all`. This method adds a specified amount to every stored
account and is used by the runtime to periodically regenerate balances. In-memory
test ledgers implement the same interface for parity with the on-disk backends.

## Feature Flags

Persistence backends are selected via Cargo features. The default backend uses
[`sled`](https://crates.io/crates/sled) for a simple embedded database.

- `persist-sled` *(default)* – store the mana ledger using sled.
- `persist-rocksdb` – store the ledger using RocksDB. Enable with:

```bash
cargo build --features persist-rocksdb
```

When using RocksDB at runtime, pass `--mana-ledger-backend rocksdb` and a path
ending in `.rocks` to the node binary.

## Mutual Aid Tokens

This crate provides helper functions `grant_mutual_aid` and `use_mutual_aid` for
minting and burning non-transferable credits under the `mutual_aid` token class.
These credits facilitate community support but cannot be transferred between
accounts.

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 