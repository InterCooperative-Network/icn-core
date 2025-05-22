#![doc = include_str!("../README.md")]

//! # ICN API Crate
//! This crate provides the primary API endpoints for interacting with InterCooperative Network (ICN) nodes.
//! It defines service interfaces, data structures for requests and responses, and potentially server/client implementations.
//! The API aims for clarity, modularity, and extensibility, typically using JSON-RPC or gRPC.

// Depending on icn_common crate
use icn_common::{NodeInfo, NodeStatus, CommonError, ICN_CORE_VERSION, DagBlock, Cid, Did};
// Remove direct use of icn_dag::put_block and icn_dag::get_block which use global store
// use icn_dag::{put_block as dag_put_block, get_block as dag_get_block};
use icn_dag::StorageService; // Import the trait
use std::sync::{Arc, Mutex}; // To accept the storage service
// Added imports for network functionality
use icn_network::{PeerId, NetworkMessage, NetworkService, StubNetworkService};
// Added imports for governance functionality
use icn_governance::{GovernanceModule, ProposalId, ProposalType, VoteOption, Proposal};
use serde::{Serialize, Deserialize};
use std::str::FromStr;

pub mod governance_trait;
use crate::governance_trait::{
    GovernanceApi, 
    SubmitProposalRequest as GovernanceSubmitProposalRequest, // Renamed to avoid conflict
    CastVoteRequest as GovernanceCastVoteRequest,       // Renamed to avoid conflict
    ProposalInputType
};

/// Planned: Define a trait for the ICN API service for RPC implementation.
// pub trait IcnApiService {
//    async fn get_node_info(&self) -> Result<NodeInfo, CommonError>;
//    async fn get_node_status(&self) -> Result<NodeStatus, CommonError>;
//    async fn submit_dag_block(&self, block: DagBlock) -> Result<Cid, CommonError>;
//    async fn retrieve_dag_block(&self, cid: Cid) -> Result<Option<DagBlock>, CommonError>;
    // TODO: Add other API methods: submit_transaction, query_data, etc.
// }

/// Retrieves basic information about the ICN node.
/// This function would typically be part of an RPC service.
pub fn get_node_info() -> Result<NodeInfo, CommonError> {
    Ok(NodeInfo {
        version: ICN_CORE_VERSION.to_string(),
        name: "ICN Node (Default Name)".to_string(),
        status_message: "Node is operational".to_string(),
    })
}

/// Retrieves the current operational status of the ICN node.
/// This function simulates a potential error if the node is considered "offline".
pub fn get_node_status(is_simulated_online: bool) -> Result<NodeStatus, CommonError> {
    if !is_simulated_online {
        return Err(CommonError::NodeOffline("Node is currently simulated offline.".to_string()));
    }

    // In a real scenario, these values would be fetched from the node's internal state.
    Ok(NodeStatus {
        is_online: true,
        peer_count: 5, // Example value
        current_block_height: 1000, // Example value
        version: ICN_CORE_VERSION.to_string(),
    })
}

/// Submits a DagBlock to the provided DAG store.
/// Returns the CID of the stored block upon success.
pub fn submit_dag_block(
    storage: Arc<Mutex<dyn StorageService<DagBlock> + Send + Sync>>,
    block_data_json: String,
) -> Result<Cid, CommonError> {
    let block: DagBlock = serde_json::from_str(&block_data_json)
        .map_err(|e| CommonError::DeserializationError(format!("Failed to parse DagBlock JSON for submission: {} (Input: '{}')", e, block_data_json)))?;

    // TODO: Validate the block. Especially, recalculate its CID based on data and links
    // and ensure it matches block.cid. If not, return CommonError::DagValidationError.
    // For now, we trust the provided CID.

    let mut store = storage.lock().map_err(|_e| CommonError::StorageError("API: Failed to acquire lock on storage for put.".to_string()))?;

    store.put(&block)
        .map_err(|e| match e {
            CommonError::StorageError(msg) => CommonError::StorageError(format!("API: Failed to store DagBlock: {}", msg)),
            CommonError::DagValidationError(msg) => CommonError::DagValidationError(format!("API: Invalid DagBlock: {}", msg)),
            CommonError::SerializationError(msg) => CommonError::SerializationError(format!("API: Serialization error during put: {}", msg)),
            CommonError::DeserializationError(msg) => CommonError::DeserializationError(format!("API: Deserialization error during put: {}", msg)),
            _ => CommonError::ApiError(format!("API: Unexpected error during store.put: {:?}", e)),
        })?;
    Ok(block.cid.clone())
}

