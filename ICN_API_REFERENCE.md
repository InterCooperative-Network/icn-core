# ICN Core Complete API Reference

> **⚠️ Development Status**: Many API endpoints return mock data or connect to stub implementations. This reference describes the intended API structure, but actual functionality may be limited.

**Base URL:** `http://127.0.0.1:7845`  
**Version:** 0.2.0-beta  
**Base Path:** `/api/v1`  
**Content-Type:** `application/json`

---

## 🚧 **API Overview**

ICN Core provides an **experimental HTTP API** with 60+ endpoint structures covering cooperative digital infrastructure concepts. While endpoints are well-structured, many connect to stub implementations or return mock data.

**Current Reality**: API server starts and handles requests, but backend services may be incomplete. Use for development and testing only.

### **Authentication**
```http
# API Key
x-api-key: your-api-key

# Bearer Token  
Authorization: Bearer your-token

# DID-based (Planned)
Authorization: DID did:example:123... <signature>
```

---

## 📊 **Complete Endpoint Reference**

### **System Information (4 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/info` | GET | Node information and DID | ✅ |
| `/status` | GET | Real-time node status | ✅ |
| `/health` | GET | Health check with detailed metrics | ✅ |
| `/ready` | GET | Readiness probe | ✅ |
| `/metrics` | GET | Prometheus metrics | ✅ |

---

### **Governance System (8 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/governance/proposals` | GET | List all governance proposals | ✅ |
| `/governance/proposal/{id}` | GET | Get specific proposal details | ✅ |
| `/governance/submit` | POST | Submit new governance proposal | ✅ |
| `/governance/vote` | POST | Cast vote on proposal | ✅ |
| `/governance/delegate` | POST | Delegate voting power | ✅ |
| `/governance/revoke` | POST | Revoke delegation | ✅ |
| `/governance/close` | POST | Close voting on proposal | ✅ |
| `/governance/execute` | POST | Execute approved proposal | ✅ |

---

### **Identity & Credentials (10 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/identity/verify` | POST | Verify credential proof | ✅ |
| `/identity/generate-proof` | POST | Generate ZK credential proof | ✅ |
| `/identity/verify-proof` | POST | Verify ZK proof | ✅ |
| `/identity/verify/revocation` | POST | Verify revocation proof | ✅ |
| `/identity/verify/batch` | POST | Batch proof verification | ✅ |
| `/identity/credentials/issue` | POST | Issue new credential | ✅ |
| `/identity/credentials/verify` | POST | Verify credential authenticity | ✅ |
| `/identity/credentials/revoke` | POST | Revoke credential | ✅ |
| `/identity/credentials/schemas` | GET | List credential schemas | ✅ |
| `/identity/credentials/disclose` | POST | Selective credential disclosure | ✅ |
| `/identity/credentials/{cid}` | GET | Get credential by CID | ✅ |

---

### **Mesh Computing (12 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/mesh/submit` | POST | Submit mesh job | ✅ |
| `/mesh/jobs` | GET | List all mesh jobs | ✅ |
| `/mesh/jobs/{job_id}` | GET | Get job status and details | ✅ |
| `/mesh/jobs/{job_id}/progress` | GET | Get job progress updates | ✅ |
| `/mesh/jobs/{job_id}/stream` | GET | Real-time job output stream | ✅ |
| `/mesh/jobs/{job_id}/cancel` | POST | Cancel running job | ✅ |
| `/mesh/jobs/{job_id}/resume` | POST | Resume paused job | ✅ |
| `/mesh/metrics` | GET | Mesh network metrics | ✅ |
| `/mesh/receipt` | POST | Submit execution receipt | ✅ |
| `/mesh/stub/bid` | POST | Submit test bid (dev) | ✅ |
| `/mesh/stub/receipt` | POST | Submit test receipt (dev) | ✅ |

---

### **Federation Management (8 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/federation/peers` | GET | List federation peers | ✅ |
| `/federation/peers` | POST | Add federation peer | ✅ |
| `/federation/join` | POST | Join federation | ✅ |
| `/federation/leave` | POST | Leave federation | ✅ |
| `/federation/status` | GET | Federation status | ✅ |
| `/federation/init` | POST | Initialize federation | ✅ |
| `/federation/sync` | POST | Synchronize federation state | ✅ |

---

### **Cooperative Management (7 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/cooperative/register` | POST | Register new cooperative | ✅ |
| `/cooperative/search` | POST | Search cooperatives | ✅ |
| `/cooperative/profile/{did}` | GET | Get cooperative profile | ✅ |
| `/cooperative/trust` | POST | Add trust relationship | ✅ |
| `/cooperative/trust/{did}` | GET | Get trust information | ✅ |
| `/cooperative/capabilities/{type}` | GET | Get capability providers | ✅ |
| `/cooperative/registry/stats` | GET | Registry statistics | ✅ |

---

### **DAG Storage (8 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/dag/put` | POST | Store DAG block | ✅ |
| `/dag/get` | POST | Retrieve DAG block | ✅ |
| `/dag/meta` | POST | Get block metadata | ✅ |
| `/dag/root` | GET | Get DAG root hash | ✅ |
| `/dag/status` | GET | DAG storage status | ✅ |
| `/dag/pin` | POST | Pin block (prevent GC) | ✅ |
| `/dag/unpin` | POST | Unpin block | ✅ |
| `/dag/prune` | POST | Prune unpinned blocks | ✅ |

