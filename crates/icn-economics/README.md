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
- **Computational Mana System**: Mana regeneration directly tied to computational resource contribution
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

#### Computational Mana System - Resource-Based Regeneration
The computational mana system implements the core ICN vision where mana regeneration is directly tied to computational resources contributed by nodes:

```rust
// Configure mana service tied to computational resources
let mana_service = ComputationalManaService::new(
    ComputationalManaConfig {
        base_regeneration_per_hour: 10,
        max_capacity_multiplier: 5.0,
        minimum_contribution_threshold: 100.0,
        federation_pool_factor: 1.0,
        ..Default::default()
    },
    system_info_provider,
    time_provider,
);

// Register node's computational contribution
mana_service.update_node_contribution(
    node_did,
    ComputationalCapacity {
        cpu_cores: 16,
        memory_mb: 32 * 1024, // 32GB
        storage_mb: 2 * 1024 * 1024, // 2TB
        network_mbps: 1000, // 1Gbps
        gpu_compute_units: Some(4),
    },
    0.95, // 95% uptime
    150,  // Jobs completed
    10,   // Jobs failed
    750.0, // Compute hours contributed
).await?;

// Mana regeneration automatically scales with contribution
let regeneration_rate = mana_service.calculate_mana_regeneration_rate(&node_did).await?;
let max_capacity = mana_service.calculate_max_mana_capacity(&node_did).await?;
```

Key features:
- **Resource-Based Scoring**: CPU, memory, storage, network, and GPU contribute to mana generation
- **Reliability Factors**: Uptime and job success rates affect mana allocation
- **Federation Awareness**: Supply/demand across the federation influences individual rates
- **Merit-Based Allocation**: Higher contributors get more mana regeneration and capacity
- **Automatic Capacity Detection**: Real system resources are detected and factored in

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

The ICN mana system now directly correlates mana regeneration with computational resource contribution:

#### Resource-Based Regeneration
All persistent ledger backends expose a bulk credit operation via
`ManaLedger::credit_all`. This method adds a specified amount to every stored
account and is used by the runtime to periodically regenerate balances. The regeneration 
amounts are now calculated by the `ComputationalManaService` based on:

- **Computational Capacity**: CPU cores, memory, storage, network bandwidth, and GPU units
- **Reliability Metrics**: Uptime percentage and job success/failure rates  
- **Historical Contribution**: Total compute hours contributed to the federation
- **Federation Health**: Supply/demand ratio across the federated network

#### Mana Allocation Formula
```
mana_regeneration_rate = base_rate × contribution_factor × federation_factor

Where:
- contribution_factor = (computational_score × reliability_factor × contribution_bonus) / minimum_threshold
- federation_factor = federation_health_factor (higher when supply exceeds demand)
- computational_score = weighted sum of CPU, memory, storage, network, GPU resources
- reliability_factor = uptime_percentage × success_rate
```

Nodes with insufficient computational contribution (below `minimum_contribution_threshold`) 
receive no mana regeneration, ensuring only active resource contributors participate 
in the economic system.

In-memory test ledgers implement the same interface for parity with the on-disk backends.

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