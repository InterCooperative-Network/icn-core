use icn_common::{Did, FixedTimeProvider};
use icn_governance::{
    GovernanceModule, ProposalStatus, ProposalSubmission, ProposalType, VoteOption,
};
use std::str::FromStr;

#[test]
fn delegation_affects_tally() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let mut gov = GovernanceModule::new();
    let alice = Did::from_str("did:example:alice").unwrap();
    let bob = Did::from_str("did:example:bob").unwrap();
    let carol = Did::from_str("did:example:carol").unwrap();
    gov.add_member(alice.clone());
    gov.add_member(bob.clone());
    gov.add_member(carol.clone());
    gov.set_quorum(2);
    gov.set_threshold(0.6);

    let pid = gov
        .submit_proposal(
            ProposalSubmission {
                proposer: alice.clone(),
                proposal_type: ProposalType::GenericText("delegate".into()),
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
    gov.open_voting(&pid).unwrap();

    gov.delegate_vote(alice.clone(), bob.clone()).unwrap();
    gov.cast_vote(bob.clone(), &pid, VoteOption::Yes, &time_provider)
        .unwrap();
    gov.cast_vote(carol.clone(), &pid, VoteOption::No, &time_provider)
        .unwrap();

    let (status, _) = gov.close_voting_period(&pid, &time_provider).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);

    // revoke and try again
    let pid2 = gov
        .submit_proposal(
            ProposalSubmission {
                proposer: alice.clone(),
                proposal_type: ProposalType::GenericText("delegate2".into()),
                description: "desc2".into(),
                duration_secs: 60,
                quorum: None,
                threshold: None,
                content_cid: None,
            timelock_delay: None,
            },
            &time_provider,
        )
        .unwrap();
    gov.open_voting(&pid2).unwrap();
    gov.delegate_vote(alice.clone(), bob.clone()).unwrap();
    gov.revoke_delegation(alice.clone());
    gov.cast_vote(bob.clone(), &pid2, VoteOption::Yes, &time_provider)
        .unwrap();
    gov.cast_vote(carol.clone(), &pid2, VoteOption::No, &time_provider)
        .unwrap();

    let (status2, _) = gov.close_voting_period(&pid2, &time_provider).unwrap();
    assert_eq!(status2, ProposalStatus::Rejected);
}