/// Retrieves a DagBlock from the provided DAG store by its CID.
/// The result is an Option<DagBlock> to indicate if the block was found.
pub fn retrieve_dag_block(
    storage: Arc<Mutex<dyn StorageService<DagBlock> + Send + Sync>>,
    cid_json: String,
) -> Result<Option<DagBlock>, CommonError> {
    let cid: Cid = serde_json::from_str(&cid_json)
        .map_err(|e| CommonError::DeserializationError(format!("Failed to parse CID JSON for retrieval: {} (Input: '{}')", e, cid_json)))?;
    
    let store = storage.lock().map_err(|_e| CommonError::StorageError("API: Failed to acquire lock on storage for get.".to_string()))?;

    store.get(&cid)
        .map_err(|e| match e {
            CommonError::StorageError(msg) => CommonError::StorageError(format!("API: Failed to retrieve DagBlock: {}", msg)),
            CommonError::DeserializationError(msg) => CommonError::DeserializationError(format!("API: Deserialization error during get: {}", msg)),
            // Note: get typically shouldn't cause SerializationError or DagValidationError unless the store is corrupted
            _ => CommonError::ApiError(format!("API: Unexpected error during store.get: {:?}", e)),
        })
}

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

// --- Governance API Functions ---

/// Concrete implementation for the Governance API
pub struct GovernanceApiImpl {
    pub gov_module: Arc<Mutex<GovernanceModule>>,
}

impl GovernanceApiImpl {
    pub fn new(gov_module: Arc<Mutex<GovernanceModule>>) -> Self {
        Self { gov_module }
    }
}

impl GovernanceApi for GovernanceApiImpl {
    fn submit_proposal(&self, request: GovernanceSubmitProposalRequest) -> Result<ProposalId, CommonError> {
        let proposer_did = Did::from_str(&request.proposer_did)
            .map_err(|e| CommonError::InvalidInputError(format!("Invalid proposer_did format: {}. Error: {:?}", request.proposer_did, e)))?;

        let core_proposal_type = match request.proposal {
            ProposalInputType::SystemParameterChange { param, value } => {
                ProposalType::SystemParameterChange(param, value)
            }
            ProposalInputType::MemberAdmission { did } => {
                let member_did = Did::from_str(&did)
                    .map_err(|e| CommonError::InvalidInputError(format!("Invalid member DID format for admission: {}. Error: {:?}", did, e)))?;
                ProposalType::NewMemberInvitation(member_did)
            }
            ProposalInputType::SoftwareUpgrade { version } => {
                ProposalType::SoftwareUpgrade(version)
            }
            ProposalInputType::GenericText { text } => {
                ProposalType::GenericText(text)
            }
        };

        let mut module = self.gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for submitting proposal".to_string()))?;
        
        module.submit_proposal(proposer_did, core_proposal_type, request.description, request.duration_secs)
    }

    fn cast_vote(&self, request: GovernanceCastVoteRequest) -> Result<(), CommonError> {
        let voter_did = Did::from_str(&request.voter_did)
            .map_err(|e| CommonError::InvalidInputError(format!("Invalid voter_did format: {}. Error: {:?}", request.voter_did, e)))?;
        
        let proposal_id = ProposalId::from_str(&request.proposal_id)
             .map_err(|e| CommonError::InvalidInputError(format!("Invalid proposal_id format: {}. Error: {:?}", request.proposal_id, e)))?;

        let vote_option = match request.vote_option.to_lowercase().as_str() {
            "yes" => VoteOption::Yes,
            "no" => VoteOption::No,
            "abstain" => VoteOption::Abstain,
            _ => return Err(CommonError::InvalidInputError(format!("Invalid vote option: {}. Must be one of 'yes', 'no', 'abstain'.", request.vote_option))),
        };

        let mut module = self.gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for casting vote".to_string()))?;
        module.cast_vote(voter_did, &proposal_id, vote_option)
    }

    fn get_proposal(&self, id: ProposalId) -> Result<Option<Proposal>, CommonError> {
        let module = self.gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for getting proposal".to_string()))?;
        module.get_proposal(&id)
    }

    fn list_proposals(&self) -> Result<Vec<Proposal>, CommonError> {
        let module = self.gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for listing proposals".to_string()))?;
        module.list_proposals()
    }
}

