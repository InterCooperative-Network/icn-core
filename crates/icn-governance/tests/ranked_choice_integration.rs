//! Integration tests for ranked choice voting functionality

use icn_governance::{
    BallotId, BallotValidator, Candidate, CandidateId, Election, ElectionId, EligibilityRules,
    RankedChoiceBallot, RankedChoiceBallotValidator, RankedChoiceVotingSystem, Signature,
    VotingError, VotingPeriod, VotingSystem,
};
use icn_common::Did;
use icn_identity::KeyDidResolver;
use std::sync::Arc;
use std::time::SystemTime;

fn create_test_did(id: &str) -> Did {
    // Create a simple test DID
    Did {
        method: "test".to_string(),
        id_string: id.to_string(),
        path: None,
        query: None,
        fragment: None,
    }
}

fn create_test_election() -> Election {
    let now = SystemTime::now();
    let voting_period = VotingPeriod {
        start_time: now,
        end_time: now + std::time::Duration::from_secs(3600), // 1 hour voting period
    };

    let candidates = vec![
        Candidate {
            id: CandidateId("alice".to_string()),
            name: "Alice Smith".to_string(),
            description: "Experienced leader with a focus on sustainability".to_string(),
            metadata: None,
        },
        Candidate {
            id: CandidateId("bob".to_string()),
            name: "Bob Jones".to_string(),
            description: "Community organizer with grassroots experience".to_string(),
            metadata: None,
        },
        Candidate {
            id: CandidateId("charlie".to_string()),
            name: "Charlie Brown".to_string(),
            description: "Technical expert with innovation focus".to_string(),
            metadata: None,
        },
    ];

    let eligibility_rules = EligibilityRules {
        required_credentials: vec![],
        min_reputation: None,
        required_federation: None,
        custom_rules: None,
    };

    Election {
        election_id: ElectionId("test-election-2023".to_string()),
        title: "ICN Governance Council Election 2023".to_string(),
        description: "Annual election for the ICN Governance Council".to_string(),
        candidates,
        voting_period,
        eligibility_rules,
        content_cid: None,
        creator: create_test_did("election-admin"),
        created_at: now,
    }
}

fn create_test_ballot(
    ballot_id: &str,
    voter_id: &str,
    election_id: &str,
    preferences: Vec<&str>,
) -> RankedChoiceBallot {
    RankedChoiceBallot {
        ballot_id: BallotId(ballot_id.to_string()),
        voter_did: create_test_did(voter_id),
        election_id: ElectionId(election_id.to_string()),
        preferences: preferences
            .into_iter()
            .map(|s| CandidateId(s.to_string()))
            .collect(),
        timestamp: SystemTime::now(),
        signature: Signature {
            algorithm: "ed25519".to_string(),
            value: vec![0u8; 64], // Mock signature
        },
    }
}

#[tokio::test]
async fn test_ranked_choice_voting_integration() {
    // Create voting system with DID resolver
    let did_resolver = Arc::new(KeyDidResolver);
    let voting_system = RankedChoiceVotingSystem::new(did_resolver, 1);

    // Create test ballots representing different voting scenarios
    let ballots = vec![
        // Voter 1: Alice first, Bob second, Charlie third
        create_test_ballot("ballot-001", "voter-001", "test-election-2023", vec!["alice", "bob", "charlie"]),
        // Voter 2: Alice first, Charlie second, Bob third  
        create_test_ballot("ballot-002", "voter-002", "test-election-2023", vec!["alice", "charlie", "bob"]),
        // Voter 3: Bob first, Alice second, Charlie third
        create_test_ballot("ballot-003", "voter-003", "test-election-2023", vec!["bob", "alice", "charlie"]),
        // Voter 4: Charlie first, Bob second, Alice third
        create_test_ballot("ballot-004", "voter-004", "test-election-2023", vec!["charlie", "bob", "alice"]),
        // Voter 5: Alice first, Bob second, Charlie third
        create_test_ballot("ballot-005", "voter-005", "test-election-2023", vec!["alice", "bob", "charlie"]),
    ];

    // Validate all ballots
    for ballot in &ballots {
        assert!(voting_system.validate_ballot(ballot).is_ok());
    }

    // Count votes using ranked choice algorithm
    let result = voting_system.count_votes(ballots).unwrap();

    // Verify the results
    assert_eq!(result.election_id, ElectionId("test-election-2023".to_string()));
    assert_eq!(result.total_ballots, 5);
    
    // Alice should win with 3 first-choice votes (majority)
    assert_eq!(result.winner, Some(CandidateId("alice".to_string())));
    
    // Should complete in one round since Alice has a majority
    assert_eq!(result.rounds.len(), 1);
    
    let first_round = &result.rounds[0];
    assert_eq!(first_round.round_number, 1);
    
    // Verify vote counts: Alice=3, Bob=1, Charlie=1
    assert_eq!(first_round.vote_counts.get(&CandidateId("alice".to_string())), Some(&3));
    assert_eq!(first_round.vote_counts.get(&CandidateId("bob".to_string())), Some(&1));
    assert_eq!(first_round.vote_counts.get(&CandidateId("charlie".to_string())), Some(&1));
}

