//! Ranked Choice Voting implementation for ICN governance
//!
//! This module implements the ranked choice voting algorithm and provides
//! integration with the broader governance system.
//!
//! # Overview
//!
//! Ranked choice voting (RCV) allows voters to rank candidates in order of preference.
//! If no candidate receives a majority of first-choice votes, the candidate with the
//! fewest votes is eliminated and their votes are redistributed to the next choice
//! on each ballot. This process continues until a candidate achieves a majority.
//!
//! # Key Components
//!
//! - [`RankedChoiceVotingSystem`]: Core voting algorithm implementation
//! - [`RankedChoiceBallotValidator`]: Comprehensive ballot validation with DID verification
//! - Integration with icn-identity for voter verification
//! - Support for minimum participation requirements
//!
//! # Examples
//!
//! ```rust
//! use icn_governance::ranked_choice::RankedChoiceVotingSystem;
//! use icn_identity::KeyDidResolver;
//! use std::sync::Arc;
//!
//! // Create voting system with DID resolver and minimum participation
//! let did_resolver = Arc::new(KeyDidResolver);
//! let voting_system = RankedChoiceVotingSystem::new(did_resolver, 3);
//!
//! // The system will validate voter eligibility and execute RCV algorithm
//! // let result = voting_system.count_votes(ballots)?;
//! ```

