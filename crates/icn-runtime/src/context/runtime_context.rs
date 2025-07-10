//! RuntimeContext struct and implementations.

use super::errors::HostAbiError;
use super::mana::SimpleManaLedger;
use super::mesh_network::{DefaultMeshNetworkService, JobAssignmentNotice, MeshNetworkService};
use super::signers::Signer;
use super::stubs::{StubDagStore, StubMeshNetworkService};
use super::{DagStorageService, DagStoreMutexType};
use crate::metrics::{JOBS_ACTIVE_GAUGE, JOBS_COMPLETED, JOBS_FAILED, JOBS_SUBMITTED};
use dashmap::DashMap;
use icn_common::{Cid, CommonError, DagBlock, Did};
use icn_economics::ManaLedger;
use icn_governance::GovernanceModule;
use icn_identity::ExecutionReceipt as IdentityExecutionReceipt;
use icn_mesh::metrics::{JOB_PROCESS_TIME, PENDING_JOBS_GAUGE};
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
            MeshNetworkServiceType::Stub(s) => {
                s.try_receive_receipt(job_id, expected_executor, timeout)
                    .await
            }
            MeshNetworkServiceType::Default(s) => {
                s.try_receive_receipt(job_id, expected_executor, timeout)
                    .await
            }
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
use super::mesh_network::{PROPOSAL_COST_MANA, VOTE_COST_MANA};
use icn_governance::{ProposalId, ProposalSubmission, ProposalType, VoteOption};
use serde::{Deserialize, Serialize};

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

/// Parameters for [`RuntimeContext`] construction.
pub struct RuntimeContextParams {
    pub current_identity: Did,
    pub mesh_network_service: Arc<MeshNetworkServiceType>,
    pub signer: Arc<dyn Signer>,
    pub did_resolver: Arc<dyn icn_identity::DidResolver>,
    pub dag_store: Arc<DagStoreMutexType<DagStorageService>>,
    pub mana_ledger: SimpleManaLedger,
    pub reputation_path: PathBuf,
    pub policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
    pub time_provider: Arc<dyn icn_common::TimeProvider>,
}

