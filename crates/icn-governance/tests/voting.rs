use icn_common::Did;
use icn_governance::{
    GovernanceModule, ProposalStatus, ProposalSubmission, ProposalType, VoteOption,
};
use std::str::FromStr;

#[test]
fn vote_tally_and_execute() {
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
        })
        .unwrap();

    // open voting period
    gov.open_voting(&pid).unwrap();

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

    // close immediately since early closing is allowed
    let (status, (yes, no, abstain)) = gov.close_voting_period(&pid).unwrap();
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
        })
        .unwrap();

    gov.open_voting(&pid).unwrap();

    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();

    let (status, (yes, no, abstain)) = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Rejected);
    assert_eq!((yes, no, abstain), (1, 0, 0));
}

#[test]
fn reject_due_to_threshold() {
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
        })
        .unwrap();

    gov.open_voting(&pid).unwrap();

    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    gov.cast_vote(
        Did::from_str("did:example:charlie").unwrap(),
        &pid,
        VoteOption::No,
    )
    .unwrap();

    let (status, (yes, no, abstain)) = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Rejected);
    assert_eq!((yes, no, abstain), (1, 1, 0));
}

#[test]
fn auto_close_after_deadline() {
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
        })
        .unwrap();

    gov.open_voting(&pid).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));
    gov.close_expired_proposals().unwrap();
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Rejected);
    assert!(gov
        .cast_vote(
            Did::from_str("did:example:alice").unwrap(),
            &pid,
            VoteOption::Yes
        )
        .is_err());
}

#[test]
fn vote_fails_after_expiration() {
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
        })
        .unwrap();
    gov.open_voting(&pid).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));
    assert!(gov
        .cast_vote(
            Did::from_str("did:example:alice").unwrap(),
            &pid,
            VoteOption::Yes
        )
        .is_err());
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Rejected);
}

#[test]
fn close_before_deadline_errors() {
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
        })
        .unwrap();
    gov.open_voting(&pid).unwrap();
    gov.cast_vote(
        Did::from_str("did:example:alice").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    let (status, _) = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
}

#[test]
fn member_removal_affects_outcome() {
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
    gov.cast_vote(
        Did::from_str("did:example:charlie").unwrap(),
        &pid,
        VoteOption::No,
    )
    .unwrap();

    gov.remove_member(&Did::from_str("did:example:charlie").unwrap());

    let (status, (yes, no, abstain)) = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
    assert_eq!((yes, no, abstain), (2, 0, 0));
}
