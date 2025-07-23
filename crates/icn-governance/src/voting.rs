//! Core voting primitives and traits for ICN governance
//! 
//! This module provides the foundational types and traits for implementing
//! sophisticated voting mechanisms including ranked choice voting.

use icn_common::{Cid, Did};
use std::any::Any;
use std::time::SystemTime;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Comprehensive error types for voting operations
#[derive(Debug, Error)]
pub enum VotingError {
    #[error("Invalid ballot format: {0}")]
    InvalidBallot(String),
    #[error("Voter not eligible: {0}")]
    IneligibleVoter(String),
    #[error("Duplicate vote detected")]
    DuplicateVote,
    #[error("Election not active")]
    ElectionInactive,
    #[error("Signature verification failed")]
    InvalidSignature,
    #[error("Invalid candidate selection: {0}")]
    InvalidCandidate(String),
    #[error("Ballot contains duplicate preferences")]
    DuplicatePreferences,
    #[error("Election not found: {0}")]
    ElectionNotFound(String),
    #[error("Voting period has ended")]
    VotingPeriodEnded,
    #[error("Voting period has not started")]
    VotingPeriodNotStarted,
    #[error("Invalid preference ranking: {0}")]
    InvalidRanking(String),
}

/// Unique identifier for a ballot
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BallotId(pub String);

impl std::fmt::Display for BallotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for an election
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ElectionId(pub String);

impl std::fmt::Display for ElectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a candidate
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CandidateId(pub String);

impl std::fmt::Display for CandidateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Digital signature for ballot verification
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Signature {
    pub algorithm: String,
    pub value: Vec<u8>,
}

/// Core trait for voting system implementations
pub trait VotingSystem {
    type Ballot;
    type Result;
    type Error;
    
    /// Validate a ballot's format and eligibility
    fn validate_ballot(&self, ballot: &Self::Ballot) -> Result<(), Self::Error>;
    
    /// Count votes from a collection of ballots
    fn count_votes(&self, ballots: Vec<Self::Ballot>) -> Result<Self::Result, Self::Error>;
    
    /// Check if a voter is eligible to participate
    fn is_eligible_voter(&self, voter_id: &str) -> Result<bool, Self::Error>;
}

/// Trait for validating ballots across different voting systems
pub trait BallotValidator {
    /// Validate the format and structure of a ballot
    fn validate_format(&self, ballot: &dyn Any) -> Result<(), VotingError>;
    
    /// Verify the cryptographic signature of a ballot
    fn validate_signature(&self, ballot: &dyn Any) -> Result<(), VotingError>;
    
    /// Check for duplicate votes from the same voter
    fn check_duplicate(&self, ballot: &dyn Any) -> Result<(), VotingError>;
}

/// Candidate information for elections
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Candidate {
    pub id: CandidateId,
    pub name: String,
    pub description: String,
    pub metadata: Option<Cid>, // Additional candidate information stored in DAG
}

/// Voting period specification
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VotingPeriod {
    pub start_time: SystemTime,
    pub end_time: SystemTime,
}

impl VotingPeriod {
    /// Check if the voting period is currently active
    pub fn is_active(&self) -> bool {
        let now = SystemTime::now();
        now >= self.start_time && now <= self.end_time
    }
    
    /// Check if the voting period has ended
    pub fn has_ended(&self) -> bool {
        SystemTime::now() > self.end_time
    }
    
    /// Check if the voting period has not started yet
    pub fn has_not_started(&self) -> bool {
        SystemTime::now() < self.start_time
    }
}

/// Rules for determining voter eligibility
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EligibilityRules {
    /// Required DID credentials or patterns
    pub required_credentials: Vec<String>,
    /// Minimum reputation score (if applicable)
    pub min_reputation: Option<f64>,
    /// Required federation membership
    pub required_federation: Option<String>,
    /// Custom eligibility criteria stored in DAG
    pub custom_rules: Option<Cid>,
}

/// Election configuration and metadata
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Election {
    pub election_id: ElectionId,
    pub title: String,
    pub description: String,
    pub candidates: Vec<Candidate>,
    pub voting_period: VotingPeriod,
    pub eligibility_rules: EligibilityRules,
    /// Content stored in DAG for additional election data
    pub content_cid: Option<Cid>,
    /// Creator of the election
    pub creator: Did,
    /// Election creation timestamp
    pub created_at: SystemTime,
}

/// Ranked choice ballot implementation
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RankedChoiceBallot {
    pub ballot_id: BallotId,
    pub voter_did: Did,
    pub election_id: ElectionId,
    /// Ordered list of candidate preferences (1st choice, 2nd choice, etc.)
    pub preferences: Vec<CandidateId>,
    pub timestamp: SystemTime,
    pub signature: Signature,
}

impl RankedChoiceBallot {
    /// Create a new ranked choice ballot
    pub fn new(
        ballot_id: BallotId,
        voter_did: Did,
        election_id: ElectionId,
        preferences: Vec<CandidateId>,
        signature: Signature,
    ) -> Self {
        Self {
            ballot_id,
            voter_did,
            election_id,
            preferences,
            timestamp: SystemTime::now(),
            signature,
        }
    }
    
