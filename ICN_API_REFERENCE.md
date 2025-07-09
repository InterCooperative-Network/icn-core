# ICN Core API Reference

**Base URL:** `http://127.0.0.1:7845`  
**Version:** 0.1.0-dev-functional  
**Content-Type:** `application/json`

---

## 📋 **Quick Reference**

| Endpoint | Method | Description | Status |
|----------|---------|-------------|--------|
| `/info` | GET | Node information and DID | ✅ Working |
| `/status` | GET | Real-time node status | ✅ Working |
| `/health` | GET | Health check endpoint | ✅ Working |
| `/ready` | GET | Readiness probe | ✅ Working |
| `/mesh/submit` | POST | Submit a mesh job | ✅ Working |
| `/mesh/jobs` | GET | List mesh computing jobs | ✅ Working |
| `/mesh/jobs/:job_id` | GET | Get specific job status | ✅ Working |
| `/mesh/receipts` | POST | Submit execution receipt | ✅ Working |
| `/governance/proposals` | GET | List governance proposals | ✅ Working |
| `/governance/proposal/:id` | GET | Fetch a proposal | ✅ Working |
| `/governance/submit` | POST | Submit a proposal | ✅ Working |
| `/governance/vote` | POST | Cast a vote | ✅ Working |
| `/governance/close` | POST | Close voting | ✅ Working |
| `/governance/execute` | POST | Execute proposal | ✅ Working |
| `/dag/put` | POST | Store data in DAG | ✅ Working |
| `/dag/get` | POST | Retrieve data from DAG | ✅ Working |
| `/dag/meta` | POST | Retrieve DAG metadata | ✅ Working |
| `/dag/pin` | POST | Pin a DAG block | ✅ Working |
| `/dag/unpin` | POST | Unpin a DAG block | ✅ Working |
| `/dag/prune` | POST | Prune unpinned blocks | ✅ Working |
| `/network/local-peer-id` | GET | Show local peer ID | ✅ Working |
| `/network/connect` | POST | Connect to a peer | ✅ Working |
| `/network/peers` | GET | List network peers | ✅ Working |
| `/transaction/submit` | POST | Submit a transaction | ✅ Working |
| `/data/query` | POST | Query data | ✅ Working |
| `/contracts` | POST | Upload WASM contract | ✅ Working |
| `/federation/peers` | GET | List federation peers | ✅ Working |
| `/federation/peers` | POST | Add federation peer | ✅ Working |
| `/metrics` | GET | Prometheus metrics | ✅ Working |

---

## 🔍 **Node Information & Status**

### Get Node Information
**Endpoint:** `GET /info`  
**Description:** Returns basic node information including version, name, DID, and mana balance.

**Example Request:**
```bash
curl -s http://127.0.0.1:7845/info | jq .
```

**Example Response:**
```json
{
  "version": "0.1.0-dev-functional",
  "name": "ICN Node",
  "status_message": "Node DID: did:key:z6Mkou2jwqcofqj6FC4MpRpQjPQh1Neyo2v9jcgJNNfMzdof, Mana: 0"
}
```

### Get Node Status
**Endpoint:** `GET /status`  
**Description:** Returns real-time node operational status.

**Example Request:**
```bash
curl -s http://127.0.0.1:7845/status | jq .
```

**Example Response:**
```json
{
  "is_online": true,
  "peer_count": 0,
  "current_block_height": 0,
  "version": "0.1.0-dev-functional"
}
```

### Health Check
**Endpoint:** `GET /health`  
**Description:** Basic health check endpoint.

**Example Request:**
```bash
curl -s http://127.0.0.1:7845/health
```

**Example Response:**
```
(Empty response - endpoint exists but returns no data)
```

---

## 🕸️ **Mesh Computing System**

### List Mesh Jobs
**Endpoint:** `GET /mesh/jobs`  
**Description:** Returns a list of all mesh computing jobs in the system.

**Example Request:**
```bash
curl -s http://127.0.0.1:7845/mesh/jobs | jq .
```

**Example Response:**
```json
{
  "jobs": []
}
```


---

## 🏛️ **Governance System**

### List Governance Proposals
**Endpoint:** `GET /governance/proposals`  
**Description:** Returns a list of all governance proposals.

**Example Request:**
```bash
curl -s http://127.0.0.1:7845/governance/proposals | jq .
```

**Example Response:**
```json
[]
```


---

## 🔗 **DAG Storage System**

### Store Data in DAG
**Endpoint:** `POST /dag/put`  
**Description:** Stores data in the content-addressed DAG storage system.

**Request Body:** JSON object with `data` field containing byte array.

**Example Request:**
```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"data": [72, 101, 108, 108, 111]}' \
  http://127.0.0.1:7845/dag/put
```

**Example Response:**
```json
{
  "version": 1,
  "codec": 113,
  "hash_alg": 18,
  "hash_bytes": [146, 227, 3, 47, 214, 96, 140, 111, 217, 87, 178, 73, 119, 117, 228, 211, 220, 187, 224, 70, 219, 4, 125, 104, 224, 178, 86, 237, 73, 183, 184, 101]
}
```

**Data Format:**
- **`data`**: Array of bytes representing the content to store
- **Response**: Returns a CID (Content Identifier) object with version, codec, hash algorithm, and hash bytes

### Retrieve Data from DAG
**Endpoint:** `POST /dag/get`  
**Description:** Retrieves data from the DAG using a Content Identifier (CID).

