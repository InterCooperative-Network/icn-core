# ICN HTTP API

The ICN node exposes a REST interface for all functionality. This document describes the complete API surface, authentication requirements, and security features.

## Authentication & Security

### API Authentication
- **API Key**: All endpoints require the configured `x-api-key` header if an API key is set via `--api-key`
- **Bearer Token**: If an `auth_token` is configured via `--auth-token` or `--auth-token-path`, requests must also include `Authorization: Bearer <token>`
- **Rate Limiting**: When no API key is set, the `--open-rate-limit` option controls unauthenticated requests per minute

### TLS/HTTPS Support
- **TLS Configuration**: When `--tls-cert-path` and `--tls-key-path` are provided, the server only accepts HTTPS connections
- **Certificate Management**: Supports standard X.509 certificates for production deployments
- **Security Headers**: HTTPS endpoints include appropriate security headers

### Example Authenticated Request
```bash
curl -X GET https://localhost:8080/info \
  -H "x-api-key: your-api-key" \
  -H "Authorization: Bearer your-auth-token"
```

## Core Node Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| GET | `/info` | Node metadata including version and name | Optional |
| GET | `/status` | Current node health and peer connectivity | Optional |
| GET | `/metrics` | Prometheus metrics for monitoring | Optional |

## DAG Storage Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/dag/put` | Store a content-addressed block | Yes |
| POST | `/dag/get` | Retrieve a block by CID | Yes |
| POST | `/dag/meta` | Retrieve metadata for a block | Yes |

### Example DAG Operations
```bash
# Store a DAG block
curl -X POST https://localhost:8080/dag/put \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{"data": "your-data", "links": []}'

# Retrieve a DAG block
curl -X POST https://localhost:8080/dag/get \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{"cid": "your-cid-string"}'

# Retrieve DAG block metadata
curl -X POST https://localhost:8080/dag/meta \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{"cid": "your-cid-string"}'
```

## Governance Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/governance/submit` | Submit a governance proposal | Yes |
| POST | `/governance/vote` | Cast a vote on a proposal | Yes |
| GET | `/governance/proposals` | List all proposals | Yes |
| GET | `/governance/proposal/:id` | Fetch a specific proposal | Yes |
| POST | `/governance/close` | Close voting and return tally | Yes |

### Example Governance Operations
```bash
# Submit a proposal
curl -X POST https://localhost:8080/governance/submit \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{
    "title": "Increase mesh job timeout",
    "description": "Increase the maximum timeout for mesh jobs to 300 seconds",
    "proposal_type": "ParameterChange"
  }'

# Cast a vote
curl -X POST https://localhost:8080/governance/vote \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{
    "proposal_id": "proposal-uuid",
    "vote": "Yes"
  }'
```

The `/governance/close` endpoint returns a JSON object:

```json
{
  "status": "Accepted",
  "yes": 2,
  "no": 0,
  "abstain": 1
}
```

`status` is the final `ProposalStatus` string, and the numeric fields represent
the counted votes.

## Mesh Computing Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/mesh/submit` | Submit a mesh job for distributed execution | Yes |
| GET | `/mesh/jobs` | List all mesh jobs | Yes |
| GET | `/mesh/jobs/:job_id` | Get the status of a specific job | Yes |
| POST | `/mesh/receipts` | Submit an execution receipt | Yes |

### Example Mesh Operations
```bash
# Submit a mesh job
curl -X POST https://localhost:8080/mesh/submit \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{
    "command": "echo hello world",
    "max_cost": 100,
    "timeout_seconds": 60
  }'

# Check job status
curl -X GET https://localhost:8080/mesh/jobs/job-uuid \
  -H "x-api-key: your-api-key"
```

## Federation Management Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| GET | `/federation/peers` | List known federation peers | Yes |
| POST | `/federation/join` | Join a federation by adding a peer | Yes |
| POST | `/federation/leave` | Leave a federation by removing a peer | Yes |
| GET | `/federation/status` | Get current federation status | Yes |

