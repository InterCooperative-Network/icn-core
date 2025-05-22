use icn_common::CommonError;
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
}


// Define ProposalInputType and SubmitProposalRequest as per Step 2
#[derive(Serialize, Deserialize, Debug, Clone)] // Added Clone
#[serde(tag = "type", content = "data")]
pub enum ProposalInputType {
    SystemParameterChange { param: String, value: String },
    MemberAdmission { did: String },
    SoftwareUpgrade { version: String }, // Matches ProposalType more closely
    GenericText { text: String }, // Matches ProposalType more closely
    // Add more as needed
}

#[derive(Serialize, Deserialize, Debug, Clone)] // Added Clone
pub struct SubmitProposalRequest {
    pub proposer_did: String, // Assuming DID is represented as a String for the API layer
    pub proposal: ProposalInputType,
    pub description: String,
    pub duration_secs: u64,
}


pub trait GovernanceApi {
    fn submit_proposal(&self, request: SubmitProposalRequest) -> Result<ProposalId, CommonError>;
    fn cast_vote(&self, request: CastVoteRequest) -> Result<(), CommonError>;
    fn get_proposal(&self, id: ProposalId) -> Result<Option<Proposal>, CommonError>;
    fn list_proposals(&self) -> Result<Vec<Proposal>, CommonError>;
} 