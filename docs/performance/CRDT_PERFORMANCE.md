# CRDT Performance Tuning

This guide provides practical tuning parameters and throughput expectations for the CRDT subsystem. Values are based on development deployments and may change as the implementation matures.

## Recommended Settings

- **Message Batch Size**: 50 updates per gossip round
- **Sync Interval**: 500 ms between CRDT sync rounds
- **State Snapshot Interval**: 5 minutes for durable storage checkpoints
- **Max Delta History**: Keep the last 10,000 operations in memory

These values balance responsiveness with resource usage for small federations (under 20 nodes). Larger deployments may require increased batch sizes and longer snapshot intervals.

## Expected Throughput

On a single node using the default Rust implementation and RocksDB backend:

- ~10k simple counter operations per second
- ~5k map or set operations per second
- Latency for eventual consistency across 10 nodes is typically under 2 seconds

Throughput decreases as object size grows or when persistence is enabled. Monitor disk latency closely when using spinning disks.

## Monitoring Tips

Use the existing Prometheus metrics to track CRDT health:

- `crdt_apply_ops_total` – total operations applied
- `crdt_conflict_resolutions_total` – merges requiring conflict resolution
- `crdt_snapshot_duration_seconds` – time spent writing snapshots

Set alerts if snapshot duration spikes or if the operation rate drops unexpectedly. Grafana dashboards under `monitoring/` include example panels.

