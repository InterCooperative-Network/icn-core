# ICN Reputation System

The InterCooperative Network (ICN) reputation system provides comprehensive trust management and reputation scoring for cooperative networks. This crate implements sophisticated trust graph algorithms for calculating, propagating, and aggregating trust relationships between cooperatives.

## Features

### Core Reputation Tracking
- **ReputationStore trait**: Abstract interface for storing and retrieving reputation scores
- **Execution tracking**: Record successful/failed job executions and their impact on reputation
- **Proof verification tracking**: Track zero-knowledge proof attempts and outcomes
- **Multiple storage backends**: In-memory, Sled, SQLite, and RocksDB implementations

### Advanced Trust Graph System
- **Trust Graph Data Structures**: Efficient representation of trust relationships between cooperatives
- **Trust Score Calculation Engine**: PageRank-style algorithms and weighted trust propagation
- **Trust Path Discovery**: Find optimal trust paths between cooperatives through intermediaries
- **Trust Decay Models**: Time-based and distance-based trust degradation
- **Trust Aggregation**: Combine multiple trust signals into composite scores

## Quick Start

```rust
use icn_reputation::{TrustGraph, TrustEdge, TrustCalculationEngine};
use icn_common::{Did, FixedTimeProvider};
use std::str::FromStr;

// Create trust graph
let mut graph = TrustGraph::new();
let time_provider = FixedTimeProvider::new(1640995200);

let coop_a = Did::from_str("did:icn:coop:food-collective").unwrap();
let coop_b = Did::from_str("did:icn:coop:tech-workers").unwrap();

// Add trust relationship
let trust_edge = TrustEdge::new(coop_a.clone(), coop_b.clone(), 0.85, 1640995200);
graph.add_edge(trust_edge);

// Calculate trust scores
let engine = TrustCalculationEngine::new();
let scores = engine.calculate_pagerank_scores(&graph, &time_provider);

println!("Trust scores: {:?}", scores);
```

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.
