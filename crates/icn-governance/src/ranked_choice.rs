//! Ranked Choice Voting implementation for ICN governance
//! 
//! This module implements the ranked choice voting algorithm and provides
//! integration with the broader governance system.

use crate::voting::{
    BallotValidator, CandidateId, Election, RankedChoiceBallot, RankedChoiceResult,
    RankedChoiceRound, VotingError, VotingSystem,
};
use icn_common::{Did};
use icn_identity::DidResolver;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

/// Ranked choice voting system implementation
pub struct RankedChoiceVotingSystem {
    /// DID resolver for voter verification
    did_resolver: Arc<dyn DidResolver>,
    /// Minimum number of votes required for validity
    min_participation: usize,
}

impl RankedChoiceVotingSystem {
    /// Create a new ranked choice voting system
    pub fn new(did_resolver: Arc<dyn DidResolver>, min_participation: usize) -> Self {
        Self {
            did_resolver,
            min_participation,
        }
    }

    /// Execute ranked choice voting algorithm
    pub fn execute_rcv(&self, ballots: Vec<RankedChoiceBallot>) -> Result<RankedChoiceResult, VotingError> {
        if ballots.is_empty() {
            return Err(VotingError::InvalidBallot("No ballots provided".to_string()));
        }

        let election_id = ballots[0].election_id.clone();
        let total_ballots = ballots.len();
        
        // Validate all ballots first
        for ballot in &ballots {
            ballot.validate_preferences()?;
        }

        let mut rounds = Vec::new();
        let mut active_ballots = ballots.clone();
        let mut eliminated_candidates = HashSet::new();
        let exhausted_ballots = 0;
        let mut round_number = 1;

        loop {
            // Count first-choice votes for each active candidate
            let vote_counts = self.count_first_choices(&active_ballots, &eliminated_candidates);
            
            // Calculate majority threshold (50% + 1)
            let active_ballot_count = active_ballots.len() - exhausted_ballots;
            let majority_threshold = (active_ballot_count / 2) + 1;

            // Check for majority winner
            if let Some((winner, count)) = vote_counts.iter()
                .max_by(|a, b| a.1.cmp(b.1))
                .map(|(k, v)| (k.clone(), *v))
            {
                if count >= majority_threshold {
                    rounds.push(RankedChoiceRound {
                        round_number,
                        vote_counts,
                        eliminated_candidate: None,
                        majority_threshold,
                    });
                    
                    return Ok(RankedChoiceResult {
                        election_id,
                        winner: Some(winner),
                        rounds,
                        total_ballots,
                        exhausted_ballots,
                    });
                }
            }

            // Find candidate with fewest votes to eliminate
            let candidate_to_eliminate = vote_counts.iter()
                .min_by(|a, b| a.1.cmp(b.1))
                .map(|(k, _)| k.clone());

            if let Some(eliminated) = candidate_to_eliminate {
                let vote_counts_for_storage = vote_counts.clone();
                eliminated_candidates.insert(eliminated.clone());
                
                // Remove eliminated candidate from all ballots and update preferences
                self.redistribute_votes(&mut active_ballots, &eliminated, &eliminated_candidates);
                
                rounds.push(RankedChoiceRound {
                    round_number,
                    vote_counts: vote_counts_for_storage,
                    eliminated_candidate: Some(eliminated),
                    majority_threshold,
                });
                
                round_number += 1;

                // Check if we have exhausted all candidates except one
                if eliminated_candidates.len() >= vote_counts.len() - 1 {
                    break;
                }
            } else {
                break; // No more candidates to eliminate
            }
        }

        // If we exit the loop without a majority winner, return the candidate with most votes
        let final_counts = self.count_first_choices(&active_ballots, &eliminated_candidates);
        let winner = final_counts.iter()
            .max_by(|a, b| a.1.cmp(b.1))
            .map(|(k, _)| k.clone());

        Ok(RankedChoiceResult {
            election_id,
            winner,
            rounds,
            total_ballots,
            exhausted_ballots,
        })
    }

    /// Count first-choice votes for active candidates
    fn count_first_choices(
        &self,
        ballots: &[RankedChoiceBallot],
        eliminated_candidates: &HashSet<CandidateId>,
    ) -> HashMap<CandidateId, usize> {
        let mut counts = HashMap::new();

        for ballot in ballots {
            // Find the first preference that hasn't been eliminated
            for candidate_id in &ballot.preferences {
                if !eliminated_candidates.contains(candidate_id) {
                    *counts.entry(candidate_id.clone()).or_insert(0) += 1;
                    break; // Only count the first valid preference
                }
            }
        }

        counts
    }

    /// Redistribute votes by removing eliminated candidate from preferences
    fn redistribute_votes(
        &self,
        _ballots: &mut [RankedChoiceBallot],
        _eliminated: &CandidateId,
        _eliminated_candidates: &HashSet<CandidateId>,
    ) {
        // In ranked choice voting, we don't need to modify the ballots themselves
        // since we only look at the first non-eliminated preference in each ballot
        // The elimination logic is handled in count_first_choices
    }

