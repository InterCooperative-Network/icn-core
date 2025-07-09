//! RuntimeContext struct and implementations.

use super::errors::HostAbiError;
use super::mana::SimpleManaLedger;
use super::mesh_network::{DefaultMeshNetworkService, JobAssignmentNotice, MeshNetworkService};
use super::signers::Signer;
use super::stubs::{StubDagStore, StubMeshNetworkService};
use super::{DagStorageService, DagStoreMutexType};
use dashmap::DashMap;
use icn_common::{CommonError, Did, DagBlock, DagLink, Cid, SignatureBytes, NodeScope};
use icn_governance::GovernanceModule;
use icn_identity::ExecutionReceipt as IdentityExecutionReceipt;
use icn_mesh::{ActualMeshJob, JobId, JobState};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::{mpsc, Mutex as TokioMutex};

/// Enumeration of mesh network service types to work around async trait issues
#[derive(Debug)]
pub enum MeshNetworkServiceType {
    Stub(StubMeshNetworkService),
    Default(DefaultMeshNetworkService),
}

#[async_trait::async_trait]
impl MeshNetworkService for MeshNetworkServiceType {
    fn as_any(&self) -> &dyn std::any::Any {
        match self {
            MeshNetworkServiceType::Stub(s) => s.as_any(),
            MeshNetworkServiceType::Default(s) => s.as_any(),
        }
    }

    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.announce_job(job).await,
            MeshNetworkServiceType::Default(s) => s.announce_job(job).await,
        }
    }

    async fn announce_proposal(&self, proposal_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.announce_proposal(proposal_bytes).await,
            MeshNetworkServiceType::Default(s) => s.announce_proposal(proposal_bytes).await,
        }
    }

    async fn announce_vote(&self, vote_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.announce_vote(vote_bytes).await,
            MeshNetworkServiceType::Default(s) => s.announce_vote(vote_bytes).await,
        }
    }

    async fn collect_bids_for_job(
        &self,
        job_id: &JobId,
        duration: StdDuration,
    ) -> Result<Vec<icn_mesh::MeshJobBid>, HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.collect_bids_for_job(job_id, duration).await,
            MeshNetworkServiceType::Default(s) => s.collect_bids_for_job(job_id, duration).await,
        }
    }

    async fn notify_executor_of_assignment(
        &self,
        notice: &JobAssignmentNotice,
    ) -> Result<(), HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.notify_executor_of_assignment(notice).await,
            MeshNetworkServiceType::Default(s) => s.notify_executor_of_assignment(notice).await,
        }
    }

    async fn try_receive_receipt(
        &self,
        job_id: &JobId,
        expected_executor: &Did,
        timeout: StdDuration,
    ) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.try_receive_receipt(job_id, expected_executor, timeout).await,
            MeshNetworkServiceType::Default(s) => s.try_receive_receipt(job_id, expected_executor, timeout).await,
        }
    }
}

/// Core runtime context for the ICN node.
pub struct RuntimeContext {
    pub current_identity: Did,
    pub mana_ledger: SimpleManaLedger,
    pub pending_mesh_jobs_tx: mpsc::Sender<ActualMeshJob>,
    pub pending_mesh_jobs_rx: TokioMutex<mpsc::Receiver<ActualMeshJob>>,
    pub job_states: Arc<DashMap<JobId, JobState>>,
    pub governance_module: Arc<DagStoreMutexType<GovernanceModule>>,
    pub mesh_network_service: Arc<MeshNetworkServiceType>,
    pub signer: Arc<dyn Signer>,
    pub did_resolver: Arc<dyn icn_identity::DidResolver>,
    pub dag_store: Arc<DagStoreMutexType<DagStorageService>>,
    pub reputation_store: Arc<dyn icn_reputation::ReputationStore>,
    pub parameters: Arc<DashMap<String, String>>,
    pub policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
    pub time_provider: Arc<dyn icn_common::TimeProvider>,
    pub default_receipt_wait_ms: u64,
}

