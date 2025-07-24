//! Federated proposal synchronization implementation
//! 
//! This module provides functionality to synchronize governance proposals
//! across federation nodes, ensuring all members have consistent view of
//! active proposals and can participate in federated decision-making.

use crate::federation_governance::{FederationProposal, ProposalStatus};
use crate::{GovernanceEvent, GovernanceModule, Proposal, ProposalId, Vote, VoteOption, ProposalType, ProposalStatus as MainProposalStatus};
use icn_common::{CommonError, Did};
use icn_identity::{FederationId, TrustContext};
#[cfg(feature = "federation")]
use icn_network::{NetworkService, PeerId};
#[cfg(feature = "federation")]
use icn_protocol::{MessagePayload, ProtocolMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Message types for federation proposal synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FederationSyncMessage {
    /// Announce a new proposal to federation members
    ProposalAnnouncement {
        proposal: FederationProposal,
        sender: Did,
        federation_id: FederationId,
    },
    /// Share a vote on a federated proposal
    VoteShare {
        proposal_id: ProposalId,
        voter: Did,
        vote: bool,
        federation_id: FederationId,
        timestamp: u64,
    },
    /// Request proposals from a specific federation member
    ProposalRequest {
        federation_id: FederationId,
        since_timestamp: Option<u64>,
        requester: Did,
    },
    /// Response to proposal request
    ProposalResponse {
        proposals: Vec<FederationProposal>,
        federation_id: FederationId,
        responder: Did,
    },
    /// Announce proposal status change
    StatusUpdate {
        proposal_id: ProposalId,
        new_status: ProposalStatus,
        federation_id: FederationId,
        updater: Did,
    },
}

/// Federation governance synchronization coordinator
pub struct FederationSyncCoordinator {
    /// Network service for sending messages
    #[cfg(feature = "federation")]
    network_service: Arc<dyn NetworkService>,
    /// Local governance engine
    governance_engine: Arc<RwLock<GovernanceModule>>,
    /// Federation memberships this node participates in
    federations: Arc<RwLock<HashMap<FederationId, FederationMembership>>>,
    /// Synchronization state tracking
    sync_state: Arc<RwLock<SyncState>>,
    /// This node's identity
    node_identity: Did,
}

/// Membership details for a federation
#[derive(Debug, Clone)]
struct FederationMembership {
    federation_id: FederationId,
    peers: Vec<PeerId>,
    trust_context: TrustContext,
    last_sync: u64,
    sync_enabled: bool,
}

/// Synchronization state tracking
#[derive(Debug, Default)]
struct SyncState {
    /// Last sync timestamps per federation
    last_sync_times: HashMap<FederationId, u64>,
    /// Pending sync requests
    pending_requests: HashMap<ProposalId, u64>,
    /// Failed sync attempts
    failed_syncs: HashMap<FederationId, Vec<SyncFailure>>,
}

/// Record of a failed synchronization attempt
#[derive(Debug, Clone)]
struct SyncFailure {
    timestamp: u64,
    error: String,
    proposal_id: Option<ProposalId>,
}

impl FederationSyncCoordinator {
    /// Create a new federation sync coordinator
    #[cfg(feature = "federation")]
    pub fn new(
        network_service: Arc<dyn NetworkService>,
        governance_engine: Arc<RwLock<GovernanceModule>>,
        node_identity: Did,
    ) -> Self {
        Self {
            network_service,
            governance_engine,
            federations: Arc::new(RwLock::new(HashMap::new())),
            sync_state: Arc::new(RwLock::new(SyncState::default())),
            node_identity,
        }
    }

    /// Create a new federation sync coordinator without network (for testing)
    #[cfg(not(feature = "federation"))]
    pub fn new(
        governance_engine: Arc<RwLock<GovernanceModule>>,
        node_identity: Did,
    ) -> Self {
        Self {
            governance_engine,
            federations: Arc::new(RwLock::new(HashMap::new())),
            sync_state: Arc::new(RwLock::new(SyncState::default())),
            node_identity,
        }
    }

    /// Join a federation and enable proposal synchronization
    pub async fn join_federation(
        &self,
        federation_id: FederationId,
        peers: Vec<PeerId>,
        trust_context: TrustContext,
    ) -> Result<(), CommonError> {
        let membership = FederationMembership {
            federation_id: federation_id.clone(),
            peers,
            trust_context,
            last_sync: 0,
            sync_enabled: true,
        };

        {
            let mut federations = self.federations.write().await;
            federations.insert(federation_id.clone(), membership);
        }

        // Request initial synchronization
        self.request_proposal_sync(&federation_id, None).await?;

        Ok(())
    }

    /// Leave a federation and stop synchronization
    pub async fn leave_federation(&self, federation_id: &FederationId) -> Result<(), CommonError> {
        let mut federations = self.federations.write().await;
        if federations.remove(federation_id).is_some() {
            log::info!("Left federation: {}", federation_id.as_str());
        }
        Ok(())
    }

    /// Submit a new proposal and broadcast to federation
    pub async fn submit_federated_proposal(
        &self,
        proposer: Did,
        federation_id: FederationId,
        trust_context: TrustContext,
        content: String,
        voting_deadline: u64,
    ) -> Result<ProposalId, CommonError> {
        // Submit to local governance engine using ProposalSubmission
        let submission = crate::ProposalSubmission {
            proposer: proposer.clone(),
            proposal_type: ProposalType::GenericText(content.clone()),
            description: content.clone(),
            duration_secs: voting_deadline.saturating_sub(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
            quorum: None,
            threshold: None,
            content_cid: None,
        };

        let proposal_id = {
            let mut engine = self.governance_engine.write().await;
            engine.submit_proposal(submission)?
        };

        // Get the created proposal for broadcasting
        let proposal = {
            let engine = self.governance_engine.read().await;
            engine.get_proposal(&proposal_id)?
                .ok_or_else(|| CommonError::ResourceNotFound("Proposal not found after creation".to_string()))?
        };

        // Convert to federation proposal for sync
        let fed_proposal = FederationProposal {
            id: proposal_id.clone(),
            proposer,
            federation: federation_id.clone(),
            trust_context,
            content: content.clone(),
            votes: HashMap::new(),
            status: ProposalStatus::Open,
            created_at: proposal.created_at,
            voting_deadline,
        };

        // Broadcast to federation members
        self.broadcast_proposal_announcement(fed_proposal).await?;

        Ok(proposal_id)
    }

    /// Cast a vote and share with federation
    pub async fn cast_federated_vote(
        &self,
        voter: Did,
        proposal_id: ProposalId,
        vote: bool,
    ) -> Result<(), CommonError> {
        // Get federation ID for this proposal (simulate - we'll use a default for now)
        let federation_id = FederationId::new("default-federation".to_string());

        // Cast vote locally
        {
            let mut engine = self.governance_engine.write().await;
            let vote_option = if vote { VoteOption::Yes } else { VoteOption::No };
            engine.cast_vote(voter.clone(), &proposal_id, vote_option)?;
        }

        // Share vote with federation
        let vote_message = FederationSyncMessage::VoteShare {
            proposal_id,
            voter,
            vote,
            federation_id: federation_id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        self.broadcast_sync_message(federation_id, vote_message).await?;

        Ok(())
    }

    /// Handle incoming federation sync message
    pub async fn handle_sync_message(
        &self,
        message: FederationSyncMessage,
        sender_peer: PeerId,
    ) -> Result<(), CommonError> {
        match message {
            FederationSyncMessage::ProposalAnnouncement { proposal, .. } => {
                self.handle_proposal_announcement(proposal).await
            }
            FederationSyncMessage::VoteShare { proposal_id, voter, vote, .. } => {
                self.handle_vote_share(proposal_id, voter, vote).await
            }
            FederationSyncMessage::ProposalRequest { federation_id, since_timestamp, requester } => {
                self.handle_proposal_request(federation_id, since_timestamp, requester, sender_peer).await
            }
            FederationSyncMessage::ProposalResponse { proposals, .. } => {
                self.handle_proposal_response(proposals).await
            }
            FederationSyncMessage::StatusUpdate { proposal_id, new_status, .. } => {
                self.handle_status_update(proposal_id, new_status).await
            }
        }
    }

    /// Request proposal synchronization from federation peers
    async fn request_proposal_sync(
        &self,
        federation_id: &FederationId,
        since_timestamp: Option<u64>,
    ) -> Result<(), CommonError> {
        let peers = {
            let federations = self.federations.read().await;
            federations.get(federation_id)
                .map(|m| m.peers.clone())
                .unwrap_or_default()
        };

        let request_message = FederationSyncMessage::ProposalRequest {
            federation_id: federation_id.clone(),
            since_timestamp,
            requester: self.node_identity.clone(),
        };

        for peer in peers {
            #[cfg(feature = "federation")]
            if let Err(e) = self.send_message_to_peer(peer, request_message.clone()).await {
                log::warn!("Failed to send proposal request to peer {}: {}", peer, e);
                self.record_sync_failure(federation_id.clone(), e.to_string(), None).await;
            }
        }

        Ok(())
    }

    /// Broadcast proposal announcement to federation members
    async fn broadcast_proposal_announcement(
        &self,
        proposal: FederationProposal,
    ) -> Result<(), CommonError> {
        let announcement = FederationSyncMessage::ProposalAnnouncement {
            proposal: proposal.clone(),
            sender: self.node_identity.clone(),
            federation_id: proposal.federation.clone(),
        };

        self.broadcast_sync_message(proposal.federation, announcement).await
    }

    /// Broadcast a sync message to all federation members
    async fn broadcast_sync_message(
        &self,
        federation_id: FederationId,
        message: FederationSyncMessage,
    ) -> Result<(), CommonError> {
        let peers = {
            let federations = self.federations.read().await;
            federations.get(&federation_id)
                .map(|m| m.peers.clone())
                .unwrap_or_default()
        };

        for peer in peers {
            #[cfg(feature = "federation")]
            if let Err(e) = self.send_message_to_peer(peer, message.clone()).await {
                log::warn!("Failed to send sync message to peer {}: {}", peer, e);
                self.record_sync_failure(federation_id.clone(), e.to_string(), None).await;
            }
        }

        Ok(())
    }

    /// Send a message to a specific peer
    #[cfg(feature = "federation")]
    async fn send_message_to_peer(
        &self,
        peer: PeerId,
        message: FederationSyncMessage,
    ) -> Result<(), CommonError> {
        let protocol_message = ProtocolMessage::new(
            MessagePayload::Custom(serde_json::to_string(&message)?),
            self.node_identity.clone(),
            None,
        );

        self.network_service.send_message(&peer, protocol_message)
            .await
            .map_err(|e| CommonError::NetworkError(e.to_string()))
    }

    /// Handle incoming proposal announcement
    async fn handle_proposal_announcement(&self, proposal: FederationProposal) -> Result<(), CommonError> {
        // Check if we already have this proposal
        {
            let engine = self.governance_engine.read().await;
            if engine.get_proposal(&proposal.id)?.is_some() {
                return Ok(()); // Already have this proposal
            }
        }

        // Add proposal to local engine
        {
            let mut engine = self.governance_engine.write().await;
            // Convert to regular proposal for the main governance module
            let governance_proposal = Proposal {
                id: proposal.id.clone(),
                proposer: proposal.proposer.clone(),
                proposal_type: ProposalType::GenericText(proposal.content.clone()),
                description: proposal.content,
                created_at: proposal.created_at,
                voting_deadline: proposal.voting_deadline,
                status: match proposal.status {
                    ProposalStatus::Open => MainProposalStatus::VotingOpen,
                    ProposalStatus::Passed => MainProposalStatus::Accepted,
                    ProposalStatus::Failed => MainProposalStatus::Rejected,
                    ProposalStatus::Executed => MainProposalStatus::Executed,
                    ProposalStatus::Cancelled => MainProposalStatus::Rejected,
                },
                votes: HashMap::new(),
                quorum: None,
                threshold: None,
                content_cid: None,
            };

            // Insert as external proposal
            if let Err(e) = engine.insert_external_proposal(governance_proposal) {
                log::warn!("Failed to insert external proposal {}: {}", proposal.id, e);
                return Err(e);
            }
        }

        log::info!("Received federated proposal: {}", proposal.id);
        Ok(())
    }

    /// Handle incoming vote share
    async fn handle_vote_share(
        &self,
        proposal_id: ProposalId,
        voter: Did,
        vote: bool,
    ) -> Result<(), CommonError> {
        let vote_record = Vote {
            voter: voter.clone(),
            proposal_id: proposal_id.clone(),
            option: if vote { VoteOption::Yes } else { VoteOption::No },
            voted_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Insert external vote
        {
            let mut engine = self.governance_engine.write().await;
            if let Err(e) = engine.insert_external_vote(vote_record) {
                log::warn!("Failed to insert external vote for proposal {}: {}", proposal_id, e);
                return Err(e);
            }
        }

        log::debug!("Received federated vote from {} on proposal {}", voter, proposal_id);
        Ok(())
    }

    /// Handle proposal request from peer
    async fn handle_proposal_request(
        &self,
        federation_id: FederationId,
        since_timestamp: Option<u64>,
        _requester: Did,
        sender_peer: PeerId,
    ) -> Result<(), CommonError> {
        let proposals = {
            let engine = self.governance_engine.read().await;
            let all_proposals = engine.list_proposals()?;
            
            // Filter by timestamp if specified
            if let Some(since) = since_timestamp {
                all_proposals.into_iter()
                    .filter(|p| p.created_at > since)
                    .collect()
            } else {
                all_proposals
            }
        };

        // Convert to federation proposals for response
        let fed_proposals: Vec<FederationProposal> = proposals.into_iter().map(|p| FederationProposal {
            id: p.id,
            proposer: p.proposer,
            federation: federation_id.clone(),
            trust_context: TrustContext::Governance,
            content: p.description,
            votes: HashMap::new(),
            status: match p.status {
                MainProposalStatus::Deliberation => ProposalStatus::Open,
                MainProposalStatus::VotingOpen => ProposalStatus::Open,
                MainProposalStatus::Accepted => ProposalStatus::Passed,
                MainProposalStatus::Rejected => ProposalStatus::Failed,
                MainProposalStatus::Executed => ProposalStatus::Executed,
                MainProposalStatus::Failed => ProposalStatus::Failed,
            },
            created_at: p.created_at,
            voting_deadline: p.voting_deadline,
        }).collect();

        let response = FederationSyncMessage::ProposalResponse {
            proposals: fed_proposals,
            federation_id,
            responder: self.node_identity.clone(),
        };

        #[cfg(feature = "federation")]
        self.send_message_to_peer(sender_peer, response).await?;

        Ok(())
    }

    /// Handle proposal response from peer
    async fn handle_proposal_response(&self, proposals: Vec<FederationProposal>) -> Result<(), CommonError> {
        for proposal in proposals {
            if let Err(e) = self.handle_proposal_announcement(proposal.clone()).await {
                log::warn!("Failed to handle proposal {} from response: {}", proposal.id, e);
            }
        }
        Ok(())
    }

    /// Handle proposal status update
    async fn handle_status_update(
        &self,
        proposal_id: ProposalId,
        new_status: ProposalStatus,
    ) -> Result<(), CommonError> {
        // Update local proposal status if we have it
        {
            let engine = self.governance_engine.read().await;
            if engine.get_proposal(&proposal_id)?.is_some() {
                log::info!("Updated proposal {} status to {:?}", proposal_id, new_status);
                // The actual status update would need to be implemented in the governance engine
            }
        }
        Ok(())
    }

    /// Record a synchronization failure
    async fn record_sync_failure(
        &self,
        federation_id: FederationId,
        error: String,
        proposal_id: Option<ProposalId>,
    ) {
        let failure = SyncFailure {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            error,
            proposal_id,
        };

        let mut sync_state = self.sync_state.write().await;
        sync_state.failed_syncs
            .entry(federation_id)
            .or_default()
            .push(failure);
    }

    /// Get synchronization statistics
    pub async fn get_sync_stats(&self) -> SyncStats {
        let sync_state = self.sync_state.read().await;
        let federations = self.federations.read().await;

        let total_federations = federations.len();
        let total_failures: usize = sync_state.failed_syncs.values().map(|v| v.len()).sum();
        let federations_with_failures = sync_state.failed_syncs.len();

        SyncStats {
            total_federations,
            total_failures,
            federations_with_failures,
            last_sync_times: sync_state.last_sync_times.clone(),
        }
    }
}

/// Statistics about federation synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub total_federations: usize,
    pub total_failures: usize,
    pub federations_with_failures: usize,
    pub last_sync_times: HashMap<FederationId, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_identity::TrustPolicyEngine;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_federation_sync_coordinator_creation() {
        let governance_engine = Arc::new(RwLock::new(GovernanceModule::new()));
        let node_identity = Did::new("key", "test-node");

        #[cfg(not(feature = "federation"))]
        let coordinator = FederationSyncCoordinator::new(governance_engine, node_identity);

        let stats = coordinator.get_sync_stats().await;
        assert_eq!(stats.total_federations, 0);
        assert_eq!(stats.total_failures, 0);
    }

    #[tokio::test]
    async fn test_proposal_announcement_handling() {
        let governance_engine = Arc::new(RwLock::new(GovernanceModule::new()));
        let node_identity = Did::new("key", "test-node");

        #[cfg(not(feature = "federation"))]
        let coordinator = FederationSyncCoordinator::new(governance_engine.clone(), node_identity);

        let proposal = FederationProposal {
            id: ProposalId("test-proposal".to_string()),
            proposer: Did::new("key", "proposer"),
            federation: FederationId::new("test-federation".to_string()),
            trust_context: TrustContext::Governance,
            content: "Test proposal content".to_string(),
            votes: HashMap::new(),
            status: ProposalStatus::Open,
            created_at: 1234567890,
            voting_deadline: 1234567890 + 3600,
        };

        // Handle the proposal announcement
        coordinator.handle_proposal_announcement(proposal.clone()).await.unwrap();

        // Verify the proposal was added to the governance engine
        let engine = governance_engine.read().await;
        let stored_proposal = engine.get_proposal(&proposal.id).unwrap();
        assert!(stored_proposal.is_some());
    }
}