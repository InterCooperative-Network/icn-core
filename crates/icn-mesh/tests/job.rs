use icn_common::{Cid, Did};
use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, JobSpec};
use std::str::FromStr;

#[test]
fn job_creation_sign_verify() {
    let (sk, vk) = generate_ed25519_keypair();
    let did = Did::from_str(&did_key_from_verifying_key(&vk)).unwrap();
    let job = ActualMeshJob {
        id: JobId(Cid::new_v1_sha256(0x55, b"job")),
        manifest_cid: Cid::new_v1_sha256(0x55, b"manifest"),
        spec: JobSpec::default(),
        creator_did: did.clone(),
        cost_mana: 42,
        max_execution_wait_ms: None,
        signature: SignatureBytes(vec![]),
    };
    let signed = job.clone().sign(&sk).unwrap();
    assert!(signed.verify_signature(&vk).is_ok());

    let mut tampered = signed.clone();
    tampered.cost_mana = 100;
    assert!(tampered.verify_signature(&vk).is_err());
}
