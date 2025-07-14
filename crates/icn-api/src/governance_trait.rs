use icn_common::{CommonError, ZkCredentialProof, ZkRevocationProof};
use icn_governance::{Proposal, ProposalId};
use serde::{Deserialize, Serialize}; // Added for ProposalInputType later

// Forward declare for CastVoteRequest for now, will be defined fully later.
// This avoids a direct dependency cycle if CastVoteRequest were to use parts of GovernanceApi or vice-versa in complex ways.
// However, for simple data structures, it's fine to define them directly here or in a shared types module.
#[derive(Serialize, Deserialize, Debug)]
pub struct CastVoteRequest {
    pub voter_did: String, // Assuming DID is represented as a String for the API layer
    pub proposal_id: String, // ProposalId represented as String for API layer
    pub vote_option: String, // e.g., "Yes", "No", "Abstain" - will map to VoteOption enum
    #[serde(default)]
    pub credential_proof: Option<ZkCredentialProof>,
    #[serde(default)]
    pub revocation_proof: Option<ZkRevocationProof>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DelegateRequest {
    pub from_did: String,
    pub to_did: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RevokeDelegationRequest {
    pub from_did: String,
}

// Define ProposalInputType and SubmitProposalRequest as per Step 2
#[derive(Serialize, Deserialize, Debug, Clone)] // Added Clone
#[serde(tag = "type", content = "data")]
pub enum ProposalInputType {
    SystemParameterChange {
        param: String,
        value: String,
    },
    MemberAdmission {
        did: String,
    },
    /// Remove an existing member from the cooperative
    RemoveMember {
        did: String,
    },
    SoftwareUpgrade {
        version: String,
    }, // Matches ProposalType more closely
    GenericText {
        text: String,
    }, // Matches ProposalType more closely
    Resolution {
        actions: Vec<ResolutionActionInput>,
    },
    // Add more as needed
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", content = "data")]
pub enum ResolutionActionInput {
    PauseCredential { cid: String },
    FreezeReputation { did: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)] // Added Clone
pub struct SubmitProposalRequest {
    pub proposer_did: String, // Assuming DID is represented as a String for the API layer
    pub proposal: ProposalInputType,
    pub description: String,
    pub duration_secs: u64,
    pub quorum: Option<usize>,
    pub threshold: Option<f32>,
    pub body: Option<Vec<u8>>,
    #[serde(default)]
    pub credential_proof: Option<ZkCredentialProof>,
    #[serde(default)]
    pub revocation_proof: Option<ZkRevocationProof>,
}

pub trait GovernanceApi {
    fn submit_proposal(&self, request: SubmitProposalRequest) -> Result<ProposalId, CommonError>;
    fn cast_vote(&self, request: CastVoteRequest) -> Result<(), CommonError>;
    fn get_proposal(&self, id: ProposalId) -> Result<Option<Proposal>, CommonError>;
    fn list_proposals(&self) -> Result<Vec<Proposal>, CommonError>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CloseProposalResponse {
    pub status: String,
    pub yes: usize,
    pub no: usize,
    pub abstain: usize,
}
