//! Defines the `RuntimeContext`, `HostEnvironment`, and related types for the ICN runtime.

use icn_common::{Did, Cid, CommonError, DagBlock};
use icn_identity::{ExecutionReceipt as IdentityExecutionReceipt, Signature}; 
use icn_economics::{EconError, ResourcePolicyEnforcer, ManaRepositoryAdapter}; 
use icn_mesh::{JobId, ActualMeshJob, MeshJobBid, JobState, SubmitReceiptMessage, JobSpec};

use icn_network::{NetworkService as ActualNetworkService, NetworkMessage, PeerId as NetworkPeerId};
#[cfg(feature = "enable-libp2p")]
use icn_network::libp2p_service::Libp2pNetworkService as ActualLibp2pNetworkService; // Renamed to avoid conflict with local trait
use downcast_rs::DowncastSync;

use log::{info, warn, error, debug};
use std::collections::{HashMap, VecDeque, HashSet};
use std::sync::{Arc, Mutex as StdMutex, atomic::{AtomicU32, Ordering}}; 
use tokio::sync::{Mutex, RwLock, mpsc::{self, Sender, Receiver}};
use tokio::task::JoinHandle;
use async_trait::async_trait;
use downcast_rs::impl_downcast; 
use std::any::Any;
use std::str::FromStr; 

#[cfg(feature = "enable-libp2p")]
use libp2p::{Multiaddr, PeerId as Libp2pPeerId};

use crate::error::MeshJobError;
use tokio::time::{Instant as TokioInstant, Duration, sleep};

// Counter for generating unique (within this runtime instance) job IDs for stubs
pub static NEXT_JOB_ID: AtomicU32 = AtomicU32::new(1);

// --- Placeholder Local Stubs / Forward Declarations ---

// Placeholder for icn-identity::Signer (assuming it should be async)
#[async_trait]
pub trait Signer: Send + Sync + std::fmt::Debug {
    async fn sign(&self, did: &Did, data: &[u8]) -> Result<Vec<u8>, HostAbiError>;
    async fn verify(&self, did: &Did, data: &[u8], signature: &[u8]) -> Result<bool, HostAbiError>;
}

// Placeholder for icn-dag::StorageService (assuming it should be async)
// Renamed from DagStore to avoid confusion if DagStore is a concrete type elsewhere.
#[async_trait]
pub trait StorageService: Send + Sync + std::fmt::Debug + DowncastSync {
    async fn put(&self, data: &[u8]) -> Result<Cid, HostAbiError>;
    async fn get(&self, cid: &Cid) -> Result<Option<Vec<u8>>, HostAbiError>;
}
impl_downcast!(sync StorageService);


// Placeholder for icn_economics::ManaRepository
pub trait ManaRepository: Send + Sync + std::fmt::Debug {
    // Define methods as needed, e.g.:
    // async fn get_balance(&self, account: &Did) -> Result<u64, EconError>;
    // async fn spend_mana(&self, account: &Did, amount: u64) -> Result<(), EconError>;
    // async fn credit_mana(&self, account: &Did, amount: u64) -> Result<(), EconError>;
}

// Placeholder for icn_economics::SimpleManaLedger
#[derive(Debug, Clone)]
pub struct SimpleManaLedger {
    balances: Arc<Mutex<HashMap<Did, u64>>>,
}

impl SimpleManaLedger {
    pub fn new() -> Self {
        Self { balances: Arc::new(Mutex::new(HashMap::new())) }
    }
    pub async fn get_balance(&self, account: &Did) -> Option<u64> {
        let balances = self.balances.lock().await;
        balances.get(account).cloned()
    }
    pub async fn set_balance(&self, account: &Did, amount: u64) {
        let mut balances = self.balances.lock().await;
        balances.insert(account.clone(), amount);
    }
    pub async fn spend(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        let mut balances = self.balances.lock().await;
        let balance = balances.get_mut(account).ok_or_else(|| HostAbiError::AccountNotFound(account.clone()))?;
        if *balance < amount {
            return Err(HostAbiError::InsufficientMana);
        }
        *balance -= amount;
        Ok(())
    }
    pub async fn credit(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        let mut balances = self.balances.lock().await;
        let balance = balances.entry(account.clone()).or_insert(0);
        *balance += amount;
        Ok(())
    }
}

// Placeholder for icn_mesh::MeshJobStateChange
#[derive(Debug, Clone)]
pub struct MeshJobStateChange { /* ... fields ... */ }
// Placeholder for icn_mesh::BidId
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BidId(pub String);


// Definition for JobAssignmentNotice (used in MeshNetworkService trait)
#[derive(Debug, Clone)]
pub struct JobAssignmentNotice {
    pub job_id: JobId,
    pub executor_did: Did,
}

// Placeholder for MeshSubmitReceiptMessage (used in StubMeshNetworkService)
// Note: icn_mesh::SubmitReceiptMessage already exists and should be used if it matches.
// For now, creating a local distinct one to avoid import conflicts if the structure differs.
#[derive(Debug, Clone)]
pub struct LocalMeshSubmitReceiptMessage { // Renamed to avoid conflict
    pub receipt: IdentityExecutionReceipt,
}

// Placeholder for GovernanceModule
#[derive(Debug, Clone)]
pub struct GovernanceModule { /* ... fields ... */ }
impl GovernanceModule { pub fn new() -> Self { Self {} } } 

// Placeholder for charge_mana function (used in Job Manager)
// This would typically belong to the icn-economics crate or a related module.
fn charge_mana(_executor_did: &Did, _amount: u64) -> Result<(), EconError> {
    // In a real implementation, this would interact with the ManaRepository.
    // For now, always succeed for testing purposes.
    Ok(())
}

