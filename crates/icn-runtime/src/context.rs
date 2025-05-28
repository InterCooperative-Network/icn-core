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
use std::sync::{Arc};
use std::time::{Duration as StdDuration, Instant as StdInstant};
use tokio::sync::{Mutex as TokioMutex};
use std::sync::atomic::AtomicU32;

use async_trait::async_trait;

use std::str::FromStr; 

#[cfg(feature = "enable-libp2p")]
use libp2p::{Multiaddr, PeerId as Libp2pPeerId};

use icn_identity::{generate_ed25519_keypair, did_key_from_verifying_key, SigningKey, VerifyingKey, sign_message, verify_signature as identity_verify_signature, EdSignature, SIGNATURE_LENGTH};

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
    fn verifying_key_ref(&self) -> &VerifyingKey;
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
    balances: Arc<TokioMutex<HashMap<Did, u64>>>,
}

impl SimpleManaLedger {
    pub fn new() -> Self {
        Self { balances: Arc::new(TokioMutex::new(HashMap::new())) }
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

impl Default for GovernanceModule {
    fn default() -> Self {
        Self::new()
    }
}

// Placeholder for charge_mana function (used in Job Manager)
// This would typically belong to the icn-economics crate or a related module.
// fn charge_mana(_executor_did: &Did, _amount: u64) -> Result<(), EconError> {
// In a real implementation, this would interact with the ManaRepository.
// For now, always succeed for testing purposes.
// Ok(())
// }

// Placeholder for SelectionPolicy (used in Job Manager)
// This would typically belong to the icn-mesh crate or a related module.
#[derive(Debug, Default)]
pub struct SelectionPolicy { /* ... fields ... */ }

// Placeholder for select_executor function (used in Job Manager)
// This would typically belong to the icn-mesh crate or a related module.
// fn select_executor(bids: Vec<MeshJobBid>, _policy: SelectionPolicy) -> Option<Did> {
// Simplistic: return the DID of the first bidder if any
// bids.first().map(|bid| bid.executor_did.clone())
// }
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
    async fn collect_bids_for_job(&self, job_id: &JobId, duration: StdDuration) -> Result<Vec<MeshJobBid>, HostAbiError>;
    async fn notify_executor_of_assignment(&self, notice: &JobAssignmentNotice) -> Result<(), HostAbiError>;
    async fn try_receive_receipt(&self, job_id: &JobId, expected_executor: &Did, timeout: StdDuration) -> Result<Option<IdentityExecutionReceipt>, HostAbiError>;
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

    async fn collect_bids_for_job(&self, job_id: &JobId, duration: StdDuration) -> Result<Vec<MeshJobBid>, HostAbiError> {
        debug!("[DefaultMeshNetworkService] Collecting bids for job {:?} for {:?}", job_id, duration);
        let mut bids = Vec::new();
        let mut receiver = self.inner.subscribe()
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to subscribe for bids: {}", e)))?;
        
        let end_time = StdInstant::now() + duration;

        loop {
            match tokio::time::timeout_at(tokio::time::Instant::from_std(end_time), receiver.recv()).await {
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

    async fn try_receive_receipt(&self, job_id: &JobId, expected_executor: &Did, timeout_duration: StdDuration) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
        debug!("[DefaultMeshNetworkService] Trying to receive receipt for job {:?} from {:?} for {:?}", job_id, expected_executor, timeout_duration);
        let mut receiver = self.inner.subscribe()
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to subscribe for receipts: {}", e)))?;

        let end_time = StdInstant::now() + timeout_duration;

        loop {
            match tokio::time::timeout_at(tokio::time::Instant::from_std(end_time), receiver.recv()).await {
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
    pub pending_mesh_jobs: Arc<TokioMutex<VecDeque<ActualMeshJob>>>,
    pub job_states: Arc<TokioMutex<HashMap<JobId, JobState>>>,
    pub governance_module: Arc<TokioMutex<GovernanceModule>>,
    pub mesh_network_service: Arc<dyn MeshNetworkService>, // Uses local MeshNetworkService trait
    pub signer: Arc<dyn Signer>, 
    pub dag_store: Arc<dyn StorageService>, // Uses local StorageService trait
}

impl RuntimeContext {
    pub fn new(
        current_identity: Did, 
        mesh_network_service: Arc<dyn MeshNetworkService>,
        signer: Arc<dyn Signer>,
        dag_store: Arc<dyn StorageService>
    ) -> Arc<Self> {
        let job_states = Arc::new(TokioMutex::new(HashMap::new()));
        let pending_mesh_jobs = Arc::new(TokioMutex::new(VecDeque::new()));

        Arc::new(Self {
            current_identity,
            mana_ledger: SimpleManaLedger::new(),
            pending_mesh_jobs,
            job_states,
            governance_module: Arc::new(TokioMutex::new(GovernanceModule::new())),
            mesh_network_service,
            signer,
            dag_store,
        })
    }

    #[cfg(feature = "enable-libp2p")]
    pub async fn new_with_libp2p_network(current_identity_str: &str, bootstrap_peers: Option<Vec<(Libp2pPeerId, Multiaddr)>>) -> Result<Arc<Self>, CommonError> {
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

    pub fn new_with_stubs(current_identity_str: &str) -> Arc<Self> {
        let current_identity = Did::from_str(current_identity_str).expect("Invalid DID for test context in new_with_stubs");
        Self::new(
            current_identity, 
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
            Arc::new(StubDagStore::new())
        )
    }

    pub fn new_with_stubs_and_mana(current_identity_str: &str, initial_mana: u64) -> Arc<Self> {
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

    async fn wait_for_and_process_receipt(self: Arc<Self>, job: ActualMeshJob, assigned_executor_did: Did) -> Result<(), HostAbiError> {
        info!("[JobManagerDetail] Waiting for receipt for job {:?} from executor {:?}", job.id, assigned_executor_did);
        // TODO: Use job.max_execution_wait_ms or a configurable default from job spec or runtime config
        let receipt_timeout = StdDuration::from_secs(60); 

        match self.mesh_network_service.try_receive_receipt(&job.id, &assigned_executor_did, receipt_timeout).await {
            Ok(Some(receipt)) => {
                info!("[JobManagerDetail] Received receipt for job {:?}: {:?}", job.id, receipt);
                
                // Verify signature of the receipt - this needs the public key of the *actual* executor.
                // This is a critical part that needs a DID resolution mechanism or a way to get the executor's VK.
                // For now, the existing logic used the RuntimeContext's signer, which is INCORRECT unless the node is executing its own job.
                // Placeholder: We need a way to resolve assigned_executor_did to its VerifyingKey.
                // This is a simplification and potential security issue if not handled correctly.
                // We assume the receipt's signature has been verified by the executor submitting it,
                // and the network layer provides some authenticity. A full verification here would be better.
                // Let's proceed with anchoring and assume signature is valid for now to simplify the JobManager flow.
                // A more robust system would: 
                // 1. Fetch VerifyingKey for receipt.executor_did (e.g., from a DID document or a trusted registry)
                // 2. Call receipt.verify_against_key(&retrieved_verifying_key)

                // Simplified: Log if verification fails but proceed to anchor for testing pipeline flow.
                // In production, an invalid signature should prevent anchoring and fail the job.
                let temp_vk_did_string = did_key_from_verifying_key(&self.signer.verifying_key_ref()); // Assuming signer has verifying_key_ref()
                if Did::from_str(&temp_vk_did_string).unwrap_or_default() == receipt.executor_did {
                    if let Err(e) = receipt.verify_against_key(&self.signer.verifying_key_ref()) {
                        error!("[JobManagerDetail] Receipt signature VERIFICATION FAILED for job {:?}: {}. Proceeding to anchor for stub testing only.", job.id, e);
                        // In a real system: return Err(HostAbiError::SignatureError(...));
                    } else {
                        info!("[JobManagerDetail] Receipt signature VERIFIED for job {:?}", job.id);
                    }
                } else {
                    warn!("[JobManagerDetail] Executor DID {:?} on receipt does not match context signer DID {:?}. Cannot verify signature with context signer. Assuming valid for stub testing.", receipt.executor_did, temp_vk_did_string);
                }

                match self.anchor_receipt(&receipt).await {
                    Ok(receipt_cid) => {
                        info!("[JobManagerDetail] Receipt for job {:?} anchored successfully: {:?}", job.id, receipt_cid);
                        let job_states_guard = self.job_states.lock().await;
                        job_states_guard.insert(job.id.clone(), JobState::Completed { receipt: receipt.clone() });
                        // TODO: Credit mana to executor, update reputation, etc.
                        Ok(())
                    }
                    Err(e) => {
                        error!("[JobManagerDetail] Failed to anchor receipt for job {:?}: {}. Marking as Failed (AnchorFailed).", job.id, e);
                        let job_states_guard = self.job_states.lock().await;
                        job_states_guard.insert(job.id.clone(), JobState::Failed { reason: format!("Failed to anchor receipt: {}", e) });
                        Err(HostAbiError::DagOperationFailed(format!("Failed to anchor receipt: {}",e)))
                    }
                }
            }
            Ok(None) => {
                warn!("[JobManagerDetail] No receipt received for job {:?} within timeout. Marking as Failed (NoReceipt).", job.id);
                let job_states_guard = self.job_states.lock().await;
                job_states_guard.insert(job.id.clone(), JobState::Failed { reason: "No receipt received within timeout".to_string() });
                Err(HostAbiError::NetworkError("No receipt received within timeout".to_string()))
            }
            Err(e) => {
                error!("[JobManagerDetail] Error while trying to receive receipt for job {:?}: {}. Marking as Failed (ReceiptError).", job.id, e);
                let job_states_guard = self.job_states.lock().await;
                job_states_guard.insert(job.id.clone(), JobState::Failed { reason: format!("Error receiving receipt: {}", e) });
                Err(e)
            }
        }
    }

    pub async fn spawn_mesh_job_manager(self: Arc<Self>) {
        info!("[JobManager] Starting background mesh job manager task...");
        let self_clone = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(StdDuration::from_secs(5));
            loop {
                interval.tick().await;
                debug!("[JobManagerLoop] Tick: Checking for pending jobs...");

                let mut jobs_to_requeue = VecDeque::new();
                let mut jobs_processed_in_cycle = 0;

                // Process jobs from the pending queue
                while let Some(job) = { // job here is ActualMeshJob
                    let mut pending_jobs_guard = self_clone.pending_mesh_jobs.lock().await;
                    let popped_job = pending_jobs_guard.pop_front();
                    drop(pending_jobs_guard);
                    popped_job
                } {
                    jobs_processed_in_cycle += 1;
                    let current_job_id = job.id.clone(); // Clone for use in logs and map keys

                    // Get current state from the central job_states map
                    let mut job_states_guard = self_clone.job_states.lock().await;
                    let current_job_state = job_states_guard.get(&current_job_id).cloned();
                    drop(job_states_guard); // Release lock quickly

                    info!("[JobManagerLoop] Processing job: {:?}, current state from map: {:?}", current_job_id, current_job_state);

                    match current_job_state {
                        Some(JobState::Pending) => {
                            info!("[JobManagerLoop] Job {:?} is Pending. Announcing...", current_job_id);
                            if let Err(e) = self_clone.mesh_network_service.announce_job(&job).await {
                                error!("[JobManagerLoop] Failed to announce job {:?}: {}. Re-queuing.", current_job_id, e);
                                jobs_to_requeue.push_back(job); // Re-queue the ActualMeshJob
                                continue;
                            }
                            info!("[JobManagerLoop] Job {:?} announced. Collecting bids...", current_job_id);
                            let bid_collection_duration = StdDuration::from_secs(10);
                            
                            let bids_result = self_clone.mesh_network_service
                                .collect_bids_for_job(&current_job_id, bid_collection_duration)
                                .await;

                            let bids = match bids_result {
                                Ok(b) => b,
                                Err(e) => {
                                    error!("[JobManagerLoop] Bid collection failed for job {:?}: {}. Re-queuing.", current_job_id, e);
                                    jobs_to_requeue.push_back(job); // Re-queue the ActualMeshJob
                                    continue;
                                }
                            };

                            if bids.is_empty() {
                                warn!("[JobManagerLoop] No bids received for job {:?}. Marking as Failed (NoBids).", current_job_id);
                                let job_states_guard = self_clone.job_states.lock().await;
                                job_states_guard.insert(current_job_id.clone(), JobState::Failed { reason: "No bids received".to_string() });
                                drop(job_states_guard);
                                // TODO: Refund mana to submitter if applicable
                                continue;
                            }

                            info!("[JobManagerLoop] Received {} bids for job {:?}. Selecting executor...", bids.len(), current_job_id);
                            if let Some(selected_bid) = bids.into_iter().next() { // Simplistic selection
                                let new_state = JobState::Assigned { executor: selected_bid.executor_did.clone() };
                                info!("[JobManagerLoop] Job {:?} assigned to executor {:?}. Notifying...", current_job_id, selected_bid.executor_did);
                                
                                let job_states_guard = self_clone.job_states.lock().await;
                                job_states_guard.insert(current_job_id.clone(), new_state);
                                drop(job_states_guard);
        
                                let notice = JobAssignmentNotice {
                                    job_id: current_job_id.clone(),
                                    executor_did: selected_bid.executor_did.clone(),
                                };

                                if let Err(e) = self_clone.mesh_network_service.notify_executor_of_assignment(&notice).await {
                                    error!("[JobManagerLoop] Failed to notify executor for job {:?}: {}. Reverting to Pending.", current_job_id, e);
                                    let job_states_guard = self_clone.job_states.lock().await;
                                    job_states_guard.insert(current_job_id.clone(), JobState::Pending); // Revert state in map
                                    drop(job_states_guard);
                                    jobs_to_requeue.push_back(job); // Re-queue the ActualMeshJob
                                    continue;
                                }
                                
                                info!("[JobManagerLoop] Job {:?} successfully assigned. Spawning receipt monitor.", current_job_id);
                                // self_clone is Arc<RuntimeContext> here
                                let task_ctx = self_clone.clone(); // Clone the Arc for the new task
                                tokio::spawn(async move {
                                    if let Err(e) = task_ctx.wait_for_and_process_receipt(job, selected_bid.executor_did).await {
                                        error!("[JobManagerDetail] Error in wait_for_and_process_receipt for job {:?}: {:?}", current_job_id, e);
                                    }
                                });
                            } else {
                                warn!("[JobManagerLoop] Executor selection failed for job {:?} despite having bids. Re-queuing (state remains Pending).", current_job_id);
                                jobs_to_requeue.push_back(job); // Re-queue the ActualMeshJob, state is still Pending in map
                                continue;
                            }
                        }
                        Some(JobState::Assigned { ref executor }) => {
                            debug!("[JobManagerLoop] Job {:?} is Assigned to {:?}. Receipt handling in separate task. Re-queuing for later check.", current_job_id, executor);
                            // This job will be re-added to pending_mesh_jobs. Its state in job_states is still Assigned.
                            // The wait_for_and_process_receipt task is responsible for updating it to Completed or Failed.
                            // If that task fails or the job times out, we might need an additional mechanism here
                            // to detect stale "Assigned" jobs and move them to Failed.
                            // For now, simply re-queueing means it will be picked up again.
                            // Consider adding a timestamp to JobState::Assigned for timeout logic here.
                            jobs_to_requeue.push_back(job); // Re-queue ActualMeshJob
                        }
                        Some(JobState::Completed { .. }) | Some(JobState::Failed { .. }) => {
                            info!("[JobManagerLoop] Job {:?} is in a terminal state. No action.", current_job_id);
                            // Do not re-queue jobs that are completed or failed.
                        }
                        None => { // Job was in pending_mesh_jobs but not in job_states map
                            error!("[JobManagerLoop] Job {:?} found in pending queue but not in job_states map! This should not happen. Discarding.", current_job_id);
                            // This indicates an inconsistency. The job object exists but its state is unknown.
                            // For safety, we probably shouldn't process it.
                        }
                    }
                } // End while let Some(job)

                // Re-queue jobs that need another attempt
                if !jobs_to_requeue.is_empty() {
                    let mut pending_jobs_guard = self_clone.pending_mesh_jobs.lock().await;
                    for job_to_requeue in jobs_to_requeue {
                        // Ensure it's added to the back for fairness
                        pending_jobs_guard.push_back(job_to_requeue);
                    }
                    drop(pending_jobs_guard);
                }

                if jobs_processed_in_cycle == 0 {
                    let pending_jobs_guard = self_clone.pending_mesh_jobs.lock().await;
                    let is_empty = pending_jobs_guard.is_empty();
                    drop(pending_jobs_guard);
                    if is_empty {
                        debug!("[JobManagerLoop] No jobs processed and no pending jobs. Manager is idle.");
                    }
                }
            } // End loop
        }); // End tokio::spawn
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

        // Verify the receipt signature against the signer's public key
        // This assumes the signer in the context is the one whose keys should verify all receipts.
        // This might be too simplistic; a real system might need to fetch the specific executor's VK.
        let signer_pk_bytes = self.signer.public_key_bytes();
        let verifying_key_bytes_array: [u8; 32] = signer_pk_bytes.as_slice().try_into()
            .map_err(|_| HostAbiError::CryptoError("Signer public key is not 32 bytes".to_string()))?;
        let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes_array)
            .map_err(|e| HostAbiError::CryptoError(format!("Failed to create verifying key from signer: {}", e)))?;
        
        // Check if the DID derived from the signer's public key matches the receipt's executor_did
        let temp_vk_did_string = did_key_from_verifying_key(&verifying_key);
        // TODO: This equality check might be too strict if DIDs can have different representations
        // that are semantically equivalent. For did:key this should be fine if canonical.
        if Did::from_str(&temp_vk_did_string).unwrap_or_default() == receipt.executor_did {
            if let Err(e) = receipt.verify_against_key(&self.signer.verifying_key_ref()) {
                return Err(HostAbiError::SignatureError(format!("Receipt signature verification failed for job {:?}, executor {:?}: {}", receipt.job_id, receipt.executor_did, e)));
            }
        } else {
            // This case is tricky: if the context's signer is NOT the executor, how do we verify?
            // For now, we assume the context's signer *is* the one who should verify, or this is an error.
            // A more robust system would fetch the executor_did's public key from a DID document or cache.
            warn!("Receipt executor DID {:?} does not match current signer DID {:?}. Verification might be incorrect if signer is not the executor.", receipt.executor_did, temp_vk_did_string);
            // Attempt verification anyway with the context's signer; this branch implies a mismatch.
            // If the context signer IS the executor, this is redundant. If not, this will likely fail.
            if let Err(e) = receipt.verify_against_key(&verifying_key) {
                return Err(HostAbiError::SignatureError(format!("Receipt signature verification failed (DID mismatch) for job {:?}, executor {:?}: {}", receipt.job_id, receipt.executor_did, e)));
            }
        }

        // If signature is valid, store the receipt in DAG
        let final_receipt_bytes = serde_json::to_vec(receipt)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to serialize final receipt for DAG: {}", e)))?;
        
        let cid = self.dag_store.put(&final_receipt_bytes).await?;
        println!("[CONTEXT] Anchored receipt for job_id {:?} with CID: {:?}. Executor: {:?}. Receipt cost {}ms.", 
                 receipt.job_id, cid, receipt.executor_did, receipt.cpu_ms);
        
        { 
            let job_states_guard = self.job_states.lock().await;
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

    pub async fn execute_governance_proposal(&mut self, _proposal_id_str: &str) -> Result<(), HostAbiError> {
        todo!("Implement full governance proposal execution logic");
    }
}

// --- Supporting: RuntimeContext::new_for_test ---
impl RuntimeContext {
    pub fn new_for_test(
        current_identity: Did,
        signer: StubSigner, 
        mesh_network_service: Arc<StubMeshNetworkService>,
        dag_store: Arc<StubDagStore>,
    ) -> Arc<Self> {
        let job_states = Arc::new(TokioMutex::new(HashMap::new())); 
        let pending_mesh_jobs = Arc::new(TokioMutex::new(VecDeque::new())); 
        let mana_ledger = SimpleManaLedger::new();
        let governance_module = Arc::new(TokioMutex::new(GovernanceModule::default()));

        Arc::new(Self {
            current_identity,
            mana_ledger,
            pending_mesh_jobs,
            job_states,
            governance_module,
            mesh_network_service, 
            signer: Arc::new(signer), 
            dag_store, 
        })
    }
}
// --- End Supporting: RuntimeContext::new_for_test ---

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

    // Added for wait_for_and_process_receipt helper
    pub fn verifying_key_ref(&self) -> &VerifyingKey {
        &self.pk
    }
}

// #[async_trait] // No longer async
impl Signer for StubSigner {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, HostAbiError> {
        Ok(sign_message(&self.sk, payload).to_bytes().to_vec())
    }

    fn verify(&self, payload: &[u8], signature_bytes: &[u8], public_key_bytes: &[u8]) -> Result<bool, HostAbiError> {
        let pk_array: [u8; 32] = public_key_bytes.try_into()
            .map_err(|_| HostAbiError::InvalidParameters("Public key bytes not 32 bytes long".to_string()))?;
        let verifying_key = VerifyingKey::from_bytes(&pk_array)
            .map_err(|e| HostAbiError::CryptoError(format!("Failed to create verifying key: {}", e)))?;
        
        let signature_array: [u8; SIGNATURE_LENGTH] = signature_bytes.try_into()
            .map_err(|_| HostAbiError::InvalidParameters(format!("Signature not {} bytes long", SIGNATURE_LENGTH)))?;
        let signature = EdSignature::from_bytes(&signature_array); // ed25519_dalek::Signature::from_bytes
            // .map_err(|e| HostAbiError::CryptoError(format!("Failed to create signature from bytes: {}", e)))?;

        Ok(identity_verify_signature(&verifying_key, payload, &signature))
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        self.pk.as_bytes().to_vec()
    }

    fn did(&self) -> Did {
        // Assuming self.did_string is a valid DID string like "did:key:z..."
        Did::from_str(&self.did_string).expect("Failed to parse internally generated DID string")
    }

    fn verifying_key_ref(&self) -> &VerifyingKey {
        &self.pk
    }
}

#[derive(Debug, Clone)]
pub struct StubDagStore { // Renamed from StubStorageService for consistency if tests use this name
    store: Arc<TokioMutex<HashMap<Cid, Vec<u8>>>>,
}
impl StubDagStore {
    pub fn new() -> Self {
        Self { store: Arc::new(TokioMutex::new(HashMap::new())) }
    }
    pub async fn all(&self) -> Result<HashMap<Cid, Vec<u8>>, HostAbiError> {
        let store_lock = self.store.lock().await;
        Ok(store_lock.clone())
    }
}

impl Default for StubDagStore {
    fn default() -> Self {
        Self::new()
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
    staged_bids: Arc<TokioMutex<HashMap<JobId, VecDeque<MeshJobBid>>>>,
    staged_receipts: Arc<TokioMutex<VecDeque<LocalMeshSubmitReceiptMessage>>>, // Using local placeholder & TokioMutex
}
impl StubMeshNetworkService { 
    pub fn new() -> Self { 
        Self {
            staged_bids: Arc::new(TokioMutex::new(HashMap::new())),
            staged_receipts: Arc::new(TokioMutex::new(VecDeque::new())),
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

impl Default for StubMeshNetworkService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MeshNetworkService for StubMeshNetworkService { // Implements local MeshNetworkService trait
    fn as_any(&self) -> &dyn std::any::Any { self }

    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        println!("[StubMeshNetworkService] Announced job: {:?}", job.id);
        Ok(())
    }

    async fn collect_bids_for_job(&self, job_id: &JobId, _duration: StdDuration) -> Result<Vec<MeshJobBid>, HostAbiError> {
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

    async fn try_receive_receipt(&self, _job_id: &JobId, _expected_executor: &Did, _timeout_duration: StdDuration) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
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
// mod reputation_updater { 
// 
//     use icn_identity::ExecutionReceipt as IdentityExecutionReceipt;
// 
//     #[derive(Debug, Default)]
//     pub struct ReputationUpdater;
// 
//     impl ReputationUpdater {
//         pub fn new() -> Self { Self }
//         pub fn submit(&self, _receipt: &IdentityExecutionReceipt) {
//             // Placeholder for reputation update logic
//             log::info!("[ReputationUpdater STUB] Submitted receipt: {:?}", _receipt.job_id);
//         }
//     }
// } 
// } 
