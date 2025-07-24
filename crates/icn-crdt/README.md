# ICN CRDT

CRDT (Conflict-free Replicated Data Type) implementations for the InterCooperative Network (ICN) real-time synchronization.

This crate provides mathematically guaranteed conflict-free state synchronization across distributed ICN nodes, enabling offline-first operation, multi-node redundancy, and seamless democratic coordination at scale.

## Features

- **G-Counter**: Grow-only counter for mana credits and monotonic values
- **PN-Counter**: Increment/decrement counter for mana balances  
- **OR-Set**: Observed-remove set for group and federation memberships
- **LWW-Register**: Last-writer-wins register for simple attributed values
- **CRDT Map**: Nested CRDT structures for complex state management
- **Vector Clock**: Causality tracking for proper conflict resolution
- **Gossip Protocol**: Efficient state synchronization across network peers

## CRDT Types Mapping

| ICN Feature | CRDT Type | Use Case |
|-------------|-----------|----------|
| Mana Credits | G-Counter | Monotonic mana generation |
| Mana Balances | PN-Counter | Mana spending/earning |
| Group Membership | OR-Set | Add/remove members |
| Federation Membership | OR-Set | Join/leave federations |
| Node Status | LWW-Register | Current node state |
| Reputation Scores | CRDT Map | Per-DID reputation tracking |
| Governance Proposals | CRDT Map | Proposal status and votes |

## Integration

This crate integrates with:
- **ICN DAG**: All CRDT operations are stored as immutable blocks for auditability
- **ICN Runtime**: CRDT operations available via Host ABI for CCL contracts
- **ICN Network**: Gossip protocol for peer-to-peer state synchronization
- **ICN Economics**: CRDT-backed mana ledgers for conflict-free accounting
- **ICN Reputation**: CRDT-backed reputation stores for trust management
- **ICN Governance**: CRDT-backed proposal and voting systems

## Usage

```rust
use icn_crdt::{GCounter, PNCounter, ORSet, VectorClock, NodeId};
use icn_common::Did;
use std::str::FromStr;

// Mana balance tracking
let node_id = NodeId::new("node1".to_string());
let mut mana_counter = PNCounter::new("mana_balance".to_string());
mana_counter.increment(&node_id, 100).unwrap(); // Earn mana
mana_counter.decrement(&node_id, 30).unwrap();  // Spend mana

// Group membership
let mut members = ORSet::new("group_members".to_string(), node_id.clone());
let alice = Did::from_str("did:key:alice").unwrap();
members.add(alice.clone());

// Merge states from different nodes
let other_node = NodeId::new("node2".to_string());
let other_members = ORSet::new("group_members".to_string(), other_node);
members.merge(&other_members);
```