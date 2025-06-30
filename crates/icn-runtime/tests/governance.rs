use icn_common::Did;
use icn_governance::{ProposalId, ProposalStatus, VoteOption};
use icn_runtime::context::RuntimeContext;
use icn_runtime::{
    host_cast_governance_vote, host_close_governance_proposal_voting,
    host_create_governance_proposal, host_execute_governance_proposal,
};
use std::str::FromStr;

#[tokio::test]
async fn proposal_can_be_closed_and_executed() {
    // setup context
    let ctx = RuntimeContext::new_with_stubs("did:icn:test:alice").unwrap();
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(Did::from_str("did:icn:test:alice").unwrap());
        gov.add_member(Did::from_str("did:icn:test:bob").unwrap());
        gov.add_member(Did::from_str("did:icn:test:charlie").unwrap());
        gov.set_quorum(2);
        gov.set_threshold(0.5);
    }
    // create proposal to add Dave
    let payload = serde_json::json!({
        "proposal_type_str": "NewMemberInvitation",
        "type_specific_payload": b"did:icn:test:dave".to_vec(),
        "description": "invite dave",
        "duration_secs": 60
    });
    let pid_str = host_create_governance_proposal(&ctx, &payload.to_string())
        .await
        .expect("create proposal");
    let pid = ProposalId(pid_str.clone());
    // cast votes directly using governance module
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.cast_vote(
            Did::from_str("did:icn:test:bob").unwrap(),
            &pid,
            VoteOption::Yes,
        )
        .unwrap();
        gov.cast_vote(
            Did::from_str("did:icn:test:charlie").unwrap(),
            &pid,
            VoteOption::Yes,
        )
        .unwrap();
    }
    // close voting
    let status = host_close_governance_proposal_voting(&ctx, &pid_str)
        .await
        .expect("close voting");
    assert_eq!(status, format!("{:?}", ProposalStatus::Accepted));
    // execute proposal
    host_execute_governance_proposal(&ctx, &pid_str)
        .await
        .expect("execute proposal");
    {
        let gov = ctx.governance_module.lock().await;
        assert!(gov
            .members()
            .contains(&Did::from_str("did:icn:test:dave").unwrap()));
        let prop = gov.get_proposal(&pid).unwrap().unwrap();
        assert_eq!(prop.status, ProposalStatus::Executed);
    }
}

#[tokio::test]
async fn proposal_creation_requires_mana() {
    let ctx = RuntimeContext::new_with_stubs_and_mana(
        "did:icn:test:mana_proposal",
        icn_runtime::context::PROPOSAL_COST_MANA - 1,
    )
    .unwrap();

    let payload = serde_json::json!({
        "proposal_type_str": "GenericText",
        "type_specific_payload": b"hi".to_vec(),
        "description": "test",
        "duration_secs": 60
    });

    let result = host_create_governance_proposal(&ctx, &payload.to_string()).await;
    assert!(matches!(
        result,
        Err(icn_runtime::context::HostAbiError::InsufficientMana)
    ));
}

#[tokio::test]
async fn vote_requires_mana() {
    // Give just enough mana for proposal creation
    let ctx = RuntimeContext::new_with_stubs_and_mana(
        "did:icn:test:mana_vote",
        icn_runtime::context::PROPOSAL_COST_MANA,
    )
    .unwrap();

    let payload = serde_json::json!({
        "proposal_type_str": "GenericText",
        "type_specific_payload": b"hey".to_vec(),
        "description": "test",
        "duration_secs": 60
    });
    let pid = host_create_governance_proposal(&ctx, &payload.to_string())
        .await
        .expect("create proposal");

    let vote_payload = serde_json::json!({
        "proposal_id_str": pid,
        "vote_option_str": "yes"
    });

    let result = host_cast_governance_vote(&ctx, &vote_payload.to_string()).await;
    assert!(matches!(
        result,
        Err(icn_runtime::context::HostAbiError::InsufficientMana)
    ));
}

#[tokio::test]
async fn vote_succeeds_with_sufficient_mana() {
    let ctx = RuntimeContext::new_with_stubs_and_mana(
        "did:icn:test:enough_mana",
        icn_runtime::context::PROPOSAL_COST_MANA + icn_runtime::context::VOTE_COST_MANA,
    )
    .unwrap();

    let payload = serde_json::json!({
        "proposal_type_str": "GenericText",
        "type_specific_payload": b"hello".to_vec(),
        "description": "test",
        "duration_secs": 60
    });
    let pid = host_create_governance_proposal(&ctx, &payload.to_string())
        .await
        .expect("create proposal");

    let vote_payload = serde_json::json!({
        "proposal_id_str": pid,
        "vote_option_str": "yes"
    });

    let result = host_cast_governance_vote(&ctx, &vote_payload.to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn lifecycle_with_member_add_and_remove() {
    let ctx = RuntimeContext::new_with_stubs("did:icn:test:alice").unwrap();
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(Did::from_str("did:icn:test:alice").unwrap());
        gov.add_member(Did::from_str("did:icn:test:bob").unwrap());
        gov.set_quorum(2);
        gov.set_threshold(0.5);
    }

    let payload = serde_json::json!({
        "proposal_type_str": "NewMemberInvitation",
        "type_specific_payload": b"did:icn:test:dave".to_vec(),
        "description": "invite dave",
        "duration_secs": 60
    });
    let pid_str = host_create_governance_proposal(&ctx, &payload.to_string())
        .await
        .expect("create proposal");
    let pid = ProposalId(pid_str.clone());
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.cast_vote(
            Did::from_str("did:icn:test:bob").unwrap(),
            &pid,
            VoteOption::Yes,
        )
        .unwrap();
    }
    let status = host_close_governance_proposal_voting(&ctx, &pid_str)
        .await
        .expect("close voting");
    assert_eq!(status, format!("{:?}", ProposalStatus::Accepted));

    host_execute_governance_proposal(&ctx, &pid_str)
        .await
        .expect("execute proposal");

    {
        let mut gov = ctx.governance_module.lock().await;
        assert!(gov
            .members()
            .contains(&Did::from_str("did:icn:test:dave").unwrap()));
        gov.remove_member(&Did::from_str("did:icn:test:dave").unwrap());
        assert!(!gov
            .members()
            .contains(&Did::from_str("did:icn:test:dave").unwrap()));
    }
}
