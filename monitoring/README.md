# ICN Production Monitoring

This directory contains comprehensive monitoring configurations for ICN (InterCooperative Network) production deployments. The monitoring stack provides federation-wide visibility, alerting, and performance tracking.

## Overview

ICN's production monitoring consists of several key components:

- **Prometheus Metrics**: 60+ custom metrics across all ICN components
- **Recording Rules**: Pre-computed aggregations for performance
- **Alerting Rules**: 25+ alerts covering critical operational scenarios
- **Grafana Dashboards**: Visual monitoring of federation health and performance
- **Health Checks**: Built-in endpoints for system validation

## Quick Start

### 1. Deploy Prometheus

```bash
# Create Prometheus configuration
cat <<EOF > prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 30s

rule_files:
  - "recording-rules.yml"
  - "alerting-rules.yml"

scrape_configs:
  - job_name: 'icn-node'
    scrape_interval: 30s
    static_configs:
      - targets: ['localhost:7845']  # ICN node metrics endpoint
    metrics_path: /metrics

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

EOF

# Start Prometheus with ICN rules
docker run -d \
  --name prometheus \
  -p 9090:9090 \
  -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml \
  -v $(pwd)/prometheus/:/etc/prometheus/rules/ \
  prom/prometheus:latest \
  --config.file=/etc/prometheus/prometheus.yml \
  --storage.tsdb.path=/prometheus \
  --web.console.libraries=/etc/prometheus/console_libraries \
  --web.console.templates=/etc/prometheus/consoles \
  --web.enable-lifecycle \
  --web.enable-admin-api
```

### 2. Deploy Grafana with ICN Dashboard

```bash
# Start Grafana
docker run -d \
  --name grafana \
  -p 3000:3000 \
  -e "GF_SECURITY_ADMIN_PASSWORD=admin" \
  grafana/grafana:latest

# Import ICN dashboard (after Grafana is running)
curl -X POST \
  http://admin:admin@localhost:3000/api/dashboards/db \
  -H 'Content-Type: application/json' \
  -d @grafana/icn-federation-overview.json
```

### 3. Configure Alerting (Optional)

```bash
# Deploy Alertmanager
docker run -d \
  --name alertmanager \
  -p 9093:9093 \
  -v $(pwd)/alertmanager.yml:/etc/alertmanager/alertmanager.yml \
  prom/alertmanager:latest
```

## Monitoring Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   ICN Nodes     │    │   Prometheus     │    │    Grafana      │
│                 │    │                  │    │                 │
│ ┌─────────────┐ │    │ ┌──────────────┐ │    │ ┌─────────────┐ │
│ │/metrics     │◄├────┤ │ Recording    │ │    │ │ ICN         │ │
│ │/health      │ │    │ │ Rules        │ │    │ │ Dashboard   │ │
│ │/ready       │ │    │ │              │ │    │ │             │ │
│ └─────────────┘ │    │ ├──────────────┤ │    │ └─────────────┘ │
│                 │    │ │ Alerting     │ │    │                 │
│ Multiple Nodes  │    │ │ Rules        │ │    │ Real-time       │
│ in Federation   │    │ └──────────────┘ │    │ Visualization   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │  Alertmanager    │
                       │                  │
                       │ Notifications:   │
                       │ • Slack          │
                       │ • Email          │
                       │ • PagerDuty      │
                       └──────────────────┘
