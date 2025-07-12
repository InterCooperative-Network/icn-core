#![doc = include_str!("../README.md")]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::default_constructed_unit_structs)]
#![allow(clippy::get_first)]

//! # ICN API Crate
//! This crate provides the primary API endpoints for interacting with InterCooperative Network (ICN) nodes.
//! It defines service interfaces, data structures for requests and responses, and potentially server/client implementations.
//! The API aims for clarity, modularity, and extensibility, typically using JSON-RPC or gRPC.

// Depending on icn_common crate
use icn_common::{
    compute_merkle_cid, retry_with_backoff, Cid, CircuitBreaker, CircuitBreakerError, CommonError,
    DagBlock, Did, NodeInfo, NodeStatus, SystemTimeProvider, ICN_CORE_VERSION,
};
// Remove direct use of icn_dag::put_block and icn_dag::get_block which use global store
// use icn_dag::{put_block as dag_put_block, get_block as dag_get_block};
use icn_dag::AsyncStorageService; // Import the async storage trait
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex}; // To accept the storage service
use tokio::sync::Mutex as AsyncMutex;
// Added imports for network functionality
use icn_network::{NetworkService, PeerId};

use icn_protocol::ProtocolMessage;
// Added imports for governance functionality
use icn_governance::{
    scoped_policy::{DagPayloadOp, PolicyCheckResult, ScopedPolicyEnforcer},
    GovernanceModule, Proposal, ProposalId, ProposalType, VoteOption,
};
use std::str::FromStr;
use std::time::Duration;

static HTTP_BREAKER: Lazy<AsyncMutex<CircuitBreaker<SystemTimeProvider>>> = Lazy::new(|| {
    AsyncMutex::new(CircuitBreaker::new(
        SystemTimeProvider,
        3,
        Duration::from_secs(5),
    ))
});

pub mod dag_trait;
pub mod federation_trait;
pub mod governance_trait;
/// Prometheus metrics helpers
pub mod metrics;
use crate::dag_trait::DagApi;
use crate::governance_trait::{
    CastVoteRequest as GovernanceCastVoteRequest, // Renamed to avoid conflict
    GovernanceApi,
    ProposalInputType,
    SubmitProposalRequest as GovernanceSubmitProposalRequest, // Renamed to avoid conflict
};

/// Planned: Define a trait for the ICN API service for RPC implementation.
// pub trait IcnApiService {
//    async fn get_node_info(&self) -> Result<NodeInfo, CommonError>;
//    async fn get_node_status(&self) -> Result<NodeStatus, CommonError>;
//    async fn submit_dag_block(&self, block: DagBlock) -> Result<Cid, CommonError>;
//    async fn retrieve_dag_block(&self, cid: Cid) -> Result<Option<DagBlock>, CommonError>;
/// Submits a generic [`Transaction`] to the network.
///
/// At this early stage the function simply validates that the JSON
/// represents a well formed `Transaction` and returns its ID.
/// A real implementation would persist the transaction and broadcast it
/// across the network.
pub fn submit_transaction(tx_json: String) -> Result<String, CommonError> {
    let tx: icn_common::Transaction = serde_json::from_str(&tx_json).map_err(|e| {
        CommonError::DeserializationError(format!(
            "Failed to parse Transaction JSON: {} (Input: '{}')",
            e, tx_json
        ))
    })?;

    if tx.id.is_empty() {
        return Err(CommonError::InvalidInputError(
            "Transaction id cannot be empty".to_string(),
        ));
    }

    Ok(tx.id)
}

/// Queries data from the provided DAG store by CID.
///
/// Returns the [`DagBlock`] if found, or `Ok(None)` if the block does not exist.
pub async fn query_data(
    storage: Arc<tokio::sync::Mutex<dyn AsyncStorageService<DagBlock> + Send>>,
    cid_json: String,
) -> Result<Option<DagBlock>, CommonError> {
    let cid: Cid = serde_json::from_str(&cid_json).map_err(|e| {
        CommonError::DeserializationError(format!(
            "Failed to parse CID JSON for query: {} (Input: '{}')",
            e, cid_json
        ))
    })?;

    let store_guard = storage.lock().await;
    store_guard.get(&cid).await.map_err(|e| match e {
        CommonError::StorageError(msg) => {
            CommonError::StorageError(format!("API: Failed to query DagBlock: {}", msg))
        }
        CommonError::PolicyDenied(msg) => CommonError::PolicyDenied(format!("API: {}", msg)),
        other => CommonError::ApiError(format!("API: Unexpected error: {:?}", other)),
    })
}
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
        return Err(CommonError::NodeOffline(
            "Node is currently simulated offline.".to_string(),
        ));
    }

    // In a real scenario, these values would be fetched from the node's internal state.
    Ok(NodeStatus {
        is_online: true,
        peer_count: 5,              // Example value
        current_block_height: 1000, // Example value
        version: ICN_CORE_VERSION.to_string(),
    })
}

