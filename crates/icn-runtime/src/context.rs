//! Defines the `RuntimeContext`, `HostEnvironment`, and related types for the ICN runtime.

use icn_common::{Did, Cid, CommonError}; // Removed JobId as CommonJobId
use std::collections::{HashMap, VecDeque}; // Added HashMap for ManaLedger
use std::str::FromStr; // For Did::from_str in new_with_dummy_mana
use std::sync::atomic::AtomicU32; // Modified import for Ordering
use icn_governance::GovernanceModule; // Assuming this can be imported
pub use icn_mesh::JobState; // Import from icn-mesh
use icn_mesh::{ActualMeshJob, MeshJobBid, MeshJobAnnounce, select_executor, SelectionPolicy, JobId, JobAssignmentNotice, SubmitReceiptMessage as MeshSubmitReceiptMessage}; // Import from icn-mesh
use icn_economics::{charge_mana, EconError}; // For mana charging
use icn_identity::ExecutionReceipt as IdentityExecutionReceipt; // Import and alias ExecutionReceipt
use tokio::sync::Mutex; // For channel-based communication if needed and Mutex for shared state
use tokio::time::{sleep, Duration, Instant as TokioInstant}; // For timeouts and Tokio Instant
use std::sync::Arc; // For shared state
use async_trait::async_trait; // For async traits
use downcast_rs::{impl_downcast, DowncastSync};

// Import network service from icn-network
use icn_network::libp2p_service::{Libp2pNetworkService, Libp2pPeerId, Multiaddr}; // Corrected path & added types
use icn_network::{NetworkMessage, NetworkService as GenericNetworkService, PeerId as GenericPeerId};

// Counter for generating unique (within this runtime instance) job IDs for stubs
pub static NEXT_JOB_ID: AtomicU32 = AtomicU32::new(1);

// --- Placeholder Types ---
// TODO: Define these types properly, likely in a shared crate (e.g., icn-governance or icn-common)
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

// Fully commented out old placeholder structs
// /// Placeholder for a job specification submitted to the mesh.
// #[derive(Debug, Clone)]
// pub struct MeshJob { ... }
// /// Placeholder for a Job ID returned by the mesh network.
// #[derive(Debug, Clone, PartialEq, Eq, Hash)] 
// pub struct OldJobId(pub String);
// /// Represents a receipt of execution that can be signed and anchored.
// #[derive(Debug, Clone)] 
// pub struct OldExecutionReceipt { ... }


// --- Core Service Traits ---

/// Trait for a service that can sign data on behalf of a DID.
pub trait Signer: Send + Sync + std::fmt::Debug {
    /// Signs the given data (e.g., a receipt hash) using the key associated with the provided DID.
    fn sign(&self, did: &Did, data: &[u8]) -> Result<Vec<u8>, HostAbiError>;
    /// Verifies a signature against the given data and DID.
    fn verify(&self, did: &Did, data: &[u8], signature: &[u8]) -> Result<bool, HostAbiError>;
}

/// Trait for a service that can store and retrieve data in a content-addressable DAG.
pub trait DagStore: Send + Sync + std::fmt::Debug + DowncastSync {
    /// Stores a block of data and returns its CID.
    fn put(&self, data: &[u8]) -> Result<Cid, HostAbiError>;
    /// Retrieves a block of data by its CID.
    fn get(&self, cid: &Cid) -> Result<Option<Vec<u8>>, HostAbiError>;
}
impl_downcast!(sync DagStore);

/// Trait for a service that can broadcast and receive mesh-specific messages.
#[async_trait]
pub trait MeshNetworkService: Send + Sync + std::fmt::Debug + DowncastSync {
    async fn announce_job(&self, announcement: MeshJobAnnounce) -> Result<(), HostAbiError>;
    async fn collect_bids_for_job(&self, job_id: JobId, duration: Duration) -> Result<Vec<MeshJobBid>, HostAbiError>;
    /// Broadcasts the job assignment to the selected executor (and potentially other listeners).
    async fn broadcast_assignment(&self, notice: JobAssignmentNotice) -> Result<(), HostAbiError>;
    /// Attempts to receive a submitted execution receipt (non-blocking).
    async fn try_receive_receipt(&self) -> Result<Option<MeshSubmitReceiptMessage>, HostAbiError>;
}
impl_downcast!(sync MeshNetworkService);

