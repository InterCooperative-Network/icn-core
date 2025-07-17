// crates/icn-cli/src/credential_lifecycle.rs
//! Credential lifecycle management CLI commands

use clap::Subcommand;
use icn_common::{Cid, Did, ZkCredentialProof};
use icn_identity::cooperative_schemas::{
    SkillCredential, CooperativeMembership, ServiceProvider, TrustAttestation
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple reputation levels for credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReputationLevel {
    Newcomer,
    Contributor,
    Coordinator,
    Steward,
}

/// Simple reputation credential structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationCredential {
    pub score: u32,
    pub level: ReputationLevel,
    pub evidence_cids: Vec<Cid>,
    pub calculation_method: String,
    pub calculated_at: u64,
}

/// Credential lifecycle commands
#[derive(Subcommand, Debug)]
pub enum CredentialLifecycleCommands {
    /// Issue a new credential
    Issue {
        #[clap(subcommand)]
        credential_type: IssueCommands,
    },
    /// Present a credential for verification
    Present {
        #[clap(help = "Path to credential file or '-' for stdin")]
        credential_path: String,
        #[clap(help = "Context where credential is being presented")]
        context: String,
    },
    /// Verify a presented credential
    Verify {
        #[clap(help = "Path to credential file or '-' for stdin")]
        credential_path: String,
        #[clap(long, help = "Required verification level (basic|enhanced|strict)")]
        level: Option<String>,
    },
    /// Anchor a credential disclosure to the DAG
    Anchor {
        #[clap(help = "Path to credential disclosure file")]
        disclosure_path: String,
        #[clap(long, help = "Additional metadata as JSON")]
        metadata: Option<String>,
    },
    /// Show credential status and history
    Status {
        #[clap(help = "Credential CID")]
        cid: String,
    },
    /// List credentials by holder or issuer
    List {
        #[clap(long, help = "Filter by holder DID")]
        holder: Option<String>,
        #[clap(long, help = "Filter by issuer DID")]
        issuer: Option<String>,
        #[clap(long, help = "Filter by credential type")]
        credential_type: Option<String>,
    },
    /// Revoke a credential
    Revoke {
        #[clap(help = "Credential CID to revoke")]
        cid: String,
        #[clap(long, help = "Reason for revocation")]
        reason: String,
    },
    /// Run example credential flows
    Example {
        #[clap(subcommand)]
        flow: ExampleFlows,
    },
}

/// Credential issuance commands
#[derive(Subcommand, Debug)]
pub enum IssueCommands {
    /// Issue a skill credential
    Skill {
        #[clap(long, help = "Holder's DID")]
        holder: String,
        #[clap(long, help = "Skill name")]
        skill_name: String,
        #[clap(long, help = "Proficiency level (1-10)")]
        level: u8,
        #[clap(long, help = "Years of experience")]
        years_experience: u32,
        #[clap(long, help = "Endorsed by DID")]
        endorsed_by: Option<String>,
        #[clap(long, help = "Evidence links (comma-separated URLs)")]
        evidence: Option<String>,
    },
    /// Issue a cooperative membership credential
    Membership {
        #[clap(long, help = "Member's DID")]
        holder: String,
        #[clap(long, help = "Cooperative name")]
        cooperative_name: String,
        #[clap(long, help = "Membership level")]
        level: String,
        #[clap(long, help = "Member since (YYYY-MM-DD)")]
        member_since: String,
        #[clap(long, help = "Voting rights")]
        voting_rights: bool,
    },
    /// Issue a service provider credential
    Service {
        #[clap(long, help = "Provider's DID")]
        holder: String,
        #[clap(long, help = "Service type")]
        service_type: String,
        #[clap(long, help = "Service description")]
        description: String,
        #[clap(long, help = "Certification level")]
        certification_level: String,
    },
    /// Issue a reputation credential
    Reputation {
        #[clap(long, help = "Holder's DID")]
        holder: String,
        #[clap(long, help = "Reputation score")]
        score: u32,
        #[clap(long, help = "Reputation level (newcomer|contributor|coordinator|steward)")]
        level: String,
        #[clap(long, help = "Evidence CIDs (comma-separated)")]
        evidence: Option<String>,
    },
}

