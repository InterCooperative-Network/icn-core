use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use ark_serialize::CanonicalSerialize;
use ark_std::rand::{rngs::StdRng, SeedableRng};

use icn_api::submit_dag_block;
use icn_common::{
    compute_merkle_cid, Cid, DagBlock, Did, ZkCredentialProof, ZkProofType, ZkRevocationProof,
};
use icn_governance::scoped_policy::InMemoryPolicyEnforcer;
use icn_runtime::context::{DagStorageService, DagStoreMutexType, StubDagStore};
use icn_zk::{prove, setup, AgeOver18Circuit};

fn valid_groth16_proof_bytes() -> Vec<u8> {
    let circuit = AgeOver18Circuit {
        birth_year: 2000,
        current_year: 2020,
    };
    let mut rng = StdRng::seed_from_u64(42);
    let pk = setup(circuit.clone(), &mut rng).expect("setup");
    let proof = prove(&pk, circuit, &mut rng).expect("prove");
    let mut bytes = Vec::new();
    proof.serialize_uncompressed(&mut bytes).unwrap();
    bytes
}

fn create_block(author: &Did) -> DagBlock {
    let data = b"proposal".to_vec();
    let timestamp = 0;
    let cid = compute_merkle_cid(0x55, &data, &[], timestamp, author, &None, &None);
    DagBlock {
        cid,
        data,
        links: vec![],
        timestamp,
        author_did: author.clone(),
        signature: None,
        scope: None,
    }
}

fn setup_enforcer(actor: &Did) -> Arc<InMemoryPolicyEnforcer> {
    let mut submitters = HashSet::new();
    submitters.insert(actor.clone());
    Arc::new(InMemoryPolicyEnforcer::new(
        submitters,
        HashSet::new(),
        HashMap::new(),
        true,
    ))
}

#[tokio::test]
async fn submit_block_requires_credential_proof() {
    let actor = Did::from_str("did:icn:test:alice").unwrap();
    let store: Arc<DagStoreMutexType<DagStorageService>> =
        Arc::new(DagStoreMutexType::new(StubDagStore::new()));
    let enforcer = setup_enforcer(&actor);

    let block = create_block(&actor);
    let json = serde_json::to_string(&block).unwrap();
    let result = submit_dag_block(
        store.clone(),
        json,
        Some(enforcer.clone()),
        actor.clone(),
        None,
        None,
    )
    .await;
    match result {
        Err(icn_common::CommonError::PolicyDenied(reason)) => {
            assert!(reason.contains("credential proof required"))
        }
        other => panic!("unexpected result: {:?}", other),
    }
}

#[tokio::test]
async fn submit_block_with_invalid_proof_fails() {
    let actor = Did::from_str("did:icn:test:alice").unwrap();
    let store: Arc<DagStoreMutexType<DagStorageService>> =
        Arc::new(DagStoreMutexType::new(StubDagStore::new()));
    let enforcer = setup_enforcer(&actor);

    let block = create_block(&actor);
    let json = serde_json::to_string(&block).unwrap();

    let bad_proof = ZkCredentialProof {
        issuer: actor.clone(),
        holder: actor.clone(),
        claim_type: "test".into(),
        proof: Vec::new(),
        schema: Cid::new_v1_sha256(0x55, b"s"),
        vk_cid: None,
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: ZkProofType::Groth16,
        verification_key: None,
        public_inputs: None,
    };

    let res = submit_dag_block(
        store.clone(),
        json,
        Some(enforcer.clone()),
        actor.clone(),
        Some(bad_proof),
        None,
    )
    .await;
    match res {
        Err(icn_common::CommonError::PolicyDenied(reason)) => {
            assert!(reason.contains("credential proof invalid"))
        }
        other => panic!("unexpected result: {:?}", other),
    }
}

#[tokio::test]
async fn submit_block_with_valid_and_revocation_proof() {
    let actor = Did::from_str("did:icn:test:alice").unwrap();
    let store: Arc<DagStoreMutexType<DagStorageService>> =
        Arc::new(DagStoreMutexType::new(StubDagStore::new()));
    let enforcer = setup_enforcer(&actor);

    let block = create_block(&actor);
    let json = serde_json::to_string(&block).unwrap();

    let proof_bytes = valid_groth16_proof_bytes();
    let cred_proof = ZkCredentialProof {
        issuer: actor.clone(),
        holder: actor.clone(),
        claim_type: "age_over_18".into(),
        proof: proof_bytes.clone(),
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        vk_cid: None,
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: ZkProofType::Groth16,
        verification_key: None,
        public_inputs: None,
    };

    let rev_proof = ZkRevocationProof {
        issuer: actor.clone(),
        subject: actor.clone(),
        proof: proof_bytes,
        backend: ZkProofType::Groth16,
        verification_key: None,
        public_inputs: None,
    };

    let cred = cred_proof.clone();
    let rev = rev_proof.clone();

    // invalid revocation proof
    let bad_rev = ZkRevocationProof {
        issuer: actor.clone(),
        subject: actor.clone(),
        proof: Vec::new(),
        backend: ZkProofType::Groth16,
        verification_key: None,
        public_inputs: None,
    };
    let res = submit_dag_block(
        store.clone(),
        json.clone(),
        Some(enforcer.clone()),
        actor.clone(),
        Some(cred.clone()),
        Some(bad_rev),
    )
    .await;
    match res {
        Err(icn_common::CommonError::PolicyDenied(reason)) => {
            assert!(reason.contains("revocation proof invalid"))
        }
        other => panic!("unexpected result: {:?}", other),
    }

    // valid proofs
    let res = submit_dag_block(store, json, Some(enforcer), actor, Some(cred), Some(rev)).await;
    assert!(res.is_ok());
}
