use icn_common::Did;
use icn_governance::{GovernanceModule, ProposalStatus, ProposalType, VoteOption};
use std::str::FromStr;

#[test]
fn open_voting_transitions_from_pending() {
    let mut gov = GovernanceModule::new();
    let pid = gov
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::GenericText("pending".into()),
            "desc".into(),
            60,
            None,
            None,
            None,
        )
        .unwrap();
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Pending);

    gov.open_voting(&pid).unwrap();
    let prop = gov.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::VotingOpen);
}

#[test]
fn vote_rejected_before_opening() {
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    let pid = gov
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::GenericText("vote".into()),
            "desc".into(),
            60,
            None,
            None,
            None,
        )
        .unwrap();

    let res = gov.cast_vote(
        Did::from_str("did:example:alice").unwrap(),
        &pid,
        VoteOption::Yes,
    );
    assert!(res.is_err());

    gov.open_voting(&pid).unwrap();
    assert!(gov
        .cast_vote(
            Did::from_str("did:example:alice").unwrap(),
            &pid,
            VoteOption::Yes
        )
        .is_ok());
}
