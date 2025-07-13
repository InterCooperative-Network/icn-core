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

See [`docs/examples/zk_age_over_18.json`](examples/zk_age_over_18.json) for a sample proof payload.
See [`docs/examples/zk_membership.json`](examples/zk_membership.json) for a membership proof example.

### Groth16KeyManager
`Groth16KeyManager` generates Groth16 parameters, stores them under
`~/.icn/zk/`, and signs the verifying key with an Ed25519 key. Use
`load_proving_key` and `verify_key_signature` to access the stored parameters
and confirm their authenticity.

## ZK Proof-of-Revocation

Credentials sometimes need to be invalidated without revealing the
specific credential. A *proof-of-revocation* allows a holder to show that a
credential has been revoked or, conversely, to prove they are **not** on a
revocation list without disclosing which credential they possess.

1. **Issuer publishes list** – Revoked credential identifiers are collected in
   a signed `RevocationList` anchored to the DAG.
2. **Holder generates proof** – Using the list's content identifier (CID), the
   holder produces a zero-knowledge proof that their credential identifier
   either appears on the list (for transparency) or does not (for selective
   disclosure).
3. **Verifier checks** – The verifier confirms the list's signature and verifies
   the proof with the corresponding verification key. No credential details are
   leaked during this process.

The `icn-zk` crate exposes circuits for building these proofs so that
revocation status can be proven privately in federated environments.
