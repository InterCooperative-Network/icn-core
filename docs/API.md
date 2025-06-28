# ICN HTTP API

The ICN node exposes a REST interface. All endpoints require the configured `x-api-key` header if an API key is set. If an `auth_token` is configured, requests must also include `Authorization: Bearer <token>`. When no API key is set the `--open-rate-limit` option controls the number of unauthenticated requests allowed per minute. If `tls_cert_path` and `tls_key_path` are supplied the server only accepts HTTPS connections.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/info` | Node metadata including version and name |
| GET | `/status` | Current node health and peer connectivity |
| POST | `/dag/put` | Store a content-addressed block |
| POST | `/dag/get` | Retrieve a block by CID |
| POST | `/transaction/submit` | Submit a transaction to be processed |
| POST | `/data/query` | Query stored data |
| POST | `/governance/submit` | Submit a governance proposal |
| POST | `/governance/vote` | Cast a vote on a proposal |
| GET | `/governance/proposals` | List all proposals |
| GET | `/governance/proposal/:id` | Fetch a specific proposal |
| POST | `/mesh/submit` | Submit a mesh job for distributed execution |
| GET | `/mesh/jobs` | List all mesh jobs |
| GET | `/mesh/jobs/:job_id` | Get the status of a job |
| POST | `/mesh/receipts` | Submit an execution receipt |
| POST | `/contracts` | Upload or update a WASM contract |
| GET | `/federation/peers` | List known federation peers |
| POST | `/federation/peers` | Add a federation peer |

