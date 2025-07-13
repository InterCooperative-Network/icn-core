use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use icn_common::{compute_merkle_cid, Cid, DagBlock, DagLink, Did, NodeScope, ZkCredentialProof};
use icn_governance::scoped_policy::{
    DagPayloadOp, InMemoryPolicyEnforcer, PolicyCheckResult, ScopedPolicyEnforcer,
};
use icn_identity::{
    credential::CredentialIssuer,
    generate_ed25519_keypair,
    zk::{Groth16Circuit, Groth16Prover},
};
use icn_runtime::context::RuntimeContext;

#[derive(Debug, PartialEq, Eq)]
enum PolicyError {
    Unauthorized,
    InvalidParent,
}

async fn anchor_block_with_policy<E: ScopedPolicyEnforcer>(
    ctx: &RuntimeContext,
    block: &DagBlock,
    enforcer: &E,
    proof: Option<&ZkCredentialProof>,
) -> Result<(), PolicyError> {
    if let PolicyCheckResult::Denied { .. } = enforcer.check_permission(
        DagPayloadOp::SubmitBlock,
        &block.author_did,
        block.scope.as_ref(),
        proof,
    ) {
        return Err(PolicyError::Unauthorized);
    }

    {
        let store = ctx.dag_store.lock().await;
        for link in &block.links {
            if !store.contains(&link.cid).unwrap() {
                return Err(PolicyError::InvalidParent);
            }
        }
    }

    {
        let mut store = ctx.dag_store.lock().await;
        store.put(block).unwrap();
    }
    Ok(())
}

#[tokio::test]
async fn authorized_dag_write_succeeds() {
    let ctx = RuntimeContext::new_with_stubs("did:example:alice").unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let mut submitters = HashSet::new();
    submitters.insert(alice.clone());
    let enforcer = InMemoryPolicyEnforcer::new(submitters, HashSet::new(), HashMap::new(), false);

    let data = b"block".to_vec();
    let ts = 0u64;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &alice, &None, &None);
    let block = DagBlock {
        cid: cid.clone(),
        data,
        links: vec![],
        timestamp: ts,
        author_did: alice.clone(),
        signature: None,
        scope: None,
    };

    anchor_block_with_policy(&ctx, &block, &enforcer, None)
        .await
        .expect("write succeeds");

    let stored = ctx.dag_store.lock().await.get(&cid).unwrap();
    assert!(stored.is_some());
}

#[tokio::test]
async fn unauthorized_write_denied() {
    let ctx = RuntimeContext::new_with_stubs("did:example:alice").unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let mut submitters = HashSet::new();
    submitters.insert(alice.clone());
    let enforcer = InMemoryPolicyEnforcer::new(submitters, HashSet::new(), HashMap::new(), false);

    let eve = Did::from_str("did:example:eve").unwrap();
    let data = b"bad".to_vec();
    let ts = 0u64;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &eve, &None, &None);
    let block = DagBlock {
        cid,
        data,
        links: vec![],
        timestamp: ts,
        author_did: eve.clone(),
        signature: None,
        scope: None,
    };

    let res = anchor_block_with_policy(&ctx, &block, &enforcer, None).await;
    assert_eq!(res, Err(PolicyError::Unauthorized));
}

#[tokio::test]
async fn invalid_parent_is_rejected() {
    let ctx = RuntimeContext::new_with_stubs("did:example:alice").unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let mut submitters = HashSet::new();
    submitters.insert(alice.clone());
    let enforcer = InMemoryPolicyEnforcer::new(submitters, HashSet::new(), HashMap::new(), false);

    let missing_cid = compute_merkle_cid(0x71, b"parent", &[], 0, &alice, &None, &None);
    let link = DagLink {
        cid: missing_cid,
        name: "parent".into(),
        size: 0,
    };
    let data = b"child".to_vec();
    let ts = 0u64;
    let cid = compute_merkle_cid(0x71, &data, &[link.clone()], ts, &alice, &None, &None);
    let block = DagBlock {
        cid,
        data,
        links: vec![link],
        timestamp: ts,
        author_did: alice.clone(),
        signature: None,
        scope: None,
    };

    let res = anchor_block_with_policy(&ctx, &block, &enforcer, None).await;
    assert_eq!(res, Err(PolicyError::InvalidParent));
}

