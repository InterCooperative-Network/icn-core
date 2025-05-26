//! Defines the `RuntimeContext`, `HostEnvironment`, and related types for the ICN runtime.

use icn_common::{Did, Cid, CommonError};
use icn_identity::{ExecutionReceipt as IdentityExecutionReceipt}; 
use icn_economics::{EconError};
use icn_mesh::{JobId, ActualMeshJob, MeshJobBid, JobState};

use icn_network::{NetworkService as ActualNetworkService, NetworkMessage};
#[cfg(feature = "enable-libp2p")]
use icn_network::libp2p_service::Libp2pNetworkService as ActualLibp2pNetworkService; 
use downcast_rs::{DowncastSync, impl_downcast}; 

use log::{info, warn, error, debug};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex as StdMutex, RwLock};
use std::time::{Duration as StdDuration, Instant as TokioInstant};
use tokio::sync::{Mutex as TokioMutex, oneshot, watch, broadcast, Barrier};

use async_trait::async_trait;

use std::str::FromStr; 

#[cfg(feature = "enable-libp2p")]
use libp2p::{Multiaddr, PeerId as Libp2pPeerId};

use crate::error::MeshJobError;
use tokio::time::{Duration, sleep};
use icn_identity::{generate_ed25519_keypair, did_key_from_verifying_key, SigningKey, VerifyingKey, sign_message, verify_signature as identity_verify_signature, SignatureBytes as IdentitySignatureBytes};
use ed25519_dalek::Signature as EdSignature; // For converting to/from bytes

// Counter for generating unique (within this runtime instance) job IDs for stubs
pub static NEXT_JOB_ID: AtomicU32 = AtomicU32::new(1);

// --- Placeholder Local Stubs / Forward Declarations ---

