//! Core voting primitives and traits for ICN governance
//!
//! This module provides the foundational types and traits for implementing
//! sophisticated voting mechanisms including ranked choice voting.
//!
//! # Key Components
//!
//! ## Core Traits
//! - [`VotingSystem`]: Generic interface for voting algorithm implementations
//! - [`BallotValidator`]: Comprehensive ballot validation (format, signatures, duplicates)
//!
//! ## Data Structures  
//! - [`RankedChoiceBallot`]: Cryptographically-signed ballots with preference ordering
//! - [`Election`]: Complete election configuration with candidates and eligibility rules
//! - [`EligibilityRules`]: Configurable voter eligibility requirements
//!
//! ## DAG Integration
//! - [`BallotAnchoringService`]: Permanent ballot storage using content-addressed DAG
//!
//! # Examples
//!
//! ```rust
//! use icn_governance::voting::{RankedChoiceBallot, BallotId, CandidateId, EligibilityRules, ElectionId, Signature};
//! use icn_common::{Did, DidDocument};
//! use std::time::SystemTime;
//!
//! // Create voter eligibility rules
//! let rules = EligibilityRules::federation_members_only("my-federation".to_string());
//! assert!(rules.has_restrictions());
//!
//! // Create a ballot with ranked preferences
//! let ballot = RankedChoiceBallot {
//!     ballot_id: BallotId("ballot-001".to_string()),
//!     voter_did: DidDocument {
//!         id: Did::default(),
//!         public_key: vec![0u8; 32]
//!     },
//!     election_id: ElectionId("election-123".to_string()),
//!     preferences: vec![
//!         CandidateId("alice".to_string()),
//!         CandidateId("bob".to_string()),
//!     ],
//!     timestamp: SystemTime::now(),
//!     signature: Signature {
//!         algorithm: "ed25519".to_string(),
//!         value: vec![0u8; 64],
//!     },
//! };
//!
//! // Validate ballot preferences
//! assert!(ballot.validate_preferences().is_ok());
//! assert_eq!(ballot.first_choice(), Some(&CandidateId("alice".to_string())));
//! ```

use icn_common::{Cid, Did, DidDocument, Signable};
use icn_dag::StorageService;
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

/// Trait for federation membership registry
pub trait FederationRegistry {
    /// Check if a DID is a member of a specific federation
    fn is_member(&self, did: &Did, federation_id: &str) -> Result<bool, VotingError>;

    /// Get all federations that a DID is a member of
    fn get_memberships(&self, did: &Did) -> Result<Vec<String>, VotingError>;

    /// Add a DID to a federation (admin operation)
    fn add_member(&mut self, did: &Did, federation_id: &str) -> Result<(), VotingError>;

    /// Remove a DID from a federation (admin operation)  
    fn remove_member(&mut self, did: &Did, federation_id: &str) -> Result<(), VotingError>;
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

impl EligibilityRules {
    /// Create basic eligibility rules that accept any valid DID
    pub fn open_to_all() -> Self {
        Self {
            required_credentials: vec![],
            min_reputation: None,
            required_federation: None,
            custom_rules: None,
        }
    }

    /// Create eligibility rules requiring federation membership
    pub fn federation_members_only(federation_id: String) -> Self {
        Self {
            required_credentials: vec![],
            min_reputation: None,
            required_federation: Some(federation_id),
            custom_rules: None,
        }
    }

    /// Create eligibility rules requiring minimum reputation
    pub fn reputation_gated(min_score: f64) -> Self {
        Self {
            required_credentials: vec![],
            min_reputation: Some(min_score),
            required_federation: None,
            custom_rules: None,
        }
    }

    /// Check if the rules have any restrictions
    pub fn has_restrictions(&self) -> bool {
        !self.required_credentials.is_empty()
            || self.min_reputation.is_some()
            || self.required_federation.is_some()
            || self.custom_rules.is_some()
    }

    /// Validate a voter's DID document against these eligibility rules
    pub fn validate_voter(&self, voter_did_doc: &DidDocument) -> Result<bool, VotingError> {
        self.validate_voter_with_context(voter_did_doc, None, None, None)
    }