/// Submits a DagBlock to the provided DAG store.
/// Returns the CID of the stored block upon success.
pub async fn submit_dag_block(
    storage: Arc<tokio::sync::Mutex<dyn AsyncStorageService<DagBlock> + Send>>,
    block_data_json: String,
    policy_enforcer: Option<Arc<dyn ScopedPolicyEnforcer>>,
    actor: Did,
) -> Result<Cid, CommonError> {
    let block: DagBlock = serde_json::from_str(&block_data_json).map_err(|e| {
        CommonError::DeserializationError(format!(
            "Failed to parse DagBlock JSON for submission: {} (Input: '{}')",
            e, block_data_json
        ))
    })?;

    let expected_cid = compute_merkle_cid(
        block.cid.codec,
        &block.data,
        &block.links,
        block.timestamp,
        &block.author_did,
        &block.signature,
        &block.scope,
    );
    if expected_cid != block.cid {
        return Err(CommonError::DagValidationError("CID mismatch".to_string()));
    }

    if let Some(enforcer) = &policy_enforcer {
        if let PolicyCheckResult::Denied { reason } =
            enforcer.check_permission(DagPayloadOp::SubmitBlock, &actor, block.scope.as_ref())
        {
            return Err(CommonError::PolicyDenied(reason));
        }
    }

    let mut store = storage.lock().await;

    store.put(&block).await.map_err(|e| match e {
        CommonError::StorageError(msg) => {
            CommonError::StorageError(format!("API: Failed to store DagBlock: {}", msg))
        }
        CommonError::DagValidationError(msg) => {
            CommonError::DagValidationError(format!("API: Invalid DagBlock: {}", msg))
        }
        CommonError::SerializationError(msg) => {
            CommonError::SerializationError(format!("API: Serialization error during put: {}", msg))
        }
        CommonError::DeserializationError(msg) => CommonError::DeserializationError(format!(
            "API: Deserialization error during put: {}",
            msg
        )),
        CommonError::PolicyDenied(msg) => CommonError::PolicyDenied(format!("API: {}", msg)),
        _ => CommonError::ApiError(format!("API: Unexpected error during store.put: {:?}", e)),
    })?;
    Ok(block.cid.clone())
}

/// Retrieves a DagBlock from the provided DAG store by its CID.
/// The result is an Option<DagBlock> to indicate if the block was found.
pub async fn retrieve_dag_block(
    storage: Arc<tokio::sync::Mutex<dyn AsyncStorageService<DagBlock> + Send>>,
    cid_json: String,
) -> Result<Option<DagBlock>, CommonError> {
    let cid: Cid = serde_json::from_str(&cid_json).map_err(|e| {
        CommonError::DeserializationError(format!(
            "Failed to parse CID JSON for retrieval: {} (Input: '{}')",
            e, cid_json
        ))
    })?;

    let store = storage.lock().await;

    store.get(&cid).await.map_err(|e| match e {
        CommonError::StorageError(msg) => {
            CommonError::StorageError(format!("API: Failed to retrieve DagBlock: {}", msg))
        }
        CommonError::DeserializationError(msg) => CommonError::DeserializationError(format!(
            "API: Deserialization error during get: {}",
            msg
        )),
        CommonError::PolicyDenied(msg) => CommonError::PolicyDenied(format!("API: {}", msg)),
        // Note: get typically shouldn't cause SerializationError or DagValidationError unless the store is corrupted
        _ => CommonError::ApiError(format!("API: Unexpected error during store.get: {:?}", e)),
    })
}

/// Retrieve [`DagBlock`] metadata by CID.
pub async fn get_dag_metadata(
    storage: Arc<tokio::sync::Mutex<dyn AsyncStorageService<DagBlock> + Send>>,
    cid_json: String,
) -> Result<Option<icn_dag::DagBlockMetadata>, CommonError> {
    let cid: Cid = serde_json::from_str(&cid_json).map_err(|e| {
        CommonError::DeserializationError(format!(
            "Failed to parse CID JSON for metadata retrieval: {} (Input: '{}')",
            e, cid_json
        ))
    })?;

    let store = storage.lock().await;
    let block_opt = store.get(&cid).await.map_err(|e| match e {
        CommonError::StorageError(msg) => {
            CommonError::StorageError(format!("API: Failed to retrieve DagBlock: {}", msg))
        }
        CommonError::DeserializationError(msg) => CommonError::DeserializationError(format!(
            "API: Deserialization error during get: {}",
            msg
        )),
        CommonError::PolicyDenied(msg) => CommonError::PolicyDenied(format!("API: {}", msg)),
        _ => CommonError::ApiError(format!("API: Unexpected error during store.get: {:?}", e)),
    })?;

    Ok(block_opt.map(|b| icn_dag::metadata_from_block(&b)))
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
    /// Create a new [`GovernanceApiImpl`] wrapping the given [`GovernanceModule`].
    ///
    /// This helper is used by higher layers (e.g. RPC) to access governance
    /// functionality through the [`GovernanceApi`] trait.
    pub fn new(gov_module: Arc<Mutex<GovernanceModule>>) -> Self {
        Self { gov_module }
    }
}

