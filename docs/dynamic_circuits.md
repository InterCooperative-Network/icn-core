# Dynamic Circuit Registry

This guide covers how zero-knowledge circuits are managed in the InterCooperative Network. Circuits can be added or upgraded without rebuilding nodes thanks to a dedicated registry and stable versioning rules.

## Circuit Registry Concepts

- Each circuit is identified by a unique **slug** and semantic **version** (for example `age_over_18` version `1.0.0`).
- Registered circuits store a Groth16 proving key, a verifying key and optional metadata.
- Nodes consult the registry when verifying credential proofs so that the correct versioned parameters are used.
- Circuits may be bundled with the node or registered dynamically through the HTTP API.

## Versioning Rules and Compatibility

The registry follows semantic versioning for circuit parameters:

- **Patch versions** fix parameter generation bugs without changing the proving
  key format.
- **Minor versions** must remain backward compatible with earlier minor releases
  of the same major line.
- **Major versions** indicate breaking changes. Older versions remain available
  so historical proofs continue to verify.

Each proof embeds the circuit slug and version so verifiers can load exactly the
parameters it expects.

## Registry API

The HTTP API is implemented in [`crates/icn-node`](../crates/icn-node/src) using
the [`CircuitRegistry`](../crates/icn-node/src/circuit_registry.rs) type.
Request and response bodies are defined in
[`icn_api::circuits`](../crates/icn-api/src/circuits.rs) and include
`RegisterCircuitRequest`, `CircuitResponse` and `CircuitVersionsResponse`.

### Register a Circuit

```bash
POST /circuits/register
{
  "slug": "age_over_18",
  "version": "1.0.0",
  "proving_key": "<base64>",
  "verification_key": "<base64>"
}
```

The node stores the parameters and returns a JSON acknowledgement. Retrieve
parameters with `GET /circuits/{slug}/{version}` or list versions with
`GET /circuits/{slug}`.

### Retrieve Metadata

```bash
GET /circuits/age_over_18/1.0.0

Response:
{
  "slug": "age_over_18",
  "version": "1.0.0",
  "verification_key": "<bytes>"
}
```

## Storage Layout

The [`CircuitRegistry`](../crates/icn-node/src/circuit_registry.rs) stores all
registered parameters under `~/.icn/zk/`:

```text
~/.icn/zk/
├── registry.sqlite     # database of CircuitParameters
└── <slug>/
    ├── proving_key.bin
    ├── verifying_key.bin
    └── verifying_key.sig
```

Raw files are handled by `Groth16KeyManager`, while structured entries are saved
as [`CircuitParameters`](../crates/icn-zk/src/params.rs). Prepared verifying keys
are cached in memory for speedy verification.

## Database-Backed Registry

Circuit metadata and parameter bytes now persist in an embedded database located
under `~/.icn/zk/registry.sqlite`. Storing circuits in a database means they
survive node restarts and can be synchronized across deployments. Each entry is
keyed by the circuit slug and semantic version.

### Backup and Restore

Use `icn-cli` to export or re-import the circuit registry. This works with any
database backend:

```bash
# Backup circuit registry
icn-cli zk backup --path ./backups/circuits

# Restore from a backup
icn-cli zk restore --path ./backups/circuits
```

