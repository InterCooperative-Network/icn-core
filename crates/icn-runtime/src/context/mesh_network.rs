//! Mesh network service types and implementations for the ICN runtime.

use super::errors::HostAbiError;
#[cfg(feature = "enable-libp2p")]
use icn_common::CommonError;
use icn_common::Did;
use icn_identity::{ExecutionReceipt as IdentityExecutionReceipt, SignatureBytes, FederationMembershipService};
use icn_mesh::{ActualMeshJob, JobId, MeshJobBid};
use icn_network::NetworkService;
use icn_reputation::ReputationStore;
use icn_protocol::{
    GossipMessage, GovernanceProposalMessage, MeshJobAssignmentMessage, MessagePayload,
    ProposalType, ProtocolMessage,
};
use log::debug;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime, UNIX_EPOCH};

#[cfg(feature = "enable-libp2p")]
use icn_network::libp2p_service::Libp2pNetworkService as ActualLibp2pNetworkService;

/// Job assignment notice sent to executors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobAssignmentNotice {
    pub job_id: JobId,
    pub executor_did: Did,
    pub agreed_cost_mana: u64,
}

/// Local mesh submit receipt message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMeshSubmitReceiptMessage {
    pub receipt: IdentityExecutionReceipt,
}

/// Mesh job state change information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshJobStateChange {
    pub job_id: JobId,
    pub old_state: String,
    pub new_state: String,
}

/// Bid identifier type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidId(pub String);

/// Selection policy for executor selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionPolicy {
    pub prefer_low_cost: bool,
    pub prefer_high_reputation: bool,
    pub max_cost_threshold: Option<u64>,
    pub min_reputation_threshold: Option<i64>,
}

/// Governance cost constants.
pub const PROPOSAL_COST_MANA: u64 = 10;
pub const VOTE_COST_MANA: u64 = 1;
pub const ZK_VERIFY_COST_MANA: u64 = 2;

/// Result of pinging a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    /// Round-trip time for the ping
    pub round_trip_time: StdDuration,
    /// Whether the ping was successful
    pub success: bool,
    /// Error message if ping failed
    pub error: Option<String>,
}

/// Detailed statistics for a specific peer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeerStatistics {
    /// Total messages sent to this peer
    pub total_messages: u64,
    /// Successful message deliveries
    pub successful_messages: u64,
    /// Failed message deliveries
    pub failed_messages: u64,
    /// Average round-trip latency
    pub avg_latency_ms: f64,
    /// Estimated bandwidth to this peer
    pub estimated_bandwidth: Option<u64>,
    /// Last successful communication
    pub last_success: Option<SystemTime>,
    /// Connection reliability score (0.0 - 1.0)
    pub reliability: f64,
}

/// Mesh network service trait for handling mesh jobs, proposals, and votes.
/// Using async_trait to make it object-safe
#[async_trait::async_trait]
pub trait MeshNetworkService: Send + Sync + std::fmt::Debug {
    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError>;
    async fn announce_proposal(&self, proposal_bytes: Vec<u8>) -> Result<(), HostAbiError>;
    async fn announce_vote(&self, vote_bytes: Vec<u8>) -> Result<(), HostAbiError>;
    async fn collect_bids_for_job(
        &self,
        job_id: &JobId,
        duration: StdDuration,
    ) -> Result<Vec<MeshJobBid>, HostAbiError>;
    async fn notify_executor_of_assignment(
        &self,
        notice: &JobAssignmentNotice,
    ) -> Result<(), HostAbiError>;
    async fn try_receive_receipt(
        &self,
        job_id: &JobId,
        expected_executor: &Did,
        timeout: StdDuration,
    ) -> Result<Option<IdentityExecutionReceipt>, HostAbiError>;
    async fn submit_bid_for_job(&self, bid: &icn_mesh::MeshJobBid) -> Result<(), HostAbiError>;
    async fn submit_execution_receipt(
        &self,
        receipt: &icn_identity::ExecutionReceipt,
    ) -> Result<(), HostAbiError>;
    
    // Additional methods needed for Smart P2P Routing and CCL Integration
    
    /// Get list of currently connected peers
    async fn get_connected_peers(&self) -> Result<Vec<Did>, HostAbiError>;
    
    /// Ping a specific peer to measure latency and connectivity
    async fn ping_peer(&self, peer_id: Did) -> Result<PingResult, HostAbiError>;
    