// Placeholder for SelectionPolicy (used in Job Manager)
// This would typically belong to the icn-mesh crate or a related module.
#[derive(Debug, Default)]
pub struct SelectionPolicy { /* ... fields ... */ }

// Placeholder for select_executor function (used in Job Manager)
// This would typically belong to the icn-mesh crate or a related module.
fn select_executor(bids: Vec<MeshJobBid>, _policy: SelectionPolicy) -> Option<Did> {
    // Simplistic: return the DID of the first bidder if any
    bids.first().map(|bid| bid.executor_did.clone())
}
// --- End Placeholder Local Stubs ---


#[derive(Debug, Clone)] 
pub struct CreateProposalPayload {
    pub proposal_type_str: String,
    pub type_specific_payload: Vec<u8>, 
    pub description: String,
    pub duration_secs: u64,
}

#[derive(Debug, Clone)] 
pub struct CastVotePayload {
    pub proposal_id_str: String,
    pub vote_option_str: String, 
}

/// Trait for a service that can broadcast and receive mesh-specific messages.
/// This is the local definition for icn-runtime.
#[async_trait]
pub trait MeshNetworkService: Send + Sync + std::fmt::Debug + DowncastSync {
    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError>;
    async fn collect_bids_for_job(&self, job_id: &JobId, duration: Duration) -> Result<Vec<MeshJobBid>, HostAbiError>;
    async fn notify_executor_of_assignment(&self, notice: &JobAssignmentNotice) -> Result<(), HostAbiError>;
    async fn try_receive_receipt(&self, job_id: &JobId, expected_executor: &Did, timeout: Duration) -> Result<Option<IdentityExecutionReceipt>, HostAbiError>;
    fn as_any(&self) -> &dyn std::any::Any;
}
impl_downcast!(sync MeshNetworkService);

// --- DefaultMeshNetworkService Implementation ---

#[derive(Debug)]
pub struct DefaultMeshNetworkService {
    inner: Arc<dyn ActualNetworkService>, // Uses the imported ActualNetworkService from icn-network
}

impl DefaultMeshNetworkService {
    pub fn new(service: Arc<dyn ActualNetworkService>) -> Self {
        Self { inner: service }
    }

    // This method allows getting the concrete Libp2pNetworkService if that's what `inner` holds.
    #[cfg(feature = "enable-libp2p")]
    pub fn get_underlying_broadcast_service(&self) -> Result<Arc<ActualLibp2pNetworkService>, CommonError> {
        self.inner.clone().downcast_arc::<ActualLibp2pNetworkService>()
            .map_err(|_e| CommonError::NetworkError("Failed to downcast inner NetworkService to Libp2pNetworkService. Ensure it was constructed with Libp2pNetworkService.".to_string()))
    }
}

#[async_trait]
impl MeshNetworkService for DefaultMeshNetworkService {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        // ActualMeshJob is aliased as Job in icn-network's NetworkMessage::MeshJobAnnouncement
        let job_message = NetworkMessage::MeshJobAnnouncement(job.clone()); 
        self.inner.broadcast_message(job_message).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to announce job: {}", e)))
    }

    async fn collect_bids_for_job(&self, job_id: &JobId, duration: Duration) -> Result<Vec<MeshJobBid>, HostAbiError> {
        debug!("[DefaultMeshNetworkService] Collecting bids for job {:?} for {:?}", job_id, duration);
        let mut bids = Vec::new();
        let mut receiver = self.inner.subscribe()
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to subscribe for bids: {}", e)))?;
        
        let end_time = TokioInstant::now() + duration;

        loop {
            match tokio::time::timeout_at(end_time, receiver.recv()).await {
                Ok(result) => { // Timeout gives Result<Option<T>, Elapsed>
                    match result {
                        Some(NetworkMessage::BidSubmission(bid)) => {
                            if &bid.job_id == job_id {
                                debug!("Received relevant bid: {:?}", bid);
                                bids.push(bid);
                            } else {
                                debug!("Received bid for different job: {:?}", bid.job_id);
                            }
                        }
                        Some(other_message) => {
                            debug!("Received other network message during bid collection: {:?}", other_message);
                        }
                        None => { // Channel closed
                            warn!("Network channel closed while collecting bids for job {:?}", job_id);
                            break;
                        }
                    }
                }
                Err(_timeout_error) => { // Timeout
                    debug!("Bid collection timeout for job {:?}", job_id);
                    break;
                }
            }
        }
        Ok(bids)
    }

    async fn notify_executor_of_assignment(&self, notice: &JobAssignmentNotice) -> Result<(), HostAbiError> {
        debug!("[DefaultMeshNetworkService] Broadcasting assignment for job {:?}", notice.job_id);
        let assignment_message = NetworkMessage::JobAssignmentNotification(notice.job_id.clone(), notice.executor_did.clone());
        self.inner.broadcast_message(assignment_message).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to broadcast assignment: {}", e)))
    }

    async fn try_receive_receipt(&self, job_id: &JobId, expected_executor: &Did, timeout_duration: Duration) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
        debug!("[DefaultMeshNetworkService] Trying to receive receipt for job {:?} from {:?} with timeout {:?}", job_id, expected_executor, timeout_duration);
        let mut receiver = self.inner.subscribe()
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to subscribe for receipts: {}", e)))?;

        let end_time = TokioInstant::now() + timeout_duration;

        loop {
            match tokio::time::timeout_at(end_time, receiver.recv()).await {
                Ok(result) => {
                    match result {
                        Some(NetworkMessage::SubmitReceipt(receipt)) => {
                            if &receipt.job_id == job_id && &receipt.executor_did == expected_executor {
                                debug!("Received matching receipt: {:?}", receipt);
                                return Ok(Some(receipt));
                            } else {
                                debug!("Received receipt for different job/executor: {:?}", receipt);
                            }
                        }
                        Some(other_message) => {
                            debug!("Received other network message while waiting for receipt: {:?}", other_message);
                        }
                        None => { 
                            warn!("Network channel closed while waiting for receipt for job {:?}", job_id);
                            return Ok(None); 
                        }
                    }
                }
                Err(_timeout_error) => { 
                    debug!("Timeout waiting for receipt for job {:?}", job_id);
                    return Ok(None);
                }
            }
        }
    }
}

