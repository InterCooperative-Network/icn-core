use icn_common::{Did, FixedTimeProvider};
use icn_governance::{
    GovernanceModule, ProposalStatus, ProposalSubmission, ProposalType, VoteOption,
};
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[test]
fn callback_runs_on_execute() {
    let time_provider = FixedTimeProvider::new(1640995200);
    let executed = Arc::new(AtomicBool::new(false));
    let mut gov = GovernanceModule::new();
    let flag = executed.clone();
    gov.set_callback(move |_p: &icn_governance::Proposal| {
        flag.store(true, Ordering::SeqCst);
        Ok(())
    });
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.set_quorum(2);

    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::GenericText("hi".into()),
            description: "test".into(),
            duration_secs: 1,
            quorum: None,
            threshold: None,
            content_cid: None,
        }, &time_provider)
        .unwrap();
    gov.open_voting(&pid).unwrap();
    gov.cast_vote(
        Did::from_str("did:example:alice").unwrap(),
        &pid,
        VoteOption::Yes,
        &time_provider,
    )
    .unwrap();
    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
        &time_provider,
    )
    .unwrap();
    let (status, _) = gov.close_voting_period(&pid, &time_provider).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
    gov.execute_proposal(&pid).unwrap();
    assert!(executed.load(Ordering::SeqCst));
}
