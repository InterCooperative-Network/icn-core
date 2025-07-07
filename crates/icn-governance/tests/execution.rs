use icn_common::Did;
use icn_governance::{GovernanceModule, ProposalStatus, ProposalType, VoteOption};
use std::str::FromStr;

#[test]
#[ignore]
fn execute_new_member_invitation_proposal() {
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.set_quorum(2);

    let pid = gov
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::NewMemberInvitation(Did::from_str("did:example:dave").unwrap()),
            "invite dave".into(),
            60,
            None,
            None,
        )
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
    assert_eq!(
        gov.close_voting_period(&pid).unwrap(),
        ProposalStatus::Accepted
    );
    gov.execute_proposal(&pid).unwrap();
    assert!(gov
        .members()
        .contains(&Did::from_str("did:example:dave").unwrap()));
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Executed);
}

#[test]
#[ignore]
fn execute_remove_member_proposal() {
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.set_quorum(2);

    let pid = gov
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::RemoveMember(Did::from_str("did:example:bob").unwrap()),
            "remove bob".into(),
            60,
            None,
            None,
        )
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
    assert_eq!(
        gov.close_voting_period(&pid).unwrap(),
        ProposalStatus::Accepted
    );
    gov.execute_proposal(&pid).unwrap();
    assert!(!gov
        .members()
        .contains(&Did::from_str("did:example:bob").unwrap()));
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Executed);
}