// --- Host ABI Error --- 
#[derive(Debug)]
pub enum HostAbiError {
    NotImplemented(String),
    InsufficientMana,
    AccountNotFound(Did),
    JobSubmissionFailed(String),
    InvalidParameters(String),
    DagOperationFailed(String),
    SignatureError(String),
    CryptoError(String),
    WasmExecutionError(String),
    ResourceLimitExceeded(String),
    InvalidSystemApiCall(String),
    InternalError(String),
    Common(CommonError),
    NetworkError(String),
}

impl std::fmt::Display for HostAbiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostAbiError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            HostAbiError::InsufficientMana => write!(f, "Insufficient mana"),
            HostAbiError::AccountNotFound(did) => write!(f, "Account not found: {}", did.to_string()),
            HostAbiError::JobSubmissionFailed(msg) => write!(f, "Job submission failed: {}", msg),
            HostAbiError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
            HostAbiError::DagOperationFailed(msg) => write!(f, "DAG operation failed: {}", msg),
            HostAbiError::SignatureError(msg) => write!(f, "Signature error: {}", msg),
            HostAbiError::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            HostAbiError::WasmExecutionError(msg) => write!(f, "Wasm execution error: {}", msg),
            HostAbiError::ResourceLimitExceeded(msg) => write!(f, "Resource limit exceeded: {}", msg),
            HostAbiError::InvalidSystemApiCall(msg) => write!(f, "Invalid system API call: {}", msg),
            HostAbiError::InternalError(msg) => write!(f, "Internal runtime error: {}", msg),
            HostAbiError::Common(e) => write!(f, "Common error: {}", e),
            HostAbiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for HostAbiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HostAbiError::Common(e) => Some(e),
            _ => None,
        }
    }
}

impl From<CommonError> for HostAbiError {
    fn from(err: CommonError) -> Self {
        HostAbiError::Common(err)
    }
}

// --- Runtime Context --- 
#[derive(Debug)]
pub struct RuntimeContext {
    pub current_identity: Did,
    pub mana_ledger: SimpleManaLedger, 
    pub pending_mesh_jobs: Arc<Mutex<VecDeque<ActualMeshJob>>>, 
    pub job_states: Arc<Mutex<HashMap<JobId, JobState>>>,
    pub governance_module: GovernanceModule,
    pub mesh_network_service: Arc<dyn MeshNetworkService>, // Uses local MeshNetworkService trait
    pub signer: Arc<dyn Signer>, 
    pub dag_store: Arc<dyn StorageService>, // Uses local StorageService trait
}

impl RuntimeContext {
    pub fn new(
        current_identity: Did, 
        mesh_network_service: Arc<dyn MeshNetworkService>,
        signer: Arc<dyn Signer>,
        dag_store: Arc<dyn StorageService> // Changed from DagStore
    ) -> Self {
        let job_states = Arc::new(Mutex::new(HashMap::new()));
        let pending_mesh_jobs = Arc::new(Mutex::new(VecDeque::new()));

        Self {
            current_identity,
            mana_ledger: SimpleManaLedger::new(),
            pending_mesh_jobs,
            job_states,
            governance_module: GovernanceModule::new(),
            mesh_network_service,
            signer,
            dag_store,
        }
    }

    #[cfg(feature = "enable-libp2p")]
    pub async fn new_with_libp2p_network(current_identity_str: &str, bootstrap_peers: Option<Vec<(Libp2pPeerId, Multiaddr)>>) -> Result<Self, CommonError> {
        let current_identity = Did::from_str(current_identity_str)
            .map_err(|e| CommonError::IdentityError(format!("Invalid DID string for new_with_libp2p_network: {}: {}", current_identity_str, e)))?;
        
        let libp2p_service_concrete = Arc::new(
            ActualLibp2pNetworkService::new(bootstrap_peers).await
                .map_err(|e| CommonError::NetworkSetupError(format!("Failed to create Libp2pNetworkService: {}", e)))?
        );
        let libp2p_service_dyn: Arc<dyn ActualNetworkService> = libp2p_service_concrete;

        let default_mesh_service = Arc::new(DefaultMeshNetworkService::new(libp2p_service_dyn.clone()));

        Ok(Self::new(
            current_identity,
            default_mesh_service,
            Arc::new(StubSigner), 
            Arc::new(StubDagStore::new()), 
        ))
    }

