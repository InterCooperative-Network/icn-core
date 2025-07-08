#![doc = include_str!("../README.md")]

//! # ICN Protocol Crate
//! This crate defines core message formats and protocol definitions for ICN,
//! ensuring interoperability between different components and nodes.
//!
//! This is the single source of truth for all network protocol messages,
//! including mesh computing, governance, DAG operations, and federation management.

use icn_common::{Cid, Did, DagBlock, CommonError, NodeInfo};
use icn_identity::{ExecutionReceipt, SignatureBytes};
use serde::{Deserialize, Serialize};

/// Protocol version for message compatibility
pub const ICN_PROTOCOL_VERSION: u32 = 1;

// === Core Protocol Message Envelope ===

/// Main protocol message envelope that wraps all ICN communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    /// Protocol version for compatibility checking
    pub version: u32,
    /// The actual message payload
    pub payload: MessagePayload,
    /// DID of the sender
    pub sender: Did,
    /// Optional target recipient DID (None for broadcast)
    pub recipient: Option<Did>,
    /// Message timestamp (Unix seconds)
    pub timestamp: u64,
    /// Cryptographic signature over the message
    pub signature: SignatureBytes,
}

/// All possible message payload types in the ICN protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    // === Mesh Computing Messages ===
    /// Announce a new job to potential executors
    MeshJobAnnouncement(MeshJobAnnouncementMessage),
    /// Submit a bid to execute a specific job
    MeshBidSubmission(MeshBidSubmissionMessage),
    /// Notify network of job assignment to executor
    MeshJobAssignment(MeshJobAssignmentMessage),
    /// Submit execution receipt with results
    MeshReceiptSubmission(MeshReceiptSubmissionMessage),
    
    // === DAG and Storage Messages ===
    /// Announce availability of a new DAG block
    DagBlockAnnouncement(DagBlockAnnouncementMessage),
    /// Request a specific DAG block by CID
    DagBlockRequest(DagBlockRequestMessage),
    /// Response with requested DAG block data
    DagBlockResponse(DagBlockResponseMessage),
    
    // === Governance Messages ===
    /// Announce a new governance proposal
    GovernanceProposalAnnouncement(GovernanceProposalMessage),
    /// Broadcast a vote on a proposal
    GovernanceVoteAnnouncement(GovernanceVoteMessage),
    /// Request governance state synchronization
    GovernanceStateSyncRequest(GovernanceStateSyncRequestMessage),
    
    // === Federation Management ===
    /// Request to join a federation
    FederationJoinRequest(FederationJoinRequestMessage),
    /// Response to federation join request
    FederationJoinResponse(FederationJoinResponseMessage),
    /// Request federation state synchronization
    FederationSyncRequest(FederationSyncRequestMessage),
    
    // === Network Management ===
    /// Generic gossip message for flexible communication
    GossipMessage(GossipMessage),
    /// Heartbeat/ping message for connectivity testing
    HeartbeatMessage(HeartbeatMessage),
    /// Peer discovery and capability advertisement
    PeerDiscoveryMessage(PeerDiscoveryMessage),
}

// === Mesh Computing Protocol Messages ===

/// Job specification for mesh execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobSpec {
    /// The type of job to execute
    pub kind: JobKind,
    /// Input data CIDs required for execution
    pub inputs: Vec<Cid>,
    /// Expected output names
    pub outputs: Vec<String>,
    /// Minimum resource requirements
    pub required_resources: ResourceRequirements,
}

/// Types of jobs that can be executed in the mesh
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobKind {
    /// Simple echo job for testing
    Echo { payload: String },
    /// Execute a CCL WASM module
    CclWasm,
    /// Generic placeholder for future job types
    Generic,
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResourceRequirements {
    /// Number of CPU cores needed
    pub cpu_cores: u32,
    /// Memory required in megabytes
    pub memory_mb: u32,
    /// Storage space needed in megabytes
    pub storage_mb: u32,
    /// Maximum execution time in seconds
    pub max_execution_time_secs: u64,
}

/// Announce a new job available for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshJobAnnouncementMessage {
    /// Unique job identifier
    pub job_id: Cid,
    /// CID of the job manifest with full details
    pub manifest_cid: Cid,
    /// DID of the job creator
    pub creator_did: Did,
    /// Maximum mana willing to pay
    pub max_cost_mana: u64,
    /// Brief job specification for filtering
    pub job_spec: JobSpec,
    /// Deadline for bid submissions (Unix timestamp)
    pub bid_deadline: u64,
}

