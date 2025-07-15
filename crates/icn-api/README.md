# ICN API Crate

This crate provides the primary API interfaces for interacting with InterCooperative Network (ICN) nodes. It defines service traits, data structures for requests and responses, and comprehensive HTTP API documentation for external consumption.

## Overview

The ICN API is designed for clarity, modularity, and extensibility. It provides:

- **Service Traits**: Abstract interfaces for core ICN functionality
- **Data Transfer Objects (DTOs)**: Standardized request/response structures
- **HTTP API Endpoints**: RESTful interface for external applications
- **Type Safety**: Rust-first design with comprehensive type definitions

## Core API Modules

- `governance_trait` - Proposal submission, voting, and delegation
- `identity_trait` - DID management and credential operations
- `federation_trait` - Multi-node federation management
- `mutual_aid_trait` - Resource sharing and mutual aid
- `dag_trait` - Content-addressed storage operations

---

# HTTP API Reference for Web UI Integration

This section provides comprehensive documentation for all HTTP endpoints available for `icn-web-ui` integration.

## Authentication

All API endpoints require authentication via one of these methods:

### API Key Authentication
```http
x-api-key: your-api-key-here
```

### Bearer Token Authentication
```http
Authorization: Bearer your-bearer-token-here
```

### DID-Based Authentication (Coming Soon)
```http
Authorization: DID did:example:123... <signature>
```

## Base URL

Default base URL: `http://localhost:7845`

For HTTPS (with TLS configured): `https://localhost:7845`

## Common Response Formats

### Success Response
```json
{
  "status": "success",
  "data": { ... }
}
```

### Error Response
```json
{
  "error": "Error description",
  "details": { ... },
  "correlation_id": "uuid"
}
```

---

## Governance API

### Submit Proposal
Submit a new governance proposal for community voting.

```http
POST /governance/submit
Content-Type: application/json
```

**Request Body:**
```json
{
  "proposer_did": "did:example:alice",
  "proposal": {
    "type": "SystemParameterChange",
    "data": {
      "param": "mana_regeneration_rate",
      "value": "0.1"
    }
  },
  "description": "Increase mana regeneration rate to improve network participation",
  "duration_secs": 604800,
  "quorum": 10,
  "threshold": 0.6,
  "body": null,
  "credential_proof": null,
  "revocation_proof": null
}
```

**Proposal Types:**
- `SystemParameterChange` - Modify network parameters
- `MemberAdmission` - Add new cooperative member
- `RemoveMember` - Remove existing member
- `SoftwareUpgrade` - Propose software version upgrade
- `GenericText` - General discussion proposal
- `Resolution` - Execute specific actions

**Response:**
```json
"proposal_id_string"
```

### Cast Vote
Vote on an active proposal.

```http
POST /governance/vote
Content-Type: application/json
```

**Request Body:**
```json
{
  "voter_did": "did:example:bob",
  "proposal_id": "proposal_id_string",
  "vote_option": "Yes",
  "credential_proof": null,
  "revocation_proof": null
}
```

**Vote Options:** `"Yes"`, `"No"`, `"Abstain"`

**Response:**
```json
"Vote cast successfully"
```

### List Proposals
Get all governance proposals with their current status.

```http
GET /governance/proposals
```

**Response:**
```json
[
  {
    "id": "proposal_id",
    "proposer": "did:example:alice",
    "proposal_type": { ... },
    "description": "Proposal description",
    "status": "Open",
    "created_at": "2025-01-08T10:00:00Z",
    "voting_deadline": "2025-01-15T10:00:00Z",
    "votes": {
      "yes": 5,
      "no": 2,
      "abstain": 1
    },
    "quorum": 10,
    "threshold": 0.6
  }
]
```

### Get Proposal Details
Get detailed information about a specific proposal.

```http
GET /governance/proposal/{proposal_id}
```

**Response:**
```json
{
  "id": "proposal_id",
  "proposer": "did:example:alice",
  "proposal_type": { ... },
  "description": "Detailed proposal description",
  "status": "Open",
  "created_at": "2025-01-08T10:00:00Z",
  "voting_deadline": "2025-01-15T10:00:00Z",
  "votes": {
    "yes": 5,
    "no": 2,
    "abstain": 1
  },
  "detailed_votes": [
    {
      "voter": "did:example:bob",
      "option": "Yes",
      "timestamp": "2025-01-08T11:00:00Z"
    }
  ],
  "quorum": 10,
  "threshold": 0.6
}
```

