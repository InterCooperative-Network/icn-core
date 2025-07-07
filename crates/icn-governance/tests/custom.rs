use icn_common::Did;
use icn_governance::{GovernanceModule, ProposalStatus, ProposalType, VoteOption};
use std::str::FromStr;

#[test]
fn proposal_specific_quorum_and_threshold() {
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.add_member(Did::from_str("did:example:charlie").unwrap());

    let pid = gov
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::GenericText("custom".into()),
            "desc".into(),
            1,
            Some(3),
            Some(0.75),
        )
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

    let status = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Rejected);
}