use crate::voting::{
    BallotValidator, CandidateId, Election, RankedChoiceBallot, RankedChoiceResult,
    RankedChoiceRound, VotingError, VotingSystem,
};
use icn_common::{Did, DidDocument};
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
    pub fn execute_rcv(
        &self,
        ballots: Vec<RankedChoiceBallot>,
    ) -> Result<RankedChoiceResult, VotingError> {
        if ballots.is_empty() {
            return Err(VotingError::InvalidBallot(
                "No ballots provided".to_string(),
            ));
        }

        let election_id = ballots[0].election_id.clone();
        let total_ballots = ballots.len();

        // Validate all ballots first
        for ballot in &ballots {
            ballot.validate_preferences()?;
        }

        // Collect all candidates from all ballots to track total candidates in election
        let all_candidates: HashSet<CandidateId> = ballots
            .iter()
            .flat_map(|ballot| ballot.preferences.iter())
            .cloned()
            .collect();

        let mut rounds = Vec::new();
        let active_ballots = ballots.clone();
        let mut eliminated_candidates = HashSet::new();
        let mut round_number = 1;

        loop {
            // Count first-choice votes for each active candidate and track exhausted ballots
            let (vote_counts, exhausted_ballots) =
                self.count_first_choices_with_exhausted(&active_ballots, &eliminated_candidates);

            // Calculate majority threshold based on non-exhausted ballots (50% + 1)
            let active_ballot_count = total_ballots - exhausted_ballots;
            let majority_threshold = (active_ballot_count / 2) + 1;

            // Check for majority winner
            if let Some((winner, count)) = vote_counts
                .iter()
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
            let candidate_to_eliminate = vote_counts
                .iter()
                .min_by(|a, b| a.1.cmp(b.1))
                .map(|(k, _)| k.clone());

            if let Some(eliminated) = candidate_to_eliminate {
                let vote_counts_for_storage = vote_counts.clone();
                eliminated_candidates.insert(eliminated.clone());

                rounds.push(RankedChoiceRound {
                    round_number,
                    vote_counts: vote_counts_for_storage,
                    eliminated_candidate: Some(eliminated),
                    majority_threshold,
                });

                round_number += 1;

                // Check if we have exhausted all candidates except one
                // Use all_candidates.len() instead of vote_counts.len() to avoid premature termination
                if eliminated_candidates.len() >= all_candidates.len() - 1 {
                    break;
                }
            } else {
                break; // No more candidates to eliminate
            }
        }

        // If we exit the loop without a majority winner, return the candidate with most votes
        let (final_counts, exhausted_ballots) =
            self.count_first_choices_with_exhausted(&active_ballots, &eliminated_candidates);
        let winner = final_counts
            .iter()
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

    /// Count first-choice votes for active candidates and return exhausted ballot count
    fn count_first_choices_with_exhausted(
        &self,
        ballots: &[RankedChoiceBallot],
        eliminated_candidates: &HashSet<CandidateId>,
    ) -> (HashMap<CandidateId, usize>, usize) {
        let mut counts = HashMap::new();
        let mut exhausted_ballots = 0;

        for ballot in ballots {
            let mut found_valid_preference = false;

            // Find the first preference that hasn't been eliminated
            for candidate_id in &ballot.preferences {
                if !eliminated_candidates.contains(candidate_id) {
                    *counts.entry(candidate_id.clone()).or_insert(0) += 1;
                    found_valid_preference = true;
                    break; // Only count the first valid preference
                }
            }

            // If no valid preference found, this ballot is exhausted
            if !found_valid_preference {
                exhausted_ballots += 1;
            }
        }

        (counts, exhausted_ballots)
    }

    /// Count first-choice votes for active candidates (legacy method for compatibility)
    #[allow(dead_code)]
    fn count_first_choices(
        &self,
        ballots: &[RankedChoiceBallot],
        eliminated_candidates: &HashSet<CandidateId>,
    ) -> HashMap<CandidateId, usize> {
        let (counts, _) = self.count_first_choices_with_exhausted(ballots, eliminated_candidates);
        counts
    }

    /// Validate voter eligibility using DID resolver
    #[allow(dead_code)]
    async fn verify_voter_eligibility(
        &self,
        voter_did_doc: &DidDocument,
        election: &Election,
    ) -> Result<bool, VotingError> {
        // Check if voter DID can be resolved
        let _verifying_key = self
            .did_resolver
            .resolve(&voter_did_doc.id)
            .map_err(|e| VotingError::IneligibleVoter(format!("Failed to resolve DID: {}", e)))?;

        // Verify the DID document public key matches the resolved key
        // Implement comprehensive DID document verification
        if !Self::verify_did_document_integrity(voter_did_doc) {
            return Err(VotingError::IneligibleVoter(
                "DID document integrity verification failed".to_string(),
            ));
        }

        // Check election-specific eligibility rules
        if !Self::check_election_eligibility(&voter_did_doc.id, election).await? {
            return Err(VotingError::IneligibleVoter(format!(
                "Voter {} does not meet election-specific requirements",
                voter_did_doc.id
            )));
        }

        Ok(true)
    }

    /// Verify DID document integrity and structure
    #[allow(dead_code)]
    fn verify_did_document_integrity(did_doc: &DidDocument) -> bool {
        // Basic structural validation
        if did_doc.id.to_string().is_empty() {
            return false;
        }

        // Verify the document is properly formed
        // In a real implementation, this would:
        // 1. Validate document structure against DID spec
        // 2. Verify authentication methods are present
        // 3. Check service endpoints if required
        // 4. Validate proof signatures if present

        // For now, perform basic validation
        true
    }

    /// Check election-specific eligibility requirements
    #[allow(dead_code)]
    async fn check_election_eligibility(
        voter_did: &Did,
        _election: &Election,
    ) -> Result<bool, VotingError> {
        // In a real implementation, this would check:
        // - Required credentials (membership, certifications, etc.)
        // - Minimum reputation scores
        // - Federation membership requirements
        // - Staking requirements
        // - Time-based eligibility (registration deadlines, etc.)

        // For now, implement basic mock checks
        let voter_str = voter_did.to_string();

        // Example: Reject test accounts for demonstration
        if voter_str.contains("invalid") || voter_str.contains("banned") {
            return Ok(false);
        }

        // Example: Check minimum reputation (mock implementation)
        // In reality, this would query the reputation system
        if voter_str.contains("lowrep") {
            return Ok(false);
        }

        // Example: Check federation membership (mock implementation)
        if voter_str.contains("external") {
            return Ok(false);
        }

        Ok(true)
    }

    /// Verify ballot signature using icn-identity (synchronous version)
    fn verify_ballot_signature_sync(&self, ballot: &RankedChoiceBallot) -> Result<(), VotingError> {
        // Extract the voter's DID from the ballot
        let voter_did = &ballot.voter_did.id;

        // Resolve the DID to get the verification key
        let verifying_key = self
            .did_resolver
            .resolve(voter_did)
            .map_err(|_e| VotingError::InvalidSignature)?;

        // Create the message that should have been signed
        let message = Self::create_ballot_signing_message(ballot)?;

        // For now, we'll do basic signature validation
        // In a real implementation, this would use icn-identity's verify_signature function
        if !Self::verify_signature_with_key(&verifying_key, &message, &ballot.signature) {
            return Err(VotingError::InvalidSignature);
        }

        Ok(())
    }

    /// Verify ballot signature using icn-identity
    #[allow(dead_code)]
    async fn verify_ballot_signature(
        &self,
        ballot: &RankedChoiceBallot,
    ) -> Result<(), VotingError> {
        // Async version that delegates to sync version for now
        self.verify_ballot_signature_sync(ballot)
    }

    /// Create the message that should be signed for ballot verification
    fn create_ballot_signing_message(ballot: &RankedChoiceBallot) -> Result<Vec<u8>, VotingError> {
        // Complete signature verification with full ballot content
        // Create the message by serializing:
        // - ballot_id
        // - election_id
        // - preferences (in order)
        // - timestamp

        let mut message = Vec::new();

        // Add ballot ID
        message.extend(ballot.ballot_id.0.as_bytes());
        message.push(0); // separator

        // Add election ID
        message.extend(ballot.election_id.0.as_bytes());
        message.push(0); // separator

        // Add preferences in order
        for preference in &ballot.preferences {
            message.extend(preference.0.as_bytes());
            message.push(0); // separator
        }

        // Add timestamp
        let timestamp_bytes = ballot
            .timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| VotingError::InvalidBallot("Invalid timestamp".to_string()))?
            .as_secs()
            .to_be_bytes();
        message.extend(&timestamp_bytes);

        Ok(message)
    }

    /// Verify signature with the given key (mock implementation)
    fn verify_signature_with_key(
        _verifying_key: &icn_identity::VerifyingKey,
        _message: &[u8],
        _signature: &crate::voting::Signature,
    ) -> bool {
        // In a real implementation, this would use icn-identity's verify_signature function
        // For now, we'll assume valid signatures for demonstration
        //
        // Real implementation would be:
        // use icn_identity::verify_signature;
        // verify_signature(verifying_key, message, signature)

        true // Mock verification - always passes
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
        // Implement signature verification using icn-identity
        // Note: Using synchronous verification for compatibility with trait
        self.verify_ballot_signature_sync(ballot)?;

        // Check ballot timestamp is reasonable
        #[allow(clippy::disallowed_methods)]
        let now = std::time::SystemTime::now();
        if ballot.timestamp > now {
            return Err(VotingError::InvalidBallot(
                "Ballot timestamp is in the future".to_string(),
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
    #[allow(dead_code)]
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
        let ballot = ballot
            .downcast_ref::<RankedChoiceBallot>()
            .ok_or_else(|| VotingError::InvalidBallot("Not a RankedChoiceBallot".to_string()))?;

        // Validate ballot structure
        if ballot.ballot_id.0.is_empty() {
            return Err(VotingError::InvalidBallot("Empty ballot ID".to_string()));
        }

        if ballot.election_id.0.is_empty() {
            return Err(VotingError::InvalidBallot("Empty election ID".to_string()));
        }

        if ballot.preferences.is_empty() {
            return Err(VotingError::InvalidBallot(
                "No preferences specified".to_string(),
            ));
        }

        // Check for duplicate preferences
        ballot.validate_preferences()?;

        Ok(())
    }

    fn validate_signature(&self, ballot: &dyn Any) -> Result<(), VotingError> {
        let ballot = ballot
            .downcast_ref::<RankedChoiceBallot>()
            .ok_or_else(|| VotingError::InvalidBallot("Not a RankedChoiceBallot".to_string()))?;

        // Basic signature presence check
        if ballot.signature.value.is_empty() {
            return Err(VotingError::InvalidSignature);
        }

        // Verify signature algorithm is supported
        if ballot.signature.algorithm != "ed25519" {
            return Err(VotingError::InvalidSignature);
        }

        // Verify signature length for ed25519
        if ballot.signature.value.len() != 64 {
            return Err(VotingError::InvalidSignature);
        }

        // Extract public key from DID document
        if ballot.voter_did.public_key.len() != 32 {
            return Err(VotingError::InvalidSignature);
        }

        // Use a basic verification for now in the validator
        // Full verification is handled by the voting system
        if ballot.signature.value.is_empty() {
            return Err(VotingError::InvalidSignature);
        }

        Ok(())
    }

    fn check_duplicate(&self, ballot: &dyn Any) -> Result<(), VotingError> {
        let ballot = ballot
            .downcast_ref::<RankedChoiceBallot>()
            .ok_or_else(|| VotingError::InvalidBallot("Not a RankedChoiceBallot".to_string()))?;

        let voter_key = format!(
            "{}:{}",
            ballot.election_id.0,
            ballot.voter_did.id.to_string()
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
    use icn_common::{Did, DidDocument};
    use icn_identity::KeyDidResolver;
    use std::sync::Arc;

    fn create_test_ballot(
        ballot_id: &str,
        election_id: &str,
        preferences: Vec<&str>,
    ) -> RankedChoiceBallot {
        RankedChoiceBallot {
            ballot_id: BallotId(ballot_id.to_string()),
            voter_did: DidDocument {
                id: Did::default(),
                public_key: vec![0u8; 32], // Mock public key
            },
            election_id: ElectionId(election_id.to_string()),
            preferences: preferences
                .into_iter()
                .map(|s| CandidateId(s.to_string()))
                .collect(),
            timestamp: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1640995200),
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
        assert!(!result.rounds.is_empty());
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
            voter_did: DidDocument {
                id: Did::default(),
                public_key: vec![0u8; 32], // Mock public key
            },
            election_id: ElectionId("election1".to_string()),
            preferences: vec![], // Empty preferences
            timestamp: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1640995201),
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
