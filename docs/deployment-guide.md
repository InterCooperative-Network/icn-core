# ICN Deployment Guide

This guide provides minimal examples for launching `icn-node` in common scenarios.

For details on the HTTP API exposed by the node see [API.md](API.md).

## Single Node Local

This mode runs a standalone node for development or testing. By default the node
starts in **production mode** with persistent storage and Ed25519 signing. Use
`--test-mode` or set `ICN_TEST_MODE=true` to launch with stub services and
in-memory storage.

```bash
icn-node --storage-backend memory --http-listen-addr 127.0.0.1:7845 \
         --api-key mylocalkey \
         --tls-cert-path ./cert.pem --tls-key-path ./key.pem
```

Set `ICN_STORAGE_PATH` and related variables to control where persistent DAG
data is written:

```bash
ICN_STORAGE_PATH=./icn_data/node.sled ICN_MANA_LEDGER_PATH=./icn_data/mana.sled \
icn-node --storage-backend sled
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

## Identity

ICN nodes generate a new DID and Ed25519 key on first launch. To supply an
existing key, set the path and passphrase environment variable name or provide
`ICN_NODE_DID_PATH` and `ICN_NODE_PRIVATE_KEY_PATH`:

```toml
[identity]
key_path = "/secrets/node.key.enc"
key_passphrase_env = "ICN_KEY_PASSPHRASE"
```

The passphrase is read from the environment variable referenced by
`key_passphrase_env`. When the plain-text paths are provided the node loads the
DID and private key instead of generating new ones.

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

Configuration values can be provided in the node's TOML file under the `[network]`
section or via environment variables:

```bash
ICN_NETWORK_FAILURE_THRESHOLD=3
ICN_NETWORK_OPEN_TIMEOUT_SECS=5
ICN_NETWORK_RETRY_MAX_ATTEMPTS=3
ICN_NETWORK_RETRY_INITIAL_DELAY_MS=100
ICN_NETWORK_RETRY_MAX_DELAY_MS=1000
```

Environment variables override values from the config file, allowing quick tuning
without editing files.

## Job Retries and Blacklist

Mesh jobs may fail due to executor errors or temporary network issues. The node
will retry a job several times before giving up and will temporarily blacklist
executors that repeatedly fail.

### Configuring Retry Count

Set the maximum attempts in the `[mesh]` section:

```toml
[mesh]
job_retry_count = 5
```

The same value can be provided via environment variable:

```bash
ICN_MESH_JOB_RETRY_COUNT=5
```

### Executor Blacklist

Executors that exceed the failure threshold are banned for a cooldown period.

```toml
[mesh]
blacklist_after_failures = 3
blacklist_cooldown_secs = 600
```

Check the current blacklist using the CLI:

```bash
icn-cli mesh blacklist
```

Refer to [API.md](API.md#mesh-computing-endpoints) for mesh job endpoints and
[TROUBLESHOOTING.md](TROUBLESHOOTING.md#executor-blacklist) for recovery tips.

## Rollback Semantics

When a job execution fails after exhausting retries, the runtime emits a
`RollbackEvent` and restores the previous state. Rollbacks are persisted in the
event store.

Inspect recent rollbacks with:

```bash
icn-cli events --type rollback --tail 20
```

You can also query `/events?type=rollback` via the HTTP API. See
[EVENT_SOURCING.md](EVENT_SOURCING.md) for event store design details.

## Monitoring with Prometheus & Grafana

The devnet includes optional monitoring services. Launch the stack with the
`monitoring` profile to enable Prometheus and Grafana:

```bash
cd icn-devnet
docker-compose --profile monitoring up -d
```

To monitor existing nodes without the devnet, run the standalone stack:
```bash
docker compose -f docker-compose-monitoring.yml up -d
```

Prometheus will be reachable at <http://localhost:9090> and Grafana at
<http://localhost:3000> (`admin` / `icnfederation`). Import the dashboards from
`icn-devnet/grafana/` to visualize node metrics.

Runtime metrics now include counters for WASM resource limiter denials:

```text
wasm_memory_growth_denied_total - memory growth denied by the limiter
wasm_table_growth_denied_total  - table growth denied by the limiter
```

## Runtime Configuration Templates

`RuntimeConfigBuilder` offers a fluent way to construct configuration files in
code. The builder can start from predefined templates found in the
`templates` module.

### Production Configuration

```rust
use icn_runtime::{RuntimeConfigBuilder, templates};

let prod_config = RuntimeConfigBuilder::new()
    .apply_template(templates::production_server)
    .node_did("did:key:z6MkProdNode01")
    .build_unchecked();
prod_config.to_file("production.toml")?;
```

### Isolated Testing Configuration

```rust
use icn_runtime::{RuntimeConfigBuilder, templates};

let test_config = RuntimeConfigBuilder::new()
    .apply_template(templates::isolated_testing)
    .build_unchecked();
test_config.to_file("testing.toml")?;
```

For more advanced composition patterns, see
[`config_builder.rs`](../crates/icn-runtime/examples/config_builder.rs).



## Large Federation Script
For quick experiments with more than ten nodes, use `scripts/deploy_large_federation.sh`.
It generates a temporary compose file with nodes K–T and starts Prometheus/Grafana.

```bash
./scripts/deploy_large_federation.sh
```

See [deployment-automation.md](deployment-automation.md) for Terraform and Ansible examples.