    /// Validate that preferences don't contain duplicates
    pub fn validate_preferences(&self) -> Result<(), VotingError> {
        let mut seen = std::collections::HashSet::new();
        for candidate_id in &self.preferences {
            if !seen.insert(candidate_id) {
                return Err(VotingError::DuplicatePreferences);
            }
        }
        Ok(())
    }
    
    /// Get the voter's first choice candidate
    pub fn first_choice(&self) -> Option<&CandidateId> {
        self.preferences.first()
    }
    
    /// Get the voter's nth choice candidate (0-indexed)
    pub fn nth_choice(&self, n: usize) -> Option<&CandidateId> {
        self.preferences.get(n)
    }
}

/// Results of a ranked choice voting tally
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RankedChoiceResult {
    pub election_id: ElectionId,
    pub winner: Option<CandidateId>,
    pub rounds: Vec<RankedChoiceRound>,
    pub total_ballots: usize,
    pub exhausted_ballots: usize, // Ballots with no remaining valid preferences
}

/// Results from a single round of ranked choice voting
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RankedChoiceRound {
    pub round_number: usize,
    pub vote_counts: std::collections::HashMap<CandidateId, usize>,
    pub eliminated_candidate: Option<CandidateId>,
    pub majority_threshold: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ballot_id_display() {
        let ballot_id = BallotId("ballot-123".to_string());
        assert_eq!(ballot_id.to_string(), "ballot-123");
    }

    #[test]
    fn test_election_id_display() {
        let election_id = ElectionId("election-456".to_string());
        assert_eq!(election_id.to_string(), "election-456");
    }

    #[test]
    fn test_candidate_id_display() {
        let candidate_id = CandidateId("candidate-789".to_string());
        assert_eq!(candidate_id.to_string(), "candidate-789");
    }

    #[test]
    fn test_voting_period_active() {
        let now = SystemTime::now();
        let start = now - std::time::Duration::from_secs(60);
        let end = now + std::time::Duration::from_secs(60);
        
        let period = VotingPeriod {
            start_time: start,
            end_time: end,
        };
        
        assert!(period.is_active());
        assert!(!period.has_ended());
        assert!(!period.has_not_started());
    }

    #[test]
    fn test_voting_period_ended() {
        let now = SystemTime::now();
        let start = now - std::time::Duration::from_secs(120);
        let end = now - std::time::Duration::from_secs(60);
        
        let period = VotingPeriod {
            start_time: start,
            end_time: end,
        };
        
        assert!(!period.is_active());
        assert!(period.has_ended());
        assert!(!period.has_not_started());
    }

    #[test]
    fn test_voting_period_not_started() {
        let now = SystemTime::now();
        let start = now + std::time::Duration::from_secs(60);
        let end = now + std::time::Duration::from_secs(120);
        
        let period = VotingPeriod {
            start_time: start,
            end_time: end,
        };
        
        assert!(!period.is_active());
        assert!(!period.has_ended());
        assert!(period.has_not_started());
    }

    #[test]
    fn test_ranked_choice_ballot_preferences_validation() {
        let ballot = RankedChoiceBallot {
            ballot_id: BallotId("test-ballot".to_string()),
            voter_did: Did::default(),
            election_id: ElectionId("test-election".to_string()),
            preferences: vec![
                CandidateId("alice".to_string()),
                CandidateId("bob".to_string()),
                CandidateId("charlie".to_string()),
            ],
            timestamp: SystemTime::now(),
            signature: Signature {
                algorithm: "ed25519".to_string(),
                value: vec![0u8; 64],
            },
        };
        
        assert!(ballot.validate_preferences().is_ok());
    }

    #[test]
    fn test_ranked_choice_ballot_duplicate_preferences() {
        let ballot = RankedChoiceBallot {
            ballot_id: BallotId("test-ballot".to_string()),
            voter_did: Did::default(),
            election_id: ElectionId("test-election".to_string()),
            preferences: vec![
                CandidateId("alice".to_string()),
                CandidateId("bob".to_string()),
                CandidateId("alice".to_string()), // Duplicate
            ],
            timestamp: SystemTime::now(),
            signature: Signature {
                algorithm: "ed25519".to_string(),
                value: vec![0u8; 64],
            },
        };
        
        assert!(matches!(
            ballot.validate_preferences(),
            Err(VotingError::DuplicatePreferences)
        ));
    }

    #[test]
    fn test_ranked_choice_ballot_choices() {
        let ballot = RankedChoiceBallot {
            ballot_id: BallotId("test-ballot".to_string()),
            voter_did: Did::default(),
            election_id: ElectionId("test-election".to_string()),
            preferences: vec![
                CandidateId("alice".to_string()),
                CandidateId("bob".to_string()),
                CandidateId("charlie".to_string()),
            ],
            timestamp: SystemTime::now(),
            signature: Signature {
                algorithm: "ed25519".to_string(),
                value: vec![0u8; 64],
            },
        };
        
        assert_eq!(ballot.first_choice(), Some(&CandidateId("alice".to_string())));
        assert_eq!(ballot.nth_choice(0), Some(&CandidateId("alice".to_string())));
        assert_eq!(ballot.nth_choice(1), Some(&CandidateId("bob".to_string())));
        assert_eq!(ballot.nth_choice(2), Some(&CandidateId("charlie".to_string())));
        assert_eq!(ballot.nth_choice(3), None);
    }
}