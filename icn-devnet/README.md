# 🌐 ICN Federation Devnet

A containerized 3-node ICN federation for development, testing, and demonstration purposes.

## 🚀 Quick Start

### Launch the Federation

```bash
# From the icn-devnet directory
./launch_federation.sh
```

This will:
- Start 3 ICN nodes with P2P networking
- Wait for network convergence
- Test mesh job submission and execution
- Display federation status

### Access the Nodes

Once running, you can access:

- **Node A**: http://localhost:5001 (Bootstrap node)
- **Node B**: http://localhost:5002 (Worker node)
- **Node C**: http://localhost:5003 (Worker node)

### Step-by-Step Setup

1. **Install prerequisites** – Docker and Docker Compose must be available.
2. **Clone** this repository and `cd` into `icn-devnet`.
3. **Choose a storage backend** by editing `ICN_STORAGE_BACKEND` in
   `docker-compose.yml`. Use `memory` for ephemeral runs or `file` to persist
   data under each node's `data/` volume.
4. **Enable security** (optional): set an `ICN_HTTP_API_KEY` for each node and,
   if TLS is desired, provide `ICN_TLS_CERT_PATH` and `ICN_TLS_KEY_PATH`.
5. **Launch the federation** with `./launch_federation.sh`.

### 10-Node Load Test

The repository includes a helper script for spinning up a **10 node** devnet
and submitting a configurable number of test jobs:

```bash
# From the repository root
NUM_JOBS=50 ./scripts/run_10node_devnet.sh
```

