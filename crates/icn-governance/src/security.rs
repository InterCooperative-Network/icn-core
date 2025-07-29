//! Security enhancements for ICN Governance voting systems
//!
//! This module provides security-hardened implementations for governance
//! operations including ballot validation, signature verification, and
//! protection against various attack vectors.

use crate::voting::{RankedChoiceBallot, Signature, VotingError};
use icn_common::{Signable, TimeProvider};
use icn_identity::{
    security::{secure_validate_did, secure_verify_signature, SecurityConfig},
    verifying_key_from_did_key, EdSignature,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::SystemTime;

/// Security configuration for governance operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceSecurityConfig {
    /// Base cryptographic security config
    pub crypto_config: SecurityConfig,
    /// Maximum ballot size in bytes
    pub max_ballot_size: usize,
    /// Maximum number of preferences allowed
    pub max_preferences: usize,
    /// Maximum election duration in seconds
    pub max_election_duration: u64,
    /// Enable ballot replay protection
    pub replay_protection: bool,
    /// Maximum time skew allowed for ballot timestamps (seconds)
    pub max_time_skew: u64,
    /// Allow clearing replay protection cache (testing/maintenance)
    pub allow_cache_clear: bool,
}

impl Default for GovernanceSecurityConfig {
    fn default() -> Self {
        Self {
            crypto_config: SecurityConfig::default(),
            max_ballot_size: 64 * 1024,               // 64KB
            max_preferences: 100,                     // Reasonable limit for ranked choice
            max_election_duration: 30 * 24 * 60 * 60, // 30 days
            replay_protection: true,
            max_time_skew: 300,       // 5 minutes
            allow_cache_clear: false, // Disallow by default for security
        }
    }
}

/// Security-hardened ballot validator
pub struct SecureBallotValidator {
    config: GovernanceSecurityConfig,
    seen_ballots: HashSet<String>, // For replay protection
}

impl SecureBallotValidator {
    /// Create a new secure ballot validator
    pub fn new(config: GovernanceSecurityConfig) -> Self {
        Self {
            config,
            seen_ballots: HashSet::new(),
        }
    }

    /// Comprehensive ballot validation with security checks
    pub fn validate_ballot(
        &mut self,
        ballot: &RankedChoiceBallot,
        time_provider: &dyn TimeProvider,
    ) -> Result<(), VotingError> {
        // 1. Validate DID format
        secure_validate_did(&ballot.voter_did.id, &self.config.crypto_config)
            .map_err(|_e| VotingError::InvalidSignature)?;

        // 2. Validate ballot structure
        self.validate_ballot_structure(ballot)?;

        // 3. Validate timestamp
        self.validate_timestamp(ballot, time_provider)?;

        // 4. Check for replay attacks
        if self.config.replay_protection {
            self.check_replay_protection(ballot)?;
        }

        // 5. Validate cryptographic signature
        self.validate_signature_secure(ballot)?;

        // 6. Mark ballot as seen (for replay protection)
        if self.config.replay_protection {
            let ballot_hash = self.compute_ballot_hash(ballot)?;
            self.seen_ballots.insert(ballot_hash);
        }

        Ok(())
    }

    /// Validate ballot structure and constraints
    fn validate_ballot_structure(&self, ballot: &RankedChoiceBallot) -> Result<(), VotingError> {
        // Check ballot ID format
        if ballot.ballot_id.0.is_empty() || ballot.ballot_id.0.len() > 256 {
            return Err(VotingError::InvalidBallot(
                "Invalid ballot ID length".to_string(),
            ));
        }

        // Check election ID format
        if ballot.election_id.0.is_empty() || ballot.election_id.0.len() > 256 {
            return Err(VotingError::InvalidBallot(
                "Invalid election ID length".to_string(),
            ));
        }

        // Check preferences count
        if ballot.preferences.len() > self.config.max_preferences {
            return Err(VotingError::InvalidBallot(format!(
                "Too many preferences: {} > {}",
                ballot.preferences.len(),
                self.config.max_preferences
            )));
        }

        // Check for empty preferences
        if ballot.preferences.is_empty() {
            return Err(VotingError::InvalidBallot(
                "Ballot must have at least one preference".to_string(),
            ));
        }

        // Validate preference duplicates (already done by ballot.validate_preferences())
        ballot.validate_preferences()?;

        // Check overall ballot size
        let ballot_size = self.estimate_ballot_size(ballot);
        if ballot_size > self.config.max_ballot_size {
            return Err(VotingError::InvalidBallot(format!(
                "Ballot too large: {} > {} bytes",
                ballot_size, self.config.max_ballot_size
            )));
        }

        Ok(())
    }

