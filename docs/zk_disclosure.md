# Zero-Knowledge Proof Disclosure Guide

This document provides a short overview of when zero-knowledge proofs (ZKPs) are useful within ICN systems, how they can be generated, and a basic example flow.

---

## When to Use Zero-Knowledge Proofs
- **Selective Disclosure:** Reveal only the fact that a statement is true without exposing underlying data. Useful for credential verification (e.g., confirming age or membership).
- **Privacy-Preserving Audits:** Allow nodes to prove compliance with network policies without revealing sensitive logs.
- **Anonymous Voting:** Enable participants to vote without linking ballots to identities while still proving legitimacy.

## Generating Proofs
1. **Define the Statement** – Encode the claim to be proven (e.g., "age is over 18") in a circuit or constraint system.
2. **Prepare Inputs** – Collect private inputs (such as the birth year) and any necessary public parameters.
3. **Run the Prover** – Use a ZKP library or tool to generate the proof artifact and a corresponding public verification key.
4. **Publish Proof** – Send the proof alongside any required public data to the verifying party.
5. **Verify** – The verifier checks the proof using the verification key without learning the private inputs.

## Example Flow: Proving Age Over 18
1. **Holder** possesses a credential with a birthdate.
2. **Holder** runs a proving tool to create a ZKP that asserts "birthdate + 18 years <= current date".
3. **Holder** sends the resulting proof to a verifier (e.g., a cooperative registry).
4. **Verifier** checks the proof. If valid, it accepts that the holder is over 18 without ever seeing the actual birthdate.

See [`docs/examples/zk_example.json`](examples/zk_example.json) for a minimal JSON representation of a proof.

### Credential Proof JSON Format

`ZkCredentialProof` objects exchanged with ICN nodes are JSON encoded. Optional
fields may be omitted when not needed.

```json
{
  "issuer": "did:key:example:issuer",
  "holder": "did:key:example:holder",
  "claim_type": "age_over_18",
  "proof": [1, 2, 3],
  "schema": "bafyschemacid",
  "disclosed_fields": [],
  "challenge": null,
  "backend": "groth16",
  "verification_key": [1, 2, 3],
  "public_inputs": { "age": 21 }
}
```

## Available Circuits
The `icn-zk` crate exposes reusable circuits that can be compiled into proofs:

- `AgeOver18Circuit` – proves a birth year is at least 18 years in the past.
- `MembershipCircuit` – proves the subject is a registered member.
- `MembershipProofCircuit` – proves a private membership flag equals the expected value.
- `ReputationCircuit` – proves a reputation score meets a required threshold.
- `TimestampValidityCircuit` – proves a timestamp falls within a valid range.
- `BalanceRangeCircuit` – proves a private balance lies between a public minimum and maximum.
- `AgeRepMembershipCircuit` – proves age over 18, membership status, and reputation threshold in one proof.

See [`docs/examples/zk_age_over_18.json`](examples/zk_age_over_18.json) for a sample proof payload.
See [`docs/examples/zk_membership.json`](examples/zk_membership.json) for a membership proof example.

### Groth16KeyManager
`Groth16KeyManager` generates Groth16 parameters, stores them under
`~/.icn/zk/`, and signs the verifying key with an Ed25519 key. Use
`load_proving_key` and `verify_key_signature` to access the stored parameters
and confirm their authenticity.

## Runtime ABI

Two host functions are provided for working with zero-knowledge proofs:

- **`host_verify_zk_proof`** (`ABI index 25`) – accepts a JSON encoded
  [`ZkCredentialProof`](../crates/icn-common/src/zk.rs) and returns `true` when
  verification succeeds.
- **`host_generate_zk_proof`** (`ABI index 26`) – creates a dummy proof object
  from supplied parameters, useful for testing flows without a real prover.

When called from WASM, use the `wasm_host_verify_zk_proof` and
`wasm_host_generate_zk_proof` wrappers which handle passing strings in and out
of guest memory.

## Production Verification

Production nodes must verify the authenticity of the Groth16 verifying key
before accepting proofs. `Groth16KeyManager` stores `verifying_key.bin` and a
matching `verifying_key.sig` under `~/.icn/zk/`. The signature is created using
an Ed25519 signing key and can be checked with `verify_key_signature`.

`Groth16Verifier` caches prepared verifying keys in memory keyed by their CID to
avoid redundant deserialization. When a proof includes the `verification_key`
bytes, the verifier computes its CID and stores the prepared key. Proofs may
also supply a `vk_cid` pointing to the same key.

Example proof payload referencing a cached key:

```json
{
  "issuer": "did:key:federation",
  "holder": "did:key:member",
  "claim_type": "membership",
  "proof": "0xabc123",
  "schema": "bafyschemacid",
  "verification_key": "0xdeadbeef",
  "vk_cid": "bafyverifykeycid",
  "public_inputs": { "membership": true }
}
```
## Proof of Revocation

Verifiable credentials can be invalidated without revealing their contents. The `ZkRevocationProof` structure allows a holder to prove that a credential has not been revoked while keeping the revocation registry private.

1. **Issuer** marks the credential ID in its revocation registry and generates a zero-knowledge revocation proof.
2. **Holder** presents this `ZkRevocationProof` together with their credential or credential proof.
3. **Verifier** calls `verify_revocation` on a configured `ZkRevocationVerifier` implementation. If the proof succeeds, the credential is considered active without exposing registry entries.

Revocation proofs can be produced by `icn-identity`'s `Groth16Prover` or any custom prover implementing `ZkProver`.

## Example API Requests

Verify a credential proof:

```bash
curl -X POST http://localhost:7845/identity/verify \
     -H "Content-Type: application/json" \
     --data @docs/examples/zk_membership.json
```

Submit a DAG block with an attached proof:

```bash
curl -X POST http://localhost:7845/dag/put \
     -H "Content-Type: application/json" \
     --data @docs/examples/dag_put_with_proof.json
```

Nodes can enforce proof submission by creating the
`InMemoryPolicyEnforcer` with `require_proof` set to `true`.