impl GovernanceApi for GovernanceApiImpl {
    fn submit_proposal(
        &self,
        request: GovernanceSubmitProposalRequest,
    ) -> Result<ProposalId, CommonError> {
        let proposer_did = Did::from_str(&request.proposer_did).map_err(|e| {
            CommonError::InvalidInputError(format!(
                "Invalid proposer_did format: {}. Error: {:?}",
                request.proposer_did, e
            ))
        })?;

        let core_proposal_type = match request.proposal {
            ProposalInputType::SystemParameterChange { param, value } => {
                ProposalType::SystemParameterChange(param, value)
            }
            ProposalInputType::MemberAdmission { did } => {
                let member_did = Did::from_str(&did).map_err(|e| {
                    CommonError::InvalidInputError(format!(
                        "Invalid member DID format for admission: {}. Error: {:?}",
                        did, e
                    ))
                })?;
                ProposalType::NewMemberInvitation(member_did)
            }
            ProposalInputType::RemoveMember { did } => {
                let member_did = Did::from_str(&did).map_err(|e| {
                    CommonError::InvalidInputError(format!(
                        "Invalid member DID format for removal: {}. Error: {:?}",
                        did, e
                    ))
                })?;
                ProposalType::RemoveMember(member_did)
            }
            ProposalInputType::SoftwareUpgrade { version } => {
                ProposalType::SoftwareUpgrade(version)
            }
            ProposalInputType::GenericText { text } => ProposalType::GenericText(text),
        };

        let mut module = self.gov_module.lock().map_err(|_e| {
            CommonError::ApiError(
                "Failed to lock governance module for submitting proposal".to_string(),
            )
        })?;

        module.submit_proposal(icn_governance::ProposalSubmission {
            proposer: proposer_did,
            proposal_type: core_proposal_type,
            description: request.description,
            duration_secs: request.duration_secs,
            quorum: request.quorum,
            threshold: request.threshold,
            content_cid: None,
        })
    }

    fn cast_vote(&self, request: GovernanceCastVoteRequest) -> Result<(), CommonError> {
        let voter_did = Did::from_str(&request.voter_did).map_err(|e| {
            CommonError::InvalidInputError(format!(
                "Invalid voter_did format: {}. Error: {:?}",
                request.voter_did, e
            ))
        })?;

        let proposal_id = ProposalId::from_str(&request.proposal_id).map_err(|e| {
            CommonError::InvalidInputError(format!(
                "Invalid proposal_id format: {}. Error: {:?}",
                request.proposal_id, e
            ))
        })?;

        let vote_option = match request.vote_option.to_lowercase().as_str() {
            "yes" => VoteOption::Yes,
            "no" => VoteOption::No,
            "abstain" => VoteOption::Abstain,
            _ => {
                return Err(CommonError::InvalidInputError(format!(
                    "Invalid vote option: {}. Must be one of 'yes', 'no', 'abstain'.",
                    request.vote_option
                )))
            }
        };

        let mut module = self.gov_module.lock().map_err(|_e| {
            CommonError::ApiError("Failed to lock governance module for casting vote".to_string())
        })?;
        module.cast_vote(voter_did, &proposal_id, vote_option)
    }

    fn get_proposal(&self, id: ProposalId) -> Result<Option<Proposal>, CommonError> {
        let module = self.gov_module.lock().map_err(|_e| {
            CommonError::ApiError(
                "Failed to lock governance module for getting proposal".to_string(),
            )
        })?;
        module.get_proposal(&id)
    }

    fn list_proposals(&self) -> Result<Vec<Proposal>, CommonError> {
        let module = self.gov_module.lock().map_err(|_e| {
            CommonError::ApiError(
                "Failed to lock governance module for listing proposals".to_string(),
            )
        })?;
        module.list_proposals()
    }
}

/// Concrete implementation for DAG API operations.
pub struct DagApiImpl<S> {
    pub store: Arc<AsyncMutex<S>>,
}

impl<S> DagApiImpl<S> {
    pub fn new(store: Arc<AsyncMutex<S>>) -> Self {
        Self { store }
    }
}

