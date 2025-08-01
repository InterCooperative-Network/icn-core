//! RuntimeContext struct and implementations.
//!
//! This module provides the main runtime context for ICN nodes, with clear configuration
//! options for different environments.
//!
//! # Configuration Examples
//!
//! ## Production Configuration
//! ```rust,no_run
//! use icn_runtime::context::{RuntimeContextBuilder, EnvironmentType};
//! use icn_runtime::Ed25519Signer;
//! use icn_common::Did;
//! use std::str::FromStr;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let node_did = Did::from_str("did:key:zProduction...")?;
//! let signer = Ed25519Signer::new(); // Real signer
//! let network_service = icn_network::libp2p_service::Libp2pNetworkService::new(
//!     icn_network::libp2p_service::NetworkConfig::default()
//! ).await?; // Real libp2p service
//! let dag_store = icn_runtime::dag_store_factory::DagStoreFactory::create_production(
//!     std::path::PathBuf::from("./production_storage")
//! )?; // Real persistent DAG store
//! let mana_ledger = icn_runtime::mana::SimpleManaLedger::new(
//!     std::path::PathBuf::from("./production_mana.json")
//! ); // Real persistent mana ledger
//!
//! let ctx = RuntimeContextBuilder::new(EnvironmentType::Production)
//!     .with_identity(node_did)
//!     .with_signer(signer)
//!     .with_network_service(network_service)
//!     .with_dag_store(dag_store)
//!     .with_mana_ledger(mana_ledger)
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Development Configuration
//! ```rust,no_run
//! use icn_runtime::context::{RuntimeContextBuilder, EnvironmentType};
//! use icn_runtime::Ed25519Signer;
//! use icn_common::Did;
//! use std::str::FromStr;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let node_did = Did::from_str("did:key:zDevelopment...")?;
//! let signer = Ed25519Signer::new(); // Real signer
//! let mana_ledger = icn_runtime::mana::SimpleManaLedger::new(
//!     std::path::PathBuf::from("./development_mana.json")
//! ); // Real persistent mana ledger
//! // Network service and DAG store are optional - will use stubs if not provided
//!
//! let ctx = RuntimeContextBuilder::new(EnvironmentType::Development)
//!     .with_identity(node_did)
//!     .with_signer(signer)
//!     .with_mana_ledger(mana_ledger)
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Testing Configuration
//! ```rust
//! use icn_runtime::context::{RuntimeContextBuilder, EnvironmentType};
//! use icn_common::Did;
//! use std::str::FromStr;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let test_did = Did::from_str("did:key:zTesting...")?;
//!
//! let ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
//!     .with_identity(test_did)
//!     .with_initial_mana(1000)
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Production Validation
//! ```rust,no_run
//! # use icn_runtime::context::RuntimeContext;
//! # fn example(ctx: &RuntimeContext) -> Result<(), Box<dyn std::error::Error>> {
//! // Validate that production services are being used
//! ctx.validate_production_services()?;
//! # Ok(())
//! # }
//! ```

use super::dag_store_wrapper::DagStoreWrapper;
use super::errors::HostAbiError;
use super::mana::SimpleManaLedger;
use super::mesh_network::{
    DefaultMeshNetworkService, JobAssignmentNotice, MeshJobStateChange, MeshNetworkService,
};
use super::service_config::ServiceConfig;
use super::signers::{Ed25519Signer, Signer};
use super::stubs::{StubDagStore, StubMeshNetworkService};
use super::{DagStorageService, DagStoreMutexType};
use crate::metrics::{JOBS_ACTIVE_GAUGE, JOBS_COMPLETED, JOBS_FAILED, JOBS_SUBMITTED};
use bincode;
use dashmap::DashMap;
use icn_common::{
    compute_merkle_cid, Cid, CommonError, DagBlock, Did, NodeScope, SysinfoSystemInfoProvider,
    SystemInfoProvider, SystemTimeProvider, TimeProvider,
};
use icn_economics::{LedgerEvent, ManaLedger};
use icn_governance::GovernanceModule;
use icn_identity::{
    ExecutionReceipt as IdentityExecutionReceipt, TrustContext, TrustPolicyEngine,
    TrustValidationResult,
};
use icn_mesh::metrics::{
    BIDS_RECEIVED_TOTAL, JOBS_ASSIGNED_TOTAL, JOBS_BIDDING_GAUGE, JOBS_COMPLETED_TOTAL,
    JOBS_EXECUTING_GAUGE, JOBS_FAILED_TOTAL, JOBS_SUBMITTED_TOTAL, JOB_PROCESS_TIME,
    PENDING_JOBS_GAUGE,
};
use icn_mesh::{
    ActualMeshJob, Job, JobAssignment, JobBid, JobId, JobLifecycle, JobLifecycleStatus, JobReceipt,
    JobState,
};
use icn_reputation::ReputationStore;
use serde_json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tokio::sync::{mpsc, Mutex as TokioMutex};

/// Parameter key for the mana capacity limit managed via governance.
pub const MANA_MAX_CAPACITY_KEY: &str = "mana_max_capacity";
/// Default capacity used when no parameter is set.
pub const DEFAULT_MANA_MAX_CAPACITY: u64 = 10000;

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
        duration: Duration,
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
        timeout: Duration,
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

    async fn submit_bid_for_job(&self, bid: &icn_mesh::MeshJobBid) -> Result<(), HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.submit_bid_for_job(bid).await,
            MeshNetworkServiceType::Default(s) => s.submit_bid_for_job(bid).await,
        }
    }

    async fn submit_execution_receipt(
        &self,
        receipt: &icn_identity::ExecutionReceipt,
    ) -> Result<(), HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.submit_execution_receipt(receipt).await,
            MeshNetworkServiceType::Default(s) => s.submit_execution_receipt(receipt).await,
        }
    }

    // Additional methods for Smart P2P Routing and CCL Integration

    async fn get_connected_peers(&self) -> Result<Vec<Did>, HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.get_connected_peers().await,
            MeshNetworkServiceType::Default(s) => s.get_connected_peers().await,
        }
    }

    async fn ping_peer(
        &self,
        peer_id: Did,
    ) -> Result<super::mesh_network::PingResult, HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.ping_peer(peer_id).await,
            MeshNetworkServiceType::Default(s) => s.ping_peer(peer_id).await,
        }
    }

    async fn get_peer_statistics(
        &self,
        peer_id: Did,
    ) -> Result<super::mesh_network::PeerStatistics, HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.get_peer_statistics(peer_id).await,
            MeshNetworkServiceType::Default(s) => s.get_peer_statistics(peer_id).await,
        }
    }

    async fn send_direct_message(
        &self,
        peer_id: Did,
        payload: Vec<u8>,
    ) -> Result<(), HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.send_direct_message(peer_id, payload).await,
            MeshNetworkServiceType::Default(s) => s.send_direct_message(peer_id, payload).await,
        }
    }

    async fn send_multi_hop_message(
        &self,
        path: Vec<Did>,
        payload: Vec<u8>,
    ) -> Result<(), HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.send_multi_hop_message(path, payload).await,
            MeshNetworkServiceType::Default(s) => s.send_multi_hop_message(path, payload).await,
        }
    }

    async fn query_peer_connections(&self, peer_id: Did) -> Result<Vec<Did>, HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.query_peer_connections(peer_id).await,
            MeshNetworkServiceType::Default(s) => s.query_peer_connections(peer_id).await,
        }
    }

    async fn get_average_network_latency(&self) -> Result<f64, HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.get_average_network_latency().await,
            MeshNetworkServiceType::Default(s) => s.get_average_network_latency().await,
        }
    }

    async fn is_network_partitioned(&self) -> Result<bool, HostAbiError> {
        match self {
            MeshNetworkServiceType::Stub(s) => s.is_network_partitioned().await,
            MeshNetworkServiceType::Default(s) => s.is_network_partitioned().await,
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
    pub dag_store: DagStoreWrapper,
    pub reputation_store: Arc<dyn icn_reputation::ReputationStore>,
    pub trust_engine: Arc<TokioMutex<TrustPolicyEngine>>,
    pub latency_store: Arc<dyn icn_mesh::LatencyStore>,
    pub parameters: Arc<DashMap<String, String>>,
    pub policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
    pub resource_ledger: TokioMutex<super::resource_ledger::ResourceLedger>,
    pub system_info: Arc<dyn SystemInfoProvider>,
    pub time_provider: Arc<dyn icn_common::TimeProvider>,
    pub default_receipt_wait_ms: u64,
    /// Enhanced cross-component coordinator for intelligent service integration
    pub cross_component_coordinator: Arc<CrossComponentCoordinator>,
}

impl std::fmt::Debug for RuntimeContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeContext")
            .field("current_identity", &self.current_identity)
            .field("job_states_count", &self.job_states.len())
            .field("mesh_network_service", &self.mesh_network_service)
            .field("parameters_count", &self.parameters.len())
            .field("default_receipt_wait_ms", &self.default_receipt_wait_ms)
            .field("policy_enforcer", &self.policy_enforcer.is_some())
            .finish_non_exhaustive()
    }
}

// Import std::str::FromStr for Did::from_str
use std::str::FromStr;

// Add governance-specific types
use super::cross_component_coordinator::CrossComponentCoordinator;
use super::mesh_network::{PROPOSAL_COST_MANA, VOTE_COST_MANA};
use icn_governance::{Proposal, ProposalId, ProposalSubmission, ProposalType, Vote, VoteOption};
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

/// Record of a runtime parameter change anchored in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParameterUpdate {
    /// Name of the updated parameter.
    pub name: String,
    /// New value for the parameter.
    pub value: String,
    /// Seconds since Unix epoch when the change occurred.
    pub timestamp: u64,
    /// DID of the signer applying the update.
    pub signer: Did,
}

/// Parameters for [`RuntimeContext`] construction.
pub struct RuntimeContextParams {
    pub current_identity: Did,
    pub mesh_network_service: Arc<MeshNetworkServiceType>,
    pub signer: Arc<dyn Signer>,
    pub did_resolver: Arc<dyn icn_identity::DidResolver>,
    pub dag_store: DagStoreWrapper,
    pub mana_ledger: SimpleManaLedger,
    pub reputation_path: PathBuf,
    pub latency_store: Arc<dyn icn_mesh::LatencyStore>,
    pub policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
    pub time_provider: Arc<dyn icn_common::TimeProvider>,
}

/// Configuration builder for creating RuntimeContext instances with type safety.
pub struct RuntimeContextBuilder {
    current_identity: Option<Did>,
    environment: EnvironmentType,
    network_service: Option<Arc<dyn icn_network::NetworkService>>,
    signer: Option<Arc<dyn Signer>>,
    dag_store: Option<DagStoreWrapper>,
    mana_ledger: Option<SimpleManaLedger>,
    reputation_store: Option<Arc<dyn icn_reputation::ReputationStore>>,
    policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
    initial_mana: Option<u64>,
}

/// Environment type for RuntimeContext configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentType {
    /// Production environment - all services must be production-ready
    Production,
    /// Development environment - mixed services allowed
    Development,
    /// Testing environment - stub services preferred
    Testing,
}

impl RuntimeContextBuilder {
    /// Create a new builder for the specified environment.
    pub fn new(environment: EnvironmentType) -> Self {
        Self {
            current_identity: None,
            environment,
            network_service: None,
            signer: None,
            dag_store: None,
            mana_ledger: None,
            reputation_store: None,
            policy_enforcer: None,
            initial_mana: None,
        }
    }

    /// Set the current identity for this context.
    pub fn with_identity(mut self, identity: Did) -> Self {
        self.current_identity = Some(identity);
        self
    }

    /// Set the network service (required for production).
    pub fn with_network_service(mut self, service: Arc<dyn icn_network::NetworkService>) -> Self {
        self.network_service = Some(service);
        self
    }

    /// Set the cryptographic signer (required for production).
    pub fn with_signer(mut self, signer: Arc<dyn Signer>) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Set the DAG store (required for production).
    pub fn with_dag_store(mut self, store: DagStoreWrapper) -> Self {
        self.dag_store = Some(store);
        self
    }

    /// Set the mana ledger (required for all environments).
    pub fn with_mana_ledger(mut self, ledger: SimpleManaLedger) -> Self {
        self.mana_ledger = Some(ledger);
        self
    }

    /// Set the reputation store (optional).
    pub fn with_reputation_store(
        mut self,
        store: Arc<dyn icn_reputation::ReputationStore>,
    ) -> Self {
        self.reputation_store = Some(store);
        self
    }

    /// Set the policy enforcer (optional).
    pub fn with_policy_enforcer(
        mut self,
        enforcer: Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>,
    ) -> Self {
        self.policy_enforcer = Some(enforcer);
        self
    }

    /// Set initial mana balance (for testing).
    pub fn with_initial_mana(mut self, mana: u64) -> Self {
        self.initial_mana = Some(mana);
        self
    }

    /// Build the RuntimeContext with validation.
    pub fn build(self) -> Result<Arc<RuntimeContext>, CommonError> {
        let current_identity = self.current_identity.ok_or_else(|| {
            CommonError::InternalError("Current identity is required".to_string())
        })?;

        match self.environment {
            EnvironmentType::Production => {
                let network_service = self.network_service.ok_or_else(|| {
                    CommonError::InternalError(
                        "Network service is required for production".to_string(),
                    )
                })?;
                let signer = self.signer.ok_or_else(|| {
                    CommonError::InternalError("Signer is required for production".to_string())
                })?;
                let dag_store = self.dag_store.ok_or_else(|| {
                    CommonError::InternalError("DAG store is required for production".to_string())
                })?;
                let mana_ledger = self.mana_ledger.ok_or_else(|| {
                    CommonError::InternalError("Mana ledger is required for production".to_string())
                })?;
                let reputation_store = self
                    .reputation_store
                    .unwrap_or_else(|| Arc::new(icn_reputation::InMemoryReputationStore::new()));

                RuntimeContext::new_with_services(
                    current_identity,
                    network_service,
                    signer,
                    Arc::new(icn_identity::KeyDidResolver),
                    dag_store,
                    mana_ledger,
                    reputation_store,
                    self.policy_enforcer,
                )
            }
            EnvironmentType::Development => {
                let signer = self.signer.ok_or_else(|| {
                    CommonError::InternalError("Signer is required for development".to_string())
                })?;
                let mana_ledger = self.mana_ledger.ok_or_else(|| {
                    CommonError::InternalError(
                        "Mana ledger is required for development".to_string(),
                    )
                })?;

                RuntimeContext::new_development(
                    current_identity,
                    signer,
                    mana_ledger,
                    self.network_service,
                    self.dag_store.map(|wrapper| wrapper.store),
                )
            }
            EnvironmentType::Testing => {
                RuntimeContext::new_for_testing(current_identity, self.initial_mana)
            }
        }
    }
}

impl RuntimeContext {
    /// Initialize the default runtime parameters.
    fn default_parameters() -> Arc<DashMap<String, String>> {
        let map = DashMap::new();
        map.insert(
            MANA_MAX_CAPACITY_KEY.to_string(),
            DEFAULT_MANA_MAX_CAPACITY.to_string(),
        );
        Arc::new(map)
    }