    /// Validate a voter's DID document against these eligibility rules with optional context
    pub fn validate_voter_with_context(
        &self,
        voter_did_doc: &DidDocument,
        reputation_store: Option<&dyn icn_reputation::ReputationStore>,
        dag_storage: Option<&dyn icn_dag::StorageService<icn_common::DagBlock>>,
        federation_registry: Option<&dyn FederationRegistry>,
    ) -> Result<bool, VotingError> {
        // Check required credentials (basic implementation)
        if !self.required_credentials.is_empty() {
            // Basic credential verification - check if the DID contains required credential patterns
            let did_str = voter_did_doc.id.to_string();
            for required_cred in &self.required_credentials {
                if !did_str.contains(required_cred) {
                    return Err(VotingError::IneligibleVoter(format!(
                        "Required credential '{}' not found in DID",
                        required_cred
                    )));
                }
            }
        }

        // Check minimum reputation (with reputation system integration)
        if let Some(min_rep) = self.min_reputation {
            if let Some(rep_store) = reputation_store {
                let voter_reputation = rep_store.get_reputation(&voter_did_doc.id);
                if (voter_reputation as f64) < min_rep {
                    return Err(VotingError::IneligibleVoter(format!(
                        "Voter reputation {} below minimum required {}",
                        voter_reputation, min_rep
                    )));
                }
            } else {
                // If no reputation store provided, we can't verify - this might be acceptable
                // in some contexts but we should warn
                log::warn!("Reputation requirement specified but no reputation store provided for validation");
                return Err(VotingError::IneligibleVoter(format!(
                    "Cannot verify minimum reputation {} - no reputation store available",
                    min_rep
                )));
            }
        }

        // Check federation membership
        if let Some(ref federation_id) = self.required_federation {
            if let Some(fed_registry) = federation_registry {
                if !fed_registry.is_member(&voter_did_doc.id, federation_id)? {
                    return Err(VotingError::IneligibleVoter(format!(
                        "Voter is not a member of required federation '{}'",
                        federation_id
                    )));
                }
            } else {
                // Basic federation membership check - for now, check if the DID contains the federation ID
                let did_str = voter_did_doc.id.to_string();
                if !did_str.contains(federation_id) {
                    return Err(VotingError::IneligibleVoter(format!(
                        "DID does not indicate membership in federation '{}'",
                        federation_id
                    )));
                }
            }
        }

        // Check custom rules stored in DAG
        if let Some(ref custom_rules_cid) = self.custom_rules {
            if let Some(storage) = dag_storage {
                match storage.get(custom_rules_cid) {
                    Ok(Some(rule_block)) => {
                        // Deserialize and evaluate custom rules
                        match self.evaluate_custom_rules(&rule_block.data, voter_did_doc) {
                            Ok(eligible) => {
                                if !eligible {
                                    return Err(VotingError::IneligibleVoter(
                                        "Voter does not meet custom eligibility rules".to_string(),
                                    ));
                                }
                            }
                            Err(e) => {
                                return Err(VotingError::IneligibleVoter(format!(
                                    "Error evaluating custom rules: {}",
                                    e
                                )));
                            }
                        }
                    }
                    Ok(None) => {
                        return Err(VotingError::IneligibleVoter(
                            "Custom rules not found in DAG storage".to_string(),
                        ));
                    }
                    Err(e) => {
                        return Err(VotingError::IneligibleVoter(format!(
                            "Error retrieving custom rules from DAG: {}",
                            e
                        )));
                    }
                }
            } else {
                return Err(VotingError::IneligibleVoter(
                    "Custom rules specified but no DAG storage provided for validation".to_string(),
                ));
            }
        }

        // If all checks pass, voter is eligible
        Ok(true)
    }

