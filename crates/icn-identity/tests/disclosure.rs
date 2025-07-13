use icn_common::{Cid, Did};
use icn_identity::{
    credential::CredentialIssuer,
    generate_ed25519_keypair,
    zk::{DummyProver, DummyVerifier, ZkVerifier},
};
use std::collections::HashMap;

#[test]
fn selective_disclosure_with_proof() {
    let (sk, vk) = generate_ed25519_keypair();
    let issuer_did = Did::new("key", "issuer");
    let holder_did = Did::new("key", "holder");

    let issuer = CredentialIssuer::new(issuer_did.clone(), sk).with_prover(Box::new(DummyProver));

    let mut claims = HashMap::new();
    claims.insert("age".to_string(), "30".to_string());
    claims.insert("role".to_string(), "tester".to_string());

    let (cred, _) = issuer
        .issue(
            holder_did.clone(),
            claims,
            Some(Cid::new_v1_sha256(0x55, b"schema")),
            None,
            None,
            Some(&["age"]),
        )
        .unwrap();

    // Disclose only the role field
    let (disclosed, proof) = cred.disclose_with_proof(&["role"], &DummyProver).unwrap();

    // Verify disclosed field signature
    disclosed.verify(&vk).unwrap();

    // Proof should mention the undisclosed field
    assert_eq!(proof.disclosed_fields, vec!["age".to_string()]);
    let verifier = DummyVerifier;
    assert!(verifier.verify(&proof).unwrap());
}