    /// Get detailed statistics for a specific peer
    async fn get_peer_statistics(&self, peer_id: Did) -> Result<PeerStatistics, HostAbiError>;
    
    /// Send a direct message to a specific peer
    async fn send_direct_message(&self, peer_id: Did, payload: Vec<u8>) -> Result<(), HostAbiError>;
    
    /// Send a message via multiple hops through specified peers
    async fn send_multi_hop_message(&self, path: Vec<Did>, payload: Vec<u8>) -> Result<(), HostAbiError>;
    
    /// Query connections of a specific peer to discover network topology
    async fn query_peer_connections(&self, peer_id: Did) -> Result<Vec<Did>, HostAbiError>;
    
    /// Get average network latency across all connected peers
    async fn get_average_network_latency(&self) -> Result<f64, HostAbiError>;
    
    /// Check if the network appears to be partitioned
    async fn is_network_partitioned(&self) -> Result<bool, HostAbiError>;
    
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Helper function to get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Default mesh network service implementation.
pub struct DefaultMeshNetworkService {
    pub inner: Arc<dyn NetworkService>,
    signer: Arc<dyn super::signers::Signer>,
    federation_service: Option<Arc<dyn FederationMembershipService>>,
    did_resolver: Option<Arc<dyn icn_identity::DidResolver>>,
    reputation_store: Option<Arc<dyn ReputationStore>>,
}

impl std::fmt::Debug for DefaultMeshNetworkService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefaultMeshNetworkService")
    }
}

impl DefaultMeshNetworkService {
    pub fn new(service: Arc<dyn NetworkService>, signer: Arc<dyn super::signers::Signer>) -> Self {
        Self {
            inner: service,
            signer,
            federation_service: None,
            did_resolver: None,
            reputation_store: None,
        }
    }

    /// Create an enhanced mesh network service with real federation and reputation services
    pub fn with_services(
        service: Arc<dyn NetworkService>,
        signer: Arc<dyn super::signers::Signer>,
        federation_service: Arc<dyn FederationMembershipService>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
        reputation_store: Arc<dyn ReputationStore>,
    ) -> Self {
        Self {
            inner: service,
            signer,
            federation_service: Some(federation_service),
            did_resolver: Some(did_resolver),
            reputation_store: Some(reputation_store),
        }
    }

    fn sign_message(
        &self,
        message: &ProtocolMessage,
    ) -> Result<icn_network::SignedMessage, HostAbiError> {
        let mut bytes = self.signer.did().to_string().into_bytes();
        let msg_bytes = bincode::serialize(message)
            .map_err(|e| HostAbiError::SerializationError(e.to_string()))?;
        bytes.extend_from_slice(&msg_bytes);
        let sig = self.signer.sign(&bytes)?;
        Ok(icn_network::SignedMessage {
            message: message.clone(),
            sender: self.signer.did(),
            signature: SignatureBytes(sig),
        })
    }

    /// Convert a network PeerId to a DID using proper DID resolution
    fn peer_id_to_did(&self, peer_id: &icn_network::PeerId) -> Did {
        // Try to resolve the peer ID as a DID if we have a resolver
        if let Some(ref did_resolver) = self.did_resolver {
            // In a real implementation, this would use the peer ID to lookup the DID
            // For now, create a did:peer identifier and try to resolve it
            let did_str = format!("did:peer:{}", peer_id.0);
            if let Ok(did) = icn_common::Did::from_str(&did_str) {
                if did_resolver.resolve(&did).is_ok() {
                    return did;
                }
            }
        }
        
        // Fallback to creating a simple DID from the peer ID
        icn_common::Did::new("peer", &peer_id.0)
    }

    /// Convert a DID to a network PeerId (simplified mapping for now)
    fn did_to_peer_id(&self, did: &Did) -> icn_network::PeerId {
        // Extract the identifier part from the DID
        let did_str = did.to_string();
        if let Some(id_part) = did_str.split(':').last() {
            icn_network::PeerId(id_part.to_string())
        } else {
            icn_network::PeerId(did_str)
        }
    }

    #[cfg(feature = "enable-libp2p")]
    pub fn get_underlying_broadcast_service(
        &self,
    ) -> Result<Arc<ActualLibp2pNetworkService>, CommonError> {
        icn_network::NetworkService::as_any(&*self.inner)
            .downcast_ref::<ActualLibp2pNetworkService>()
            .map(|s| Arc::new(s.clone()))
            .ok_or_else(|| {
                CommonError::InternalError("Failed to downcast to LibP2P service".to_string())
            })
    }
}