#[tokio::test]
async fn test_ballot_validator_integration() {
    let did_resolver = Arc::new(KeyDidResolver);
    let validator = RankedChoiceBallotValidator::new(did_resolver);

    // Test valid ballot
    let valid_ballot = create_test_ballot("ballot-001", "voter-001", "test-election", vec!["alice", "bob"]);
    
    assert!(validator.validate_format(&valid_ballot).is_ok());
    assert!(validator.validate_signature(&valid_ballot).is_ok());
    assert!(validator.check_duplicate(&valid_ballot).is_ok());

    // Test duplicate detection
    let duplicate_ballot = create_test_ballot("ballot-002", "voter-001", "test-election", vec!["bob", "alice"]);
    
    // Second ballot from same voter should trigger duplicate detection
    assert!(matches!(
        validator.check_duplicate(&duplicate_ballot),
        Err(VotingError::DuplicateVote)
    ));
}

#[tokio::test]
async fn test_voting_system_eligibility() {
    let did_resolver = Arc::new(KeyDidResolver);
    let voting_system = RankedChoiceVotingSystem::new(did_resolver, 3);

    // Test voter eligibility
    assert!(voting_system.is_eligible_voter("did:test:voter-001").is_ok());
    
    // Test insufficient participation
    let single_ballot = vec![
        create_test_ballot("ballot-001", "voter-001", "test-election", vec!["alice"])
    ];
    
    let result = voting_system.count_votes(single_ballot);
    assert!(matches!(result, Err(VotingError::InvalidBallot(_))));
}

#[tokio::test]
async fn test_ballot_preference_validation() {
    // Test ballot with duplicate preferences
    let invalid_ballot = create_test_ballot("ballot-001", "voter-001", "test-election", vec!["alice", "bob", "alice"]);
    
    assert!(matches!(
        invalid_ballot.validate_preferences(),
        Err(VotingError::DuplicatePreferences)
    ));

    // Test ballot with valid preferences
    let valid_ballot = create_test_ballot("ballot-002", "voter-002", "test-election", vec!["alice", "bob", "charlie"]);
    
    assert!(valid_ballot.validate_preferences().is_ok());
    assert_eq!(valid_ballot.first_choice(), Some(&CandidateId("alice".to_string())));
    assert_eq!(valid_ballot.nth_choice(1), Some(&CandidateId("bob".to_string())));
    assert_eq!(valid_ballot.nth_choice(2), Some(&CandidateId("charlie".to_string())));
    assert_eq!(valid_ballot.nth_choice(3), None);
}

#[tokio::test]
async fn test_election_configuration() {
    let election = create_test_election();
    
    // Verify election properties
    assert_eq!(election.election_id, ElectionId("test-election-2023".to_string()));
    assert_eq!(election.title, "ICN Governance Council Election 2023");
    assert_eq!(election.candidates.len(), 3);
    
    // Verify candidates
    let candidate_names: Vec<_> = election.candidates.iter().map(|c| &c.name).collect();
    assert!(candidate_names.contains(&&"Alice Smith".to_string()));
    assert!(candidate_names.contains(&&"Bob Jones".to_string()));
    assert!(candidate_names.contains(&&"Charlie Brown".to_string()));
    
    // Verify voting period is active
    assert!(election.voting_period.is_active());
    assert!(!election.voting_period.has_ended());
    assert!(!election.voting_period.has_not_started());
}