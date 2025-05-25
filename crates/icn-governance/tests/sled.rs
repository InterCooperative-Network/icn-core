#[cfg(feature = "persist-sled")]
mod tests {
    use icn_governance::{GovernanceModule, ProposalType, VoteOption};
    use icn_common::Did;
    use std::str::FromStr;
    use tempfile::tempdir;

    #[tokio::test]
    async fn sled_round_trip() {
        // temp DB directory
        let dir = tempdir().unwrap();
        let mut gov = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap(); // Pass PathBuf

        // 1. submit
        let pid = gov
            .submit_proposal(
                Did::from_str("did:example:alice").unwrap(),
                ProposalType::GenericText("hello".into()),
                "desc".into(),
                60,
            )
            .unwrap();

        // 2. vote
        gov.cast_vote(
            Did::from_str("did:example:bob").unwrap(),
            &pid,
            VoteOption::Yes,
        )
        .unwrap();

        // 3. persist + reload
        drop(gov);
        let gov2 = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap(); // Pass PathBuf

        let prop_opt = gov2.get_proposal(&pid).unwrap();
        let prop = prop_opt.as_ref().expect("Proposal should exist after reload");
        assert_eq!(prop.id, pid);
        assert_eq!(prop.votes.len(), 1);
    }
} 