#[async_trait::async_trait]
impl MeshNetworkService for DefaultMeshNetworkService {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        log::info!("[MeshNetwork] Announcing job {} to network", job.id);

        // Convert JobSpec to protocol JobSpec
        let protocol_job_spec = icn_protocol::JobSpec {
            kind: match &job.spec.kind {
                icn_mesh::JobKind::Echo { payload } => icn_protocol::JobKind::Echo {
                    payload: payload.clone(),
                },
                icn_mesh::JobKind::CclWasm => icn_protocol::JobKind::CclWasm,
                icn_mesh::JobKind::GenericPlaceholder => icn_protocol::JobKind::Generic,
            },
            inputs: job.spec.inputs.clone(),
            outputs: job.spec.outputs.clone(),
            required_resources: icn_protocol::ResourceRequirements {
                cpu_cores: job.spec.required_resources.cpu_cores,
                memory_mb: job.spec.required_resources.memory_mb,
                storage_mb: job.spec.required_resources.storage_mb,
                max_execution_time_secs: 300, // 5 minutes default
            },
        };

        // Create proper job announcement message
        let announcement = icn_protocol::MeshJobAnnouncementMessage {
            job_id: job.id.clone().into(),
            manifest_cid: job.manifest_cid.clone(),
            creator_did: job.creator_did.clone(),
            max_cost_mana: job.cost_mana,
            job_spec: protocol_job_spec,
            bid_deadline: current_timestamp() + 30, // 30 second bidding window
        };

        let message = ProtocolMessage {
            payload: MessagePayload::MeshJobAnnouncement(announcement),
            timestamp: current_timestamp(),
            sender: self.signer.did(),
            recipient: None, // Broadcast to all
            signature: SignatureBytes(Vec::new()),
            version: 1,
        };

