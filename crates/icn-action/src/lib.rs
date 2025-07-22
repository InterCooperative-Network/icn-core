//! ICN Action Encoding and QR/NFC Support
//!
//! This crate provides URL-based action encoding for ICN, enabling seamless
//! QR code and NFC interactions for identity sharing, token transfers,
//! governance participation, and cooperative actions.
//!
//! # Action URL Scheme
//! 
//! Actions are encoded as URLs using the `icn://` scheme:
//! - `icn://share?did=did:icn:alice` - Share identity
//! - `icn://transfer?token=seed&amount=10&to=did:icn:bob` - Transfer tokens
//! - `icn://vote?proposal=cid:...&vote=approve` - Vote on proposal
//! - `icn://join?coop=federation-name` - Join federation
//! - `icn://verify?vc=cid:...` - Verify credential
//!
//! # Example Usage
//!
//! ```rust
//! use icn_action::{Action, ActionEncoder, QrGenerator};
//! use icn_common::Did;
//! 
//! // Create an identity sharing action
//! let did = Did::from_str("did:icn:alice")?;
//! let action = Action::ShareIdentity { did };
//! 
//! // Encode as URL
//! let url = ActionEncoder::encode(&action)?;
//! println!("Action URL: {}", url);
//! 
//! // Generate QR code
//! let qr_code = QrGenerator::generate_png(&url, 256)?;
//! 
//! // Decode back from URL
//! let decoded_action = ActionEncoder::decode(&url)?;
//! ```

use icn_common::{Did, Cid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;


pub mod qr;
pub mod encoder;

pub use qr::QrGenerator;
pub use encoder::ActionEncoder;

/// Errors that can occur during action processing
#[derive(Error, Debug)]
pub enum ActionError {
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    
    #[error("Invalid parameter value: {0}")]
    InvalidParameter(String),
    
    #[error("Unsupported action type: {0}")]
    UnsupportedAction(String),
    
    #[error("QR code generation failed: {0}")]
    QrGeneration(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("URL parsing error: {0}")]
    UrlParsing(#[from] url::ParseError),
    
    #[error("DID parsing error: {0}")]
    DidParsing(String),
}

/// Types of actions that can be encoded in QR codes or NFC tags
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// Share identity (DID)
    ShareIdentity {
        did: Did,
    },
    
    /// Share content by CID
    ShareContent {
        cid: Cid,
        title: Option<String>,
        description: Option<String>,
    },
    
    /// Transfer tokens
    TransferToken {
        token: String,
        amount: u64,
        to: Did,
        memo: Option<String>,
    },
    
    /// Vote on a proposal
    Vote {
        proposal: Cid,
        vote: VoteChoice,
        voter: Option<Did>,
    },
    
    /// Join a federation/cooperative
    JoinFederation {
        federation_id: String,
        invitation_code: Option<String>,
    },
    
    /// Verify a credential
    VerifyCredential {
        credential: Cid,
        challenge: Option<String>,
    },
    
    /// Submit a job to the mesh
    SubmitJob {
        job_spec: Cid,
        submitter: Did,
        max_cost: Option<u64>,
    },
    
    /// Generic action with custom parameters
    Custom {
        action_type: String,
        parameters: HashMap<String, String>,
    },
}

/// Vote choices for governance proposals
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VoteChoice {
    Approve,
    Reject,
    Abstain,
}

impl FromStr for VoteChoice {
    type Err = ActionError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "approve" | "yes" | "y" => Ok(VoteChoice::Approve),
            "reject" | "no" | "n" => Ok(VoteChoice::Reject),
            "abstain" | "a" => Ok(VoteChoice::Abstain),
            _ => Err(ActionError::InvalidParameter(format!("Invalid vote choice: {}", s))),
        }
    }
}

impl std::fmt::Display for VoteChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoteChoice::Approve => write!(f, "approve"),
            VoteChoice::Reject => write!(f, "reject"),
            VoteChoice::Abstain => write!(f, "abstain"),
        }
    }
}

/// Metadata for QR code generation
#[derive(Debug, Clone)]
pub struct QrMetadata {
    pub size: u32,
    pub border: u32,
    pub error_correction: QrErrorCorrection,
    pub format: QrFormat,
}

impl Default for QrMetadata {
    fn default() -> Self {
        Self {
            size: 256,
            border: 4,
            error_correction: QrErrorCorrection::Medium,
            format: QrFormat::Png,
        }
    }
}

/// QR code error correction levels
#[derive(Debug, Clone)]
pub enum QrErrorCorrection {
    Low,
    Medium, 
    Quartile,
    High,
}

/// QR code output formats
#[derive(Debug, Clone)]
pub enum QrFormat {
    Png,
    Svg,
    Terminal,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vote_choice_parsing() {
        assert_eq!(VoteChoice::from_str("approve").unwrap(), VoteChoice::Approve);
        assert_eq!(VoteChoice::from_str("YES").unwrap(), VoteChoice::Approve);
        assert_eq!(VoteChoice::from_str("reject").unwrap(), VoteChoice::Reject);
        assert_eq!(VoteChoice::from_str("NO").unwrap(), VoteChoice::Reject);
        assert_eq!(VoteChoice::from_str("abstain").unwrap(), VoteChoice::Abstain);
        assert!(VoteChoice::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_vote_choice_display() {
        assert_eq!(VoteChoice::Approve.to_string(), "approve");
        assert_eq!(VoteChoice::Reject.to_string(), "reject");  
        assert_eq!(VoteChoice::Abstain.to_string(), "abstain");
    }
}