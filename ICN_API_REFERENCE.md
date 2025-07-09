# ICN Core API Reference
This document has been condensed and now points to the canonical [API documentation](docs/API.md). The table below lists the HTTP endpoints exposed by an ICN node.

## Quick Reference

| Method | Path | Description |
|-------|------|-------------|
| GET | `/info` | Node metadata including version and name |
| GET | `/status` | Current node health and peer connectivity |
| GET | `/health` | Basic health check |
| GET | `/ready` | Readiness probe for orchestration |
| GET | `/metrics` | Prometheus metrics |
| GET | `/network/local-peer-id` | Return the local peer identifier |
| POST | `/network/connect` | Connect to a peer by multiaddress |
| GET | `/network/peers` | List currently connected peers |
| POST | `/dag/put` | Store a content-addressed block |
| POST | `/dag/get` | Retrieve a block by CID |
| POST | `/dag/meta` | Retrieve metadata for a block |
| POST | `/dag/pin` | Pin a block to prevent pruning |
| POST | `/dag/unpin` | Unpin a previously pinned block |
| POST | `/dag/prune` | Remove unpinned blocks |
| POST | `/transaction/submit` | Submit a transaction |
| POST | `/data/query` | Query stored data with filters |
| POST | `/governance/submit` | Submit a governance proposal |
| POST | `/governance/vote` | Cast a vote on a proposal |
| POST | `/governance/delegate` | Delegate voting power to another DID |
| POST | `/governance/revoke` | Revoke a prior delegation |
| POST | `/governance/close` | Close voting and return tally |
| POST | `/governance/execute` | Execute an approved proposal |
| GET | `/governance/proposals` | List all proposals |
| GET | `/governance/proposal/:id` | Fetch a specific proposal |
| POST | `/mesh/submit` | Submit a mesh job |
| GET | `/mesh/jobs` | List all mesh jobs |
| GET | `/mesh/jobs/:job_id` | Get the status of a specific job |
| POST | `/mesh/receipts` | Submit an execution receipt |
| POST | `/contracts` | Upload or update a WASM contract |
| GET | `/contracts` | List deployed contracts |
| POST | `/contracts/execute` | Execute a WASM contract |
| GET | `/federation/peers` | List known federation peers |
| POST | `/federation/peers` | Add a peer to the federation |
| POST | `/federation/join` | Join a federation |
| POST | `/federation/leave` | Leave a federation |
| GET | `/federation/status` | Get current federation status |

For detailed request/response examples and authentication requirements see [docs/API.md](docs/API.md).
