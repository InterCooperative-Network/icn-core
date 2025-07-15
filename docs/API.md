# ICN HTTP API

The ICN node exposes a REST interface for all functionality. This document describes the complete API surface, authentication requirements, and security features.

An OpenAPI specification is available at [docs/openapi.yaml](openapi.yaml) for automatic client generation.

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

### API Versioning

All endpoints use a versioned base path. The current prefix is `/api/v1`. When
breaking changes are introduced, a new prefix will be added (e.g. `/api/v2`).
Clients should include this prefix when making requests.

## Core Node Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| GET | `/info` | Node metadata including version and name | Optional |
| GET | `/status` | Current node status information | Optional |
| GET | `/health` | Basic health check | Optional |
| GET | `/ready` | Readiness probe for orchestration | Optional |
| GET | `/metrics` | Prometheus metrics for monitoring | Optional |

## DAG Storage Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/dag/put` | Store a content-addressed block | Yes |
| POST | `/dag/get` | Retrieve a block by CID | Yes |
| POST | `/dag/meta` | Retrieve metadata for a block | Yes |
| POST | `/dag/pin` | Pin a block to prevent pruning | Yes |
| POST | `/dag/unpin` | Remove a pin from a block | Yes |
| POST | `/dag/prune` | Garbage collect unpinned blocks | Yes |
| GET | `/dag/status` | Current DAG root and sync state | Optional |

### Example DAG Operations
```bash
# Store a DAG block and receive a CID string
curl -X POST https://localhost:8080/dag/put \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{"data": "your-data"}'
# => "bafy...cid-string"

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

# Pin a DAG block
curl -X POST https://localhost:8080/dag/pin \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{"cid": "your-cid-string"}'

# Unpin a DAG block
curl -X POST https://localhost:8080/dag/unpin \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{"cid": "your-cid-string"}'

# Check DAG synchronization status
curl -X GET https://localhost:8080/dag/status \
  -H "x-api-key: your-api-key"
```

## Transaction Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/transaction/submit` | Submit a signed transaction to the runtime | Yes |

## Resource Token Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| GET | `/tokens/classes` | List available token classes | Yes |
| POST | `/tokens/class` | Create a new token class | Yes |
| POST | `/tokens/mint` | Mint resource tokens | Yes |
| POST | `/tokens/transfer` | Transfer resource tokens | Yes |
| POST | `/tokens/burn` | Burn resource tokens | Yes |

## Governance Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/governance/submit` | Submit a governance proposal | Yes |
| POST | `/governance/vote` | Cast a vote on a proposal | Yes |
| POST | `/governance/delegate` | Delegate voting power to another DID | Yes |
| POST | `/governance/revoke` | Revoke a previous delegation | Yes |
| GET | `/governance/proposals` | List all proposals | Yes |
| GET | `/governance/proposal/:id` | Fetch a specific proposal | Yes |
| POST | `/governance/close` | Close voting and return tally | Yes |
| POST | `/governance/execute` | Execute an accepted proposal | Yes |

Proposers and voters may include a `credential_proof` object proving
membership or other criteria. See [zk_disclosure.md](zk_disclosure.md) for the
full JSON structure. When a policy enforces proofs, requests missing this field
are rejected.

### Example Governance Operations
```bash
# Submit a proposal
curl -X POST https://localhost:8080/governance/submit \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{
    "title": "Increase mesh job timeout",
    "description": "Increase the maximum timeout for mesh jobs to 300 seconds",
    "proposal_type": "ParameterChange",
    "credential_proof": {
      "issuer": "did:key:federation",
      "holder": "did:key:alice",
      "claim_type": "membership",
      "proof": "0x123456",
      "backend": "groth16"
    }
  }'

# Cast a vote
curl -X POST https://localhost:8080/governance/vote \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{
    "proposal_id": "proposal-uuid",
    "vote": "Yes",
    "credential_proof": {
      "issuer": "did:key:federation",
      "holder": "did:key:alice",
      "claim_type": "membership",
      "proof": "0x123456",
      "backend": "groth16"
    }
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
| POST | `/federation/peers` | Add a new federation peer | Yes |
| POST | `/federation/join` | Join a federation via peer ID | Yes |
| POST | `/federation/leave` | Voluntarily leave the federation | Yes |
| GET | `/federation/status` | Current federation status | Yes |

### Example Federation Operations
```bash
# Add a federation peer
curl -X POST https://localhost:8080/federation/peers \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{"peer_id": "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."}'

