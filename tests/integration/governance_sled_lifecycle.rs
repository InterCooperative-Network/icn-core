use icn_governance::{GovernanceModule, ProposalStatus, ProposalType, VoteOption};
use icn_common::Did;
use std::str::FromStr;
use tempfile::tempdir;

#[cfg(feature = "persist-sled")]
#[tokio::test]
async fn governance_proposal_lifecycle_sled() {
    let dir = tempdir().unwrap();
    let mut gov = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap();
    gov.add_member(Did::from_str("did:example:alice").unwrap());
    gov.add_member(Did::from_str("did:example:bob").unwrap());
    gov.set_quorum(2);

    let pid = gov
        .submit_proposal(
            Did::from_str("did:example:alice").unwrap(),
            ProposalType::GenericText("test".into()),
            "desc".into(),
            60,
        )
        .unwrap();
    gov
        .cast_vote(Did::from_str("did:example:bob").unwrap(), &pid, VoteOption::Yes)
        .unwrap();
    let status = gov.close_voting_period(&pid).unwrap();
    assert_eq!(status, ProposalStatus::Accepted);
    gov.execute_proposal(&pid).unwrap();

    drop(gov);
    let gov2 = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap();
    let prop = gov2.get_proposal(&pid).unwrap().unwrap();
    assert_eq!(prop.status, ProposalStatus::Executed);
}
