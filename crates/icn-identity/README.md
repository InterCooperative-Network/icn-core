# ICN Identity Crate

This crate manages decentralized identities (DIDs), verifiable credentials (VCs), and cryptographic operations for users and nodes within the InterCooperative Network (ICN).

See [CONTEXT.md](../../CONTEXT.md) for ICN Core design philosophy and crate roles.
See [docs/ASYNC_OVERVIEW.md](../../docs/ASYNC_OVERVIEW.md) for async API guidelines.

## Purpose

The `icn-identity` crate is responsible for:

*   **Decentralized Identifiers (DIDs):** Generating, resolving, and managing DIDs according to relevant specifications (e.g., W3C DID Core).
*   **Verifiable Credentials (VCs):** Issuing, verifying, and managing VCs for attestations and authorizations within the network.
*   **Cryptographic Operations:** Providing and managing cryptographic keys, signatures, and encryption/decryption services necessary for identity and secure communication.
*   **Authentication & Authorization:** Defining mechanisms for proving identity and controlling access to resources or actions.

This crate is fundamental for establishing trust and security within the ICN.

## Public API Style

The API style emphasizes:

*   **Security:** Strong cryptographic practices and resistance to identity-related attacks.
*   **Interoperability:** Adherence to established standards for DIDs and VCs to ensure compatibility with other systems.
*   **Usability:** Clear interfaces for managing identities and credentials.

## DID Methods

* **`did:key`** – deterministic key-based identifiers with helper functions for
  creation and resolving to verifying keys.
* **`did:web`** – identifiers served from standard `did.json` documents. The
  [`WebDidResolver`] can fetch these documents over HTTPS at runtime to obtain
  the public key.
* **`did:peer`** – basic support for [algorithm 0](https://identity.foundation/peer-did-method-spec/).
  Keys can be encoded with [`did_peer_from_verifying_key`] and resolved with
  [`verifying_key_from_did_peer`].

### `did:web` segment validation

`did_web_from_parts` returns an error if any domain label or path segment
contains characters other than ASCII letters, digits, `-`, `_`, or `.` or if a
segment exceeds 63 characters. Domains longer than 253 characters are also
rejected.

## Zero-Knowledge Provers

Credential issuance can optionally generate zero-knowledge proofs via the
`ZkProver` trait. This crate includes the following prover implementations:

- `DummyProver` – generates placeholder proofs for testing.
- `BulletproofsProver` – produces range proofs using the Bulletproofs protocol.
- `Groth16Prover` – generic prover for Groth16 circuits such as age, membership or reputation checks.

## Credential Revocation Workflow

Zero-knowledge revocation proofs allow verifiers to check that a credential remains valid without revealing registry details.

1. An issuer records the credential identifier in its revocation registry and creates a `ZkRevocationProof`.
2. The holder includes this proof when presenting the credential or a credential proof.
3. Verifiers call `verify_revocation` using their configured `ZkRevocationVerifier`. A successful result confirms the credential is not revoked.

## Delegated Credentials

Delegated credentials allow one DID to delegate authority to another. A chain
of such credentials proves transitive delegation from the original issuer to the
final holder.

```rust
use icn_identity::{
    delegated_credential::{DelegatedCredential, verify_delegation_chain},
    generate_ed25519_keypair, did_key_from_verifying_key, KeyDidResolver,
};
use icn_common::Did;
use std::str::FromStr;

let (sk_a, vk_a) = generate_ed25519_keypair();
let did_a = Did::from_str(&did_key_from_verifying_key(&vk_a)).unwrap();
let (sk_b, vk_b) = generate_ed25519_keypair();
let did_b = Did::from_str(&did_key_from_verifying_key(&vk_b)).unwrap();
let (sk_c, vk_c) = generate_ed25519_keypair();
let did_c = Did::from_str(&did_key_from_verifying_key(&vk_c)).unwrap();

let d1 = DelegatedCredential::new(did_a.clone(), did_b.clone(), &sk_a);
let d2 = DelegatedCredential::new(did_b.clone(), did_c.clone(), &sk_b);

let resolver = KeyDidResolver;
verify_delegation_chain(&did_a, &[d1, d2], &resolver).unwrap();
```

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root of the `icn-core` repository for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE). 
