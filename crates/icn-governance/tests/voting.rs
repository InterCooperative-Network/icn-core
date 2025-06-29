use icn_common::Did;
use icn_governance::{GovernanceModule, ProposalStatus, ProposalType, VoteOption};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn vote_tally_and_execute() {
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.add_member(Did::from_str("did:example:charlie").unwrap());
    gov.set_quorum(2);
    gov.set_threshold(0.5);

    let pid = gov
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::NewMemberInvitation(Did::from_str("did:example:dave").unwrap()),
            "add dave".into(),
            60,
        )
        .unwrap();

    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    gov.cast_vote(
        Did::from_str("did:example:charlie").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();

    let status = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);

    gov.execute_proposal(&pid).unwrap();
    assert!(gov
        .members()
        .contains(&Did::from_str("did:example:dave").unwrap()));

    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Executed);
}

#[test]
fn reject_due_to_quorum() {
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.add_member(Did::from_str("did:example:charlie").unwrap());
    gov.set_quorum(3);
    gov.set_threshold(0.5);

    let pid = gov
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::GenericText("hi".into()),
            "desc".into(),
            60,
        )
        .unwrap();

    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();

    let status = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Rejected);
}

#[test]
fn vote_fails_after_expiration() {
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());

    let pid = gov
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::GenericText("hi".into()),
            "desc".into(),
            1,
        )
        .unwrap();

    sleep(Duration::from_secs(2));

    assert!(gov
        .cast_vote(
            Did::from_str("did:example:alice").unwrap(),
            &pid,
            VoteOption::Yes,
        )
        .is_err());

    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Rejected);
}

#[test]
fn expired_proposal_auto_rejected_on_close() {
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());

    let pid = gov
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::GenericText("bye".into()),
            "desc".into(),
            1,
        )
        .unwrap();

    sleep(Duration::from_secs(2));

    let status = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Rejected);

    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Rejected);
}
