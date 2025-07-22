//! Action URL encoding and decoding
//!
//! This module handles the conversion between Action structs and icn:// URLs.

use crate::{Action, ActionError, VoteChoice};
use icn_common::{Did, Cid};
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;
use base64::{Engine as _};

/// Encodes and decodes actions to/from URLs
pub struct ActionEncoder;

impl ActionEncoder {
    /// Encode an action as an icn:// URL
    pub fn encode(action: &Action) -> Result<String, ActionError> {
        let mut url = Url::parse("icn://action").map_err(|e| ActionError::UrlParsing(e))?;
        
        match action {
            Action::ShareIdentity { did } => {
                url.set_path("share");
                url.query_pairs_mut()
                    .append_pair("did", &did.to_string());
            },
            
            Action::ShareContent { cid, title, description } => {
                url.set_path("share");
                url.query_pairs_mut()
                    .append_pair("cid", &cid.to_string());
                
                if let Some(title) = title {
                    url.query_pairs_mut().append_pair("title", title);
                }
                if let Some(description) = description {
                    url.query_pairs_mut().append_pair("description", description);
                }
            },
            
            Action::TransferToken { token, amount, to, memo } => {
                url.set_path("transfer");
                url.query_pairs_mut()
                    .append_pair("token", token)
                    .append_pair("amount", &amount.to_string())
                    .append_pair("to", &to.to_string());
                
                if let Some(memo) = memo {
                    url.query_pairs_mut().append_pair("memo", memo);
                }
            },
            
            Action::Vote { proposal, vote, voter } => {
                url.set_path("vote");
                url.query_pairs_mut()
                    .append_pair("proposal", &proposal.to_string())
                    .append_pair("vote", &vote.to_string());
                
                if let Some(voter) = voter {
                    url.query_pairs_mut().append_pair("voter", &voter.to_string());
                }
            },
            
            Action::JoinFederation { federation_id, invitation_code } => {
                url.set_path("join");
                url.query_pairs_mut()
                    .append_pair("federation", federation_id);
                
                if let Some(code) = invitation_code {
                    url.query_pairs_mut().append_pair("code", code);
                }
            },
            
            Action::VerifyCredential { credential, challenge } => {
                url.set_path("verify");
                url.query_pairs_mut()
                    .append_pair("vc", &credential.to_string());
                
                if let Some(challenge) = challenge {
                    url.query_pairs_mut().append_pair("challenge", challenge);
                }
            },
            
            Action::SubmitJob { job_spec, submitter, max_cost } => {
                url.set_path("submit");
                url.query_pairs_mut()
                    .append_pair("job", &job_spec.to_string())
                    .append_pair("submitter", &submitter.to_string());
                
                if let Some(cost) = max_cost {
                    url.query_pairs_mut().append_pair("max_cost", &cost.to_string());
                }
            },
            
            Action::Custom { action_type, parameters } => {
                url.set_path(action_type);
                for (key, value) in parameters {
                    url.query_pairs_mut().append_pair(key, value);
                }
            },
        }
        
        Ok(url.to_string())
    }
    
