# ICN Core Monitoring Guide

This guide provides comprehensive monitoring and observability setup for ICN Core deployments.

## Overview

ICN Core provides extensive metrics through Prometheus endpoints, health checks, and logging. This document covers:

- **Metrics Collection**: Prometheus metrics and custom dashboards
- **Health Monitoring**: Health and readiness endpoints
- **Alerting**: Critical alerts and notification setup
- **Logging**: Structured logging and log aggregation
- **Performance Monitoring**: System and application performance tracking

## Metrics Endpoint

### Prometheus Metrics

ICN nodes expose metrics at `/metrics` endpoint in Prometheus format:

```bash
curl http://localhost:7845/metrics
```

### Core Metrics Categories

#### 1. Network Metrics
- `network_peer_count` - Connected peers
- `network_kademlia_peers` - Peers in Kademlia DHT
- `network_bytes_sent_total` - Total bytes sent
- `network_bytes_received_total` - Total bytes received
- `network_messages_sent_total` - Total messages sent
- `network_messages_received_total` - Total messages received
- `network_ping_rtt_*_ms` - Network latency metrics

#### 2. Mesh Computing Metrics
- `mesh_jobs_submitted_total` - Jobs submitted to mesh
- `mesh_jobs_completed_total` - Successfully completed jobs
- `mesh_jobs_failed_total` - Failed jobs
- `mesh_jobs_pending` - Currently pending jobs
- `mesh_jobs_executing` - Currently executing jobs
- `mesh_job_*_duration_seconds` - Job timing metrics

#### 3. Economics Metrics
- `economics_mana_balance_queries_total` - Mana balance queries
- `economics_mana_spending_operations_total` - Mana spending operations
- `economics_mana_credit_operations_total` - Mana credit operations
- `mana_supply_total` - Total mana in circulation
- `mana_regeneration_rate` - Average regeneration rate

#### 4. Governance Metrics
- `governance_proposals_submitted_total` - Proposals submitted
- `governance_votes_cast_total` - Votes cast
- `governance_proposals_executed_total` - Proposals executed
- `federation_members_total` - Federation members

#### 5. System Metrics
- `node_uptime_seconds` - Node uptime
- `system_memory_usage_bytes` - Memory usage
- `system_cpu_usage_percent` - CPU usage
- `http_requests_total` - HTTP requests
- `http_request_duration_seconds` - HTTP request duration
- `http_errors_total` - HTTP errors

#### 6. DAG Storage Metrics
- `dag_put_calls_total` - DAG put operations
- `dag_get_calls_total` - DAG get operations
- `dag_storage_size_bytes` - DAG storage size
- `dag_block_height` - Current block height

## Health Endpoints

### Health Check (`/health`)

Returns overall node health status:

```json
{
  "status": "OK",
  "timestamp": 1703001234,
  "uptime_seconds": 86400,
  "checks": {
    "runtime": "OK",
    "dag_store": "OK",
    "network": "OK",
    "mana_ledger": "OK"
  }
}
```

Example JSON dashboards are available in [`monitoring/dashboards`](../monitoring/dashboards). Import these files into Grafana to visualize runtime metrics such as `runtime_receipts_anchored_total` and `mana_accounts`.

**Status Codes:**
- `200` - Node is healthy
- `503` - Node is unhealthy

### Readiness Check (`/ready`)

Returns node readiness for serving requests:

```json
{
  "ready": true,
  "timestamp": 1703001234,
  "checks": {
    "can_serve_requests": true,
    "mana_ledger_available": true,
    "dag_store_available": true,
    "network_initialized": true
  }
}
```

**Status Codes:**
- `200` - Node is ready
- `503` - Node is not ready

## Prometheus Configuration

### Prometheus Configuration File

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "icn_alerts.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  - job_name: 'icn-nodes'
    static_configs:
      - targets: ['node1:7845', 'node2:7845', 'node3:7845']
    metrics_path: '/metrics'
    scrape_interval: 10s
    
  - job_name: 'icn-health'
    static_configs:
      - targets: ['node1:7845', 'node2:7845', 'node3:7845']
    metrics_path: '/health'
    scrape_interval: 30s
