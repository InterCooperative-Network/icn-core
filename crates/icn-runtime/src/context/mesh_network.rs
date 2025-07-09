//! Mesh network service types and implementations for the ICN runtime.

use super::errors::HostAbiError;
use icn_common::{CommonError, Did};
use icn_identity::{ExecutionReceipt as IdentityExecutionReceipt, SignatureBytes};
use icn_mesh::{ActualMeshJob, JobId, MeshJobBid};
use icn_network::NetworkService;
use icn_protocol::{MessagePayload, ProtocolMessage, MeshJobAssignmentMessage, MeshJobAnnouncementMessage, GovernanceProposalMessage, GovernanceVoteMessage, ProposalType, VoteOption, JobSpec, ResourceRequirements, JobKind};
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
    inner: Arc<dyn NetworkService>,
}

impl std::fmt::Debug for DefaultMeshNetworkService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DefaultMeshNetworkService")
    }
}

impl DefaultMeshNetworkService {
    pub fn new(service: Arc<dyn NetworkService>) -> Self {
        Self { inner: service }
    }

    #[cfg(feature = "enable-libp2p")]
    pub fn get_underlying_broadcast_service(
        &self,
    ) -> Result<Arc<ActualLibp2pNetworkService>, CommonError> {
        icn_network::NetworkService::as_any(&*self.inner)
            .downcast_ref::<ActualLibp2pNetworkService>()
            .map(|s| Arc::new(s.clone()))
            .ok_or_else(|| CommonError::InternalError("Failed to downcast to LibP2P service".to_string()))
    }
}

#[async_trait::async_trait]
impl MeshNetworkService for DefaultMeshNetworkService {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        debug!("DefaultMeshNetworkService: announcing job {:?}", job.id);
        
        let job_bytes = bincode::serialize(job).map_err(|e| {
            HostAbiError::InternalError(format!("Failed to serialize job: {}", e))
        })?;
        
        let message = ProtocolMessage {
            payload: MessagePayload::MeshJobAnnouncement(MeshJobAnnouncementMessage {
                job_id: job.id.clone().into(),
                manifest_cid: job.manifest_cid.clone(),
                creator_did: job.creator_did.clone(),
                max_cost_mana: job.cost_mana,
                job_spec: icn_protocol::JobSpec {
                    kind: match &job.spec.kind {
                        icn_mesh::JobKind::Echo { payload } => JobKind::Echo { payload: payload.clone() },
                        icn_mesh::JobKind::CclWasm => JobKind::CclWasm,
                        icn_mesh::JobKind::GenericPlaceholder => JobKind::Generic,
                    },
                    inputs: job.spec.inputs.clone(),
                    outputs: job.spec.outputs.clone(),
                    required_resources: ResourceRequirements {
                        cpu_cores: job.spec.required_resources.cpu_cores,
                        memory_mb: job.spec.required_resources.memory_mb,
                        storage_mb: 0, // TODO: Add storage to mesh Resources
                        max_execution_time_secs: job.max_execution_wait_ms.unwrap_or(3600000) / 1000,
                    },
                },
                bid_deadline: current_timestamp() + 3600,
            }),
            timestamp: current_timestamp(),
            sender: job.creator_did.clone(),
            recipient: None,
            signature: SignatureBytes(vec![0u8; 64]), // Stub signature
            version: 1,
        };
        