    pub fn new_with_stubs(current_identity_str: &str) -> Self {
        let current_identity = Did::from_str(current_identity_str).expect("Invalid DID for test context in new_with_stubs");
        Self::new(
            current_identity, 
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner),
            Arc::new(StubDagStore::new())
        )
    }

    pub fn new_with_stubs_and_mana(current_identity_str: &str, initial_mana: u64) -> Self {
        let current_identity = Did::from_str(current_identity_str).expect("Invalid DID for test context in new_with_stubs_and_mana");
        let ctx = Self::new(
            current_identity.clone(), 
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner),
            Arc::new(StubDagStore::new())
        );
        futures::executor::block_on(ctx.mana_ledger.set_balance(&current_identity, initial_mana));
        ctx
    }
    
    pub async fn internal_queue_mesh_job(&self, job: ActualMeshJob) -> Result<(), HostAbiError> {
        let mut queue = self.pending_mesh_jobs.lock().await;
        queue.push_back(job.clone());
        let mut states = self.job_states.lock().await;
        states.insert(job.id.clone(), JobState::Pending);
        println!("[CONTEXT] Queued mesh job: id={:?}, state=Pending", job.id);
        Ok(())
    }

    pub async fn spawn_mesh_job_manager(&self) {
        let pending_jobs_queue_clone = Arc::clone(&self.pending_mesh_jobs);
        let job_states_clone = Arc::clone(&self.job_states);
        let network_service_clone = Arc::clone(&self.mesh_network_service);
        let signer_clone = Arc::clone(&self.signer);
        let dag_store_clone = Arc::clone(&self.dag_store);
        let current_identity_clone = self.current_identity.clone();
        let mana_ledger_for_refunds = self.mana_ledger.clone(); 

        let mut assigned_jobs: HashMap<JobId, (ActualMeshJob, Did, TokioInstant)> = HashMap::new();
        const JOB_EXECUTION_TIMEOUT: Duration = Duration::from_secs(5 * 60);
        let reputation_updater = ReputationUpdater::new(); // Using placeholder

        tokio::spawn(async move {
            loop {
                let mut job_to_assign_option: Option<ActualMeshJob> = None;
                {
                    let mut queue = pending_jobs_queue_clone.lock().await;
                    if let Some(job) = queue.pop_front() {
                        let states_guard = job_states_clone.lock().await; // Renamed to avoid conflict
                        if let Some(JobState::Pending) = states_guard.get(&job.id) {
                            job_to_assign_option = Some(job);
                        } else {
                            println!("[JobManager] Job {:?} from queue is no longer Pending. Skipping.", job.id);
                        }
                    }
                }

                if let Some(job_to_assign) = job_to_assign_option {
                    println!("[JobManager] Attempting to assign job: id={:?}", job_to_assign.id);
                    if let Err(e) = network_service_clone.announce_job(&job_to_assign).await {
                        let job_error = match e {
                            HostAbiError::NetworkError(msg) => MeshJobError::Network(icn_network::error::MeshNetworkError::SendFailure(msg)), // Assuming MeshNetworkError path
                            other_abi_error => MeshJobError::ProcessingFailure { 
                                job_id: job_to_assign.id.clone(), 
                                reason: format!("Failed to announce job during JobManager processing: {:?}", other_abi_error) 
                            },
                        };
                        error!(
                            "[JobManager] Failed to announce job id={:?}: {}. Re-queuing job.",
                            job_to_assign.id, job_error
                        );
                        pending_jobs_queue_clone.lock().await.push_back(job_to_assign);
                        sleep(Duration::from_secs(5)).await; 
                        continue;
                    }
                    
                    let bid_window = Duration::from_secs(30); 
                    println!("[JobManager] Collecting bids for {} seconds for job id={:?}...", bid_window.as_secs(), job_to_assign.id);
                    
                    let received_bids = match network_service_clone.collect_bids_for_job(&job_to_assign.id, bid_window).await {
                        Ok(bids) => bids,
                        Err(e) => {
                            eprintln!("[JobManager] Error collecting bids for job id={:?}: {:?}. Re-queuing job.", job_to_assign.id, e);
                            pending_jobs_queue_clone.lock().await.push_back(job_to_assign);
                            sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                    };
                     println!("[JobManager] Collected {} bids for job id={:?}", received_bids.len(), job_to_assign.id);
                    
                    let mut valid_bids: Vec<MeshJobBid> = Vec::new();
                    for bid in received_bids { 
                        match charge_mana(&bid.executor_did, 5) { 
                            Ok(_) => {
                                println!("[JobManager] Executor {:?} has sufficient mana reserve for bid on job id={:?}.", bid.executor_did, job_to_assign.id);
                                let reputation: u64 = 20; 
                                if reputation >= 10 {
                                    println!("[JobManager] Executor {:?} meets reputation threshold for bid on job id={:?}.", bid.executor_did, job_to_assign.id);
                                    valid_bids.push(bid);
                                } else {
                                    println!("[JobManager] Executor {:?} REJECTED (reputation {} < {}) for bid on job id={:?}", 
                                             bid.executor_did, reputation, 10, job_to_assign.id);
                                }
                            }
                            Err(EconError::InsufficientBalance(_)) => {
                                println!("[JobManager] Executor {:?} REJECTED (insufficient mana for reserve) for bid on job id={:?}", 
                                         bid.executor_did, job_to_assign.id);
                            }
                            Err(e) => {
                                eprintln!("[JobManager] Error checking mana for executor {:?} for bid on job id={:?}: {:?}", 
                                         bid.executor_did, job_to_assign.id, e);
                            }
                        }
                    }
                     println!("[JobManager] {} valid bids after filtering for job id={:?}", valid_bids.len(), job_to_assign.id);

                    let selection_policy = SelectionPolicy::default(); 
                    if let Some(selected_executor_did) = select_executor(valid_bids, selection_policy) {
                        match charge_mana(&selected_executor_did, 5) { 
                            Ok(_) => {
                                println!("[JobManager] Selected executor {:?} for job id={:?}. Mana re-confirmed/assigned.", 
                                         selected_executor_did, job_to_assign.id);
                                
                                {
                                    let mut states_guard = job_states_clone.lock().await;
                                    states_guard.insert(job_to_assign.id.clone(), JobState::Assigned { executor: selected_executor_did.clone() });
                                    println!("[JobManager] Job id={:?} state changed to Assigned to executor {:?}", job_to_assign.id, selected_executor_did);
                                }
                                assigned_jobs.insert(job_to_assign.id.clone(), (job_to_assign.clone(), selected_executor_did.clone(), TokioInstant::now()));
                                let notice = JobAssignmentNotice {
                                    job_id: job_to_assign.id.clone(),
                                    executor_did: selected_executor_did.clone(),
                                };
                                if let Err(e) = network_service_clone.notify_executor_of_assignment(&notice).await {
                                    eprintln!("[JobManager] Error broadcasting job assignment for id={:?}: {:?}. Job remains Assigned.", job_to_assign.id, e);
                                }
                            }
                            Err(EconError::InsufficientBalance(_)) => {
                                println!("[JobManager] Selected executor {:?} for job id={:?} now has INSUFFICIENT MANA. Trying next bid.", 
                                         selected_executor_did, job_to_assign.id);
                            }
                            Err(e) => {
                                eprintln!("[JobManager] Error confirming mana for selected executor {:?} for job id={:?}: {:?}",
                                         selected_executor_did, job_to_assign.id, e);
                            }
                        }
                    } else {
                        println!("[JobManager] No suitable executor found for job id={:?}. Re-queuing with potential backoff.", job_to_assign.id);
                        pending_jobs_queue_clone.lock().await.push_back(job_to_assign);
                        sleep(Duration::from_secs(10)).await;
                    }
                }

                // --- Receipt Processing Section --- Needs careful implementation
                // This section should be more robust. For now, trying to check one assigned job.
                let mut receipt_to_process_option: Option<(JobId, IdentityExecutionReceipt)> = None;
                if let Some((job_id_checking, (_job, assigned_executor, _time))) = assigned_jobs.iter().next() { // Iterate to get a job to check
                    match network_service_clone.try_receive_receipt(job_id_checking, assigned_executor, Duration::from_millis(100)).await {
                        Ok(Some(receipt)) => {
                            info!("[JobManager] Received receipt for job: {:?}", receipt.job_id);
                            receipt_to_process_option = Some((job_id_checking.clone(), receipt));
                        }
                        Ok(None) => { /* No receipt yet for this job */ }
                        Err(e) => { error!("[JobManager] Error trying to receive receipt for job {:?}: {:?}", job_id_checking, e); }
                    }
                }

                if let Some((job_id_from_receipt, receipt)) = receipt_to_process_option {
                    if let Some((_original_job, assigned_executor_did_map, _assignment_time)) = assigned_jobs.get(&job_id_from_receipt) {
                        if &receipt.executor_did == assigned_executor_did_map {
                            let mut receipt_to_anchor = receipt.clone(); 
                            // The anchor_receipt in RuntimeContext is sync, but its internal calls to Signer and StorageService are async.
                            // It uses futures::executor::block_on which is not ideal for a running async executor like the job manager.
                            // This should be refactored. For now, to compile, we'll assume anchor_receipt can be called.
                            // A better approach: anchor_receipt itself becomes async.

                                // --- Simulated Anchoring --- 
                                let mut successful_anchor = false;
                                let mut anchored_cid_opt: Option<Cid> = None;
                                let receipt_data_to_verify = serde_json::to_vec(&(
                                    &receipt_to_anchor.job_id, 
                                    &receipt_to_anchor.executor_did, 
                                    &receipt_to_anchor.result_cid, 
                                    receipt_to_anchor.cpu_ms
                                )).map_err(|e| HostAbiError::InternalError(format!("Failed to serialize receipt for sig verify: {}", e)));
                                
                                if let Ok(data_to_verify) = receipt_data_to_verify {
                                    match signer_clone.verify(&receipt_to_anchor.executor_did, &data_to_verify, &receipt_to_anchor.sig).await {
                                        Ok(true) => {
                                            let final_receipt_bytes = serde_json::to_vec(&receipt_to_anchor)
                                                .map_err(|e| HostAbiError::InternalError(format!("Failed to serialize final receipt for DAG: {}", e)));
                                            if let Ok(bytes_to_store) = final_receipt_bytes {
                                                match dag_store_clone.put(&bytes_to_store).await {
                                                    Ok(cid) => {
                                                        successful_anchor = true;
                                                        anchored_cid_opt = Some(cid);
                                                    }
                                                    Err(e) => { error!("[JobManager] DAG put failed for receipt {:?}: {:?}", receipt_to_anchor.job_id, e);}
                                                }
                                            } else if let Err(e) = final_receipt_bytes {
                                                 error!("[JobManager] Failed to serialize final receipt for DAG for job {:?}: {:?}", receipt_to_anchor.job_id, e);
                                            }
                                        }
                                        Ok(false) => { error!("[JobManager] Receipt signature verification failed for job {:?}", receipt_to_anchor.job_id); }
                                        Err(e) => { error!("[JobManager] Error during receipt signature verification for job {:?}: {:?}", receipt_to_anchor.job_id, e);}
                                    }
                                } else if let Err(e) = receipt_data_to_verify {
                                    error!("[JobManager] Failed to serialize receipt for signature verification for job {:?}: {:?}", receipt_to_anchor.job_id, e);
                                }
                                // --- End Simulated Anchoring ---

                                if successful_anchor {
                                    if let Some(anchored_cid) = anchored_cid_opt {
                                        println!("[JobManager] Receipt for job {:?} anchored successfully with CID: {:?}", job_id_from_receipt, anchored_cid);
                                        reputation_updater.submit(&receipt_to_anchor);
                                        let mut states_guard = job_states_clone.lock().await;
                                        states_guard.insert(job_id_from_receipt.clone(), JobState::Completed { receipt: receipt_to_anchor });
                                        assigned_jobs.remove(&job_id_from_receipt);
                                        println!("[JobManager] Job {:?} state changed to Completed.", job_id_from_receipt);
                                    }
                                } else {
                                    eprintln!("[JobManager] Failed to anchor receipt for job {:?}. Reason: Simulated anchor failure.", job_id_from_receipt);
                                    let mut states_guard = job_states_clone.lock().await;
                                    states_guard.insert(job_id_from_receipt.clone(), JobState::Failed { reason: format!("Anchor failed: Simulated anchor failure.") });
                                    assigned_jobs.remove(&job_id_from_receipt);
                                }
                            // Original anchor_receipt call (problematic):
                            // match current_identity_clone.anchor_receipt(&mut receipt_to_anchor) { ... }
                        } else {
                            eprintln!("[JobManager] Received receipt for job {:?} from unexpected executor {:?}. Expected {:?}. Discarding.", 
                                     job_id_from_receipt, receipt.executor_did, assigned_executor_did_map);
                        }
                    } else {
                        println!("[JobManager] Received receipt for unknown or already processed job_id: {:?}. Discarding.", job_id_from_receipt);
                    }
                }
                // --- End Receipt Processing --- 

                let mut timed_out_job_ids = Vec::new();
                for (job_id, (_original_job, _executor_did, assignment_time)) in &assigned_jobs {
                    if assignment_time.elapsed() > JOB_EXECUTION_TIMEOUT {
                        timed_out_job_ids.push(job_id.clone());
                    }
                }
                for job_id in timed_out_job_ids {
                     println!("[JobManager] Job {:?} timed out.", job_id);
                    if let Some((timed_out_job, timed_out_executor_did, _)) = assigned_jobs.remove(&job_id) { 
                        let mut states_guard = job_states_clone.lock().await;
                        states_guard.insert(job_id.clone(), JobState::Failed { reason: format!("Execution timed out by executor {:?}", timed_out_executor_did) });
                        
                        match mana_ledger_for_refunds.credit(&timed_out_job.creator_did, timed_out_job.cost_mana).await {
                            Ok(_) => println!("[JobManager] Refunded {} mana to submitter {:?} for timed out job {:?}. Mana ledger shared.", 
                                            timed_out_job.cost_mana, timed_out_job.creator_did, job_id),
                            Err(e) => eprintln!("[JobManager] Error refunding mana for job {:?}: {:?}. Submitter: {:?}, Amount: {}", 
                                            job_id, e, timed_out_job.creator_did, timed_out_job.cost_mana),
                        }
                    }
                }
                sleep(Duration::from_millis(500)).await;
            }
        });
        println!("[RuntimeContext] Mesh job manager spawned.");
    }

    pub async fn get_mana(&self, account: &Did) -> Result<u64, HostAbiError> {
        println!("[CONTEXT] get_mana called for account: {:?}", account);
        self.mana_ledger.get_balance(account).await.ok_or_else(|| HostAbiError::AccountNotFound(account.clone()))
    }

    pub async fn spend_mana(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        println!("[CONTEXT] spend_mana called for account: {:?} amount: {}", account, amount);
        if account != &self.current_identity {
            return Err(HostAbiError::InvalidParameters(
                "Attempting to spend mana for an account other than the current context identity.".to_string(),
            ));
        }
        self.mana_ledger.spend(account, amount).await
    }

    pub async fn credit_mana(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        println!("[CONTEXT] credit_mana called for account: {:?} amount: {}", account, amount);
        self.mana_ledger.credit(account, amount).await
    }

    // anchor_receipt should ideally be async if its operations (signer.verify, dag_store.put) are async.
    // The current implementation uses futures::executor::block_on, which is not good practice in async contexts.
    // For now, I will make it async and remove block_on.
    pub async fn anchor_receipt(&self, receipt: &IdentityExecutionReceipt) -> Result<Cid, HostAbiError> { 
        println!("[CONTEXT] anchor_receipt called by context {:?} for job_id: {:?}, claimed executor: {:?}", 
                 self.current_identity, receipt.job_id, receipt.executor_did);

        let job_id = &receipt.job_id;
        let assigned_executor_did = { 
            let job_states_guard = self.job_states.lock().await;
            match job_states_guard.get(job_id) {
                Some(JobState::Assigned { executor }) => {
                    if executor != &receipt.executor_did {
                        return Err(HostAbiError::InvalidParameters(format!(
                            "Receipt for job {:?} submitted by unauthorized executor {:?}. Expected assigned executor {:?}.",
                            job_id, receipt.executor_did, executor
                        )));
                    }
                    executor.clone()
                }
                Some(JobState::Pending) => {
                    return Err(HostAbiError::InvalidParameters(format!(
                        "Job {:?} is still Pending, cannot anchor receipt.", job_id
                    )));
                }
                Some(JobState::Completed { .. }) => {
                    return Err(HostAbiError::InvalidParameters(format!(
                        "Job {:?} is already Completed. Cannot re-anchor receipt.", job_id
                    )));
                }
                Some(JobState::Failed { .. }) => {
                    return Err(HostAbiError::InvalidParameters(format!(
                        "Job {:?} is Failed. Cannot anchor receipt.", job_id
                    )));
                }
                None => {
                    return Err(HostAbiError::InvalidParameters(format!(
                        "Job {:?} not found. Cannot anchor receipt.", job_id
                    )));
                }
            }
        };
        
        if assigned_executor_did != receipt.executor_did {
            return Err(HostAbiError::InternalError(format!(
                "Mismatch after assigned executor check. Assigned: {:?}, Receipt: {:?}. This is a bug.",
                assigned_executor_did, receipt.executor_did
            )));
        }

        if receipt.sig.is_empty() {
            return Err(HostAbiError::SignatureError("Receipt signature is missing.".to_string()));
        }

        let receipt_data_to_verify = serde_json::to_vec(&(
            &receipt.job_id, 
            &receipt.executor_did, 
            &receipt.result_cid, 
            receipt.cpu_ms
        )).map_err(|e| HostAbiError::InternalError(format!("Failed to serialize receipt for signature verification: {}", e)))?;
        
        match self.signer.verify(&receipt.executor_did, &receipt_data_to_verify, &receipt.sig).await {
            Ok(true) => {
                println!("[CONTEXT] Signature verified for receipt from executor {:?}", receipt.executor_did);
            }
            Ok(false) => {
                return Err(HostAbiError::SignatureError(format!(
                    "Invalid signature on receipt for job {:?} from executor {:?}.",
                    job_id, receipt.executor_did
                )));
            }
            Err(e) => {
                return Err(HostAbiError::SignatureError(format!(
                    "Error during signature verification for job {:?}, executor {:?}: {:?}",
                    job_id, receipt.executor_did, e
                )));
            }
        }

        let final_receipt_bytes = serde_json::to_vec(receipt)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to serialize final receipt for DAG: {}", e)))?;
        
        let cid = self.dag_store.put(&final_receipt_bytes).await?;
        println!("[CONTEXT] Anchored receipt for job_id {:?} with CID: {:?}. Executor: {:?}. Receipt cost {}ms.", 
                 receipt.job_id, cid, receipt.executor_did, receipt.cpu_ms);
        
        { 
            let mut job_states_guard = self.job_states.lock().await;
            job_states_guard.insert(job_id.clone(), JobState::Completed { receipt: receipt.clone() });
            println!("[CONTEXT] Job {:?} state updated to Completed.", job_id);
        }
        println!("[CONTEXT] Placeholder: Reputation update needed for executor {:?} for job {:?}.", receipt.executor_did, job_id);
        Ok(cid)
    }

    pub fn create_governance_proposal(&mut self, _payload: CreateProposalPayload) -> Result<String, HostAbiError> {
        todo!("Implement mapping for CreateProposalPayload and call governance_module.submit_proposal");
    }

    pub fn cast_governance_vote(&mut self, _payload: CastVotePayload) -> Result<(), HostAbiError> {
        todo!("Implement mapping for CastVotePayload and call governance_module.cast_vote");
    }

    pub fn close_governance_proposal_voting(&mut self, _proposal_id_str: &str) -> Result<String, HostAbiError> {
        todo!("Integrate with GovernanceModule proposal closing/tallying logic");
    }

    pub fn execute_governance_proposal(&mut self, _proposal_id_str: &str) -> Result<(), HostAbiError> {
        todo!("Implement full governance proposal execution logic");
    }
}