// --- Old Governance API Functions (to be removed or adapted) ---
// These functions are now replaced by the GovernanceApiImpl methods.
// They are commented out to ensure the build uses the new trait-based approach.
// Consider how downstream users (e.g. RPC layer, CLI) will call these.
// For now, we assume they will instantiate GovernanceApiImpl and use its methods.

// /// API endpoint to submit a new governance proposal.
// pub fn submit_proposal_api(
//     gov_module: Arc<Mutex<GovernanceModule>>,
//     request_json: String,
// ) -> Result<String, CommonError> { // Returns ProposalId as String
//     let request: SubmitProposalRequest = serde_json::from_str(&request_json)
//         .map_err(|e| CommonError::DeserializationError(format!("Failed to parse SubmitProposalRequest JSON: {}", e)))?;
// 
//     let proposer_did = Did::from_str(&request.proposer_did)
//         .map_err(|e| CommonError::InvalidInputError(format!("Invalid proposer_did format: {:?}", e)))?;
//     
//     // Deserialize ProposalType from request.proposal_type_json
//     // This is a bit manual; a more robust solution might involve a tagged enum for ProposalType on the API boundary
//     let proposal_type: ProposalType = serde_json::from_value(request.proposal_type_json.clone()).map_err(|e| CommonError::DeserializationError(format!("Failed to parse ProposalType from JSON value {:?}: {}", request.proposal_type_json, e)))?;
//     
//     let mut module = gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for submitting proposal".to_string()))?;\n    
//     let proposal_id = module.submit_proposal(proposer_did, proposal_type, request.description, request.duration_secs)?;\n    Ok(proposal_id.0)\n}
// 
// /// API endpoint to cast a vote on a proposal.
// pub fn cast_vote_api(
//     gov_module: Arc<Mutex<GovernanceModule>>,
//     request_json: String,
// ) -> Result<(), CommonError> {
//     let request: CastVoteRequest = serde_json::from_str(&request_json)
//         .map_err(|e| CommonError::DeserializationError(format!("Failed to parse CastVoteRequest JSON: {}", e)))?;
// 
//     let voter_did = Did::from_str(&request.voter_did)
//         .map_err(|e| CommonError::InvalidInputError(format!("Invalid voter_did format: {:?}", e)))?;\n    let proposal_id = ProposalId(request.proposal_id);\n    let vote_option = match request.vote_option.to_lowercase().as_str() {\n        "yes" => VoteOption::Yes,
//         "no" => VoteOption::No,
//         "abstain" => VoteOption::Abstain,
//         _ => return Err(CommonError::InvalidInputError(format!("Invalid vote option: {}. Must be one of 'yes', 'no', 'abstain'.", request.vote_option))),
//     };
// 
//     let mut module = gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for casting vote".to_string()))?;\n    module.cast_vote(voter_did, &proposal_id, vote_option)\n}
// 
// /// API endpoint to get a specific proposal by its ID.
// pub fn get_proposal_api(
//     gov_module: Arc<Mutex<GovernanceModule>>,
//     proposal_id_json: String, // Proposal ID as a JSON string
// ) -> Result<Option<Proposal>, CommonError> { // Returns the full Proposal struct (needs to be serializable)
//     let proposal_id_str: String = serde_json::from_str(&proposal_id_json)
//         .map_err(|e| CommonError::DeserializationError(format!("Failed to parse Proposal ID JSON: {}", e)))?;\n    let proposal_id = ProposalId::from_str(&proposal_id_str)\n        .map_err(|e| CommonError::InvalidInputError(format!("Invalid ProposalId format: {}", e)))?;\n\n    let module = gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for getting proposal".to_string()))?;\n    module.get_proposal(&proposal_id) // This now returns Result<Option<Proposal>, CommonError>\n}
// 
// /// API endpoint to list all current proposals.
// pub fn list_proposals_api(
//     gov_module: Arc<Mutex<GovernanceModule>>,
// ) -> Result<Vec<Proposal>, CommonError> { // Returns a list of full Proposal structs
//     let module = gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for listing proposals".to_string()))?;\n    module.list_proposals() // This now returns Result<Vec<Proposal>, CommonError>\n}

// --- Network API Functions ---