#[tokio::test]
async fn scope_membership_enforced() {
    let ctx = RuntimeContext::new_with_stubs("did:example:alice").unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let scope = NodeScope("testscope".into());
    let mut submitters = HashSet::new();
    submitters.insert(alice.clone());
    let mut memberships = HashMap::new();
    memberships.insert(scope.clone(), {
        let mut set = HashSet::new();
        set.insert(alice.clone());
        set
    });
    let enforcer = InMemoryPolicyEnforcer::new(submitters, HashSet::new(), memberships, false);

    let data = b"scoped".to_vec();
    let ts = 0u64;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &alice, &None, &Some(scope.clone()));
    let block = DagBlock {
        cid,
        data,
        links: vec![],
        timestamp: ts,
        author_did: alice,
        signature: None,
        scope: Some(scope),
    };

    anchor_block_with_policy(&ctx, &block, &enforcer, None)
        .await
        .expect("scoped write succeeds");
}

#[tokio::test]
async fn proof_required_without_proof_fails() {
    let ctx = RuntimeContext::new_with_stubs("did:example:alice").unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let mut submitters = HashSet::new();
    submitters.insert(alice.clone());
    let enforcer = InMemoryPolicyEnforcer::new(submitters, HashSet::new(), HashMap::new(), true);

    let data = b"block".to_vec();
    let ts = 0u64;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &alice, &None, &None);
    let block = DagBlock {
        cid,
        data,
        links: vec![],
        timestamp: ts,
        author_did: alice.clone(),
        signature: None,
        scope: None,
    };

    let res = anchor_block_with_policy(&ctx, &block, &enforcer, None).await;
    assert_eq!(res, Err(PolicyError::Unauthorized));
}

#[tokio::test]
async fn proof_required_invalid_proof_fails() {
    let ctx = RuntimeContext::new_with_stubs("did:example:alice").unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let mut submitters = HashSet::new();
    submitters.insert(alice.clone());
    let enforcer = InMemoryPolicyEnforcer::new(submitters, HashSet::new(), HashMap::new(), true);

    let invalid = ZkCredentialProof {
        issuer: alice.clone(),
        holder: alice.clone(),
        claim_type: "age_over_18".into(),
        proof: vec![0u8; 10],
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        vk_cid: None,
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: icn_common::ZkProofType::Groth16,
        verification_key: None,
        public_inputs: None,
    };

    let data = b"block".to_vec();
    let ts = 0u64;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &alice, &None, &None);
    let block = DagBlock {
        cid,
        data,
        links: vec![],
        timestamp: ts,
        author_did: alice.clone(),
        signature: None,
        scope: None,
    };

    let res = anchor_block_with_policy(&ctx, &block, &enforcer, Some(&invalid)).await;
    assert_eq!(res, Err(PolicyError::Unauthorized));
}

#[tokio::test]
async fn proof_required_valid_proof_allows() {
    let ctx = RuntimeContext::new_with_stubs("did:example:alice").unwrap();
    let alice = Did::from_str("did:example:alice").unwrap();
    let mut submitters = HashSet::new();
    submitters.insert(alice.clone());
    let enforcer = InMemoryPolicyEnforcer::new(submitters, HashSet::new(), HashMap::new(), true);

    let (sk, _) = generate_ed25519_keypair();
    let issuer =
        CredentialIssuer::new(alice.clone(), sk).with_prover(Box::new(Groth16Prover::default()));
    let mut claims = HashMap::new();
    claims.insert("birth_year".to_string(), "2000".to_string());
    let (_, proof_opt) = issuer
        .issue(
            alice.clone(),
            claims,
            Some(Cid::new_v1_sha256(0x55, b"schema")),
            Some(&[]),
            Some(Groth16Circuit::AgeOver18 { current_year: 2020 }),
        )
        .unwrap();
    let proof = proof_opt.expect("proof");

    let data = b"block".to_vec();
    let ts = 0u64;
    let cid = compute_merkle_cid(0x71, &data, &[], ts, &alice, &None, &None);
    let block = DagBlock {
        cid: cid.clone(),
        data,
        links: vec![],
        timestamp: ts,
        author_did: alice.clone(),
        signature: None,
        scope: None,
    };

    anchor_block_with_policy(&ctx, &block, &enforcer, Some(&proof))
        .await
        .expect("write succeeds with proof");
    let stored = ctx.dag_store.lock().await.get(&cid).unwrap();
    assert!(stored.is_some());
}