### Delegate Vote
Delegate voting power to another member.

```http
POST /governance/delegate
Content-Type: application/json
```

**Request Body:**
```json
{
  "from_did": "did:example:alice",
  "to_did": "did:example:bob"
}
```

### Revoke Delegation
Revoke previously delegated voting power.

```http
POST /governance/revoke
Content-Type: application/json
```

**Request Body:**
```json
{
  "from_did": "did:example:alice"
}
```

---

## Identity & Credentials API

### Issue Credential
Issue a new verifiable credential.

```http
POST /identity/credentials/issue
Content-Type: application/json
```

**Request Body:**
```json
{
  "issuer": "did:example:issuer",
  "holder": "did:example:holder",
  "attributes": {
    "name": "Alice Smith",
    "role": "cooperative_member",
    "membership_level": "verified"
  },
  "schema": "QmSchemaHash...",
  "expiration": 1735689600
}
```

**Response:**
```json
{
  "cid": "QmCredentialHash...",
  "credential": {
    "issuer": "did:example:issuer",
    "holder": "did:example:holder",
    "attributes": { ... },
    "schema": "QmSchemaHash...",
    "expiration": 1735689600,
    "signature": "..."
  }
}
```

### Verify Credential
Verify the authenticity of a credential.

```http
POST /identity/credentials/verify
Content-Type: application/json
```

**Request Body:**
```json
{
  "credential": {
    "issuer": "did:example:issuer",
    "holder": "did:example:holder",
    "attributes": { ... },
    "schema": "QmSchemaHash...",
    "expiration": 1735689600,
    "signature": "..."
  }
}
```

**Response:**
```json
{
  "valid": true
}
```

### Get Credential
Retrieve a credential by its CID.

```http
GET /identity/credentials/{cid}
```

**Response:**
```json
{
  "cid": "QmCredentialHash...",
  "credential": { ... }
}
```

### List Credential Schemas
Get all available credential schemas.

```http
GET /identity/credentials/schemas
```

**Response:**
```json
[
  "QmSchemaHash1...",
  "QmSchemaHash2...",
  "QmSchemaHash3..."
]
```

### Generate Zero-Knowledge Proof
Generate a ZK proof for credential claims.

```http
POST /identity/generate-proof
Content-Type: application/json
```

**Request Body:**
```json
{
  "issuer": "did:example:issuer",
  "holder": "did:example:holder",
  "claim_type": "membership_verification",
  "schema": "QmSchemaHash...",
  "backend": "groth16",
  "public_inputs": {
    "min_membership_level": 1
  }
}
```

### Verify Zero-Knowledge Proof
Verify a ZK proof without revealing underlying data.

```http
POST /identity/verify
Content-Type: application/json
```

---

## Federation Management API

### List Federation Peers
Get all peers in the current federation.

```http
GET /federation/peers
```

**Response:**
```json
[
  "12D3KooWPeer1...",
  "12D3KooWPeer2...",
  "12D3KooWPeer3..."
]
```

### Join Federation
Join an existing federation by connecting to a peer.

```http
POST /federation/join
Content-Type: application/json
```

**Request Body:**
```json
{
  "peer": "12D3KooWPeerAddress..."
}
```

### Leave Federation
Leave the current federation.

```http
POST /federation/leave
Content-Type: application/json
```

**Request Body:**
```json
{
  "peer": "12D3KooWPeerAddress..."
}
```

### Federation Status
Get current federation status and statistics.

```http
GET /federation/status
```

**Response:**
```json
{
  "peer_count": 5,
  "peers": [
    "12D3KooWPeer1...",
    "12D3KooWPeer2..."
  ]
}
```

---

## Mesh Computing API

### Submit Mesh Job
Submit a computational job to the mesh network.

```http
POST /mesh/submit
Content-Type: application/json
```

**Request Body:**
```json
{
  "job_spec": {
    "image": "python:3.9",
    "command": ["python", "-c", "print('Hello, ICN!')"],
    "resources": {
      "cpu_cores": 1,
      "memory_mb": 512,
      "storage_mb": 1024
    }
  },
  "submitter_did": "did:example:submitter",
  "max_cost": 1000,
  "timeout_seconds": 300
}
```

