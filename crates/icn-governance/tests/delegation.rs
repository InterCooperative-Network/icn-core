use icn_common::Did;
use icn_governance::{GovernanceModule, ProposalStatus, ProposalType, VoteOption};
use std::str::FromStr;

#[test]
fn delegation_affects_tally() {
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
            alice.clone(),
            ProposalType::GenericText("delegate".into()),
            "desc".into(),
            60,
            None,
            None,
        )
        .unwrap();
    gov.open_voting(&pid).unwrap();

    gov.delegate_vote(alice.clone(), bob.clone()).unwrap();
    gov.cast_vote(bob.clone(), &pid, VoteOption::Yes).unwrap();
    gov.cast_vote(carol.clone(), &pid, VoteOption::No).unwrap();

    let status = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);

    // revoke and try again
    let pid2 = gov
        .submit_proposal(
            alice.clone(),
            ProposalType::GenericText("delegate2".into()),
            "desc".into(),
            60,
            None,
            None,
        )
        .unwrap();
    gov.open_voting(&pid2).unwrap();
    gov.delegate_vote(alice.clone(), bob.clone()).unwrap();
    gov.revoke_delegation(alice.clone());
    gov.cast_vote(bob.clone(), &pid2, VoteOption::Yes).unwrap();
    gov.cast_vote(carol.clone(), &pid2, VoteOption::No).unwrap();

    let status2 = gov.close_voting_period(&pid2).unwrap();
    assert_eq!(status2, ProposalStatus::Rejected);
}