// --- DefaultMeshNetworkService Implementation ---

#[derive(Debug)]
pub struct DefaultMeshNetworkService {
    network_service: Arc<Libp2pNetworkService>,
    // Receiver for all messages. This needs careful handling for multiple concurrent calls
    // to collect_bids_for_job or try_receive_receipt.
    // For now, we might clone the receiver or use a broadcast channel if Libp2pNetworkService supports it.
    // Based on current Libp2pNetworkService, each call to subscribe gives a new MPSC receiver.
    // This means collect_bids_for_job and try_receive_receipt will need to manage their own subscriptions.
}

impl DefaultMeshNetworkService {
    pub fn new(network_service: Arc<Libp2pNetworkService>) -> Self {
        Self { network_service }
    }
}

#[async_trait]
impl MeshNetworkService for DefaultMeshNetworkService {
    async fn announce_job(&self, announcement: MeshJobAnnounce) -> Result<(), HostAbiError> {
        println!("[DefaultMeshNetworkService] Announcing job: {:?}", announcement.job_id);
        let job_message = NetworkMessage::MeshJobAnnouncement(
            icn_mesh::ActualMeshJob { // Assuming MeshJobAnnounce can be converted or we construct ActualMeshJob
                id: announcement.job_id.clone(),
                manifest_cid: announcement.manifest_cid.clone(),
                spec: icn_mesh::JobSpec::default(), // JobSpec might need to be part of MeshJobAnnounce
                creator_did: announcement.creator_did.clone(),
                cost_mana: announcement.cost_mana,
            }
        );
        self.network_service.broadcast_message(job_message).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to announce job: {}", e)))
    }

    async fn collect_bids_for_job(&self, job_id: JobId, duration: Duration) -> Result<Vec<MeshJobBid>, HostAbiError> {
        println!("[DefaultMeshNetworkService] Collecting bids for job {:?} for {:?}", job_id, duration);
        let mut bids = Vec::new();
        let mut receiver = self.network_service.subscribe()
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to subscribe for bids: {}", e)))?;
        
        let end_time = TokioInstant::now() + duration;

        loop {
            match tokio::time::timeout_at(end_time, receiver.recv()).await {
                Ok(Some(Some(message))) => { // Outer Some for timeout, inner Some for channel message
                    if let NetworkMessage::BidSubmission(bid) = message {
                        if bid.job_id == job_id {
                            println!("[DefaultMeshNetworkService] Received bid for job {:?}: {:?}", job_id, bid);
                            bids.push(bid);
                        }
                    }
                }
                Ok(Some(None)) => { // Channel closed
                    eprintln!("[DefaultMeshNetworkService] Bid collection channel closed for job {:?}", job_id);
                    break;
                }
                Ok(None) => { // Should not happen with mpsc receiver.recv()
                    break;
                }
                Err(_) => { // Timeout
                    println!("[DefaultMeshNetworkService] Bid collection timed out for job {:?}", job_id);
                    break;
                }
            }
        }
        Ok(bids)
    }

    async fn broadcast_assignment(&self, notice: JobAssignmentNotice) -> Result<(), HostAbiError> {
        println!("[DefaultMeshNetworkService] Broadcasting assignment for job {:?}", notice.job_id);
        let assignment_message = NetworkMessage::JobAssignmentNotification(notice.job_id.clone(), notice.executor_did.clone());
        self.network_service.broadcast_message(assignment_message).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to broadcast assignment: {}", e)))
    }

