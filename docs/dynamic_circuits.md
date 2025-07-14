# Dynamic Circuit Registry

This guide covers how zero-knowledge circuits are managed in the InterCooperative Network. Circuits can be added or upgraded without rebuilding nodes thanks to a dedicated registry and stable versioning rules.

## Circuit Registry Concepts

- Each circuit is identified by a unique **slug** and semantic **version** (for example `age_over_18` version `1.0.0`).
- Registered circuits store a Groth16 proving key, a verifying key and optional metadata.
- Nodes consult the registry when verifying credential proofs so that the correct versioned parameters are used.
- Circuits may be bundled with the node or registered dynamically through the HTTP API.

## Versioning Rules and Compatibility

- Minor version bumps must remain backward compatible with proving keys generated for earlier minor versions.
- Breaking changes require a new major version. Older versions remain in the registry for verifiers that still rely on them.
- Proofs include a circuit slug and version so verifiers can select the appropriate parameters.
- Patch versions are reserved for non-breaking metadata updates or key rotations.

## Registering Circuits via the API

The registry is manipulated through the `/circuits` endpoints.

### API Overview

- `POST /circuits/register` – upload a new circuit version and its parameters
- `GET  /circuits/{slug}` – list all versions for a circuit slug
- `GET  /circuits/{slug}/{version}` – fetch parameters and metadata for a specific version

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

The node stores the parameters and returns a content identifier (CID) for later reference. Circuits can be fetched with `GET /circuits/{slug}/{version}`.

## Proving Key and Parameter Storage

`Groth16KeyManager` persists parameters under `~/.icn/zk/<circuit>/`:

- `proving_key.bin` – compressed Groth16 proving key
- `verifying_key.bin` – verifying key bytes
- `verifying_key.sig` – Ed25519 signature authenticating the key

Parameters registered via the API are saved as [`CircuitParameters`](../crates/icn-zk/src/params.rs) within the registry database. Prepared verifying keys are cached in memory to speed up verification.

## Database-Backed Registry

Circuit metadata and parameter bytes now persist in an embedded database located
under `~/.icn/zk/registry.sqlite`. Storing circuits in a database means they
survive node restarts and can be synchronized across deployments. Each entry is
keyed by the circuit slug and semantic version.

The `circuits` table contains:

- `slug` – circuit identifier
- `version` – semantic version string
- `params` – serialized [`CircuitParameters`](../crates/icn-zk/src/params.rs)
- `meta` – optional JSON metadata

### Backup and Restore

Use `icn-cli` to export or re-import the circuit registry. This works with any
database backend:

```bash
# Backup circuit registry
icn-cli zk backup --path ./backups/circuits

# Restore from a backup
icn-cli zk restore --path ./backups/circuits
```

### Usage Examples

Using the `icn-sdk` crate the registry can be accessed programmatically:

```rust
use icn_sdk::{IcnClient, RegisterCircuitRequest};
use icn_zk::params::CircuitParameters;

let client = IcnClient::new("http://localhost:7845")?;
let params = CircuitParameters::from_proving_key(&proving_key)?;
let req = RegisterCircuitRequest {
    slug: "age_over_18".into(),
    version: "1.0.0".into(),
    parameters: params,
    metadata: None,
};
client.register_circuit(&req).await?;

let info = client.circuit_info("age_over_18", "1.0.0").await?;
println!("registered by {}", info.uploader);
```

