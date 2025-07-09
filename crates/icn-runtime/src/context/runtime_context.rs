//! RuntimeContext struct and implementations.

use super::errors::HostAbiError;
use super::mana::SimpleManaLedger;
use super::mesh_network::{DefaultMeshNetworkService, JobAssignmentNotice, MeshNetworkService};
use super::signers::Signer;
use super::stubs::{StubDagStore, StubMeshNetworkService};
use super::DagStorageService;
use dashmap::DashMap;
use icn_common::{Cid, CommonError, DagBlock, Did};
use icn_governance::GovernanceModule;
use icn_identity::ExecutionReceipt as IdentityExecutionReceipt;
use icn_mesh::{ActualMeshJob, JobId, JobState};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::{mpsc, Mutex as TokioMutex};

#[cfg(feature = "async")]
use tokio::sync::Mutex as DagStoreMutex;
#[cfg(not(feature = "async"))]
use std::sync::Mutex as DagStoreMutex;

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
    pub governance_module: Arc<DagStoreMutex<GovernanceModule>>,
    pub mesh_network_service: Arc<MeshNetworkServiceType>,
    pub signer: Arc<dyn Signer>,
    pub did_resolver: Arc<dyn icn_identity::DidResolver>,
    pub dag_store: Arc<DagStoreMutex<dyn DagStorageService<DagBlock> + Send>>,
    pub reputation_store: Arc<dyn icn_reputation::ReputationStore>,
    pub parameters: Arc<DashMap<String, String>>,
    pub policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
    pub time_provider: Arc<dyn icn_common::TimeProvider>,
    pub default_receipt_wait_ms: u64,
}

impl RuntimeContext {
    /// Create a new context with stubs for testing.
    pub fn new_with_stubs(current_identity_str: &str) -> Result<Arc<Self>, CommonError> {
        let current_identity = Did::from_str(current_identity_str)
            .map_err(|e| CommonError::InvalidFormat(format!("Invalid DID: {}", e)))?;
        
        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutex::new(GovernanceModule::new()));
        let mesh_network_service = Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new()));
        let signer = Arc::new(super::signers::StubSigner::new());
        let did_resolver = Arc::new(icn_identity::StubDidResolver::new());
        let dag_store = Arc::new(TokioMutex::new(StubDagStore::new()));
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
            dag_store: Arc::new(TokioMutex::new(StubDagStore::new())) as Arc<DagStoreMutex<dyn DagStorageService<DagBlock> + Send>>,
            reputation_store,
            parameters,
            policy_enforcer,
            time_provider,
            default_receipt_wait_ms: 30000,
        }))
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
        
        let block = DagBlock::new(receipt_bytes, None);
        let cid = block.cid.clone();
        
        // Store in DAG
        {
            let mut dag_store = self.dag_store.lock().await;
            dag_store.put(&block).await
                .map_err(|e| HostAbiError::DagOperationFailed(format!("Failed to store receipt: {}", e)))?;
        }
        
        Ok(cid)
    }
}

// Import std::str::FromStr for Did::from_str
use std::str::FromStr; 