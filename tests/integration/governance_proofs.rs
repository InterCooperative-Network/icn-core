use icn_api::governance_trait::{CastVoteRequest, GovernanceApi, GovernanceApiImpl, ProposalInputType, SubmitProposalRequest};
use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_governance::GovernanceModule;
use std::sync::{Arc, Mutex};

fn dummy_proof(valid: bool) -> ZkCredentialProof {
    ZkCredentialProof {
        issuer: Did::new("key", "issuer"),
        holder: Did::new("key", "holder"),
        claim_type: "test".into(),
        proof: if valid { vec![1, 2, 3] } else { Vec::new() },
        schema: Cid::new_v1_sha256(0x55, b"schema"),
        vk_cid: None,
        disclosed_fields: Vec::new(),
        challenge: None,
        backend: ZkProofType::Other("dummy".into()),
        verification_key: None,
        public_inputs: None,
    }
}

#[test]
fn proposal_submission_proof_validation() {
    let gov = Arc::new(Mutex::new(GovernanceModule::new()));
    let api = GovernanceApiImpl::new(gov.clone());

    let ok_req = SubmitProposalRequest {
        proposer_did: "did:example:alice".into(),
        proposal: ProposalInputType::GenericText { text: "hi".into() },
        description: "test".into(),
        duration_secs: 60,
        quorum: None,
        threshold: None,
        body: None,
        credential_proof: Some(dummy_proof(true)),
        revocation_proof: None,
    };
    assert!(api.submit_proposal(ok_req).is_ok());

    let bad_req = SubmitProposalRequest {
        proposer_did: "did:example:alice".into(),
        proposal: ProposalInputType::GenericText { text: "bye".into() },
        description: "bad".into(),
        duration_secs: 60,
        quorum: None,
        threshold: None,
        body: None,
        credential_proof: Some(dummy_proof(false)),
        revocation_proof: None,
    };
    assert!(api.submit_proposal(bad_req).is_err());
}

#[test]
fn vote_proof_validation() {
    let gov = Arc::new(Mutex::new(GovernanceModule::new()));
    let api = GovernanceApiImpl::new(gov.clone());
    let submit = SubmitProposalRequest {
        proposer_did: "did:example:alice".into(),
        proposal: ProposalInputType::GenericText { text: "hi".into() },
        description: "test".into(),
        duration_secs: 60,
        quorum: None,
        threshold: None,
        body: None,
        credential_proof: Some(dummy_proof(true)),
        revocation_proof: None,
    };
    let pid = api.submit_proposal(submit).unwrap();
    {
        let mut gm = gov.lock().unwrap();
        gm.open_voting(&pid).unwrap();
    }

    let ok_vote = CastVoteRequest {
        voter_did: "did:example:bob".into(),
        proposal_id: pid.0.clone(),
        vote_option: "yes".into(),
        credential_proof: Some(dummy_proof(true)),
        revocation_proof: None,
    };
    assert!(api.cast_vote(ok_vote).is_ok());

    let bad_vote = CastVoteRequest {
        voter_did: "did:example:bob".into(),
        proposal_id: pid.0.clone(),
        vote_option: "yes".into(),
        credential_proof: Some(dummy_proof(false)),
        revocation_proof: None,
    };
    assert!(api.cast_vote(bad_vote).is_err());
}