---

### **Network Operations (3 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/network/local-peer-id` | GET | Get local peer ID | ✅ |
| `/network/connect` | POST | Connect to peer | ✅ |
| `/network/peers` | GET | List connected peers | ✅ |

---

### **Account & Economics (5 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/account/{did}/mana` | GET | Get mana balance | ✅ |
| `/keys` | GET | Get node keys | ✅ |
| `/reputation/{did}` | GET | Get reputation score | ✅ |
| `/transaction/submit` | POST | Submit transaction | ✅ |
| `/resources/event` | POST | Submit resource event | ✅ |
| `/resources/ledger` | GET | Get resource ledger | ✅ |

---

### **Zero-Knowledge Circuits (3 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/circuits/register` | POST | Register ZK circuit | ✅ |
| `/circuits/{slug}/{version}` | GET | Get circuit by version | ✅ |
| `/circuits/{slug}` | GET | List circuit versions | ✅ |

---

### **Data Operations (2 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/data/query` | POST | Query stored data | ✅ |
| `/contracts` | POST | Upload WASM contract | ✅ |

---

### **Real-Time Communication (1 endpoint)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/ws` | WebSocket | Real-time event stream | ✅ |

---

### **Sync & Monitoring (2 endpoints)**

| Endpoint | Method | Description | Status |
|----------|--------|-------------|--------|
| `/sync/status` | GET | Synchronization status | ✅ |

---

## 📋 **API Usage Examples**

### **Submit Governance Proposal**
```bash
curl -X POST http://localhost:7845/governance/submit \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-key" \
  --data '{
    "proposer_did": "did:key:alice",
    "proposal": {
      "type": "SystemParameterChange",
      "data": {
        "param": "mana_regeneration_rate", 
        "value": "0.1"
      }
    },
    "description": "Increase mana regeneration rate",
    "duration_secs": 604800,
    "quorum": 10,
    "threshold": 0.6
  }'
```

### **Submit Mesh Job**
```bash
curl -X POST http://localhost:7845/mesh/submit \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-key" \
  --data '{
    "job_spec": {
      "image": "python:3.9",
      "command": ["python", "-c", "print(\"Hello, ICN!\")"],
      "resources": {
        "cpu_cores": 1,
        "memory_mb": 512,
        "storage_mb": 1024
      }
    },
    "submitter_did": "did:key:submitter",
    "max_cost": 1000,
    "timeout_seconds": 300
  }'
```

### **Join Federation**
```bash
curl -X POST http://localhost:7845/federation/join \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-key" \
  --data '{
    "peer": "12D3KooWPeerAddress..."
  }'
```

### **Issue Credential**
```bash
curl -X POST http://localhost:7845/identity/credentials/issue \
  -H "Content-Type: application/json" \
  -H "x-api-key: your-key" \
  --data '{
    "issuer": "did:key:issuer",
    "holder": "did:key:holder", 
    "attributes": {
      "name": "Alice Smith",
      "role": "cooperative_member",
      "membership_level": "verified"
    },
    "schema": "QmSchemaHash...",
    "expiration": 1735689600
  }'
```

### **Get System Status**
```bash
curl http://localhost:7845/status \
  -H "x-api-key: your-key"
```

---

## 🔧 **Response Formats**

### **Success Response**
```json
{
  "status": "success",
  "data": { ... },
  "timestamp": "2025-01-08T10:00:00Z"
}
```

### **Error Response**
```json
{
  "error": "Error description",
  "details": { ... },
  "correlation_id": "uuid-string",
  "timestamp": "2025-01-08T10:00:00Z"
}
```

---

## 🚀 **Rate Limiting & Security**

### **Rate Limits**
- **Authenticated requests**: No limit (with valid API key/token)
- **Unauthenticated requests**: 60 requests per minute
- **Rate limit headers**: Included in responses

### **Security Features**
- **API Key Authentication**: `x-api-key` header
- **Bearer Token Support**: `Authorization: Bearer <token>`
- **CORS Support**: Configurable origins
- **Request Correlation**: `x-correlation-id` header
- **TLS Support**: HTTPS with configurable certificates

---

## 📡 **WebSocket Events**

### **Real-Time Event Subscriptions**
```javascript
const ws = new WebSocket('ws://localhost:7845/ws');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Event:', message.type, message.data);
};

// Available event types:
// - proposal_status_changed
// - job_progress_updated  
// - federation_peer_added
// - mana_balance_changed
// - network_event
```

---

## 🔗 **TypeScript SDK**

### **Installation & Usage**
```bash
npm install @icn/client-sdk
```

```typescript
import { ICNClient } from '@icn/client-sdk';

const client = new ICNClient({
  baseUrl: 'http://localhost:7845',
  apiKey: 'your-api-key'
});

// Type-safe API calls
const proposals = await client.governance.listProposals();
const jobStatus = await client.mesh.getJobStatus(jobId);
```

---

## 📊 **API Statistics**

- **Total Endpoints**: 60+
- **Governance**: 8 endpoints
- **Identity**: 11 endpoints  
- **Mesh Computing**: 12 endpoints
- **Federation**: 8 endpoints
- **Cooperative**: 7 endpoints
- **Storage**: 8 endpoints
- **System**: 5 endpoints
- **Network**: 3 endpoints
- **Other**: 8+ endpoints

**Endpoints are well-structured with good error handling and authentication design. Backend implementations vary in completeness.**