# List federation peers
curl -X GET https://localhost:8080/federation/peers \
  -H "x-api-key: your-api-key"
```

## Network & Discovery Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| GET | `/network/local-peer-id` | Return the node's peer ID | Yes |
| POST | `/network/connect` | Connect to a peer by multiaddress | Yes |
| GET | `/network/peers` | List currently connected peers | Yes |

## Contract & WASM Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/contracts` | Upload or update a WASM contract | Yes |

## Data Query Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/data/query` | Query stored data with filters | Yes |

## Circuit Registry Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/circuits/register` | Register Groth16 circuit parameters | Yes |
| GET | `/circuits/{slug}/{version}` | Fetch circuit verifying key | Yes |
| GET | `/circuits/{slug}` | List available circuit versions | Yes |

## Credential Proof Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| POST | `/identity/generate-proof` | Generate a zero-knowledge credential proof | Yes |
| POST | `/identity/verify-proof` | Verify a credential proof | Yes |

### Example Proof Generation
```bash
curl -X POST https://localhost:8080/identity/generate-proof \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{"issuer":"did:key:issuer","holder":"did:key:holder","claim_type":"age_over_18","schema":"bafyschema","backend":"groth16","public_inputs":{"birth_year":2000,"current_year":2020}}'
```
Response `200 OK`
```json
{
  "issuer": "did:key:issuer",
  "holder": "did:key:holder",
  "claim_type": "age_over_18",
  "proof": "...",
  "schema": "bafyschema",
  "backend": "groth16"
}
```

### Example Proof Verification
```bash
curl -X POST https://localhost:8080/identity/verify-proof \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-api-key" \
  -d '{"issuer":"did:key:issuer","holder":"did:key:holder","claim_type":"age_over_18","proof":"0xdead","schema":"bafyschema","backend":"groth16"}'
```
Response `200 OK`
```json
{ "verified": true }
```
## Example Requests & Responses

### GET `/info`
```bash
curl -i http://localhost:8080/info
```
Response `200 OK`
```json
{
  "version": "0.1.0-dev-functional",
  "name": "ICN Node",
  "status_message": "ready"
}
```

### GET `/status`
```bash
curl -i http://localhost:8080/status
```
Response `200 OK`
```json
{
  "is_online": true,
  "peer_count": 3,
  "current_block_height": 42,
  "version": "0.1.0-dev-functional"
}
```

### GET `/health`
```bash
curl -i http://localhost:8080/health
```
Response `200 OK`
```json
{
  "status": "ok",
  "timestamp": 1728000000,
  "uptime_seconds": 3600,
  "checks": {
    "runtime": "ok",
    "dag_store": "ok",
    "network": "ok",
    "mana_ledger": "ok"
  }
}
```

### GET `/ready`
```bash
curl -i http://localhost:8080/ready
```
Response `200 OK`
```json
{
  "ready": true,
  "timestamp": 1728000000,
  "checks": {
    "can_serve_requests": true,
    "mana_ledger_available": true,
    "dag_store_available": true,
    "network_initialized": true
  }
}
```

### GET `/metrics`
```bash
curl -i http://localhost:8080/metrics
```
Response `200 OK`
```text
# HELP icn_requests_total Total HTTP requests
icn_requests_total{path="/info"} 5
```

### POST `/dag/put`
```bash
curl -X POST http://localhost:8080/dag/put \
  -H "Content-Type: application/json" \
  -d '{"data":"aGVsbG8="}'
```
Response `200 OK`
```json
"bafyblockcid"
```

### POST `/dag/get`
```bash
curl -X POST http://localhost:8080/dag/get \
  -H "Content-Type: application/json" \
  -d '{"cid":"bafyblockcid"}'
```
Response `200 OK`
```json
"aGVsbG8="
```