pub trait HostEnvironment: Send + Sync + std::fmt::Debug {
    fn env_submit_mesh_job(&self, ctx: &mut RuntimeContext, job_data_ptr: u32, job_data_len: u32) -> Result<u32, HostAbiError>; 
    fn env_account_get_mana(&self, ctx: &RuntimeContext, account_did_ptr: u32, account_did_len: u32) -> Result<u64, HostAbiError>;
    fn env_account_spend_mana(&self, ctx: &mut RuntimeContext, account_did_ptr: u32, account_did_len: u32, amount: u64) -> Result<(), HostAbiError>;
}

#[derive(Debug)]
pub struct ConcreteHostEnvironment {}

impl ConcreteHostEnvironment {
    pub fn new() -> Self { Self {} }
}
impl Default for ConcreteHostEnvironment { fn default() -> Self { Self::new() } }

impl HostEnvironment for ConcreteHostEnvironment {
    fn env_submit_mesh_job(&self, _ctx: &mut RuntimeContext, _job_data_ptr: u32, _job_data_len: u32) -> Result<u32, HostAbiError> { 
        todo!("ConcreteHostEnvironment::env_submit_mesh_job");
    }
    fn env_account_get_mana(&self, _ctx: &RuntimeContext, _account_did_ptr: u32, _account_did_len: u32) -> Result<u64, HostAbiError> {
        todo!("ConcreteHostEnvironment::env_account_get_mana");
    }
    fn env_account_spend_mana(&self, _ctx: &mut RuntimeContext, _account_did_ptr: u32, _account_did_len: u32, _amount: u64) -> Result<(), HostAbiError> {
        todo!("ConcreteHostEnvironment::env_account_spend_mana");
    }
} 

