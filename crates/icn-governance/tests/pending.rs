use icn_common::{Did, FixedTimeProvider};
use icn_governance::{
    GovernanceModule, ProposalStatus, ProposalSubmission, ProposalType, VoteOption,
};
use std::str::FromStr;

#[test]
fn open_voting_transitions_from_deliberation() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let mut gov = GovernanceModule::new();
    let pid = gov
        .submit_proposal(
            ProposalSubmission {
                proposer: Did::from_str("did:example:alice").unwrap(),
                proposal_type: ProposalType::GenericText("pending".into()),
                description: "desc".into(),
                duration_secs: 60,
                quorum: None,
                threshold: None,
                content_cid: None,
                timelock_delay: None,
            },
            &time_provider,
        )
        .unwrap();
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Deliberation);

    gov.open_voting(&pid).unwrap();
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::VotingOpen);
}

#[test]
fn vote_rejected_before_opening() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    let pid = gov
        .submit_proposal(
            ProposalSubmission {
                proposer: Did::from_str("did:example:alice").unwrap(),
                proposal_type: ProposalType::GenericText("vote".into()),
                description: "desc".into(),
                duration_secs: 60,
                quorum: None,
                threshold: None,
                content_cid: None,
                timelock_delay: None,
            },
            &time_provider,
        )
        .unwrap();

    let res = gov.cast_vote(
        Did::from_str("did:example:alice").unwrap(),
        &pid,
        VoteOption::Yes,
        &time_provider,
    );
    assert!(res.is_err());

    gov.open_voting(&pid).unwrap();
    assert!(gov
        .cast_vote(
            Did::from_str("did:example:alice").unwrap(),
            &pid,
            VoteOption::Yes,
            &time_provider
        )
        .is_ok());
}