/// Submit a bid to execute a specific job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshBidSubmissionMessage {
    /// ID of the job being bid on
    pub job_id: Cid,
    /// DID of the executor submitting the bid
    pub executor_did: Did,
    /// Proposed cost in mana
    pub cost_mana: u64,
    /// Estimated execution time in seconds
    pub estimated_duration_secs: u64,
    /// Resources being offered
    pub offered_resources: ResourceRequirements,
    /// Executor's current reputation score
    pub reputation_score: u64,
}

/// Notify network of job assignment to selected executor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshJobAssignmentMessage {
    /// ID of the assigned job
    pub job_id: Cid,
    /// DID of the selected executor
    pub executor_did: Did,
    /// Final agreed cost in mana
    pub agreed_cost_mana: u64,
    /// Expected completion deadline
    pub completion_deadline: u64,
    /// Optional manifest CID for executor reference
    pub manifest_cid: Option<Cid>,
}

/// Submit execution receipt with job results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshReceiptSubmissionMessage {
    /// The execution receipt with results
    pub receipt: ExecutionReceipt,
    /// Additional execution metadata
    pub execution_metadata: ExecutionMetadata,
}

/// Additional metadata about job execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Total wall-clock time for execution
    pub wall_time_ms: u64,
    /// Peak memory usage during execution
    pub peak_memory_mb: u32,
    /// Exit code of the executed process
    pub exit_code: i32,
    /// Optional execution logs or stdout
    pub execution_logs: Option<String>,
}

// === DAG and Storage Protocol Messages ===

/// Announce availability of a new DAG block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagBlockAnnouncementMessage {
    /// CID of the available block
    pub block_cid: Cid,
    /// Size of the block in bytes
    pub block_size: u64,
    /// Number of links from this block
    pub link_count: u32,
    /// Block creation timestamp
    pub created_at: u64,
}

/// Request a specific DAG block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagBlockRequestMessage {
    /// CID of the requested block
    pub block_cid: Cid,
    /// Priority of the request (0-255, higher = more urgent)
    pub priority: u8,
}

/// Response with requested DAG block data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagBlockResponseMessage {
    /// CID of the block being sent
    pub block_cid: Cid,
    /// The actual block data (None if not found)
    pub block_data: Option<DagBlock>,
    /// Error message if block could not be provided
    pub error: Option<String>,
}

// === Governance Protocol Messages ===

/// Announce a new governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposalMessage {
    /// Unique proposal identifier
    pub proposal_id: String,
    /// DID of the proposer
    pub proposer_did: Did,
    /// Type of proposal action
    pub proposal_type: ProposalType,
    /// Human-readable description
    pub description: String,
    /// Voting deadline (Unix timestamp)
    pub voting_deadline: u64,
    /// Serialized proposal data
    pub proposal_data: Vec<u8>,
}

/// Types of governance proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Change a system parameter
    ParameterChange { parameter: String, new_value: String },
    /// Add a new member to governance
    MembershipInvitation { new_member: Did },
    /// Remove a member from governance
    MembershipRevocation { member_to_remove: Did },
    /// Upgrade system software version
    SoftwareUpgrade { version: String },
    /// Generic text-based proposal
    TextProposal,
}

/// Broadcast a vote on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceVoteMessage {
    /// ID of the proposal being voted on
    pub proposal_id: String,
    /// DID of the voter
    pub voter_did: Did,
    /// The vote choice
    pub vote_option: VoteOption,
    /// Timestamp when vote was cast
    pub voted_at: u64,
    /// Optional justification for the vote
    pub justification: Option<String>,
}

/// Possible vote options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteOption {
    /// Vote in favor
    Yes,
    /// Vote against
    No,
    /// Abstain from voting
    Abstain,
}

/// Request governance state synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStateSyncRequestMessage {
    /// Sync proposals after this timestamp
    pub since_timestamp: Option<u64>,
    /// Maximum number of proposals to sync
    pub max_proposals: Option<u32>,
}