```

## Metrics Categories

### Core System Metrics
- **Node Health**: `icn:node_health_score`, `up`, `node_uptime_seconds`
- **Resource Usage**: `system_memory_usage_bytes`, `system_cpu_usage_percent`
- **API Performance**: `http_request_duration_seconds`, `http_requests_total`, `http_errors_total`

### Federation Metrics
- **Network**: `network_peer_count`, `network_ping_rtt_avg_ms`, `network_bytes_sent_total`
- **Connectivity**: `icn:federation_health_ratio`, `icn:node_peer_connectivity_ratio`
- **Consensus**: `icn:federation_node_count`, `icn:federation_healthy_nodes`

### Economic Metrics
- **Mana Management**: `mana_supply_total`, `economics_mana_spending_operations_total`
- **Resource Flow**: `icn:federation_mana_velocity`, `icn:mana_regeneration_rate`
- **Account Activity**: `runtime_mana_accounts_total`, `economics_mana_balance_queries_total`

### Job Execution Metrics
- **Throughput**: `icn:job_throughput_5m`, `icn:federation_job_completion_rate`
- **Quality**: `icn:job_success_rate_5m`, `runtime_jobs_failed_total`
- **Performance**: `mesh_job_execution_duration_seconds`, `icn:avg_job_duration_5m`
- **Queue Management**: `mesh_jobs_pending`, `icn:federation_job_backlog`

### Governance Metrics
- **Activity**: `governance_proposals_submitted_total`, `governance_votes_cast_total`
- **Participation**: `icn:governance_voting_participation`
- **Execution**: `governance_proposals_executed_total`

### Security Metrics
- **Authentication**: `auth_failures_total`, `icn:auth_failure_rate_5m`
- **Verification**: `identity_proofs_verified_total`, `identity_proof_verification_failures_total`
- **Rate Limiting**: `rate_limit_violations_total`

### Storage Metrics
- **DAG Operations**: `dag_put_calls_total`, `dag_get_calls_total`
- **Growth**: `dag_storage_size_bytes`, `icn:dag_growth_rate_1h`
- **Receipts**: `runtime_receipts_anchored_total`, `icn:receipt_anchoring_rate_5m`

## Alert Severity Levels

### Critical Alerts (Immediate Response Required)
- **ICNNodeDown**: Node completely unavailable
- **ICNNodeUnhealthy**: Node health score below 50%
- **ICNFederationPartition**: Federation health below 60%
- **ICNLowNodeCount**: Fewer than 3 nodes in federation
- **ICNHighAuthFailureRate**: Possible security attack

### Warning Alerts (Investigation Required)
- **ICNHighJobFailureRate**: Job success rate below 80%
- **ICNJobBacklogGrowing**: Job queue growing rapidly
- **ICNSlowJobExecution**: Average job duration above 5 minutes
- **ICNLowPeerConnectivity**: Node connectivity below 50%
- **ICNHighMemoryUsage**: Memory usage above 90%

### Info Alerts (Awareness Only)
- **ICNStagnantGovernance**: No governance activity for 24 hours

## Dashboard Features

The ICN Federation Overview dashboard provides:

### Key Performance Indicators (KPIs)
- Federation health score with color-coded status
- Active node count (total vs healthy)
- Real-time job throughput metrics
- Total mana supply across federation

### Time Series Visualizations
- Node health trends over time
- Job execution patterns and failures
- Network connectivity and latency
- Economic activity (mana flow)
- Governance participation rates
- System resource utilization
- API performance metrics

### Interactive Features
- Instance filtering (view specific nodes)
- Federation selection (multi-federation support)
- Alert annotations on graphs
- Real-time alert display panel

## Operational Procedures

### Daily Monitoring Checklist

1. **Federation Health Check**
   ```bash
   # Check overall federation status
   curl http://prometheus:9090/api/v1/query?query=icn:federation_health_ratio
   
   # Verify all nodes are responding
   curl http://prometheus:9090/api/v1/query?query=up{job="icn-node"}
   ```

2. **Performance Review**
   ```bash
   # Check job throughput
   curl http://prometheus:9090/api/v1/query?query=icn:federation_job_completion_rate
   
   # Review error rates
   curl http://prometheus:9090/api/v1/query?query=icn:job_success_rate_5m
   ```

3. **Resource Monitoring**
   ```bash
   # Check memory usage across nodes
   curl http://prometheus:9090/api/v1/query?query=icn:node_memory_usage_ratio
   
   # Review mana supply levels
   curl http://prometheus:9090/api/v1/query?query=icn:federation_total_mana
   ```

### Alert Response Procedures

#### Critical Alert: ICNNodeDown
1. **Immediate Actions**:
   - Check node process status: `systemctl status icn-node`
   - Review node logs: `journalctl -u icn-node -n 100`
   - Verify network connectivity to the node

2. **Diagnosis**:
   - Check system resources (memory, disk, CPU)
   - Verify configuration files
   - Test network ports (7845 for HTTP API)

3. **Resolution**:
   - Restart node service if process crashed
   - Scale resources if resource exhaustion
   - Fix configuration issues if detected

#### Warning Alert: ICNHighJobFailureRate
1. **Investigation**:
   - Check executor health and availability
   - Review job specifications for resource requirements
   - Verify mana balances for job submission

2. **Potential Causes**:
   - Insufficient executor capacity
   - Network connectivity issues
   - Resource constraints (mana, CPU, memory)
   - Invalid job specifications

3. **Resolution Actions**:
   - Scale up executor capacity
   - Fix network connectivity issues
   - Adjust resource allocation
   - Review and fix job specifications

### Performance Tuning

#### Metrics Collection Optimization
```yaml
# Adjust scrape intervals based on environment
global:
  scrape_interval: 30s      # Production: 30s, Dev: 60s
  evaluation_interval: 30s  # Production: 30s, Dev: 60s

