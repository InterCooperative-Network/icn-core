use icn_common::Did;
use icn_governance::{
    GovernanceModule, ProposalStatus, ProposalSubmission, ProposalType, VoteOption,
};
use std::str::FromStr;

#[test]
fn execute_new_member_invitation_proposal() {
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.set_quorum(2);

    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::NewMemberInvitation(
                Did::from_str("did:example:dave").unwrap(),
            ),
            description: "invite dave".into(),
            duration_secs: 1,
            quorum: None,
            threshold: None,
            content_cid: None,
        })
        .unwrap();
    gov.open_voting(&pid).unwrap();
    gov.cast_vote(
        Did::from_str("did:example:alice").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    let (status, _) = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
    gov.execute_proposal(&pid).unwrap();
    assert!(gov
        .members()
        .contains(&Did::from_str("did:example:dave").unwrap()));
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Executed);
}

#[test]
fn execute_remove_member_proposal() {
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.set_quorum(2);

    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::RemoveMember(Did::from_str("did:example:bob").unwrap()),
            description: "remove bob".into(),
            duration_secs: 1,
            quorum: None,
            threshold: None,
            content_cid: None,
        })
        .unwrap();
    gov.open_voting(&pid).unwrap();
    gov.cast_vote(
        Did::from_str("did:example:alice").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    let (status, _) = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
    gov.execute_proposal(&pid).unwrap();
    assert!(!gov
        .members()
        .contains(&Did::from_str("did:example:bob").unwrap()));
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Executed);
}

#[test]
fn execute_parameter_change_callback() {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    let params: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut gov = GovernanceModule::new();
    let params_clone = params.clone();
    gov.set_callback(move |p| {
        if let ProposalType::SystemParameterChange(k, v) = &p.proposal_type {
            params_clone.lock().unwrap().insert(k.clone(), v.clone());
        }
        Ok(())
    });
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.set_quorum(2);

    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::SystemParameterChange("limit".into(), "10".into()),
            description: "update limit".into(),
            duration_secs: 1,
            quorum: None,
            threshold: None,
            content_cid: None,
        })
        .unwrap();
    gov.open_voting(&pid).unwrap();
    gov.cast_vote(
        Did::from_str("did:example:alice").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    let (status, _) = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
    gov.execute_proposal(&pid).unwrap();
    assert_eq!(params.lock().unwrap().get("limit").unwrap(), "10");
}

#[test]
fn execute_budget_allocation_callback() {
    use std::sync::{Arc, Mutex};

    let balance = Arc::new(Mutex::new(0u64));
    let mut gov = GovernanceModule::new();
    let bal_clone = balance.clone();
    gov.set_callback(move |p| {
        if let ProposalType::BudgetAllocation(amount, _purpose) = &p.proposal_type {
            let mut b = bal_clone.lock().unwrap();
            *b += *amount;
        }
        Ok(())
    });
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.set_quorum(2);

    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::BudgetAllocation(50, "dev".into()),
            description: "fund dev".into(),
            duration_secs: 1,
            quorum: None,
            threshold: None,
            content_cid: None,
        })
        .unwrap();
    gov.open_voting(&pid).unwrap();
    gov.cast_vote(
        Did::from_str("did:example:alice").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    let (status, _) = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
    gov.execute_proposal(&pid).unwrap();
    assert_eq!(*balance.lock().unwrap(), 50);
}
