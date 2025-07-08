use icn_common::Did;
use icn_governance::{GovernanceModule, ProposalType};
use std::str::FromStr;

#[test]
fn insert_external_proposal_round_trip() {
    let mut gov1 = GovernanceModule::new();
    let pid = gov1
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::GenericText("hi".into()),
            "desc".into(),
            60,
            None,
            None,
            None,
        )
        .unwrap();
    let proposal = gov1.get_proposal(&pid).unwrap().unwrap();

    let mut gov2 = GovernanceModule::new();
    gov2.insert_external_proposal(proposal.clone()).unwrap();
    let prop2 = gov2.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop2.id, pid);
    assert_eq!(prop2.description, proposal.description);
}