    /// Decode an icn:// URL into an action
    pub fn decode(url_str: &str) -> Result<Action, ActionError> {
        let url = Url::parse(url_str).map_err(|e| ActionError::UrlParsing(e))?;
        
        if url.scheme() != "icn" {
            return Err(ActionError::InvalidUrl(
                format!("Expected icn:// scheme, got {}", url.scheme())
            ));
        }
        
        let path = url.path().trim_start_matches('/');
        let params: HashMap<String, String> = url.query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        
        match path {
            "share" => {
                if let Some(did_str) = params.get("did") {
                    let did = Did::from_str(did_str)
                        .map_err(|_| ActionError::DidParsing(did_str.clone()))?;
                    Ok(Action::ShareIdentity { did })
                } else if let Some(cid_str) = params.get("cid") {
                    let cid = Cid::from_str(cid_str)
                        .map_err(|_| ActionError::InvalidParameter(format!("Invalid CID: {}", cid_str)))?;
                    Ok(Action::ShareContent {
                        cid,
                        title: params.get("title").cloned(),
                        description: params.get("description").cloned(),
                    })
                } else {
                    Err(ActionError::MissingParameter("did or cid".to_string()))
                }
            },
            
            "transfer" => {
                let token = params.get("token")
                    .ok_or_else(|| ActionError::MissingParameter("token".to_string()))?
                    .clone();
                
                let amount = params.get("amount")
                    .ok_or_else(|| ActionError::MissingParameter("amount".to_string()))?
                    .parse::<u64>()
                    .map_err(|_| ActionError::InvalidParameter("amount must be a number".to_string()))?;
                
                let to_str = params.get("to")
                    .ok_or_else(|| ActionError::MissingParameter("to".to_string()))?;
                let to = Did::from_str(to_str)
                    .map_err(|_| ActionError::DidParsing(to_str.clone()))?;
                
                Ok(Action::TransferToken {
                    token,
                    amount,
                    to,
                    memo: params.get("memo").cloned(),
                })
            },
            
            "vote" => {
                let proposal_str = params.get("proposal")
                    .ok_or_else(|| ActionError::MissingParameter("proposal".to_string()))?;
                let proposal = Cid::from_str(proposal_str)
                    .map_err(|_| ActionError::InvalidParameter(format!("Invalid proposal CID: {}", proposal_str)))?;
                
                let vote_str = params.get("vote")
                    .ok_or_else(|| ActionError::MissingParameter("vote".to_string()))?;
                let vote = VoteChoice::from_str(vote_str)?;
                
                let voter = if let Some(voter_str) = params.get("voter") {
                    Some(Did::from_str(voter_str)
                        .map_err(|_| ActionError::DidParsing(voter_str.clone()))?)
                } else {
                    None
                };
                
                Ok(Action::Vote {
                    proposal,
                    vote,
                    voter,
                })
            },
            
            "join" => {
                let federation_id = params.get("federation")
                    .ok_or_else(|| ActionError::MissingParameter("federation".to_string()))?
                    .clone();
                
                Ok(Action::JoinFederation {
                    federation_id,
                    invitation_code: params.get("code").cloned(),
                })
            },
            
            "verify" => {
                let credential_str = params.get("vc")
                    .ok_or_else(|| ActionError::MissingParameter("vc".to_string()))?;
                let credential = Cid::from_str(credential_str)
                    .map_err(|_| ActionError::InvalidParameter(format!("Invalid credential CID: {}", credential_str)))?;
                
                Ok(Action::VerifyCredential {
                    credential,
                    challenge: params.get("challenge").cloned(),
                })
            },
            
            "submit" => {
                let job_str = params.get("job")
                    .ok_or_else(|| ActionError::MissingParameter("job".to_string()))?;
                let job_spec = Cid::from_str(job_str)
                    .map_err(|_| ActionError::InvalidParameter(format!("Invalid job CID: {}", job_str)))?;
                
                let submitter_str = params.get("submitter")
                    .ok_or_else(|| ActionError::MissingParameter("submitter".to_string()))?;
                let submitter = Did::from_str(submitter_str)
                    .map_err(|_| ActionError::DidParsing(submitter_str.clone()))?;
                
                let max_cost = if let Some(cost_str) = params.get("max_cost") {
                    Some(cost_str.parse::<u64>()
                        .map_err(|_| ActionError::InvalidParameter("max_cost must be a number".to_string()))?)
                } else {
                    None
                };
                
                Ok(Action::SubmitJob {
                    job_spec,
                    submitter,
                    max_cost,
                })
            },
            
            _ => {
                Ok(Action::Custom {
                    action_type: path.to_string(),
                    parameters: params,
                })
            }
        }
    }
    
    /// Create a shortened URL for QR codes (returns base64 encoded action)
    pub fn encode_compact(action: &Action) -> Result<String, ActionError> {
        let json = serde_json::to_string(action)?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(json.as_bytes());
        Ok(format!("icn://x?d={}", encoded))
    }
    
    /// Decode a compact URL
    pub fn decode_compact(url_str: &str) -> Result<Action, ActionError> {
        let url = Url::parse(url_str).map_err(|e| ActionError::UrlParsing(e))?;
        
        if url.scheme() != "icn" || url.path() != "/x" {
            return Err(ActionError::InvalidUrl("Not a compact ICN URL".to_string()));
        }
        
        let params: HashMap<String, String> = url.query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        
        let data = params.get("d")
            .ok_or_else(|| ActionError::MissingParameter("d".to_string()))?;
        
        let decoded = base64::engine::general_purpose::STANDARD.decode(data)
            .map_err(|_| ActionError::InvalidParameter("Invalid base64 data".to_string()))?;
        
        let json_str = String::from_utf8(decoded)
            .map_err(|_| ActionError::InvalidParameter("Invalid UTF-8 data".to_string()))?;
        
        let action: Action = serde_json::from_str(&json_str)?;
        Ok(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_share_identity_encoding() {
        let did = Did::from_str("did:icn:alice").unwrap();
        let action = Action::ShareIdentity { did: did.clone() };
        
        let url = ActionEncoder::encode(&action).unwrap();
        assert!(url.starts_with("icn://action/share?"));
        assert!(url.contains("did=did%3Aicn%3Aalice"));
        
        let decoded = ActionEncoder::decode(&url).unwrap();
        assert_eq!(decoded, action);
    }
    
    #[test]
    fn test_transfer_token_encoding() {
        let to = Did::from_str("did:icn:bob").unwrap();
        let action = Action::TransferToken {
            token: "seed".to_string(),
            amount: 100,
            to: to.clone(),
            memo: Some("payment".to_string()),
        };
        
        let url = ActionEncoder::encode(&action).unwrap();
        let decoded = ActionEncoder::decode(&url).unwrap();
        assert_eq!(decoded, action);
    }
    
    #[test]
    fn test_vote_encoding() {
        let proposal = Cid::from_str("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").unwrap();
        let action = Action::Vote {
            proposal,
            vote: VoteChoice::Approve,
            voter: None,
        };
        
        let url = ActionEncoder::encode(&action).unwrap();
        let decoded = ActionEncoder::decode(&url).unwrap();
        assert_eq!(decoded, action);
    }
    
    #[test]
    fn test_compact_encoding() {
        let did = Did::from_str("did:icn:alice").unwrap();
        let action = Action::ShareIdentity { did };
        
        let compact_url = ActionEncoder::encode_compact(&action).unwrap();
        assert!(compact_url.starts_with("icn://x?d="));
        
        let decoded = ActionEncoder::decode_compact(&compact_url).unwrap();
        assert_eq!(decoded, action);
    }
}