**Status:** ⚠️ **API Format Issue** - Endpoint expects string CID format, but PUT returns object format.

**Expected Request Body:** JSON object with `cid` field (format TBD).

**Example Request:**
```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"cid": "TBD - string format needed"}' \
  http://127.0.0.1:7845/dag/get
```


---

## 🌐 **Network & Identity**

### List Network Peers
**Endpoint:** `GET /network/peers`
**Description:** Returns a list of connected network peers.

**Example Request:**
```bash
curl -s http://127.0.0.1:7845/network/peers | jq .
```

**Example Response:**
```
(Empty response - no P2P peers without libp2p feature)
```

### Local Peer ID
**Endpoint:** `GET /network/local-peer-id`
**Description:** Returns the node's own peer identifier.

### Connect to a Peer
**Endpoint:** `POST /network/connect`
**Description:** Connect to another peer using a multiaddr string.

---

## 📊 **Metrics & Monitoring**

### Get Prometheus Metrics
**Endpoint:** `GET /metrics`  
**Description:** Returns Prometheus-formatted metrics for monitoring.

**Example Request:**
```bash
curl -s http://127.0.0.1:7845/metrics
```

**Example Response:**
```
# HELP host_submit_mesh_job_calls Number of host_submit_mesh_job calls.
# TYPE host_submit_mesh_job_calls counter
host_submit_mesh_job_calls_total 0
# HELP host_get_pending_mesh_jobs_calls Number of host_get_pending_mesh_jobs calls.
# TYPE host_get_pending_mesh_jobs_calls counter
host_get_pending_mesh_jobs_calls_total 0
# HELP host_account_get_mana_calls Number of host_account_get_mana calls.
# TYPE host_account_get_mana_calls counter
host_account_get_mana_calls_total 0
# HELP host_account_spend_mana_calls Number of host_account_spend_mana calls.
# TYPE host_account_spend_mana_calls counter
host_account_spend_mana_calls_total 0
# EOF
```

**Available Metrics:**
- `host_submit_mesh_job_calls_total`: Number of mesh job submissions
- `host_get_pending_mesh_jobs_calls_total`: Number of pending job queries
- `host_account_get_mana_calls_total`: Number of mana balance queries
- `host_account_spend_mana_calls_total`: Number of mana spending operations

---

## 🔧 **Configuration & Setup**

### Environment Variables
```bash
# Required for RocksDB compilation
export ROCKSDB_LIB_DIR=/usr/lib/x86_64-linux-gnu
export ROCKSDB_STATIC=0
```

### Build Command
```bash
cargo build --package icn-node
```

### Run Command
```bash
cargo run --package icn-node -- \
  --node-name "ICN-Node" \
  --http-listen-addr "127.0.0.1:7845" \
  --storage-backend rocksdb \
  --storage-path "./data/dag" \
  --mana-ledger-backend file \
  --mana-ledger-path "./data/mana.db"
```

### Available Storage Backends
- `memory`: In-memory storage (volatile)
- `file`: File-based persistence
- `rocksdb`: RocksDB database backend (recommended)
- `sqlite`: SQLite database backend (requires feature flag)
- `sled`: Sled database backend (requires feature flag)

---

## 🧪 **Testing Examples**

### Basic Health Check Script
```bash
#!/bin/bash
echo "=== ICN Node Health Check ==="
curl -s http://127.0.0.1:7845/info | jq .
echo
curl -s http://127.0.0.1:7845/status | jq .
```

### DAG Storage Test
```bash
#!/bin/bash
echo "=== Testing DAG Storage ==="
# Store "Hello" as bytes
RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
  -d '{"data": [72, 101, 108, 108, 111]}' \
  http://127.0.0.1:7845/dag/put)
echo "Stored data, CID: $RESULT"
```

### Metrics Monitoring
```bash
#!/bin/bash
echo "=== ICN Metrics ==="
curl -s http://127.0.0.1:7845/metrics | grep -E "host_.*_total"
```

---

## 🚨 **Error Handling**

### Common Error Responses
- **400 Bad Request**: Invalid JSON format or missing required fields
- **404 Not Found**: Endpoint doesn't exist or resource not found
- **500 Internal Server Error**: Server-side error (check logs)

### JSON Deserialization Errors
- **Message**: "Failed to deserialize the JSON body into the target type"
- **Cause**: Incorrect JSON structure or data types
- **Solution**: Verify JSON format matches expected schema

### Example Error Response
```json
{
  "error": "Invalid request format",
  "message": "Expected array of bytes for data field",
  "timestamp": "2024-12-08T19:00:00Z"
}
```

---

## 🔮 **Future Endpoints** (Not Yet Implemented)

### Mesh Computing
- `POST /mesh/jobs` - Submit a new mesh job
- `GET /mesh/jobs/{id}` - Get specific job details
- `DELETE /mesh/jobs/{id}` - Cancel a job

### Governance
- `POST /governance/proposals` - Create a new proposal
- `POST /governance/vote` - Cast a vote on a proposal
- `GET /governance/proposals/{id}` - Get specific proposal details

### Identity & Accounts
- `POST /identity/keys` - Generate new keys
- `GET /identity/keys` - List available keys
- `POST /account/transfer` - Transfer mana between accounts

---

**API Reference Generated:** December 2024  
**Node Version:** 0.1.0-dev-functional  **Documentation Status:** ✅ Complete for working endpoints 