#[async_trait::async_trait]
impl<S> dag_trait::DagApi for DagApiImpl<S>
where
    S: AsyncStorageService<DagBlock> + Send,
{
    async fn get_dag_root(&self) -> Result<Option<Cid>, CommonError> {
        let store = self.store.lock().await;
        icn_dag::current_root(&*store).await
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

/// API endpoint to discover network peers. The caller supplies a [`NetworkService`]
/// implementation, allowing either the default stub or a real libp2p service.
/// `bootstrap_nodes_str` are optional bootstrap addresses for discovery.
pub async fn discover_peers_api(
    network_service: Arc<dyn NetworkService>,
    bootstrap_nodes_str: Vec<String>,
) -> Result<Vec<PeerId>, CommonError> {
    // In a real scenario, bootstrap_nodes_str might need parsing into a more specific type.
    // For discover_peers, we might want to pass a single optional peer, or handle multiple if the underlying service supports it.
    // For now, let's take the first bootstrap node as an example if provided, or None.
    let discovery_param: Option<String> = bootstrap_nodes_str.get(0).cloned();
    network_service
        .discover_peers(discovery_param)
        .await
        .map_err(|e| {
            CommonError::ApiError(format!(
                "Failed to discover peers via network service: {:?}",
                e
            ))
        })
}

/// API endpoint to send a message to a specific peer. The caller supplies a [`NetworkService`]
/// implementation (stub or libp2p).
/// `peer_id_str` is the string representation of the target [`PeerId`].
/// `message_json` is a JSON string representation of the [`ProtocolMessage`].
pub async fn send_network_message_api(
    network_service: Arc<dyn NetworkService>,
    peer_id_str: String,
    message_json: String,
) -> Result<(), CommonError> {
    let peer_id = PeerId(peer_id_str); // Assuming PeerId is a simple wrapper around String for now.

    // Deserialize the ProtocolMessage from JSON.
    // This requires [`ProtocolMessage`] to implement `Deserialize`.
    let message: ProtocolMessage = serde_json::from_str(&message_json).map_err(|e| {
        CommonError::DeserializationError(format!(
            "Failed to parse ProtocolMessage JSON: {}. Input: {}",
            e, message_json
        ))
    })?;

    network_service
        .send_message(&peer_id, message)
        .await
        .map_err(|e| {
            CommonError::ApiError(format!(
                "Failed to send message via network service: {:?}",
                e
            ))
        })
}

/// Retrieve the local peer ID from an ICN node via HTTP.
pub async fn http_get_local_peer_id(api_url: &str) -> Result<String, CommonError> {
    let url = format!("{}/network/local-peer-id", api_url.trim_end_matches('/'));
    use std::time::Duration;
    let res = {
        let breaker = HTTP_BREAKER.lock().await;
        breaker
            .call(|| async {
                retry_with_backoff(
                    || async {
                        reqwest::get(&url).await.map_err(|e| {
                            CommonError::ApiError(format!("Failed to send request: {}", e))
                        })
                    },
                    3,
                    Duration::from_millis(100),
                    Duration::from_secs(2),
                )
                .await
            })
            .await
    };
    let res = res.map_err(|e| match e {
        CircuitBreakerError::Open => CommonError::NetworkUnhealthy("circuit open".to_string()),
        CircuitBreakerError::Inner(err) => err,
    })?;
    if res.status().is_success() {
        res.json::<serde_json::Value>()
            .await
            .map_err(|e| CommonError::DeserializationError(e.to_string()))
            .and_then(|v| {
                v.get("peer_id")
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        CommonError::DeserializationError("Missing peer_id field".to_string())
                    })
            })
    } else {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        Err(CommonError::ApiError(format!(
            "Request failed {}: {}",
            status, text
        )))
    }
}