### Example Federation Operations
```bash
# Join a federation
curl -X POST https://localhost:8080/federation/join \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{"peer_id": "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."}'

# Get federation status
curl -X GET https://localhost:8080/federation/status \
  -H "x-api-key: your-api-key"

# Leave a federation
curl -X POST https://localhost:8080/federation/leave \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{"peer_id": "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."}'
```

## Network & Discovery Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/network/discover-peers` | Trigger peer discovery | Yes |
| POST | `/network/send-message` | Send a message to a specific peer | Yes |
| POST | `/network/connect` | Connect to a peer by multiaddress | Yes |
| GET | `/network/peers` | List currently connected peers | Yes |

## Contract & WASM Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/contracts` | Upload or update a WASM contract | Yes |
| GET | `/contracts` | List deployed contracts | Yes |
| POST | `/contracts/execute` | Execute a WASM contract | Yes |

## Data Query Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/data/query` | Query stored data with filters | Yes |
| GET | `/data/stats` | Get storage statistics | Yes |

## Error Responses

All endpoints return structured error responses in JSON format:

```json
{
  "error": "ErrorType",
  "message": "Human-readable error description",
  "details": {
    "additional": "context information"
  },
  "timestamp": "2025-01-01T00:00:00Z"
}
```

Common HTTP status codes:
- `200` - Success
- `400` - Bad Request (invalid parameters)
- `401` - Unauthorized (missing or invalid authentication)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found (resource doesn't exist)
- `429` - Too Many Requests (rate limit exceeded)
- `500` - Internal Server Error

## Configuration Options

### Node Startup
```bash
# Basic HTTP server
./icn-node --http-listen-addr 0.0.0.0:8080

# With API key
./icn-node --http-listen-addr 0.0.0.0:8080 --api-key "secure-api-key"

# With bearer token authentication
./icn-node --http-listen-addr 0.0.0.0:8080 \
  --api-key "secure-api-key" \
  --auth-token "bearer-token"

# With TLS (HTTPS only)
./icn-node --http-listen-addr 0.0.0.0:8443 \
  --api-key "secure-api-key" \
  --auth-token "bearer-token" \
  --tls-cert-path ./certs/server.crt \
  --tls-key-path ./certs/server.key
```

### Rate Limiting
```bash
# Allow 100 unauthenticated requests per minute
./icn-node --http-listen-addr 0.0.0.0:8080 --open-rate-limit 100
```

For tuning circuit breaker thresholds and retry delays, see the
[deployment guide](docs/deployment-guide.md#circuit-breaker-and-retry).

## Monitoring & Observability

### Prometheus Metrics
The `/metrics` endpoint exposes Prometheus-compatible metrics:
- Request counts and latencies per endpoint
- Network peer connection status
- Mesh job execution statistics
- DAG storage utilization
- Mana balance and transaction metrics

### Audit Logging
All significant operations are logged to the `audit` log target:
- Job submissions and completions
- Governance proposals and votes
- Federation peer changes
- Authentication failures

## Client Libraries

### ICN CLI
The `icn-cli` tool provides a convenient command-line interface to all API endpoints:

```bash
# Configure CLI to use your node
export ICN_NODE_URL="https://localhost:8443"
export ICN_API_KEY="your-api-key"
export ICN_AUTH_TOKEN="your-bearer-token"

# Use CLI commands
icn-cli info
icn-cli federation status
icn-cli governance propose "Increase timeout to 300s"
icn-cli dag meta '{"cid":"bafy..."}'
```

### HTTP Clients
Standard HTTP clients can interact with the API. See the examples above for curl usage patterns.

## Security Considerations

1. **Always use HTTPS** in production environments
2. **Protect API keys and bearer tokens** - treat them as passwords
3. **Rotate authentication tokens** regularly
4. **Monitor audit logs** for suspicious activity
5. **Use rate limiting** to prevent abuse
6. **Validate all inputs** on the client side before submission
7. **Implement proper error handling** for all API calls

## Support

For API support and questions:
- **Documentation**: [docs.intercooperative.network](https://docs.intercooperative.network)
- **GitHub Issues**: [github.com/InterCooperative-Network/icn-core/issues](https://github.com/InterCooperative-Network/icn-core/issues)
- **Community**: [GitHub Discussions](https://github.com/InterCooperative-Network/icn-core/discussions)

