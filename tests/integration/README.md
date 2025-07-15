# ICN Comprehensive End-to-End Testing

This directory contains comprehensive end-to-end tests for the InterCooperative Network (ICN) that validate the complete system functionality in a near-production environment.

## Overview

The comprehensive E2E test (`comprehensive_e2e.rs`) validates the entire ICN mesh job lifecycle including:

- **Multi-node federation setup and convergence**
- **Complete mesh job lifecycle** (submit → bid → execute → complete)
- **DAG receipt anchoring and queries**
- **Mana balance tracking and automatic refunds**
- **Prometheus metrics collection**
- **Performance under load**

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust toolchain (nightly)
- Available ports: 5001-5003 (nodes), 9090 (Prometheus), 3000 (Grafana)

### Running the Test

```bash
# Quick run with fresh federation
./scripts/run_comprehensive_e2e_test.sh

# Use existing federation (if already running)
ICN_DEVNET_RUNNING=1 ./scripts/run_comprehensive_e2e_test.sh

# Keep federation running after test
./scripts/run_comprehensive_e2e_test.sh --keep-running

# Verbose output
./scripts/run_comprehensive_e2e_test.sh --verbose
```

### Manual Test Execution

```bash
# Start federation with monitoring
cd icn-devnet
docker-compose --profile monitoring up -d

# Wait for stabilization
sleep 60

# Run the test
cargo test --release comprehensive_mesh_job_e2e_test --features="enable-libp2p" -- --nocapture

# Optional: Keep federation running
export ICN_DEVNET_RUNNING=1
```

## Test Architecture

### Federation Setup

The test uses a 3-node federation:

- **Node A** (localhost:5001): Bootstrap node and job submitter
- **Node B** (localhost:5002): Executor node
- **Node C** (localhost:5003): Executor node

### Monitoring Stack

- **Prometheus** (localhost:9090): Metrics collection
- **Grafana** (localhost:3000): Dashboard visualization
- **Node metrics** (localhost:5001-5003/metrics): Individual node metrics

### Test Flow

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Phase 1       │    │   Phase 2       │    │   Phase 3       │
│   Federation    │───▶│   Job Lifecycle │───▶│   Mana Economics│
│   Health Check  │    │   Validation    │    │   Validation    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                        │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Phase 6       │    │   Phase 5       │    │   Phase 4       │
│   Performance   │◀───│   Load Testing  │◀───│   DAG Integrity │
│   Validation    │    │   (5 jobs)      │    │   Validation    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Test Configuration

### Configuration File

Edit `tests/integration/test_config.toml` to customize test parameters:

```toml
[general]
test_timeout = 600
max_retries = 20
retry_delay = 3

[load_test]
num_concurrent_jobs = 5
job_submission_interval = 500
load_test_duration = 300

[validation]
min_success_rate = 0.95
max_failure_rate = 0.05
```

### Environment Variables

- `ICN_DEVNET_RUNNING`: Use existing federation
- `ICN_E2E_TEST_TIMEOUT`: Test timeout in seconds
- `ICN_TEST_MODE`: Enable test-specific behaviors
- `RUST_LOG`: Control logging verbosity

## Test Results and Monitoring

### Real-Time Monitoring

During test execution, monitor progress at:

- **Prometheus**: http://localhost:9090 - Raw metrics and queries
- **Grafana**: http://localhost:3000 - Visual dashboards (admin/icnfederation)
- **Node APIs**: http://localhost:5001-5003/info - Node status

### Test Reports

After completion, find results in:

- `test_results/comprehensive_e2e_*.log` - Detailed test logs
- `test_results/comprehensive_e2e_report_*.html` - HTML test report
- `test_results/diagnostics_*/` - Diagnostic information (on failure)

### Key Metrics to Monitor

| Metric | Description | Expected Value |
|--------|-------------|----------------|
| `icn_jobs_submitted_total` | Total jobs submitted | Increasing |
| `icn_jobs_completed_total` | Total jobs completed | ≥ 95% of submitted |
| `icn_job_process_time` | Job execution duration | < 60s average |
| `icn_mana_balance` | Mana balance changes | Decreasing/regenerating |
| `icn_peer_count` | Network connectivity | ≥ 2 peers per node |
| `icn_dag_blocks_total` | DAG receipt anchoring | Increasing |

## Job Types Tested

### 1. Fibonacci Calculation
- **Input**: 25
- **Expected**: 75025
- **Cost**: 200 mana
- **Timeout**: 30s

