# Dynamic Circuit Registry

This guide covers how zero-knowledge circuits are managed in the InterCooperative Network. Circuits can be added or upgraded without rebuilding nodes thanks to a dedicated registry and stable versioning rules.

## Circuit Registry Concepts

- Each circuit is identified by a unique **slug** and semantic **version** (for example `age_over_18` version `1.0.0`).
- Registered circuits store a Groth16 proving key, a verifying key and optional metadata.
- Nodes consult the registry when verifying credential proofs so that the correct versioned parameters are used.
- Circuits may be bundled with the node or registered dynamically through the HTTP API.
- Circuit metadata is represented by [`icn_zk::CircuitParameters`](../crates/icn-zk/src/params.rs) and managed through the `CircuitParametersStorage` trait.

## Versioning Rules and Compatibility

- Minor version bumps must remain backward compatible with proving keys generated for earlier minor versions.
- Patch versions are reserved for documentation or metadata changes only and MUST not alter the constraint system.
- Breaking changes require a new major version. Older versions remain in the registry for verifiers that still rely on them.
- Proofs include a circuit slug and version so verifiers can select the appropriate parameters.

## Registering Circuits via the API

The registry is manipulated through the `/circuits` endpoints.

### API Endpoints

- `POST /circuits/register` – store a new circuit version
- `GET  /circuits/{slug}/{version}` – fetch parameters and metadata
- `GET  /circuits` – list all slugs with available versions

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

### Retrieve Circuit Metadata

```bash
GET /circuits/age_over_18/1.0.0
```

Example response:

```json
{
  "slug": "age_over_18",
  "version": "1.0.0",
  "cid": "bafy...",
  "metadata": {}
}
```

## Proving Key and Parameter Storage

`Groth16KeyManager` persists parameters under `~/.icn/zk/`:

- `proving_key.bin` – compressed Groth16 proving key
- `verifying_key.bin` – verifying key bytes
- `verifying_key.sig` – Ed25519 signature authenticating the key

Parameters registered via the API are saved as [`CircuitParameters`](../crates/icn-zk/src/params.rs) within the registry database. Prepared verifying keys are cached in memory to speed up verification.

### Storage Layout

Circuit files are kept under the node data directory using the following structure:

```
~/.icn/registry/<slug>/<version>/
    proving_key.bin
    verifying_key.bin
    verifying_key.sig
    metadata.json
```

`icn_identity::zk::Groth16KeyManager` writes keys in this layout and any `CircuitParametersStorage` implementation may use the same paths for persistence.
