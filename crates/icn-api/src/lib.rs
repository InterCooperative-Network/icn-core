#![doc = include_str!("../README.md")]

//! # ICN API Crate
//! This crate provides the primary API endpoints for interacting with InterCooperative Network (ICN) nodes.
//! It defines service interfaces, data structures for requests and responses, and potentially server/client implementations.
//! The API aims for clarity, modularity, and extensibility, typically using JSON-RPC or gRPC.

// Depending on icn_common crate
use icn_common::{NodeInfo, NodeStatus, CommonError, ICN_CORE_VERSION, DagBlock, Cid};
// Remove direct use of icn_dag::put_block and icn_dag::get_block which use global store
// use icn_dag::{put_block as dag_put_block, get_block as dag_get_block};
use icn_dag::StorageService; // Import the trait
use std::sync::{Arc, Mutex}; // To accept the storage service
// Added imports for network functionality
use icn_network::{PeerId, NetworkMessage, NetworkService, StubNetworkService};
// Added imports for governance functionality
use icn_governance::{GovernanceModule, ProposalId, ProposalType, VoteOption, Proposal, Vote};
use icn_common::Did; // Ensure Did is in scope

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

// Structs for API request/response, if different from core governance types or for JSON convenience
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SubmitProposalRequest {
    pub proposer_did: String, // DID as string
    pub proposal_type_json: serde_json::Value, // Flexible for different proposal types
    pub description: String,
    pub duration_secs: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CastVoteRequest {
    pub voter_did: String, // DID as string
    pub proposal_id: String,
    pub vote_option: String, // "yes", "no", "abstain"
}

/// API endpoint to submit a new governance proposal.
pub fn submit_proposal_api(
    gov_module: Arc<Mutex<GovernanceModule>>,
    request_json: String,
) -> Result<String, CommonError> { // Returns ProposalId as String
    let request: SubmitProposalRequest = serde_json::from_str(&request_json)
        .map_err(|e| CommonError::DeserializationError(format!("Failed to parse SubmitProposalRequest JSON: {}", e)))?;

    let proposer_did = Did(request.proposer_did);
    
    // Deserialize ProposalType from request.proposal_type_json
    // This is a bit manual; a more robust solution might involve a tagged enum for ProposalType on the API boundary
    let proposal_type: ProposalType = serde_json::from_value(request.proposal_type_json.clone()).map_err(|e| CommonError::DeserializationError(format!("Failed to parse ProposalType from JSON value {:?}: {}", request.proposal_type_json, e)))?;
    
    let mut module = gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for submitting proposal".to_string()))?;
    
    let proposal_id = module.submit_proposal(proposer_did, proposal_type, request.description, request.duration_secs)?;
    Ok(proposal_id.0)
}

/// API endpoint to cast a vote on a proposal.
pub fn cast_vote_api(
    gov_module: Arc<Mutex<GovernanceModule>>,
    request_json: String,
) -> Result<(), CommonError> {
    let request: CastVoteRequest = serde_json::from_str(&request_json)
        .map_err(|e| CommonError::DeserializationError(format!("Failed to parse CastVoteRequest JSON: {}", e)))?;

    let voter_did = Did(request.voter_did);
    let proposal_id = ProposalId(request.proposal_id);
    let vote_option = match request.vote_option.to_lowercase().as_str() {
        "yes" => VoteOption::Yes,
        "no" => VoteOption::No,
        "abstain" => VoteOption::Abstain,
        _ => return Err(CommonError::InvalidInputError(format!("Invalid vote option: {}. Must be one of 'yes', 'no', 'abstain'.", request.vote_option))),
    };

    let mut module = gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for casting vote".to_string()))?;
    module.cast_vote(voter_did, &proposal_id, vote_option)
}

/// API endpoint to get a specific proposal by its ID.
pub fn get_proposal_api(
    gov_module: Arc<Mutex<GovernanceModule>>,
    proposal_id_json: String, // Proposal ID as a JSON string
) -> Result<Option<Proposal>, CommonError> { // Returns the full Proposal struct (needs to be serializable)
    let proposal_id_str: String = serde_json::from_str(&proposal_id_json)
        .map_err(|e| CommonError::DeserializationError(format!("Failed to parse Proposal ID JSON: {}", e)))?;
    let proposal_id = ProposalId(proposal_id_str);

    let module = gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for getting proposal".to_string()))?;
    Ok(module.get_proposal(&proposal_id).cloned()) // Clone to return an owned Proposal
}

/// API endpoint to list all current proposals.
pub fn list_proposals_api(
    gov_module: Arc<Mutex<GovernanceModule>>,
) -> Result<Vec<Proposal>, CommonError> { // Returns a list of full Proposal structs
    let module = gov_module.lock().map_err(|_e| CommonError::ApiError("Failed to lock governance module for listing proposals".to_string()))?;
    Ok(module.list_proposals().into_iter().cloned().collect()) // Clone to return owned Proposals
}

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
    use icn_dag::{InMemoryDagStore, FileDagStore}; // For creating test stores
    use tempfile::tempdir; // For FileDagStore tests
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
        let proposer_did_str = "did:example:proposer123".to_string();

        // Example: SystemParameterChange proposal
        let proposal_type_json = serde_json::json!({
            "SystemParameterChange": ["max_block_size", "2MB"]
        });
        let submit_req = SubmitProposalRequest {
            proposer_did: proposer_did_str.clone(),
            proposal_type_json,
            description: "Increase max block size".to_string(),
            duration_secs: 86400 * 7, // 7 days
        };
        let submit_req_json = serde_json::to_string(&submit_req).unwrap();

        let proposal_id_str = match submit_proposal_api(Arc::clone(&gov_module), submit_req_json) {
            Ok(id) => id,
            Err(e) => panic!("submit_proposal_api failed: {:?}", e),
        };
        assert!(!proposal_id_str.is_empty());

        // Test get_proposal_api
        let proposal_id_json = serde_json::to_string(&proposal_id_str).unwrap();
        match get_proposal_api(Arc::clone(&gov_module), proposal_id_json) {
            Ok(Some(proposal)) => {
                assert_eq!(proposal.id.0, proposal_id_str);
                assert_eq!(proposal.proposer.0, proposer_did_str);
                assert_eq!(proposal.description, "Increase max block size");
                if let ProposalType::SystemParameterChange(param, val) = proposal.proposal_type {
                    assert_eq!(param, "max_block_size");
                    assert_eq!(val, "2MB");
                } else {
                    panic!("Incorrect proposal type retrieved");
                }
            }
            Ok(None) => panic!("Proposal not found via get_proposal_api"),
            Err(e) => panic!("get_proposal_api failed: {:?}", e),
        }

        // Test list_proposals_api
        match list_proposals_api(Arc::clone(&gov_module)) {
            Ok(proposals) => {
                assert_eq!(proposals.len(), 1);
                assert_eq!(proposals[0].id.0, proposal_id_str);
            }
            Err(e) => panic!("list_proposals_api failed: {:?}", e),
        }
    }

    #[test]
    fn test_cast_vote_api() {
        let gov_module = new_test_governance_module();
        let proposer_did = Did("did:example:proposer_for_vote_test".to_string());
        let voter_did_str = "did:example:voter456".to_string();

        let proposal_type = ProposalType::GenericText("A test proposal for voting".to_string());
        let proposal_id = gov_module.lock().unwrap().submit_proposal(proposer_did, proposal_type, "Vote test".to_string(), 60).unwrap();

        let cast_vote_req = CastVoteRequest {
            voter_did: voter_did_str.clone(),
            proposal_id: proposal_id.0.clone(),
            vote_option: "yes".to_string(),
        };
        let cast_vote_req_json = serde_json::to_string(&cast_vote_req).unwrap();

        match cast_vote_api(Arc::clone(&gov_module), cast_vote_req_json) {
            Ok(_) => { /* Expected */ }
            Err(e) => panic!("cast_vote_api failed for valid vote: {:?}", e),
        }

        // Verify vote was cast
        let proposal = gov_module.lock().unwrap().get_proposal(&proposal_id).unwrap().clone();
        assert_eq!(proposal.votes.len(), 1);
        assert_eq!(proposal.votes.get(&Did(voter_did_str)).unwrap().option, VoteOption::Yes);

        // Test invalid vote option
        let cast_vote_req_invalid = CastVoteRequest {
            voter_did: "did:example:voter789".to_string(),
            proposal_id: proposal_id.0.clone(),
            vote_option: "maybe".to_string(),
        };
        let cast_vote_req_invalid_json = serde_json::to_string(&cast_vote_req_invalid).unwrap();
        match cast_vote_api(Arc::clone(&gov_module), cast_vote_req_invalid_json) {
            Err(CommonError::InvalidInputError(_)) => { /* Expected */ }
            _ => panic!("cast_vote_api did not return InvalidInputError for invalid option"),
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