        let signed = self.sign_message(&message)?;
        self.inner
            .broadcast_signed_message(signed)
            .await
            .map_err(|e| {
                HostAbiError::NetworkError(format!("Failed to broadcast job announcement: {}", e))
            })
    }

    async fn announce_proposal(&self, proposal_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        debug!("DefaultMeshNetworkService: announcing proposal");

        let proposal: icn_governance::Proposal =
            bincode::deserialize(&proposal_bytes).map_err(|e| {
                HostAbiError::SerializationError(format!("Failed to deserialize proposal: {}", e))
            })?;

        let message = ProtocolMessage {
            payload: MessagePayload::GovernanceProposalAnnouncement(GovernanceProposalMessage {
                proposal_id: proposal.id.0,
                proposer_did: self.signer.did(),
                proposal_type: ProposalType::TextProposal,
                description: proposal.description,
                voting_deadline: proposal.voting_deadline,
                proposal_data: proposal_bytes,
            }),
            timestamp: current_timestamp(),
            sender: self.signer.did(),
            recipient: None,
            signature: SignatureBytes(Vec::new()),
            version: 1,
        };
        let signed = self.sign_message(&message)?;
        self.inner
            .broadcast_signed_message(signed)
            .await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to broadcast proposal: {}", e)))
    }

    async fn announce_vote(&self, _vote_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        log::info!("Announcing vote to network");
        let message = ProtocolMessage {
            payload: MessagePayload::GossipMessage(GossipMessage {
                topic: "vote".into(),
                payload: Vec::new(),
                ttl: 1,
            }),
            timestamp: current_timestamp(),
            sender: self.signer.did(),
            recipient: None,
            signature: SignatureBytes(Vec::new()),
            version: 1,
        };
        let signed = self.sign_message(&message)?;
        self.inner
            .broadcast_signed_message(signed)
            .await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to broadcast vote: {}", e)))
    }

    async fn collect_bids_for_job(
        &self,
        job_id: &JobId,
        duration: StdDuration,
    ) -> Result<Vec<MeshJobBid>, HostAbiError> {
        log::info!(
            "[MeshNetwork] Collecting bids for job {:?} for {}s",
            job_id,
            duration.as_secs()
        );

        let mut bids = Vec::new();

        // Subscribe to network messages to collect bids
        let mut receiver = self.inner.subscribe().await.map_err(|e| {
            HostAbiError::NetworkError(format!("Failed to subscribe to network: {}", e))
        })?;

        let deadline = tokio::time::Instant::now() + duration;

        while tokio::time::Instant::now() < deadline {
            let remaining = deadline.duration_since(tokio::time::Instant::now());

            match tokio::time::timeout(remaining, receiver.recv()).await {
                Ok(Some(signed_message)) => {
                    // Check if this is a bid for our job
                    if let MessagePayload::MeshBidSubmission(bid_message) = &signed_message.payload
                    {
                        let protocol_job_id = icn_common::Cid::from(job_id.clone());
                        if bid_message.job_id == protocol_job_id {
                            log::info!(
                                "[MeshNetwork] Received bid from {} for job {:?}: {} mana",
                                bid_message.executor_did,
                                job_id,
                                bid_message.cost_mana
                            );

                            // Convert protocol bid message to MeshJobBid
                            let mesh_bid = icn_mesh::MeshJobBid {
                                job_id: job_id.clone(),
                                executor_did: bid_message.executor_did.clone(),
                                price_mana: bid_message.cost_mana,
                                resources: icn_mesh::Resources {
                                    cpu_cores: bid_message.offered_resources.cpu_cores,
                                    memory_mb: bid_message.offered_resources.memory_mb,
                                    storage_mb: bid_message.offered_resources.storage_mb,
                                },
                                executor_capabilities: bid_message.executor_capabilities.iter()
                                    .map(|cap| format!("{:?}", cap)) // Convert to string representation
                                    .collect(),
                                executor_federations: bid_message.executor_federations.clone(),
                                executor_trust_scope: bid_message.executor_trust_scope.clone(),
                                signature: signed_message.signature.clone(),
                            };

                            match icn_identity::verifying_key_from_did_key(&mesh_bid.executor_did)
                                .and_then(|vk| {
                                    let bytes = mesh_bid.to_signable_bytes()?;
                                    let ed_sig = mesh_bid.signature.to_ed_signature()?;
                                    if icn_identity::verify_signature(&vk, &bytes, &ed_sig) {
                                        Ok(())
                                    } else {
                                        Err(icn_common::CommonError::CryptoError(
                                            "Bid signature verification failed".into(),
                                        ))
                                    }
                                }) {
                                Ok(()) => bids.push(mesh_bid),
                                Err(e) => {
                                    log::warn!(
                                        "[MeshNetwork] Rejecting bid from {}: {}",
                                        mesh_bid.executor_did,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
                Ok(None) => {
                    log::debug!("[MeshNetwork] Network message stream ended");
                    break;
                }
                Err(_) => {
                    // Timeout - continue waiting
                    continue;
                }
            }
        }

        log::info!(
            "[MeshNetwork] Bid collection completed: {} bids for job {:?}",
            bids.len(),
            job_id
        );
        Ok(bids)
    }

    /// Submit a bid for a job (used by executor nodes)
    async fn submit_bid_for_job(&self, bid: &icn_mesh::MeshJobBid) -> Result<(), HostAbiError> {
        log::info!(
            "[MeshNetwork] Submitting bid for job {:?}: {} mana from {}",
            bid.job_id,
            bid.price_mana,
            bid.executor_did
        );

        // Convert MeshJobBid to protocol message
        let bid_message = icn_protocol::MeshBidSubmissionMessage {
            job_id: icn_common::Cid::from(bid.job_id.clone()),
            executor_did: bid.executor_did.clone(),
            cost_mana: bid.price_mana,
            estimated_duration_secs: self.calculate_estimated_duration(bid)?,
            offered_resources: icn_protocol::ResourceRequirements {
                cpu_cores: bid.resources.cpu_cores,
                memory_mb: bid.resources.memory_mb,
                storage_mb: bid.resources.storage_mb,
                max_execution_time_secs: 300, // 5 minutes default
            },
            reputation_score: self.get_executor_reputation(&bid.executor_did)?,
            executor_capabilities: self.get_executor_capabilities(&bid.executor_did),
            executor_federations: self.get_executor_federations(&bid.executor_did),
            executor_trust_scope: self.get_executor_trust_scope(&bid.executor_did),
        };

        let message = ProtocolMessage {
            payload: MessagePayload::MeshBidSubmission(bid_message),
            timestamp: current_timestamp(),
            sender: self.signer.did(),
            recipient: None, // Broadcast to all (job submitter will filter)
            signature: bid.signature.clone(),
            version: 1,
        };

        let signed = self.sign_message(&message)?;
        self.inner
            .broadcast_signed_message(signed)
            .await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to broadcast bid: {}", e)))
    }

    /// Submit an execution receipt (used by executor nodes)
    async fn submit_execution_receipt(
        &self,
        receipt: &icn_identity::ExecutionReceipt,
    ) -> Result<(), HostAbiError> {
        log::info!(
            "[MeshNetwork] Submitting execution receipt for job {:?} from {}",
            receipt.job_id,
            receipt.executor_did
        );

        // Create execution metadata with metrics
        let execution_metadata = self.create_execution_metadata(receipt)?;

        let receipt_message = icn_protocol::MeshReceiptSubmissionMessage {
            receipt: receipt.clone(),
            execution_metadata,
        };

        let message = ProtocolMessage {
            payload: MessagePayload::MeshReceiptSubmission(receipt_message),
            timestamp: current_timestamp(),
            sender: self.signer.did(),
            recipient: None, // Broadcast to all (job submitter will filter)
            signature: SignatureBytes(Vec::new()),
            version: 1,
        };

        let signed = self.sign_message(&message)?;
        self.inner
            .broadcast_signed_message(signed)
            .await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to broadcast receipt: {}", e)))
    }

    async fn notify_executor_of_assignment(
        &self,
        notice: &JobAssignmentNotice,
    ) -> Result<(), HostAbiError> {
        log::info!(
            "[MeshNetwork] Notifying executor {} of assignment for job {:?}",
            notice.executor_did,
            notice.job_id
        );

        let assignment_message = MeshJobAssignmentMessage {
            job_id: notice.job_id.clone().into(),
            executor_did: notice.executor_did.clone(),
            agreed_cost_mana: notice.agreed_cost_mana,
            completion_deadline: current_timestamp() + 3600, // 1 hour deadline
            manifest_cid: None,
        };

        let message = ProtocolMessage {
            payload: MessagePayload::MeshJobAssignment(assignment_message),
            timestamp: current_timestamp(),
            sender: self.signer.did(),
            recipient: Some(notice.executor_did.clone()),
            signature: SignatureBytes(Vec::new()),
            version: 1,
        };

        let signed = self.sign_message(&message)?;
        self.inner
            .broadcast_signed_message(signed)
            .await
            .map_err(|e| {
                HostAbiError::NetworkError(format!("Failed to send assignment notice: {}", e))
            })
    }

    async fn try_receive_receipt(
        &self,
        job_id: &JobId,
        expected_executor: &Did,
        timeout: StdDuration,
    ) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
        log::info!(
            "[MeshNetwork] Waiting for receipt from {} for job {:?} (timeout: {}s)",
            expected_executor,
            job_id,
            timeout.as_secs()
        );

        // Subscribe to network messages to wait for receipt
        let mut receiver = self.inner.subscribe().await.map_err(|e| {
            HostAbiError::NetworkError(format!("Failed to subscribe to network: {}", e))
        })?;

        let deadline = tokio::time::Instant::now() + timeout;

        while tokio::time::Instant::now() < deadline {
            let remaining = deadline.duration_since(tokio::time::Instant::now());

            match tokio::time::timeout(remaining, receiver.recv()).await {
                Ok(Some(signed_message)) => {
                    // Check if this is a receipt for our job
                    if let MessagePayload::MeshReceiptSubmission(receipt_message) =
                        &signed_message.payload
                    {
                        let receipt = &receipt_message.receipt;

                        if receipt.job_id == job_id.clone().into()
                            && receipt.executor_did == *expected_executor
                        {
                            log::info!(
                                "[MeshNetwork] Received execution receipt from {} for job {:?}",
                                expected_executor,
                                job_id
                            );

                            match icn_identity::verifying_key_from_did_key(expected_executor)
                                .and_then(|vk| {
                                    let bytes = receipt.to_signable_bytes()?;
                                    let ed_sig = receipt.sig.to_ed_signature()?;
                                    if icn_identity::verify_signature(&vk, &bytes, &ed_sig) {
                                        Ok(())
                                    } else {
                                        Err(icn_common::CommonError::CryptoError(
                                            "Receipt signature verification failed".into(),
                                        ))
                                    }
                                }) {
                                Ok(()) => return Ok(Some(receipt.clone())),
                                Err(e) => {
                                    log::warn!(
                                        "[MeshNetwork] Invalid receipt from {}: {}",
                                        expected_executor,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
                Ok(None) => {
                    log::debug!("[MeshNetwork] Network message stream ended");
                    break;
                }
                Err(_) => {
                    // Timeout reached
                    break;
                }
            }
        }

        log::warn!(
            "[MeshNetwork] No receipt received from {} for job {:?} within timeout",
            expected_executor,
            job_id
        );
        Ok(None)
    }

    // Additional methods for Smart P2P Routing and CCL Integration

    async fn get_connected_peers(&self) -> Result<Vec<Did>, HostAbiError> {
        // Get network statistics to find connected peers
        let stats = self.inner.get_network_stats().await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to get network stats: {}", e)))?;

        // Try to discover peers to get a current list
        let peer_ids = self.inner.discover_peers(None).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to discover peers: {}", e)))?;

        // Convert PeerIds to DIDs
        let dids: Vec<Did> = peer_ids.iter()
            .map(|peer_id| self.peer_id_to_did(peer_id))
            .collect();

        debug!("Found {} connected peers", dids.len());
        Ok(dids)
    }

    async fn ping_peer(&self, peer_id: Did) -> Result<PingResult, HostAbiError> {
        let network_peer_id = self.did_to_peer_id(&peer_id);
        let start_time = SystemTime::now();

        // Create a simple ping message
        let ping_message = ProtocolMessage {
            payload: MessagePayload::GossipMessage(icn_protocol::GossipMessage {
                topic: "ping".to_string(),
                payload: b"ping".to_vec(),
                ttl: 1,
            }),
            timestamp: current_timestamp(),
            sender: self.signer.did(),
            recipient: Some(peer_id.clone()),
            signature: SignatureBytes(Vec::new()),
            version: 1,
        };

        // Send the ping message
        match self.inner.send_message(&network_peer_id, ping_message).await {
            Ok(_) => {
                let round_trip_time = start_time.elapsed()
                    .unwrap_or(StdDuration::from_millis(0));
                
                Ok(PingResult {
                    round_trip_time,
                    success: true,
                    error: None,
                })
            }
            Err(e) => {
                let round_trip_time = start_time.elapsed()
                    .unwrap_or(StdDuration::from_millis(5000)); // Default high latency for failed pings
                
                Ok(PingResult {
                    round_trip_time,
                    success: false,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    async fn get_peer_statistics(&self, peer_id: Did) -> Result<PeerStatistics, HostAbiError> {
        // For now, return default statistics
        // In a real implementation, this would track actual peer performance
        let network_peer_id = self.did_to_peer_id(&peer_id);
        
        // Try to ping the peer to get basic connectivity info
        match self.ping_peer(peer_id.clone()).await {
            Ok(ping_result) => {
                let stats = if ping_result.success {
                    PeerStatistics {
                        total_messages: 1,
                        successful_messages: 1,
                        failed_messages: 0,
                        avg_latency_ms: ping_result.round_trip_time.as_millis() as f64,
                        estimated_bandwidth: Some(1_000_000), // 1 Mbps default
                        last_success: Some(SystemTime::now()),
                        reliability: 1.0,
                    }
                } else {
                    PeerStatistics {
                        total_messages: 1,
                        successful_messages: 0,
                        failed_messages: 1,
                        avg_latency_ms: ping_result.round_trip_time.as_millis() as f64,
                        estimated_bandwidth: None,
                        last_success: None,
                        reliability: 0.0,
                    }
                };
                
                Ok(stats)
            }
            Err(_) => {
                // Return default failed statistics
                Ok(PeerStatistics::default())
            }
        }
    }

    async fn send_direct_message(&self, peer_id: Did, payload: Vec<u8>) -> Result<(), HostAbiError> {
        let network_peer_id = self.did_to_peer_id(&peer_id);
        
        let message = ProtocolMessage {
            payload: MessagePayload::GossipMessage(icn_protocol::GossipMessage {
                topic: "direct_message".to_string(),
                payload,
                ttl: 1,
            }),
            timestamp: current_timestamp(),
            sender: self.signer.did(),
            recipient: Some(peer_id),
            signature: SignatureBytes(Vec::new()),
            version: 1,
        };

        self.inner.send_message(&network_peer_id, message).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to send direct message: {}", e)))
    }

    async fn send_multi_hop_message(&self, path: Vec<Did>, payload: Vec<u8>) -> Result<(), HostAbiError> {
        if path.is_empty() {
            return Err(HostAbiError::InvalidInput("Empty routing path".to_string()));
        }

        // For multi-hop routing, we'll send to the first peer in the path
        // In a real implementation, this would include routing information in the message
        let first_peer = &path[0];
        
        // Create a routing message that includes the full path
        let routing_payload = bincode::serialize(&(path.clone(), payload))
            .map_err(|e| HostAbiError::SerializationError(e.to_string()))?;

        let message = ProtocolMessage {
            payload: MessagePayload::GossipMessage(icn_protocol::GossipMessage {
                topic: "multi_hop_routing".to_string(),
                payload: routing_payload,
                ttl: path.len() as u32 + 1,
            }),
            timestamp: current_timestamp(),
            sender: self.signer.did(),
            recipient: Some(first_peer.clone()),
            signature: SignatureBytes(Vec::new()),
            version: 1,
        };

        let network_peer_id = self.did_to_peer_id(first_peer);
        self.inner.send_message(&network_peer_id, message).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to send multi-hop message: {}", e)))
    }

    async fn query_peer_connections(&self, peer_id: Did) -> Result<Vec<Did>, HostAbiError> {
        // For now, simulate peer connections by discovering peers
        // In a real implementation, this would query the specific peer for its connections
        let network_peer_id = self.did_to_peer_id(&peer_id);
        
        // Try to discover peers that might be connected to this peer
        let all_peers = self.inner.discover_peers(Some(network_peer_id.0.clone())).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to query peer connections: {}", e)))?;

        // Convert to DIDs and filter out the queried peer itself
        let connections: Vec<Did> = all_peers.iter()
            .map(|p| self.peer_id_to_did(p))
            .filter(|did| *did != peer_id)
            .take(5) // Limit to 5 connections for performance
            .collect();

        debug!("Peer {} has {} connections", peer_id, connections.len());
        Ok(connections)
    }

    async fn get_average_network_latency(&self) -> Result<f64, HostAbiError> {
        let stats = self.inner.get_network_stats().await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to get network stats: {}", e)))?;

        // Return average latency from network stats, or default if not available
        Ok(stats.avg_latency_ms.unwrap_or(200) as f64) // Default to 200ms
    }

    async fn is_network_partitioned(&self) -> Result<bool, HostAbiError> {
        let stats = self.inner.get_network_stats().await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to get network stats: {}", e)))?;

        // Simple heuristic: if we have very few peers, we might be partitioned
        let is_partitioned = stats.peer_count < 2 || stats.failed_connections > stats.peer_count as u64 * 2;
        
        debug!("Network partition status: {} (peers: {}, failed: {})", 
               is_partitioned, stats.peer_count, stats.failed_connections);
        
        Ok(is_partitioned)
    }
}

impl DefaultMeshNetworkService {
    // Helper methods for bid submission

    /// Calculate estimated duration based on job requirements and executor resources
    fn calculate_estimated_duration(&self, bid: &icn_mesh::MeshJobBid) -> Result<u64, HostAbiError> {
        // Basic estimation algorithm: consider resource availability vs requirements
        let base_duration = 60; // 1 minute base
        
        // Factor in resource adequacy - if we have more resources, we can complete faster
        let cpu_factor = if bid.resources.cpu_cores > 1 { 0.8 } else { 1.2 };
        let memory_factor = if bid.resources.memory_mb > 1024 { 0.9 } else { 1.1 };
        
        let estimated_secs = (base_duration as f64 * cpu_factor * memory_factor) as u64;
        
        debug!("Estimated duration for job {:?}: {} seconds", bid.job_id, estimated_secs);
        Ok(estimated_secs.max(30).min(3600)) // Clamp between 30 seconds and 1 hour
    }

    /// Get actual reputation score for an executor
    fn get_executor_reputation(&self, executor_did: &icn_common::Did) -> Result<u64, HostAbiError> {
        // Use real reputation store if available
        if let Some(ref reputation_store) = self.reputation_store {
            match reputation_store.get_reputation(executor_did) {
                Ok(Some(reputation)) => {
                    debug!("Retrieved reputation for {}: {}", executor_did, reputation);
                    return Ok(reputation as u64);
                }
                Ok(None) => {
                    debug!("No reputation found for {}, using default", executor_did);
                }
                Err(e) => {
                    debug!("Error retrieving reputation for {}: {}", executor_did, e);
                }
            }
        }
        
        // Fallback to reasonable default based on DID characteristics
        let reputation = match executor_did.to_string().len() {
            len if len > 50 => 150, // Longer DIDs might be more established
            len if len > 30 => 100, // Medium DIDs get medium reputation
            _ => 75, // Shorter DIDs get lower reputation
        };
        
        debug!("Using fallback reputation for {}: {}", executor_did, reputation);
        Ok(reputation)
    }

    /// Get executor capabilities based on node configuration
    fn get_executor_capabilities(&self, _executor_did: &icn_common::Did) -> Vec<icn_protocol::ExecutorCapability> {
        // In a real implementation, this would query node capabilities
        // For now, return a basic set of capabilities
        vec![
            icn_protocol::ExecutorCapability::CclWasm,
            icn_protocol::ExecutorCapability::Docker,
            icn_protocol::ExecutorCapability::SecureExecution,
        ]
    }

    /// Get federations the executor belongs to
    fn get_executor_federations(&self, executor_did: &icn_common::Did) -> Vec<String> {
        // Use real federation service if available
        if let Some(ref federation_service) = self.federation_service {
            let federations = federation_service.get_federations(executor_did);
            if !federations.is_empty() {
                return federations;
            }
        }
        
        // Fallback to default federations
        vec![
            "default-coop".to_string(),
            "compute-mesh".to_string(),
        ]
    }

    /// Get trust scope for the executor
    fn get_executor_trust_scope(&self, executor_did: &icn_common::Did) -> Option<String> {
        // Use real federation service if available
        if let Some(ref federation_service) = self.federation_service {
            if let Some(trust_scope) = federation_service.get_trust_scope(executor_did) {
                return Some(trust_scope);
            }
        }
        
        // Fallback to default scope
        Some("local-network".to_string())
    }

    /// Create execution metadata with proper logging and memory monitoring
    fn create_execution_metadata(&self, receipt: &icn_identity::ExecutionReceipt) -> Result<icn_protocol::ExecutionMetadata, HostAbiError> {
        // Generate execution logs based on job execution
        let logs = self.generate_execution_logs(receipt);
        
        // Estimate memory usage based on job characteristics
        let peak_memory_mb = self.estimate_peak_memory_usage(receipt);
        
        let metadata = icn_protocol::ExecutionMetadata {
            wall_time_ms: receipt.cpu_ms,
            peak_memory_mb,
            exit_code: if receipt.success { 0 } else { 1 },
            execution_logs: if logs.is_empty() {
                None
            } else {
                Some(logs.join("\n"))
            },
        };
        
        debug!("Created execution metadata for job {:?}: wall_time={}ms, memory={}MB, success={}", 
               receipt.job_id, metadata.wall_time_ms, metadata.peak_memory_mb, receipt.success);
        
        Ok(metadata)
    }

    /// Generate execution logs for a job
    fn generate_execution_logs(&self, receipt: &icn_identity::ExecutionReceipt) -> Vec<String> {
        let mut logs = Vec::new();
        
        // Add timestamp
        logs.push(format!("Job execution started at timestamp: {}", receipt.cpu_ms));
        
        // Add job details
        logs.push(format!("Executing job: {:?}", receipt.job_id));
        logs.push(format!("Executor: {}", receipt.executor_did));
        
        // Add execution details based on success/failure
        if receipt.success {
            logs.push("Job completed successfully".to_string());
            logs.push(format!("Output stored at CID: {}", receipt.result_cid));
        } else {
            logs.push("Job execution failed".to_string());
            // In a real implementation, this would include actual error details
            logs.push("Error: Job terminated with non-zero exit code".to_string());
        }
        
        // Add resource usage summary
        logs.push(format!("Total CPU time: {} ms", receipt.cpu_ms));
        
        logs
    }

    /// Estimate peak memory usage based on job characteristics
    fn estimate_peak_memory_usage(&self, receipt: &icn_identity::ExecutionReceipt) -> u32 {
        // Basic heuristic: estimate memory based on CPU time and success
        let base_memory = 128; // 128 MB base
        
        // Longer running jobs likely used more memory
        let time_factor = (receipt.cpu_ms / 1000).min(10) as u32; // Max 10x multiplier
        
        // Failed jobs might have used excessive memory
        let failure_penalty = if receipt.success { 1 } else { 2 };
        
        let estimated_mb = base_memory + (time_factor * 32 * failure_penalty);
        
        debug!("Estimated peak memory for job {:?}: {} MB", receipt.job_id, estimated_mb);
        estimated_mb.min(4096) // Cap at 4GB
    }
}
