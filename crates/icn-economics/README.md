# ICN Economics Crate

This crate handles the economic protocols of the InterCooperative Network (ICN).

See [CONTEXT.md](../CONTEXT.md) for ICN Core design philosophy and crate roles.

## Purpose

The `icn-economics` crate is responsible for:

*   **Token Models:** Defining and managing the native digital assets of the ICN (e.g., Mana or other utility tokens).
*   **Ledger Management:** Implementing or interfacing with the distributed ledger that records transactions and account balances.
*   **Transaction Logic:** Defining the rules for valid transactions, including transfers, fees, and smart contract interactions related to economic activity.
*   **Incentive Mechanisms:** Potentially including staking, rewards, and other economic incentives for network participation.

This crate is crucial for the sustainable operation and value exchange within the ICN.

## Public API Style

The API style emphasizes:

*   **Security:** Robustness against common financial vulnerabilities.
*   **Accuracy:** Precise and auditable tracking of economic states.
*   **Interoperability:** Clear interfaces for other crates (e.g., `icn-governance`, `icn-runtime`) to interact with economic functions.

### Mana Regeneration

Ledger implementations may support bulk credit operations.
`ManaLedger::credit_all` adds a specified amount to every account and is used by
the runtime to periodically regenerate balances.

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

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 