    /// Evaluate custom eligibility rules (basic JSON-based implementation)
    fn evaluate_custom_rules(
        &self,
        rule_data: &[u8],
        voter_did_doc: &DidDocument,
    ) -> Result<bool, String> {
        // Parse custom rules from JSON
        let rules: serde_json::Value = serde_json::from_slice(rule_data)
            .map_err(|e| format!("Invalid custom rules JSON: {}", e))?;

        // Basic rule evaluation - this can be extended with more sophisticated rule engines
        if let Some(required_did_patterns) = rules
            .get("required_did_patterns")
            .and_then(|v| v.as_array())
        {
            let did_str = voter_did_doc.id.to_string();
            for pattern in required_did_patterns {
                if let Some(pattern_str) = pattern.as_str() {
                    if !did_str.contains(pattern_str) {
                        return Ok(false);
                    }
                }
            }
        }

        if let Some(required_key_length) =
            rules.get("min_public_key_length").and_then(|v| v.as_u64())
        {
            if voter_did_doc.public_key.len() < required_key_length as usize {
                return Ok(false);
            }
        }

        // All custom rules passed
        Ok(true)
    }
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
    pub voter_did: DidDocument,
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
        voter_did: DidDocument,
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

impl Signable for RankedChoiceBallot {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, icn_common::CommonError> {
        let mut bytes = Vec::new();

        // Include ballot_id with separator
        bytes.extend_from_slice(self.ballot_id.0.as_bytes());
        bytes.push(b'\0'); // Separator to prevent collision

        // Include voter DID (from DID document) with separator
        bytes.extend_from_slice(self.voter_did.id.to_string().as_bytes());
        bytes.push(b'\0'); // Separator to prevent collision

        // Include election_id with separator
        bytes.extend_from_slice(self.election_id.0.as_bytes());
        bytes.push(b'\0'); // Separator to prevent collision

        // Include preferences in order (critical for ranked choice integrity)
        for preference in &self.preferences {
            bytes.extend_from_slice(preference.0.as_bytes());
            bytes.push(b'\0'); // Separator to prevent ambiguity
        }

        // Include timestamp for replay protection - use full u128 to prevent overflow
        let timestamp_nanos = self
            .timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        bytes.extend_from_slice(&timestamp_nanos.to_le_bytes());