`NUM_JOBS` controls how many jobs are submitted to the first node (default
`20`). The script automatically starts Prometheus and Grafana, so you can run
`docker-compose --profile monitoring up -d` to visualize metrics. See the
[deployment guide’s monitoring section](../docs/deployment-guide.md#monitoring-with-prometheus--grafana)
for more details.

### Test Job Submission

> **⚠️ Important**: Use `X-API-Key` header (not `Authorization: Bearer`)

```bash
# Submit a mesh job to any node
curl -X POST http://localhost:5001/mesh/submit \
  -H "Content-Type: application/json" \
  -H "X-API-Key: devnet-a-key" \
  -d '{
    "manifest_cid": "bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354",
    "spec_json": {
      "kind": {
        "Echo": {
          "payload": "Hello Federation!"
        }
      }
    },
    "cost_mana": 100
  }'

# Check job status (replace JOB_ID with actual ID from response)
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/mesh/jobs/JOB_ID

# List all jobs
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/mesh/jobs
```

**Expected Response:**
```json
{
  "job_id": "bafkrfz2acgrvhdag6q2rs7h5buh2i6omqqhffnrvatziwrlrnx3elqyp"
}
```

## 10-Node Federation

For extended testing you can spin up a larger federation using the helper
`scripts/run_10node_devnet.sh` script. Run it from the repository root to start
ten nodes and submit a batch of sample jobs:

```bash
scripts/run_10node_devnet.sh
```

Set the `NUM_JOBS` environment variable to control how many jobs are submitted.
Prometheus and Grafana can be enabled by passing `--profile monitoring` to
`docker-compose` before running the script:

```bash
docker-compose --profile monitoring up -d
scripts/run_10node_devnet.sh
```

This exposes Prometheus at `http://localhost:9090` and Grafana at
`http://localhost:3000`.

## 🏗️ Architecture

### Network Topology

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Node A    │────▶│   Node B    │────▶│   Node C    │
│ (Bootstrap) │     │  (Worker)   │     │  (Worker)   │
│ :5001       │     │ :5002       │     │ :5003       │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                  P2P Mesh Network
                  (libp2p gossipsub + mDNS)
```

### Services

- **ICN Nodes**: 3 containerized nodes with HTTP APIs and P2P networking
- **Prometheus**: Metrics collection (optional, with `--profile monitoring`)
- **Grafana**: Dashboard visualization (optional, with `--profile monitoring`)
- **Alertmanager**: Alert routing (optional, with `--profile monitoring`)

### Mana System

Each node is automatically initialized with **1000 mana** on startup. This enables:
- Job submission and execution
- Network participation
- Resource usage tracking

Mana is automatically refunded if jobs fail (e.g., no bids received).

## 📋 Configuration

### Environment Variables

Each node is configured via environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `ICN_NODE_NAME` | Human-readable node name | `Federation-Node-A` |
| `ICN_HTTP_LISTEN_ADDR` | HTTP API bind address | `0.0.0.0:7845` |
| `ICN_P2P_LISTEN_ADDR` | P2P networking bind address | `/ip4/0.0.0.0/tcp/4001` |
| `ICN_ENABLE_P2P` | Enable P2P networking | `true` |
| `ICN_ENABLE_MDNS` | Enable mDNS peer discovery | `true` |
| `ICN_BOOTSTRAP_PEERS` | Comma-separated bootstrap peers | `/ip4/node-a/tcp/4001/p2p/...` |
| `ICN_STORAGE_BACKEND` | Storage backend type | `memory` or `file` |
| `ICN_HTTP_API_KEY` | API authentication key | `devnet-a-key` |

### Ports

| Service | Host Port | Container Port | Purpose |
|---------|-----------|----------------|---------|
| Node A HTTP | 5001 | 7845 | REST API |
| Node A P2P | 4001 | 4001 | libp2p networking |
| Node B HTTP | 5002 | 7845 | REST API |
| Node B P2P | 4002 | 4001 | libp2p networking |
| Node C HTTP | 5003 | 7845 | REST API |
| Node C P2P | 4003 | 4001 | libp2p networking |
| Prometheus | 9090 | 9090 | Metrics (optional) |
| Grafana | 3000 | 3000 | Dashboard (optional) |
| Alertmanager | 9093 | 9093 | Alert routing (optional) |

### Example `docker-compose.yml`

```yaml
services:
  icn-node-a:
    environment:
      - ICN_NODE_NAME=Node-A
      - ICN_HTTP_LISTEN_ADDR=0.0.0.0:7845
      - ICN_STORAGE_BACKEND=file
      - ICN_HTTP_API_KEY=devnet-a-key
      - ICN_ENABLE_MDNS=true
    volumes:
      - ./data/node-a:/app/data

  icn-node-b:
    environment:
      - ICN_NODE_NAME=Node-B
      - ICN_HTTP_LISTEN_ADDR=0.0.0.0:7845
      - ICN_BOOTSTRAP_PEERS=/dns4/icn-node-a/tcp/4001/p2p/<NODE_A_PEER_ID>
      - ICN_HTTP_API_KEY=devnet-b-key
      - ICN_ENABLE_MDNS=true
```

## 🛠️ Advanced Usage

### Manual Control

```bash
# Start federation manually
docker-compose up -d

# View logs
docker-compose logs -f

# Scale to 5 nodes
docker-compose up --scale icn-node-b=3 -d

# Stop federation
docker-compose down

# Clean up everything
docker-compose down --volumes --remove-orphans
```

### With Monitoring

```bash
# Start with Prometheus, Grafana, and Alertmanager
docker-compose --profile monitoring up -d

# Access monitoring
# Prometheus: http://localhost:9090
# Grafana: http://localhost:3000 (admin/icnfederation)
# Alertmanager: http://localhost:9093
```

The default Alertmanager configuration sends emails to
`alerts@intercooperative.network`. Modify `alertmanager.yml` to customize
receivers or routing rules. Sample alert rules live in `alert.rules.yml` and
cover basic node availability. Extend these rules or hook Alertmanager into your
own notification channels as needed.

### Grafana Dashboards

Pre-built dashboards are available under `grafana/`.

1. Start the monitoring profile with `docker-compose --profile monitoring up -d`.
2. Open Grafana at [http://localhost:3000](http://localhost:3000) (admin/icnfederation).
3. Navigate to **Dashboards → Import** and upload any of the JSON files from `grafana/` such as `icn-devnet-overview.json`, `icn-jobs-network.json`, or `icn-governance.json`.

### Development Mode

```bash
# Rebuild images and start
docker-compose up --build -d

# Follow logs from all services
docker-compose logs -f
```

## 🧪 Testing

### Automated Tests

The launch script includes automated tests:
- Node health checks
- P2P network convergence
- Mesh job submission and tracking

### Manual Testing

```bash
# Check node info
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/info

# Check node status and peer count
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/status

# Submit and track a job
job_id=$(curl -s -X POST http://localhost:5001/mesh/submit \
  -H "Content-Type: application/json" \
  -H "X-API-Key: devnet-a-key" \
  -d '{
    "manifest_cid": "bafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354",
    "spec_json": {
      "kind": {
        "Echo": {
          "payload": "test message"
        }
      }
    },
    "cost_mana": 50
  }' | jq -r '.job_id')

curl -H "X-API-Key: devnet-a-key" http://localhost:5001/mesh/jobs/$job_id
```

## 🐛 Troubleshooting

### Common Issues

**"Account not found" Error during Job Submission:**

This was a critical bug that has been **FIXED** in the current version. If you encounter this error:

1. **Check node startup logs** for mana initialization:
   ```bash
   docker-compose logs icn-node-a | grep -i "mana\|initialized"
   ```
   
   You should see: `✅ Node initialized with 1000 mana`

2. **If mana initialization failed**, check for detailed error messages in logs:
   ```bash
   docker-compose logs icn-node-a | grep -i "error\|failed"
   ```

3. **Restart the node** if mana initialization failed:
   ```bash
   docker-compose restart icn-node-a
   ```

**Wrong API Key Header:**
- Use `X-API-Key: devnet-a-key` (not `Authorization: Bearer`)
- Each node has its own API key: `devnet-a-key`, `devnet-b-key`, `devnet-c-key`

**Invalid JSON Structure:**
The job submission requires this exact structure:
```json
{
  "manifest_cid": "string",
  "spec_json": {
    "kind": {
      "Echo": {
        "payload": "your message"
      }
    }
  },
  "cost_mana": 50
}
```

**Nodes not connecting:**
- Check Docker network connectivity
- Verify P2P ports are not blocked
- mDNS should automatically discover peers
- Check container logs: `docker-compose logs icn-node-a`

**Job stuck in "pending" or "failed - no bids":**
- This is expected in single-node testing (no other executors available)
- Mana is automatically refunded when jobs fail
- For multi-node execution, ensure all nodes are running and connected

**Build failures:**
- Ensure all ICN crates compile: `cargo build --release`
- Check Dockerfile dependencies
- Clear Docker build cache: `docker system prune -a`

### Debugging Commands

```bash
# View all logs with timestamps
docker-compose logs -t

# View specific node logs
docker-compose logs icn-node-a

# Follow logs in real-time with grep filter
docker-compose logs -f | grep -E "(ERROR|WARN|mana|job)"

# Check mana balance and job states
curl -H "X-API-Key: devnet-a-key" http://localhost:5001/mesh/jobs
```

### Reset Environment

```bash
# Complete cleanup
docker-compose down --volumes --remove-orphans
docker system prune -f

# Rebuild and restart
docker-compose up --build -d
```

For persistent deployments see the [DAG Backup and Restore guide](../docs/deployment-guide.md#dag-backup-and-restore).

## 🔧 Recent Fixes

### Mana Initialization Fix (v1.0.1)

**Issue**: Nodes were failing to initialize mana accounts, causing `"Account not found"` errors during job submission.

**Root Cause**: Silent panic in `crates/icn-node/src/node.rs` when mana initialization failed due to improper error handling.

**Solution**: Replaced `.expect()` call with proper error handling that logs detailed error information and gracefully handles initialization failures.

**Verification**: Look for `✅ Node initialized with 1000 mana` in startup logs.

## 🎯 Next Steps

This devnet serves as the foundation for:
- Web UI integration testing
- CLI tool development
- Public demonstration deployments
- Load testing and performance optimization
- Multi-federation scenarios

## 📚 Related Documentation

- [Phase 3 HTTP Gateway](../PHASE_3_HTTP_GATEWAY_SUCCESS.md)
- [Phase 2B Cross-Node Execution](../PHASE_2B_SUCCESS.md)- [ICN Core Architecture](../README.md)
- [10 Node Devnet Results](../docs/ten_node_results.md)