/// Retrieve the list of peers from an ICN node via HTTP.
pub async fn http_get_peer_list(api_url: &str) -> Result<Vec<String>, CommonError> {
    let url = format!("{}/network/peers", api_url.trim_end_matches('/'));
    use std::time::Duration;
    let res = {
        let breaker = HTTP_BREAKER.lock().await;
        breaker
            .call(|| async {
                retry_with_backoff(
                    || async {
                        reqwest::get(&url).await.map_err(|e| {
                            CommonError::ApiError(format!("Failed to send request: {}", e))
                        })
                    },
                    3,
                    Duration::from_millis(100),
                    Duration::from_secs(2),
                )
                .await
            })
            .await
    };
    let res = res.map_err(|e| match e {
        CircuitBreakerError::Open => CommonError::NetworkUnhealthy("circuit open".to_string()),
        CircuitBreakerError::Inner(err) => err,
    })?;
    if res.status().is_success() {
        res.json::<Vec<String>>()
            .await
            .map_err(|e| CommonError::DeserializationError(e.to_string()))
    } else {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        Err(CommonError::ApiError(format!(
            "Request failed {}: {}",
            status, text
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::DagLink; // For test setup
    use icn_dag::TokioFileDagStore; // Async file-based store for tests
    use icn_governance::GovernanceModule; // For governance tests
    use icn_network::StubNetworkService; // For network tests
    use icn_protocol::{DagBlockRequestMessage, GossipMessage, MessagePayload, ProtocolMessage};
    use tempfile::tempdir;

    // Helper to create a default in-memory store for tests
    fn new_test_storage() -> Arc<tokio::sync::Mutex<dyn AsyncStorageService<DagBlock> + Send>> {
        let dir = tempdir().unwrap();
        Arc::new(tokio::sync::Mutex::new(
            TokioFileDagStore::new(dir.keep()).unwrap(),
        ))
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

    #[tokio::test]
    async fn submit_dag_block_valid_cid() {
        let storage = new_test_storage();
        let child_cid = Cid::new_v1_sha256(0x71, b"child data");
        let link = DagLink {
            cid: child_cid,
            name: "child".to_string(),
            size: 9,
        };
        let data = b"parent".to_vec();
        let ts = 0u64;
        let author = Did::new("key", "tester");
        let sig_opt = None;
        let cid = super::compute_merkle_cid(
            0x71,
            &data,
            std::slice::from_ref(&link),
            ts,
            &author,
            &sig_opt,
            &None,
        );
        let block = DagBlock {
            cid: cid.clone(),
            data: data.clone(),
            links: vec![link],
            timestamp: ts,
            author_did: author,
            signature: sig_opt,
            scope: None,
        };
        let block_json = serde_json::to_string(&block).unwrap();
        let result = submit_dag_block(storage, block_json, None, block.author_did.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), cid);
    }

    #[tokio::test]
    async fn submit_dag_block_invalid_cid() {
        let storage = new_test_storage();
        let child_cid = Cid::new_v1_sha256(0x71, b"child data");
        let link = DagLink {
            cid: child_cid,
            name: "child".to_string(),
            size: 9,
        };
        let data = b"parent".to_vec();
        let wrong_cid = Cid::new_v1_sha256(0x71, b"wrong");
        let block = DagBlock {
            cid: wrong_cid,
            data,
            links: vec![link],
            timestamp: 0,
            author_did: Did::new("key", "tester"),
            signature: None,
            scope: None,
        };
        let block_json = serde_json::to_string(&block).unwrap();
        let result = submit_dag_block(storage, block_json, None, block.author_did.clone()).await;
        match result {
            Err(CommonError::DagValidationError(_)) => {}
            other => panic!("expected DagValidationError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_submit_and_retrieve_dag_block_api() {
        let storage = new_test_storage();
        let data = b"api test block data for error refinement".to_vec();
        let cid = Cid::new_v1_sha256(0x71, &data); // Use more specific data for test CID
        let link_cid = Cid::new_v1_sha256(0x71, b"api link for error refinement");
        let link = DagLink {
            cid: link_cid,
            name: "apilink_error_refine".to_string(),
            size: 8,
        };
        let ts = 0u64;
        let author = Did::new("key", "tester");
        let sig_opt = None;
        let block = DagBlock {
            cid: cid.clone(),
            data: data.clone(),
            links: vec![link],
            timestamp: ts,
            author_did: author,
            signature: sig_opt,
            scope: None,
        };

        let block_json = serde_json::to_string(&block).unwrap();
        match submit_dag_block(
            Arc::clone(&storage),
            block_json.clone(),
            None,
            block.author_did.clone(),
        )
        .await
        {
            Ok(submitted_cid) => assert_eq!(submitted_cid, cid),
            Err(e) => panic!("submit_dag_block failed: {:?}", e),
        }

        // Test retrieval of existing block
        let cid_json = serde_json::to_string(&cid).unwrap();
        match retrieve_dag_block(Arc::clone(&storage), cid_json.clone()).await {
            Ok(Some(retrieved_block)) => {
                assert_eq!(retrieved_block.cid, cid);
                assert_eq!(retrieved_block.data, data);
            }
            Ok(None) => panic!("Block submitted via API not found (should exist)"),
            Err(e) => panic!("retrieve_dag_block for existing block failed: {:?}", e),
        }

        // Test retrieving non-existent block
        let non_existent_data = b"non-existent-api-error-refine";
        let non_existent_cid = Cid::new_v1_sha256(0x71, non_existent_data);
        let non_existent_cid_json = serde_json::to_string(&non_existent_cid).unwrap();
        match retrieve_dag_block(Arc::clone(&storage), non_existent_cid_json).await {
            Ok(None) => { /* Expected: block not found, API returns Ok(None) */ }
            Ok(Some(_)) => panic!("Found a non-existent block via API (should be None)"),
            Err(e) => panic!("retrieve_dag_block for non-existent CID failed: {:?}", e),
        }

        // Test invalid JSON for submit_dag_block
        let invalid_block_json = "this is not valid json";
        match submit_dag_block(
            Arc::clone(&storage),
            invalid_block_json.to_string(),
            None,
            Did::new("key", "tester"),
        )
        .await
        {
            Err(CommonError::DeserializationError(msg)) => {
                assert!(msg.contains("Failed to parse DagBlock JSON"));
            }
            _ => panic!("submit_dag_block with invalid JSON did not return DeserializationError"),
        }

        // Test invalid JSON for retrieve_dag_block
        let invalid_cid_json = "nor is this";
        match retrieve_dag_block(Arc::clone(&storage), invalid_cid_json.to_string()).await {
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
        let api = GovernanceApiImpl::new(gov_module.clone());

        let submit_req = GovernanceSubmitProposalRequest {
            proposer_did: "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8".to_string(),
            proposal: ProposalInputType::GenericText {
                text: "A simple text proposal".to_string(),
            },
            description: "Test description".to_string(),
            duration_secs: 3600,
            quorum: None,
            threshold: None,
            body: None,
        };

        let proposal_id_res = api.submit_proposal(submit_req);
        assert!(
            proposal_id_res.is_ok(),
            "submit_proposal failed: {:?}",
            proposal_id_res.err()
        );
        let proposal_id = proposal_id_res.unwrap();

        let retrieved_proposal_res = api.get_proposal(proposal_id.clone());
        assert!(
            retrieved_proposal_res.is_ok(),
            "get_proposal failed: {:?}",
            retrieved_proposal_res.err()
        );
        let retrieved_proposal_opt = retrieved_proposal_res.unwrap();
        assert!(
            retrieved_proposal_opt.is_some(),
            "Proposal not found after submission"
        );
        let retrieved_proposal = retrieved_proposal_opt.unwrap();
        assert_eq!(retrieved_proposal.id, proposal_id);
        assert!(
            matches!(retrieved_proposal.proposal_type, ProposalType::GenericText(s) if s == "A simple text proposal")
        );
    }

    #[test]
    fn test_cast_vote_api() {
        let gov_module = new_test_governance_module();
        let api = GovernanceApiImpl::new(gov_module.clone());
        let proposer_did_str = "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8";

        let submit_req = GovernanceSubmitProposalRequest {
            proposer_did: proposer_did_str.to_string(),
            proposal: ProposalInputType::GenericText {
                text: "Proposal to vote on".to_string(),
            },
            description: "Voting test".to_string(),
            duration_secs: 3600,
            quorum: None,
            threshold: None,
            body: None,
        };
        let proposal_id = api
            .submit_proposal(submit_req)
            .expect("Submitting proposal for vote test failed");

        let voter_did_str = "did:key:z6MkjchhcVbWZkAbNGRsM4ac3gR3eNnYtD9tYtFv9T9xL4xH";
        let cast_vote_req = GovernanceCastVoteRequest {
            voter_did: voter_did_str.to_string(),
            proposal_id: proposal_id.0.clone(), // ProposalId(String) -> String
            vote_option: "yes".to_string(),
        };

        let vote_res = api.cast_vote(cast_vote_req);
        assert!(vote_res.is_ok(), "cast_vote failed: {:?}", vote_res.err());
    }

    // --- Tests for Network API Functions ---
    #[tokio::test]
    async fn test_discover_peers_api() {
        // Made async
        let bootstrap_nodes = vec!["/ip4/127.0.0.1/tcp/12345/p2p/QmSimulatedPeer".to_string()];
        let service = Arc::new(StubNetworkService::default()) as Arc<dyn NetworkService>;
        match discover_peers_api(service, bootstrap_nodes).await {
            Ok(peers) => {
                assert!(
                    !peers.is_empty(),
                    "Expected some peers to be discovered (stubbed)"
                );
                // Further assertions can be made if StubNetworkService returns predictable peers
            }
            Err(e) => panic!("discover_peers_api failed: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_send_network_message_api_success() {
        // Made async
        let peer_id_str = "test_peer_123".to_string();
        // Using GossipSub as a generic message type for the test, as Ping variant doesn't exist.
        let message_to_send = ProtocolMessage::new(
            MessagePayload::GossipMessage(GossipMessage {
                topic: "test_topic".to_string(),
                payload: b"hello world".to_vec(),
                ttl: 1,
            }),
            Did::new("key", "tester"),
            None,
        );
        let message_json = serde_json::to_string(&message_to_send).unwrap();

        let service = Arc::new(StubNetworkService::default()) as Arc<dyn NetworkService>;
        let result = send_network_message_api(service, peer_id_str, message_json).await;
        assert!(
            result.is_ok(),
            "send_network_message_api failed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_send_network_message_api_peer_not_found() {
        // Made async
        let peer_id_str = "unknown_peer_id".to_string(); // StubNetworkService simulates error for this peer
        let dummy_cid = Cid::new_v1_sha256(0x55, b"test_cid_for_req_block");
        let message_to_send = ProtocolMessage::new(
            MessagePayload::DagBlockRequest(DagBlockRequestMessage {
                block_cid: dummy_cid,
                priority: 0,
            }),
            Did::new("key", "tester"),
            None,
        );
        let message_json = serde_json::to_string(&message_to_send).unwrap();

        let service = Arc::new(StubNetworkService::default()) as Arc<dyn NetworkService>;
        let result = send_network_message_api(service, peer_id_str, message_json).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            CommonError::ApiError(msg) => assert!(
                msg.contains("Peer with ID unknown_peer_id not found")
                    || msg.contains("Failed to send message to peer: unknown_peer_id")
            ),
            other_err => panic!("Expected ApiError for peer not found, got {:?}", other_err),
        }
    }

    #[tokio::test]
    async fn test_send_network_message_api_invalid_json() {
        // Made async
        let peer_id_str = "QmTestPeerInvalidJson".to_string();
        let invalid_message_json = "this is not valid json for a network message";

        let service = Arc::new(StubNetworkService::default()) as Arc<dyn NetworkService>;
        match send_network_message_api(service, peer_id_str, invalid_message_json.to_string()).await
        {
            Err(CommonError::DeserializationError(msg)) => {
                assert!(msg.contains("Failed to parse ProtocolMessage JSON"));
            }
            Ok(_) => panic!("send_network_message_api should have failed for invalid JSON input"),
            Err(e) => panic!(
                "send_network_message_api with invalid JSON returned an unexpected error: {:?}",
                e
            ),
        }
    }

    #[test]
    fn test_submit_transaction() {
        let tx = icn_common::Transaction {
            id: "tx1".to_string(),
            timestamp: 1,
            sender_did: Did::new("key", "alice"),
            recipient_did: None,
            payload_type: "test".to_string(),
            payload: b"hello".to_vec(),
            nonce: 0,
            mana_limit: 100,
            mana_price: 1,
            signature: None,
        };
        let tx_json = serde_json::to_string(&tx).unwrap();
        let res = submit_transaction(tx_json);
        assert_eq!(res.unwrap(), "tx1".to_string());
    }

    #[tokio::test]
    async fn test_query_data() {
        use icn_dag::TokioFileDagStore;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let store: Arc<tokio::sync::Mutex<dyn AsyncStorageService<DagBlock> + Send>> = Arc::new(
            tokio::sync::Mutex::new(TokioFileDagStore::new(dir.keep()).unwrap()),
        );
        let data = b"query block".to_vec();
        let ts = 0u64;
        let author = Did::new("key", "tester");
        let sig_opt = None;
        let cid = compute_merkle_cid(0x71, &data, &[], ts, &author, &sig_opt, &None);
        let block = DagBlock {
            cid: cid.clone(),
            data: data.clone(),
            links: vec![],
            timestamp: ts,
            author_did: author,
            signature: sig_opt,
            scope: None,
        };
        {
            let mut guard = store.lock().await;
            guard.put(&block).await.unwrap();
        }

        let cid_json = serde_json::to_string(&cid).unwrap();
        let res = query_data(store, cid_json).await.unwrap().unwrap();
        assert_eq!(res.data, data);
    }

    struct DenyAllStore;

    #[async_trait::async_trait]
    impl AsyncStorageService<DagBlock> for DenyAllStore {
        async fn put(&mut self, _block: &DagBlock) -> Result<(), CommonError> {
            Err(CommonError::PolicyDenied("put blocked".to_string()))
        }

        async fn get(&self, _cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
            Err(CommonError::PolicyDenied("get blocked".to_string()))
        }

        async fn delete(&mut self, _cid: &Cid) -> Result<(), CommonError> {
            Err(CommonError::PolicyDenied("delete blocked".to_string()))
        }

        async fn contains(&self, _cid: &Cid) -> Result<bool, CommonError> {
            Err(CommonError::PolicyDenied("contains blocked".to_string()))
        }

        async fn list_blocks(&self) -> Result<Vec<DagBlock>, CommonError> {
            Err(CommonError::PolicyDenied("list blocked".to_string()))
        }

        async fn pin_block(&mut self, _cid: &Cid) -> Result<(), CommonError> {
            Err(CommonError::PolicyDenied("pin blocked".to_string()))
        }

        async fn unpin_block(&mut self, _cid: &Cid) -> Result<(), CommonError> {
            Err(CommonError::PolicyDenied("unpin blocked".to_string()))
        }

        async fn prune_expired(&mut self, _now: u64) -> Result<Vec<Cid>, CommonError> {
            Err(CommonError::PolicyDenied("prune blocked".to_string()))
        }

        async fn set_ttl(&mut self, _cid: &Cid, _ttl: Option<u64>) -> Result<(), CommonError> {
            Err(CommonError::PolicyDenied("set_ttl blocked".to_string()))
        }

        async fn get_metadata(
            &self,
            _cid: &Cid,
        ) -> Result<Option<icn_dag::BlockMetadata>, CommonError> {
            Err(CommonError::PolicyDenied(
                "get_metadata blocked".to_string(),
            ))
        }

        fn as_any(&self) -> &(dyn std::any::Any + 'static) {
            self
        }

        fn as_any_mut(&mut self) -> &mut (dyn std::any::Any + 'static) {
            self
        }
    }

    #[tokio::test]
    async fn policy_error_propagates_on_put() {
        let store: Arc<tokio::sync::Mutex<dyn AsyncStorageService<DagBlock> + Send>> =
            Arc::new(tokio::sync::Mutex::new(DenyAllStore));

        let data = b"block".to_vec();
        let ts = 0u64;
        let author = Did::new("key", "tester");
        let sig_opt = None;
        let cid = compute_merkle_cid(0x71, &data, &[], ts, &author, &sig_opt, &None);
        let block = DagBlock {
            cid: cid.clone(),
            data,
            links: vec![],
            timestamp: ts,
            author_did: author,
            signature: sig_opt,
            scope: None,
        };
        let block_json = serde_json::to_string(&block).unwrap();
        match submit_dag_block(store, block_json, None, block.author_did.clone()).await {
            Err(CommonError::PolicyDenied(msg)) => assert!(msg.contains("blocked")),
            other => panic!("Expected PolicyDenied, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn policy_error_propagates_on_get() {
        let store: Arc<tokio::sync::Mutex<dyn AsyncStorageService<DagBlock> + Send>> =
            Arc::new(tokio::sync::Mutex::new(DenyAllStore));

        let cid = Cid::new_v1_sha256(0x71, b"a");
        let cid_json = serde_json::to_string(&cid).unwrap();
        match retrieve_dag_block(store, cid_json).await {
            Err(CommonError::PolicyDenied(msg)) => assert!(msg.contains("blocked")),
            other => panic!("Expected PolicyDenied, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn policy_error_propagates_on_query() {
        let store: Arc<tokio::sync::Mutex<dyn AsyncStorageService<DagBlock> + Send>> =
            Arc::new(tokio::sync::Mutex::new(DenyAllStore));

        let cid = Cid::new_v1_sha256(0x71, b"a");
        let cid_json = serde_json::to_string(&cid).unwrap();
        match query_data(store, cid_json).await {
            Err(CommonError::PolicyDenied(msg)) => assert!(msg.contains("blocked")),
            other => panic!("Expected PolicyDenied, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn submit_dag_block_allowed_by_policy() {
        use icn_governance::scoped_policy::InMemoryPolicyEnforcer;
        use std::collections::{HashMap, HashSet};

        let store = new_test_storage();
        let actor = Did::new("key", "allowed");
        let mut submitters = HashSet::new();
        submitters.insert(actor.clone());
        let enforcer = InMemoryPolicyEnforcer::new(submitters, HashSet::new(), HashMap::new());

        let data = b"block".to_vec();
        let ts = 0u64;
        let sig_opt = None;
        let cid = compute_merkle_cid(0x71, &data, &[], ts, &actor, &sig_opt, &None);
        let block = DagBlock {
            cid: cid.clone(),
            data,
            links: vec![],
            timestamp: ts,
            author_did: actor.clone(),
            signature: sig_opt,
            scope: None,
        };

        let block_json = serde_json::to_string(&block).unwrap();
        let res = submit_dag_block(
            Arc::clone(&store),
            block_json,
            Some(Arc::new(enforcer)),
            actor.clone(),
        )
        .await;

        assert_eq!(res.unwrap(), cid);
    }

    #[tokio::test]
    async fn submit_dag_block_denied_by_policy() {
        use icn_governance::scoped_policy::InMemoryPolicyEnforcer;
        use std::collections::{HashMap, HashSet};

        let store = new_test_storage();
        let actor = Did::new("key", "denied");
        let submitters = HashSet::new();
        let enforcer = InMemoryPolicyEnforcer::new(submitters, HashSet::new(), HashMap::new());

        let data = b"block".to_vec();
        let ts = 0u64;
        let sig_opt = None;
        let cid = compute_merkle_cid(0x71, &data, &[], ts, &actor, &sig_opt, &None);
        let block = DagBlock {
            cid,
            data,
            links: vec![],
            timestamp: ts,
            author_did: actor.clone(),
            signature: sig_opt,
            scope: None,
        };
        let block_json = serde_json::to_string(&block).unwrap();

        match submit_dag_block(
            Arc::clone(&store),
            block_json,
            Some(Arc::new(enforcer)),
            actor.clone(),
        )
        .await
        {
            Err(CommonError::PolicyDenied(msg)) => {
                assert!(msg.contains("not authorized"));
            }
            other => panic!("expected PolicyDenied, got {:?}", other),
        }
    }
}