**Response:**
```json
{
  "job_id": "job_uuid_string"
}
```

### List Jobs
Get all jobs with their current status.

```http
GET /mesh/jobs
```

**Response:**
```json
[
  {
    "id": "job_uuid",
    "submitter": "did:example:submitter",
    "status": "Running",
    "submitted_at": "2025-01-08T10:00:00Z",
    "executor": "did:example:executor",
    "cost": 850,
    "progress": 0.7
  }
]
```

### Get Job Status
Get detailed status of a specific job.

```http
GET /mesh/jobs/{job_id}
```

**Response:**
```json
{
  "id": "job_uuid",
  "submitter": "did:example:submitter",
  "status": "Completed",
  "submitted_at": "2025-01-08T10:00:00Z",
  "started_at": "2025-01-08T10:05:00Z",
  "completed_at": "2025-01-08T10:15:00Z",
  "executor": "did:example:executor",
  "cost": 850,
  "result": {
    "output": "Hello, ICN!",
    "exit_code": 0
  }
}
```

---

## Account & Mana API

### Get Mana Balance
Get the current mana balance for a DID.

```http
GET /account/{did}/mana
```

**Response:**
```json
{
  "balance": 5000
}
```

### Get Account Information
Get comprehensive account information.

```http
GET /keys
```

**Response:**
```json
{
  "did": "did:example:node",
  "public_key_bs58": "5Gv8YR1c2..."
}
```

---

## Reputation API

### Get Reputation Score
Get the reputation score for a DID.

```http
GET /reputation/{did}
```

**Response:**
```json
{
  "score": 0.85,
  "frozen": false
}
```

---

## DAG Storage API

### Store DAG Block
Store a new block in the DAG.

```http
POST /dag/put
Content-Type: application/json
```

**Request Body:**
```json
{
  "data": "base64_encoded_data",
  "links": ["QmParentHash1...", "QmParentHash2..."],
  "author_did": "did:example:author",
  "scope": "public"
}
```

### Retrieve DAG Block
Get a block from the DAG by CID.

```http
POST /dag/get
Content-Type: application/json
```

**Request Body:**
```json
{
  "cid": "QmBlockHash..."
}
```

---

## System Information API

### Node Information
Get basic node information.

```http
GET /info
```

**Response:**
```json
{
  "name": "ICN Node",
  "version": "0.1.0",
  "did": "did:example:node",
  "capabilities": ["governance", "mesh", "federation"]
}
```

### Node Status
Get current node operational status.

```http
GET /status
```

**Response:**
```json
{
  "is_online": true,
  "peer_count": 8,
  "current_block_height": 12450,
  "version": "0.1.0"
}
```

### Health Check
Simple health check endpoint.

```http
GET /health
```

**Response:**
```json
{
  "status": "healthy"
}
```

### Metrics
Get Prometheus-format metrics.

```http
GET /metrics
```

---

## Error Codes

Common HTTP status codes and their meanings:

- `200 OK` - Request successful
- `201 Created` - Resource created successfully
- `400 Bad Request` - Invalid request format or parameters
- `401 Unauthorized` - Authentication required or failed
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server error

---

## Rate Limiting

The API implements rate limiting to prevent abuse:

- **Authenticated requests**: No limit (with valid API key/token)
- **Unauthenticated requests**: 60 requests per minute by default
- **Rate limit headers**: Included in responses when applicable

---

## WebSocket Support (Planned)

Real-time updates for web UI will be available via WebSocket connections:

```
wss://localhost:7845/ws
```

**Planned events:**
- Proposal status changes
- Job progress updates
- New federation peers
- Mana balance changes
- Network events

---

## TypeScript Client SDK (Planned)

A comprehensive TypeScript client SDK will be available for web applications:

```typescript
import { IcnClient } from '@icn/client-sdk';

const client = new IcnClient({
  baseUrl: 'http://localhost:7845',
  apiKey: 'your-api-key'
});

// Type-safe API calls
const proposals = await client.governance.listProposals();
const jobStatus = await client.mesh.getJobStatus(jobId);
```

This SDK will provide:
- Full TypeScript type definitions
- Automatic request/response validation
- Built-in error handling
- WebSocket event subscriptions
- Authentication management 