// === Federation Management Messages ===

/// Request to join a federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationJoinRequestMessage {
    /// DID of the requesting node
    pub requesting_node: Did,
    /// Identifier of the federation to join
    pub federation_id: String,
    /// Node's capabilities and resources
    pub node_capabilities: NodeCapabilities,
    /// Optional referral from existing member
    pub referral_from: Option<Did>,
}

/// Node capabilities for federation membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    /// Available compute resources
    pub compute_resources: ResourceRequirements,
    /// Supported job types
    pub supported_job_kinds: Vec<JobKind>,
    /// Network bandwidth capabilities
    pub network_bandwidth_mbps: u32,
    /// Storage capacity in GB
    pub storage_capacity_gb: u64,
    /// Node uptime percentage (0-100)
    pub uptime_percentage: f32,
}

/// Response to federation join request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationJoinResponseMessage {
    /// DID of the requesting node
    pub requesting_node: Did,
    /// Whether the request was accepted
    pub accepted: bool,
    /// Reason for acceptance/rejection
    pub reason: String,
    /// Federation configuration if accepted
    pub federation_config: Option<FederationConfig>,
}

/// Federation configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Federation identifier
    pub federation_id: String,
    /// List of federation members
    pub members: Vec<Did>,
    /// Governance parameters
    pub governance_params: GovernanceParams,
    /// Economic parameters
    pub economic_params: EconomicParams,
}

/// Governance parameters for federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceParams {
    /// Minimum quorum for proposals
    pub min_quorum: u32,
    /// Vote threshold for acceptance (0.0-1.0)
    pub vote_threshold: f32,
    /// Voting period in seconds
    pub voting_period_secs: u64,
}

/// Economic parameters for federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicParams {
    /// Base mana regeneration rate per hour
    pub base_mana_regen_rate: u64,
    /// Reputation multiplier for mana regen
    pub reputation_multiplier: f32,
    /// Maximum mana capacity
    pub max_mana_capacity: u64,
}

/// Request federation state synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationSyncRequestMessage {
    /// Federation to sync with
    pub federation_id: String,
    /// Sync events after this timestamp
    pub since_timestamp: Option<u64>,
    /// Types of data to sync
    pub sync_types: Vec<SyncDataType>,
}

/// Types of federation data that can be synchronized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncDataType {
    /// Sync member list
    Members,
    /// Sync governance proposals
    Proposals,
    /// Sync job announcements
    Jobs,
    /// Sync reputation data
    Reputation,
}

// === Network Management Messages ===

/// Generic gossip message for flexible communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    /// Topic identifier for message routing
    pub topic: String,
    /// Message payload
    pub payload: Vec<u8>,
    /// Time-to-live for message propagation
    pub ttl: u32,
}

/// Heartbeat message for connectivity testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    /// Sequence number for tracking
    pub sequence: u64,
    /// Timestamp when heartbeat was sent
    pub sent_at: u64,
    /// Node status information
    pub node_status: NodeStatus,
}

/// Node status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    /// Whether node is online and available
    pub is_online: bool,
    /// Number of connected peers
    pub peer_count: u32,
    /// Current block height in DAG
    pub block_height: u64,
    /// Software version running
    pub version: String,
    /// Available resources
    pub available_resources: ResourceRequirements,
}

/// Peer discovery and capability advertisement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDiscoveryMessage {
    /// DID of the advertising node
    pub advertiser_did: Did,
    /// Node capabilities
    pub capabilities: NodeCapabilities,
    /// Network addresses for connection
    pub addresses: Vec<String>,
    /// Services offered by this node
    pub services: Vec<ServiceType>,
}

/// Types of services a node can offer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    /// Job execution service
    MeshExecution,
    /// DAG storage service
    DagStorage,
    /// Governance participation
    Governance,
    /// Federation management
    Federation,
    /// General relay/routing
    Relay,
}

// === Message Helper Functions ===