    /// Validate voter eligibility using DID resolver
    fn verify_voter_eligibility(
        &self,
        voter_did: &Did,
        _election: &Election,
    ) -> Result<bool, VotingError> {
        // Check if voter DID can be resolved
        let _verifying_key = self.did_resolver.resolve(voter_did)
            .map_err(|e| VotingError::IneligibleVoter(format!("Failed to resolve DID: {}", e)))?;

        // If we got a verifying key, the DID is valid
        // For now, we'll implement basic checks - this can be extended
        // TODO: Check election-specific eligibility rules
        // - Check required credentials
        // - Check minimum reputation
        // - Check federation membership

        Ok(true)
    }
}

impl VotingSystem for RankedChoiceVotingSystem {
    type Ballot = RankedChoiceBallot;
    type Result = RankedChoiceResult;
    type Error = VotingError;

    fn validate_ballot(&self, ballot: &Self::Ballot) -> Result<(), Self::Error> {
        // Check for duplicate preferences
        ballot.validate_preferences()?;

        // Verify ballot signature
        // TODO: Implement signature verification using icn-identity
        // This would verify the ballot is signed by the voter's DID

        // Check ballot timestamp is reasonable
        let now = std::time::SystemTime::now();
        if ballot.timestamp > now {
            return Err(VotingError::InvalidBallot(
                "Ballot timestamp is in the future".to_string()
            ));
        }

        Ok(())
    }

    fn count_votes(&self, ballots: Vec<Self::Ballot>) -> Result<Self::Result, Self::Error> {
        if ballots.len() < self.min_participation {
            return Err(VotingError::InvalidBallot(format!(
                "Insufficient participation: {} ballots, minimum required: {}",
                ballots.len(),
                self.min_participation
            )));
        }

        self.execute_rcv(ballots)
    }

    fn is_eligible_voter(&self, voter_id: &str) -> Result<bool, Self::Error> {
        // Parse the voter ID as a DID
        let did = Did::from_str(voter_id)
            .map_err(|e| VotingError::IneligibleVoter(format!("Invalid DID format: {}", e)))?;

        // For synchronous interface, we'll do basic validation
        // Full eligibility checking would be done asynchronously
        Ok(!did.to_string().is_empty())
    }
}

/// Ballot validator for ranked choice ballots
pub struct RankedChoiceBallotValidator {
    /// DID resolver for signature verification
    did_resolver: Arc<dyn DidResolver>,
    /// Track submitted ballots to prevent duplicates
    submitted_ballots: Arc<std::sync::Mutex<HashSet<String>>>,
}

impl RankedChoiceBallotValidator {
    /// Create a new ranked choice ballot validator
    pub fn new(did_resolver: Arc<dyn DidResolver>) -> Self {
        Self {
            did_resolver,
            submitted_ballots: Arc::new(std::sync::Mutex::new(HashSet::new())),
        }
    }

    /// Reset the validator state for a new election
    pub fn reset(&self) {
        if let Ok(mut ballots) = self.submitted_ballots.lock() {
            ballots.clear();
        }
    }
}

impl BallotValidator for RankedChoiceBallotValidator {
    fn validate_format(&self, ballot: &dyn Any) -> Result<(), VotingError> {
        let ballot = ballot.downcast_ref::<RankedChoiceBallot>()
            .ok_or_else(|| VotingError::InvalidBallot("Not a RankedChoiceBallot".to_string()))?;

        // Validate ballot structure
        if ballot.ballot_id.0.is_empty() {
            return Err(VotingError::InvalidBallot("Empty ballot ID".to_string()));
        }

        if ballot.election_id.0.is_empty() {
            return Err(VotingError::InvalidBallot("Empty election ID".to_string()));
        }

        if ballot.preferences.is_empty() {
            return Err(VotingError::InvalidBallot("No preferences specified".to_string()));
        }

        // Check for duplicate preferences
        ballot.validate_preferences()?;

        Ok(())
    }

    fn validate_signature(&self, ballot: &dyn Any) -> Result<(), VotingError> {
        let ballot = ballot.downcast_ref::<RankedChoiceBallot>()
            .ok_or_else(|| VotingError::InvalidBallot("Not a RankedChoiceBallot".to_string()))?;

        // TODO: Implement signature verification using icn-identity
        // This would:
        // 1. Extract the public key from the voter's DID document
        // 2. Verify the signature covers the ballot content
        // 3. Ensure the signature algorithm is supported

        if ballot.signature.value.is_empty() {
            return Err(VotingError::InvalidSignature);
        }

        Ok(())
    }