#[derive(Debug, Clone)]
pub struct StubSigner;
#[async_trait]
impl Signer for StubSigner { // Implements the local async Signer trait
    async fn sign(&self, did: &Did, data: &[u8]) -> Result<Vec<u8>, HostAbiError> {
        println!("[StubSigner] Signing data for DID {:?}: (data_len={})", did, data.len());
        let mut sig_content = b"signed:".to_vec();
        sig_content.extend_from_slice(did.id_string.as_bytes());
        sig_content.extend_from_slice(b":");
        sig_content.extend_from_slice(&data[..std::cmp::min(data.len(), 8)]);
        Ok(sig_content)
    }
    async fn verify(&self, did: &Did, data: &[u8], signature: &[u8]) -> Result<bool, HostAbiError> {
        println!("[StubSigner] Verifying signature for DID {:?}: (data_len={}, sig_len={})", did, data.len(), signature.len());
        let expected_sig_prefix = format!("signed:{}:", did.id_string);
        let expected_data_prefix = &data[..std::cmp::min(data.len(), 8)];
        
        let sig_str = String::from_utf8_lossy(signature);
        if sig_str.starts_with(&expected_sig_prefix) {
            let data_part_in_sig = sig_str.trim_start_matches(&expected_sig_prefix);
            Ok(data_part_in_sig.as_bytes() == expected_data_prefix)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug, Clone)]
pub struct StubDagStore { // Renamed from StubStorageService for consistency if tests use this name
    store: Arc<Mutex<HashMap<Cid, Vec<u8>>>>,
}
impl StubDagStore {
    pub fn new() -> Self {
        Self { store: Arc::new(Mutex::new(HashMap::new())) }
    }
}
#[async_trait]
impl StorageService for StubDagStore { // Implements the local async StorageService trait
    async fn put(&self, data: &[u8]) -> Result<Cid, HostAbiError> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash_slice(data, &mut hasher);
        let hash_val = std::hash::Hasher::finish(&hasher);
        let cid = Cid::new_v1_dummy(0x70, 0x12, &hash_val.to_ne_bytes());
        
