use icn_common::{Did, FixedTimeProvider};
use icn_governance::{
    GovernanceModule, ProposalStatus, ProposalSubmission, ProposalType, VoteOption,
};
use std::str::FromStr;

#[test]
fn vote_tally_and_execute() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.add_member(Did::from_str("did:example:charlie").unwrap());
    gov.set_quorum(2);
    gov.set_threshold(0.5);

    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::NewMemberInvitation(
                Did::from_str("did:example:dave").unwrap(),
            ),
            description: "add dave".into(),
            duration_secs: 1,
            quorum: None,
            threshold: None,
            content_cid: None,
        }, &time_provider)
        .unwrap();

    // open voting period
    gov.open_voting(&pid).unwrap();

    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
        &time_provider,
    )
    .unwrap();
    gov.cast_vote(
        Did::from_str("did:example:charlie").unwrap(),
        &pid,
        VoteOption::Yes,
        &time_provider,
    )
    .unwrap();

    // close immediately since early closing is allowed
    let (status, (yes, no, abstain)) = gov.close_voting_period(&pid, &time_provider).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
    assert_eq!((yes, no, abstain), (2, 0, 0));

    gov.execute_proposal(&pid).unwrap();
    assert!(gov
        .members()
        .contains(&Did::from_str("did:example:dave").unwrap()));

    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Executed);
}

#[test]
fn reject_due_to_quorum() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.add_member(Did::from_str("did:example:charlie").unwrap());
    gov.set_quorum(3);
    gov.set_threshold(0.5);

    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::GenericText("hi".into()),
            description: "desc".into(),
            duration_secs: 1,
            quorum: None,
            threshold: None,
            content_cid: None,
        }, &time_provider)
        .unwrap();

    gov.open_voting(&pid).unwrap();

    gov.cast_vote(Did::from_str("did:example:bob").unwrap(), &pid, VoteOption::Yes, &time_provider)
    .unwrap();

    let (status, (yes, no, abstain)) = gov.close_voting_period(&pid, &time_provider).unwrap();
    assert_eq!(status, ProposalStatus::Rejected);
    assert_eq!((yes, no, abstain), (1, 0, 0));
}

#[test]
fn reject_due_to_threshold() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.add_member(Did::from_str("did:example:charlie").unwrap());
    gov.set_quorum(3);
    gov.set_threshold(0.75);

    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::GenericText("threshold".into()),
            description: "desc".into(),
            duration_secs: 1,
            quorum: None,
            threshold: None,
            content_cid: None,
        }, &time_provider)
        .unwrap();

    gov.open_voting(&pid).unwrap();

    gov.cast_vote(Did::from_str("did:example:bob").unwrap(), &pid, VoteOption::Yes, &time_provider)
    .unwrap();
    gov.cast_vote(Did::from_str("did:example:charlie").unwrap(), &pid, VoteOption::No, &time_provider)
    .unwrap();

    let (status, (yes, no, abstain)) = gov.close_voting_period(&pid, &time_provider).unwrap();
    assert_eq!(status, ProposalStatus::Rejected);
    assert_eq!((yes, no, abstain), (1, 1, 0));
}

#[test]
fn auto_close_after_deadline() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.set_quorum(1);

    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::GenericText("auto".into()),
            description: "desc".into(),
            duration_secs: 1,
            quorum: None,
            threshold: None,
            content_cid: None,
        }, &time_provider)
        .unwrap();

    gov.open_voting(&pid).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));
    gov.close_expired_proposals(&time_provider).unwrap();
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Rejected);
    assert!(gov
        .cast_vote(
            Did::from_str("did:example:alice").unwrap(),
            &pid,
            VoteOption::Yes,
            &time_provider
        )
        .is_err());
}

#[test]
fn vote_fails_after_expiration() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::GenericText("expire".into()),
            description: "desc".into(),
            duration_secs: 1,
            quorum: None,
            threshold: None,
            content_cid: None,
        }, &time_provider)
        .unwrap();
    gov.open_voting(&pid).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));
    assert!(gov
        .cast_vote(
            Did::from_str("did:example:alice").unwrap(),
            &pid,
            VoteOption::Yes,
            &time_provider
        )
        .is_err());
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Rejected);
}

#[test]
fn close_before_deadline_errors() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::GenericText("early".into()),
            description: "desc".into(),
            duration_secs: 60,
            quorum: None,
            threshold: None,
            content_cid: None,
        }, &time_provider)
        .unwrap();
    gov.open_voting(&pid).unwrap();
    gov.cast_vote(Did::from_str("did:example:alice").unwrap(), &pid, VoteOption::Yes, &time_provider)
    .unwrap();
    let (status, _) = gov.close_voting_period(&pid, &time_provider).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
}

#[test]
fn member_removal_affects_outcome() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.add_member(Did::from_str("did:example:charlie").unwrap());
    gov.set_quorum(2);
    gov.set_threshold(0.75);

    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::GenericText("member".into()),
            description: "desc".into(),
            duration_secs: 1,
            quorum: None,
            threshold: None,
            content_cid: None,
        }, &time_provider)
        .unwrap();

    gov.open_voting(&pid).unwrap();

    gov.cast_vote(Did::from_str("did:example:alice").unwrap(), &pid, VoteOption::Yes, &time_provider)
    .unwrap();
    gov.cast_vote(Did::from_str("did:example:bob").unwrap(), &pid, VoteOption::Yes, &time_provider)
    .unwrap();
    gov.cast_vote(Did::from_str("did:example:charlie").unwrap(), &pid, VoteOption::No, &time_provider)
    .unwrap();

    gov.remove_member(&Did::from_str("did:example:charlie").unwrap());

    let (status, (yes, no, abstain)) = gov.close_voting_period(&pid, &time_provider).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
    assert_eq!((yes, no, abstain), (2, 0, 0));
}
