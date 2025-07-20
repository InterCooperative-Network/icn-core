//! Mesh network service types and implementations for the ICN runtime.

use super::errors::HostAbiError;
#[cfg(feature = "enable-libp2p")]
use icn_common::CommonError;
use icn_common::Did;
use icn_identity::{ExecutionReceipt as IdentityExecutionReceipt, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, MeshJobBid};
use icn_network::NetworkService;
use icn_protocol::{
    GossipMessage, GovernanceProposalMessage, MeshJobAssignmentMessage, MessagePayload,
    ProposalType, ProtocolMessage,
};
use log::debug;
use serde::{Deserialize, Serialize};
// use std::str::FromStr; // Not needed currently
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
                                executor_capabilities: vec![], // TODO: Extract from bid message
                                executor_federations: vec![],  // TODO: Extract from bid message
                                executor_trust_scope: None,    // TODO: Extract from bid message
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
            estimated_duration_secs: 60, // TODO: Calculate based on job requirements
            offered_resources: icn_protocol::ResourceRequirements {
                cpu_cores: bid.resources.cpu_cores,
                memory_mb: bid.resources.memory_mb,
                storage_mb: bid.resources.storage_mb,
                max_execution_time_secs: 300, // 5 minutes default
            },
            reputation_score: 100, // TODO: Get actual reputation score
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
        let logs: Vec<String> = Vec::new(); // TODO: Implement execution logging
        let execution_metadata = icn_protocol::ExecutionMetadata {
            wall_time_ms: receipt.cpu_ms,
            peak_memory_mb: 0, // TODO: Implement memory monitoring
            exit_code: if receipt.success { 0 } else { 1 },
            execution_logs: if logs.is_empty() {
                None
            } else {
                Some(logs.join("\n"))
            },
        };

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
}