    async fn try_receive_receipt(&self) -> Result<Option<MeshSubmitReceiptMessage>, HostAbiError> {
        println!("[DefaultMeshNetworkService] Trying to receive receipt (not fully implemented, needs persistent subscription or shared receiver)");
        // This is a placeholder. A real implementation would need to manage a persistent subscription
        // or have a shared receiver from Libp2pNetworkService.
        // For now, it subscribes, tries to receive one message, and that's it. This is not robust.
        let mut receiver = self.network_service.subscribe()
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to subscribe for receipts: {}", e)))?;

        match receiver.try_recv() {
            Ok(NetworkMessage::SubmitReceipt(receipt)) => {
                println!("[DefaultMeshNetworkService] Received receipt: {:?}", receipt.job_id);
                // The MeshSubmitReceiptMessage struct from icn-mesh might be different from icn_identity::ExecutionReceipt
                // We need to ensure types match or convert. For now, assume direct usage is okay if types are compatible.
                // Let's assume MeshSubmitReceiptMessage is a wrapper around ExecutionReceipt.
                // The type from network is icn_identity::ExecutionReceipt
                // The type expected by MeshNetworkService trait is icn_mesh::SubmitReceiptMessage
                // We need to find definition of icn_mesh::SubmitReceiptMessage
                // From earlier search:
                // pub struct SubmitReceiptMessage { pub receipt: icn_identity::ExecutionReceipt }
                Ok(Some(MeshSubmitReceiptMessage { receipt }))
            }
            Ok(_) => Ok(None), // Other message types
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                eprintln!("[DefaultMeshNetworkService] Receipt collection channel disconnected.");
                Err(HostAbiError::NetworkError("Receipt channel disconnected".to_string()))
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
    NetworkError(String), // Added for network-specific errors from DefaultMeshNetworkService
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

// --- Mana Ledger (Simple In-Memory Version) ---
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

// --- Runtime Context --- 
#[derive(Debug)]
pub struct RuntimeContext {
    pub current_identity: Did,
    pub mana_ledger: SimpleManaLedger, 
    pub pending_mesh_jobs: Arc<Mutex<VecDeque<ActualMeshJob>>>, 
    pub job_states: Arc<Mutex<HashMap<JobId, JobState>>>,
    pub governance_module: GovernanceModule,
    pub mesh_network_service: Arc<dyn MeshNetworkService>,
    pub signer: Arc<dyn Signer>, 
    pub dag_store: Arc<dyn DagStore>,
}

impl RuntimeContext {
    pub fn new(
        current_identity: Did, 
        mesh_network_service: Arc<dyn MeshNetworkService>,
        signer: Arc<dyn Signer>,
        dag_store: Arc<dyn DagStore>
    ) -> Self {
        let job_states = Arc::new(Mutex::new(HashMap::new()));
        let pending_mesh_jobs = Arc::new(Mutex::new(VecDeque::new()));

        // Spawn the job manager task if it's not already running.
        // This needs to be handled carefully to avoid multiple spawns if RuntimeContext is created multiple times.
        // For now, we assume it's called once per logical "node" setup.
        // The actual spawning is now done by calling `spawn_mesh_job_manager` explicitly after context creation.

        Self {
            current_identity,
            mana_ledger: SimpleManaLedger::new(),
            pending_mesh_jobs,
            job_states,
            governance_module: GovernanceModule::new(), // TODO: Pass config or state
            mesh_network_service,
            signer,
            dag_store,
        }
    }

    pub async fn new_with_libp2p_network(current_identity_str: &str, bootstrap_peers: Option<Vec<(Libp2pPeerId, Multiaddr)>>) -> Result<Self, CommonError> {
        let current_identity = Did::from_str(current_identity_str)
            .map_err(|e| CommonError::IdentityError(format!("Invalid DID string: {}: {}", current_identity_str, e)))?;
        
        let libp2p_service = Arc::new(
            icn_network::libp2p_service::Libp2pNetworkService::new(bootstrap_peers).await
                .map_err(|e| CommonError::NetworkSetupError(format!("Failed to create Libp2pNetworkService: {}", e)))?
        );
        let default_mesh_service = Arc::new(DefaultMeshNetworkService::new(libp2p_service.clone()));

        Ok(Self::new(
            current_identity,
            default_mesh_service,
            Arc::new(StubSigner), // Using StubSigner for now
            Arc::new(StubDagStore::new()), // Using StubDagStore for now
        ))
    }

    pub fn new_with_stubs(current_identity_str: &str) -> Self {
        let current_identity = Did::from_str(current_identity_str).expect("Invalid DID for test context");
        Self::new(
            current_identity, 
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner),
            Arc::new(StubDagStore::new())
        )
    }

    pub fn new_with_stubs_and_mana(current_identity_str: &str, initial_mana: u64) -> Self {
        let current_identity = Did::from_str(current_identity_str).expect("Invalid DID for test context");
        let ctx = Self::new(
            current_identity.clone(), 
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner),
            Arc::new(StubDagStore::new())
        );
        // `set_balance` is now async, but this function is sync.
        // For test setup, block on the future.
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
        // Share the same Arc<Mutex<HashMap<...>>> by cloning the Arc, not the SimpleManaLedger struct itself.
        let mana_ledger_for_refunds = self.mana_ledger.clone(); 

        let mut assigned_jobs: HashMap<JobId, (ActualMeshJob, Did, TokioInstant)> = HashMap::new();
        const JOB_EXECUTION_TIMEOUT: Duration = Duration::from_secs(5 * 60);
        let reputation_updater = crate::ReputationUpdater::new();

        tokio::spawn(async move {
            loop {
                let mut job_to_assign_option: Option<ActualMeshJob> = None;
                {
                    let mut queue = pending_jobs_queue_clone.lock().await;
                    if let Some(job) = queue.pop_front() {
                        let states = job_states_clone.lock().await;
                        if let Some(JobState::Pending) = states.get(&job.id) {
                            job_to_assign_option = Some(job);
                        } else {
                            println!("[JobManager] Job {:?} from queue is no longer Pending. Skipping.", job.id);
                        }
                    }
                }

                if let Some(job_to_assign) = job_to_assign_option {
                    println!("[JobManager] Attempting to assign job: id={:?}", job_to_assign.id);
                    let announcement = MeshJobAnnounce {
                        job_id: job_to_assign.id.clone(),
                        manifest_cid: job_to_assign.manifest_cid.clone(),
                        creator_did: job_to_assign.creator_did.clone(),
                        cost_mana: job_to_assign.cost_mana,
                    };
                    if let Err(e) = network_service_clone.announce_job(announcement).await {
                        eprintln!("[JobManager] Error broadcasting job announcement for id={:?}: {:?}. Re-queuing job.", job_to_assign.id, e);
                        pending_jobs_queue_clone.lock().await.push_back(job_to_assign);
                        sleep(Duration::from_secs(5)).await; 
                        continue;
                    }
                    
                    let bid_window = Duration::from_secs(30); 
                    println!("[JobManager] Collecting bids for {} seconds for job id={:?}...", bid_window.as_secs(), job_to_assign.id);
                    
                    let received_bids = match network_service_clone.collect_bids_for_job(job_to_assign.id.clone(), bid_window).await {
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
                                    let mut states = job_states_clone.lock().await;
                                    states.insert(job_to_assign.id.clone(), JobState::Assigned { executor: selected_executor_did.clone() });
                                    println!("[JobManager] Job id={:?} state changed to Assigned to executor {:?}", job_to_assign.id, selected_executor_did);
                                }
                                assigned_jobs.insert(job_to_assign.id.clone(), (job_to_assign.clone(), selected_executor_did.clone(), TokioInstant::now()));
                                let notice = JobAssignmentNotice {
                                    job_id: job_to_assign.id.clone(),
                                    executor_did: selected_executor_did.clone(),
                                };
                                if let Err(e) = network_service_clone.broadcast_assignment(notice).await {
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

                if let Ok(Some(receipt_msg)) = network_service_clone.try_receive_receipt().await {
                    let job_id_from_receipt = receipt_msg.receipt.job_id.clone();
                    println!("[JobManager] Received receipt for job_id: {:?}", job_id_from_receipt);

                    if let Some((_original_job, assigned_executor_did, _assignment_time)) = assigned_jobs.get(&job_id_from_receipt) {
                        if receipt_msg.receipt.executor_did == *assigned_executor_did {
                            let mut receipt_to_anchor = receipt_msg.receipt.clone(); 
                            let temp_anchor_ctx = RuntimeContext::new(
                                current_identity_clone.clone(),
                                network_service_clone.clone(),
                                signer_clone.clone(),
                                dag_store_clone.clone()
                            );
                            match temp_anchor_ctx.anchor_receipt(&mut receipt_to_anchor) {
                                Ok(anchored_cid) => {
                                    println!("[JobManager] Receipt for job {:?} anchored successfully with CID: {:?}", job_id_from_receipt, anchored_cid);
                                    reputation_updater.submit(&receipt_to_anchor);
                                    let mut states = job_states_clone.lock().await;
                                    states.insert(job_id_from_receipt.clone(), JobState::Completed { receipt: receipt_to_anchor });
                                    assigned_jobs.remove(&job_id_from_receipt);
                                    println!("[JobManager] Job {:?} state changed to Completed.", job_id_from_receipt);
                                }
                                Err(e) => {
                                    eprintln!("[JobManager] Failed to anchor receipt for job {:?}: {:?}", job_id_from_receipt, e);
                                    let mut states = job_states_clone.lock().await;
                                    states.insert(job_id_from_receipt.clone(), JobState::Failed { reason: format!("Anchor failed: {:?}", e) });
                                    assigned_jobs.remove(&job_id_from_receipt);
                                }
                            }
                        } else {
                            eprintln!("[JobManager] Received receipt for job {:?} from unexpected executor {:?}. Expected {:?}. Discarding.", 
                                     job_id_from_receipt, receipt_msg.receipt.executor_did, assigned_executor_did);
                        }
                    } else {
                        println!("[JobManager] Received receipt for unknown or already processed job_id: {:?}. Discarding.", job_id_from_receipt);
                    }
                } else if let Err(e) = network_service_clone.try_receive_receipt().await {
                    eprintln!("[JobManager] Error calling try_receive_receipt: {:?}", e);
                }

                let mut timed_out_job_ids = Vec::new();
                for (job_id, (_original_job, _executor_did, assignment_time)) in &assigned_jobs {
                    if assignment_time.elapsed() > JOB_EXECUTION_TIMEOUT {
                        timed_out_job_ids.push(job_id.clone());
                    }
                }
                for job_id in timed_out_job_ids {
                    println!("[JobManager] Job {:?} timed out.", job_id);
                    if let Some((timed_out_job, timed_out_executor_did, _)) = assigned_jobs.remove(&job_id) { // Remove and get job details
                        let mut states = job_states_clone.lock().await;
                        states.insert(job_id.clone(), JobState::Failed { reason: format!("Execution timed out by executor {:?}", timed_out_executor_did) });
                        
                        // Refund the original submitter
                        // mana_ledger_for_refunds is now the same shared ledger instance
                        match mana_ledger_for_refunds.credit(&timed_out_job.creator_did, timed_out_job.cost_mana).await {
                            Ok(_) => println!("[JobManager] Refunded {} mana to submitter {:?} for timed out job {:?}. Mana ledger shared.", 
                                            timed_out_job.cost_mana, timed_out_job.creator_did, job_id),
                            Err(e) => eprintln!("[JobManager] Error refunding mana for job {:?}: {:?}. Submitter: {:?}, Amount: {}", 
                                            job_id, e, timed_out_job.creator_did, timed_out_job.cost_mana),
                        }
                        // TODO: Potentially penalize the executor who timed out (reputation system)
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

    pub fn anchor_receipt(&self, receipt: &IdentityExecutionReceipt) -> Result<Cid, HostAbiError> { 
        println!("[CONTEXT] anchor_receipt called by context {:?} for job_id: {:?}, claimed executor: {:?}", 
                 self.current_identity, receipt.job_id, receipt.executor_did);

        // 1. Fetch JobState and Validate Assigned Executor
        let job_id = &receipt.job_id;
        let assigned_executor_did = { // Scope for job_states lock
            let job_states_guard = futures::executor::block_on(self.job_states.lock());
            match job_states_guard.get(job_id) {
                Some(JobState::Assigned { executor }) => {
                    if executor != &receipt.executor_did {
                        return Err(HostAbiError::InvalidParameters(format!(
                            "Receipt for job {:?} submitted by unauthorized executor {:?}. Expected assigned executor {:?}.",
                            job_id, receipt.executor_did, executor
                        )));
                    }
                    executor.clone() // DID of the correctly assigned executor
                }
                Some(JobState::Pending) => {
                    return Err(HostAbiError::InvalidParameters(format!(
                        "Job {:?} is still Pending, cannot anchor receipt.", job_id
                    )));
                }
                Some(JobState::Completed { .. }) => {
                     // NOTE: Consider if re-anchoring a completed job's receipt is allowed or an error.
                     // For now, let's treat it as an error to prevent duplicate processing.
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
        
        // At this point, assigned_executor_did is confirmed to be receipt.executor_did.
        // Sanity check, though the logic above should ensure this.
        if assigned_executor_did != receipt.executor_did {
            // This should ideally be unreachable if the logic above is correct.
            return Err(HostAbiError::InternalError(format!(
                "Mismatch after assigned executor check. Assigned: {:?}, Receipt: {:?}. This is a bug.",
                assigned_executor_did, receipt.executor_did
            )));
        }

        // 2. Verify the signature on the receipt.
        // The receipt.sig should have been populated by the executor when they created/signed it.
        if receipt.sig.is_empty() {
            return Err(HostAbiError::SignatureError("Receipt signature is missing.".to_string()));
        }

        let receipt_data_to_verify = serde_json::to_vec(&(
            &receipt.job_id, 
            &receipt.executor_did, 
            &receipt.result_cid, 
            receipt.cpu_ms
        )).map_err(|e| HostAbiError::InternalError(format!("Failed to serialize receipt for signature verification: {}", e)))?;

        match self.signer.verify(&receipt.executor_did, &receipt_data_to_verify, &receipt.sig) {
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

        // 3. If valid, anchor the receipt to the DAG.
        let final_receipt_bytes = serde_json::to_vec(receipt)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to serialize final receipt for DAG: {}", e)))?;

        let cid = self.dag_store.put(&final_receipt_bytes)?;
        println!("[CONTEXT] Anchored receipt for job_id {:?} with CID: {:?}. Executor: {:?}. Receipt cost {}ms.", 
                 receipt.job_id, cid, receipt.executor_did, receipt.cpu_ms);
        
        // 4. Update the job state to Completed.
        // NOTE: This needs to be done carefully to avoid deadlocks if the job manager also holds this lock.
        // For now, let's assume this function is called in a context where it's safe to update.
        // In a more complex system, this state update might be signaled back to the job manager.
        { // Scope for job_states lock
            let mut job_states_guard = futures::executor::block_on(self.job_states.lock());
            job_states_guard.insert(job_id.clone(), JobState::Completed { receipt: receipt.clone() });
            println!("[CONTEXT] Job {:?} state updated to Completed.", job_id);
        }

        // TODO: Trigger reputation update for receipt.executor_did based on receipt.cpu_ms or other metrics.
        // For now, just a placeholder print.
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
impl Signer for StubSigner {
    fn sign(&self, did: &Did, data: &[u8]) -> Result<Vec<u8>, HostAbiError> {
        println!("[StubSigner] Signing data for DID {:?}: (data_len={})", did, data.len());
        let mut sig_content = b"signed:".to_vec();
        sig_content.extend_from_slice(did.id_string.as_bytes());
        sig_content.extend_from_slice(b":");
        sig_content.extend_from_slice(&data[..std::cmp::min(data.len(), 8)]);
        Ok(sig_content)
    }
    fn verify(&self, did: &Did, data: &[u8], signature: &[u8]) -> Result<bool, HostAbiError> {
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
pub struct StubDagStore {
    store: Arc<Mutex<HashMap<Cid, Vec<u8>>>>,
}
impl StubDagStore {
    pub fn new() -> Self {
        Self { store: Arc::new(Mutex::new(HashMap::new())) }
    }
}
impl DagStore for StubDagStore {
    fn put(&self, data: &[u8]) -> Result<Cid, HostAbiError> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash_slice(data, &mut hasher);
        let hash_val = std::hash::Hasher::finish(&hasher);
        let cid = Cid::new_v1_dummy(0x70, 0x12, &hash_val.to_ne_bytes());
        
        let mut store_lock = futures::executor::block_on(self.store.lock());
        store_lock.insert(cid.clone(), data.to_vec());
        println!("[StubDagStore] Stored data with CID: {:?}", cid);
        Ok(cid)
    }
    fn get(&self, cid: &Cid) -> Result<Option<Vec<u8>>, HostAbiError> {
        let store_lock = futures::executor::block_on(self.store.lock());
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
    staged_receipts: Arc<Mutex<VecDeque<MeshSubmitReceiptMessage>>>,
}
impl StubMeshNetworkService { 
    pub fn new() -> Self { 
        Self {
            staged_bids: Arc::new(Mutex::new(HashMap::new())),
            staged_receipts: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    // Test helper to stage a bid
    pub async fn stage_bid(&self, job_id: JobId, bid: MeshJobBid) {
        let mut bids_map = self.staged_bids.lock().await;
        bids_map.entry(job_id).or_default().push_back(bid);
    }
    // Test helper to stage a receipt
    pub async fn stage_receipt(&self, receipt_message: MeshSubmitReceiptMessage) {
        let mut receipts_queue = self.staged_receipts.lock().await;
        receipts_queue.push_back(receipt_message);
    }
}

#[async_trait]
impl MeshNetworkService for StubMeshNetworkService {
    async fn announce_job(&self, announcement: MeshJobAnnounce) -> Result<(), HostAbiError> {
        println!("[StubMeshNetworkService] Announced job: {:?}", announcement.job_id);
        Ok(())
    }

    async fn collect_bids_for_job(&self, job_id: JobId, _duration: Duration) -> Result<Vec<MeshJobBid>, HostAbiError> {
        println!("[StubMeshNetworkService] Collecting bids for job {:?}.", job_id);
        let mut bids_map = self.staged_bids.lock().await;
        if let Some(job_bids_queue) = bids_map.get_mut(&job_id) {
            let bids: Vec<MeshJobBid> = job_bids_queue.drain(..).collect();
            println!("[StubMeshNetworkService] Found {} staged bids for job {:?}", bids.len(), job_id);
            Ok(bids)
        } else {
            println!("[StubMeshNetworkService] No staged bids found for job {:?}. Returning empty vec.", job_id);
            Ok(Vec::new()) 
        }
    }

    async fn broadcast_assignment(&self, notice: JobAssignmentNotice) -> Result<(), HostAbiError> {
        println!("[StubMeshNetworkService] Broadcast assignment for job {:?} to executor {:?}", notice.job_id, notice.executor_did);
        Ok(())
    }

    async fn try_receive_receipt(&self) -> Result<Option<MeshSubmitReceiptMessage>, HostAbiError> {
        let mut receipts_queue = self.staged_receipts.lock().await;
        if let Some(receipt_msg) = receipts_queue.pop_front() {
            println!("[StubMeshNetworkService] try_receive_receipt: Popped staged receipt for job {:?}", receipt_msg.receipt.job_id);
            Ok(Some(receipt_msg))
        } else {
            // println!("[StubMeshNetworkService] try_receive_receipt called. No staged receipts. Returning None.");
            Ok(None)
        }
    }
} 