    /// Validate ballot timestamp
    fn validate_timestamp(
        &self,
        ballot: &RankedChoiceBallot,
        time_provider: &dyn TimeProvider,
    ) -> Result<(), VotingError> {
        let now =
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(time_provider.unix_seconds());
        let ballot_time = ballot.timestamp;

        // Check if timestamp is too far in the future
        if ballot_time > now {
            let diff = ballot_time.duration_since(now).unwrap_or_default();
            if diff.as_secs() > self.config.max_time_skew {
                return Err(VotingError::InvalidBallot(
                    "Ballot timestamp too far in the future".to_string(),
                ));
            }
        }

        // Check if timestamp is too far in the past (basic sanity check)
        if let Ok(age) = now.duration_since(ballot_time) {
            if age.as_secs() > self.config.max_election_duration {
                return Err(VotingError::InvalidBallot(
                    "Ballot timestamp too old".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Check for replay attacks
    fn check_replay_protection(&self, ballot: &RankedChoiceBallot) -> Result<(), VotingError> {
        let ballot_hash = self.compute_ballot_hash(ballot)?;

        if self.seen_ballots.contains(&ballot_hash) {
            return Err(VotingError::DuplicateVote);
        }

        Ok(())
    }

    /// Compute a unique hash for the ballot to prevent replays
    fn compute_ballot_hash(&self, ballot: &RankedChoiceBallot) -> Result<String, VotingError> {
        use sha2::{Digest, Sha256};

        let signable_bytes = ballot.to_signable_bytes().map_err(|e| {
            VotingError::InvalidBallot(format!("Failed to serialize ballot: {}", e))
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&signable_bytes);
        hasher.update(&ballot.signature.value);

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Validate cryptographic signature using hardened verification
    fn validate_signature_secure(&self, ballot: &RankedChoiceBallot) -> Result<(), VotingError> {
        // Get the verifying key from the DID
        let verifying_key = verifying_key_from_did_key(&ballot.voter_did.id)
            .map_err(|_| VotingError::InvalidSignature)?;

        // Convert governance signature to Ed25519 signature
        let ed_signature = self.convert_signature(&ballot.signature)?;

        // Get signable bytes
        let message = ballot.to_signable_bytes().map_err(|e| {
            VotingError::InvalidBallot(format!("Failed to serialize ballot: {}", e))
        })?;

        // Use hardened signature verification
        let is_valid = secure_verify_signature(
            &verifying_key,
            &message,
            &ed_signature,
            &self.config.crypto_config,
        )
        .map_err(|_| VotingError::InvalidSignature)?;

        if !is_valid {
            return Err(VotingError::InvalidSignature);
        }

        Ok(())
    }

    /// Convert governance Signature to Ed25519 signature
    fn convert_signature(&self, signature: &Signature) -> Result<EdSignature, VotingError> {
        // Validate algorithm
        if signature.algorithm != "ed25519" {
            return Err(VotingError::InvalidSignature);
        }

        // Validate signature length
        if signature.value.len() != 64 {
            return Err(VotingError::InvalidSignature);
        }

        // Convert to Ed25519 signature
        let sig_bytes: [u8; 64] = signature
            .value
            .as_slice()
            .try_into()
            .map_err(|_| VotingError::InvalidSignature)?;

        Ok(EdSignature::from_bytes(&sig_bytes))
    }

    /// Estimate ballot size for validation
    fn estimate_ballot_size(&self, ballot: &RankedChoiceBallot) -> usize {
        let mut size = 0;
        size += ballot.ballot_id.0.len();
        size += 64; // DID approximate size
        size += ballot.election_id.0.len();
        size += ballot.preferences.iter().map(|p| p.0.len()).sum::<usize>();
        size += 8; // timestamp
        size += ballot.signature.value.len();
        size += ballot.signature.algorithm.len();
        size
    }

    /// Clear replay protection cache (for testing or maintenance)
    pub fn clear_replay_cache(&mut self) {
        if !self.config.allow_cache_clear {
            panic!("Clearing the replay cache is not allowed in the current configuration.");
        }
        self.seen_ballots.clear();
    }

    /// Get replay protection cache statistics
    pub fn get_cache_stats(&self) -> (usize, bool) {
        (self.seen_ballots.len(), self.config.replay_protection)
    }
}

/// Security audit result for governance operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceSecurityAudit {
    /// Overall security score
    pub security_score: u8,
    /// Ballot validation statistics
    pub ballot_stats: BallotValidationStats,
    /// Security issues found
    pub issues: Vec<GovernanceSecurityIssue>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Ballot validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotValidationStats {
    pub total_ballots_processed: u64,
    pub valid_ballots: u64,
    pub invalid_ballots: u64,
    pub replay_attempts_detected: u64,
    pub signature_failures: u64,
    pub format_errors: u64,
}

/// Security issue specific to governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceSecurityIssue {
    pub severity: String,
    pub category: String,
    pub description: String,
    pub ballot_id: Option<String>,
    pub recommendation: String,
}

/// Enhanced ballot signing for secure ballot creation
pub struct SecureBallotSigner {
    config: GovernanceSecurityConfig,
}

impl SecureBallotSigner {
    pub fn new(config: GovernanceSecurityConfig) -> Self {
        Self { config }
    }

    /// Sign a ballot with enhanced security
    pub fn sign_ballot(
        &self,
        ballot: &mut RankedChoiceBallot,
        signing_key: &icn_identity::SigningKey,
    ) -> Result<(), VotingError> {
        // Validate ballot structure before signing
        if ballot.preferences.len() > self.config.max_preferences {
            return Err(VotingError::InvalidBallot(
                "Too many preferences".to_string(),
            ));
        }

        // Get signable bytes
        let message = ballot.to_signable_bytes().map_err(|e| {
            VotingError::InvalidBallot(format!("Failed to serialize ballot: {}", e))
        })?;

        // Use hardened signing
        let ed_signature = icn_identity::security::secure_sign_message(
            signing_key,
            &message,
            &self.config.crypto_config,
        )
        .map_err(|_| VotingError::InvalidSignature)?;

        // Convert to governance signature format
        ballot.signature = Signature {
            algorithm: "ed25519".to_string(),
            value: ed_signature.to_bytes().to_vec(),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voting::{BallotId, CandidateId, ElectionId};
    use icn_common::{Did, DidDocument};
    use icn_identity::{did_key_from_verifying_key, generate_ed25519_keypair};
    use std::str::FromStr;

    fn create_test_ballot() -> RankedChoiceBallot {
        let (_, pk) = generate_ed25519_keypair();
        let did_str = did_key_from_verifying_key(&pk);
        let did = Did::from_str(&did_str).unwrap();

        RankedChoiceBallot {
            ballot_id: BallotId("test-ballot-001".to_string()),
            voter_did: DidDocument {
                id: did,
                public_key: pk.as_bytes().to_vec(),
            },
            election_id: ElectionId("test-election".to_string()),
            preferences: vec![
                CandidateId("alice".to_string()),
                CandidateId("bob".to_string()),
            ],
            timestamp: SystemTime::now(),
            signature: Signature {
                algorithm: "ed25519".to_string(),
                value: vec![0u8; 64], // Placeholder
            },
        }
    }

    #[test]
    fn test_ballot_structure_validation() {
        let config = GovernanceSecurityConfig::default();
        let validator = SecureBallotValidator::new(config);

        let mut ballot = create_test_ballot();

        // Test invalid ballot ID
        ballot.ballot_id = BallotId("".to_string());
        let result = validator.validate_ballot_structure(&ballot);
        assert!(result.is_err());

        // Test too many preferences
        ballot = create_test_ballot();
        ballot.preferences = (0..200)
            .map(|i| CandidateId(format!("candidate_{}", i)))
            .collect();
        let result = validator.validate_ballot_structure(&ballot);
        assert!(result.is_err());
    }

    #[test]
    fn test_timestamp_validation() {
        let config = GovernanceSecurityConfig::default();
        let validator = SecureBallotValidator::new(config);

        let mut ballot = create_test_ballot();

        // Test future timestamp
        ballot.timestamp = SystemTime::now() + std::time::Duration::from_secs(1000);
        let result = validator.validate_timestamp(&ballot);
        assert!(result.is_err());

        // Test very old timestamp
        ballot.timestamp = SystemTime::UNIX_EPOCH;
        let result = validator.validate_timestamp(&ballot);
        assert!(result.is_err());
    }

    #[test]
    fn test_replay_protection() {
        let config = GovernanceSecurityConfig::default();
        let mut validator = SecureBallotValidator::new(config);

        let ballot = create_test_ballot();

        // First validation should succeed (structure issues aside)
        let hash1 = validator.compute_ballot_hash(&ballot).unwrap();
        validator.seen_ballots.insert(hash1);

        // Second validation should fail due to replay
        let result = validator.check_replay_protection(&ballot);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VotingError::DuplicateVote));
    }

    #[test]
    fn test_secure_ballot_signing() {
        let config = GovernanceSecurityConfig::default();
        let signer = SecureBallotSigner::new(config);

        let (sk, pk) = generate_ed25519_keypair();
        let did_str = did_key_from_verifying_key(&pk);
        let did = Did::from_str(&did_str).unwrap();

        let mut ballot = RankedChoiceBallot {
            ballot_id: BallotId("test-signing".to_string()),
            voter_did: DidDocument {
                id: did,
                public_key: pk.as_bytes().to_vec(),
            },
            election_id: ElectionId("test-election".to_string()),
            preferences: vec![CandidateId("alice".to_string())],
            timestamp: SystemTime::now(),
            signature: Signature {
                algorithm: "ed25519".to_string(),
                value: vec![0u8; 64],
            },
        };

        // Sign the ballot
        let result = signer.sign_ballot(&mut ballot, &sk);
        assert!(result.is_ok());

        // Verify the signature was updated
        assert_ne!(ballot.signature.value, vec![0u8; 64]);
        assert_eq!(ballot.signature.algorithm, "ed25519");
        assert_eq!(ballot.signature.value.len(), 64);
    }

    #[test]
    fn test_ballot_size_estimation() {
        let config = GovernanceSecurityConfig::default();
        let validator = SecureBallotValidator::new(config);

        let ballot = create_test_ballot();
        let size = validator.estimate_ballot_size(&ballot);

        // Should be reasonable size
        assert!(size > 0);
        assert!(size < 10000); // Should be less than 10KB for a simple ballot
    }

    #[test]
    fn test_signature_conversion() {
        let config = GovernanceSecurityConfig::default();
        let validator = SecureBallotValidator::new(config);

        // Valid signature
        let valid_sig = Signature {
            algorithm: "ed25519".to_string(),
            value: vec![0u8; 64],
        };
        let result = validator.convert_signature(&valid_sig);
        assert!(result.is_ok());

        // Invalid algorithm
        let invalid_sig = Signature {
            algorithm: "rsa".to_string(),
            value: vec![0u8; 64],
        };
        let result = validator.convert_signature(&invalid_sig);
        assert!(result.is_err());

        // Invalid length
        let invalid_sig = Signature {
            algorithm: "ed25519".to_string(),
            value: vec![0u8; 32], // Wrong length
        };
        let result = validator.convert_signature(&invalid_sig);
        assert!(result.is_err());
    }
}
