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

### Test Job Submission

```bash
# Submit a mesh job to any node
curl -X POST http://localhost:5001/mesh/submit \
  -H "Content-Type: application/json" \
  -d '{
    "manifest_cid": "cidv1-85-20-test_manifest",
    "spec_json": { "Echo": { "payload": "Hello Federation!" } },
    "cost_mana": 100
  }'

# Check job status (replace JOB_ID with actual ID from response)
curl http://localhost:5001/mesh/jobs/JOB_ID

# List all jobs
curl http://localhost:5001/mesh/jobs
```

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
                  (libp2p gossipsub)
```

### Services

- **ICN Nodes**: 3 containerized nodes with HTTP APIs and P2P networking
- **Prometheus**: Metrics collection (optional, with `--profile monitoring`)
- **Grafana**: Dashboard visualization (optional, with `--profile monitoring`)

## 📋 Configuration

### Environment Variables

Each node is configured via environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `ICN_NODE_NAME` | Human-readable node name | `Federation-Node-A` |
| `ICN_HTTP_LISTEN_ADDR` | HTTP API bind address | `0.0.0.0:7845` |
| `ICN_P2P_LISTEN_ADDR` | P2P networking bind address | `/ip4/0.0.0.0/tcp/4001` |
| `ICN_ENABLE_P2P` | Enable P2P networking | `true` |
| `ICN_BOOTSTRAP_PEERS` | Comma-separated bootstrap peers | `/ip4/node-a/tcp/4001/p2p/...` |
| `ICN_STORAGE_BACKEND` | Storage backend type | `memory` or `file` |

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

### Example `docker-compose.yml`

```yaml
services:
  icn-node-a:
    environment:
      - ICN_NODE_NAME=Node-A
      - ICN_HTTP_LISTEN_ADDR=0.0.0.0:7845
      - ICN_STORAGE_BACKEND=file
      - ICN_HTTP_API_KEY=devnet-a-key
    volumes:
      - ./data/node-a:/app/data

  icn-node-b:
    environment:
      - ICN_NODE_NAME=Node-B
      - ICN_HTTP_LISTEN_ADDR=0.0.0.0:7845
      - ICN_BOOTSTRAP_PEERS=/dns4/icn-node-a/tcp/4001/p2p/<NODE_A_PEER_ID>
      - ICN_HTTP_API_KEY=devnet-b-key
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
# Start with Prometheus and Grafana
docker-compose --profile monitoring up -d

# Access monitoring
# Prometheus: http://localhost:9090
# Grafana: http://localhost:3000 (admin/icnfederation)
```

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
curl http://localhost:5001/info

# Check node status and peer count
curl http://localhost:5001/status

# Submit and track a job
job_id=$(curl -s -X POST http://localhost:5001/mesh/submit \
  -H "Content-Type: application/json" \
  -d '{"manifest_cid":"test","spec_json":{"Echo":{"payload":"test"}},"cost_mana":50}' \
  | jq -r '.job_id')

curl http://localhost:5001/mesh/jobs/$job_id
```

## 🐛 Troubleshooting

### Common Issues

**Nodes not connecting:**
- Check Docker network connectivity
- Verify P2P ports are not blocked
- Check container logs: `docker-compose logs icn-node-a`

**Job submission failing:**
- Ensure nodes have mana (auto-initialized in devnet)
- Check job specification format
- Verify HTTP API is responding

**Build failures:**
- Ensure all ICN crates compile: `cargo build --release`
- Check Dockerfile dependencies
- Clear Docker build cache: `docker system prune -a`

### Logs

```bash
# View all logs
docker-compose logs

# View specific node logs
docker-compose logs icn-node-a

# Follow logs in real-time
docker-compose logs -f
```

### Reset Environment

```bash
# Complete cleanup
docker-compose down --volumes --remove-orphans
docker system prune -f
```

## 🎯 Next Steps

This devnet serves as the foundation for:
- Web UI integration testing
- CLI tool development
- Public demonstration deployments
- Load testing and performance optimization
- Multi-federation scenarios

## 📚 Related Documentation

- [Phase 3 HTTP Gateway](../PHASE_3_HTTP_GATEWAY_SUCCESS.md)
- [Phase 2B Cross-Node Execution](../PHASE_2B_SUCCESS.md)
- [ICN Core Architecture](../README.md) 