impl RuntimeContext {
    /// Create a new context with stubs for testing.
    pub fn new_with_stubs(current_identity_str: &str) -> Result<Arc<Self>, CommonError> {
        let current_identity = Did::from_str(current_identity_str)
            .map_err(|e| CommonError::InternalError(format!("Invalid DID: {}", e)))?;

        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let mesh_network_service =
            Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new()));
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
            dag_store: Arc::new(DagStoreMutexType::new(StubDagStore::new()))
                as Arc<DagStoreMutexType<DagStorageService>>,
            reputation_store,
            parameters,
            policy_enforcer,
            time_provider,
            default_receipt_wait_ms: 30000,
        }))
    }

    /// Create a new context with stubs and initial mana balance (convenience method for tests).
    pub fn new_with_stubs_and_mana(current_identity_str: &str, initial_mana: u64) -> Result<Arc<Self>, CommonError> {
        let ctx = Self::new_with_stubs(current_identity_str)?;
        let current_identity = Did::from_str(current_identity_str)
            .map_err(|e| CommonError::InternalError(format!("Invalid DID: {}", e)))?;
        ctx.mana_ledger.set_balance(&current_identity, initial_mana)
            .map_err(|e| CommonError::InternalError(format!("Failed to set initial mana: {}", e)))?;
        Ok(ctx)
    }

    /// Create a new context with ledger path (convenience method for tests).
    pub fn new_with_ledger_path(
        current_identity_str: &str,
        ledger_path: PathBuf,
        signer: Arc<dyn Signer>,
    ) -> Result<Arc<Self>, CommonError> {
        let current_identity = Did::from_str(current_identity_str)
            .map_err(|e| CommonError::InternalError(format!("Invalid DID: {}", e)))?;

        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let mesh_network_service = Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new()));
        let did_resolver = Arc::new(icn_identity::KeyDidResolver);
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
        let parameters = Arc::new(DashMap::new());
        let policy_enforcer = None;
        let time_provider = Arc::new(icn_common::SystemTimeProvider);
        let mana_ledger = SimpleManaLedger::new(ledger_path);

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
            dag_store: Arc::new(DagStoreMutexType::new(StubDagStore::new()))
                as Arc<DagStoreMutexType<DagStorageService>>,
            reputation_store,
            parameters,
            policy_enforcer,
            time_provider,
            default_receipt_wait_ms: 30000,
        }))
    }

    /// Create a new context with ledger path and time provider (convenience method for tests).
    pub fn new_with_ledger_path_and_time(
        current_identity_str: &str,
        ledger_path: PathBuf,
        time_provider: Arc<dyn icn_common::TimeProvider>,
        signer: Arc<dyn Signer>,
    ) -> Result<Arc<Self>, CommonError> {
        let current_identity = Did::from_str(current_identity_str)
            .map_err(|e| CommonError::InternalError(format!("Invalid DID: {}", e)))?;

        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let mesh_network_service = Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new()));
        let did_resolver = Arc::new(icn_identity::KeyDidResolver);
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
        let parameters = Arc::new(DashMap::new());
        let policy_enforcer = None;
        let mana_ledger = SimpleManaLedger::new(ledger_path);

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
            dag_store: Arc::new(DagStoreMutexType::new(StubDagStore::new()))
                as Arc<DagStoreMutexType<DagStorageService>>,
            reputation_store,
            parameters,
            policy_enforcer,
            time_provider,
            default_receipt_wait_ms: 30000,
        }))
    }

    /// Create a new context with real libp2p (convenience method for integration tests).
    #[cfg(feature = "enable-libp2p")]
    pub async fn new_with_real_libp2p(
        node_did_string: &str,
        listen_addrs: Vec<libp2p::Multiaddr>,
        bootstrap_peers: Option<Vec<(libp2p::PeerId, libp2p::Multiaddr)>>,
        signer: Arc<dyn Signer>,
    ) -> Result<Arc<Self>, CommonError> {
        let dag_store = Arc::new(DagStoreMutexType::new(StubDagStore::new()))
            as Arc<DagStoreMutexType<DagStorageService>>;
        let mana_ledger_path = PathBuf::from("./temp_mana_ledger");
        let reputation_db_path = PathBuf::from("./temp_reputation_db");
        let did_resolver = Arc::new(icn_identity::KeyDidResolver);
        
        Self::new_with_real_libp2p_and_mdns(
            node_did_string,
            listen_addrs,
            bootstrap_peers,
            dag_store,
            mana_ledger_path,
            reputation_db_path,
            true, // enable_mdns
            signer,
            did_resolver,
        ).await
    }

                // === CLEAN CONFIGURATION METHODS ===

    /// Create a new context for production with all production services.
    pub fn new_for_production(
        current_identity: Did,
        signer: Arc<dyn Signer>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
        dag_store: Arc<DagStoreMutexType<DagStorageService>>,
        mana_ledger: SimpleManaLedger,
        reputation_store: Arc<dyn icn_reputation::ReputationStore>,
        policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
        time_provider: Arc<dyn icn_common::TimeProvider>,
        network_service: Arc<dyn icn_network::NetworkService>,
    ) -> Arc<Self> {
        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let parameters = Arc::new(DashMap::new());

        // Use real production network service
        let mesh_network_service = Arc::new(MeshNetworkServiceType::Default(
            DefaultMeshNetworkService::new(network_service, signer.clone())
        ));

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

    /// Create a new context for development with mixed services.
    pub fn new_for_development(
        current_identity: Did,
        signer: Arc<dyn Signer>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
        dag_store: Arc<DagStoreMutexType<DagStorageService>>,
        mana_ledger: SimpleManaLedger,
        network_service: Option<Arc<dyn icn_network::NetworkService>>,
    ) -> Arc<Self> {
        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
        let parameters = Arc::new(DashMap::new());
        let policy_enforcer = None;
        let time_provider = Arc::new(icn_common::SystemTimeProvider);

        // Use real network service if provided, otherwise stub
        let mesh_network_service = match network_service {
            Some(network_service) => Arc::new(MeshNetworkServiceType::Default(
                DefaultMeshNetworkService::new(network_service, signer.clone())
            )),
            None => Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new())),
        };

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

    /// Create a new context for testing with all stub services.
    pub fn new_for_testing(
        current_identity: Did,
        initial_mana: Option<u64>,
    ) -> Result<Arc<Self>, CommonError> {
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
        let mana_ledger = SimpleManaLedger::new(PathBuf::from("./temp_mana_ledger_test"));

        let ctx = Arc::new(Self {
            current_identity: current_identity.clone(),
            mana_ledger,
            pending_mesh_jobs_tx: tx,
            pending_mesh_jobs_rx: TokioMutex::new(rx),
            job_states,
            governance_module,
            mesh_network_service,
            signer,
            did_resolver,
            dag_store: Arc::new(DagStoreMutexType::new(StubDagStore::new()))
                as Arc<DagStoreMutexType<DagStorageService>>,
            reputation_store,
            parameters,
            policy_enforcer,
            time_provider,
            default_receipt_wait_ms: 30000,
        });

        // Set initial mana if provided
        if let Some(mana) = initial_mana {
            ctx.mana_ledger.set_balance(&current_identity, mana)
                .map_err(|e| CommonError::InternalError(format!("Failed to set initial mana: {}", e)))?;
        }

        Ok(ctx)
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

    /// Create a new context with real libp2p and mDNS services.
    #[cfg(feature = "enable-libp2p")]
    pub async fn new_with_real_libp2p_and_mdns(
        node_did_string: &str,
        listen_addrs: Vec<libp2p::Multiaddr>,
        bootstrap_peers: Option<Vec<(libp2p::PeerId, libp2p::Multiaddr)>>,
        dag_store: Arc<DagStoreMutexType<DagStorageService>>,
        mana_ledger_path: PathBuf,
        _reputation_db_path: PathBuf,
        enable_mdns: bool,
        signer: Arc<dyn Signer>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
    ) -> Result<Arc<Self>, CommonError> {
        use icn_network::libp2p_service::NetworkConfig;
        use std::str::FromStr;

        // Parse DID from string
        let current_identity = Did::from_str(node_did_string)
            .map_err(|e| CommonError::InternalError(format!("Invalid DID: {}", e)))?;

        // Create libp2p network config
        let config = NetworkConfig {
            listen_addresses: listen_addrs,
            bootstrap_peers: bootstrap_peers.unwrap_or_default(),
            enable_mdns,
            ..Default::default()
        };

        // Create libp2p network service
        let network_service = Arc::new(
            icn_network::libp2p_service::Libp2pNetworkService::new(config)
                .await
                .map_err(|e| {
                    CommonError::NetworkError(format!("Failed to create libp2p service: {}", e))
                })?,
        );

        let mesh_network_service = Arc::new(MeshNetworkServiceType::Default(
            DefaultMeshNetworkService::new(network_service, signer.clone()),
        ));

        // Use provided DAG store
        let dag_store = dag_store;

        // Create mana ledger
        let mana_ledger = SimpleManaLedger::new(mana_ledger_path);

        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
        let parameters = Arc::new(DashMap::new());
        let policy_enforcer = None;
        let time_provider = Arc::new(icn_common::SystemTimeProvider);

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
            dag_store,
            reputation_store,
            parameters,
            policy_enforcer,
            time_provider,
            default_receipt_wait_ms: 30000,
        }))
    }

    /// Create a new context with custom mana ledger and time provider.
    pub fn new_with_mana_ledger_and_time(params: RuntimeContextParams) -> Arc<Self> {
        let RuntimeContextParams {
            current_identity,
            mesh_network_service,
            signer,
            did_resolver,
            dag_store,
            mana_ledger,
            reputation_path: _reputation_path,
            policy_enforcer,
            time_provider,
        } = params;
        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
        let parameters = Arc::new(DashMap::new());

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
        JOBS_SUBMITTED.inc();
        PENDING_JOBS_GAUGE.inc();
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
        let receipt_bytes = bincode::serialize(receipt).map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to serialize receipt: {}", e))
        })?;

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
            let mut dag_store = self.dag_store.lock().await;
            dag_store.put(&block).await.map_err(|e| {
                HostAbiError::DagOperationFailed(format!("Failed to store receipt: {}", e))
            })?;
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

        let mut gov = self.governance_module.lock().await;
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
                let mut dag_store = self.dag_store.lock().await;
                dag_store.put(&block).await.map_err(|e| {
                    HostAbiError::DagOperationFailed(format!(
                        "Failed to store proposal body: {}",
                        e
                    ))
                })?;
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
            .map_err(|e| {
                HostAbiError::InternalError(format!("Failed to submit proposal: {}", e))
            })?;

        let proposal = gov
            .get_proposal(&pid)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to get proposal: {}", e)))?
            .ok_or_else(|| {
                HostAbiError::InternalError("Proposal just inserted should exist".to_string())
            })?;

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

        let mut gov = self.governance_module.lock().await;
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

        let mut gov = self.governance_module.lock().await;
        let (status, (yes, no, abstain)) = gov
            .close_voting_period(&proposal_id)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to close voting: {}", e)))?;

        let proposal = gov
            .get_proposal(&proposal_id)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to get proposal: {}", e)))?
            .ok_or_else(|| {
                HostAbiError::InternalError("Proposal should exist after closing".to_string())
            })?;

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

        let mut gov = self.governance_module.lock().await;
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
                        log::warn!(
                            "Failed to broadcast executed proposal {:?}: {}",
                            proposal_id,
                            e
                        );
                    }
                }
                Err(e) => {
                    return Err(HostAbiError::InternalError(format!(
                        "Failed to execute proposal: {}",
                        e
                    )));
                }
            }
        } else {
            return Err(HostAbiError::InvalidParameters(
                "Proposal not found".to_string(),
            ));
        }

        Ok(())
    }

    /// Delegate vote to another DID.
    pub async fn delegate_vote(&self, from_did: &str, to_did: &str) -> Result<(), HostAbiError> {
        let from = Did::from_str(from_did)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid from DID: {}", e)))?;
        let to = Did::from_str(to_did)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid to DID: {}", e)))?;

        // Only allow delegating your own vote
        if from != self.current_identity {
            return Err(HostAbiError::PermissionDenied(
                "Can only delegate your own vote".to_string(),
            ));
        }

        let mut gov = self.governance_module.lock().await;
        gov.delegate_vote(from, to)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to delegate vote: {}", e)))?;

        Ok(())
    }

    /// Revoke vote delegation.
    pub async fn revoke_delegation(&self, from_did: &str) -> Result<(), HostAbiError> {
        let from = Did::from_str(from_did)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid from DID: {}", e)))?;

        // Only allow revoking your own delegation
        if from != self.current_identity {
            return Err(HostAbiError::PermissionDenied(
                "Can only revoke your own delegation".to_string(),
            ));
        }

        let mut gov = self.governance_module.lock().await;
        gov.revoke_delegation(from);
        Ok(())
    }

    /// Update a system parameter.
    async fn update_parameter(&self, key: String, value: String) -> Result<(), HostAbiError> {
        self.parameters.insert(key.clone(), value.clone());
        log::info!("Updated parameter {} to {}", key, value);
        Ok(())
    }

    /// Spawn the mesh job manager task.
    pub async fn spawn_mesh_job_manager(self: Arc<Self>) {
        let ctx = self.clone();

        tokio::spawn(async move {
            log::info!("Starting mesh job manager background task");

            // Get exclusive access to the receiver
            let mut rx = ctx.pending_mesh_jobs_rx.lock().await;

            loop {
                match rx.recv().await {
                    Some(job) => {
                        let job_id = job.id.clone();
                        log::info!("Job manager received job: {:?}", job_id);

                        // Store the job in the job_states map with Pending state
                        ctx.job_states.insert(job_id.clone(), JobState::Pending);
                        PENDING_JOBS_GAUGE.dec();

                        // For now, just handle CCL WASM jobs with auto-execution
                        // Regular mesh jobs would go through the full lifecycle (announce, bid, etc.)
                        if job.spec.kind.is_ccl_wasm() {
                            JOBS_ACTIVE_GAUGE.inc();
                            let start = std::time::Instant::now();
                            log::info!("Auto-executing CCL WASM job: {:?}", job_id);

                            // Spawn a task to handle CCL WASM execution
                            let ctx_clone = ctx.clone();
                            let job_clone = job.clone();

                            tokio::spawn(async move {
                                match Self::execute_ccl_wasm_job(&ctx_clone, &job_clone).await {
                                    Ok(receipt) => {
                                        log::info!(
                                            "CCL WASM job {:?} completed successfully",
                                            job_clone.id
                                        );
                                        JOBS_COMPLETED.inc();
                                        JOB_PROCESS_TIME.observe(start.elapsed().as_secs_f64());
                                        JOBS_ACTIVE_GAUGE.dec();
                                        ctx_clone.job_states.insert(
                                            job_clone.id.clone(),
                                            JobState::Completed { receipt },
                                        );
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "CCL WASM job {:?} failed: {}",
                                            job_clone.id,
                                            e
                                        );
                                        JOBS_FAILED.inc();
                                        JOB_PROCESS_TIME.observe(start.elapsed().as_secs_f64());
                                        JOBS_ACTIVE_GAUGE.dec();
                                        ctx_clone.job_states.insert(
                                            job_clone.id.clone(),
                                            JobState::Failed {
                                                reason: e.to_string(),
                                            },
                                        );
                                    }
                                }
                            });
                        } else {
                            // For non-CCL WASM jobs, we'll implement full mesh lifecycle later
                            // For now, just keep them in Pending state
                            log::info!("Non-CCL WASM job {:?} queued as pending (full mesh lifecycle not yet implemented)", job_id);
                        }
                    }
                    None => {
                        log::warn!("Job manager channel closed, stopping background task");
                        break;
                    }
                }
            }

            log::info!("Mesh job manager background task stopped");
        });

        log::info!("Mesh job manager spawned successfully");
    }

    /// Execute a CCL WASM job using the built-in executor
    async fn execute_ccl_wasm_job(
        ctx: &Arc<RuntimeContext>,
        job: &ActualMeshJob,
    ) -> Result<icn_identity::ExecutionReceipt, HostAbiError> {
        use crate::executor::{JobExecutor, WasmExecutor, WasmExecutorConfig};

        // Create a WASM executor
        let executor = WasmExecutor::new(
            ctx.clone(),
            ctx.signer.clone(),
            WasmExecutorConfig::default(),
        );

        // Execute the job and anchor the receipt
        let _receipt_cid = executor.execute_and_anchor_job(job).await?;

        // Get the receipt by executing the job directly
        let receipt = executor
            .execute_job(job)
            .await
            .map_err(|e| HostAbiError::InternalError(format!("WASM execution failed: {}", e)))?;

        Ok(receipt)
    }

    /// Spawn the mana regenerator task with specified amount and interval.
    pub async fn spawn_mana_regenerator(
        self: Arc<Self>, 
        regeneration_amount: u64, 
        interval: StdDuration
    ) {
        let ctx = self.clone();

        tokio::spawn(async move {
            log::info!("Starting mana regenerator background task with amount {} every {:?}", regeneration_amount, interval);

            loop {
                tokio::time::sleep(interval).await;
                
                // Get all accounts and regenerate mana based on reputation
                let accounts = ctx.mana_ledger.all_accounts();
                for account in accounts {
                    let reputation = ctx.reputation_store.get_reputation(&account);
                    let regeneration = regeneration_amount * (reputation as u64).max(1);
                    
                    if let Err(e) = ctx.mana_ledger.credit(&account, regeneration) {
                        log::warn!("Failed to regenerate mana for account {}: {}", account, e);
                    } else {
                        log::debug!("Regenerated {} mana for account {}", regeneration, account);
                    }
                }
            }
        });

        log::info!("Mana regenerator spawned successfully");
    }

    /// Perform a single integrity check on the DAG store.
    pub async fn integrity_check_once(&self) -> Result<(), CommonError> {
        log::info!("Performing integrity check on DAG store");
        
        // This is a simplified integrity check - in a real implementation,
        // this would verify CIDs match content, check signatures, etc.
        
        // For now, just return success
        Ok(())
    }

    /// Shutdown network services.
    pub async fn shutdown_network(&self) -> Result<(), CommonError> {
        // For now, this is a stub implementation
        // In a full implementation, this would properly shutdown the network service
        log::info!("Network shutdown requested (stub implementation)");
        Ok(())
    }

    /// Get access to the underlying libp2p service if available.
    #[cfg(feature = "enable-libp2p")]
    pub fn get_libp2p_service(
        &self,
    ) -> Result<Arc<icn_network::libp2p_service::Libp2pNetworkService>, CommonError> {
        match &*self.mesh_network_service {
            MeshNetworkServiceType::Default(default_service) => {
                default_service.get_underlying_broadcast_service()
            }
            MeshNetworkServiceType::Stub(_) => Err(CommonError::InternalError(
                "Cannot get libp2p service from stub implementation".to_string(),
            )),
        }
    }

    #[cfg(not(feature = "enable-libp2p"))]
    pub fn get_libp2p_service(&self) -> Result<(), CommonError> {
        Err(CommonError::InternalError(
            "libp2p feature not enabled".to_string(),
        ))
    }
}
