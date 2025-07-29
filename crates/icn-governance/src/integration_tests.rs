//! Integration tests for governance primitives
//!
//! This module provides comprehensive tests for the governance system,
//! including ranked-choice voting, federated proposals, and edge cases.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ranked_choice::RankedChoiceVotingSystem;
    use crate::voting::{
        BallotId, CandidateId, ElectionId, RankedChoiceBallot, Signature, VotingSystem,
    };
    use icn_common::{Did, DidDocument, FixedTimeProvider};
    use icn_identity::KeyDidResolver;
    use std::sync::Arc;
    use std::time::SystemTime;

    /// Test the complete governance workflow with ranked choice voting
    #[test]
    fn test_complete_governance_workflow() {
        let time_provider = FixedTimeProvider::new(1640995200); // Start of 2022
        let mut governance = GovernanceModule::new();
        let voter_did = Did::default();
        governance.add_member(voter_did.clone());
        governance.set_quorum(1);
        governance.set_threshold(0.5);

        // Test proposal submission
        let submission = ProposalSubmission {
            proposer: voter_did.clone(),
            proposal_type: ProposalType::GenericText("Test ranked choice proposal".to_string()),
            description: "Testing governance integration".to_string(),
            duration_secs: 3600,
            quorum: None,
            threshold: None,
            content_cid: None,
        };

        let proposal_id = governance
            .submit_proposal(submission, &time_provider)
            .unwrap();

        // Test opening voting
        governance.open_voting(&proposal_id).unwrap();

        // Test voting
        governance
            .cast_vote(
                voter_did.clone(),
                &proposal_id,
                VoteOption::Yes,
                &time_provider,
            )
            .unwrap();

        // Test closing voting period
        let (status, (yes_votes, no_votes, abstain_votes)) = governance
            .close_voting_period(&proposal_id, &time_provider)
            .unwrap();

        assert_eq!(status, ProposalStatus::Accepted);
        assert_eq!(yes_votes, 1);
        assert_eq!(no_votes, 0);
        assert_eq!(abstain_votes, 0);
    }

    /// Test ranked choice voting with multiple candidates and elimination rounds
    #[test]
    fn test_ranked_choice_integration() {
        let did_resolver = Arc::new(KeyDidResolver);
        let rcv_system = RankedChoiceVotingSystem::new(did_resolver, 3);

        // Create test ballots with different preference orders
        let ballots = vec![
            create_test_ballot("ballot1", "election1", vec!["alice", "bob", "charlie"]),
            create_test_ballot("ballot2", "election1", vec!["bob", "alice", "charlie"]),
            create_test_ballot("ballot3", "election1", vec!["charlie", "alice", "bob"]),
            create_test_ballot("ballot4", "election1", vec!["alice", "charlie", "bob"]),
        ];

        // Test vote counting
        let result = rcv_system.count_votes(ballots).unwrap();

        assert_eq!(result.election_id, ElectionId("election1".to_string()));
        assert!(result.winner.is_some());
        assert_eq!(result.total_ballots, 4);
        assert!(!result.rounds.is_empty());
    }

    /// Test governance edge cases and error conditions
    #[test]
    fn test_governance_edge_cases() {
        let time_provider = FixedTimeProvider::new(1640995200); // Start of 2022
        let mut governance = GovernanceModule::new();
        let voter_did = Did::default();

        // Test voting without membership
        let submission = ProposalSubmission {
            proposer: voter_did.clone(),
            proposal_type: ProposalType::GenericText("Test proposal".to_string()),
            description: "Testing edge cases".to_string(),
            duration_secs: 3600,
            quorum: None,
            threshold: None,
            content_cid: None,
        };

        let proposal_id = governance
            .submit_proposal(submission, &time_provider)
            .unwrap();
        governance.open_voting(&proposal_id).unwrap();

        // Vote from non-member should work but not count toward quorum
        governance
            .cast_vote(
                voter_did.clone(),
                &proposal_id,
                VoteOption::Yes,
                &time_provider,
            )
            .unwrap();

        // Set high quorum that can't be met
        governance.set_quorum(10);
        let (status, _) = governance
            .close_voting_period(&proposal_id, &time_provider)
            .unwrap();

        // Should be rejected due to insufficient quorum
        assert_eq!(status, ProposalStatus::Rejected);
    }

    /// Test proposal expiration functionality
    #[test]
    fn test_proposal_expiration() {
        let time_provider = FixedTimeProvider::new(1640995200); // Start of 2022
        let mut governance = GovernanceModule::new();
        let voter_did = Did::default();
        governance.add_member(voter_did.clone());

        let submission = ProposalSubmission {
            proposer: voter_did.clone(),
            proposal_type: ProposalType::GenericText("Expiring proposal".to_string()),
            description: "Testing expiration".to_string(),
            duration_secs: 1, // Very short duration
            quorum: Some(1),
            threshold: Some(0.5),
            content_cid: None,
        };

        let proposal_id = governance
            .submit_proposal(submission, &time_provider)
            .unwrap();
        governance.open_voting(&proposal_id).unwrap();

        // Wait for expiration (simulate time passing)
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Try to vote on expired proposal
        let vote_result =
            governance.cast_vote(voter_did, &proposal_id, VoteOption::Yes, &time_provider);

        // Should fail due to expired deadline
        assert!(vote_result.is_err());
    }

    /// Test vote delegation functionality
    #[test]
    fn test_vote_delegation() {
        let time_provider = FixedTimeProvider::new(1640995200); // Start of 2022
        let mut governance = GovernanceModule::new();
        let delegator = Did::new("key", "delegator");
        let delegate = Did::new("key", "delegate");

        governance.add_member(delegator.clone());
        governance.add_member(delegate.clone());
        governance.set_quorum(1);
        governance.set_threshold(0.5);

        // Set up delegation
        governance
            .delegate_vote(delegator.clone(), delegate.clone())
            .unwrap();

        let submission = ProposalSubmission {
            proposer: delegate.clone(),
            proposal_type: ProposalType::GenericText("Delegation test".to_string()),
            description: "Testing vote delegation".to_string(),
            duration_secs: 3600,
            quorum: None,
            threshold: None,
            content_cid: None,
        };

        let proposal_id = governance
            .submit_proposal(submission, &time_provider)
            .unwrap();
        governance.open_voting(&proposal_id).unwrap();

        // Only delegate votes, but should count for both
        governance
            .cast_vote(
                delegate.clone(),
                &proposal_id,
                VoteOption::Yes,
                &time_provider,
            )
            .unwrap();

        let (status, (yes_votes, _, _)) = governance
            .close_voting_period(&proposal_id, &time_provider)
            .unwrap();

        assert_eq!(status, ProposalStatus::Accepted);
        assert_eq!(yes_votes, 2); // Should count delegate's vote for both members
    }

    /// Test integration with event store
    #[test]
    fn test_event_store_integration() {
        use icn_eventstore::MemoryEventStore;

        let time_provider = FixedTimeProvider::new(1640995200); // Start of 2022
        let event_store = Box::new(MemoryEventStore::new());
        let mut governance = GovernanceModule::with_event_store(event_store);

        let voter_did = Did::default();
        governance.add_member(voter_did.clone());

        let submission = ProposalSubmission {
            proposer: voter_did.clone(),
            proposal_type: ProposalType::GenericText("Event store test".to_string()),
            description: "Testing event store integration".to_string(),
            duration_secs: 3600,
            quorum: None,
            threshold: None,
            content_cid: None,
        };

        let proposal_id = governance
            .submit_proposal(submission, &time_provider)
            .unwrap();
        governance.open_voting(&proposal_id).unwrap();
        governance
            .cast_vote(voter_did, &proposal_id, VoteOption::Yes, &time_provider)
            .unwrap();

        // Verify events were recorded
        if let Some(store) = governance.event_store() {
            let events = store.lock().unwrap().query(None).unwrap();
            assert!(!events.is_empty());

            // Should have at least ProposalSubmitted and VoteCast events
            let has_proposal_submitted = events
                .iter()
                .any(|e| matches!(e, GovernanceEvent::ProposalSubmitted(_)));
            let has_vote_cast = events
                .iter()
                .any(|e| matches!(e, GovernanceEvent::VoteCast(_)));

            assert!(has_proposal_submitted);
            assert!(has_vote_cast);
        }
    }

    /// Test member management operations
    #[test]
    fn test_member_management() {
        let time_provider = FixedTimeProvider::new(1640995200); // Start of 2022
        let mut governance = GovernanceModule::new();
        let member1 = Did::new("key", "member1");
        let member2 = Did::new("key", "member2");

        // Test adding members
        governance.add_member(member1.clone());
        governance.add_member(member2.clone());

        assert!(governance.members().contains(&member1));
        assert!(governance.members().contains(&member2));
        assert_eq!(governance.members().len(), 2);

        // Test removing member
        governance.remove_member(&member1);
        assert!(!governance.members().contains(&member1));
        assert!(governance.members().contains(&member2));
        assert_eq!(governance.members().len(), 1);

        // Test proposal to add new member
        let submission = ProposalSubmission {
            proposer: member2.clone(),
            proposal_type: ProposalType::NewMemberInvitation(member1.clone()),
            description: "Re-add member1".to_string(),
            duration_secs: 3600,
            quorum: Some(1),
            threshold: Some(0.5),
            content_cid: None,
        };

        let proposal_id = governance
            .submit_proposal(submission, &time_provider)
            .unwrap();
        governance.open_voting(&proposal_id).unwrap();
        governance
            .cast_vote(member2, &proposal_id, VoteOption::Yes, &time_provider)
            .unwrap();
        governance
            .close_voting_period(&proposal_id, &time_provider)
            .unwrap();

        // Execute the proposal to add member back
        governance.execute_proposal(&proposal_id).unwrap();

        // Member should be added back
        assert!(governance.members().contains(&member1));
        assert_eq!(governance.members().len(), 2);
    }

    // Helper function to create test ballots
    fn create_test_ballot(
        ballot_id: &str,
        election_id: &str,
        preferences: Vec<&str>,
    ) -> RankedChoiceBallot {
        RankedChoiceBallot {
            ballot_id: BallotId(ballot_id.to_string()),
            voter_did: DidDocument {
                id: Did::default(),
                public_key: vec![0u8; 32],
            },
            election_id: ElectionId(election_id.to_string()),
            preferences: preferences
                .into_iter()
                .map(|s| CandidateId(s.to_string()))
                .collect(),
            timestamp: SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1640995200),
            signature: Signature {
                algorithm: "ed25519".to_string(),
                value: vec![0u8; 64],
            },
        }
    }
}
