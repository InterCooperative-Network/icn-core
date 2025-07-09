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
    let result_json = host_close_governance_proposal_voting(&ctx, &pid_str)
        .await
        .expect("close voting");
    let result: icn_api::governance_trait::CloseProposalResponse =
        serde_json::from_str(&result_json).unwrap();
    assert_eq!(result.status, format!("{:?}", ProposalStatus::Accepted));
    assert_eq!((result.yes, result.no, result.abstain), (2, 0, 0));
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
    let res_json = host_close_governance_proposal_voting(&ctx, &pid_str)
        .await
        .expect("close voting");
    let res: icn_api::governance_trait::CloseProposalResponse =
        serde_json::from_str(&res_json).unwrap();
    assert_eq!(res.status, format!("{:?}", ProposalStatus::Accepted));
    assert_eq!((res.yes, res.no, res.abstain), (1, 0, 0));

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

#[tokio::test]
async fn execution_rewards_proposer() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:test:alice", 0).unwrap();
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(Did::from_str("did:icn:test:alice").unwrap());
        gov.add_member(Did::from_str("did:icn:test:bob").unwrap());
        gov.set_quorum(1);
        gov.set_threshold(0.5);
    }

    let payload = serde_json::json!({
        "proposal_type_str": "GenericText",
        "type_specific_payload": b"hello".to_vec(),
        "description": "hello world",
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
    let res_json = host_close_governance_proposal_voting(&ctx, &pid_str)
        .await
        .expect("close voting");
    let res: icn_api::governance_trait::CloseProposalResponse =
        serde_json::from_str(&res_json).unwrap();
    assert_eq!((res.yes, res.no, res.abstain), (1, 0, 0));

    let mana_before = ctx.mana_ledger.get_balance(&ctx.current_identity);
    let rep_before = ctx.reputation_store.get_reputation(&ctx.current_identity);

    host_execute_governance_proposal(&ctx, &pid_str)
        .await
        .expect("execute proposal");

    let mana_after = ctx.mana_ledger.get_balance(&ctx.current_identity);
    let rep_after = ctx.reputation_store.get_reputation(&ctx.current_identity);

    assert_eq!(mana_after, mana_before + 1);
    assert!(rep_after > rep_before);
}

#[tokio::test]
async fn failed_execution_no_rewards() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:test:alice", 0).unwrap();
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(Did::from_str("did:icn:test:alice").unwrap());
    }

    ctx.reputation_store
        .record_execution(&ctx.current_identity, true, 0);

    let payload = serde_json::json!({
        "proposal_type_str": "GenericText",
        "type_specific_payload": b"oops".to_vec(),
        "description": "bad exec",
        "duration_secs": 60
    });
    let pid_str = host_create_governance_proposal(&ctx, &payload.to_string())
        .await
        .expect("create proposal");

    let mana_before = ctx.mana_ledger.get_balance(&ctx.current_identity);
    let rep_before = ctx.reputation_store.get_reputation(&ctx.current_identity);

    let res = host_execute_governance_proposal(&ctx, &pid_str).await;
    assert!(res.is_err());

    let mana_after = ctx.mana_ledger.get_balance(&ctx.current_identity);
    let rep_after = ctx.reputation_store.get_reputation(&ctx.current_identity);

    assert_eq!(mana_after, mana_before);
    assert!(rep_after < rep_before);
}

#[tokio::test]
async fn proposal_body_is_stored_in_dag() {
    let ctx = RuntimeContext::new_with_stubs("did:icn:test:cid").unwrap();
    {
        let mut gov = ctx.governance_module.lock().await;
        gov.add_member(Did::from_str("did:icn:test:cid").unwrap());
    }
    let body_bytes = b"full proposal text".to_vec();
    let payload = serde_json::json!({
        "proposal_type_str": "GenericText",
        "type_specific_payload": b"short".to_vec(),
        "description": "with body",
        "duration_secs": 60,
        "body": body_bytes,
    });
    let pid_str = host_create_governance_proposal(&ctx, &payload.to_string())
        .await
        .expect("create proposal");
    let pid = ProposalId(pid_str);
    let gov = ctx.governance_module.lock().await;
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    let cid = prop.content_cid.expect("cid stored");
    drop(gov);
    let store = ctx.dag_store.lock().await;
    let block = store.get(&cid).unwrap().expect("block stored");
    assert_eq!(block.data, b"full proposal text");
}
