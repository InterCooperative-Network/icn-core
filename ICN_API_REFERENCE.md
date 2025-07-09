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
| `/health` | GET | Health check endpoint | ⚠️ Empty |
| `/mesh/jobs` | GET | List mesh computing jobs | ✅ Working |
| `/mesh/bids` | GET | List mesh job bids | ✅ Working |
| `/governance/proposals` | GET | List governance proposals | ✅ Working |
| `/governance/votes` | GET | List governance votes | ✅ Working |
| `/dag/put` | POST | Store data in DAG | ✅ Working |
| `/dag/get` | POST | Retrieve data from DAG | ⚠️ Format issue |
| `/dag/blocks` | GET | List DAG blocks | ⚠️ Empty |
| `/dag/info` | GET | DAG information | ⚠️ Empty |
| `/network/peers` | GET | List network peers | ⚠️ Empty |
| `/account/{did}` | GET | Account information | ⚠️ Empty |
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

### List Mesh Job Bids
**Endpoint:** `GET /mesh/bids`  
**Description:** Returns a list of all bids for mesh computing jobs.

**Example Request:**
```bash
curl -s http://127.0.0.1:7845/mesh/bids | jq .
```

**Example Response:**
```
(Empty response - no bids currently)
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

### List Governance Votes
**Endpoint:** `GET /governance/votes`  
**Description:** Returns a list of all votes cast on governance proposals.

**Example Request:**
```bash
curl -s http://127.0.0.1:7845/governance/votes | jq .
```

**Example Response:**
```
(Empty response - no votes currently)
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

### List DAG Blocks
**Endpoint:** `GET /dag/blocks`  
**Description:** Returns a list of all blocks stored in the DAG.

**Example Request:**
```bash
curl -s http://127.0.0.1:7845/dag/blocks | jq .
```

**Example Response:**
```
(Empty response - no blocks or endpoint not implemented)
```

### DAG Information
**Endpoint:** `GET /dag/info`  
**Description:** Returns information about the DAG storage system.

**Example Request:**
```bash
curl -s http://127.0.0.1:7845/dag/info | jq .
```

**Example Response:**
```
(Empty response - endpoint not implemented)
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

### Get Account Information
**Endpoint:** `GET /account/{did}`  
**Description:** Returns account information for a specific DID.

**Example Request:**
```bash
curl -s "http://127.0.0.1:7845/account/did:key:z6Mkou2jwqcofqj6FC4MpRpQjPQh1Neyo2v9jcgJNNfMzdof" | jq .
```

**Example Response:**
```
(Empty response - endpoint not fully implemented)
```

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
- `POST /mesh/bids` - Submit a bid for a job

### Governance
- `POST /governance/proposals` - Create a new proposal
- `POST /governance/votes` - Cast a vote on a proposal
- `GET /governance/proposals/{id}` - Get specific proposal details

### Identity & Accounts
- `POST /identity/keys` - Generate new keys
- `GET /identity/keys` - List available keys
- `POST /account/transfer` - Transfer mana between accounts

---

**API Reference Generated:** December 2024  
**Node Version:** 0.1.0-dev-functional  
**Documentation Status:** ✅ Complete for working endpoints 