/// Example credential flows
#[derive(Subcommand, Debug)]
pub enum ExampleFlows {
    /// Full skill-to-voting flow
    SkillToVoting {
        #[clap(long, help = "Participant's DID")]
        participant: String,
        #[clap(long, help = "Skill to demonstrate")]
        skill: String,
    },
    /// Reputation update cycle
    ReputationCycle {
        #[clap(long, help = "Participant's DID")]
        participant: String,
    },
    /// Cross-federation credential verification
    CrossFederation {
        #[clap(long, help = "Source federation DID")]
        source_federation: String,
        #[clap(long, help = "Target federation DID")]
        target_federation: String,
        #[clap(long, help = "Credential to transfer")]
        credential_cid: String,
    },
}

/// Credential presentation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialPresentation {
    pub credential: ZkCredentialProof,
    pub context: String,
    pub timestamp: u64,
    pub nonce: String,
    pub presenter: Did,
}

/// Credential verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialVerificationResult {
    pub valid: bool,
    pub verification_level: String,
    pub verified_claims: HashMap<String, serde_json::Value>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub timestamp: u64,
}

/// Credential disclosure (for anchoring)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialDisclosure {
    pub credential_cid: Cid,
    pub disclosed_fields: Vec<String>,
    pub presentation_context: String,
    pub verifier: Did,
    pub timestamp: u64,
    pub proof_of_presentation: ZkCredentialProof,
}

/// Credential status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStatus {
    pub cid: Cid,
    pub issuer: Did,
    pub holder: Did,
    pub credential_type: String,
    pub issued_at: u64,
    pub valid_until: Option<u64>,
    pub revoked: bool,
    pub revoked_at: Option<u64>,
    pub revocation_reason: Option<String>,
    pub presentations: Vec<CredentialPresentation>,
    pub anchored_disclosures: Vec<Cid>,
}

/// Credential metadata for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMetadata {
    pub cid: Cid,
    pub issuer: Did,
    pub holder: Did,
    pub credential_type: String,
    pub issued_at: u64,
    pub status: String,
}

/// Implementation functions (would be in main.rs or a separate module)
impl CredentialLifecycleCommands {
    /// Generate a skill credential
    pub fn generate_skill_credential(
        _issuer: &Did,
        _holder: &Did,
        skill_name: &str,
        level: u8,
        years_experience: u32,
        endorsed_by: Option<&Did>,
        evidence: Option<Vec<String>>,
    ) -> Result<SkillCredential, String> {
        let skill_cred = SkillCredential {
            skill_name: skill_name.to_string(),
            proficiency_level: level,
            years_experience,
            endorsed_by: endorsed_by.cloned(),
            evidence_links: evidence.unwrap_or_default(),
        };
        
        Ok(skill_cred)
    }
    
    /// Generate a cooperative membership credential
    pub fn generate_membership_credential(
        _issuer: &Did,
        _holder: &Did,
        cooperative_name: &str,
        level: &str,
        member_since: u64,
        voting_rights: bool,
    ) -> Result<CooperativeMembership, String> {
        let membership = CooperativeMembership {
            cooperative_name: cooperative_name.to_string(),
            membership_level: level.to_string(),
            member_since,
            voting_rights,
            delegated_to: None,
        };
        
        Ok(membership)
    }
    
