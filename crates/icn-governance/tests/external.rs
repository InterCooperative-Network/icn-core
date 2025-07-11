use icn_common::Did;
use icn_governance::{GovernanceModule, ProposalSubmission, ProposalType};
use std::str::FromStr;

#[test]
fn insert_external_proposal() {
    let mut gov = GovernanceModule::new();
    let pid = gov
        .submit_proposal(ProposalSubmission {
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::GenericText("external".into()),
            description: "desc".into(),
            duration_secs: 60,
            quorum: None,
            threshold: None,
            content_cid: None,
        })
        .unwrap();
    let proposal = gov.get_proposal(&pid).unwrap().unwrap();

    let mut gov2 = GovernanceModule::new();
    gov2.insert_external_proposal(proposal.clone()).unwrap();
    let prop2 = gov2.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop2.id, pid);
    assert_eq!(prop2.description, proposal.description);
}
