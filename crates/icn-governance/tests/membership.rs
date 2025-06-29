use icn_common::Did;
use icn_governance::{GovernanceModule, ProposalStatus, ProposalType, VoteOption};
use std::str::FromStr;

#[test]
fn new_member_invitation_executes() {
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
        )
        .unwrap();

    gov.cast_vote(Did::from_str("did:example:alice").unwrap(), &pid, VoteOption::Yes).unwrap();
    gov.cast_vote(Did::from_str("did:example:bob").unwrap(), &pid, VoteOption::Yes).unwrap();

    assert_eq!(gov.close_voting_period(&pid).unwrap(), ProposalStatus::Accepted);
    gov.execute_proposal(&pid).unwrap();

    assert!(gov.members().contains(&Did::from_str("did:example:dave").unwrap()));
}

#[test]
fn remove_member_proposal_executes() {
    let mut gov = GovernanceModule::new();
    let alice = Did::from_str("did:example:alice").unwrap();
    let bob = Did::from_str("did:example:bob").unwrap();
    gov.add_member(alice.clone());
    gov.add_member(bob.clone());
    gov.set_quorum(2);

    let pid = gov
        .submit_proposal(
            alice.clone(),
            ProposalType::RemoveMember(bob.clone()),
            "remove bob".into(),
            60,
        )
        .unwrap();

    gov.cast_vote(alice.clone(), &pid, VoteOption::Yes).unwrap();
    gov.cast_vote(bob.clone(), &pid, VoteOption::Yes).unwrap();

    assert_eq!(gov.close_voting_period(&pid).unwrap(), ProposalStatus::Accepted);
    gov.execute_proposal(&pid).unwrap();

    assert!(!gov.members().contains(&bob));
}