### POST `/dag/meta`
```bash
curl -X POST http://localhost:8080/dag/meta \
  -H "Content-Type: application/json" \
  -d '{"cid":"bafyblockcid"}'
```
Response `200 OK`
```json
{
  "size": 1024,
  "timestamp": 1728000000,
  "author_did": "did:key:alice",
  "links": []
}
```

### POST `/dag/pin`
```bash
curl -X POST http://localhost:8080/dag/pin \
  -H "Content-Type: application/json" \
  -d '{"cid":"bafyblockcid","ttl":3600}'
```
Response `200 OK`
```json
{
  "version": 1,
  "codec": 0,
  "hash_alg": 1,
  "hash_bytes": "AAEC"
}
```

### POST `/dag/unpin`
```bash
curl -X POST http://localhost:8080/dag/unpin \
  -H "Content-Type: application/json" \
  -d '{"cid":"bafyblockcid"}'
```
Response `200 OK`
```json
{
  "version": 1,
  "codec": 0,
  "hash_alg": 1,
  "hash_bytes": "AAEC"
}
```

### POST `/dag/prune`
```bash
curl -X POST http://localhost:8080/dag/prune -H "Content-Type: application/json" -d '{}'
```
Response `200 OK`
```json
{"pruned": true}
```

### POST `/transaction/submit`
```bash
curl -X POST http://localhost:8080/transaction/submit \
  -H "Content-Type: application/json" \
  -d '{"id":"tx-1","payload_type":"Transfer"}'
```
Response `200 OK`
```json
"tx-1"
```

### POST `/governance/submit`
```bash
curl -X POST http://localhost:8080/governance/submit \
  -H "Content-Type: application/json" \
  -d '{
    "proposer_did": "did:key:alice",
    "proposal": {},
    "credential_proof": {
      "issuer": "did:key:federation",
      "holder": "did:key:alice",
      "claim_type": "membership",
      "proof": "0x123456",
      "backend": "groth16"
    }
  }'
```
Response `200 OK`
```json
"prop-1"
```

### POST `/governance/vote`
```bash
curl -X POST http://localhost:8080/governance/vote \
  -H "Content-Type: application/json" \
  -d '{
    "voter_did": "did:key:alice",
    "proposal_id": "prop-1",
    "vote_option": "Yes",
    "credential_proof": {
      "issuer": "did:key:federation",
      "holder": "did:key:alice",
      "claim_type": "membership",
      "proof": "0x123456",
      "backend": "groth16"
    }
  }'
```
Response `200 OK`
```json
{"accepted": true}
```

### POST `/governance/delegate`
```bash
curl -X POST http://localhost:8080/governance/delegate \
  -H "Content-Type: application/json" \
  -d '{"from_did":"did:key:alice","to_did":"did:key:bob"}'
```
Response `200 OK`
```json
{"delegated": true}
```

### POST `/governance/revoke`
```bash
curl -X POST http://localhost:8080/governance/revoke \
  -H "Content-Type: application/json" \
  -d '{"from_did":"did:key:alice"}'
```
Response `200 OK`
```json
{"revoked": true}
```

### GET `/governance/proposals`
```bash
curl -i http://localhost:8080/governance/proposals
```
Response `200 OK`
```json
[
  {"proposal_id":"prop-1","description":"Increase timeout"}
]
```

### GET `/governance/proposal/:id`
```bash
curl -i http://localhost:8080/governance/proposal/prop-1
```
Response `200 OK`
```json
{
  "proposal_id": "prop-1",
  "description": "Increase timeout",
  "status": "Open"
}
```

### POST `/governance/close`
```bash
curl -X POST http://localhost:8080/governance/close \
  -H "Content-Type: application/json" \
  -d '{"proposal_id":"prop-1"}'
```
Response `200 OK`
```json
{
  "status": "Accepted",
  "yes": 2,
  "no": 0,
  "abstain": 1
}
```

### POST `/governance/execute`
```bash
curl -X POST http://localhost:8080/governance/execute \
  -H "Content-Type: application/json" \
  -d '{"proposal_id":"prop-1"}'
```
Response `200 OK`
```json
{"executed": true}
```

### POST `/mesh/submit`
```bash
curl -X POST http://localhost:8080/mesh/submit \
  -H "Content-Type: application/json" \
  -d '{"manifest_cid":"bafyjobmanifest","spec_bytes":"BASE64_SPEC"}'
```
Response `200 OK`
```json
{"job_id":"job-123"}
```