        self.inner.broadcast_message(message).await.map_err(|e| {
            HostAbiError::NetworkError(format!("Failed to broadcast job: {}", e))
        })
    }

    async fn announce_proposal(&self, proposal_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        debug!("DefaultMeshNetworkService: announcing proposal");
        
        let message = ProtocolMessage {
            payload: MessagePayload::GovernanceProposalAnnouncement(GovernanceProposalMessage {
                proposal_id: "proposal_id".to_string(), // TODO: Generate a real proposal ID
                proposer_did: Did::from_str("did:example:system").unwrap(), // TODO: Use actual system identity
                proposal_type: ProposalType::TextProposal,
                description: "System governance proposal".to_string(),
                voting_deadline: current_timestamp() + 3600,
                proposal_data: proposal_bytes,
            }),
            timestamp: current_timestamp(),
            sender: Did::from_str("did:example:system").unwrap(), // TODO: Use actual system identity
            recipient: None,
            signature: SignatureBytes(vec![0u8; 64]), // Stub signature
            version: 1,
        };
        
        self.inner.broadcast_message(message).await.map_err(|e| {
            HostAbiError::NetworkError(format!("Failed to broadcast proposal: {}", e))
        })
    }

    async fn announce_vote(&self, vote_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        debug!("DefaultMeshNetworkService: announcing vote");
        
        let message = ProtocolMessage {
            payload: MessagePayload::GovernanceVoteAnnouncement(GovernanceVoteMessage {
                proposal_id: "proposal_id".to_string(), // TODO: Generate a real proposal ID
                voter_did: Did::from_str("did:example:system").unwrap(), // TODO: Use actual system identity
                vote_option: VoteOption::Yes,
                voted_at: current_timestamp(),
                justification: Some("System vote".to_string()),
            }),
            timestamp: current_timestamp(),
            sender: Did::from_str("did:example:system").unwrap(), // TODO: Use actual system identity
            recipient: None,
            signature: SignatureBytes(vec![0u8; 64]), // Stub signature
            version: 1,
        };
        
        self.inner.broadcast_message(message).await.map_err(|e| {
            HostAbiError::NetworkError(format!("Failed to broadcast vote: {}", e))
        })
    }

    async fn collect_bids_for_job(
        &self,
        job_id: &JobId,
        duration: StdDuration,
    ) -> Result<Vec<MeshJobBid>, HostAbiError> {
        debug!("DefaultMeshNetworkService: collecting bids for job {:?}", job_id);
        
        let bids = Vec::new();
        let timeout = tokio::time::sleep(duration);
        tokio::pin!(timeout);
        
        loop {
            tokio::select! {
                _ = &mut timeout => {
                    debug!("Bid collection timeout for job {:?}", job_id);
                    break;
                }
                // For now, just simulate waiting - in real implementation would listen for messages
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                    // Placeholder - real implementation would check for incoming bid messages
                    break;
                }
            }
        }
        
        Ok(bids)
    }

    async fn notify_executor_of_assignment(
        &self,
        notice: &JobAssignmentNotice,
    ) -> Result<(), HostAbiError> {
        debug!(
            "DefaultMeshNetworkService: notifying executor {:?} of assignment for job {:?}",
            notice.executor_did, notice.job_id
        );
        
        let assignment_message = MeshJobAssignmentMessage {
            job_id: notice.job_id.clone().into(),
            executor_did: notice.executor_did.clone(),
            agreed_cost_mana: 100, // TODO: Use actual agreed cost
            completion_deadline: current_timestamp() + 3600,
            manifest_cid: None,
        };
        
        let message = ProtocolMessage {
            payload: MessagePayload::MeshJobAssignment(assignment_message),
            timestamp: current_timestamp(),
            sender: Did::from_str("did:example:system").unwrap(), // TODO: Use actual system identity
            recipient: Some(notice.executor_did.clone()),
            signature: SignatureBytes(vec![0u8; 64]), // Stub signature
            version: 1,
        };
        
        // For now, just broadcast - in real implementation would send directly
        self.inner.broadcast_message(message).await.map_err(|e| {
            HostAbiError::NetworkError(format!("Failed to send assignment notice: {}", e))
        })
    }

    async fn try_receive_receipt(
        &self,
        job_id: &JobId,
        expected_executor: &Did,
        timeout_duration: StdDuration,
    ) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
        debug!(
            "DefaultMeshNetworkService: waiting for receipt for job {:?} from executor {:?}",
            job_id, expected_executor
        );
        
        let timeout = tokio::time::sleep(timeout_duration);
        tokio::pin!(timeout);
        
        loop {
            tokio::select! {
                _ = &mut timeout => {
                    debug!("Receipt timeout for job {:?}", job_id);
                    return Ok(None);
                }
                // For now, just simulate waiting - in real implementation would listen for messages
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                    // Placeholder - real implementation would check for incoming receipt messages
                    return Ok(None);
                }
            }
        }
    }
} 