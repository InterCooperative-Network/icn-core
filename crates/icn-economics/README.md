# ICN Economics Crate

> **✅ Development Status**: Economic protocols, mana systems, and ledger management have working implementations. Token operations, transaction validation, and cross-cooperative economic features are functional. Production hardening and security review needed.

This crate handles the economic protocols of the InterCooperative Network (ICN).

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-economics` crate is responsible for:

*   **Token Models:** Defining and managing the native digital assets of the ICN (e.g., Mana or other utility tokens). ✅ **Working**
*   **Ledger Management:** Implementing or interfacing with the distributed ledger that records transactions and account balances. ✅ **Working**
*   **Transaction Logic:** Defining the rules for valid transactions, including transfers, fees, and CCL contract interactions related to economic activity. ✅ **Working**
*   **Incentive Mechanisms:** Economic incentives for network participation, including staking, rewards, and resource allocation. 🔨 **In Development**

This crate provides the economic foundation for sustainable operation and value exchange within the ICN.

## Current Implementation Status

### ✅ Working Features
- **Resource Token Operations**: Complete mint/transfer/burn lifecycle with proper balance tracking
- **Token Class Management**: Creation and management of fungible and non-fungible token classes
- **Mana System Integration**: Balance tracking, spending validation, and regeneration across multiple backends
- **Cross-Cooperative Support**: Token operations work across different cooperative scopes
- **Multiple Storage Backends**: File, Sled, SQLite, RocksDB implementations
- **Mutual Aid Tokens**: Helper functions for community support credits
- **Transaction Validation**: Basic validation infrastructure with balance checks

### 🔨 In Development
- **Advanced Economic Automation**: Sophisticated pricing algorithms and resource allocation
- **Enhanced Metrics Collection**: Comprehensive monitoring and analytics
- **Production Security**: Cryptographic review and hardening
- **Complex Economic Policies**: Advanced incentive mechanisms and governance integration

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