impl MessagePayload {
    /// Get a string identifier for the message type
    pub fn message_type(&self) -> &'static str {
        match self {
            MessagePayload::MeshJobAnnouncement(_) => "MeshJobAnnouncement",
            MessagePayload::MeshBidSubmission(_) => "MeshBidSubmission", 
            MessagePayload::MeshJobAssignment(_) => "MeshJobAssignment",
            MessagePayload::MeshReceiptSubmission(_) => "MeshReceiptSubmission",
            MessagePayload::DagBlockAnnouncement(_) => "DagBlockAnnouncement",
            MessagePayload::DagBlockRequest(_) => "DagBlockRequest",
            MessagePayload::DagBlockResponse(_) => "DagBlockResponse",
            MessagePayload::GovernanceProposalAnnouncement(_) => "GovernanceProposalAnnouncement",
            MessagePayload::GovernanceVoteAnnouncement(_) => "GovernanceVoteAnnouncement",
            MessagePayload::GovernanceStateSyncRequest(_) => "GovernanceStateSyncRequest",
            MessagePayload::FederationJoinRequest(_) => "FederationJoinRequest",
            MessagePayload::FederationJoinResponse(_) => "FederationJoinResponse",
            MessagePayload::FederationSyncRequest(_) => "FederationSyncRequest",
            MessagePayload::GossipMessage(_) => "GossipMessage",
            MessagePayload::HeartbeatMessage(_) => "HeartbeatMessage",
            MessagePayload::PeerDiscoveryMessage(_) => "PeerDiscoveryMessage",
        }
    }
}

impl ProtocolMessage {
    /// Create a new protocol message with current timestamp
    pub fn new(payload: MessagePayload, sender: Did, recipient: Option<Did>) -> Self {
        Self {
            version: ICN_PROTOCOL_VERSION,
            payload,
            sender,
            recipient,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            signature: SignatureBytes(vec![]), // To be filled by signing
        }
    }
}

// === Legacy Support ===

/// Legacy federation join request for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationJoinRequest {
    /// DID of the requesting node
    pub node: Did,
    /// Identifier of the federation to join
    pub federation_id: String,
}

/// Legacy federation join response for backward compatibility  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationJoinResponse {
    /// DID of the node requesting membership
    pub node: Did,
    /// Whether the request was accepted
    pub accepted: bool,
}

/// Placeholder function demonstrating use of common types for protocol messages.
pub fn serialize_protocol_message(
    info: &NodeInfo,
    message_type: u16,
) -> Result<Vec<u8>, CommonError> {
    let msg_string = format!(
        "Msg type {} from node: {} (v{})",
        message_type, info.name, info.version
    );
    Ok(msg_string.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::ICN_CORE_VERSION;
    use std::str::FromStr;

    #[test]
    fn test_serialize_protocol_message() {
        let node_info = NodeInfo {
            name: "ProtoNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Protocol active".to_string(),
        };
        let result = serialize_protocol_message(&node_info, 1);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
        assert!(String::from_utf8(bytes).unwrap().contains("ProtoNode"));
    }

    #[test]
    fn test_protocol_message_creation() {
        let sender = Did::from_str("did:key:test").unwrap();
        let payload = MessagePayload::HeartbeatMessage(HeartbeatMessage {
            sequence: 1,
            sent_at: 1234567890,
            node_status: NodeStatus {
                is_online: true,
                peer_count: 5,
                block_height: 100,
                version: "1.0.0".to_string(),
                available_resources: ResourceRequirements::default(),
            },
        });

        let msg = ProtocolMessage::new(payload, sender, None);
        assert_eq!(msg.version, ICN_PROTOCOL_VERSION);
        assert_eq!(msg.payload.message_type(), "HeartbeatMessage");
    }

    #[test]
    fn test_message_payload_types() {
        let job_announcement = MessagePayload::MeshJobAnnouncement(MeshJobAnnouncementMessage {
            job_id: icn_common::Cid::new_v1_sha256(0x55, b"test"),
            manifest_cid: icn_common::Cid::new_v1_sha256(0x55, b"manifest"),
            creator_did: Did::from_str("did:key:creator").unwrap(),
            max_cost_mana: 100,
            job_spec: JobSpec {
                kind: JobKind::Echo { payload: "test".to_string() },
                inputs: vec![],
                outputs: vec!["result".to_string()],
                required_resources: ResourceRequirements::default(),
            },
            bid_deadline: 1234567890,
        });

        assert_eq!(job_announcement.message_type(), "MeshJobAnnouncement");
    }
}
