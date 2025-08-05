# ICN Economics Crate

> **✅ Development Status**: Economic protocols, mana systems, and ledger management have comprehensive working implementations. Token operations, transaction validation, cross-cooperative economic features, and advanced automation are fully functional. Ready for security review and production hardening.

This crate handles the economic protocols of the InterCooperative Network (ICN).

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-economics` crate is responsible for:

*   **Token Models:** Defining and managing the native digital assets of the ICN (e.g., Mana or other utility tokens). ✅ **Working**
*   **Ledger Management:** Implementing or interfacing with the distributed ledger that records transactions and account balances. ✅ **Working**
*   **Transaction Logic:** Defining the rules for valid transactions, including transfers, fees, and CCL contract interactions related to economic activity. ✅ **Working**
*   **Incentive Mechanisms:** Economic incentives for network participation, including staking, rewards, and resource allocation. ✅ **Working**
*   **Cross-Cooperative Coordination:** Economic protocols for resource sharing and coordination between different cooperative federations. ✅ **Working**
*   **Economic Automation:** Advanced algorithms for dynamic pricing, resource allocation optimization, and policy enforcement. ✅ **Working**

This crate provides the economic foundation for sustainable operation and value exchange within the ICN.

## Current Implementation Status

### ✅ Fully Working Features
- **Resource Token Operations**: Complete mint/transfer/burn lifecycle with proper balance tracking
- **Token Class Management**: Creation and management of fungible and non-fungible token classes
- **Mana System Integration**: Balance tracking, spending validation, and regeneration across multiple backends
- **Cross-Cooperative Marketplace**: Full marketplace functionality with trust-based inter-federation trading
- **Multiple Storage Backends**: File, Sled, SQLite, RocksDB implementations with production features
- **Mutual Aid Tokens**: Helper functions for community support credits
- **Transaction Validation**: Comprehensive validation infrastructure with balance checks and policy enforcement
- **Economic Automation Engine**: Advanced automation with policy enforcement, health monitoring, and optimization
- **Predictive Pricing Models**: Sophisticated pricing algorithms with trend analysis and economic health factors
- **Federation Economic Coordination**: Multi-federation resource sharing with trust management
- **Economic Health Monitoring**: Real-time health metrics, alerting, and comprehensive analytics

### 🔨 Production Hardening In Progress
- **Security Review**: Cryptographic implementations and economic policy validation
- **Scale Testing**: Validation under production-scale transaction volumes
- **Advanced Economic Policies**: Complex incentive mechanisms and governance integration
- **Cross-Federation Protocol**: Network protocol standardization for federation coordination

## Public API Style

The API style emphasizes:

*   **Security:** Robustness against common financial vulnerabilities and economic attacks.
*   **Accuracy:** Precise and auditable tracking of economic states with comprehensive validation.
*   **Interoperability:** Clear interfaces for other crates (e.g., `icn-governance`, `icn-runtime`) to interact with economic functions.
*   **Cross-Cooperative:** Native support for multi-federation economic coordination and resource sharing.

### Advanced Economic Features

#### Cross-Cooperative Resource Sharing
```rust
// Register federation for economic coordination
engine.register_federation(
    "partner-cooperative".to_string(),
    resource_inventory,
)?;

// Create cross-cooperative resource request
let request_id = engine.create_cross_cooperative_request(
    "gpu-compute".to_string(),    // Resource type
    1000,                         // Amount needed
    15.0,                         // Max price per unit
    0.9,                          // Urgency level
    0.7,                          // Min trust level required
    24,                           // Duration in hours
).await?;
```

#### Economic Optimization
```rust
// Run comprehensive economic optimization
let optimization_result = engine.run_economic_optimization().await?;

// Get cross-cooperative statistics
let stats = engine.get_cross_cooperative_stats();
println!("Active federations: {}", stats["federation_count"]);
println!("Economic health: {:.2}", stats["last_objective_value"]);
```

#### Advanced Pricing Strategies
```rust
// Configure sophisticated pricing for cross-cooperative trades
let pricing_policy = CrossCooperativePolicy {
    pricing_strategy: CrossCooperativePricingStrategy::MarketWithTrustDiscount {
        base_markup: 1.2,           // 20% markup over cost
        trust_discount: 0.1,        // Up to 10% discount for trusted partners
    },
    min_trust_level: 0.5,           // Require moderate trust
    max_resource_share: 0.3,        // Share up to 30% of resources
    auto_approval_threshold: 1000,  // Auto-approve small requests
    local_priority_weight: 1.5,     // Prefer local requests
};
```

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