# Reduce metric cardinality if needed
metric_relabel_configs:
  - source_labels: [__name__]
    regex: 'detailed_metric_.*'
    action: drop
```

#### Recording Rule Optimization
```yaml
# Adjust intervals based on usage patterns
groups:
  - name: high_frequency_rules
    interval: 30s           # For alerts and dashboards
    rules: [...]
    
  - name: low_frequency_rules
    interval: 300s          # For long-term analysis
    rules: [...]
```

## Troubleshooting

### Common Issues

#### Metrics Not Appearing
1. **Check ICN node metrics endpoint**: `curl http://node:7845/metrics`
2. **Verify Prometheus target discovery**: Check `/targets` in Prometheus UI
3. **Review Prometheus logs**: `docker logs prometheus`

#### High Alert Volume
1. **Review alert thresholds**: May need adjustment for your environment
2. **Check for cascading failures**: Fix root cause alerts first
3. **Temporarily silence non-critical alerts**: Use Alertmanager silences

#### Dashboard Performance Issues
1. **Reduce query time ranges**: Use shorter time windows
2. **Optimize complex queries**: Simplify PromQL expressions
3. **Increase Grafana resources**: Scale up memory/CPU

### Advanced Configuration

#### Multi-Federation Monitoring
```yaml
# Prometheus federation configuration
scrape_configs:
  - job_name: 'federation-primary'
    static_configs:
      - targets: ['primary-node:7845']
    
  - job_name: 'federation-secondary'  
    static_configs:
      - targets: ['secondary-node:7845']
```

#### High Availability Setup
```yaml
# Prometheus HA with external labels
global:
  external_labels:
    replica: 'prometheus-1'
    region: 'us-west-2'
    
# Use Thanos or Cortex for long-term storage
```

## Security Considerations

### Metrics Security
- **Access Control**: Restrict Prometheus and Grafana access
- **Network Security**: Use TLS for all monitoring traffic
- **Data Retention**: Configure appropriate retention policies

### Sensitive Metrics
The following metrics may contain sensitive information:
- Node identity and location data
- Economic transaction patterns
- Governance voting patterns

Consider implementing access controls and data anonymization for sensitive environments.

## Support and Maintenance

### Regular Maintenance Tasks
- **Weekly**: Review alert accuracy and adjust thresholds
- **Monthly**: Analyze performance trends and capacity planning
- **Quarterly**: Update monitoring configurations and dashboards

### Monitoring the Monitoring
- Set up alerts for Prometheus itself
- Monitor Grafana performance and availability
- Track metric ingestion rates and storage usage

For additional support, refer to the ICN Operations Guide or contact the ICN DevOps team. 