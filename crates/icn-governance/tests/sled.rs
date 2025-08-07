#[cfg(feature = "persist-sled")]
mod tests {
    use icn_common::{Did, FixedTimeProvider};
    use icn_governance::{
        GovernanceModule, ProposalStatus, ProposalSubmission, ProposalType, VoteOption, Proposal, ProposalSponsorship,
    };
    use std::str::FromStr;
    use tempfile::tempdir;

    #[tokio::test]
    async fn sled_round_trip() {
        let time_provider = FixedTimeProvider::new(1640995200);
        // temp DB directory
        let dir = tempdir().unwrap();
        let mut gov = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap(); // Pass PathBuf

        // 1. submit
        let pid = gov
            .submit_proposal(
                ProposalSubmission {
                    proposer: Did::from_str("did:example:alice").unwrap(),
                    proposal_type: ProposalType::GenericText("hello".into()),
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

        // 2. vote
        gov.cast_vote(
            Did::from_str("did:example:bob").unwrap(),
            &pid,
            VoteOption::Yes,
            &time_provider,
        )
        .unwrap();

        // 3. persist + reload
        drop(gov);
        let gov2 = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap(); // Pass PathBuf

        let prop_opt = gov2.get_proposal(&pid).unwrap();
        let prop = prop_opt
            .as_ref()
            .expect("Proposal should exist after reload");
        assert_eq!(prop.id, pid);
        assert_eq!(prop.votes.len(), 1);
    }

    #[tokio::test]
    async fn sled_execute_persist() {
        let time_provider = FixedTimeProvider::new(1640995200);
        let dir = tempdir().unwrap();
        let mut gov = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap();
        gov.add_member(Did::from_str("did:example:alice").unwrap());
        gov.add_member(Did::from_str("did:example:bob").unwrap());
        gov.set_quorum(2);
        let pid = gov
            .submit_proposal(
                ProposalSubmission {
                    proposer: Did::from_str("did:example:alice").unwrap(),
                    proposal_type: ProposalType::GenericText("hi".into()),
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
        let _ = gov.close_voting_period(&pid, &time_provider).unwrap();
        gov.execute_proposal(&pid).unwrap();
        drop(gov);
        let gov2 = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap();
        let prop = gov2.get_proposal(&pid).unwrap().unwrap();
        assert_eq!(prop.status, ProposalStatus::Executed);
    }

    #[tokio::test]
    async fn sled_external_proposal_persists() {
        use icn_governance::{Proposal, ProposalId};
        use std::collections::HashMap;

        let dir = tempdir().unwrap();
        let mut gov = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap();

        let now = 1640995200u64; // Fixed timestamp
        let pid = ProposalId("ext-prop-1".to_string());
        let proposal = Proposal {
            id: pid.clone(),
            proposer: Did::from_str("did:example:alice").unwrap(),
            proposal_type: ProposalType::GenericText("external".into()),
            description: "external".into(),
            created_at: now,
            voting_deadline: now + 60,
            status: ProposalStatus::VotingOpen,
            votes: HashMap::new(),
            quorum: None,
            threshold: None,
            content_cid: None,
            timelock_delay: None,
            sponsorship: ProposalSponsorship::new(),
            accepted_at: None,
            veto: None,
        };

        gov.insert_external_proposal(proposal.clone()).unwrap();
        drop(gov);

        let gov2 = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap();
        let stored = gov2.get_proposal(&pid).unwrap().unwrap();
        assert_eq!(stored.description, proposal.description);
    }

    #[tokio::test]
    async fn sled_external_vote_persists() {
        use icn_governance::Vote;

        let time_provider = FixedTimeProvider::new(1640995200);
        let dir = tempdir().unwrap();
        let mut gov = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap();
        let pid = gov
            .submit_proposal(
                ProposalSubmission {
                    proposer: Did::from_str("did:example:alice").unwrap(),
                    proposal_type: ProposalType::GenericText("vote".into()),
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
        drop(gov);

        let mut gov2 = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap();
        let now = 1640995201u64; // Fixed timestamp
        let vote = Vote {
            voter: Did::from_str("did:example:bob").unwrap(),
            proposal_id: pid.clone(),
            option: VoteOption::Yes,
            voted_at: now,
        };
        gov2.insert_external_vote(vote).unwrap();
        drop(gov2);

        let gov3 = GovernanceModule::new_sled(dir.path().to_path_buf()).unwrap();
        let prop = gov3.get_proposal(&pid).unwrap().unwrap();
        assert_eq!(prop.votes.len(), 1);
    }
}