// Import std::str::FromStr for Did::from_str
use std::str::FromStr;

// Add governance-specific types
use icn_governance::{ProposalId, ProposalType, VoteOption, ProposalSubmission};
use serde::{Deserialize, Serialize};
use super::mesh_network::{PROPOSAL_COST_MANA, VOTE_COST_MANA};

/// Governance payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProposalPayload {
    pub proposal_type_str: String,
    pub type_specific_payload: Vec<u8>,
    pub description: String,
    pub duration_secs: u64,
    pub quorum: Option<usize>,
    pub threshold: Option<f32>,
    pub body: Option<Vec<u8>>, // raw proposal body stored in DAG
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastVotePayload {
    pub proposal_id_str: String,
    pub vote_option_str: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseProposalResult {
    pub status: String,
    pub yes: usize,
    pub no: usize,
    pub abstain: usize,
}

impl RuntimeContext {
    /// Create a new context with stubs for testing.
    pub fn new_with_stubs(current_identity_str: &str) -> Result<Arc<Self>, CommonError> {
        let current_identity = Did::from_str(current_identity_str)
            .map_err(|e| CommonError::InternalError(format!("Invalid DID: {}", e)))?;
        
        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let mesh_network_service = Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new()));
        let signer = Arc::new(super::signers::StubSigner::new());
        let did_resolver = Arc::new(icn_identity::KeyDidResolver);
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
        let parameters = Arc::new(DashMap::new());
        let policy_enforcer = None;
        let time_provider = Arc::new(icn_common::SystemTimeProvider);
        let mana_ledger = SimpleManaLedger::new(PathBuf::from("./temp_mana_ledger"));

        Ok(Arc::new(Self {
            current_identity,
            mana_ledger,
            pending_mesh_jobs_tx: tx,
            pending_mesh_jobs_rx: TokioMutex::new(rx),
            job_states,
            governance_module,
            mesh_network_service,
            signer,
            did_resolver,
            dag_store: Arc::new(DagStoreMutexType::new(StubDagStore::new())) as Arc<DagStoreMutexType<DagStorageService>>,
            reputation_store,
            parameters,
            policy_enforcer,
            time_provider,
            default_receipt_wait_ms: 30000,
        }))
    }

    /// Create a new context with proper services.
    pub fn new(
        current_identity: Did,
        mesh_network_service: Arc<MeshNetworkServiceType>,
        signer: Arc<dyn Signer>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
        dag_store: Arc<DagStoreMutexType<DagStorageService>>,
    ) -> Arc<Self> {
        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
        let parameters = Arc::new(DashMap::new());
        let policy_enforcer = None;
        let time_provider = Arc::new(icn_common::SystemTimeProvider);
        let mana_ledger = SimpleManaLedger::new(PathBuf::from("./temp_mana_ledger"));

        Arc::new(Self {
            current_identity,
            mana_ledger,
            pending_mesh_jobs_tx: tx,
            pending_mesh_jobs_rx: TokioMutex::new(rx),
            job_states,
            governance_module,
            mesh_network_service,
            signer,
            did_resolver,
            dag_store,
            reputation_store,
            parameters,
            policy_enforcer,
            time_provider,
            default_receipt_wait_ms: 30000,
        })
    }

    /// Internal queue mesh job method.
    pub async fn internal_queue_mesh_job(
        self: &Arc<Self>,
        job: ActualMeshJob,
    ) -> Result<(), HostAbiError> {
        self.pending_mesh_jobs_tx
            .send(job)
            .await
            .map_err(|e| HostAbiError::InternalError(format!("Failed to queue job: {}", e)))
    }

    /// Get mana for an account.
    pub async fn get_mana(&self, account: &Did) -> Result<u64, HostAbiError> {
        Ok(self.mana_ledger.get_balance(account))
    }

    /// Spend mana from an account.
    pub async fn spend_mana(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        self.mana_ledger.spend(account, amount)
    }

    /// Credit mana to an account.
    pub async fn credit_mana(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        self.mana_ledger.credit(account, amount)
    }

    /// Anchor an execution receipt.
    pub async fn anchor_receipt(
        &self,
        receipt: &IdentityExecutionReceipt,
    ) -> Result<Cid, HostAbiError> {
        // Create a DAG block for the receipt
        let receipt_bytes = bincode::serialize(receipt)
            .map_err(|e| HostAbiError::DagOperationFailed(format!("Failed to serialize receipt: {}", e)))?;
        
        let block = DagBlock {
            cid: receipt.result_cid.clone(),
            data: receipt_bytes,
            links: vec![],
            timestamp: self.time_provider.unix_seconds(),
            author_did: receipt.executor_did.clone(),
            signature: None,
            scope: None,
        };
        let cid = block.cid.clone();
        
        // Store in DAG
        {
            let mut dag_store = self.dag_store.lock()
                .map_err(|e| HostAbiError::DagOperationFailed(format!("Failed to lock DAG store: {}", e)))?;
            dag_store.put(&block)
                .map_err(|e| HostAbiError::DagOperationFailed(format!("Failed to store receipt: {}", e)))?;
        }
        
        Ok(cid)
    }

    /// Create a governance proposal.
    pub async fn create_governance_proposal(
        &self,
        payload: CreateProposalPayload,
    ) -> Result<String, HostAbiError> {
        self.spend_mana(&self.current_identity, PROPOSAL_COST_MANA)
            .await?;
        
        let proposal_type = match payload.proposal_type_str.to_lowercase().as_str() {
            "systemparameterchange" | "system_parameter_change" => {
                let tup: (String, String) = serde_json::from_slice(&payload.type_specific_payload)
                    .map_err(|e| {
                        HostAbiError::InvalidParameters(format!(
                            "Failed to parse system parameter payload: {}",
                            e
                        ))
                    })?;
                ProposalType::SystemParameterChange(tup.0, tup.1)
            }
            "memberadmission" | "newmemberinvitation" | "member_invitation" => {
                let did_str = String::from_utf8(payload.type_specific_payload).map_err(|e| {
                    HostAbiError::InvalidParameters(format!("Failed to parse member DID: {}", e))
                })?;
                let did = Did::from_str(&did_str)
                    .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid DID: {}", e)))?;
                ProposalType::NewMemberInvitation(did)
            }
            "softwareupgrade" | "software_upgrade" => {
                let version = String::from_utf8(payload.type_specific_payload).map_err(|e| {
                    HostAbiError::InvalidParameters(format!("Failed to parse version: {}", e))
                })?;
                ProposalType::SoftwareUpgrade(version)
            }
            "budgetallocation" | "budget_allocation" => {
                let tup: (u64, String) = serde_json::from_slice(&payload.type_specific_payload)
                    .map_err(|e| {
                        HostAbiError::InvalidParameters(format!(
                            "Failed to parse budget payload: {}",
                            e
                        ))
                    })?;
                ProposalType::BudgetAllocation(tup.0, tup.1)
            }
            "generictext" | "generic_text" => {
                let text = String::from_utf8(payload.type_specific_payload).map_err(|e| {
                    HostAbiError::InvalidParameters(format!("Failed to parse text: {}", e))
                })?;
                ProposalType::GenericText(text)
            }
            other => {
                return Err(HostAbiError::InvalidParameters(format!(
                    "Unknown proposal type: {}",
                    other
                )))
            }
        };

        let mut gov = self.governance_module.lock()
            .map_err(|e| HostAbiError::InternalError(format!("Failed to lock governance module: {}", e)))?;
        let content_cid = if let Some(body) = payload.body.clone() {
            // Create a simple block with the proposal body
            let block = DagBlock {
                cid: Cid::new_v1_sha256(0x55, &body),
                data: body,
                links: vec![],
                timestamp: self.time_provider.unix_seconds(),
                author_did: self.current_identity.clone(),
                signature: None,
                scope: None,
            };
            {
                let mut dag_store = self.dag_store.lock()
                    .map_err(|e| HostAbiError::DagOperationFailed(format!("Failed to lock DAG store: {}", e)))?;
                dag_store.put(&block)
                    .map_err(|e| HostAbiError::DagOperationFailed(format!("Failed to store proposal body: {}", e)))?;
            }
            Some(block.cid.clone())
        } else {
            None
        };
        
        let pid = gov
            .submit_proposal(ProposalSubmission {
                proposer: self.current_identity.clone(),
                proposal_type,
                description: payload.description,
                duration_secs: payload.duration_secs,
                quorum: payload.quorum,
                threshold: payload.threshold,
                content_cid,
            })
            .map_err(|e| HostAbiError::InternalError(format!("Failed to submit proposal: {}", e)))?;
        
        let proposal = gov
            .get_proposal(&pid)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to get proposal: {}", e)))?
            .ok_or_else(|| HostAbiError::InternalError("Proposal just inserted should exist".to_string()))?;
        
        drop(gov);
        
        let encoded = bincode::serialize(&proposal).map_err(|e| {
            HostAbiError::InternalError(format!("Failed to serialize proposal: {}", e))
        })?;
        
        if let Err(e) = self.mesh_network_service.announce_proposal(encoded).await {
            log::warn!("Failed to broadcast proposal {:?}: {}", pid, e);
        }
        
        Ok(pid.0.clone())
    }

    /// Cast a governance vote.
    pub async fn cast_governance_vote(&self, payload: CastVotePayload) -> Result<(), HostAbiError> {
        self.spend_mana(&self.current_identity, VOTE_COST_MANA)
            .await?;
        
        let proposal_id = ProposalId::from_str(&payload.proposal_id_str)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid proposal id: {}", e)))?;
        
        let vote_option = match payload.vote_option_str.to_lowercase().as_str() {
            "yes" => VoteOption::Yes,
            "no" => VoteOption::No,
            "abstain" => VoteOption::Abstain,
            other => {
                return Err(HostAbiError::InvalidParameters(format!(
                    "Unknown vote option: {}",
                    other
                )))
            }
        };
        
        let mut gov = self.governance_module.lock()
            .map_err(|e| HostAbiError::InternalError(format!("Failed to lock governance module: {}", e)))?;
        gov.cast_vote(self.current_identity.clone(), &proposal_id, vote_option)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to cast vote: {}", e)))?;
        
        let vote = icn_governance::Vote {
            voter: self.current_identity.clone(),
            proposal_id: proposal_id.clone(),
            option: vote_option,
            voted_at: self.time_provider.unix_seconds(),
        };
        
        drop(gov);
        
        let encoded = bincode::serialize(&vote)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to serialize vote: {}", e)))?;
        
        if let Err(e) = self.mesh_network_service.announce_vote(encoded).await {
            log::warn!("Failed to broadcast vote for {:?}: {}", proposal_id, e);
        }
        
        Ok(())
    }

    /// Close voting on a governance proposal.
    pub async fn close_governance_proposal_voting(
        &self,
        proposal_id_str: &str,
    ) -> Result<String, HostAbiError> {
        let proposal_id = ProposalId::from_str(proposal_id_str)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid proposal id: {}", e)))?;

        let mut gov = self.governance_module.lock()
            .map_err(|e| HostAbiError::InternalError(format!("Failed to lock governance module: {}", e)))?;
        let (status, (yes, no, abstain)) = gov
            .close_voting_period(&proposal_id)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to close voting: {}", e)))?;
        
        let proposal = gov
            .get_proposal(&proposal_id)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to get proposal: {}", e)))?
            .ok_or_else(|| HostAbiError::InternalError("Proposal should exist after closing".to_string()))?;
        
        drop(gov);

        let encoded = bincode::serialize(&proposal).map_err(|e| {
            HostAbiError::InternalError(format!("Failed to serialize proposal: {}", e))
        })?;
        
        if let Err(e) = self.mesh_network_service.announce_proposal(encoded).await {
            log::warn!("Failed to broadcast proposal {:?}: {}", proposal_id, e);
        }

        let result = CloseProposalResult {
            status: format!("{:?}", status),
            yes,
            no,
            abstain,
        };
        
        serde_json::to_string(&result)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to serialize tally: {}", e)))
    }

    /// Execute a governance proposal.
    pub async fn execute_governance_proposal(
        &self,
        proposal_id_str: &str,
    ) -> Result<(), HostAbiError> {
        let proposal_id = ProposalId::from_str(proposal_id_str)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid proposal id: {}", e)))?;

        let mut gov = self.governance_module.lock()
            .map_err(|e| HostAbiError::InternalError(format!("Failed to lock governance module: {}", e)))?;
        let result = gov.execute_proposal(&proposal_id);
        let proposal_opt = gov
            .get_proposal(&proposal_id)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to get proposal: {}", e)))?;
        
        drop(gov);

        if let Some(proposal) = proposal_opt {
            match result {
                Ok(()) => {
                    match &proposal.proposal_type {
                        ProposalType::SystemParameterChange(key, value) => {
                            self.update_parameter(key.clone(), value.clone()).await?;
                        }
                        _ => {
                            // For other proposal types, just log success
                            log::info!("Executed proposal {:?}", proposal_id);
                        }
                    }
                    
                    // Broadcast updated proposal
                    let encoded = bincode::serialize(&proposal).map_err(|e| {
                        HostAbiError::InternalError(format!("Failed to serialize proposal: {}", e))
                    })?;
                    
                    if let Err(e) = self.mesh_network_service.announce_proposal(encoded).await {
                        log::warn!("Failed to broadcast executed proposal {:?}: {}", proposal_id, e);
                    }
                }
                Err(e) => {
                    return Err(HostAbiError::InternalError(format!("Failed to execute proposal: {}", e)));
                }
            }
        } else {
            return Err(HostAbiError::InvalidParameters("Proposal not found".to_string()));
        }
        
        Ok(())
    }

    /// Delegate vote to another DID.
    pub async fn delegate_vote(
        &self,
        from_did: &str,
        to_did: &str,
    ) -> Result<(), HostAbiError> {
        let from = Did::from_str(from_did)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid from DID: {}", e)))?;
        let to = Did::from_str(to_did)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid to DID: {}", e)))?;

        // Only allow delegating your own vote
        if from != self.current_identity {
            return Err(HostAbiError::PermissionDenied("Can only delegate your own vote".to_string()));
        }

        let mut gov = self.governance_module.lock()
            .map_err(|e| HostAbiError::InternalError(format!("Failed to lock governance module: {}", e)))?;
        gov.delegate_vote(from, to)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to delegate vote: {}", e)))?;
        
        Ok(())
    }

    /// Revoke vote delegation.
    pub async fn revoke_delegation(
        &self,
        from_did: &str,
    ) -> Result<(), HostAbiError> {
        let from = Did::from_str(from_did)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid from DID: {}", e)))?;

        // Only allow revoking your own delegation
        if from != self.current_identity {
            return Err(HostAbiError::PermissionDenied("Can only revoke your own delegation".to_string()));
        }

        let mut gov = self.governance_module.lock()
            .map_err(|e| HostAbiError::InternalError(format!("Failed to lock governance module: {}", e)))?;
        gov.revoke_delegation(from);
        Ok(())
    }

    /// Update a system parameter.
    async fn update_parameter(&self, key: String, value: String) -> Result<(), HostAbiError> {
        self.parameters.insert(key.clone(), value.clone());
        log::info!("Updated parameter {} to {}", key, value);
        Ok(())
    }
} 