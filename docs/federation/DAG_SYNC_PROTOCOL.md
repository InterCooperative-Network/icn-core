# Federation DAG Synchronization Protocol

> **⚠️ Development Status**: The federation DAG sync process is evolving and may change in future releases.

Federated nodes exchange content-addressed updates using a directed acyclic graph (DAG) store. Each node maintains a local DAG and periodically syncs with peers.

## Sync Process

1. **Exchange Heads** – Peers share the latest known DAG heads.
2. **Request Missing Blocks** – Nodes request blocks absent from their local store.
3. **Conflict Resolution** – Divergent histories are merged using deterministic rules defined in `crates/icn-dag/src/conflict_resolution.rs`.

This protocol allows autonomous communities to collaborate while keeping local control of data.
