# CRDT Synchronization

> **⚠️ Development Status**: These notes describe real-time synchronization plans. Implementations are experimental.

ICN Core uses Conflict-free Replicated Data Types (CRDTs) to synchronize state across federation nodes. Gossip-based protocols exchange CRDT updates to eventually reach convergence.

## Real-Time Sync Protocols

1. **Gossip Broadcast** – Nodes periodically broadcast deltas to peers using libp2p gossipsub.
2. **State Vector Exchange** – Peers compare vector clocks to request missing operations.
3. **Compression** – Batch multiple operations into a single message for efficiency.

The CRDT modules live in `crates/icn-crdt/` and integrate with the DAG store for persistence.

Further improvements will focus on reducing bandwidth and handling network partitions gracefully.