        Ok(bytes)
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

/// Service for anchoring ballots in the DAG for permanent storage and verification
///
/// This service provides permanent, content-addressed storage for voting ballots
/// using the ICN DAG infrastructure. All ballots are cryptographically anchored
/// and can be retrieved and verified at any time.
///
/// # Features
/// - Permanent ballot storage with content addressing
/// - Ballot integrity verification through DAG structure
/// - Election result aggregation through ballot linking
/// - Support for any DAG storage backend implementing `StorageService`
///
/// # Examples
///
/// ```rust
/// use icn_governance::voting::BallotAnchoringService;
/// use icn_dag::InMemoryDagStore;
///
/// let storage = InMemoryDagStore::new();
/// let mut anchoring_service = BallotAnchoringService::new(storage);
///
/// // Anchor a ballot and get its CID
/// // let ballot_cid = anchoring_service.anchor_ballot(&ballot)?;
///
/// // Retrieve the ballot later
/// // let retrieved_ballot = anchoring_service.retrieve_ballot(&ballot_cid)?;
/// ```
pub struct BallotAnchoringService<S>
where
    S: StorageService<icn_common::DagBlock>,
{
    storage: S,
}

impl<S> BallotAnchoringService<S>
where
    S: StorageService<icn_common::DagBlock>,
{
    /// Create a new ballot anchoring service with the provided storage backend
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Anchor a ballot in the DAG for permanent storage
    ///
    /// This method stores a ballot in the content-addressed DAG, making it
    /// permanently available and verifiable. The ballot is serialized and
    /// wrapped in a DAG block with appropriate metadata.
    ///
    /// # Arguments
    /// * `ballot` - The ranked choice ballot to anchor
    ///
    /// # Returns
    /// * `Ok(Cid)` - The content identifier for the anchored ballot
    /// * `Err(VotingError)` - If serialization or storage fails
    ///
    /// # Examples
    /// ```rust
    /// # use icn_governance::voting::{BallotAnchoringService, RankedChoiceBallot};
    /// # use icn_dag::InMemoryDagStore;
    /// # let storage = InMemoryDagStore::new();
    /// # let mut service = BallotAnchoringService::new(storage);
    /// # let ballot = RankedChoiceBallot::new(
    /// #     BallotId("ballot-001".to_string()),
    /// #     DidDocument {
    /// #         id: Did::default(),
    /// #         public_key: vec![0u8; 32]
    /// #     },
    /// #     ElectionId("election-123".to_string()),
    /// #     vec![
    /// #         CandidateId("alice".to_string()),
    /// #         CandidateId("bob".to_string()),
    /// #     ],
    /// #     Signature {
    /// #         algorithm: "ed25519".to_string(),
    /// #         value: vec![0u8; 64],
    /// #     },
    /// # ); // Create ballot
    /// let cid = service.anchor_ballot(&ballot)?;
    /// println!("Ballot anchored with CID: {}", cid);
    /// # Ok::<(), icn_governance::VotingError>(())
    /// ```
    pub fn anchor_ballot(&mut self, ballot: &RankedChoiceBallot) -> Result<Cid, VotingError> {
        use icn_common::DagBlock;

        // Serialize the ballot for storage
        let ballot_data = serde_json::to_vec(ballot).map_err(|e| {
            VotingError::InvalidBallot(format!("Failed to serialize ballot: {}", e))
        })?;

        // Create a DAG block for the ballot
        let block = DagBlock {
            cid: Cid::new_v1_sha256(0x71, &ballot_data), // Raw codec for ballot data
            data: ballot_data,
            links: vec![], // Ballots are leaf nodes initially
            timestamp: ballot
                .timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            author_did: ballot.voter_did.id.clone(),
            signature: None, // Ballot has its own signature
            scope: None,
        };

        // Store the block in the DAG
        self.storage.put(&block).map_err(|e| {
            VotingError::InvalidBallot(format!("Failed to store ballot in DAG: {}", e))
        })?;

        Ok(block.cid)
    }

    /// Retrieve a ballot from the DAG by its CID
    pub fn retrieve_ballot(&self, cid: &Cid) -> Result<Option<RankedChoiceBallot>, VotingError> {
        let block = self.storage.get(cid).map_err(|e| {
            VotingError::InvalidBallot(format!("Failed to retrieve from DAG: {}", e))
        })?;

        match block {
            Some(dag_block) => {
                let ballot: RankedChoiceBallot =
                    serde_json::from_slice(&dag_block.data).map_err(|e| {
                        VotingError::InvalidBallot(format!("Failed to deserialize ballot: {}", e))
                    })?;
                Ok(Some(ballot))
            }
            None => Ok(None),
        }
    }

    /// Create a link between ballots (e.g., for election result aggregation)
    pub fn link_ballots(
        &mut self,
        election_id: &ElectionId,
        ballot_cids: Vec<Cid>,
    ) -> Result<Cid, VotingError> {
        use icn_common::{DagBlock, DagLink};

        // Create links to all ballots
        let links: Vec<DagLink> = ballot_cids
            .into_iter()
            .map(|cid| DagLink {
                cid,
                name: "ballot".to_string(),
                size: 0, // Size calculation could be improved
            })
            .collect();

        // Create metadata for the election result
        let election_data = serde_json::to_vec(&election_id).map_err(|e| {
            VotingError::InvalidBallot(format!("Failed to serialize election ID: {}", e))
        })?;

        let result_block = DagBlock {
            cid: Cid::new_v1_sha256(0x71, &election_data),
            data: election_data,
            links,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            author_did: Did::new("system", "governance"), // System-generated block
            signature: None,
            scope: None,
        };

        self.storage.put(&result_block).map_err(|e| {
            VotingError::InvalidBallot(format!("Failed to store election result: {}", e))
        })?;

        Ok(result_block.cid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_did_document() -> DidDocument {
        DidDocument {
            id: Did::default(),
            public_key: vec![0u8; 32], // Mock public key
        }
    }

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
            voter_did: create_test_did_document(),
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
            voter_did: create_test_did_document(),
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
            voter_did: create_test_did_document(),
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

        assert_eq!(
            ballot.first_choice(),
            Some(&CandidateId("alice".to_string()))
        );
        assert_eq!(
            ballot.nth_choice(0),
            Some(&CandidateId("alice".to_string()))
        );
        assert_eq!(ballot.nth_choice(1), Some(&CandidateId("bob".to_string())));
        assert_eq!(
            ballot.nth_choice(2),
            Some(&CandidateId("charlie".to_string()))
        );
        assert_eq!(ballot.nth_choice(3), None);
    }

    #[test]
    fn test_ballot_signable_implementation() {
        let ballot = RankedChoiceBallot {
            ballot_id: BallotId("test-ballot".to_string()),
            voter_did: create_test_did_document(),
            election_id: ElectionId("test-election".to_string()),
            preferences: vec![
                CandidateId("alice".to_string()),
                CandidateId("bob".to_string()),
            ],
            timestamp: SystemTime::now(),
            signature: Signature {
                algorithm: "ed25519".to_string(),
                value: vec![0u8; 64],
            },
        };

        // Test that ballot can be converted to signable bytes
        let signable_bytes = ballot.to_signable_bytes();
        assert!(signable_bytes.is_ok());

        let bytes = signable_bytes.unwrap();
        assert!(!bytes.is_empty());

        // Verify that different ballots produce different signatures
        let mut ballot2 = ballot.clone();
        ballot2.preferences = vec![
            CandidateId("bob".to_string()),
            CandidateId("alice".to_string()),
        ];

        let bytes2 = ballot2.to_signable_bytes().unwrap();
        assert_ne!(
            bytes, bytes2,
            "Different preference orders should produce different signable bytes"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_ballot_anchoring_service_integration() {
        use icn_dag::InMemoryDagStore;

        let storage = InMemoryDagStore::new();
        let mut anchoring_service = BallotAnchoringService::new(storage);

        let ballot = RankedChoiceBallot {
            ballot_id: BallotId("test-ballot".to_string()),
            voter_did: create_test_did_document(),
            election_id: ElectionId("test-election".to_string()),
            preferences: vec![
                CandidateId("alice".to_string()),
                CandidateId("bob".to_string()),
            ],
            timestamp: SystemTime::now(),
            signature: Signature {
                algorithm: "ed25519".to_string(),
                value: vec![0u8; 64],
            },
        };

        // Test ballot anchoring
        let ballot_cid = anchoring_service.anchor_ballot(&ballot);
        assert!(ballot_cid.is_ok(), "Ballot anchoring should succeed");

        let cid = ballot_cid.unwrap();

        // Test ballot retrieval
        let retrieved_ballot = anchoring_service.retrieve_ballot(&cid);
        assert!(retrieved_ballot.is_ok(), "Ballot retrieval should succeed");

        let retrieved = retrieved_ballot.unwrap();
        assert!(retrieved.is_some(), "Retrieved ballot should exist");

        let retrieved_ballot = retrieved.unwrap();
        assert_eq!(retrieved_ballot.ballot_id, ballot.ballot_id);
        assert_eq!(retrieved_ballot.election_id, ballot.election_id);
        assert_eq!(retrieved_ballot.preferences, ballot.preferences);
    }

    #[test]
    fn test_eligibility_rules_creation() {
        let open_rules = EligibilityRules::open_to_all();
        assert!(!open_rules.has_restrictions());

        let federation_rules =
            EligibilityRules::federation_members_only("test-federation".to_string());
        assert!(federation_rules.has_restrictions());
        assert_eq!(
            federation_rules.required_federation,
            Some("test-federation".to_string())
        );

        let reputation_rules = EligibilityRules::reputation_gated(0.75);
        assert!(reputation_rules.has_restrictions());
        assert_eq!(reputation_rules.min_reputation, Some(0.75));
    }

    #[test]
    fn test_eligibility_rules_validation() {
        let open_rules = EligibilityRules::open_to_all();
        let test_did_doc = create_test_did_document();

        // Open rules should accept any valid DID document
        let result = open_rules.validate_voter(&test_did_doc);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Rules with restrictions should indicate they need implementation
        let reputation_rules = EligibilityRules::reputation_gated(0.5);
        let result = reputation_rules.validate_voter(&test_did_doc);
        assert!(result.is_err());
        assert!(matches!(result, Err(VotingError::IneligibleVoter(_))));
    }
}

/// Basic in-memory implementation of FederationRegistry for development and testing
#[derive(Debug, Clone)]
pub struct InMemoryFederationRegistry {
    memberships: std::collections::HashMap<Did, std::collections::HashSet<String>>,
}

impl InMemoryFederationRegistry {
    pub fn new() -> Self {
        Self {
            memberships: std::collections::HashMap::new(),
        }
    }
}

impl Default for InMemoryFederationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FederationRegistry for InMemoryFederationRegistry {
    fn is_member(&self, did: &Did, federation_id: &str) -> Result<bool, VotingError> {
        Ok(self
            .memberships
            .get(did)
            .map(|federations| federations.contains(federation_id))
            .unwrap_or(false))
    }

    fn get_memberships(&self, did: &Did) -> Result<Vec<String>, VotingError> {
        Ok(self
            .memberships
            .get(did)
            .map(|federations| federations.iter().cloned().collect())
            .unwrap_or_default())
    }

    fn add_member(&mut self, did: &Did, federation_id: &str) -> Result<(), VotingError> {
        self.memberships
            .entry(did.clone())
            .or_insert_with(std::collections::HashSet::new)
            .insert(federation_id.to_string());
        Ok(())
    }

    fn remove_member(&mut self, did: &Did, federation_id: &str) -> Result<(), VotingError> {
        if let Some(federations) = self.memberships.get_mut(did) {
            federations.remove(federation_id);
            if federations.is_empty() {
                self.memberships.remove(did);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod enhanced_tests {
    use super::*;
    use icn_reputation::ReputationStore;
    use std::str::FromStr;

    #[test]
    fn test_enhanced_eligibility_validation_with_reputation() {
        use icn_reputation::InMemoryReputationStore;

        let test_did = Did::from_str("did:key:zTestEnhanced").unwrap();
        let test_did_doc = DidDocument {
            id: test_did.clone(),
            public_key: vec![0u8; 32],
        };

        let reputation_rules = EligibilityRules::reputation_gated(75.0);
        let reputation_store = InMemoryReputationStore::new();

        // Start with default reputation (0), which should be below threshold
        let result = reputation_rules.validate_voter_with_context(
            &test_did_doc,
            Some(&reputation_store),
            None,
            None,
        );

        assert!(result.is_err());
        assert!(matches!(result, Err(VotingError::IneligibleVoter(_))));

        // Simulate successful executions to build reputation above threshold
        for _ in 0..100 {
            reputation_store.record_execution(&test_did, true, 1000);
        }

        let result = reputation_rules.validate_voter_with_context(
            &test_did_doc,
            Some(&reputation_store),
            None,
            None,
        );

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_federation_registry_basic_operations() {
        let mut registry = InMemoryFederationRegistry::new();
        let test_did = Did::from_str("did:key:zTestFederation").unwrap();
        let federation_id = "test-federation";

        // Initially not a member
        assert!(!registry.is_member(&test_did, federation_id).unwrap());
        assert!(registry.get_memberships(&test_did).unwrap().is_empty());

        // Add to federation
        registry.add_member(&test_did, federation_id).unwrap();
        assert!(registry.is_member(&test_did, federation_id).unwrap());
        assert_eq!(
            registry.get_memberships(&test_did).unwrap(),
            vec![federation_id.to_string()]
        );

        // Remove from federation
        registry.remove_member(&test_did, federation_id).unwrap();
        assert!(!registry.is_member(&test_did, federation_id).unwrap());
        assert!(registry.get_memberships(&test_did).unwrap().is_empty());
    }

    #[test]
    fn test_enhanced_eligibility_validation_with_federation() {
        let test_did = Did::from_str("did:key:zTestFederationEligibility").unwrap();
        let test_did_doc = DidDocument {
            id: test_did.clone(),
            public_key: vec![0u8; 32],
        };

        let federation_rules =
            EligibilityRules::federation_members_only("test-federation".to_string());
        let mut registry = InMemoryFederationRegistry::new();

        // Not a member initially
        let result = federation_rules.validate_voter_with_context(
            &test_did_doc,
            None,
            None,
            Some(&registry),
        );

        assert!(result.is_err());
        assert!(matches!(result, Err(VotingError::IneligibleVoter(_))));

        // Add to federation
        registry.add_member(&test_did, "test-federation").unwrap();

        let result = federation_rules.validate_voter_with_context(
            &test_did_doc,
            None,
            None,
            Some(&registry),
        );

        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
