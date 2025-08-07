use icn_common::{Did, FixedTimeProvider};
use icn_governance::{
    GovernanceModule, ProposalStatus, ProposalSubmission, ProposalType, VoteOption,
};
use std::str::FromStr;

#[test]
fn custom_quorum_and_threshold() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let mut gov = GovernanceModule::new();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.add_member(Did::from_str("did:example:charlie").unwrap());

    let pid = gov
        .submit_proposal(
            ProposalSubmission {
                proposer: Did::from_str("did:example:alice").unwrap(),
                proposal_type: ProposalType::GenericText("custom".into()),
                description: "desc".into(),
                duration_secs: 60,
                quorum: Some(2),
                threshold: Some(0.75),
                content_cid: None,
                timelock_delay: None,
            },
            &time_provider,
        )
        .unwrap();

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
        VoteOption::No,
        &time_provider,
    )
    .unwrap();

    let (status, _) = gov.close_voting_period(&pid, &time_provider).unwrap();
    assert_eq!(status, ProposalStatus::Rejected);
}