// Updated Signer trait to be synchronous and match new crypto capabilities
// #[async_trait] // No longer async
pub trait Signer: Send + Sync + std::fmt::Debug {
    // async fn sign(&self, did: &Did, data: &[u8]) -> Result<Vec<u8>, HostAbiError>; // Old async version
    // async fn verify(&self, did: &Did, data: &[u8], signature: &[u8]) -> Result<bool, HostAbiError>; // Old async version
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, HostAbiError>;
    fn verify(&self, payload: &[u8], signature: &[u8], public_key_bytes: &[u8]) -> Result<bool, HostAbiError>; // Added pk_bytes
    fn public_key_bytes(&self) -> Vec<u8>;
    fn did(&self) -> Did;
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
            Arc::new(StubSigner::new()), 
            Arc::new(StubDagStore::new()), 
        ))
    }

    pub fn new_with_stubs(current_identity_str: &str) -> Self {
        let current_identity = Did::from_str(current_identity_str).expect("Invalid DID for test context in new_with_stubs");
        Self::new(
            current_identity, 
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
            Arc::new(StubDagStore::new())
        )
    }

    pub fn new_with_stubs_and_mana(current_identity_str: &str, initial_mana: u64) -> Self {
        let current_identity = Did::from_str(current_identity_str).expect("Invalid DID for test context in new_with_stubs_and_mana");
        let ctx = Self::new(
            current_identity.clone(), 
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
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
        info!("[JobManager] Starting background mesh job manager task...");
        let self_clone = self.clone(); // Clone Arc for the new task

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(StdDuration::from_secs(5)); // Check every 5 seconds
            loop {
                interval.tick().await;
                debug!("[JobManager] Checking for pending jobs and processing...");

                let mut jobs_to_requeue = VecDeque::new();
                let mut jobs_processed_in_cycle = 0;

                // Drain pending jobs
                while let Some(mut job) = {
                    let mut pending_jobs_guard = self_clone.pending_mesh_jobs.lock().await;
                    pending_jobs_guard.pop_front()
                } {
                    jobs_processed_in_cycle += 1;
                    info!("[JobManager] Processing job: {:?}, current state: {:?}", job.id, job.state);

                    match job.state {
                        JobState::Pending => {
                            info!("[JobManager] Job {:?} is Pending. Attempting to announce and find executor.", job.id);
                            // Announce job
                            if let Err(e) = self_clone.mesh_network_service.announce_job(&job).await {
                                error!("[JobManager] Failed to announce job {:?}: {}. Re-queuing.", job.id, e);
                                jobs_to_requeue.push_back(job);
                                continue;
                            }

                            // Collect bids (simplified timeout)
                            // TODO: Use job.max_bid_wait_ms or a configurable default
                            let bid_collection_duration = Duration::from_secs(10);
                            match self_clone.mesh_network_service.collect_bids_for_job(&job.id, bid_collection_duration).await {
                                Ok(bids) => {
                                    if bids.is_empty() {
                                        warn!("[JobManager] No bids received for job {:?}. Marking as Failed (NoBids).", job.id);
                                        job.state = JobState::Failed { reason: "No bids received".to_string() };
                                        let mut states_guard = self_clone.job_states.lock().await;
                                        states_guard.insert(job.id.clone(), job.state.clone());
                                        // TODO: Refund mana to submitter if applicable
                                    } else {
                                        info!("[JobManager] Received {} bids for job {:?}. Selecting executor.", bids.len(), job.id);
                                        // Select executor (simplified: first bidder)
                                        // TODO: Use actual icn_mesh::select_executor with a real SelectionPolicy
                                        if let Some(selected_bid) = bids.into_iter().next() { // Simplistic selection
                                            job.state = JobState::Assigned { executor: selected_bid.executor_did.clone() };
                                            info!("[JobManager] Job {:?} assigned to executor {:?}. Notifying executor.", job.id, selected_bid.executor_did);
                                            let mut states_guard = self_clone.job_states.lock().await;
                                            states_guard.insert(job.id.clone(), job.state.clone());

                                            let notice = JobAssignmentNotice {
                                                job_id: job.id.clone(),
                                                executor_did: selected_bid.executor_did.clone(),
                                            };
                                            if let Err(e) = self_clone.mesh_network_service.notify_executor_of_assignment(&notice).await {
                                                error!("[JobManager] Failed to notify executor for job {:?}: {}. Reverting to Pending.", job.id, e);
                                                job.state = JobState::Pending; // Revert state
                                                jobs_to_requeue.push_back(job);
                                            } else {
                                                // Successfully assigned, wait for receipt
                                                info!("[JobManager] Job {:?} successfully assigned. Waiting for receipt.", job.id);
                                                // The job remains in Assigned state, executor will submit receipt.
                                                // We will handle receipt submission in another part of the runtime context or API.
                                                // For now, we just move to next job processing cycle.
                                                // To complete the loop here for testing, we might need to call try_receive_receipt
                                                let assigned_executor_did = selected_bid.executor_did.clone();
                                                let job_id_clone = job.id.clone();
                                                let self_clone_for_receipt = self_clone.clone();
                                                let _job_clone_for_receipt = job.clone(); // job is moved if we don't clone

                                                // Spawn a task to wait for the receipt for this specific job
                                                tokio::spawn(async move {
                                                    // TODO: Use job.max_execution_wait_ms or a configurable default
                                                    let receipt_timeout = Duration::from_secs(60); 
                                                    match self_clone_for_receipt.mesh_network_service.try_receive_receipt(&job_id_clone, &assigned_executor_did, receipt_timeout).await {
                                                        Ok(Some(receipt)) => {
                                                            info!("[JobManager] Received receipt for job {:?}: {:?}", job_id_clone, receipt);
                                                            // Verify signature of the receipt
                                                            // This part needs the public key of the executor.
                                                            // For StubSigner, we assume it's verifying a receipt from itself or knows the key.
                                                            // In a real system, this would involve DID resolution.
                                                            let executor_pk_bytes = self_clone_for_receipt.signer.public_key_bytes(); // Assuming signer is the executor for now.
                                                            // THIS IS A SIMPLIFICATION: We need the actual executor's public key.
                                                            // If the signer of the context is NOT the executor, this will fail unless the executor IS the context's signer.
                                                            let temp_verifying_key = VerifyingKey::from_bytes(&executor_pk_bytes).expect("JobManager: Failed to create VK from signer bytes");

                                                            match receipt.verify_against_key(&temp_verifying_key) {
                                                                Ok(_) => {
                                                                    info!("[JobManager] Receipt signature VERIFIED for job {:?}", job_id_clone);
                                                                    // Anchor the receipt
                                                                    match self_clone_for_receipt.anchor_receipt(&receipt).await {
                                                                        Ok(receipt_cid) => {
                                                                            info!("[JobManager] Receipt for job {:?} anchored successfully: {:?}", job_id_clone, receipt_cid);
                                                                            let mut states_guard = self_clone_for_receipt.job_states.lock().await;
                                                                            states_guard.insert(job_id_clone.clone(), JobState::Completed { receipt: receipt.clone() });
                                                                            // TODO: Credit mana to executor, update reputation, etc.
                                                                        }
                                                                        Err(e) => {
                                                                            error!("[JobManager] Failed to anchor receipt for job {:?}: {}. Marking as Failed (AnchorFailed).", job_id_clone, e);
                                                                            let mut states_guard = self_clone_for_receipt.job_states.lock().await;
                                                                            states_guard.insert(job_id_clone.clone(), JobState::Failed { reason: format!("Failed to anchor receipt: {}", e) });
                                                                        }
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    error!("[JobManager] Receipt signature VERIFICATION FAILED for job {:?}: {}. Marking as Failed (InvalidSignature).", job_id_clone, e);
                                                                    let mut states_guard = self_clone_for_receipt.job_states.lock().await;
                                                                    states_guard.insert(job_id_clone.clone(), JobState::Failed { reason: format!("Invalid receipt signature: {}", e) });
                                                                }
                                                            }
                                                        }
                                                        Ok(None) => {
                                                            warn!("[JobManager] No receipt received for job {:?} within timeout. Marking as Failed (NoReceipt).", job_id_clone);
                                                            let mut states_guard = self_clone_for_receipt.job_states.lock().await;
                                                            states_guard.insert(job_id_clone.clone(), JobState::Failed { reason: "No receipt received within timeout".to_string() });
                                                        }
                                                        Err(e) => {
                                                            error!("[JobManager] Error while trying to receive receipt for job {:?}: {}. Marking as Failed (ReceiptError).", job_id_clone, e);
                                                            let mut states_guard = self_clone_for_receipt.job_states.lock().await;
                                                            states_guard.insert(job_id_clone.clone(), JobState::Failed { reason: format!("Error receiving receipt: {}", e) });
                                                        }
                                                    }
                                                });
                                            }
                                        } else {
                                            warn!("[JobManager] Executor selection failed for job {:?} despite having bids. Re-queuing.", job.id);
                                            job.state = JobState::Pending; // Revert state
                                            jobs_to_requeue.push_back(job);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("[JobManager] Failed to collect bids for job {:?}: {}. Re-queuing.", job.id, e);
                                    jobs_to_requeue.push_back(job);
                                    continue;
                                }
                            }
                        }
                        JobState::Assigned { executor } => {
                            // This state means we are waiting for the executor to provide a receipt.
                            // The spawned task from the Pending -> Assigned transition handles receipt listening.
                            // If that task fails or times out, it updates the job state directly.
                            // We could add a timeout here to ensure jobs don't stay Assigned indefinitely
                            // if the spawned task itself fails to update the state.
                            debug!("[JobManager] Job {:?} is already Assigned to {:?}. Receipt handling is in a separate task.", job.id, executor);
                            // For now, we don't re-queue it from here unless a master timeout is hit.
                            // It will be re-processed in the next cycle if still in this state without progress.
                            // To prevent busy loop on assigned jobs, we can put them into a different temporary list.
                            // For simplicity, we'll let it be checked again. Consider adding a timestamp to JobState::Assigned.
                            jobs_to_requeue.push_back(job); // Put it back to check later
                        }
                        JobState::Completed { .. } | JobState::Failed { .. } => {
                            info!("[JobManager] Job {:?} is already in a terminal state ({:?}). No further action.", job.id, job.state);
                            // No action needed for completed or failed jobs in this loop
                        }
                    }
                }

                // Re-queue jobs that need it
                if !jobs_to_requeue.is_empty() {
                    let mut pending_jobs_guard = self_clone.pending_mesh_jobs.lock().await;
                    for job_to_requeue in jobs_to_requeue {
                        pending_jobs_guard.push_back(job_to_requeue); // Add to the back for reprocessing
                    }
                }

                if jobs_processed_in_cycle == 0 && self_clone.pending_mesh_jobs.lock().await.is_empty() {
                    debug!("[JobManager] No jobs processed and no pending jobs. Manager is idle.");
                }
            }
        });
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
        info!("[CONTEXT] Attempting to anchor receipt for job {:?} from executor {:?}", receipt.job_id, receipt.executor_did);

        // Step 1: Verify the signature on the receipt.
        // For now, we assume the receipt should be verifiable by this node's own identity/signer.
        // In a real system, we would look up the public key for receipt.executor_did.
        let signer_pk_bytes = self.signer.public_key_bytes();
        let verifying_key = VerifyingKey::from_bytes(&signer_pk_bytes)
            .map_err(|e| HostAbiError::CryptoError(format!("Failed to construct VerifyingKey for self: {}", e)))?;

        match receipt.verify_against_key(&verifying_key) {
            Ok(_) => {
                info!("[CONTEXT] Receipt signature verified for job {:?}", receipt.job_id);
            }
            Err(e) => {
                error!("[CONTEXT] Receipt signature verification failed for job {:?}: {}", receipt.job_id, e);
                return Err(HostAbiError::SignatureError(format!("Receipt signature verification failed: {}", e)));
            }
        }

        // Step 2: Check for sufficient mana (already done by JobManager prior to this typically, but can double check or log)
        // For now, this is simplified.

        // Step 3: Store the receipt in DAG storage
        let final_receipt_bytes = serde_json::to_vec(receipt)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to serialize final receipt for DAG: {}", e)))?;
        
        let cid = self.dag_store.put(&final_receipt_bytes).await?;
        println!("[CONTEXT] Anchored receipt for job_id {:?} with CID: {:?}. Executor: {:?}. Receipt cost {}ms.", 
                 receipt.job_id, cid, receipt.executor_did, receipt.cpu_ms);
        
        { 
            let mut job_states_guard = self.job_states.lock().await;
            job_states_guard.insert(receipt.job_id.clone(), JobState::Completed { receipt: receipt.clone() });
            println!("[CONTEXT] Job {:?} state updated to Completed.", receipt.job_id);
        }
        println!("[CONTEXT] Placeholder: Reputation update needed for executor {:?} for job {:?}.", receipt.executor_did, receipt.job_id);
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

// StubSigner → real signer
#[derive(Debug)]
pub struct StubSigner {
    sk: SigningKey,
    pk: VerifyingKey,
    did_string: String, // Store the DID string for efficiency
}

impl StubSigner {
    pub fn new() -> Self {
        let (sk, pk) = generate_ed25519_keypair();
        let did_string = did_key_from_verifying_key(&pk);
        Self { sk, pk, did_string }
    }

    pub fn new_with_keys(sk: SigningKey, pk: VerifyingKey) -> Self {
        let did_string = did_key_from_verifying_key(&pk);
        Self { sk, pk, did_string }
    }
}

// #[async_trait] // No longer async
impl Signer for StubSigner {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, HostAbiError> {
        Ok(sign_message(&self.sk, payload).to_bytes().to_vec())
    }

    fn verify(&self, payload: &[u8], signature_bytes: &[u8], public_key_bytes: &[u8]) -> Result<bool, HostAbiError> {
        let signature = EdSignature::from_bytes(signature_bytes)
            .map_err(|e| HostAbiError::CryptoError(format!("Invalid signature format for verify: {}", e)))?;
        let pk = VerifyingKey::from_bytes(public_key_bytes)
            .map_err(|e| HostAbiError::CryptoError(format!("Invalid public key for verify: {}", e)))?;
        Ok(identity_verify_signature(&pk, payload, &signature))
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        self.pk.as_bytes().to_vec()
    }

    fn did(&self) -> Did {
        // This assumes Did::from_string can parse the full did string. Adjust if needed.
        Did::from_string(&self.did_string).expect("Failed to parse internally generated DID string")
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