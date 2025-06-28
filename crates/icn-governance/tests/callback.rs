use icn_common::Did;
use icn_governance::{GovernanceModule, ProposalStatus, ProposalType, VoteOption};
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[test]
fn callback_runs_on_execute() {
    let executed = Arc::new(AtomicBool::new(false));
    let mut gov = GovernanceModule::new();
    let flag = executed.clone();
    gov.set_callback(move |_p| {
        flag.store(true, Ordering::SeqCst);
        Ok(())
    });
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.set_quorum(2);

    let pid = gov
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::GenericText("hi".into()),
            "test".into(),
            60,
        )
        .unwrap();
    gov.cast_vote(
        Did::from_str("did:example:bob").unwrap(),
        &pid,
        VoteOption::Yes,
    )
    .unwrap();
    assert_eq!(
        gov.close_voting_period(&pid).unwrap(),
        ProposalStatus::Accepted
    );
    gov.execute_proposal(&pid).unwrap();
    assert!(executed.load(Ordering::SeqCst));
}
