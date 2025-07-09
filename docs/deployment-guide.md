# ICN Deployment Guide

This guide provides minimal examples for launching `icn-node` in common scenarios.

For details on the HTTP API exposed by the node see [API.md](API.md).

## Single Node Local

This mode runs a standalone node for development or testing.

```bash
icn-node --storage-backend memory --http-listen-addr 127.0.0.1:7845 \
         --api-key mylocalkey \
         --tls-cert-path ./cert.pem --tls-key-path ./key.pem
```

Providing certificate and key paths makes the server listen on HTTPS instead of HTTP.

A sample TOML configuration is in `configs/single_node.toml`.
You can also supply these values via configuration or environment variables:

```toml
http_listen_addr = "127.0.0.1:7845"
api_key = "mylocalkey"
tls_cert_path = "./cert.pem"
tls_key_path = "./key.pem"
```

To use RocksDB as the persistence layer, build `icn-node` with the
`persist-rocksdb` feature and set `storage_backend = "rocksdb"`.

## Small Federation

For a small group of cooperating nodes, each node may use a persistent store and
bootstrap to known peers.

```bash
icn-node --storage-backend sled --storage-path ./icn_data/node1.sled \
         --bootstrap-peers /ip4/1.2.3.4/tcp/7000/p2p/QmPeer \
         --api-key node1secret --open-rate-limit 0 \
         --tls-cert-path ./cert.pem --tls-key-path ./key.pem
```

See `configs/small_federation.toml` for an example configuration file.
A configuration file might contain:

```toml
http_listen_addr = "0.0.0.0:7845"
storage_backend = "sled"
storage_path = "./icn_data/node1.sled"
bootstrap_peers = ["/ip4/1.2.3.4/tcp/7000/p2p/QmPeer"]
api_key = "node1secret"
tls_cert_path = "./cert.pem"
tls_key_path = "./key.pem"
```

## DAG Backup and Restore

Use `icn-cli` to export and re-import the node's DAG storage. The same commands
work with any storage backend. After restoring, run `icn-cli dag verify` to
confirm integrity.

### RocksDB

```bash
# Node configured with RocksDB
icn-node --storage-backend rocksdb --storage-path ./icn_data/rocksdb

# Backup DAG data
icn-cli dag backup --path ./backups/rocksdb

# Restore and verify
icn-cli dag restore --path ./backups/rocksdb
icn-cli dag verify
```

### Sled

```bash
# Node configured with Sled
icn-node --storage-backend sled --storage-path ./icn_data/node1.sled

# Backup DAG data
icn-cli dag backup --path ./backups/sled

# Restore and verify
icn-cli dag restore --path ./backups/sled
icn-cli dag verify
```

### SQLite

```bash
# Node configured with SQLite
icn-node --storage-backend sqlite --storage-path ./icn_data/dag.sqlite

# Backup DAG data
icn-cli dag backup --path ./backups/sqlite

# Restore and verify
icn-cli dag restore --path ./backups/sqlite
icn-cli dag verify
```

### Postgres

```bash
# Node configured with Postgres
icn-node --storage-backend postgres --storage-path postgres://user:pass@localhost/icn_dag

# Backup DAG data
icn-cli dag backup --path ./backups/postgres

# Restore and verify
icn-cli dag restore --path ./backups/postgres
icn-cli dag verify
```

## Circuit Breaker and Retry

The node automatically wraps outbound network calls in a circuit breaker and retry helper. These mechanisms prevent cascading failures when peers become unreachable.

### Circuit Breaker

When a request fails repeatedly, the circuit opens and blocks further attempts for a period of time. The following options control its behaviour:

```toml
failure_threshold = 3      # errors before opening the circuit
open_timeout_secs = 5      # time to wait before a trial request
```

Increase `failure_threshold` or the timeout in noisy environments; decrease them to fail fast.

### Retry with Backoff

Operations use jittered exponential backoff retries. Tune them via:

```toml
retry_max_attempts = 3     # number of tries before giving up
retry_initial_delay_ms = 100
retry_max_delay_ms = 1000
```

These values control the helper used across HTTP and P2P operations.

## Monitoring with Prometheus & Grafana

The devnet includes optional monitoring services. Launch the stack with the
`monitoring` profile to enable Prometheus and Grafana:

```bash
cd icn-devnet
docker-compose --profile monitoring up -d
```

Prometheus will be reachable at <http://localhost:9090> and Grafana at
<http://localhost:3000> (`admin` / `icnfederation`). Import the dashboards from
`icn-devnet/grafana/` to visualize node metrics.