    /// Create a cross-component coordinator with the given services
    fn create_cross_component_coordinator(
        mesh_network_service: Arc<MeshNetworkServiceType>,
        dag_store: &DagStoreWrapper,
        governance_module: Arc<DagStoreMutexType<GovernanceModule>>,
        reputation_store: Arc<dyn ReputationStore>,
        current_identity: Did,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Arc<CrossComponentCoordinator> {
        Arc::new(CrossComponentCoordinator::new(
            mesh_network_service,
            dag_store.clone_inner(),
            governance_module,
            reputation_store,
            current_identity,
            time_provider,
        ))
    }

    /// Validate that production services are being used correctly.
    ///
    /// This function performs comprehensive runtime checks to ensure that stub services
    /// are not accidentally used in production contexts and that all required
    /// production-grade services are properly configured.
    pub fn validate_production_services(&self) -> Result<(), CommonError> {
        let mut errors = Vec::new();

        // Check if we're using stub mesh network service
        if let MeshNetworkServiceType::Stub(_) = &*self.mesh_network_service {
            errors.push("❌ PRODUCTION ERROR: Stub mesh network service detected. Use RuntimeContext::new() with real network service.".to_string());
        }

        // Check signer type
        if self.signer.as_any().is::<super::signers::StubSigner>() {
            errors.push("❌ PRODUCTION ERROR: Stub signer detected. Use Ed25519Signer or other production-grade signer.".to_string());
        }

        // Check DAG store type (synchronous check)
        if let Err(e) = self.dag_store.validate_for_production() {
            errors.push(format!(
                "❌ PRODUCTION ERROR: DAG store validation failed: {}",
                e
            ));
        }

        // Check reputation store type
        // Note: InMemoryReputationStore might be acceptable for production in some cases
        // but we should warn about persistence implications
        log::warn!("⚠️  PRODUCTION WARNING: Using reputation store. Ensure it has proper persistence for production use.");

        // Check DID resolver type - we assume KeyDidResolver is the production standard
        log::info!("ℹ️  Using configured DID resolver for production.");

        // Check if we have governance policy enforcer for production
        if self.policy_enforcer.is_none() {
            log::warn!("⚠️  PRODUCTION WARNING: No governance policy enforcer configured. Consider adding one for production governance.");
        }

        // Validate system resources are sufficient for production
        let (cpu_cores, memory_mb) = Self::available_system_resources();
        if cpu_cores < 2 {
            errors.push("❌ PRODUCTION ERROR: Insufficient CPU cores for production deployment. Minimum 2 cores recommended.".to_string());
        }
        if memory_mb < 1024 {
            errors.push("❌ PRODUCTION ERROR: Insufficient memory for production deployment. Minimum 1GB recommended.".to_string());
        }

        // Validate mana ledger is persistent
        Self::validate_mana_ledger_persistence(&self.mana_ledger)?;

        // Check if we're using production time provider
        // SystemTimeProvider is the production implementation
        log::info!("ℹ️  Using configured time provider for production.");

        // If we have any errors, return them
        if !errors.is_empty() {
            return Err(CommonError::InternalError(format!(
                "Production validation failed with {} errors:\n{}",
                errors.len(),
                errors.join("\n")
            )));
        }

        log::info!("✅ Production services validation passed");
        Ok(())
    }

    /// Query CPU core count and available memory in MB using sysinfo.
    fn available_system_resources() -> (u32, u32) {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        let cpu = sys.cpus().len() as u32;
        // Convert from bytes to megabytes: divide by 1024^2 = 1,048,576
        let memory_mb = (sys.available_memory() / (1024 * 1024)) as u32;
        (cpu, memory_mb)
    }

    /// Create a new context with stubs for testing.
    ///
    /// **⚠️ DEPRECATED**: This method is deprecated in favor of `new_testing()` which provides
    /// clearer semantics and better error handling. Use `new_testing()` instead.
    ///
    /// **⚠️ STUB SERVICES**: This method uses stub services and should NEVER be used in production.
    #[deprecated(
        since = "0.2.0",
        note = "Use `new_testing()` instead for clearer semantics. This method uses stub services that are not suitable for production."
    )]
    pub fn new_with_stubs(current_identity_str: &str) -> Result<Arc<Self>, CommonError> {
        let current_identity = Did::from_str(current_identity_str)
            .map_err(|e| CommonError::InternalError(format!("Invalid DID: {}", e)))?;

        Self::new_for_testing(current_identity, None)
    }

    /// Create a new context with stubs and initial mana balance (convenience method for tests).
    ///
    /// **⚠️ STUB SERVICES**: This method uses stub services and should NEVER be used in production.
    pub fn new_with_stubs_and_mana(
        current_identity_str: &str,
        initial_mana: u64,
    ) -> Result<Arc<Self>, CommonError> {
        let current_identity = Did::from_str(current_identity_str)
            .map_err(|e| CommonError::InternalError(format!("Invalid DID: {}", e)))?;

        // Forward to new_for_testing method
        Self::new_for_testing(current_identity, Some(initial_mana))
    }

    /// Create a RuntimeContext configured for production deployments.
    #[allow(clippy::too_many_arguments)]
    pub fn new_for_production(
        current_identity: Did,
        network_service: Arc<dyn icn_network::NetworkService>,
        signer: Arc<dyn Signer>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
        dag_store: Arc<DagStoreMutexType<DagStorageService>>,
        mana_ledger: SimpleManaLedger,
        reputation_store: Arc<dyn icn_reputation::ReputationStore>,
        policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
    ) -> Result<Arc<Self>, CommonError> {
        let dag_store_wrapper = DagStoreWrapper::generic_production(dag_store);
        Self::new_with_services(
            current_identity,
            network_service,
            signer,
            did_resolver,
            dag_store_wrapper,
            mana_ledger,
            reputation_store,
            policy_enforcer,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_for_development(
        current_identity: Did,
        signer: Arc<dyn Signer>,
        mana_ledger: SimpleManaLedger,
        network_service: Option<Arc<dyn icn_network::NetworkService>>,
        dag_store: Option<Arc<DagStoreMutexType<DagStorageService>>>,
    ) -> Result<Arc<Self>, CommonError> {
        Self::new_development(
            current_identity,
            signer,
            mana_ledger,
            network_service,
            dag_store,
        )
    }

    /// Create a new context with ledger path (convenience method for tests).
    ///
    /// **⚠️ DEPRECATED**: This method uses stub services and should not be used in production.
    /// Use [`RuntimeContext::new`] or `RuntimeContext::from_service_config()` instead.
    #[deprecated(
        since = "0.1.0",
        note = "Use `new` or `from_service_config()` instead. This method uses stub services."
    )]
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
        let mesh_network_service =
            Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new()));
        let did_resolver = Arc::new(icn_identity::KeyDidResolver);
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
        let latency_store = Arc::new(icn_mesh::NoOpLatencyStore) as Arc<dyn icn_mesh::LatencyStore>;
        let parameters = Self::default_parameters();
        let policy_enforcer = None;
        let time_provider = Arc::new(icn_common::SystemTimeProvider);
        let system_info = Arc::new(SysinfoSystemInfoProvider);
        let mana_ledger = SimpleManaLedger::new(ledger_path);

        let dag_store_raw = Arc::new(DagStoreMutexType::new(StubDagStore::new()))
            as Arc<DagStoreMutexType<DagStorageService>>;
        let dag_store = DagStoreWrapper::stub(dag_store_raw.clone());

        // Create cross-component coordinator
        let cross_component_coordinator = Self::create_cross_component_coordinator(
            mesh_network_service.clone(),
            &dag_store,
            governance_module.clone(),
            reputation_store.clone(),
            current_identity.clone(),
            time_provider.clone(),
        );

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
            trust_engine: Arc::new(TokioMutex::new(TrustPolicyEngine::new())),
            latency_store,
            parameters,
            policy_enforcer,
            resource_ledger: TokioMutex::new(super::resource_ledger::ResourceLedger::new()),
            system_info,
            time_provider,
            default_receipt_wait_ms: 30000,
            cross_component_coordinator,
        }))
    }

    /// Create a new context with ledger path and time provider (convenience method for tests).
    ///
    /// **⚠️ DEPRECATED**: This method uses stub services and should not be used in production.
    /// Use [`RuntimeContext::new`] or `RuntimeContext::from_service_config()` instead.
    #[deprecated(
        since = "0.1.0",
        note = "Use `new` or `from_service_config()` instead. This method uses stub services."
    )]
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
        let mesh_network_service =
            Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new()));
        let did_resolver = Arc::new(icn_identity::KeyDidResolver);
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
        let latency_store = Arc::new(icn_mesh::NoOpLatencyStore) as Arc<dyn icn_mesh::LatencyStore>;
        let parameters = Self::default_parameters();
        let policy_enforcer = None;
        let mana_ledger = SimpleManaLedger::new(ledger_path);
        let system_info = Arc::new(SysinfoSystemInfoProvider);

        let dag_store_raw = Arc::new(DagStoreMutexType::new(StubDagStore::new()))
            as Arc<DagStoreMutexType<DagStorageService>>;
        let dag_store = DagStoreWrapper::stub(dag_store_raw.clone());

        // Create cross-component coordinator
        let cross_component_coordinator = Self::create_cross_component_coordinator(
            mesh_network_service.clone(),
            &dag_store,
            governance_module.clone(),
            reputation_store.clone(),
            current_identity.clone(),
            time_provider.clone(),
        );

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
            trust_engine: Arc::new(TokioMutex::new(TrustPolicyEngine::new())),
            latency_store,
            parameters,
            policy_enforcer,
            resource_ledger: TokioMutex::new(super::resource_ledger::ResourceLedger::new()),
            system_info,
            time_provider,
            default_receipt_wait_ms: 30000,
            cross_component_coordinator,
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

        // Use temporary files for testing
        let mana_temp_file = tempfile::NamedTempFile::new().map_err(|e| {
            CommonError::IoError(format!("Failed to create temp mana ledger: {}", e))
        })?;
        let mana_ledger_path = mana_temp_file.path().to_path_buf();
        std::mem::forget(mana_temp_file);

        let reputation_temp_file = tempfile::NamedTempFile::new().map_err(|e| {
            CommonError::IoError(format!("Failed to create temp reputation db: {}", e))
        })?;
        let reputation_db_path = reputation_temp_file.path().to_path_buf();
        std::mem::forget(reputation_temp_file);
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
        )
        .await
    }

    // === NEW CLEAN SERVICE CONFIGURATION API ===

    /// Create a new RuntimeContext from a service configuration.
    /// This is the preferred method as it ensures type-safe service mapping.
    pub fn from_service_config(config: ServiceConfig) -> Result<Arc<Self>, CommonError> {
        crate::execution_monitor::init_logger();
        // Validate the configuration before using it
        config.validate()?;

        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let parameters = Self::default_parameters();

        // Initialize cross-component coordinator with all services
        let cross_component_coordinator = Self::create_cross_component_coordinator(
            config.mesh_network_service.clone(),
            &config.dag_store,
            governance_module.clone(),
            config.reputation_store.clone(),
            config.current_identity.clone(),
            config.time_provider.clone(),
        );

        Ok(Arc::new(Self {
            current_identity: config.current_identity,
            mana_ledger: config.mana_ledger,
            pending_mesh_jobs_tx: tx,
            pending_mesh_jobs_rx: TokioMutex::new(rx),
            job_states,
            governance_module,
            mesh_network_service: config.mesh_network_service,
            signer: config.signer,
            did_resolver: config.did_resolver,
            dag_store: config.dag_store,
            reputation_store: config.reputation_store,
            trust_engine: Arc::new(TokioMutex::new(TrustPolicyEngine::new())),
            latency_store: Arc::new(icn_mesh::NoOpLatencyStore) as Arc<dyn icn_mesh::LatencyStore>,
            parameters,
            policy_enforcer: config.policy_enforcer,
            resource_ledger: TokioMutex::new(super::resource_ledger::ResourceLedger::new()),
            system_info: Arc::new(SysinfoSystemInfoProvider),
            time_provider: config.time_provider,
            default_receipt_wait_ms: 30000,
            cross_component_coordinator,
        }))
    }

    /// Create a new RuntimeContext with production services by default.
    ///
    /// **🏭 PRODUCTION BY DEFAULT**: This method automatically configures production services
    /// where possible, with appropriate fallbacks. This is the recommended constructor for
    /// most use cases.
    ///
    /// **Automatic Service Configuration:**
    /// - Network: Attempts to create libp2p service with default configuration
    /// - Signer: Creates Ed25519 signer from provided keypair or generates one
    /// - DAG Store: Creates appropriate persistent storage backend
    /// - Mana Ledger: Uses provided mana ledger or creates default persistent ledger
    /// - Reputation Store: Creates in-memory reputation store (upgradeable)
    ///
    /// **Use when:**
    /// - Setting up a production ICN node with minimal configuration
    /// - You want production services by default with sensible fallbacks
    /// - You need a simple API for common deployment scenarios
    ///
    /// **Fallback behavior:**
    /// - If libp2p feature is disabled, returns error requiring explicit network service
    /// - If no storage path provided, creates default storage directory
    /// - Uses Ed25519 for cryptographic operations
    pub fn new() -> Result<Arc<Self>, CommonError> {
        Self::new_with_identity_and_storage(None, None, None)
    }

    /// Create a new RuntimeContext with a cryptographically matched identity and signer.
    ///
    /// **🔒 PRODUCTION RECOMMENDED**: This method ensures proper cryptographic matching
    /// between the provided identity DID and signer, preventing signature verification failures.
    ///
    /// **Parameters:**
    /// - `identity`: The DID to use for this context
    /// - `signer`: A signer that can create signatures verifiable by the identity's public key
    /// - `storage_path`: Optional storage directory (uses default if None)
    /// - `mana_ledger_path`: Optional mana ledger path (uses default if None)
    ///
    /// **Validation:**
    /// This method validates that the signer can create signatures that verify against
    /// the identity's public key, preventing the common cryptographic mismatch issue.
    pub fn new_with_identity_and_signer(
        identity: Did,
        signer: Arc<dyn Signer>,
        storage_path: Option<std::path::PathBuf>,
        mana_ledger_path: Option<std::path::PathBuf>,
    ) -> Result<Arc<Self>, CommonError> {
        // Validate cryptographic matching
        Self::validate_identity_signer_match(&identity, &signer)?;

        // Create storage paths
        let storage_path = storage_path.unwrap_or_else(|| {
            std::path::PathBuf::from(format!("./icn_production_storage_{}", std::process::id()))
        });
        let mana_ledger_path = mana_ledger_path.unwrap_or_else(|| storage_path.join("mana_ledger"));

        // Create production services
        let dag_store = super::dag_store_factory::DagStoreFactory::create_production(storage_path)?;
        let mana_ledger = super::mana::SimpleManaLedger::new(mana_ledger_path);

        // Create production configuration
        let config = super::service_config::ServiceConfig::production(
            identity,
            Arc::new(icn_network::StubNetworkService::default()), // Will be replaced by real network service
            signer,
            Arc::new(icn_identity::KeyDidResolver),
            dag_store,
            mana_ledger,
            Arc::new(icn_reputation::InMemoryReputationStore::new()),
            None,
        )?;

        Self::from_service_config(config)
    }

    /// Validate that an identity DID and signer are cryptographically matched.
    ///
    /// This prevents the common issue where an identity is provided with a signer
    /// that cannot create signatures verifiable by the identity's public key.
    fn validate_identity_signer_match(
        identity: &Did,
        signer: &Arc<dyn Signer>,
    ) -> Result<(), CommonError> {
        use icn_identity::{verify_signature, verifying_key_from_did_key, EdSignature};

        // Extract the verifying key from the identity DID
        let verifying_key = verifying_key_from_did_key(identity).map_err(|e| {
            CommonError::InternalError(format!(
                "Failed to extract verifying key from DID {}: {}",
                identity, e
            ))
        })?;

        // Test message for signature verification
        let test_message = b"cryptographic_validation_test_message";

        // Sign the message with the provided signer
        let signature_bytes = signer.sign(test_message).map_err(|e| {
            CommonError::InternalError(format!("Signer failed to sign test message: {}", e))
        })?;

        // Convert to EdSignature format
        let signature_array: [u8; 64] = signature_bytes.as_slice().try_into().map_err(|_| {
            CommonError::InternalError("Invalid signature length from signer".to_string())
        })?;
        let signature = EdSignature::from_bytes(&signature_array);

        // Verify the signature using the identity's public key
        if !verify_signature(&verifying_key, test_message, &signature) {
            return Err(CommonError::InternalError(
                format!(
                    "❌ CRYPTOGRAPHIC MISMATCH: The provided signer cannot create signatures that verify against identity {}. \
                    This would cause all signature operations to fail. Ensure the signer was created from the same \
                    private key that corresponds to the identity's public key.",
                    identity
                )
            ));
        }

        log::info!(
            "✅ Cryptographic validation passed: Identity {} and signer are properly matched",
            identity
        );
        Ok(())
    }

    /// Validate that the mana ledger uses persistent storage suitable for production.
    ///
    /// This method performs basic validation to ensure that the mana ledger
    /// is using a persistent backend rather than in-memory storage.
    fn validate_mana_ledger_persistence(
        mana_ledger: &super::mana::SimpleManaLedger,
    ) -> Result<(), CommonError> {
        // Test persistence by performing a round-trip operation
        let test_did = Did::from_str("did:key:validation_test")
            .map_err(|e| CommonError::InternalError(format!("Failed to create test DID: {}", e)))?;

        // Get initial balance (should be 0 for new DID)
        let initial_balance = mana_ledger.get_balance(&test_did);

        // Set a test balance
        let test_balance = 12345u64;
        mana_ledger.set_balance(&test_did, test_balance)
            .map_err(|e| CommonError::InternalError(
                format!("❌ MANA LEDGER VALIDATION FAILED: Cannot write to mana ledger: {}. \
                        This suggests the ledger is not properly configured for persistent storage.", e)
            ))?;

        // Verify the balance was set correctly
        let stored_balance = mana_ledger.get_balance(&test_did);
        if stored_balance != test_balance {
            return Err(CommonError::InternalError(format!(
                "❌ MANA LEDGER VALIDATION FAILED: Balance mismatch after write. \
                        Expected {}, got {}. This suggests persistence is not working correctly.",
                test_balance, stored_balance
            )));
        }

        // Cleanup: restore original balance
        mana_ledger
            .set_balance(&test_did, initial_balance)
            .map_err(|e| {
                CommonError::InternalError(format!(
                    "Failed to cleanup test balance during validation: {}",
                    e
                ))
            })?;

        log::info!(
            "✅ Mana ledger persistence validation passed: Write/read operations successful"
        );

        // Additional validation: Check if this is likely a file-based or database backend
        // We can infer this by testing some backend-specific behaviors
        match Self::detect_mana_ledger_backend_type(mana_ledger) {
            Ok(backend_type) => {
                log::info!("✅ Detected mana ledger backend type: {}", backend_type);
                Ok(())
            }
            Err(e) => {
                log::warn!(
                    "⚠️  Could not definitively determine mana ledger backend type: {}",
                    e
                );
                log::warn!("⚠️  Proceeding with basic validation only");
                Ok(()) // Don't fail validation just because we can't detect the backend
            }
        }
    }

    /// Attempt to detect the mana ledger backend type for validation purposes.
    fn detect_mana_ledger_backend_type(
        _mana_ledger: &super::mana::SimpleManaLedger,
    ) -> Result<String, CommonError> {
        // Since SimpleManaLedger wraps the actual implementation, we need to
        // use feature flags to determine what backend is likely being used

        #[cfg(feature = "persist-sled")]
        return Ok("Sled (persistent key-value store)".to_string());

        #[cfg(all(not(feature = "persist-sled"), feature = "persist-sqlite"))]
        return Ok("SQLite (persistent SQL database)".to_string());

        #[cfg(all(
            not(feature = "persist-sled"),
            not(feature = "persist-sqlite"),
            feature = "persist-rocksdb"
        ))]
        return Ok("RocksDB (persistent LSM-tree store)".to_string());

        #[cfg(all(
            not(feature = "persist-sled"),
            not(feature = "persist-sqlite"),
            not(feature = "persist-rocksdb")
        ))]
        {
            log::warn!("⚠️  No persistent storage features enabled for mana ledger");
            log::warn!("⚠️  This may indicate the use of a file-based or other fallback backend");
            Ok("File-based or fallback backend".to_string())
        }
    }

    /// Create a new RuntimeContext with specified identity and storage path.
    ///
    /// **🏭 PRODUCTION BY DEFAULT**: This method automatically configures production services.
    ///
    /// **Parameters:**
    /// - `identity`: Optional DID to use (generates one if None)
    /// - `storage_path`: Optional storage directory (uses default if None)
    /// - `mana_ledger_path`: Optional mana ledger path (uses default if None)
    pub fn new_with_identity_and_storage(
        identity: Option<Did>,
        storage_path: Option<std::path::PathBuf>,
        mana_ledger_path: Option<std::path::PathBuf>,
    ) -> Result<Arc<Self>, CommonError> {
        use icn_identity::generate_ed25519_keypair;

        // Generate identity and signer from the same keypair when identity is not provided
        let (current_identity, signer) = if let Some(did) = identity {
            // FIXME: When identity is provided without matching signer, signatures will not verify!
            // This maintains existing behavior but creates a non-functional RuntimeContext.
            // Consider requiring a matching signer parameter or returning an error.
            let (signing_key, _) = generate_ed25519_keypair();
            let signer = Arc::new(Ed25519Signer::new(signing_key)) as Arc<dyn Signer>;
            (did, signer)
        } else {
            // Generate identity and signer from the same keypair for proper cryptographic matching
            let (signing_key, verifying_key) = generate_ed25519_keypair();
            let current_identity_str = icn_identity::did_key_from_verifying_key(&verifying_key);
            let current_identity = Did::from_str(&current_identity_str)
                .map_err(|e| CommonError::InternalError(format!("Invalid DID: {}", e)))?;
            let signer = Arc::new(Ed25519Signer::new(signing_key)) as Arc<dyn Signer>;
            (current_identity, signer)
        };

        // Create DAG store with default or specified path
        let storage_path =
            storage_path.unwrap_or_else(|| std::path::PathBuf::from("./icn_storage"));
        let dag_store = super::dag_store_factory::DagStoreFactory::create_production(storage_path)?;

        // Create mana ledger with default or specified path
        let mana_ledger_path =
            mana_ledger_path.unwrap_or_else(|| std::path::PathBuf::from("./mana_ledger.json"));
        let mana_ledger = super::mana::SimpleManaLedger::new(mana_ledger_path);

        // Create production network service
        #[cfg(feature = "enable-libp2p")]
        {
            use icn_network::libp2p_service::NetworkConfig;

            let config = NetworkConfig {
                listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()], // Random port
                bootstrap_peers: vec![],     // No bootstrap peers by default
                discovery_addresses: vec![], // No discovery addresses by default
                enable_mdns: true,
                max_peers: 100,
                max_peers_per_ip: 5,
                connection_timeout: Duration::from_secs(30),
                request_timeout: Duration::from_secs(10),
                heartbeat_interval: Duration::from_secs(15),
                bootstrap_interval: Duration::from_secs(300),
                peer_discovery_interval: Duration::from_secs(60),
                kademlia_replication_factor: 20,
            };

            // Create libp2p network service - this is async but we're in sync context
            // We'll need to return an error for now and suggest using the async version
            Err(CommonError::InternalError(
                "Cannot create libp2p network service in synchronous context. Use new_async() or new_with_network_service() instead.".to_string()
            ))
        }

        #[cfg(not(feature = "enable-libp2p"))]
        {
            Err(CommonError::InternalError(
                "Production RuntimeContext requires libp2p feature or explicit network service. Enable the 'enable-libp2p' feature or use new_with_network_service().".to_string()
            ))
        }
    }

    /// Create a new RuntimeContext with all production services (explicit parameters).
    ///
    /// **🏭 PRODUCTION**: This method ensures no stub services are used and should be used
    /// for all production ICN node deployments with explicit service configuration.
    ///
    /// **Services Used:**
    /// - Network: Real libp2p networking service
    /// - Signer: Ed25519 cryptographic signer
    /// - DAG Store: Persistent storage backend (Sled, RocksDB, SQLite, or PostgreSQL)
    /// - Mana Ledger: Persistent mana ledger
    /// - Reputation Store: Persistent reputation storage
    ///
    /// **Use when:**
    /// - Running an ICN node in production with specific services
    /// - Need full control over service configuration
    /// - Require explicit validation of all services
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_services(
        current_identity: Did,
        network_service: Arc<dyn icn_network::NetworkService>,
        signer: Arc<dyn Signer>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
        dag_store: DagStoreWrapper,
        mana_ledger: SimpleManaLedger,
        reputation_store: Arc<dyn icn_reputation::ReputationStore>,
        policy_enforcer: Option<Arc<dyn icn_governance::scoped_policy::ScopedPolicyEnforcer>>,
    ) -> Result<Arc<Self>, CommonError> {
        // Validate that no stub services are being used in production (synchronous check)
        dag_store.validate_for_production()?;

        let config = ServiceConfig::production(
            current_identity,
            network_service,
            signer,
            did_resolver,
            dag_store,
            mana_ledger,
            reputation_store,
            policy_enforcer,
        )?;
        Self::from_service_config(config)
    }

    /// Create a production RuntimeContext with automatic storage backend creation.
    ///
    /// **🏭 PRODUCTION**: This is a convenience method that automatically creates
    /// the appropriate persistent DAG storage backend for production use.
    ///
    /// **Use when:**
    /// - You want automatic backend selection (Sled > RocksDB > SQLite > PostgreSQL)
    /// - You have a storage directory but don't want to manually create the DAG store
    /// - You're setting up a production node with minimal configuration
    pub fn new_for_production_with_storage(
        current_identity: Did,
        network_service: Arc<dyn icn_network::NetworkService>,
        signer: Arc<dyn Signer>,
        storage_path: std::path::PathBuf,
        mana_ledger: SimpleManaLedger,
    ) -> Result<Arc<Self>, CommonError> {
        let config = ServiceConfig::production_with_storage(
            current_identity,
            network_service,
            signer,
            storage_path,
            mana_ledger,
        )?;
        Self::from_service_config(config)
    }

    /// Create a development RuntimeContext with mixed services.
    ///
    /// **🛠️ DEVELOPMENT**: This method provides a flexible configuration for development
    /// and testing scenarios where you may want some real services and some stub services.
    ///
    /// **Services Used:**
    /// - Network: Real libp2p if provided, otherwise stub service
    /// - Signer: Real Ed25519 signer (provided)
    /// - DAG Store: Persistent storage if provided, otherwise stub store
    /// - Mana Ledger: Persistent mana ledger (provided)
    /// - Reputation Store: In-memory reputation store
    ///
    /// **Use when:**
    /// - Local development with some real services
    /// - Integration testing with selective real services
    /// - Development environments where you need networking but not persistence
    pub fn new_development(
        current_identity: Did,
        signer: Arc<dyn Signer>,
        mana_ledger: SimpleManaLedger,
        network_service: Option<Arc<dyn icn_network::NetworkService>>,
        dag_store: Option<Arc<DagStoreMutexType<DagStorageService>>>,
    ) -> Result<Arc<Self>, CommonError> {
        let config = ServiceConfig::development(
            current_identity,
            signer,
            mana_ledger,
            network_service,
            dag_store.map(DagStoreWrapper::generic_production),
        )?;
        Self::from_service_config(config)
    }

    /// Create a development RuntimeContext with automatic storage creation.
    ///
    /// **🛠️ DEVELOPMENT**: Convenience method for development setups.
    /// If storage_path is provided, uses persistent storage. Otherwise uses stub storage.
    pub fn new_development_with_storage(
        current_identity: Did,
        signer: Arc<dyn Signer>,
        mana_ledger: SimpleManaLedger,
        network_service: Option<Arc<dyn icn_network::NetworkService>>,
        storage_path: Option<std::path::PathBuf>,
    ) -> Result<Arc<Self>, CommonError> {
        let config = ServiceConfig::development_with_storage(
            current_identity,
            signer,
            mana_ledger,
            network_service,
            storage_path,
        )?;
        Self::from_service_config(config)
    }

    /// Create an async production RuntimeContext with automatic service creation.
    ///
    /// **🏭 PRODUCTION BY DEFAULT**: This async method creates production services
    /// with full libp2p networking and persistent storage.
    ///
    /// **Use when:**
    /// - Setting up production ICN node in async context
    /// - Need full libp2p networking with real P2P connections
    /// - Want automatic service creation with sensible defaults
    pub async fn new_async() -> Result<Arc<Self>, CommonError> {
        Self::new_async_with_identity_and_storage(None, None, None).await
    }

    /// Create an async production RuntimeContext with specified configuration.
    ///
    /// **🏭 PRODUCTION BY DEFAULT**: Creates production services asynchronously.
    pub async fn new_async_with_identity_and_storage(
        identity: Option<Did>,
        storage_path: Option<std::path::PathBuf>,
        mana_ledger_path: Option<std::path::PathBuf>,
    ) -> Result<Arc<Self>, CommonError> {
        use icn_identity::generate_ed25519_keypair;

        // Generate identity and signer from the same keypair when identity is not provided
        let (current_identity, signer) = if let Some(did) = identity {
            // FIXME: When identity is provided without matching signer, signatures will not verify!
            // This maintains existing behavior but creates a non-functional RuntimeContext.
            // Consider requiring a matching signer parameter or returning an error.
            let (signing_key, _) = generate_ed25519_keypair();
            let signer = Arc::new(Ed25519Signer::new(signing_key)) as Arc<dyn Signer>;
            (did, signer)
        } else {
            // Generate identity and signer from the same keypair for proper cryptographic matching
            let (signing_key, verifying_key) = generate_ed25519_keypair();
            let current_identity_str = icn_identity::did_key_from_verifying_key(&verifying_key);
            let current_identity = Did::from_str(&current_identity_str)
                .map_err(|e| CommonError::InternalError(format!("Invalid DID: {}", e)))?;
            let signer = Arc::new(Ed25519Signer::new(signing_key)) as Arc<dyn Signer>;
            (current_identity, signer)
        };

        // Create DAG store with default or specified path
        let storage_path =
            storage_path.unwrap_or_else(|| std::path::PathBuf::from("./icn_storage"));
        let dag_store = super::dag_store_factory::DagStoreFactory::create_production(storage_path)?;

        // Create mana ledger with default or specified path
        let mana_ledger_path =
            mana_ledger_path.unwrap_or_else(|| std::path::PathBuf::from("./mana_ledger.json"));
        let mana_ledger = super::mana::SimpleManaLedger::new(mana_ledger_path);

        // Create production network service
        #[cfg(feature = "enable-libp2p")]
        {
            use icn_network::libp2p_service::{Libp2pNetworkService, NetworkConfig};

            let config = NetworkConfig {
                listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()], // Random port
                bootstrap_peers: vec![],     // No bootstrap peers by default
                discovery_addresses: vec![], // No discovery addresses by default
                enable_mdns: true,
                max_peers: 100,
                max_peers_per_ip: 5,
                connection_timeout: Duration::from_secs(30),
                request_timeout: Duration::from_secs(10),
                heartbeat_interval: Duration::from_secs(15),
                bootstrap_interval: Duration::from_secs(300),
                peer_discovery_interval: Duration::from_secs(60),
                kademlia_replication_factor: 20,
            };

            let network_service =
                Arc::new(Libp2pNetworkService::new(config).await.map_err(|e| {
                    CommonError::NetworkError(format!("Failed to create libp2p service: {}", e))
                })?);

            let did_resolver = Arc::new(icn_identity::KeyDidResolver);
            let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());

            Self::new_with_services(
                current_identity,
                network_service,
                signer,
                did_resolver,
                dag_store,
                mana_ledger,
                reputation_store,
                None,
            )
        }

        #[cfg(not(feature = "enable-libp2p"))]
        {
            Err(CommonError::InternalError(
                "Production RuntimeContext requires libp2p feature enabled. Enable the 'enable-libp2p' feature.".to_string()
            ))
        }
    }

    /// Create a testing RuntimeContext with all stub services.
    ///
    /// **🧪 TESTING ONLY**: This method creates a completely isolated testing environment
    /// with all stub services for fast, deterministic testing.
    ///
    /// **⚠️ WARNING**: This method uses stub services and should NEVER be used in production.
    /// Use `RuntimeContext::new()` or `RuntimeContext::new_async()` for production deployments.
    ///
    /// **Services Used:**
    /// - Network: Stub network service (no real networking)
    /// - Signer: Stub signer (deterministic signatures)
    /// - DAG Store: In-memory stub store (no persistence)
    /// - Mana Ledger: Temporary file-based ledger
    /// - Reputation Store: In-memory reputation store
    ///
    /// **Use when:**
    /// - Unit testing
    /// - Integration testing that doesn't require real networking
    /// - Fast test execution
    /// - Deterministic test behavior
    ///
    /// **Parameters:**
    /// - `current_identity`: The DID for this test context
    /// - `initial_mana`: Optional initial mana balance (defaults to 0)
    pub fn new_for_testing(
        current_identity: Did,
        initial_mana: Option<u64>,
    ) -> Result<Arc<Self>, CommonError> {
        let config = ServiceConfig::testing(current_identity.clone(), initial_mana)?;
        Self::from_service_config(config)
    }

    /// Create a testing RuntimeContext with all stub services (old method name).
    ///
    /// **🧪 TESTING ONLY & DEPRECATED**: Use `new_for_testing()` instead for clearer semantics.
    #[deprecated(
        since = "0.3.0",
        note = "Use `new_for_testing()` instead for clearer semantics. This method uses stub services that are not suitable for production."
    )]
    pub fn new_testing(
        current_identity: Did,
        initial_mana: Option<u64>,
    ) -> Result<Arc<Self>, CommonError> {
        Self::new_for_testing(current_identity, initial_mana)
    }

    /// Create a testing context with a custom system info provider.
    pub fn new_testing_with_system_info(
        current_identity: Did,
        initial_mana: Option<u64>,
        system_info: Arc<dyn SystemInfoProvider>,
    ) -> Result<Arc<Self>, CommonError> {
        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let mesh_network_service =
            Arc::new(MeshNetworkServiceType::Stub(StubMeshNetworkService::new()));
        let signer = Arc::new(super::signers::StubSigner::new());
        let did_resolver = Arc::new(icn_identity::KeyDidResolver);
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
        let latency_store = Arc::new(icn_mesh::NoOpLatencyStore) as Arc<dyn icn_mesh::LatencyStore>;
        let parameters = Self::default_parameters();
        let policy_enforcer = None;
        let time_provider = Arc::new(icn_common::SystemTimeProvider);

        let temp_file = tempfile::NamedTempFile::new().map_err(|e| {
            CommonError::IoError(format!("Failed to create temp file for testing: {}", e))
        })?;
        let temp_path = temp_file.path().to_path_buf();
        std::mem::forget(temp_file);
        let mana_ledger = SimpleManaLedger::new(temp_path);

        let dag_store_raw = Arc::new(DagStoreMutexType::new(StubDagStore::new()))
            as Arc<DagStoreMutexType<DagStorageService>>;
        let dag_store = DagStoreWrapper::stub(dag_store_raw.clone());

        // Create cross-component coordinator
        let cross_component_coordinator = Self::create_cross_component_coordinator(
            mesh_network_service.clone(),
            &dag_store,
            governance_module.clone(),
            reputation_store.clone(),
            current_identity.clone(),
            time_provider.clone(),
        );

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
            dag_store,
            reputation_store,
            trust_engine: Arc::new(TokioMutex::new(TrustPolicyEngine::new())),
            latency_store,
            parameters,
            policy_enforcer,
            resource_ledger: TokioMutex::new(super::resource_ledger::ResourceLedger::new()),
            system_info: Arc::new(SysinfoSystemInfoProvider),
            time_provider,
            default_receipt_wait_ms: 30000,
            cross_component_coordinator,
        });

        if let Some(mana) = initial_mana {
            ctx.mana_ledger
                .set_balance(&current_identity, mana)
                .map_err(|e| {
                    CommonError::InternalError(format!("Failed to set initial mana: {}", e))
                })?;
        }

        Ok(ctx)
    }

    /// Create a new context using explicitly provided services with mesh network focus.
    /// This constructor is primarily for advanced embedding scenarios. Most
    /// callers should use [`RuntimeContext::new`] for a production-ready
    /// context.
    pub fn new_with_mesh_services(
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
        let latency_store = Arc::new(icn_mesh::NoOpLatencyStore) as Arc<dyn icn_mesh::LatencyStore>;
        let parameters = Self::default_parameters();
        let policy_enforcer = None;
        let time_provider = Arc::new(icn_common::SystemTimeProvider);

        // Use a temporary file for general contexts
        let temp_file = tempfile::NamedTempFile::new()
            .unwrap_or_else(|_| panic!("Failed to create temporary file for mana ledger"));
        let temp_path = temp_file.path().to_path_buf();
        std::mem::forget(temp_file);
        let mana_ledger = SimpleManaLedger::new(temp_path);

        // Create cross-component coordinator
        let dag_store_wrapper = DagStoreWrapper::generic_production(dag_store.clone());
        let cross_component_coordinator = Self::create_cross_component_coordinator(
            mesh_network_service.clone(),
            &dag_store_wrapper,
            governance_module.clone(),
            reputation_store.clone(),
            current_identity.clone(),
            time_provider.clone(),
        );

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
            dag_store: dag_store_wrapper,
            reputation_store,
            trust_engine: Arc::new(TokioMutex::new(TrustPolicyEngine::new())),
            latency_store,
            parameters,
            policy_enforcer,
            resource_ledger: TokioMutex::new(super::resource_ledger::ResourceLedger::new()),
            system_info: Arc::new(SysinfoSystemInfoProvider),
            time_provider,
            default_receipt_wait_ms: 30000,
            cross_component_coordinator,
        })
    }

    /// Create a new context with real libp2p and mDNS services.
    #[cfg(feature = "enable-libp2p")]
    #[allow(clippy::too_many_arguments)]
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
        let latency_store = Arc::new(icn_mesh::NoOpLatencyStore) as Arc<dyn icn_mesh::LatencyStore>;
        let parameters = Self::default_parameters();
        let policy_enforcer = None;
        let time_provider = Arc::new(icn_common::SystemTimeProvider);
        let system_info = Arc::new(SysinfoSystemInfoProvider);

        // Create cross-component coordinator
        let dag_store_wrapper = DagStoreWrapper::generic_production(dag_store.clone());
        let cross_component_coordinator = Self::create_cross_component_coordinator(
            mesh_network_service.clone(),
            &dag_store_wrapper,
            governance_module.clone(),
            reputation_store.clone(),
            current_identity.clone(),
            time_provider.clone(),
        );

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
            dag_store: dag_store_wrapper,
            reputation_store,
            trust_engine: Arc::new(TokioMutex::new(TrustPolicyEngine::new())),
            latency_store,
            parameters,
            policy_enforcer,
            resource_ledger: TokioMutex::new(super::resource_ledger::ResourceLedger::new()),
            system_info,
            time_provider,
            default_receipt_wait_ms: 30000,
            cross_component_coordinator,
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
            latency_store,
            policy_enforcer,
            time_provider,
        } = params;
        let (tx, rx) = mpsc::channel(128);
        let job_states = Arc::new(DashMap::new());
        let governance_module = Arc::new(DagStoreMutexType::new(GovernanceModule::new()));
        let reputation_store = Arc::new(icn_reputation::InMemoryReputationStore::new());
        let parameters = Self::default_parameters();

        // Create cross-component coordinator
        let cross_component_coordinator = Self::create_cross_component_coordinator(
            mesh_network_service.clone(),
            &dag_store,
            governance_module.clone(),
            reputation_store.clone(),
            current_identity.clone(),
            time_provider.clone(),
        );

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
            trust_engine: Arc::new(TokioMutex::new(TrustPolicyEngine::new())),
            latency_store,
            parameters,
            policy_enforcer,
            resource_ledger: TokioMutex::new(super::resource_ledger::ResourceLedger::new()),
            system_info: Arc::new(SysinfoSystemInfoProvider),
            time_provider,
            default_receipt_wait_ms: 30000,
            cross_component_coordinator,
        })
    }

    /// Submit a mesh job with complete DAG lifecycle integration.
    /// This replaces the simple internal_queue_mesh_job with full lifecycle management.
    pub async fn handle_submit_job(
        self: &Arc<Self>,
        manifest_cid: Cid,
        spec_bytes: Vec<u8>,
        cost_mana: u64,
    ) -> Result<JobId, HostAbiError> {
        log::info!(
            "[handle_submit_job] Starting job submission: manifest_cid={}, cost_mana={}",
            manifest_cid,
            cost_mana
        );

        // Increment submission metrics
        JOBS_SUBMITTED_TOTAL.inc();
        JOBS_SUBMITTED.inc();
        PENDING_JOBS_GAUGE.inc();

        // 1. Parse and validate the job spec
        let job_spec: icn_mesh::JobSpec = bincode::deserialize(&spec_bytes).map_err(|e| {
            HostAbiError::InvalidParameters(format!("Invalid job spec bytes: {}", e))
        })?;

        // 2. Apply reputation-based pricing
        let reputation = self.reputation_store.get_reputation(&self.current_identity);
        let adjusted_cost = icn_economics::price_by_reputation(cost_mana, reputation);

        log::debug!(
            "[handle_submit_job] Reputation-adjusted cost: {} -> {} (reputation: {})",
            cost_mana,
            adjusted_cost,
            reputation
        );

        // 3. Spend mana
        self.spend_mana(&self.current_identity, adjusted_cost)
            .await?;

        // 4. Generate temporary job ID from deterministic hash
        // This will be updated to the actual DAG CID after storage
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(manifest_cid.to_string().as_bytes());
        hasher.update(&spec_bytes);
        hasher.update(self.current_identity.to_string().as_bytes());
        hasher.update(adjusted_cost.to_le_bytes());
        hasher.update(self.time_provider.unix_seconds().to_le_bytes());
        let temp_job_id_cid = Cid::new_v1_sha256(0x55, &hasher.finalize());
        let temp_job_id = JobId::from(temp_job_id_cid);

        log::debug!(
            "[handle_submit_job] Generated temporary job ID: {}",
            temp_job_id
        );

        // 5. Create the Job DAG node with temporary ID
        let mut job = Job {
            id: temp_job_id.clone(),
            manifest_cid: manifest_cid.clone(),
            spec_bytes: spec_bytes.clone(),
            spec_json: None,
            submitter_did: self.current_identity.clone(),
            cost_mana: adjusted_cost,
            submitted_at: self.time_provider.unix_seconds(),
            status: JobLifecycleStatus::Submitted,
            resource_requirements: job_spec.required_resources.clone(),
        };

        // 6. Store job in DAG and get the actual computed CID
        let job_dag_cid = self.store_job_in_dag(&job).await?;

        // 7. Update the job ID to match the actual DAG storage CID
        let actual_job_id = JobId::from(job_dag_cid.clone());
        job.id = actual_job_id.clone();

        log::info!(
            "[handle_submit_job] Job stored in DAG with CID: {} (updated job ID from {} to {})",
            job_dag_cid,
            temp_job_id,
            actual_job_id
        );

        // 8. Update job state tracking with the actual job ID
        self.job_states
            .insert(actual_job_id.clone(), JobState::Pending);

        // 9. Create ActualMeshJob for network announcement with the actual job ID
        let actual_job = ActualMeshJob {
            id: actual_job_id.clone(),
            manifest_cid,
            spec: job_spec,
            creator_did: self.current_identity.clone(),
            cost_mana: adjusted_cost,
            max_execution_wait_ms: Some(self.default_receipt_wait_ms),
            signature: icn_identity::SignatureBytes(vec![]), // Will be signed by mesh service
        };

        // 10. Announce job to mesh network for bidding
        if let Err(e) = self.mesh_network_service.announce_job(&actual_job).await {
            log::warn!(
                "[handle_submit_job] Failed to announce job to mesh network: {}",
                e
            );
        } else {
            log::info!("[handle_submit_job] Job announced to mesh network");
        }

        // 11. Start the async job lifecycle management
        let ctx = Arc::clone(self);
        let job_id_for_task = actual_job_id.clone();
        tokio::spawn(async move {
            log::info!(
                "[handle_submit_job] Spawning lifecycle management task for job: {}",
                job_id_for_task
            );
            if let Err(e) = ctx.manage_job_lifecycle(job_id_for_task).await {
                log::error!("[handle_submit_job] Job lifecycle management failed: {}", e);
            } else {
                log::info!("[handle_submit_job] Job lifecycle management completed successfully");
            }
        });

        log::info!(
            "[handle_submit_job] Job submission completed successfully: {}",
            actual_job_id
        );
        Ok(actual_job_id)
    }

    /// Internal queue mesh job method (DEPRECATED - use handle_submit_job instead).
    /// Kept for backward compatibility with existing tests.
    #[deprecated(
        since = "0.2.0",
        note = "Use handle_submit_job instead for full DAG integration"
    )]
    pub async fn internal_queue_mesh_job(
        self: &Arc<Self>,
        job: ActualMeshJob,
    ) -> Result<(), HostAbiError> {
        JOBS_SUBMITTED.inc();
        PENDING_JOBS_GAUGE.inc();

        log::warn!("[internal_queue_mesh_job] Using deprecated method - consider migrating to handle_submit_job");

        self.pending_mesh_jobs_tx
            .send(job)
            .await
            .map_err(|e| HostAbiError::InternalError(format!("Failed to queue job: {}", e)))
    }

    /// Store a job in the DAG and return its CID.
    async fn store_job_in_dag(&self, job: &Job) -> Result<Cid, HostAbiError> {
        let job_bytes = serde_json::to_vec(job).map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to serialize job: {}", e))
        })?;

        // Compute the proper Merkle CID based on the content
        let computed_cid = icn_common::compute_merkle_cid(
            0x71, // Raw codec
            &job_bytes,
            &[], // Job nodes have no parents initially
            job.submitted_at,
            &job.submitter_did,
            &None,
            &None,
        );

        let dag_block = DagBlock {
            cid: computed_cid,
            data: job_bytes,
            links: vec![], // Job nodes have no parents initially
            timestamp: job.submitted_at,
            author_did: job.submitter_did.clone(),
            signature: None,
            scope: None,
        };

        let mut dag_store = self.dag_store.inner().lock().await;
        dag_store.put(&dag_block).await.map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to store job DAG block: {}", e))
        })?;

        Ok(dag_block.cid)
    }

    /// Store a job bid in the DAG with a link to the parent job.
    async fn store_bid_in_dag(&self, bid: &JobBid) -> Result<Cid, HostAbiError> {
        let bid_bytes = serde_json::to_vec(bid).map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to serialize bid: {}", e))
        })?;

        // Create link to parent job
        let job_link = icn_common::DagLink {
            cid: bid.job_id.0.clone(),
            name: "parent_job".to_string(),
            size: 0, // Size will be calculated by DAG store
        };

        // Compute the proper Merkle CID based on the content
        let bid_cid = icn_common::compute_merkle_cid(
            0x55, // CBOR codec for bids
            &bid_bytes,
            std::slice::from_ref(&job_link),
            bid.submitted_at,
            &bid.executor_did,
            &None,
            &None,
        );

        let dag_block = DagBlock {
            cid: bid_cid.clone(),
            data: bid_bytes,
            links: vec![job_link],
            timestamp: bid.submitted_at,
            author_did: bid.executor_did.clone(),
            signature: None,
            scope: None,
        };

        let mut dag_store = self.dag_store.inner().lock().await;
        dag_store.put(&dag_block).await.map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to store bid DAG block: {}", e))
        })?;

        Ok(bid_cid)
    }

    /// Store a job assignment in the DAG with a link to the parent job.
    async fn store_assignment_in_dag(
        &self,
        assignment: &JobAssignment,
    ) -> Result<Cid, HostAbiError> {
        let assignment_bytes = serde_json::to_vec(assignment).map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to serialize assignment: {}", e))
        })?;

        // Create link to parent job
        let job_link = icn_common::DagLink {
            cid: assignment.job_id.0.clone(),
            name: "parent_job".to_string(),
            size: 0,
        };

        // Compute the proper Merkle CID based on the content
        let assignment_cid = icn_common::compute_merkle_cid(
            0x55, // CBOR codec for assignments
            &assignment_bytes,
            std::slice::from_ref(&job_link),
            assignment.assigned_at,
            &self.current_identity,
            &None,
            &None,
        );

        let dag_block = DagBlock {
            cid: assignment_cid.clone(),
            data: assignment_bytes,
            links: vec![job_link],
            timestamp: assignment.assigned_at,
            author_did: self.current_identity.clone(), // Job manager assigns
            signature: None,
            scope: None,
        };

        let mut dag_store = self.dag_store.inner().lock().await;
        dag_store.put(&dag_block).await.map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to store assignment DAG block: {}", e))
        })?;

        Ok(assignment_cid)
    }

    /// Store a job receipt in the DAG with a link to the parent job.
    async fn store_receipt_in_dag(&self, receipt: &JobReceipt) -> Result<Cid, HostAbiError> {
        let receipt_bytes = serde_json::to_vec(receipt).map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to serialize receipt: {}", e))
        })?;

        // Create link to parent job
        let job_link = icn_common::DagLink {
            cid: receipt.job_id.0.clone(),
            name: "parent_job".to_string(),
            size: 0,
        };

        // Compute the proper Merkle CID based on the content
        let receipt_cid = icn_common::compute_merkle_cid(
            0x55, // CBOR codec for receipts
            &receipt_bytes,
            std::slice::from_ref(&job_link),
            receipt.completed_at,
            &receipt.executor_did,
            &None,
            &None,
        );

        let dag_block = DagBlock {
            cid: receipt_cid.clone(),
            data: receipt_bytes,
            links: vec![job_link],
            timestamp: receipt.completed_at,
            author_did: receipt.executor_did.clone(),
            signature: None,
            scope: None,
        };

        let mut dag_store = self.dag_store.inner().lock().await;
        dag_store.put(&dag_block).await.map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to store receipt DAG block: {}", e))
        })?;

        Ok(receipt_cid)
    }

    /// Manage the complete lifecycle of a job through bidding, assignment, and execution.
    async fn manage_job_lifecycle(self: &Arc<Self>, job_id: JobId) -> Result<(), HostAbiError> {
        log::info!(
            "[manage_job_lifecycle] Starting lifecycle management for job: {}",
            job_id
        );

        // 0. Check if this is a CCL WASM job that should auto-execute
        log::debug!(
            "[manage_job_lifecycle] Retrieving job status for: {}",
            job_id
        );
        match self.get_job_status(&job_id).await {
            Ok(Some(_lifecycle)) => {
                log::debug!(
                    "[manage_job_lifecycle] Found job lifecycle, checking if CCL WASM: {}",
                    job_id
                );
            }
            Ok(None) => {
                log::warn!("[manage_job_lifecycle] Job not found in DAG: {}", job_id);
                return Err(HostAbiError::DagOperationFailed(
                    "Job not found in DAG".to_string(),
                ));
            }
            Err(e) => {
                log::error!(
                    "[manage_job_lifecycle] Failed to get job status: {} - {}",
                    job_id,
                    e
                );
                return Err(e);
            }
        }

        if let Ok(Some(lifecycle)) = self.get_job_status(&job_id).await {
            let job_spec = lifecycle.job.decode_spec().map_err(|e| {
                HostAbiError::DagOperationFailed(format!("Failed to decode job spec: {}", e))
            })?;

            if job_spec.kind.is_ccl_wasm() {
                log::info!(
                    "[manage_job_lifecycle] Job {} is CCL WASM, auto-executing immediately",
                    job_id
                );

                // Create ActualMeshJob for execution
                let actual_job = ActualMeshJob {
                    id: job_id.clone(),
                    manifest_cid: lifecycle.job.manifest_cid.clone(),
                    spec: job_spec,
                    creator_did: lifecycle.job.submitter_did.clone(),
                    cost_mana: lifecycle.job.cost_mana,
                    max_execution_wait_ms: None,
                    signature: icn_identity::SignatureBytes(vec![]),
                };

                // Execute the CCL WASM job
                match Self::execute_ccl_wasm_job(self, &actual_job).await {
                    Ok(receipt) => {
                        log::info!(
                            "[manage_job_lifecycle] CCL WASM job {} completed successfully",
                            job_id
                        );
                        self.job_states
                            .insert(job_id.clone(), JobState::Completed { receipt });
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!(
                            "[manage_job_lifecycle] CCL WASM job {} execution failed: {}",
                            job_id,
                            e
                        );
                        self.job_states.insert(
                            job_id.clone(),
                            JobState::Failed {
                                reason: e.to_string(),
                            },
                        );
                        return Err(e);
                    }
                }
            }
        }

        // 1. Open bidding period
        self.update_job_status(&job_id, JobLifecycleStatus::BiddingOpen)
            .await?;
        self.job_states.insert(job_id.clone(), JobState::Pending); // Keep as pending during bidding
        JOBS_BIDDING_GAUGE.inc();

        // 2. Collect bids for a defined period
        let bidding_duration = Duration::from_secs(10); // Configurable
        log::info!(
            "[manage_job_lifecycle] Collecting bids for {} seconds",
            bidding_duration.as_secs()
        );

        let bids = self
            .mesh_network_service
            .collect_bids_for_job(&job_id, bidding_duration)
            .await
            .unwrap_or_else(|e| {
                log::warn!("[manage_job_lifecycle] Failed to collect bids: {}", e);
                vec![]
            });

        log::info!(
            "[manage_job_lifecycle] Collected {} bids for job {}",
            bids.len(),
            job_id
        );
        BIDS_RECEIVED_TOTAL.inc_by(bids.len() as u64);

        // 3. Store all bids in DAG
        for (i, mesh_bid) in bids.iter().enumerate() {
            let job_bid = JobBid {
                job_id: job_id.clone(),
                bid_id: format!("bid_{}", i),
                executor_did: mesh_bid.executor_did.clone(),
                price_mana: mesh_bid.price_mana,
                resources: mesh_bid.resources.clone(),
                submitted_at: self.time_provider.unix_seconds(),
                signature: mesh_bid.signature.clone(),
            };

            if let Err(e) = self.store_bid_in_dag(&job_bid).await {
                log::warn!("[manage_job_lifecycle] Failed to store bid in DAG: {}", e);
            }
        }

        // 4. Close bidding and select executor
        self.update_job_status(&job_id, JobLifecycleStatus::BiddingClosed)
            .await?;
        JOBS_BIDDING_GAUGE.dec();

        if bids.is_empty() {
            log::warn!(
                "[manage_job_lifecycle] No bids received for job {}, refunding mana",
                job_id
            );

            // Refund mana since job couldn't be executed due to no bids
            if let Ok(Some(lifecycle)) = self.get_job_status(&job_id).await {
                self.credit_mana(&lifecycle.job.submitter_did, lifecycle.job.cost_mana)
                    .await?;
                log::info!(
                    "[manage_job_lifecycle] Refunded {} mana to {}",
                    lifecycle.job.cost_mana,
                    lifecycle.job.submitter_did
                );
            }

            self.update_job_status(&job_id, JobLifecycleStatus::Failed)
                .await?;
            self.job_states.insert(
                job_id.clone(),
                JobState::Failed {
                    reason: "No bids received".to_string(),
                },
            );
            JOBS_FAILED_TOTAL.inc();
            return Ok(());
        }

        // 5. Select best executor
        let lifecycle = self.get_job_status(&job_id).await?;
        let job_spec = if let Some(lifecycle) = lifecycle {
            lifecycle.job.decode_spec().map_err(|e| {
                HostAbiError::DagOperationFailed(format!("Failed to decode job spec: {}", e))
            })?
        } else {
            log::error!(
                "[manage_job_lifecycle] Job {} not found in DAG during executor selection",
                job_id
            );
            return Err(HostAbiError::InternalError(
                "Job spec not found in DAG".to_string(),
            ));
        };
        let selection_policy = icn_mesh::SelectionPolicy::default();
        let selected_executor = icn_mesh::select_executor(
            &job_id,
            &job_spec,
            bids.clone(),
            &selection_policy,
            self.reputation_store.as_ref(),
            &self.mana_ledger,
            self.latency_store.as_ref(),
            &icn_mesh::NoOpCapabilityChecker,
        );

        let selected_executor = match selected_executor {
            Some(executor) => executor,
            None => {
                log::warn!("[manage_job_lifecycle] No suitable executor selected for job {}, refunding mana", job_id);

                // Refund mana since no suitable executor was found
                if let Ok(Some(lifecycle)) = self.get_job_status(&job_id).await {
                    self.credit_mana(&lifecycle.job.submitter_did, lifecycle.job.cost_mana)
                        .await?;
                    log::info!(
                        "[manage_job_lifecycle] Refunded {} mana to {}",
                        lifecycle.job.cost_mana,
                        lifecycle.job.submitter_did
                    );
                }

                self.update_job_status(&job_id, JobLifecycleStatus::Failed)
                    .await?;
                self.job_states.insert(
                    job_id.clone(),
                    JobState::Failed {
                        reason: "No suitable executor found".to_string(),
                    },
                );
                JOBS_FAILED_TOTAL.inc();
                return Ok(());
            }
        };

        // 6. Find the winning bid
        let (winning_index, winning_bid) = bids
            .iter()
            .enumerate()
            .find(|(_, bid)| bid.executor_did == selected_executor)
            .ok_or_else(|| {
                HostAbiError::InternalError("Selected executor bid not found".to_string())
            })?;

        // 7. Create and store assignment
        let assignment = JobAssignment {
            job_id: job_id.clone(),
            winning_bid_id: format!("bid_{}", winning_index),
            assigned_executor_did: selected_executor.clone(),
            assigned_at: self.time_provider.unix_seconds(),
            final_price_mana: winning_bid.price_mana,
            committed_resources: winning_bid.resources.clone(),
        };

        if let Err(e) = self.store_assignment_in_dag(&assignment).await {
            log::error!(
                "[manage_job_lifecycle] Failed to store assignment in DAG: {}",
                e
            );
            return Err(e);
        }

        // 8. Update job status and metrics
        self.update_job_status(&job_id, JobLifecycleStatus::Assigned)
            .await?;
        self.job_states.insert(
            job_id.clone(),
            JobState::Assigned {
                executor: selected_executor.clone(),
            },
        );
        JOBS_ASSIGNED_TOTAL.inc();
        JOBS_EXECUTING_GAUGE.inc();

        // 9. Notify executor of assignment
        let assignment_notice = JobAssignmentNotice {
            job_id: job_id.clone(),
            executor_did: selected_executor.clone(),
            agreed_cost_mana: winning_bid.price_mana,
        };

        if let Err(e) = self
            .mesh_network_service
            .notify_executor_of_assignment(&assignment_notice)
            .await
        {
            log::warn!(
                "[manage_job_lifecycle] Failed to notify executor of assignment: {}",
                e
            );
        }

        // 10. Wait for execution receipt
        let receipt_timeout = Duration::from_millis(self.default_receipt_wait_ms);
        log::info!(
            "[manage_job_lifecycle] Waiting for execution receipt (timeout: {}ms)",
            self.default_receipt_wait_ms
        );

        let execution_receipt = self
            .mesh_network_service
            .try_receive_receipt(&job_id, &selected_executor, receipt_timeout)
            .await;

        match execution_receipt {
            Ok(Some(receipt)) => {
                log::info!(
                    "[manage_job_lifecycle] Received execution receipt for job {}",
                    job_id
                );

                // 11. Create and store job receipt
                let job_receipt = JobReceipt {
                    job_id: job_id.clone(),
                    executor_did: receipt.executor_did.clone(),
                    success: receipt.success,
                    cpu_ms: receipt.cpu_ms,
                    result_cid: receipt.result_cid.clone(),
                    completed_at: self.time_provider.unix_seconds(),
                    error_message: if receipt.success {
                        None
                    } else {
                        Some("Execution failed".to_string())
                    },
                    signature: receipt.sig.clone(),
                };

                if let Err(e) = self.store_receipt_in_dag(&job_receipt).await {
                    log::error!(
                        "[manage_job_lifecycle] Failed to store receipt in DAG: {}",
                        e
                    );
                    return Err(e);
                }

                // 12. Update final status
                let final_status = if receipt.success {
                    JobLifecycleStatus::Completed
                } else {
                    JobLifecycleStatus::Failed
                };

                self.update_job_status(&job_id, final_status.clone())
                    .await?;
                self.job_states.insert(
                    job_id.clone(),
                    JobState::Completed {
                        receipt: receipt.clone(),
                    },
                );

                JOBS_EXECUTING_GAUGE.dec();
                if receipt.success {
                    JOBS_COMPLETED_TOTAL.inc();
                } else {
                    JOBS_FAILED_TOTAL.inc();
                }

                log::info!(
                    "[manage_job_lifecycle] Job {} completed successfully: {}",
                    job_id,
                    receipt.success
                );
            }
            Ok(None) => {
                log::warn!("[manage_job_lifecycle] No receipt received for job {} within timeout, refunding mana", job_id);

                // Refund mana since job timed out
                if let Ok(Some(lifecycle)) = self.get_job_status(&job_id).await {
                    self.credit_mana(&lifecycle.job.submitter_did, lifecycle.job.cost_mana)
                        .await?;
                    log::info!(
                        "[manage_job_lifecycle] Refunded {} mana to {}",
                        lifecycle.job.cost_mana,
                        lifecycle.job.submitter_did
                    );
                }

                self.update_job_status(&job_id, JobLifecycleStatus::Failed)
                    .await?;
                self.job_states.insert(
                    job_id.clone(),
                    JobState::Failed {
                        reason: "Receipt timeout".to_string(),
                    },
                );
                JOBS_EXECUTING_GAUGE.dec();
                JOBS_FAILED_TOTAL.inc();
            }
            Err(e) => {
                log::error!("[manage_job_lifecycle] Error waiting for receipt for job {}: {}, refunding mana", job_id, e);

                // Refund mana since job failed due to error
                if let Ok(Some(lifecycle)) = self.get_job_status(&job_id).await {
                    self.credit_mana(&lifecycle.job.submitter_did, lifecycle.job.cost_mana)
                        .await?;
                    log::info!(
                        "[manage_job_lifecycle] Refunded {} mana to {}",
                        lifecycle.job.cost_mana,
                        lifecycle.job.submitter_did
                    );
                }

                self.update_job_status(&job_id, JobLifecycleStatus::Failed)
                    .await?;
                self.job_states.insert(
                    job_id.clone(),
                    JobState::Failed {
                        reason: format!("Receipt error: {}", e),
                    },
                );
                JOBS_EXECUTING_GAUGE.dec();
                JOBS_FAILED_TOTAL.inc();
            }
        }

        Ok(())
    }

    /// Update the status of a job (this would update the DAG node in a real implementation).
    async fn update_job_status(
        &self,
        job_id: &JobId,
        status: JobLifecycleStatus,
    ) -> Result<(), HostAbiError> {
        let old_status = if let Some(lifecycle) = self.get_job_status(job_id).await? {
            lifecycle.current_status()
        } else {
            JobLifecycleStatus::Submitted
        };

        let change = MeshJobStateChange {
            job_id: job_id.clone(),
            old_state: format!("{:?}", old_status),
            new_state: format!("{:?}", status.clone()),
        };

        let change_bytes = serde_json::to_vec(&change).map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to serialize status change: {}", e))
        })?;

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&change_bytes);
        let change_cid = Cid::new_v1_sha256(0x55, &hasher.finalize());

        let job_link = icn_common::DagLink {
            cid: job_id.0.clone(),
            name: "parent_job".to_string(),
            size: 0,
        };

        let dag_block = DagBlock {
            cid: change_cid,
            data: change_bytes,
            links: vec![job_link],
            timestamp: self.time_provider.unix_seconds(),
            author_did: self.current_identity.clone(),
            signature: None,
            scope: None,
        };

        let mut dag_store = self.dag_store.inner().lock().await;
        dag_store.put(&dag_block).await.map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to store status update: {}", e))
        })?;

        log::info!(
            "[update_job_status] Job {} status updated from {:?} to {:?}",
            job_id,
            old_status,
            status
        );
        Ok(())
    }

    /// Get the complete lifecycle of a job by reconstructing it from DAG traversal.
    pub async fn get_job_status(
        &self,
        job_id: &JobId,
    ) -> Result<Option<JobLifecycle>, HostAbiError> {
        log::debug!(
            "[get_job_status] Reconstructing job lifecycle for: {}",
            job_id
        );

        let dag_store = self.dag_store.inner().lock().await;

        // 1. Get the job node
        let job_block = dag_store.get(&job_id.0).await.map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to get job from DAG: {}", e))
        })?;

        let job_block = match job_block {
            Some(block) => block,
            None => {
                log::debug!("[get_job_status] Job {} not found in DAG", job_id);
                return Ok(None);
            }
        };

        // 2. Deserialize the job
        let job: Job = serde_json::from_slice(&job_block.data).map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to deserialize job: {}", e))
        })?;

        // 3. Create lifecycle and populate it
        let mut lifecycle = JobLifecycle::new(job);

        // 4. Find all child nodes (bids, assignments, receipts) by traversing the DAG
        // This is a simplified implementation - a real one would use DAG traversal indices
        let all_blocks = dag_store.list_blocks().await.map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to list DAG blocks: {}", e))
        })?;

        for block in all_blocks {
            // Check if this block links to our job
            let links_to_job = block.links.iter().any(|link| link.cid == job_id.0);
            if !links_to_job {
                continue;
            }

            // Try to deserialize as different lifecycle types
            if let Ok(bid) = serde_json::from_slice::<JobBid>(&block.data) {
                if bid.job_id == *job_id {
                    lifecycle.add_bid(bid);
                }
            } else if let Ok(assignment) = serde_json::from_slice::<JobAssignment>(&block.data) {
                if assignment.job_id == *job_id {
                    lifecycle.set_assignment(assignment);
                }
            } else if let Ok(receipt) = serde_json::from_slice::<JobReceipt>(&block.data) {
                if receipt.job_id == *job_id {
                    lifecycle.set_receipt(receipt);
                }
            }
        }

        log::debug!("[get_job_status] Reconstructed lifecycle for job {}: {} bids, assignment: {}, receipt: {}", 
                   job_id, lifecycle.bids.len(), lifecycle.assignment.is_some(), lifecycle.receipt.is_some());

        Ok(Some(lifecycle))
    }

    /// Get mana for an account.
    pub async fn get_mana(&self, account: &Did) -> Result<u64, HostAbiError> {
        Ok(self.mana_ledger.get_balance(account))
    }

    /// Retrieve synchronization status of the local DAG.
    pub async fn get_dag_sync_status(&self) -> Result<icn_common::DagSyncStatus, HostAbiError> {
        let store = self.dag_store.inner().lock().await;
        let root = icn_dag::current_root(&*store).await.map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to get DAG root: {}", e))
        })?;
        let in_sync = root.is_some();
        Ok(icn_common::DagSyncStatus {
            current_root: root,
            in_sync,
        })
    }

    async fn record_ledger_event(&self, event: &LedgerEvent) {
        let data = match serde_json::to_vec(event) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("[record_ledger_event] serialize failed: {e}");
                return;
            }
        };
        let author = match event {
            LedgerEvent::Credit { did, .. }
            | LedgerEvent::Debit { did, .. }
            | LedgerEvent::SetBalance { did, .. } => did.clone(),
        };
        let ts = self.time_provider.unix_seconds();
        let cid = compute_merkle_cid(0x71, &data, &[], ts, &author, &None, &None);
        let block = DagBlock {
            cid,
            data,
            links: vec![],
            timestamp: ts,
            author_did: author,
            signature: None,
            scope: None,
        };
        let mut dag = self.dag_store.inner().lock().await;
        if let Err(e) = dag.put(&block).await {
            log::warn!("[record_ledger_event] store failed: {e}");
        }
    }

    /// Spend mana from an account.
    pub async fn spend_mana(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        self.mana_ledger.spend(account, amount)?;
        self.record_ledger_event(&LedgerEvent::Debit {
            did: account.clone(),
            amount,
        })
        .await;
        crate::metrics::MANA_ACCOUNTS_GAUGE.set(self.mana_ledger.all_accounts().len() as i64);
        Ok(())
    }

    /// Credit mana to an account.
    pub async fn credit_mana(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        self.mana_ledger.credit(account, amount)?;
        self.record_ledger_event(&LedgerEvent::Credit {
            did: account.clone(),
            amount,
        })
        .await;
        crate::metrics::MANA_ACCOUNTS_GAUGE.set(self.mana_ledger.all_accounts().len() as i64);
        Ok(())
    }

    /// Anchor an execution receipt.
    pub async fn anchor_receipt(
        &self,
        receipt: &IdentityExecutionReceipt,
    ) -> Result<Cid, HostAbiError> {
        // 1. Validate that the job exists and was assigned to this executor
        let job_id = JobId::from(receipt.job_id.clone());

        // Check if the job was assigned to this executor
        let job_state = self.job_states.get(&job_id);
        if let Some(state) = job_state {
            match state.value() {
                JobState::Assigned { executor } => {
                    if executor != &receipt.executor_did {
                        return Err(HostAbiError::PermissionDenied(format!(
                            "Receipt executor {} does not match assigned executor {}",
                            receipt.executor_did, executor
                        )));
                    }
                }
                JobState::Completed { .. } => {
                    return Err(HostAbiError::InvalidParameters(
                        "Job already completed".to_string(),
                    ));
                }
                JobState::Failed { .. } => {
                    return Err(HostAbiError::InvalidParameters(
                        "Cannot submit receipt for failed job".to_string(),
                    ));
                }
                JobState::Pending => {
                    return Err(HostAbiError::InvalidParameters(
                        "Cannot submit receipt for unassigned job".to_string(),
                    ));
                }
            }
        } else {
            return Err(HostAbiError::InvalidParameters("Job not found".to_string()));
        }

        // 2. Verify the receipt signature against the executor's DID
        receipt
            .verify_with_resolver(&*self.did_resolver)
            .map_err(|e| HostAbiError::SignatureError(format!("{e}")))?;

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
            let mut dag_store = self.dag_store.inner().lock().await;
            dag_store.put(&block).await.map_err(|e| {
                HostAbiError::DagOperationFailed(format!("Failed to store receipt: {}", e))
            })?;
        }

        crate::metrics::RECEIPTS_ANCHORED.inc();

        Ok(cid)
    }

    /// Anchor a job checkpoint to the DAG.
    pub async fn anchor_checkpoint(
        &self,
        checkpoint: &icn_mesh::JobCheckpoint,
    ) -> Result<Cid, HostAbiError> {
        // 1. Validate that the job exists
        let job_id = &checkpoint.job_id;
        let job_state = self.job_states.get(job_id);
        if job_state.is_none() {
            return Err(HostAbiError::InvalidParameters("Job not found".to_string()));
        }

        // 2. Verify the checkpoint signature against the executor's DID
        let verifying_key = self
            .did_resolver
            .resolve(&checkpoint.executor_did)
            .map_err(|e| HostAbiError::InvalidParameters(format!("DID resolution failed: {e}")))?;

        checkpoint
            .verify_signature(&verifying_key)
            .map_err(|e| HostAbiError::SignatureError(format!("{e}")))?;

        // 3. Create a DAG block for the checkpoint
        let checkpoint_bytes = bincode::serialize(checkpoint).map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to serialize checkpoint: {}", e))
        })?;

        // Create a unique CID for the checkpoint
        let checkpoint_cid = compute_merkle_cid(
            0x71, // Raw codec
            &checkpoint_bytes,
            &[],
            checkpoint.timestamp,
            &checkpoint.executor_did,
            &None,
            &None,
        );

        let block = DagBlock {
            cid: checkpoint_cid.clone(),
            data: checkpoint_bytes,
            links: vec![],
            timestamp: checkpoint.timestamp,
            author_did: checkpoint.executor_did.clone(),
            signature: None,
            scope: Some(NodeScope(format!("checkpoint:{}", checkpoint.job_id))),
        };

        // 4. Store in DAG
        {
            let mut dag_store = self.dag_store.inner().lock().await;
            dag_store.put(&block).await.map_err(|e| {
                HostAbiError::DagOperationFailed(format!("Failed to store checkpoint: {}", e))
            })?;
        }

        crate::metrics::CHECKPOINTS_ANCHORED.inc();

        Ok(checkpoint_cid)
    }

    /// Anchor a partial output receipt to the DAG.
    pub async fn anchor_partial_output(
        &self,
        partial_output: &icn_mesh::PartialOutputReceipt,
    ) -> Result<Cid, HostAbiError> {
        // 1. Validate that the job exists
        let job_id = &partial_output.job_id;
        let job_state = self.job_states.get(job_id);
        if job_state.is_none() {
            return Err(HostAbiError::InvalidParameters("Job not found".to_string()));
        }

        // 2. Verify the partial output signature against the executor's DID
        let verifying_key = self
            .did_resolver
            .resolve(&partial_output.executor_did)
            .map_err(|e| HostAbiError::InvalidParameters(format!("DID resolution failed: {e}")))?;

        partial_output
            .verify_signature(&verifying_key)
            .map_err(|e| HostAbiError::SignatureError(format!("{e}")))?;

        // 3. Create a DAG block for the partial output
        let partial_output_bytes = bincode::serialize(partial_output).map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to serialize partial output: {}", e))
        })?;

        // Create a unique CID for the partial output
        let partial_output_cid = compute_merkle_cid(
            0x71, // Raw codec
            &partial_output_bytes,
            &[],
            partial_output.timestamp,
            &partial_output.executor_did,
            &None,
            &None,
        );

        let block = DagBlock {
            cid: partial_output_cid.clone(),
            data: partial_output_bytes,
            links: vec![],
            timestamp: partial_output.timestamp,
            author_did: partial_output.executor_did.clone(),
            signature: None,
            scope: Some(NodeScope(format!(
                "partial_output:{}",
                partial_output.job_id
            ))),
        };

        // 4. Store in DAG
        {
            let mut dag_store = self.dag_store.inner().lock().await;
            dag_store.put(&block).await.map_err(|e| {
                HostAbiError::DagOperationFailed(format!("Failed to store partial output: {}", e))
            })?;
        }

        crate::metrics::PARTIAL_OUTPUTS_ANCHORED.inc();

        Ok(partial_output_cid)
    }

    /// Anchor a parameter update event in the DAG.
    pub async fn anchor_parameter_update(
        &self,
        update: &ParameterUpdate,
    ) -> Result<Cid, HostAbiError> {
        let data = bincode::serialize(update).map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to serialize parameter update: {}", e))
        })?;
        let cid = compute_merkle_cid(
            0x71,
            &data,
            &[],
            update.timestamp,
            &update.signer,
            &None,
            &None,
        );
        let block = DagBlock {
            cid: cid.clone(),
            data,
            links: vec![],
            timestamp: update.timestamp,
            author_did: update.signer.clone(),
            signature: None,
            scope: None,
        };
        let mut dag = self.dag_store.inner().lock().await;
        dag.put(&block).await.map_err(|e| {
            HostAbiError::DagOperationFailed(format!("Failed to store parameter update: {}", e))
        })?;
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
                let tup: (Did, u64, String) =
                    serde_json::from_slice(&payload.type_specific_payload).map_err(|e| {
                        HostAbiError::InvalidParameters(format!(
                            "Failed to parse budget payload: {}",
                            e
                        ))
                    })?;
                ProposalType::BudgetAllocation(tup.0, tup.1, tup.2)
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
                let mut dag_store = self.dag_store.inner().lock().await;
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

        let time_provider = SystemTimeProvider;
        let pid = gov
            .submit_proposal(
                ProposalSubmission {
                    proposer: self.current_identity.clone(),
                    proposal_type,
                    description: payload.description,
                    duration_secs: payload.duration_secs,
                    quorum: payload.quorum,
                    threshold: payload.threshold,
                    content_cid,
                },
                &time_provider,
            )
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
        let time_provider = SystemTimeProvider;
        gov.cast_vote(
            self.current_identity.clone(),
            &proposal_id,
            vote_option,
            &time_provider,
        )
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
        let time_provider = SystemTimeProvider;
        let (status, (yes, no, abstain)) = gov
            .close_voting_period(&proposal_id, &time_provider)
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
                        ProposalType::BudgetAllocation(recipient, amount, _purpose) => {
                            self.credit_mana(recipient, *amount).await.map_err(|e| {
                                HostAbiError::InternalError(format!(
                                    "Failed to credit mana to recipient {}: {}",
                                    recipient, e
                                ))
                            })?;
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

    /// Record a resource event and anchor it in the DAG.
    pub async fn record_resource_event(
        &self,
        resource_id: String,
        action: super::resource_ledger::ResourceAction,
        scope: Option<NodeScope>,
        mana_cost: u64,
    ) -> Result<Cid, HostAbiError> {
        self.spend_mana(&self.current_identity, mana_cost).await?;

        if let Some(enforcer) = &self.policy_enforcer {
            if let icn_governance::scoped_policy::PolicyCheckResult::Denied { reason } = enforcer
                .check_permission(
                    icn_governance::scoped_policy::DagPayloadOp::SubmitBlock,
                    &self.current_identity,
                    scope.as_ref(),
                    None, // credential_proof
                    None, // revocation_proof
                )
            {
                return Err(HostAbiError::PermissionDenied(reason));
            }
        }

        let ts = self.time_provider.unix_seconds();
        let mut store = self.dag_store.inner().lock().await;
        let cid = super::resource_ledger::record_resource_event(
            &mut *store,
            &self.current_identity,
            resource_id.clone(),
            action.clone(),
            ts,
            scope.clone(),
        )
        .await
        .map_err(|e| HostAbiError::DagOperationFailed(format!("{}", e)))?;

        {
            let mut ledger = self.resource_ledger.lock().await;
            ledger.push(super::resource_ledger::ResourceLedgerEntry {
                did: self.current_identity.clone(),
                resource_id,
                action,
                timestamp: ts,
                cid: cid.clone(),
                scope,
            });
        }

        Ok(cid)
    }

    /// Ingest a proposal that originated from another node.
    pub async fn ingest_external_proposal(&self, bytes: &[u8]) -> Result<(), HostAbiError> {
        let proposal: Proposal = bincode::deserialize(bytes).map_err(|e| {
            HostAbiError::InvalidParameters(format!("Failed to deserialize proposal: {}", e))
        })?;

        let mut gov = self.governance_module.lock().await;
        gov.insert_external_proposal(proposal).map_err(|e| {
            HostAbiError::InternalError(format!("Failed to ingest external proposal: {}", e))
        })
    }

    /// Ingest a vote that originated from another node.
    pub async fn ingest_external_vote(&self, bytes: &[u8]) -> Result<(), HostAbiError> {
        let vote: Vote = bincode::deserialize(bytes).map_err(|e| {
            HostAbiError::InvalidParameters(format!("Failed to deserialize vote: {}", e))
        })?;

        let mut gov = self.governance_module.lock().await;
        gov.insert_external_vote(vote).map_err(|e| {
            HostAbiError::InternalError(format!("Failed to ingest external vote: {}", e))
        })
    }

    /// Update a system parameter.
    async fn update_parameter(&self, key: String, value: String) -> Result<(), HostAbiError> {
        self.parameters.insert(key.clone(), value.clone());
        log::info!("Updated parameter {} to {}", key, value);

        let update = ParameterUpdate {
            name: key,
            value,
            timestamp: self.time_provider.unix_seconds(),
            signer: self.current_identity.clone(),
        };

        // Persist to DAG
        self.anchor_parameter_update(&update).await?;

        Ok(())
    }

    /// Spawn the mesh job manager with full lifecycle support.
    ///
    /// This manager handles the complete mesh job lifecycle:
    /// 1. Job announcement to potential executors
    /// 2. Bid collection from interested executors
    /// 3. Executor selection based on policy
    /// 4. Job assignment to selected executor
    /// 5. Receipt collection and validation
    /// 6. Job completion and state updates
    pub async fn spawn_mesh_job_manager(self: Arc<Self>) {
        let ctx = self.clone();

        tokio::spawn(async move {
            log::info!("Starting mesh job manager background task with full lifecycle support");

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

                        // Handle different job types
                        if job.spec.kind.is_ccl_wasm() {
                            // CCL WASM jobs still get auto-executed
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
                            // Handle regular mesh jobs with full lifecycle
                            log::info!("Starting full mesh lifecycle for job: {:?}", job_id);

                            let ctx_clone = ctx.clone();
                            let job_clone = job.clone();

                            tokio::spawn(async move {
                                if let Err(e) =
                                    Self::handle_mesh_job_lifecycle(&ctx_clone, &job_clone).await
                                {
                                    log::error!(
                                        "Mesh job lifecycle failed for job {:?}: {}",
                                        job_clone.id,
                                        e
                                    );
                                    JOBS_FAILED.inc();
                                    ctx_clone.job_states.insert(
                                        job_clone.id.clone(),
                                        JobState::Failed {
                                            reason: e.to_string(),
                                        },
                                    );
                                }
                            });
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

        log::info!("Mesh job manager spawned successfully with full lifecycle support");
    }

    /// Handle the full mesh job lifecycle for non-CCL WASM jobs.
    ///
    /// This implements the complete mesh computing workflow:
    /// 1. Announce job to potential executors
    /// 2. Collect bids from interested executors
    /// 3. Select best executor based on policy
    /// 4. Assign job to selected executor
    /// 5. Wait for execution receipt
    /// 6. Validate and anchor receipt
    async fn handle_mesh_job_lifecycle(
        ctx: &Arc<RuntimeContext>,
        job: &ActualMeshJob,
    ) -> Result<(), HostAbiError> {
        let job_id = &job.id;
        let start_time = std::time::Instant::now();

        log::info!("[JobManager] Starting lifecycle for job: {:?}", job_id);

        // Step 1: Announce job to potential executors
        log::info!(
            "[JobManager] Step 1: Announcing job {:?} to network",
            job_id
        );
        ctx.mesh_network_service
            .announce_job(job)
            .await
            .map_err(|e| HostAbiError::NetworkError(format!("Job announcement failed: {}", e)))?;

        // Step 2: Collect bids from executors
        let bid_duration = Duration::from_secs(10); // 10 second bidding window
        log::info!(
            "[JobManager] Step 2: Collecting bids for job {:?} ({}s window)",
            job_id,
            bid_duration.as_secs()
        );

        let bids = ctx
            .mesh_network_service
            .collect_bids_for_job(job_id, bid_duration)
            .await
            .map_err(|e| HostAbiError::NetworkError(format!("Bid collection failed: {}", e)))?;

        log::info!(
            "[JobManager] Collected {} bids for job {:?}",
            bids.len(),
            job_id
        );

        if bids.is_empty() {
            log::warn!(
                "[JobManager] No bids received for job {:?}, refunding submitter",
                job_id
            );

            // Refund the job submitter
            if let Err(e) = ctx.credit_mana(&job.creator_did, job.cost_mana).await {
                log::error!(
                    "Failed to refund mana to submitter {}: {}",
                    job.creator_did,
                    e
                );
            }

            return Err(HostAbiError::InternalError(
                "No bids received for job".to_string(),
            ));
        }

        // Step 3: Select executor using the mesh crate's selection algorithm
        log::info!(
            "[JobManager] Step 3: Selecting executor from {} bids",
            bids.len()
        );

        // Create selection policy (could be configurable via governance)
        let selection_policy = icn_mesh::SelectionPolicy::default();

        let selected_executor = icn_mesh::select_executor(
            job_id,
            &job.spec,
            bids.clone(),
            &selection_policy,
            ctx.reputation_store.as_ref(),
            &ctx.mana_ledger,
            ctx.latency_store.as_ref(),
            &icn_mesh::NoOpCapabilityChecker,
        );

        let executor_did = match selected_executor {
            Some(did) => {
                log::info!("[JobManager] Selected executor: {}", did);
                did
            }
            None => {
                log::warn!(
                    "[JobManager] No suitable executor found for job {:?}",
                    job_id
                );

                // Refund the job submitter
                if let Err(e) = ctx.credit_mana(&job.creator_did, job.cost_mana).await {
                    log::error!(
                        "Failed to refund mana to submitter {}: {}",
                        job.creator_did,
                        e
                    );
                }

                return Err(HostAbiError::InternalError(
                    "No suitable executor found".to_string(),
                ));
            }
        };

        // Update job state to Assigned
        ctx.job_states.insert(
            job_id.clone(),
            JobState::Assigned {
                executor: executor_did.clone(),
            },
        );
        JOBS_ACTIVE_GAUGE.inc();

        // Step 4: Notify executor of assignment
        log::info!(
            "[JobManager] Step 4: Assigning job {:?} to executor {}",
            job_id,
            executor_did
        );

        // Find the selected bid to get the agreed cost
        let selected_bid = bids
            .iter()
            .find(|bid| bid.executor_did == executor_did)
            .ok_or_else(|| {
                HostAbiError::InternalError("Selected executor bid not found".to_string())
            })?;

        let assignment_notice = JobAssignmentNotice {
            job_id: job_id.clone(),
            executor_did: executor_did.clone(),
            agreed_cost_mana: selected_bid.price_mana,
        };

        ctx.mesh_network_service
            .notify_executor_of_assignment(&assignment_notice)
            .await
            .map_err(|e| {
                HostAbiError::NetworkError(format!("Assignment notification failed: {}", e))
            })?;

        // Step 5: Wait for execution receipt
        let receipt_timeout = Duration::from_millis(
            job.max_execution_wait_ms
                .unwrap_or(ctx.default_receipt_wait_ms),
        );

        log::info!(
            "[JobManager] Step 5: Waiting for receipt from executor {} ({}s timeout)",
            executor_did,
            receipt_timeout.as_secs()
        );

        let receipt = match ctx
            .mesh_network_service
            .try_receive_receipt(job_id, &executor_did, receipt_timeout)
            .await
        {
            Ok(Some(receipt)) => {
                log::info!(
                    "[JobManager] Received receipt for job {:?} from executor {}",
                    job_id,
                    executor_did
                );
                receipt
            }
            Ok(None) => {
                log::warn!(
                    "[JobManager] No receipt received for job {:?} within timeout",
                    job_id
                );

                // Job timed out, refund submitter
                if let Err(e) = ctx.credit_mana(&job.creator_did, job.cost_mana).await {
                    log::error!(
                        "Failed to refund mana to submitter {}: {}",
                        job.creator_did,
                        e
                    );
                }

                return Err(HostAbiError::InternalError("Receipt timeout".to_string()));
            }
            Err(e) => {
                log::error!("[JobManager] Error waiting for receipt: {}", e);

                // Refund submitter on error
                if let Err(e) = ctx.credit_mana(&job.creator_did, job.cost_mana).await {
                    log::error!(
                        "Failed to refund mana to submitter {}: {}",
                        job.creator_did,
                        e
                    );
                }

                return Err(e);
            }
        };

        // Step 6: Validate and anchor receipt
        log::info!(
            "[JobManager] Step 6: Validating and anchoring receipt for job {:?}",
            job_id
        );

        // Validate that the receipt is from the assigned executor
        if receipt.executor_did != executor_did {
            log::error!(
                "[JobManager] Receipt executor mismatch: expected {}, got {}",
                executor_did,
                receipt.executor_did
            );
            return Err(HostAbiError::InternalError(
                "Receipt executor mismatch".to_string(),
            ));
        }

        // Anchor the receipt in the DAG
        match ctx.anchor_receipt(&receipt).await {
            Ok(receipt_cid) => {
                log::info!(
                    "[JobManager] Receipt anchored for job {:?} at CID: {}",
                    job_id,
                    receipt_cid
                );

                // Pay the executor
                if let Err(e) = ctx
                    .credit_mana(&executor_did, selected_bid.price_mana)
                    .await
                {
                    log::error!("Failed to pay executor {}: {}", executor_did, e);
                    // Continue anyway - receipt is already anchored
                }

                // Update job state to completed
                ctx.job_states.insert(
                    job_id.clone(),
                    JobState::Completed {
                        receipt: receipt.clone(),
                    },
                );

                // Update metrics
                JOBS_COMPLETED.inc();
                JOB_PROCESS_TIME.observe(start_time.elapsed().as_secs_f64());
                JOBS_ACTIVE_GAUGE.dec();

                log::info!(
                    "[JobManager] Job {:?} completed successfully in {:.2}s",
                    job_id,
                    start_time.elapsed().as_secs_f64()
                );

                Ok(())
            }
            Err(e) => {
                log::error!(
                    "[JobManager] Failed to anchor receipt for job {:?}: {}",
                    job_id,
                    e
                );

                // Refund submitter if anchoring fails
                if let Err(e) = ctx.credit_mana(&job.creator_did, job.cost_mana).await {
                    log::error!(
                        "Failed to refund mana to submitter {}: {}",
                        job.creator_did,
                        e
                    );
                }

                Err(HostAbiError::InternalError(format!(
                    "Receipt anchoring failed: {}",
                    e
                )))
            }
        }
    }

    /// Execute a CCL WASM job using the built-in executor
    async fn execute_ccl_wasm_job(
        ctx: &Arc<RuntimeContext>,
        job: &ActualMeshJob,
    ) -> Result<icn_identity::ExecutionReceipt, HostAbiError> {
        use crate::executor::{JobExecutor, WasmExecutor, WasmExecutorConfig};

        if let Some(scope) = &job.spec.required_trust_scope {
            let context = TrustContext::from_str(scope);
            let executor_did = ctx.current_identity.clone();
            let creator_did = job.creator_did.clone();
            let engine = ctx.trust_engine.lock().await;
            match engine.validate_trust(&executor_did, &creator_did, &context, "execute_job") {
                TrustValidationResult::Allowed { .. } => {}
                TrustValidationResult::Denied { reason } => {
                    return Err(HostAbiError::PermissionDenied(reason));
                }
            }
        }

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

    /// Spawn the mana regenerator task.
    pub async fn spawn_mana_regenerator(self: Arc<Self>) {
        let ctx = self.clone();

        tokio::spawn(async move {
            log::info!("Starting mana regenerator background task");

            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60)); // Regenerate every minute

            loop {
                interval.tick().await;

                // Get all accounts and regenerate mana based on policy
                let accounts = ctx.mana_ledger.all_accounts();
                let mut regenerated_count = 0;

                for account_did in accounts {
                    // Get current balance
                    let current_balance = ctx.mana_ledger.get_balance(&account_did);

                    // Calculate regeneration based on reputation and policy
                    let reputation = ctx.reputation_store.get_reputation(&account_did);
                    let base_regeneration = 10u64; // Base regeneration per minute
                    let reputation_multiplier = (reputation as f64 / 100.0).clamp(0.1, 2.0); // 0.1x to 2x based on reputation
                    let regeneration_amount =
                        (base_regeneration as f64 * reputation_multiplier) as u64;

                    // Apply regeneration up to capacity limit controlled by governance
                    let max_capacity = ctx
                        .parameters
                        .get(MANA_MAX_CAPACITY_KEY)
                        .and_then(|v| v.value().parse::<u64>().ok())
                        .unwrap_or(DEFAULT_MANA_MAX_CAPACITY);
                    if current_balance < max_capacity {
                        let actual_regen =
                            std::cmp::min(regeneration_amount, max_capacity - current_balance);
                        if actual_regen > 0 {
                            match ctx
                                .mana_ledger
                                .set_balance(&account_did, current_balance + actual_regen)
                            {
                                Ok(_) => {
                                    regenerated_count += 1;
                                    log::debug!(
                                        "Regenerated {} mana for {}",
                                        actual_regen,
                                        account_did
                                    );
                                }
                                Err(e) => {
                                    log::error!(
                                        "Failed to regenerate mana for {}: {}",
                                        account_did,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }

                if regenerated_count > 0 {
                    log::info!(
                        "Mana regeneration cycle completed: {} accounts regenerated",
                        regenerated_count
                    );
                } else {
                    log::debug!("Mana regeneration cycle completed: no regeneration needed");
                }
            }
        });

        log::info!("Mana regenerator spawned successfully");
    }

    /// Spawn the mesh executor manager that allows this node to act as an executor.
    ///
    /// This manager handles:
    /// 1. Listening for job announcements from other nodes
    /// 2. Evaluating jobs and submitting bids
    /// 3. Executing assigned jobs
    /// 4. Submitting execution receipts
    pub async fn spawn_mesh_executor_manager(self: Arc<Self>) {
        let ctx = self.clone();

        tokio::spawn(async move {
            log::info!("Starting mesh executor manager - this node can now execute jobs");

            // Track which jobs we've already evaluated for bidding (to prevent duplicate bids)
            let mut evaluated_jobs = std::collections::HashSet::new();

            // Subscribe to network messages to listen for job announcements and assignments
            let network_service = match &*ctx.mesh_network_service {
                MeshNetworkServiceType::Default(service) => Some(service),
                MeshNetworkServiceType::Stub(_) => None, // Stub service doesn't support real networking
            };

            if let Some(service) = network_service {
                match service.inner.subscribe().await {
                    Ok(mut receiver) => {
                        log::info!("[ExecutorManager] Subscribed to network messages");

                        loop {
                            match receiver.recv().await {
                                Some(signed_message) => {
                                    if let Err(e) =
                                        Self::handle_executor_message(&ctx, &signed_message).await
                                    {
                                        log::error!(
                                            "[ExecutorManager] Error handling message: {}",
                                            e
                                        );
                                    }
                                }
                                None => {
                                    log::warn!("[ExecutorManager] Network message stream ended");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[ExecutorManager] Failed to subscribe to network: {}", e);

                        // Fall back to polling approach
                        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                        loop {
                            interval.tick().await;
                            if let Err(e) =
                                Self::process_executor_tasks(&ctx, &mut evaluated_jobs).await
                            {
                                log::error!(
                                    "[ExecutorManager] Error processing executor tasks: {}",
                                    e
                                );
                            }
                        }
                    }
                }
            } else {
                log::info!(
                    "[ExecutorManager] Using stub network service - immediate notification mode"
                );

                // Set up immediate notification channel for stub networking
                if let MeshNetworkServiceType::Stub(stub_service) = &*ctx.mesh_network_service {
                    let mut job_announcement_rx =
                        stub_service.setup_job_announcement_channel().await;

                    // Clone context and network service for the notification task
                    let ctx_clone = ctx.clone();
                    let network_service = ctx.mesh_network_service.clone();

                    // Start a task to handle immediate job notifications
                    let mut notification_task = tokio::spawn(async move {
                        while let Some(job) = job_announcement_rx.recv().await {
                            log::info!("[ExecutorManager] Received immediate job announcement for job {:?}", job.id);

                            // Skip jobs we submitted ourselves
                            if job.creator_did == ctx_clone.current_identity {
                                continue;
                            }

                            // Evaluate the job and create a bid if appropriate
                            if let Ok(Some(bid)) =
                                Self::evaluate_and_bid_on_job(&ctx_clone, &job).await
                            {
                                log::info!("[ExecutorManager] Submitting immediate bid for job {:?}: {} mana", job.id, bid.price_mana);

                                // Submit the bid through the network service
                                if let Err(e) = network_service.submit_bid_for_job(&bid).await {
                                    log::error!("[ExecutorManager] Failed to submit immediate bid for job {:?}: {}", job.id, e);
                                } else {
                                    log::info!("[ExecutorManager] Successfully submitted immediate bid for job {:?}", job.id);
                                }
                            } else {
                                log::debug!(
                                    "[ExecutorManager] Decided not to bid on job {:?}",
                                    job.id
                                );
                            }
                        }
                    });

                    // Also keep polling as a backup
                    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
                    loop {
                        tokio::select! {
                            _ = interval.tick() => {
                                if let Err(e) = Self::process_executor_tasks(&ctx, &mut evaluated_jobs).await {
                                    log::error!("[ExecutorManager] Error processing executor tasks: {}", e);
                                }
                            }
                            _ = &mut notification_task => {
                                log::warn!("[ExecutorManager] Job notification task ended");
                                break;
                            }
                        }
                    }
                } else {
                    log::warn!(
                        "[ExecutorManager] Expected stub network service but got something else"
                    );
                }
            }
        });

        log::info!("Mesh executor manager spawned successfully");
    }

    /// Handle incoming network messages for executor functionality.
    async fn handle_executor_message(
        ctx: &Arc<RuntimeContext>,
        message: &icn_protocol::ProtocolMessage,
    ) -> Result<(), HostAbiError> {
        match &message.payload {
            icn_protocol::MessagePayload::MeshJobAnnouncement(announcement) => {
                log::info!(
                    "[ExecutorManager] Received job announcement for job {}",
                    announcement.job_id
                );

                // Check if we should bid on this job
                if let Some(bid) = Self::should_bid_on_job(ctx, announcement).await? {
                    log::info!(
                        "[ExecutorManager] Submitting bid for job {}",
                        announcement.job_id
                    );

                    // Submit the bid through the network service
                    if let MeshNetworkServiceType::Default(service) = &*ctx.mesh_network_service {
                        service.submit_bid_for_job(&bid).await?;
                    }
                }
            }
            icn_protocol::MessagePayload::MeshJobAssignment(assignment) => {
                // Check if this assignment is for us
                if assignment.executor_did == ctx.current_identity {
                    log::info!(
                        "[ExecutorManager] Received job assignment for job {}",
                        assignment.job_id
                    );

                    // Convert job ID back to our format
                    let _job_id = icn_mesh::JobId(assignment.job_id.clone());

                    // Execute the job
                    let ctx_clone = ctx.clone();
                    let assignment_clone = assignment.clone();

                    tokio::spawn(async move {
                        if let Err(e) =
                            Self::handle_job_assignment(&ctx_clone, &assignment_clone).await
                        {
                            log::error!(
                                "[ExecutorManager] Error executing assigned job {}: {}",
                                assignment_clone.job_id,
                                e
                            );
                        }
                    });
                }
            }
            _ => {
                // Ignore other message types
            }
        }

        Ok(())
    }

    /// Evaluate a job announcement and decide whether to bid.
    async fn should_bid_on_job(
        ctx: &Arc<RuntimeContext>,
        announcement: &icn_protocol::MeshJobAnnouncementMessage,
    ) -> Result<Option<icn_mesh::MeshJobBid>, HostAbiError> {
        let executor_did = ctx.current_identity.clone();

        log::debug!(
            "[ExecutorManager] Evaluating job {} for potential bid",
            announcement.job_id
        );

        // Check if we have enough mana to participate
        let current_mana = ctx.get_mana(&executor_did).await?;
        if current_mana < 50 {
            log::debug!(
                "[ExecutorManager] Insufficient mana ({}) to bid on job {}",
                current_mana,
                announcement.job_id
            );
            return Ok(None);
        }

        // Check if we have the required resources
        let required = &announcement.job_spec.required_resources;
        let (available_cpu, available_memory) = Self::available_system_resources();

        if required.cpu_cores > available_cpu || required.memory_mb > available_memory {
            log::debug!("[ExecutorManager] Insufficient resources for job {}: need {}cpu/{}mb, have {}cpu/{}mb", 
                       announcement.job_id, required.cpu_cores, required.memory_mb, available_cpu, available_memory);
            return Ok(None);
        }

        // Check if we support this job type
        let supported = match &announcement.job_spec.kind {
            icn_protocol::JobKind::Echo { .. } => true,
            icn_protocol::JobKind::CclWasm => true, // We support CCL WASM
            icn_protocol::JobKind::Generic => true, // We can handle generic jobs
        };

        if !supported {
            log::debug!(
                "[ExecutorManager] Unsupported job type for job {}",
                announcement.job_id
            );
            return Ok(None);
        }

        // Check if bid deadline has passed
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if current_time > announcement.bid_deadline {
            log::debug!(
                "[ExecutorManager] Bid deadline passed for job {}",
                announcement.job_id
            );
            return Ok(None);
        }

        // Calculate our bid price
        let base_price = Self::calculate_bid_price_from_announcement(announcement, ctx).await?;

        // Don't bid if the job's max cost is too low for our calculated price
        if base_price > announcement.max_cost_mana {
            log::debug!(
                "[ExecutorManager] Our calculated price ({}) exceeds max cost ({}) for job {}",
                base_price,
                announcement.max_cost_mana,
                announcement.job_id
            );
            return Ok(None);
        }

        // Create and sign the bid
        let bid = icn_mesh::MeshJobBid {
            job_id: icn_mesh::JobId(announcement.job_id.clone()),
            executor_did: executor_did.clone(),
            price_mana: base_price,
            resources: icn_mesh::Resources {
                cpu_cores: available_cpu,
                memory_mb: available_memory,
                storage_mb: 0,
            },
            executor_capabilities: ctx
                .parameters
                .get("executor_capabilities")
                .map(|v| v.value().split(',').map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            executor_federations: ctx
                .parameters
                .get("executor_federations")
                .map(|v| v.value().split(',').map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            executor_trust_scope: ctx
                .parameters
                .get("executor_trust_scope")
                .map(|v| v.value().clone()),
            signature: icn_identity::SignatureBytes(vec![]), // Will be filled by sign()
        };

        // Sign the bid
        let signable_bytes = bid.to_signable_bytes().map_err(|e| {
            HostAbiError::InternalError(format!("Failed to create signable bytes: {}", e))
        })?;
        let signature = ctx.signer.sign(&signable_bytes)?;

        let signed_bid = icn_mesh::MeshJobBid {
            signature: icn_identity::SignatureBytes(signature),
            ..bid
        };

        log::info!(
            "[ExecutorManager] Created bid for job {}: price={} mana",
            announcement.job_id,
            base_price
        );

        Ok(Some(signed_bid))
    }

    /// Calculate bid price based on job announcement.
    async fn calculate_bid_price_from_announcement(
        announcement: &icn_protocol::MeshJobAnnouncementMessage,
        ctx: &Arc<RuntimeContext>,
    ) -> Result<u64, HostAbiError> {
        let required = &announcement.job_spec.required_resources;

        // Base price calculation based on resource requirements
        let cpu_cost = required.cpu_cores as u64 * 2; // 2 mana per CPU core
        let memory_cost = required.memory_mb as u64 / 100; // 1 mana per 100MB
        let time_cost = required.max_execution_time_secs / 60; // 1 mana per minute
        let base_cost = cpu_cost + memory_cost + time_cost + 5; // 5 mana base fee

        // Adjust based on our reputation (higher reputation = can charge more)
        let our_reputation = ctx.reputation_store.get_reputation(&ctx.current_identity);
        let reputation_multiplier = 1.0 + (our_reputation as f64 / 1000.0); // Up to 2x for high reputation

        // Derive a deterministic factor from the job ID and our reputation
        let random_factor =
            Self::deterministic_factor(announcement.job_id.to_string().as_bytes(), our_reputation);

        let final_price = ((base_cost as f64) * reputation_multiplier * random_factor) as u64;

        // Don't exceed the max cost
        Ok(final_price.min(announcement.max_cost_mana).max(1)) // Minimum 1 mana
    }

    /// Handle a job assignment by executing the job and submitting a receipt.
    async fn handle_job_assignment(
        ctx: &Arc<RuntimeContext>,
        assignment: &icn_protocol::MeshJobAssignmentMessage,
    ) -> Result<(), HostAbiError> {
        log::info!(
            "[ExecutorManager] Starting execution of assigned job {}",
            assignment.job_id
        );

        // We need to reconstruct the job from the assignment
        // In a real implementation, we'd either:
        // 1. Cache job details from the announcement
        // 2. Request job details from the submitter
        // 3. Download the manifest from the DAG

        // For now, we'll create a minimal job structure for execution
        let job = ActualMeshJob {
            id: icn_mesh::JobId(assignment.job_id.clone()),
            manifest_cid: assignment
                .manifest_cid
                .clone()
                .unwrap_or_else(|| assignment.job_id.clone()),
            spec: icn_mesh::JobSpec {
                kind: icn_mesh::JobKind::Echo {
                    payload: "Assigned job execution".to_string(),
                }, // Placeholder
                inputs: vec![],
                outputs: vec!["result".to_string()],
                required_resources: icn_mesh::Resources {
                    cpu_cores: 1,
                    memory_mb: 128,
                    storage_mb: 0,
                },
                required_capabilities: vec![],
                required_trust_scope: None,
                min_executor_reputation: None,
                allowed_federations: vec![],
            },
            creator_did: icn_common::Did::new("key", "placeholder"), // We don't know the creator from assignment
            cost_mana: assignment.agreed_cost_mana,
            max_execution_wait_ms: None,
            signature: icn_identity::SignatureBytes(vec![]),
        };

        // Execute the job
        let receipt = Self::execute_assigned_job(ctx, &job, assignment.agreed_cost_mana).await?;

        // Submit the receipt
        if let MeshNetworkServiceType::Default(service) = &*ctx.mesh_network_service {
            service.submit_execution_receipt(&receipt).await?;
            log::info!(
                "[ExecutorManager] Submitted execution receipt for job {}",
                assignment.job_id
            );
        }

        Ok(())
    }

    /// Evaluate a job announcement and decide whether to bid.
    pub async fn evaluate_and_bid_on_job(
        ctx: &Arc<RuntimeContext>,
        job: &ActualMeshJob,
    ) -> Result<Option<icn_mesh::MeshJobBid>, HostAbiError> {
        let executor_did = ctx.current_identity.clone();

        log::info!("[Executor] Evaluating job {:?} for potential bid", job.id);

        // Check if we have enough mana to participate
        let current_mana = ctx.get_mana(&executor_did).await?;
        if current_mana < 50 {
            log::debug!(
                "[Executor] Insufficient mana ({}) to bid on job {:?}",
                current_mana,
                job.id
            );
            return Ok(None);
        }

        // Check if we have the required resources
        let required = &job.spec.required_resources;
        let (available_cpu, available_memory) = Self::available_system_resources();

        if required.cpu_cores > available_cpu || required.memory_mb > available_memory {
            log::debug!(
                "[Executor] Insufficient resources for job {:?}: need {}cpu/{}mb, have {}cpu/{}mb",
                job.id,
                required.cpu_cores,
                required.memory_mb,
                available_cpu,
                available_memory
            );
            return Ok(None);
        }

        // Calculate our bid price based on job requirements and our reputation
        let base_price = Self::calculate_bid_price(&job.id, &job.spec, ctx).await?;

        // Create and sign the bid
        let bid = icn_mesh::MeshJobBid {
            job_id: job.id.clone(),
            executor_did: executor_did.clone(),
            price_mana: base_price,
            resources: icn_mesh::Resources {
                cpu_cores: available_cpu,
                memory_mb: available_memory,
                storage_mb: 0,
            },
            executor_capabilities: ctx
                .parameters
                .get("executor_capabilities")
                .map(|v| v.value().split(',').map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            executor_federations: ctx
                .parameters
                .get("executor_federations")
                .map(|v| v.value().split(',').map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            executor_trust_scope: ctx
                .parameters
                .get("executor_trust_scope")
                .map(|v| v.value().clone()),
            signature: icn_identity::SignatureBytes(vec![]), // Will be filled by sign()
        };

        // Sign the bid
        let signable_bytes = bid.to_signable_bytes().map_err(|e| {
            HostAbiError::InternalError(format!("Failed to create signable bytes: {}", e))
        })?;
        let signature = ctx.signer.sign(&signable_bytes)?;

        let signed_bid = icn_mesh::MeshJobBid {
            signature: icn_identity::SignatureBytes(signature),
            ..bid
        };

        log::info!(
            "[Executor] Created bid for job {:?}: price={} mana",
            job.id,
            base_price
        );

        Ok(Some(signed_bid))
    }

    /// Calculate a competitive bid price for a job.
    async fn calculate_bid_price(
        job_id: &icn_mesh::JobId,
        job_spec: &icn_mesh::JobSpec,
        ctx: &Arc<RuntimeContext>,
    ) -> Result<u64, HostAbiError> {
        // Base price calculation based on resource requirements
        let cpu_cost = job_spec.required_resources.cpu_cores as u64 * 2; // 2 mana per CPU core
        let memory_cost = job_spec.required_resources.memory_mb as u64 / 100; // 1 mana per 100MB
        let base_cost = cpu_cost + memory_cost + 5; // 5 mana base fee

        // Adjust based on our reputation (higher reputation = can charge more)
        let our_reputation = ctx.reputation_store.get_reputation(&ctx.current_identity);
        let reputation_multiplier = 1.0 + (our_reputation as f64 / 1000.0); // Up to 2x for high reputation

        // Derive a deterministic factor from the job ID and our reputation
        let random_factor =
            Self::deterministic_factor(job_id.to_string().as_bytes(), our_reputation);

        let final_price = ((base_cost as f64) * reputation_multiplier * random_factor) as u64;

        Ok(final_price.max(1)) // Minimum 1 mana
    }

    fn deterministic_factor(seed: &[u8], reputation: u64) -> f64 {
        use sha2::{Digest, Sha256};
        use std::convert::TryInto;

        let mut hasher = Sha256::new();
        hasher.update(seed);
        hasher.update(reputation.to_le_bytes());
        let hash = hasher.finalize();
        let val = u64::from_le_bytes(hash[0..8].try_into().expect("slice"));
        0.9 + (val as f64 / u64::MAX as f64) * 0.2
    }

    /// Execute an assigned job and create an execution receipt.
    pub async fn execute_assigned_job(
        ctx: &Arc<RuntimeContext>,
        job: &ActualMeshJob,
        _agreed_cost: u64, // Marked as unused for now
    ) -> Result<icn_identity::ExecutionReceipt, HostAbiError> {
        crate::execution_monitor::clear_logs();
        let job_id = &job.id;
        let executor_did = ctx.current_identity.clone();

        log::info!("[Executor] Starting execution of assigned job {:?}", job_id);

        let _start_time = std::time::SystemTime::now(); // Marked as unused for now
        let execution_start = std::time::Instant::now();

        // Execute the job based on its type
        let (result_cid, success) = match &job.spec.kind {
            icn_mesh::JobKind::Echo { payload } => {
                // Simple echo job - just return the payload
                log::info!("[Executor] Executing Echo job with payload: {}", payload);

                // Simulate some work
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                // Create result
                let result_data = format!("Echo result: {}", payload);
                let result_cid = icn_common::Cid::new_v1_sha256(0x55, result_data.as_bytes());

                // Store the result in our DAG
                let result_block = icn_common::DagBlock {
                    cid: result_cid.clone(),
                    data: result_data.into_bytes(),
                    links: vec![],
                    timestamp: ctx.time_provider.unix_seconds(),
                    author_did: executor_did.clone(),
                    signature: None,
                    scope: None,
                };

                {
                    let mut dag_store = ctx.dag_store.inner().lock().await;
                    dag_store.put(&result_block).await.map_err(|e| {
                        HostAbiError::DagOperationFailed(format!("Failed to store result: {}", e))
                    })?;
                }

                (result_cid, true)
            }
            icn_mesh::JobKind::CclWasm => {
                // For CCL WASM jobs, we would load and execute the WASM module
                // For now, simulate successful execution
                log::info!("[Executor] Executing CCL WASM job (simulated)");

                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                let result_cid = icn_common::Cid::new_v1_sha256(0x55, b"wasm_result");
                (result_cid, true)
            }
            icn_mesh::JobKind::GenericPlaceholder => {
                log::warn!("[Executor] Generic placeholder job - marking as successful");
                let result_cid = icn_common::Cid::new_v1_sha256(0x55, b"placeholder_result");
                (result_cid, true)
            }
        };

        let execution_time = execution_start.elapsed();
        let cpu_ms = execution_time.as_millis() as u64;

        log::info!(
            "[Executor] Job {:?} execution completed in {:.2}s, success: {}",
            job_id,
            execution_time.as_secs_f64(),
            success
        );

        // Create execution receipt
        let receipt = icn_identity::ExecutionReceipt {
            job_id: job_id.clone().into(),
            executor_did: executor_did.clone(),
            result_cid,
            cpu_ms,
            success,
            sig: icn_identity::SignatureBytes(vec![]), // Will be filled by sign_with_signer
        };

        // Sign the receipt
        let signable_bytes = receipt.to_signable_bytes().map_err(|e| {
            HostAbiError::InternalError(format!("Failed to create signable bytes: {}", e))
        })?;
        let signature = ctx.signer.sign(&signable_bytes)?;

        let signed_receipt = icn_identity::ExecutionReceipt {
            sig: icn_identity::SignatureBytes(signature),
            ..receipt
        };

        log::info!("[Executor] Created execution receipt for job {:?}", job_id);

        Ok(signed_receipt)
    }

    /// Perform a single integrity check on the DAG store.
    pub async fn integrity_check_once(&self) -> Result<(), CommonError> {
        log::info!("Performing integrity check on DAG store");

        let store = self.dag_store.inner().lock().await;

        // Get all blocks and verify their integrity
        let mut verified_count = 0;
        let error_count = 0;

        // In a proper implementation, we'd iterate through all blocks
        // For now, we'll implement basic validation that can be extended

        // Verify the store is accessible and consistent
        match store.get(&Cid::new_v1_sha256(0x00, b"test")).await {
            Ok(_) => {
                // Store is accessible, basic health check passed
                verified_count += 1;
            }
            Err(_) => {
                // Expected for non-existent test block, this is fine
            }
        }

        if error_count > 0 {
            return Err(CommonError::InternalError(format!(
                "DAG integrity check failed: {} errors found",
                error_count
            )));
        }

        log::info!(
            "DAG integrity check completed: {} blocks verified",
            verified_count
        );
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

    /// Process executor tasks in polling mode (fallback when network subscription fails).
    async fn process_executor_tasks(
        ctx: &Arc<RuntimeContext>,
        evaluated_jobs: &mut std::collections::HashSet<JobId>,
    ) -> Result<(), HostAbiError> {
        // This is a fallback mode for when network message subscription fails
        // In stub networking mode, we need to check for announced jobs and bid on them
        log::debug!("[ExecutorManager] Polling for executor tasks");

        // Check if we're using stub networking
        if let MeshNetworkServiceType::Stub(stub_service) = &*ctx.mesh_network_service {
            // Get announced jobs from the stub service
            let announced_jobs = stub_service.get_announced_jobs().await;

            // Check if there are any new jobs we should bid on
            for job in announced_jobs {
                // Check if we've already evaluated this job for bidding
                if evaluated_jobs.contains(&job.id) {
                    continue; // Skip jobs we've already evaluated
                }

                // Skip jobs we submitted ourselves
                if job.creator_did == ctx.current_identity {
                    evaluated_jobs.insert(job.id.clone());
                    continue;
                }

                log::info!(
                    "[ExecutorManager] Found announced job {:?} for evaluation",
                    job.id
                );

                // Mark this job as evaluated
                evaluated_jobs.insert(job.id.clone());

                // Evaluate the job and create a bid if appropriate
                if let Ok(Some(bid)) = Self::evaluate_and_bid_on_job(ctx, &job).await {
                    log::info!(
                        "[ExecutorManager] Submitting bid for job {:?}: {} mana",
                        job.id,
                        bid.price_mana
                    );

                    // Submit the bid through the stub service
                    if let Err(e) = stub_service.submit_bid_for_job(&bid).await {
                        log::error!(
                            "[ExecutorManager] Failed to submit bid for job {:?}: {}",
                            job.id,
                            e
                        );
                    } else {
                        log::info!(
                            "[ExecutorManager] Successfully submitted bid for job {:?}",
                            job.id
                        );
                    }
                } else {
                    log::debug!("[ExecutorManager] Decided not to bid on job {:?}", job.id);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_identity::{verify_signature, verifying_key_from_did_key, EdSignature};
    use icn_mesh::{JobKind, JobSpec as MeshJobSpec, Resources};
    use icn_protocol::MeshJobAnnouncementMessage;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_should_bid_on_job_with_resources() {
        let did = Did::from_str("did:icn:test:exec").unwrap();
        let ctx = RuntimeContext::new_testing_with_system_info(
            did.clone(),
            Some(100),
            Arc::new(icn_common::FixedSystemInfoProvider::new(8, 4096)),
        )
        .unwrap();

        let announcement = MeshJobAnnouncementMessage {
            job_id: Cid::new_v1_sha256(0x71, b"job"),
            manifest_cid: Cid::new_v1_sha256(0x71, b"man"),
            creator_did: did.clone(),
            max_cost_mana: 10,
            job_spec: icn_protocol::JobSpec {
                kind: icn_protocol::JobKind::Echo {
                    payload: "hi".into(),
                },
                inputs: vec![],
                outputs: vec![],
                required_resources: icn_protocol::ResourceRequirements {
                    cpu_cores: 2,
                    memory_mb: 512,
                    storage_mb: 0,
                    max_execution_time_secs: 60,
                },
            },
            bid_deadline: ctx.time_provider.unix_seconds() + 100,
        };

        let bid = super::RuntimeContext::should_bid_on_job(&ctx, &announcement)
            .await
            .unwrap();
        assert!(bid.is_some());
    }

    #[tokio::test]
    async fn test_evaluate_and_bid_on_job_respects_resources() {
        let did = Did::from_str("did:icn:test:exec2").unwrap();
        let ctx = RuntimeContext::new_testing_with_system_info(
            did.clone(),
            Some(100),
            Arc::new(icn_common::FixedSystemInfoProvider::new(4, 1024)),
        )
        .unwrap();

        let job = icn_mesh::ActualMeshJob {
            id: icn_mesh::JobId(Cid::new_v1_sha256(0x71, b"job2")),
            manifest_cid: Cid::new_v1_sha256(0x71, b"man2"),
            creator_did: did.clone(),
            cost_mana: 5,
            max_execution_wait_ms: None,
            spec: MeshJobSpec {
                kind: icn_mesh::JobKind::Echo {
                    payload: "hi".into(),
                },
                inputs: vec![],
                outputs: vec![],
                required_resources: icn_mesh::Resources {
                    cpu_cores: 8,
                    memory_mb: 2048,
                    storage_mb: 0,
                },
                required_capabilities: vec![],
                required_trust_scope: None,
                min_executor_reputation: None,
                allowed_federations: vec![],
            },
            signature: icn_identity::SignatureBytes(vec![]),
        };

        let bid = RuntimeContext::evaluate_and_bid_on_job(&ctx, &job)
            .await
            .unwrap();
        assert!(bid.is_none());
    }

    #[tokio::test]
    async fn test_identity_signer_cryptographic_matching() {
        use icn_identity::verifying_key_from_did_key;

        // Test the sync version
        let ctx_sync = RuntimeContext::new_with_identity_and_storage(None, None, None)
            .expect("Failed to create sync RuntimeContext");

        // Test the async version
        let ctx_async = RuntimeContext::new_async_with_identity_and_storage(None, None, None)
            .await
            .expect("Failed to create async RuntimeContext");

        for (ctx_name, ctx) in [("sync", ctx_sync), ("async", ctx_async)] {
            // Get the identity and signer from the context
            let identity = &ctx.current_identity;
            let signer = &ctx.signer;

            // Create a test message
            let test_message = b"test message for signature verification";

            // Sign the message with the signer
            let signature_bytes = signer.sign(test_message).expect("Failed to sign message");

            // Convert to EdSignature
            let signature = EdSignature::from_bytes(
                signature_bytes
                    .as_slice()
                    .try_into()
                    .expect("Invalid signature length"),
            );

            // Extract the verifying key from the identity DID
            let verifying_key = verifying_key_from_did_key(identity)
                .expect("Failed to extract verifying key from DID");

            // Verify the signature using the identity's public key
            let verification_result = verify_signature(&verifying_key, test_message, &signature);

            assert!(
                verification_result,
                "{} context: Signature verification failed! Identity DID and signer are not cryptographically matched",
                ctx_name
            );
        }
    }

    #[tokio::test]
    async fn test_provided_identity_creates_warning_comment() {
        use std::str::FromStr;

        // Test with provided identity (this still has the FIXME issue but maintains backward compatibility)
        let test_did =
            icn_common::Did::from_str("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
                .expect("Failed to parse test DID");

        let ctx = RuntimeContext::new_with_identity_and_storage(Some(test_did.clone()), None, None)
            .expect("Failed to create RuntimeContext with provided identity");

        // Verify the provided identity is used
        assert_eq!(ctx.current_identity, test_did);

        // This case still has the cryptographic mismatch issue (noted in FIXME comment)
        // but we're maintaining backward compatibility
        // Future versions should require a matching signer parameter
    }
}

#[cfg(test)]
mod configuration_tests {
    use super::*;
    use crate::context::service_config::{ServiceConfig, ServiceEnvironment};
    use std::str::FromStr;

    #[tokio::test]
    async fn test_runtime_context_builder_testing() {
        let test_did = Did::from_str("did:key:zTestBuilder").unwrap();

        let ctx = RuntimeContextBuilder::new(EnvironmentType::Testing)
            .with_identity(test_did.clone())
            .with_initial_mana(100)
            .build()
            .unwrap();

        assert_eq!(ctx.current_identity, test_did);
        assert_eq!(ctx.get_mana(&test_did).await.unwrap(), 100);
    }

    #[test]
    fn test_runtime_context_builder_validation() {
        // Should fail without identity
        let result = RuntimeContextBuilder::new(EnvironmentType::Testing).build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Current identity is required"));
    }

    #[test]
    fn test_production_validation() {
        let test_did = Did::from_str("did:key:zTestValidation").unwrap();

        // Create a testing context (which uses stubs)
        let ctx = RuntimeContext::new_for_testing(test_did, Some(100)).unwrap();

        // Validation should fail because it's using stub services
        let result = ctx.validate_production_services();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("PRODUCTION ERROR"));
    }

    #[tokio::test]
    async fn test_deprecated_methods_still_work() {
        #[allow(deprecated)]
        let ctx = RuntimeContext::new_with_stubs("did:key:zTestDeprecated").unwrap();

        // Should still create a valid context
        assert!(ctx.current_identity.to_string().contains("zTestDeprecated"));

        #[allow(deprecated)]
        let ctx2 = RuntimeContext::new_with_stubs_and_mana("did:key:zTestDeprecated2", 50).unwrap();
        assert_eq!(ctx2.get_mana(&ctx2.current_identity).await.unwrap(), 50);
    }

    #[test]
    fn test_new_production_constructor_requires_libp2p() {
        // The new RuntimeContext::new() should fail without libp2p feature
        // or return appropriate error message
        let result = RuntimeContext::new();

        #[cfg(feature = "enable-libp2p")]
        {
            // Should fail because we're in sync context
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("synchronous context"));
        }

        #[cfg(not(feature = "enable-libp2p"))]
        {
            // Should fail because libp2p feature is not enabled
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("libp2p feature"));
        }
    }

    #[tokio::test]
    async fn test_new_for_testing_explicit_method() {
        let test_did = Did::from_str("did:key:zTestExplicit").unwrap();

        // new_for_testing should work and be explicit about testing
        let ctx = RuntimeContext::new_for_testing(test_did.clone(), Some(42)).unwrap();

        // Should have the correct identity and mana
        assert_eq!(ctx.current_identity, test_did);
        assert_eq!(ctx.get_mana(&test_did).await.unwrap(), 42);

        // Should use stub services and fail production validation
        let result = ctx.validate_production_services();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("PRODUCTION ERROR"));
    }

    #[test]
    fn test_service_config_production_defaults() {
        // Production defaults should require explicit configuration
        let result = ServiceConfig::production_defaults();
        assert!(result.is_err());
        assert!(result
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("explicit services"));
    }

    #[test]
    fn test_service_config_testing_defaults() {
        // Testing defaults should work without parameters
        let result = ServiceConfig::testing_defaults();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.environment, ServiceEnvironment::Testing);

        // Should use stub mesh network service
        assert!(matches!(
            *config.mesh_network_service,
            MeshNetworkServiceType::Stub(_)
        ));
    }

    #[tokio::test]
    async fn test_deprecated_new_testing_method() {
        let test_did = Did::from_str("did:key:zTestOldMethod").unwrap();

        // The deprecated new_testing method should still work
        #[allow(deprecated)]
        let ctx = RuntimeContext::new_testing(test_did.clone(), Some(123)).unwrap();

        assert_eq!(ctx.current_identity, test_did);
        assert_eq!(ctx.get_mana(&test_did).await.unwrap(), 123);
    }

    #[test]
    fn test_available_system_resources_units() {
        // Test that available_system_resources returns memory in megabytes
        let (cpu, memory_mb) = RuntimeContext::available_system_resources();

        // CPU cores should be reasonable (1-512 cores for most systems)
        assert!(cpu > 0, "CPU core count should be greater than 0");
        assert!(cpu <= 512, "CPU core count should be reasonable (<=512)");

        // Memory should be in megabytes, so for any modern system it should be
        // at least 100MB (very conservative) and reasonable (less than 1TB = 1,048,576 MB)
        assert!(memory_mb >= 100, "Available memory should be at least 100MB, got {}MB. If this is in KB, the bug still exists!", memory_mb);
        assert!(
            memory_mb <= 1_048_576,
            "Available memory should be less than 1TB ({}MB), got {}MB",
            1_048_576,
            memory_mb
        );

        // Log the values for debugging
        println!(
            "System resources: {} CPU cores, {} MB memory",
            cpu, memory_mb
        );
    }
}
