# ICN Core Production Configuration Templates

This directory contains configuration templates for different ICN Core deployment scenarios.

## Configuration Templates

### 1. Production Federation Node (`production-federation.toml`)

A production-ready configuration for nodes participating in federated networks:

```toml
[environment]
type = "production"
enable_monitoring = true
enable_metrics = true

[network]
# Production libp2p configuration
listen_addresses = ["/ip4/0.0.0.0/tcp/7946"]
max_peers = 1000
max_peers_per_ip = 5
connection_timeout_secs = 30
request_timeout_secs = 10
heartbeat_interval_secs = 15
bootstrap_interval_secs = 300
peer_discovery_interval_secs = 60
enable_mdns = false  # Disabled for production security
kademlia_replication_factor = 20

# Bootstrap peers for network joining
bootstrap_peers = [
    # Add your bootstrap peer addresses here
    # Format: "peer_id@/ip4/1.2.3.4/tcp/7946"
]

[identity]
# Enhanced DID resolution
resolver_type = "enhanced"
cache_ttl_seconds = 3600
max_cache_size = 10000
web_timeout_seconds = 30
enable_fallback = true
method_preference = ["key", "peer", "web"]

[federation]
# Federation management settings
enable_federation_discovery = true
auto_join_compute_federations = false  # Set to true if desired
auto_join_governance_federations = false
federation_sync_interval_secs = 300

# Federation trust settings
enable_federation_trust_scoring = true
trust_threshold = 0.6
max_federation_memberships = 10

[reputation]
# Reputation system configuration
enable_reputation_tracking = true
reputation_decay_rate = 0.01  # Daily decay rate
min_reputation_for_jobs = 25
reputation_boost_successful_job = 5
reputation_penalty_failed_job = -10

[mesh]
# Mesh network configuration
enable_cross_federation_jobs = true
job_timeout_secs = 300
bid_collection_timeout_secs = 30
max_concurrent_jobs = 10

# Job selection policy
prefer_high_reputation = true
prefer_federation_members = true
reputation_weight = 0.5
cost_weight = 0.3
latency_weight = 0.2

[governance]
# CCL WASM execution settings
max_execution_time_secs = 30
max_memory_bytes = 67108864  # 64MB
max_instructions = 10000000
enable_optimizations = true
optimization_level = "balanced"
enable_monitoring = true
module_cache_size = 100

[storage]
# DAG storage configuration
backend = "rocksdb"  # or "sled", "postgresql"
path = "./data/dag"
max_cache_size_mb = 512
sync_interval_secs = 60

[mana]
# Mana/economic configuration
backend = "rocksdb"  # or "sled", "sqlite"
path = "./data/mana"
regeneration_rate = 10  # mana per hour
max_capacity = 10000
initial_balance = 1000

[monitoring]
# Metrics and monitoring
enable_prometheus = true
prometheus_port = 9090
log_level = "info"
metrics_retention_days = 30
```

### 2. Development Node (`development.toml`)

Configuration for development and testing:

```toml
[environment]
type = "development"
enable_monitoring = true
enable_metrics = true

[network]
listen_addresses = ["/ip4/127.0.0.1/tcp/0"]
max_peers = 100
enable_mdns = true  # Enabled for local development
kademlia_replication_factor = 10

[identity]
resolver_type = "enhanced"
cache_ttl_seconds = 300  # Shorter cache for development
max_cache_size = 1000

[federation]
enable_federation_discovery = true
auto_join_compute_federations = true
federation_sync_interval_secs = 60

[reputation]
enable_reputation_tracking = true
min_reputation_for_jobs = 10  # Lower threshold for testing

[governance]
max_execution_time_secs = 10  # Shorter for quick testing
max_memory_bytes = 33554432   # 32MB
enable_optimizations = false  # Faster compilation for development

[storage]
backend = "sled"
path = "./dev-data/dag"

[mana]
backend = "sled"
path = "./dev-data/mana"
initial_balance = 5000  # Higher for testing
```

### 3. Testing/CI Configuration (`testing.toml`)

Minimal configuration for automated testing:

```toml
[environment]
type = "testing"
enable_monitoring = false

[network]
# Use stub services for testing
use_stubs = true

[identity]
resolver_type = "simple"  # Use simple KeyDidResolver for speed

[federation]
enable_federation_discovery = false

[reputation]
enable_reputation_tracking = false

[governance]
max_execution_time_secs = 5
enable_optimizations = false
enable_monitoring = false

[storage]
backend = "memory"

[mana]
backend = "memory"
initial_balance = 10000
```

## Environment-Specific Examples

### Docker Compose Production Setup

```yaml
# docker-compose.production.yml
version: '3.8'
services:
  icn-node:
    image: icn-core:latest
    ports:
      - "7946:7946"  # P2P networking
      - "9090:9090"  # Prometheus metrics
    environment:
      - ICN_CONFIG_FILE=/config/production-federation.toml
      - ICN_LOG_LEVEL=info
    volumes:
      - ./config:/config:ro
      - ./data:/data
      - ./logs:/logs
    restart: unless-stopped
    
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
    restart: unless-stopped
```

### Kubernetes Production Deployment

```yaml
# k8s-production.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: icn-federation-node
spec:
  serviceName: icn-federation
  replicas: 3
  selector:
    matchLabels:
      app: icn-federation-node
  template:
    metadata:
      labels:
        app: icn-federation-node
    spec:
      containers:
      - name: icn-node
        image: icn-core:latest
        ports:
        - containerPort: 7946
          name: p2p
        - containerPort: 9090
          name: metrics
        env:
        - name: ICN_CONFIG_FILE
          value: /config/production-federation.toml
        volumeMounts:
        - name: config
          mountPath: /config
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
      volumes:
      - name: config
        configMap:
          name: icn-config
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

## Configuration Validation

Use the provided validation tools to ensure your configuration is production-ready:

```bash
# Validate configuration
icn-cli config validate --config production-federation.toml

# Test network connectivity
icn-cli network test --config production-federation.toml

# Check federation discovery
icn-cli federation discover --config production-federation.toml

# Verify reputation system
icn-cli reputation status --config production-federation.toml
```

## Security Considerations

### Production Security Checklist

- [ ] **Network Security**
  - [ ] mDNS disabled in production
  - [ ] Firewall configured for P2P port range
  - [ ] TLS/encryption enabled for all communications
  - [ ] Bootstrap peers from trusted sources only

- [ ] **Identity Security**
  - [ ] Private keys stored securely (HSM recommended)
  - [ ] DID resolution over secure channels only
  - [ ] Certificate pinning for did:web resolution

- [ ] **Federation Security**
  - [ ] Federation trust policies configured
  - [ ] Reputation thresholds enforced
  - [ ] Membership validation enabled
  - [ ] Automatic federation joining disabled

- [ ] **Resource Security**
  - [ ] Resource limits enforced
  - [ ] Execution timeouts configured
  - [ ] Memory limits set appropriately
  - [ ] Storage access controls in place

## Monitoring and Observability

### Prometheus Metrics

Key metrics to monitor in production:

- `icn_peer_count` - Number of connected peers
- `icn_job_success_rate` - Job execution success rate
- `icn_reputation_average` - Average network reputation
- `icn_federation_membership_count` - Number of federation memberships
- `icn_ccl_execution_time` - CCL execution performance

### Alerting Rules

```yaml
# prometheus-alerts.yml
groups:
- name: icn-core
  rules:
  - alert: LowPeerCount
    expr: icn_peer_count < 5
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "ICN node has low peer count"
      
  - alert: HighJobFailureRate
    expr: rate(icn_job_failures_total[5m]) > 0.1
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High job failure rate detected"
```

## Troubleshooting

### Common Issues

1. **Network Connectivity**
   - Check firewall settings for P2P port
   - Verify bootstrap peers are reachable
   - Ensure libp2p feature is enabled

2. **Federation Discovery**
   - Verify network connectivity to peers
   - Check federation trust settings
   - Ensure DID resolution is working

3. **Reputation Issues**
   - Check reputation store backend configuration
   - Verify reputation calculation parameters
   - Monitor reputation decay settings

4. **Performance Issues**
   - Check CCL WASM optimization settings
   - Monitor memory and CPU usage
   - Verify storage backend performance