### GET `/mesh/jobs`
```bash
curl -i http://localhost:8080/mesh/jobs
```
Response `200 OK`
```json
{
  "jobs": [
    {"job_id":"job-123","status":"Running"}
  ]
}
```

### GET `/mesh/jobs/:job_id`
```bash
curl -i http://localhost:8080/mesh/jobs/job-123
```
Response `200 OK`
```json
{"job_id":"job-123","status":"Completed"}
```

### POST `/mesh/receipts`
```bash
curl -X POST http://localhost:8080/mesh/receipts \
  -H "Content-Type: application/json" \
  -d '{"job_id":"job-123","executor_did":"did:key:executor","success":true}'
```
Response `200 OK`
```json
{"accepted": true}
```

### GET `/federation/peers`
```bash
curl -i http://localhost:8080/federation/peers
```
Response `200 OK`
```json
[
  "12D3KooWpeer1",
  "12D3KooWpeer2"
]
```

### POST `/federation/peers`
```bash
curl -X POST http://localhost:8080/federation/peers \
  -H "Content-Type: application/json" \
  -d '{"peer":"/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."}'
```
Response `200 OK`
```json
{"peer":"/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."}
```

### POST `/federation/join`
```bash
curl -X POST http://localhost:8080/federation/join \
  -H "Content-Type: application/json" \
  -d '{"peer":"/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."}'
```
Response `200 OK`
```json
{"joined":"/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."}
```

### POST `/federation/leave`
```bash
curl -X POST http://localhost:8080/federation/leave \
  -H "Content-Type: application/json" \
  -d '{"peer":"/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."}'
```
Response `200 OK`
```json
{"left":"/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."}
```

### GET `/federation/status`
```bash
curl -i http://localhost:8080/federation/status
```
Response `200 OK`
```json
{
  "peer_count": 3,
  "peers": ["12D3KooWpeer1","12D3KooWpeer2","12D3KooWpeer3"]
}
```

### POST `/network/connect`
```bash
curl -X POST http://localhost:8080/network/connect \
  -H "Content-Type: application/json" \
  -d '{"peer":"/ip4/127.0.0.1/tcp/4001/p2p/12D3KooW..."}'
```
Response `200 OK`
```json
{"connected": true}
```

### GET `/network/local-peer-id`
```bash
curl -i http://localhost:8080/network/local-peer-id
```
Response `200 OK`
```json
{"peer_id":"12D3KooW..."}
```

### GET `/network/peers`
```bash
curl -i http://localhost:8080/network/peers
```
Response `200 OK`
```json
[
  "12D3KooWpeer1",
  "12D3KooWpeer2"
]
```

### POST `/data/query`
```bash
curl -X POST http://localhost:8080/data/query \
  -H "Content-Type: application/json" \
  -d '{"cid":"bafyblockcid"}'
```
Response `200 OK`
```json
{
  "cid": {"version":1,"codec":0,"hash_alg":1,"hash_bytes":"AAEC"},
  "data": "aGVsbG8="
}
```

### POST `/contracts`
```bash
curl -X POST http://localhost:8080/contracts \
  -H "Content-Type: application/json" \
  -d '{"source":"(module ...)"}'
```
Response `200 OK`
```json
{"manifest_cid":"bafycontractmanifest"}
```

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
icn-cli federation peers
icn-cli governance propose "Increase timeout to 300s"
icn-cli dag meta '{"cid":"bafy..."}'
```

### HTTP Clients
Standard HTTP clients can interact with the API. See the examples above for curl usage patterns.

### Rust SDK
The `icn-sdk` crate provides a typed client for interacting with the node. Add it to your `Cargo.toml`:

```toml
icn-sdk = { path = "../crates/icn-sdk" }
tokio = { version = "1", features = ["full"] }
```

Example usage:

```rust
use icn_sdk::IcnClient;

# #[tokio::main] // for a real application
async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let client = IcnClient::new("http://127.0.0.1:7845")?;
    let info = client.info().await?;
    println!("node name {}", info.name);
    Ok(())
}
```

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