### 2. Prime Number Check
- **Input**: 1000003 (large prime)
- **Expected**: true
- **Cost**: 150 mana
- **Timeout**: 60s

### 3. CPU Benchmark
- **Input**: 10000 iterations
- **Cost**: 300 mana
- **Timeout**: 120s

## Understanding Test Results

### Success Criteria

✅ **Test passes when:**
- All federation nodes are healthy and connected
- Job submission, execution, and completion work correctly
- Mana balances are tracked and refunded appropriately
- DAG receipts are anchored and verifiable
- Load test completes with >95% success rate
- Performance metrics are within acceptable ranges

### Common Issues

❌ **Test may fail due to:**
- Docker/container startup issues
- Port conflicts (5001-5003, 9090, 3000)
- Insufficient system resources
- Network connectivity issues
- Job executor selection problems

### Debugging

1. **Check container status**:
   ```bash
   docker-compose -f icn-devnet/docker-compose.yml --profile monitoring ps
   ```

2. **View container logs**:
   ```bash
   docker-compose -f icn-devnet/docker-compose.yml --profile monitoring logs icn-node-a
   ```

3. **Test node connectivity**:
   ```bash
   curl -H "X-API-Key: devnet-a-key" http://localhost:5001/info
   ```

4. **Check metrics**:
   ```bash
   curl http://localhost:9090/api/v1/query?query=icn_jobs_submitted_total
   ```

## Test Phases in Detail

### Phase 1: Federation Health
- Validates all nodes are running and responsive
- Checks API endpoints and health status
- Verifies node configuration and identity

### Phase 2: P2P Convergence
- Ensures nodes discover each other
- Validates peer connections are established
- Checks network topology convergence

### Phase 3: Metrics Collection
- Validates Prometheus is scraping node metrics
- Checks metric endpoint availability
- Verifies key metrics are being recorded

### Phase 4: Single Job Lifecycle
- Submits a computational job (Fibonacci calculation)
- Tracks through all lifecycle stages
- Validates execution result correctness

### Phase 5: Mana Economics
- Tracks mana balance changes
- Validates automatic refund mechanisms
- Checks transaction history recording

### Phase 6: DAG Integrity
- Validates receipt anchoring in DAG
- Checks receipt signature verification
- Tests cross-node receipt replication

### Phase 7: Load Testing
- Submits multiple concurrent jobs
- Tests system performance under load
- Validates completion rates and timing

### Phase 8: Performance Validation
- Analyzes Prometheus metrics
- Validates performance thresholds
- Checks resource utilization

## Extending the Test

### Adding New Job Types

1. Define job specification in `test_config.toml`
2. Add job creation logic in test harness
3. Implement result validation

### Adding New Metrics

1. Add metric queries to Prometheus validation
2. Update Grafana dashboard configuration
3. Add performance threshold checks

### Customizing Load Testing

1. Adjust `num_concurrent_jobs` in configuration
2. Modify job submission patterns
3. Add different job type distributions

## Performance Baselines

Based on test runs, expect:

- **Job submission**: < 1s latency
- **Job execution**: 10-60s depending on complexity
- **Network latency**: < 100ms between nodes
- **Memory usage**: < 200MB per node
- **CPU usage**: < 50% under normal load

## Troubleshooting Guide

### Federation Won't Start

```bash
# Clean up existing containers
docker-compose -f icn-devnet/docker-compose.yml --profile monitoring down --volumes

# Check for port conflicts
netstat -an | grep -E "(5001|5002|5003|9090|3000)"

# Restart with fresh state
docker-compose -f icn-devnet/docker-compose.yml --profile monitoring up -d
```

### Test Timeouts

```bash
# Increase timeout
export ICN_E2E_TEST_TIMEOUT=900

# Check system resources
docker stats

# Review container logs
docker-compose -f icn-devnet/docker-compose.yml --profile monitoring logs
```

### Metrics Not Available

```bash
# Check Prometheus targets
curl http://localhost:9090/api/v1/targets

# Verify node metrics endpoints
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/metrics
```

## Contributing

When adding new tests:

1. Follow existing test patterns
2. Add comprehensive documentation
3. Include performance benchmarks
4. Update monitoring dashboards
5. Add troubleshooting guidance

## Support

For issues with E2E testing:

1. Check the troubleshooting section above
2. Review test logs in `test_results/`
3. Examine container logs and metrics
4. Refer to the main ICN documentation 