# ICN Core API Reference

**Base URL:** `http://127.0.0.1:7845`
**Version:** 0.1.0-dev-functional
**Base Path:** `/api/v1`
**Content-Type:** `application/json`

---

## 📋 **Quick Reference**

| Endpoint | Method | Description | Status |
|----------|-------|-------------|--------|
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
| `/governance/delegate` | POST | Delegate voting power | ✅ Working |
| `/governance/revoke` | POST | Revoke a delegation | ✅ Working |
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
| `/federation/join` | POST | Join a federation | ✅ Working |
| `/federation/leave` | POST | Leave the federation | ✅ Working |
| `/federation/status` | GET | Current federation status | ✅ Working |
| `/metrics` | GET | Prometheus metrics | ✅ Working |

---
This document summarizes the HTTP endpoints. See [docs/API.md](docs/API.md) for complete details and authentication requirements.