```

## Alerting Rules

### Critical Alerts (`icn_alerts.yml`)

```yaml
groups:
- name: icn-core
  rules:
  # Node Health Alerts
  - alert: ICNNodeDown
    expr: up{job="icn-nodes"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "ICN Node {{ $labels.instance }} is down"
      description: "ICN node {{ $labels.instance }} has been down for more than 1 minute"

  - alert: ICNNodeNotReady
    expr: icn_node_ready == 0
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "ICN Node {{ $labels.instance }} is not ready"
      description: "ICN node {{ $labels.instance }} is not ready to serve requests"

  # Network Alerts
  - alert: ICNLowPeerCount
    expr: network_peer_count < 3
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "ICN Node {{ $labels.instance }} has low peer count"
      description: "ICN node {{ $labels.instance }} has only {{ $value }} peers connected"

  - alert: ICNNetworkLatencyHigh
    expr: network_ping_rtt_avg_ms > 1000
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "ICN Network latency is high"
      description: "Average network latency is {{ $value }}ms on {{ $labels.instance }}"

  # Job Processing Alerts
  - alert: ICNJobFailureRate
    expr: rate(mesh_jobs_failed_total[5m]) / rate(mesh_jobs_submitted_total[5m]) > 0.1
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "ICN Job failure rate is high"
      description: "Job failure rate is {{ $value | humanizePercentage }} on {{ $labels.instance }}"

  - alert: ICNJobQueueBacklog
    expr: mesh_jobs_pending > 100
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "ICN Job queue has backlog"
      description: "{{ $value }} jobs are pending on {{ $labels.instance }}"

  # System Resource Alerts
  - alert: ICNHighMemoryUsage
    expr: system_memory_usage_bytes / 1024 / 1024 / 1024 > 2
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "ICN Node memory usage is high"
      description: "Memory usage is {{ $value | humanize }}GB on {{ $labels.instance }}"

  - alert: ICNHighHTTPErrorRate
    expr: rate(http_errors_total[5m]) > 10
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "ICN HTTP error rate is high"
      description: "HTTP error rate is {{ $value }} errors/sec on {{ $labels.instance }}"

  # Economics Alerts
  - alert: ICNManaImbalance
    expr: mana_supply_total < 1000000
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "ICN Mana supply is low"
      description: "Total mana supply is {{ $value }} on {{ $labels.instance }}"

  # DAG Storage Alerts
  - alert: ICNDAGStorageGrowth
    expr: increase(dag_storage_size_bytes[1h]) > 1000000000
    for: 1h
    labels:
      severity: warning
    annotations:
      summary: "ICN DAG storage growing rapidly"
      description: "DAG storage grew by {{ $value | humanizeBytes }} in the last hour on {{ $labels.instance }}"
```

## Grafana Dashboard

### ICN Core Dashboard Configuration

```json
{
  "dashboard": {
    "title": "ICN Core Monitoring",
    "panels": [
      {
        "title": "Node Health",
        "type": "stat",
        "targets": [
          {
            "expr": "up{job=\"icn-nodes\"}",
            "legendFormat": "{{ instance }}"
          }
        ]
      },
      {
        "title": "Network Peers",
        "type": "graph",
        "targets": [
          {
            "expr": "network_peer_count",
            "legendFormat": "Connected Peers"
          },
          {
            "expr": "network_kademlia_peers",
            "legendFormat": "Kademlia Peers"
          }
        ]
      },
      {
        "title": "Job Processing",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(mesh_jobs_submitted_total[5m])",
            "legendFormat": "Submitted"
          },
          {
            "expr": "rate(mesh_jobs_completed_total[5m])",
            "legendFormat": "Completed"
          },
          {
            "expr": "rate(mesh_jobs_failed_total[5m])",
            "legendFormat": "Failed"
          }
        ]
      },
      {
        "title": "System Resources",
        "type": "graph",
        "targets": [
          {
            "expr": "system_memory_usage_bytes / 1024 / 1024 / 1024",
            "legendFormat": "Memory (GB)"
          },
          {
            "expr": "system_cpu_usage_percent",
            "legendFormat": "CPU %"
          }
        ]
      }
    ]
  }
}
```

## Logging Configuration

### Structured Logging

Enable structured logging with environment variables:

```bash
export RUST_LOG="info,icn_runtime=debug,icn_network=debug"
export ICN_LOG_FORMAT="json"
```

### Log Aggregation

Use Fluent Bit or similar for log aggregation:

```yaml
# fluent-bit.conf
[INPUT]
    Name tail
    Path /var/log/icn/*.log
    Parser json
    Tag icn.*

[OUTPUT]
    Name elasticsearch
    Match icn.*
    Host elasticsearch
    Port 9200
    Index icn-logs
```

## Docker Compose Monitoring Stack

```yaml
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - ./icn_alerts.yml:/etc/prometheus/icn_alerts.yml

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-storage:/var/lib/grafana

  alertmanager:
    image: prom/alertmanager:latest
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml

  icn-node:
    image: icn-core:latest
    ports:
      - "7845:7845"
    environment:
      - RUST_LOG=info
      - ICN_LOG_FORMAT=json

volumes:
  grafana-storage:
```

## Production Monitoring Checklist

### Essential Monitoring

- [ ] Node health checks configured
- [ ] Prometheus scraping all nodes
- [ ] Critical alerts configured
- [ ] Grafana dashboards set up
- [ ] Log aggregation configured
- [ ] Backup monitoring enabled

### Performance Monitoring

- [ ] Job processing metrics tracked
- [ ] Network latency monitored
- [ ] Resource usage monitored
- [ ] Error rates tracked
- [ ] Capacity planning metrics

### Security Monitoring

- [ ] Authentication failures tracked
- [ ] Rate limiting violations monitored
- [ ] Network security events logged
- [ ] Governance actions audited

### Operational Monitoring

- [ ] Disk space monitoring
- [ ] Database performance
- [ ] Federation health
- [ ] Mana economics health
- [ ] Smart contract execution

## Troubleshooting Common Issues

### High Memory Usage

1. Check `system_memory_usage_bytes` metric
2. Review DAG storage size
3. Check for memory leaks in logs
4. Adjust caching settings

### Network Connectivity Issues

1. Monitor `network_peer_count`
2. Check firewall/NAT configuration
3. Verify bootstrap peers
4. Review network logs

### Job Processing Problems

1. Check `mesh_jobs_pending` queue
2. Review job failure rates
3. Monitor executor availability
4. Check mana balances

### Performance Degradation

1. Monitor HTTP request latency
2. Check system resource usage
3. Review database performance
4. Analyze log patterns

## Contact and Support

For monitoring support and troubleshooting:

- Documentation: https://docs.icn.network
- Community: https://community.icn.network
- Issues: https://github.com/icn-network/icn-core/issues 