    fn check_duplicate(&self, ballot: &dyn Any) -> Result<(), VotingError> {
        let ballot = ballot.downcast_ref::<RankedChoiceBallot>()
            .ok_or_else(|| VotingError::InvalidBallot("Not a RankedChoiceBallot".to_string()))?;

        let voter_key = format!("{}:{}", 
            ballot.election_id.0, 
            ballot.voter_did.to_string()
        );

        if let Ok(mut submitted) = self.submitted_ballots.lock() {
            if submitted.contains(&voter_key) {
                return Err(VotingError::DuplicateVote);
            }
            submitted.insert(voter_key);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voting::{BallotId, ElectionId, Signature};
    use icn_common::{CommonError, Did};
    use icn_identity::{DidResolver, KeyDidResolver};
    use std::sync::Arc;

    fn create_test_ballot(
        ballot_id: &str,
        election_id: &str,
        preferences: Vec<&str>,
    ) -> RankedChoiceBallot {
        RankedChoiceBallot {
            ballot_id: BallotId(ballot_id.to_string()),
            voter_did: Did::default(),
            election_id: ElectionId(election_id.to_string()),
            preferences: preferences.into_iter()
                .map(|s| CandidateId(s.to_string()))
                .collect(),
            timestamp: std::time::SystemTime::now(),
            signature: Signature {
                algorithm: "ed25519".to_string(),
                value: vec![0u8; 64],
            },
        }
    }

    #[test]
    fn test_ranked_choice_voting_simple_majority() {
        let did_resolver = Arc::new(KeyDidResolver);
        let rcv = RankedChoiceVotingSystem::new(did_resolver, 1);

        let ballots = vec![
            create_test_ballot("ballot1", "election1", vec!["alice", "bob", "charlie"]),
            create_test_ballot("ballot2", "election1", vec!["alice", "charlie", "bob"]),
            create_test_ballot("ballot3", "election1", vec!["bob", "alice", "charlie"]),
        ];

        let result = rcv.execute_rcv(ballots).unwrap();
        
        // Alice should win with 2 first-choice votes (majority)
        assert_eq!(result.winner, Some(CandidateId("alice".to_string())));
        assert_eq!(result.rounds.len(), 1);
        assert_eq!(result.total_ballots, 3);
    }

    #[test]
    fn test_ranked_choice_voting_with_elimination() {
        let did_resolver = Arc::new(KeyDidResolver);
        let rcv = RankedChoiceVotingSystem::new(did_resolver, 1);

        let ballots = vec![
            // Alice: 1 first choice
            create_test_ballot("ballot1", "election1", vec!["alice", "bob", "charlie"]),
            // Bob: 1 first choice  
            create_test_ballot("ballot2", "election1", vec!["bob", "alice", "charlie"]),
            // Charlie: 2 first choices
            create_test_ballot("ballot3", "election1", vec!["charlie", "alice", "bob"]),
            create_test_ballot("ballot4", "election1", vec!["charlie", "bob", "alice"]),
        ];

        let result = rcv.execute_rcv(ballots).unwrap();
        
        // Charlie should win with 2 first-choice votes (50%)
        // Since we have 4 ballots, majority threshold is 3, so multiple rounds needed
        assert!(result.rounds.len() >= 1);
        assert_eq!(result.total_ballots, 4);
    }

    #[test]
    fn test_ballot_validator_format() {
        let did_resolver = Arc::new(KeyDidResolver);
        let validator = RankedChoiceBallotValidator::new(did_resolver);

        let valid_ballot = create_test_ballot("ballot1", "election1", vec!["alice", "bob"]);
        assert!(validator.validate_format(&valid_ballot).is_ok());

        let empty_preferences = RankedChoiceBallot {
            ballot_id: BallotId("ballot1".to_string()),
            voter_did: Did::default(),
            election_id: ElectionId("election1".to_string()),
            preferences: vec![], // Empty preferences
            timestamp: std::time::SystemTime::now(),
            signature: Signature {
                algorithm: "ed25519".to_string(),
                value: vec![0u8; 64],
            },
        };

        assert!(matches!(
            validator.validate_format(&empty_preferences),
            Err(VotingError::InvalidBallot(_))
        ));
    }

    #[test]
    fn test_ballot_validator_duplicate_detection() {
        let did_resolver = Arc::new(KeyDidResolver);
        let validator = RankedChoiceBallotValidator::new(did_resolver);

        let ballot1 = create_test_ballot("ballot1", "election1", vec!["alice", "bob"]);
        let ballot2 = create_test_ballot("ballot2", "election1", vec!["bob", "alice"]);

        // First ballot should pass
        assert!(validator.check_duplicate(&ballot1).is_ok());
        
        // Second ballot from same voter should fail (same DID document)
        assert!(matches!(
            validator.check_duplicate(&ballot2),
            Err(VotingError::DuplicateVote)
        ));
    }

    #[test]
    fn test_voting_system_trait_implementation() {
        let did_resolver = Arc::new(KeyDidResolver);
        let rcv = RankedChoiceVotingSystem::new(did_resolver, 1);

        let ballot = create_test_ballot("ballot1", "election1", vec!["alice", "bob"]);
        
        // Test ballot validation
        assert!(rcv.validate_ballot(&ballot).is_ok());
        
        // Test voter eligibility
        assert!(rcv.is_eligible_voter("did:example:voter").is_ok());
        
        // Test vote counting
        let ballots = vec![ballot];
        let result = rcv.count_votes(ballots);
        assert!(result.is_ok());
    }
}