use icn_common::Did;
use icn_eventstore::{EventStore, MemoryEventStore};
use icn_governance::{
    GovernanceModule, ProposalStatus, ProposalSubmission, ProposalType, VoteOption,
};
use std::str::FromStr;

#[test]
fn replay_governance_events() {
    let store = MemoryEventStore::new();
    let mut gov = GovernanceModule::with_event_store(Box::new(store));
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::GenericText("hello".into()),
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
    let _ = gov.close_voting_period(&pid).unwrap();
    gov.execute_proposal(&pid).unwrap();

    let events = gov
        .event_store()
        .unwrap()
        .lock()
        .unwrap()
        .query(None)
        .unwrap();
    let mut store2 = MemoryEventStore::new();
    for e in events.iter() {
        store2.append(e).unwrap();
    }
    let rebuilt = GovernanceModule::from_event_store(Box::new(store2)).unwrap();
    let prop = rebuilt.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Executed);
}
