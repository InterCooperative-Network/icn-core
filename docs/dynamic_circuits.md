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

## Registering Circuits via the API

The registry is manipulated through the `/circuits` endpoints.

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

`Groth16KeyManager` persists parameters under `~/.icn/zk/`:

- `proving_key.bin` – compressed Groth16 proving key
- `verifying_key.bin` – verifying key bytes
- `verifying_key.sig` – Ed25519 signature authenticating the key

Parameters registered via the API are saved as [`CircuitParameters`](../crates/icn-zk/src/params.rs) within the registry database. Prepared verifying keys are cached in memory to speed up verification.

## Registry API Overview

The registry exposes a small HTTP surface for managing circuits:

- `POST /circuits/register` – Add a new circuit version to the node.
- `GET /circuits/{slug}/{version}` – Fetch parameters and metadata for a circuit.
- `GET /circuits/{slug}/latest` – Retrieve the highest available version.
- `GET /circuits` – List all registered circuits with available versions.

Requests and responses use JSON and follow the data structures defined in
[`icn_zk`](../crates/icn-zk/src/lib.rs) and the identity subsystem.

### Versioning Policy

Circuit slugs follow [semantic versioning](https://semver.org/). Patches fix
setup mistakes without changing the proving key. Minor versions may add
optimisations or metadata while remaining backward compatible with proofs
created for earlier minors. Any change that requires regenerating the proving
key mandates a new major version. Nodes keep old versions so existing proofs
remain valid.

### Storage Layout

Parameters registered through the API are persisted under
`~/.icn/zk/{slug}/{version}/`. Each directory contains:

- `proving_key.bin` – compressed Groth16 proving key bytes.
- `verifying_key.bin` – the matching verifying key.
- `verifying_key.sig` – Ed25519 signature authenticating the verifying key.
- `metadata.json` – optional JSON with circuit details.

On startup, [`Groth16KeyManager`](../crates/icn-identity/src/zk/key_manager.rs)
loads these files and exposes them as
[`CircuitParameters`](../crates/icn-zk/src/params.rs) objects.

### Example Usage

Registering a new circuit:

```bash
curl -X POST http://localhost:7845/circuits/register \ 
  -H "Content-Type: application/json" \
  -d '{
        "slug": "age_over_18",
        "version": "1.2.0",
        "proving_key": "<base64>",
        "verification_key": "<base64>",
        "metadata": {"complexity": 10}
      }'
```

Fetching metadata later:

```bash
curl http://localhost:7845/circuits/age_over_18/latest
```

In Rust the same calls can be made using `reqwest` or the
`IcnClient` from [`icn_sdk`](../crates/icn-sdk/src/lib.rs):

```rust
use icn_sdk::IcnClient;
use icn_zk::CircuitParameters;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = IcnClient::new("http://localhost:7845")?;
let params: CircuitParameters = client
    .get("/circuits/age_over_18/1.2.0")
    .await?;
println!("loaded {} bytes", params.proving_key.len());
# Ok(()) }
```