    /// Generate a reputation credential
    pub fn generate_reputation_credential(
        _issuer: &Did,
        _holder: &Did,
        score: u32,
        level: &str,
        evidence: Option<Vec<Cid>>,
    ) -> Result<ReputationCredential, String> {
        let reputation_level = match level {
            "newcomer" => ReputationLevel::Newcomer,
            "contributor" => ReputationLevel::Contributor,
            "coordinator" => ReputationLevel::Coordinator,
            "steward" => ReputationLevel::Steward,
            _ => return Err(format!("Invalid reputation level: {}", level)),
        };
        
        let reputation = ReputationCredential {
            score,
            level: reputation_level,
            evidence_cids: evidence.unwrap_or_default(),
            calculation_method: "standard".to_string(),
            calculated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        Ok(reputation)
    }
    
    /// Create a credential presentation
    pub fn create_presentation(
        credential: ZkCredentialProof,
        context: &str,
        presenter: &Did,
    ) -> CredentialPresentation {
        use rand::Rng;
        
        let mut rng = rand::thread_rng();
        let nonce = (0..16).map(|_| format!("{:02x}", rng.gen::<u8>())).collect::<String>();
        
        CredentialPresentation {
            credential,
            context: context.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            nonce,
            presenter: presenter.clone(),
        }
    }
    
    /// Verify a credential presentation
    pub fn verify_presentation(
        presentation: &CredentialPresentation,
        required_level: &str,
    ) -> CredentialVerificationResult {
        let mut warnings = Vec::new();
        let mut _errors = Vec::new();
        let mut verified_claims = HashMap::new();
        
        // Basic validation
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Check timestamp freshness (within 1 hour)
        if current_time - presentation.timestamp > 3600 {
            warnings.push("Presentation timestamp is older than 1 hour".to_string());
        }
        
        // Verify the credential itself (simplified)
        // In a real implementation, this would use cryptographic verification
        verified_claims.insert("issuer".to_string(), 
            serde_json::Value::String(presentation.credential.issuer.to_string()));
        verified_claims.insert("holder".to_string(), 
            serde_json::Value::String(presentation.credential.holder.to_string()));
        verified_claims.insert("claim_type".to_string(), 
            serde_json::Value::String(presentation.credential.claim_type.clone()));
        
        let valid = _errors.is_empty();
        
        CredentialVerificationResult {
            valid,
            verification_level: required_level.to_string(),
            verified_claims,
            warnings,
            errors: _errors,
            timestamp: current_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::Did;
    
    #[test]
    fn test_skill_credential_generation() {
        let issuer = Did::new("key", "issuer123");
        let holder = Did::new("key", "holder456");
        
        let skill_cred = CredentialLifecycleCommands::generate_skill_credential(
            &issuer,
            &holder,
            "Rust Programming",
            8,
            3,
            None,
            Some(vec!["https://github.com/example".to_string()]),
        );
        
        assert!(skill_cred.is_ok());
        let cred = skill_cred.unwrap();
        assert_eq!(cred.skill_name, "Rust Programming");
        assert_eq!(cred.proficiency_level, 8);
        assert_eq!(cred.years_experience, 3);
    }
    
    #[test]
    fn test_credential_presentation() {
        let presenter = Did::new("key", "presenter789");
        let credential = ZkCredentialProof {
            issuer: Did::new("key", "issuer123"),
            holder: Did::new("key", "holder456"),
            claim_type: "skill".to_string(),
            proof: vec![1, 2, 3, 4],
            schema: Cid::new("test-schema"),
        };
        
        let presentation = CredentialLifecycleCommands::create_presentation(
            credential,
            "governance_vote",
            &presenter,
        );
        
        assert_eq!(presentation.context, "governance_vote");
        assert_eq!(presentation.presenter, presenter);
        assert!(!presentation.nonce.is_empty());
    }
    
    #[test]
    fn test_credential_verification() {
        let presenter = Did::new("key", "presenter789");
        let credential = ZkCredentialProof {
            issuer: Did::new("key", "issuer123"),
            holder: Did::new("key", "holder456"),
            claim_type: "skill".to_string(),
            proof: vec![1, 2, 3, 4],
            schema: Cid::new("test-schema"),
        };
        
        let presentation = CredentialLifecycleCommands::create_presentation(
            credential,
            "governance_vote",
            &presenter,
        );
        
        let result = CredentialLifecycleCommands::verify_presentation(
            &presentation,
            "basic",
        );
        
        assert!(result.valid);
        assert_eq!(result.verification_level, "basic");
        assert!(result.verified_claims.contains_key("issuer"));
    }
}