/// API endpoint to discover network peers (currently uses StubNetworkService).
/// Takes a list of bootstrap node addresses (currently ignored by stub but good for API design).
pub fn discover_peers_api(bootstrap_nodes_str: Vec<String>) -> Result<Vec<PeerId>, CommonError> {
    let network_service = StubNetworkService::default(); 
    // In a real scenario, bootstrap_nodes_str might need parsing into a more specific type.
    network_service.discover_peers(bootstrap_nodes_str)
        .map_err(|e| CommonError::ApiError(format!("Failed to discover peers via network service: {:?}", e)))
}

/// API endpoint to send a message to a specific peer (currently uses StubNetworkService).
/// `peer_id_str` is the string representation of the target PeerId.
/// `message_json` is a JSON string representation of the NetworkMessage.
pub fn send_network_message_api(peer_id_str: String, message_json: String) -> Result<(), CommonError> {
    let network_service = StubNetworkService::default();
    let peer_id = PeerId(peer_id_str); // Assuming PeerId is a simple wrapper around String for now.

    // Deserialize the NetworkMessage from JSON. 
    // This requires NetworkMessage to implement Deserialize.
    let message: NetworkMessage = serde_json::from_str(&message_json)
        .map_err(|e| CommonError::DeserializationError(format!("Failed to parse NetworkMessage JSON: {}. Input: {}", e, message_json)))?;

    network_service.send_message(&peer_id, message)
        .map_err(|e| CommonError::ApiError(format!("Failed to send message via network service: {:?}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::DagLink; // For test setup
    use icn_dag::InMemoryDagStore; // For creating test stores, removed FileDagStore
    use icn_governance::GovernanceModule; // For governance tests

    // Helper to create a default in-memory store for tests
    fn new_test_storage() -> Arc<Mutex<dyn StorageService<DagBlock> + Send + Sync>> {
        Arc::new(Mutex::new(InMemoryDagStore::new()))
    }

    // Helper to create a default governance module for tests
    fn new_test_governance_module() -> Arc<Mutex<GovernanceModule>> {
        Arc::new(Mutex::new(GovernanceModule::new()))
    }

    #[test]
    fn get_node_info_works() {
        match get_node_info() {
            Ok(info) => {
                assert_eq!(info.version, ICN_CORE_VERSION);
                assert_eq!(info.name, "ICN Node (Default Name)");
                assert_eq!(info.status_message, "Node is operational");
            }
            Err(_) => panic!("get_node_info returned an error"),
        }
    }

    #[test]
    fn get_node_status_works_when_online() {
        match get_node_status(true) {
            Ok(status) => {
                assert!(status.is_online);
                assert_eq!(status.version, ICN_CORE_VERSION);
                assert_eq!(status.peer_count, 5);
            }
            Err(e) => panic!("get_node_status returned an error when online: {:?}", e),
        }
    }

    #[test]
    fn get_node_status_errs_when_offline() {
        match get_node_status(false) {
            Ok(_) => panic!("get_node_status should have returned an error when offline"),
            Err(CommonError::NodeOffline(msg)) => {
                assert!(msg.contains("simulated offline"));
            }
            Err(e) => panic!("get_node_status returned an unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_submit_and_retrieve_dag_block_api() {
        let storage = new_test_storage();
        let data = b"api test block data for error refinement".to_vec();
        let cid = Cid::new_v1_dummy(0x71, 0x12, &data); // Use more specific data for test CID
        let link_cid = Cid::new_v1_dummy(0x71, 0x12, b"api link for error refinement");
        let link = DagLink { cid: link_cid, name: "apilink_error_refine".to_string(), size: 8 };
        let block = DagBlock {
            cid: cid.clone(),
            data: data.clone(),
            links: vec![link],
        };

        let block_json = serde_json::to_string(&block).unwrap();
        match submit_dag_block(Arc::clone(&storage), block_json.clone()) {
            Ok(submitted_cid) => assert_eq!(submitted_cid, cid),
            Err(e) => panic!("submit_dag_block failed: {:?}", e),
        }

        // Test retrieval of existing block
        let cid_json = serde_json::to_string(&cid).unwrap();
        match retrieve_dag_block(Arc::clone(&storage), cid_json.clone()) {
            Ok(Some(retrieved_block)) => {
                assert_eq!(retrieved_block.cid, cid);
                assert_eq!(retrieved_block.data, data);
            }
            Ok(None) => panic!("Block submitted via API not found (should exist)"),
            Err(e) => panic!("retrieve_dag_block for existing block failed: {:?}", e),
        }

        // Test retrieving non-existent block
        let non_existent_data = b"non-existent-api-error-refine";
        let non_existent_cid = Cid::new_v1_dummy(0x71, 0x12, non_existent_data);
        let non_existent_cid_json = serde_json::to_string(&non_existent_cid).unwrap();
        match retrieve_dag_block(Arc::clone(&storage), non_existent_cid_json) {
            Ok(None) => { /* Expected: block not found, API returns Ok(None) */ }
            Ok(Some(_)) => panic!("Found a non-existent block via API (should be None)"),
            Err(e) => panic!("retrieve_dag_block for non-existent CID failed: {:?}", e),
        }

        // Test invalid JSON for submit_dag_block
        let invalid_block_json = "this is not valid json";
        match submit_dag_block(Arc::clone(&storage), invalid_block_json.to_string()) {
            Err(CommonError::DeserializationError(msg)) => {
                assert!(msg.contains("Failed to parse DagBlock JSON"));
            }
            _ => panic!("submit_dag_block with invalid JSON did not return DeserializationError"),
        }

        // Test invalid JSON for retrieve_dag_block
        let invalid_cid_json = "nor is this";
        match retrieve_dag_block(Arc::clone(&storage), invalid_cid_json.to_string()) {
            Err(CommonError::DeserializationError(msg)) => {
                assert!(msg.contains("Failed to parse CID JSON"));
            }
            _ => panic!("retrieve_dag_block with invalid JSON did not return DeserializationError"),
        }
    }

    // --- Tests for Governance API Functions ---
    #[test]
    fn test_submit_and_get_proposal_api() {
        let gov_module = new_test_governance_module();
        let governance_api = GovernanceApiImpl::new(Arc::clone(&gov_module)); // New instance
        let proposer_did_str = "did:example:proposer123".to_string();

        // Example: SystemParameterChange proposal
        let proposal_input = ProposalInputType::SystemParameterChange {
            param: "max_block_size".to_string(),
            value: "2MB".to_string(),
        };
        let submit_req = GovernanceSubmitProposalRequest {
            proposer_did: proposer_did_str.clone(),
            proposal: proposal_input, // Use the new field and type
            description: "Increase max block size".to_string(),
            duration_secs: 86400 * 7, // 7 days
        };

        let proposal_id = match governance_api.submit_proposal(submit_req) { // Call the trait method
            Ok(id) => id,
            Err(e) => panic!("submit_proposal failed: {:?}", e),
        };
        assert!(!proposal_id.0.is_empty());

        // Test get_proposal
        match governance_api.get_proposal(proposal_id.clone()) { // Call the trait method
            Ok(Some(proposal)) => {
                assert_eq!(proposal.id, proposal_id); // Compare ProposalId directly
                assert_eq!(proposal.proposer.to_string(), proposer_did_str);
                assert_eq!(proposal.description, "Increase max block size");
                if let ProposalType::SystemParameterChange(param, val) = proposal.proposal_type {
                    assert_eq!(param, "max_block_size");
                    assert_eq!(val, "2MB");
                } else {
                    panic!("Incorrect proposal type retrieved: {:?}", proposal.proposal_type);
                }
            }
            Ok(None) => panic!("Proposal not found via get_proposal"),
            Err(e) => panic!("get_proposal failed: {:?}", e),
        }

        // Test list_proposals
        match governance_api.list_proposals() { // Call the trait method
            Ok(proposals) => {
                assert_eq!(proposals.len(), 1);
                assert_eq!(proposals[0].id, proposal_id); // Compare ProposalId
            }
            Err(e) => panic!("list_proposals failed: {:?}", e),
        }
    }

    #[test]
    fn test_cast_vote_api() {
        let gov_module = new_test_governance_module();
        let governance_api = GovernanceApiImpl::new(Arc::clone(&gov_module)); // New instance
        let proposer_did_str = "did:example:proposer_for_vote_test".to_string();
        let voter_did_str = "did:example:voter456".to_string();

        // Setup proposal using the API
        let proposal_input = ProposalInputType::GenericText { text: "A test proposal for voting".to_string() };
        let submit_req = GovernanceSubmitProposalRequest {
            proposer_did: proposer_did_str.clone(),
            proposal: proposal_input,
            description: "Vote test".to_string(),
            duration_secs: 60,
        };
        let proposal_id = governance_api.submit_proposal(submit_req).expect("Failed to submit proposal for vote test");

        let cast_vote_req = GovernanceCastVoteRequest { // Use the renamed struct
            voter_did: voter_did_str.clone(),
            proposal_id: proposal_id.0.clone(), // ProposalId is a tuple struct (String)
            vote_option: "yes".to_string(),
        };

        match governance_api.cast_vote(cast_vote_req) { // Call the trait method
            Ok(_) => { /* Expected */ }
            Err(e) => panic!("cast_vote failed for valid vote: {:?}", e),
        }

        // Verify vote was cast (direct module check is fine)
        let voter_did_core = Did::from_str(&voter_did_str).unwrap();
        let proposal_opt = gov_module.lock().unwrap().get_proposal(&proposal_id).unwrap();
        let proposal = proposal_opt.as_ref().expect("Proposal should exist after voting");
        assert_eq!(proposal.votes.len(), 1);
        assert_eq!(proposal.votes.get(&voter_did_core).unwrap().option, VoteOption::Yes);

        // Test invalid vote option
        let cast_vote_req_invalid = GovernanceCastVoteRequest { // Use the renamed struct
            voter_did: "did:example:voter789".to_string(),
            proposal_id: proposal_id.0.clone(),
            vote_option: "maybe".to_string(),
        };
        match governance_api.cast_vote(cast_vote_req_invalid) { // Call the trait method
            Err(CommonError::InvalidInputError(_)) => { /* Expected */ }
            Ok(_) => panic!("cast_vote should have returned an error for invalid option"),
            Err(e) => panic!("cast_vote returned an unexpected error type for invalid option: {:?}", e),
        }
    }

    // --- Tests for Network API Functions ---
    #[test]
    fn test_discover_peers_api() {
        let bootstrap_nodes = vec!["/ip4/127.0.0.1/tcp/12345/p2p/QmSimulatedPeer".to_string()];
        match discover_peers_api(bootstrap_nodes) {
            Ok(peers) => {
                assert!(!peers.is_empty(), "Expected some peers to be discovered (stubbed)");
                // Check if the stubbed peers are returned, e.g. based on StubNetworkService behavior
                assert!(peers.iter().any(|p| p.0 == "mock_peer_1"));
            }
            Err(e) => panic!("discover_peers_api failed: {:?}", e),
        }
    }

    #[test]
    fn test_send_network_message_api_success() {
        let peer_id_str = "mock_peer_1".to_string();
        // Example NetworkMessage: RequestBlock (ensure Cid and NetworkMessage are serializable)
        let cid_for_message = Cid::new_v1_dummy(0x70, 0x12, b"message_data"); // 0x70 is dag-pb
        let network_msg = NetworkMessage::RequestBlock(cid_for_message);
        let message_json = serde_json::to_string(&network_msg).expect("Failed to serialize NetworkMessage for test");

        match send_network_message_api(peer_id_str.clone(), message_json.clone()) {
            Ok(_) => { /* Success */ }
            Err(e) => panic!("send_network_message_api failed for successful case: {:?}", e),
        }
    }

    #[test]
    fn test_send_network_message_api_peer_not_found() {
        let peer_id_str = "unknown_peer_id".to_string(); // This peer ID causes PeerNotFound in StubNetworkService
        let cid_for_message = Cid::new_v1_dummy(0x70, 0x12, b"message_data_for_unknown");
        let network_msg = NetworkMessage::RequestBlock(cid_for_message);
        let message_json = serde_json::to_string(&network_msg).unwrap();

        match send_network_message_api(peer_id_str, message_json) {
            Err(CommonError::ApiError(api_err_msg)) => {
                // Check if the underlying error from StubNetworkService (PeerNotFound) is encapsulated.
                assert!(api_err_msg.to_lowercase().contains("peer not found") || api_err_msg.contains("PeerNotFound"));
            }
            Ok(_) => panic!("send_network_message_api should have failed for unknown peer"),
            Err(e) => panic!("send_network_message_api returned an unexpected error type for unknown peer: {:?}", e),
        }
    }

    #[test]
    fn test_send_network_message_api_invalid_json() {
        let peer_id_str = "mock_peer_1".to_string();
        let invalid_message_json = "this is not valid json for a network message";

        match send_network_message_api(peer_id_str, invalid_message_json.to_string()) {
            Err(CommonError::DeserializationError(msg)) => {
                assert!(msg.contains("Failed to parse NetworkMessage JSON"));
            }
            _ => panic!("send_network_message_api with invalid JSON did not return DeserializationError"),
        }
    }
}