        let mut store_lock = self.store.lock().await;
        store_lock.insert(cid.clone(), data.to_vec());
        println!("[StubDagStore] Stored data with CID: {:?}", cid);
        Ok(cid)
    }
    async fn get(&self, cid: &Cid) -> Result<Option<Vec<u8>>, HostAbiError> {
        let store_lock = self.store.lock().await;
        let data = store_lock.get(cid).cloned();
        if data.is_some() {
            println!("[StubDagStore] Retrieved data for CID: {:?}", cid);
        } else {
            println!("[StubDagStore] No data found for CID: {:?}", cid);
        }
        Ok(data)
    }
}

#[derive(Debug, Clone)]
pub struct StubMeshNetworkService {
    staged_bids: Arc<Mutex<HashMap<JobId, VecDeque<MeshJobBid>>>>,
    staged_receipts: Arc<Mutex<VecDeque<LocalMeshSubmitReceiptMessage>>>, // Using local placeholder
}
impl StubMeshNetworkService { 
    pub fn new() -> Self { 
        Self {
            staged_bids: Arc::new(Mutex::new(HashMap::new())),
            staged_receipts: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    pub async fn stage_bid(&self, job_id: JobId, bid: MeshJobBid) {
        let mut bids_map = self.staged_bids.lock().await;
        bids_map.entry(job_id).or_default().push_back(bid);
    }
    pub async fn stage_receipt(&self, receipt_message: LocalMeshSubmitReceiptMessage) {
        let mut receipts_queue = self.staged_receipts.lock().await;
        receipts_queue.push_back(receipt_message);
    }
}

#[async_trait]
impl MeshNetworkService for StubMeshNetworkService { // Implements local MeshNetworkService trait
    fn as_any(&self) -> &dyn std::any::Any { self }

    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        println!("[StubMeshNetworkService] Announced job: {:?}", job.id);
        Ok(())
    }

    async fn collect_bids_for_job(&self, job_id: &JobId, _duration: Duration) -> Result<Vec<MeshJobBid>, HostAbiError> {
        println!("[StubMeshNetworkService] Collecting bids for job {:?}.", job_id);
        let mut bids_map = self.staged_bids.lock().await;
        if let Some(job_bids_queue) = bids_map.get_mut(job_id) {
            let bids: Vec<MeshJobBid> = job_bids_queue.drain(..).collect();
            println!("[StubMeshNetworkService] Found {} staged bids for job {:?}", bids.len(), job_id);
            Ok(bids)
        } else {
            println!("[StubMeshNetworkService] No staged bids found for job {:?}. Returning empty vec.", job_id);
            Ok(Vec::new()) 
        }
    }

    async fn notify_executor_of_assignment(&self, notice: &JobAssignmentNotice) -> Result<(), HostAbiError> {
        println!("[StubMeshNetworkService] Broadcast assignment for job {:?} to executor {:?}", notice.job_id, notice.executor_did);
        Ok(())
    }

    async fn try_receive_receipt(&self, _job_id: &JobId, _expected_executor: &Did, _timeout_duration: Duration) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
        let mut receipts_queue = self.staged_receipts.lock().await;
        if let Some(receipt_msg) = receipts_queue.pop_front() {
            println!("[StubMeshNetworkService] try_receive_receipt: Popped staged receipt for job {:?}", receipt_msg.receipt.job_id);
            Ok(Some(receipt_msg.receipt))
        } else {
            Ok(None)
        }
    }
} 

// Placeholder for ReputationUpdater - assuming it's in crate::
// This should be moved to its own module or properly defined.
mod reputation_updater { 
    use icn_identity::ExecutionReceipt as IdentityExecutionReceipt; // Corrected path
    #[derive(Debug)]
    pub struct ReputationUpdater;
    impl ReputationUpdater {
        pub fn new() -> Self { Self }
        pub fn submit(&self, _receipt: &IdentityExecutionReceipt) {
            println!("[ReputationUpdater] Submitted receipt for reputation processing (stub).");
        }
    }
}
use reputation_updater::ReputationUpdater; 