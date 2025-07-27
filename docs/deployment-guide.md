# ICN Core Deployment Guide

> **Complete deployment guide for ICN nodes from development to production**

This guide provides comprehensive instructions for deploying ICN nodes across different environments, from local development to production federations.

**Status**: ICN Core has substantial working implementations (~65-75% complete) but requires security review and operational hardening before production use.

## 📖 Guide Overview

- [Development Setup](#development-setup) - Local testing and development
- [Single Node Deployment](#single-node-deployment) - Standalone node deployment  
- [Federation Deployment](#federation-deployment) - Multi-node federation setup
- [Production Deployment](#production-deployment) - Security-hardened production setup
- [Monitoring & Operations](#monitoring--operations) - Operational considerations
- [Backup & Recovery](#backup--recovery) - Data protection and disaster recovery

---

## 🚀 Development Setup

### Quick Start (5 Minutes)

```bash
# Clone and build
git clone https://github.com/InterCooperative-Network/icn-core
cd icn-core
just setup && just build

# Start development node
cargo run --bin icn-node -- --test-mode

# Test basic functionality
cargo run --bin icn-cli -- info
```

### Development Features

The development setup includes:
- **In-memory storage** (no persistence)
- **Stub services** for testing
- **Simplified authentication** 
- **Built-in test data**

### Development Configuration

```toml
# configs/development.toml
[runtime]
environment = "development"
test_mode = true

[network]
http_listen_addr = "127.0.0.1:7845"
p2p_enabled = false

[storage]
backend = "memory"

[identity]
generate_new_key = true

[api]
api_key = "dev-key-123"
require_auth = false
```

---

## 🏠 Single Node Deployment

### Local Production Mode

For local testing with persistent storage and real cryptography:

```bash
# Start with persistent storage
icn-node --storage-backend sled \
         --storage-path ./icn_data/node.sled \
         --http-listen-addr 127.0.0.1:7845 \
         --api-key "$(openssl rand -hex 32)"
```

### Storage Backend Options

ICN supports multiple storage backends:

| Backend | Use Case | Features | Build Requirement |
|---------|----------|----------|-------------------|
| `memory` | Development/Testing | Fast, ephemeral | Default |  
| `sled` | Single node production | Embedded, reliable | Default |
| `rocksdb` | High performance | Fast, concurrent | `--features persist-rocksdb` |
| `postgresql` | Multi-node federation | ACID, distributed | Default |
| `sqlite` | Lightweight production | SQL, portable | Default |

### Example Storage Configurations

**Sled (Recommended for single nodes)**:
```bash
ICN_STORAGE_PATH=./icn_data/node.sled \
ICN_MANA_LEDGER_PATH=./icn_data/mana.sled \
icn-node --storage-backend sled
```

**RocksDB (High performance)**:
```bash
# Build with RocksDB support
cargo build --features persist-rocksdb --bin icn-node

# Run with RocksDB
icn-node --storage-backend rocksdb \
         --storage-path ./icn_data/rocksdb
```

**PostgreSQL (Federation ready)**:
```bash
# Configure database connection
icn-node --storage-backend postgresql \
         --db-url "postgresql://user:pass@localhost/icn"
```

### Identity Management

ICN nodes generate a new DID and Ed25519 key on first launch. For persistent identity:

```toml
[identity]
# Use existing key
key_path = "/secrets/node.key.enc"
key_passphrase_env = "ICN_KEY_PASSPHRASE"

# Generate new key (default)
generate_new_key = true
```

---

## 🌐 Federation Deployment

### Multi-Node Federation Setup

A federation consists of multiple ICN nodes that coordinate through P2P networking for governance, mesh computing, and resource sharing.

### Bootstrap Node (Node A)

```bash
# Start the bootstrap node
icn-node --storage-backend postgresql \
         --db-url "postgresql://localhost/icn_node_a" \
         --http-listen-addr "0.0.0.0:7845" \
         --p2p-listen-addr "/ip4/0.0.0.0/tcp/7000" \
         --api-key "$(cat /secrets/api-key-a)" \
         --tls-cert-path "/secrets/cert.pem" \
         --tls-key-path "/secrets/key.pem"
```

### Worker Nodes (Node B, C, ...)

```bash
# Node B - connects to bootstrap node
icn-node --storage-backend postgresql \
         --db-url "postgresql://localhost/icn_node_b" \
         --http-listen-addr "0.0.0.0:7846" \
         --p2p-listen-addr "/ip4/0.0.0.0/tcp/7001" \
         --bootstrap-peers "/ip4/bootstrap-ip/tcp/7000/p2p/QmBootstrapPeerId" \
         --api-key "$(cat /secrets/api-key-b)" \
         --tls-cert-path "/secrets/cert.pem" \
         --tls-key-path "/secrets/key.pem"
```

### Federation Configuration Example

```toml
# configs/federation_node.toml
[runtime]
environment = "production"

[network]
http_listen_addr = "0.0.0.0:7845"
p2p_enabled = true
p2p_listen_addr = "/ip4/0.0.0.0/tcp/7000"
bootstrap_peers = [
    "/ip4/10.0.1.100/tcp/7000/p2p/QmBootstrapPeer",
    "/ip4/10.0.1.101/tcp/7000/p2p/QmOtherPeer"
]

[storage]
backend = "postgresql"
connection_string = "postgresql://icn_user:secure_password@db.internal/icn"

[identity]
key_path = "/secrets/node_identity.key"
key_passphrase_env = "ICN_KEY_PASSPHRASE"

[api]
api_key_env = "ICN_API_KEY"
tls_cert_path = "/secrets/tls.crt"
tls_key_path = "/secrets/tls.key"
require_auth = true
```

### Automated Federation Setup

Use the containerized devnet for automated federation testing:

```bash
# 3-node federation
cd icn-devnet
./launch_federation.sh

# 10-node load testing
NUM_JOBS=50 ./scripts/run_10node_devnet.sh
```

---

## 🏭 Production Deployment

### Security Requirements

⚠️ **Critical**: ICN requires security review before production use. Current gaps include:

- Cryptographic implementation audit needed
- Production monitoring and alerting required  
- Scale testing beyond development environments
- Operational runbooks and incident response procedures

### Production Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Load Balancer │    │   Monitoring    │    │   Backup/DR     │
│   (HAProxy/     │    │   (Prometheus/  │    │   (S3/GCS/      │
│    Nginx)       │    │    Grafana)     │    │    Azure)       │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
    ┌─────▼──────────────────────▼──────────────────────▼─────┐
    │                 ICN Federation Network                 │
    └─────┬──────────────┬─────────────────┬─────────────────┘
          │              │                 │
┌─────────▼───────┐ ┌────▼────────┐ ┌──────▼──────────┐
│   ICN Node A    │ │ ICN Node B  │ │   ICN Node C    │
│  (Bootstrap)    │ │  (Worker)   │ │   (Worker)      │
│                 │ │             │ │                 │
│ PostgreSQL DB   │ │ PostgreSQL  │ │ PostgreSQL DB   │
│ Redis Cache     │ │ DB          │ │ Redis Cache     │
└─────────────────┘ └─────────────┘ └─────────────────┘
```

### Production Node Configuration

```toml
# configs/production.toml
[runtime]
environment = "production"
log_level = "info"

[network]
http_listen_addr = "0.0.0.0:7845"
p2p_enabled = true
p2p_listen_addr = "/ip4/0.0.0.0/tcp/7000"
max_peers = 50
connection_timeout = 30

[storage]
backend = "postgresql"
connection_string = "postgresql://icn_prod_user:${DB_PASSWORD}@postgres.internal:5432/icn_production"
max_connections = 20
connection_timeout = 10

[security]
api_key_env = "ICN_API_KEY"
auth_token_env = "ICN_AUTH_TOKEN"
tls_cert_path = "/secrets/tls/cert.pem"
tls_key_path = "/secrets/tls/key.pem"
require_auth = true
open_rate_limit = 0

[identity]
key_path = "/secrets/identity/node.key"
key_passphrase_env = "ICN_KEY_PASSPHRASE"

[monitoring]
metrics_enabled = true
metrics_bind_addr = "127.0.0.1:9090"
health_check_interval = 30

[performance]
max_concurrent_jobs = 100
mana_regeneration_rate = 10
gc_interval = 3600
```

### Container Deployment

**Dockerfile**:
```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin icn-node

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/icn-node /usr/local/bin/
COPY configs/production.toml /etc/icn/config.toml

EXPOSE 7845 7000
USER 1000

CMD ["icn-node", "--config", "/etc/icn/config.toml"]
```

**docker-compose.yml**:
```yaml
version: '3.8'

services:
  icn-node:
    build: .
    ports:
      - "7845:7845"  # HTTP API
      - "7000:7000"  # P2P networking
    environment:
      - ICN_API_KEY_FILE=/run/secrets/api_key
      - ICN_KEY_PASSPHRASE_FILE=/run/secrets/key_passphrase
      - DB_PASSWORD_FILE=/run/secrets/db_password
    secrets:
      - api_key
      - key_passphrase
      - db_password
    volumes:
      - icn_data:/var/lib/icn
      - /etc/ssl/certs:/etc/ssl/certs:ro

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: icn_production
      POSTGRES_USER: icn_prod_user
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    secrets:
      - db_password
    volumes:
      - postgres_data:/var/lib/postgresql/data

secrets:
  api_key:
    file: ./secrets/api_key.txt
  key_passphrase:
    file: ./secrets/key_passphrase.txt
  db_password:
    file: ./secrets/db_password.txt

volumes:
  icn_data:
  postgres_data:
```

### Kubernetes Deployment

**deployment.yaml**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: icn-node
spec:
  replicas: 3
  selector:
    matchLabels:
      app: icn-node
  template:
    metadata:
      labels:
        app: icn-node
    spec:
      containers:
      - name: icn-node
        image: icn-core:latest
        ports:
        - containerPort: 7845
          name: http-api
        - containerPort: 7000
          name: p2p
        env:
        - name: ICN_API_KEY
          valueFrom:
            secretKeyRef:
              name: icn-secrets
              key: api-key
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secrets
              key: password
        volumeMounts:
        - name: config
          mountPath: /etc/icn
        - name: identity-keys
          mountPath: /secrets/identity
        - name: tls-certs
          mountPath: /secrets/tls
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 7845
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 7845
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: icn-config
      - name: identity-keys
        secret:
          secretName: icn-identity
      - name: tls-certs
        secret:
          secretName: icn-tls
```

---

## 📊 Monitoring & Operations

### Metrics and Observability

ICN provides Prometheus metrics for monitoring:

```bash
# Enable metrics
icn-node --metrics-enabled --metrics-bind-addr 0.0.0.0:9090
```

**Key Metrics**:
- `icn_node_uptime_seconds` - Node uptime
- `icn_mana_balance_total` - Current mana balance
- `icn_jobs_executed_total` - Mesh jobs executed
- `icn_p2p_peers_connected` - P2P peer connections
- `icn_governance_proposals_total` - Governance proposals

### Health Checks

```bash
# Node health status
curl https://your-node.domain.com/health

# Detailed system info
curl https://your-node.domain.com/status \
  -H "x-api-key: your-api-key"
```

### Log Management

Configure structured logging:

```toml
[logging]
level = "info"
format = "json"
output = "/var/log/icn/node.log"

[audit]
enabled = true
log_path = "/var/log/icn/audit.log"
```

---

## 💾 Backup & Recovery

### DAG Storage Backup

```bash
# Full DAG backup (all storage backends)
icn-cli dag backup --output ./backups/dag-backup-$(date +%Y%m%d).tar.gz

# Incremental backup (changes since last backup)
icn-cli dag backup --incremental --since 2024-01-01 \
                   --output ./backups/dag-incremental-$(date +%Y%m%d).tar.gz

# Restore from backup
icn-cli dag restore --input ./backups/dag-backup-20240101.tar.gz

# Verify data integrity after restore  
icn-cli dag verify --deep
```

### Database Backup (PostgreSQL)

```bash
# Full database backup
pg_dump icn_production | gzip > /backups/icn-db-$(date +%Y%m%d).sql.gz

# Automated backup script
#!/bin/bash
BACKUP_DIR="/backups/icn"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Database backup
pg_dump icn_production | gzip > "$BACKUP_DIR/db-$DATE.sql.gz"

# DAG backup
icn-cli dag backup --output "$BACKUP_DIR/dag-$DATE.tar.gz"

# Identity keys backup (encrypted)
tar -czf "$BACKUP_DIR/identity-$DATE.tar.gz" /secrets/identity/

# Cleanup old backups (keep last 30 days)
find "$BACKUP_DIR" -type f -mtime +30 -delete
```

### Disaster Recovery Procedure

1. **Restore Database**:
   ```bash
   createdb icn_production_new
   gunzip -c /backups/icn-db-latest.sql.gz | psql icn_production_new
   ```

2. **Restore DAG Data**:
   ```bash
   icn-cli dag restore --input /backups/dag-latest.tar.gz
   icn-cli dag verify --deep
   ```

3. **Restore Identity Keys**:
   ```bash
   tar -xzf /backups/identity-latest.tar.gz -C /secrets/
   chmod 600 /secrets/identity/*
   ```

4. **Restart Services**:
   ```bash
   # Kubernetes
   kubectl rollout restart deployment/icn-node
   
   # Docker Compose
   docker-compose restart icn-node
   ```

---

## 🚨 Production Readiness Checklist

Before deploying ICN in production, ensure:

### Security Requirements
- [ ] **Security audit completed** - Professional review of cryptographic implementations
- [ ] **Penetration testing performed** - Third-party security assessment  
- [ ] **Key management procedures** - HSM or secure key storage implemented
- [ ] **TLS certificates valid** - Proper CA-signed certificates installed
- [ ] **Authentication configured** - API keys and bearer tokens secured
- [ ] **Network security** - Firewalls and network segmentation configured

### Operational Requirements
- [ ] **Monitoring implemented** - Prometheus/Grafana or equivalent monitoring
- [ ] **Alerting configured** - Critical alerts for node failures, security issues
- [ ] **Backup procedures tested** - Regular backups and recovery procedures verified
- [ ] **Logging centralized** - Structured logs collected and indexed
- [ ] **Disaster recovery plan** - Documented procedures for system recovery
- [ ] **Performance testing** - Load testing under expected production workloads

### Technical Requirements
- [ ] **Database optimized** - Connection pooling, indexing, query optimization
- [ ] **Resource limits set** - CPU, memory, disk space monitoring and limits
- [ ] **Auto-scaling configured** - Horizontal scaling based on load (if applicable)
- [ ] **Health checks working** - Kubernetes/Docker health checks configured
- [ ] **Configuration management** - Infrastructure as code (Terraform, Ansible)

---

## 📚 Related Documentation

- **[Production Security Guide](PRODUCTION_SECURITY_GUIDE.md)** - Comprehensive security hardening
- **[API Reference](../ICN_API_REFERENCE.md)** - Complete HTTP API documentation
- **[Troubleshooting Guide](TROUBLESHOOTING.md)** - Common issues and solutions
- **[Development Guide](DEVELOPER_GUIDE.md)** - Development environment setup
- **[Federation DevNet](../icn-devnet/README.md)** - Containerized testing environment

---

## ⚠️ Important Notes

**Current Status**: ICN Core has substantial working implementations (~65-75% complete) with real P2P networking, governance systems, mesh computing, and persistent storage. However, it requires:

- **Security Review**: Cryptographic implementations need professional audit
- **Production Hardening**: Monitoring, error recovery, and operational procedures
- **Scale Testing**: Validation beyond development environments  

**Not recommended for production use** without completing security review and operational readiness requirements.

For development, testing, and research use, ICN Core provides a comprehensive platform for cooperative digital infrastructure experimentation.
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
