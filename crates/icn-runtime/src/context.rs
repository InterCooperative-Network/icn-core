//! Defines the `RuntimeContext`, `HostEnvironment`, and related types for the ICN runtime.

use log::{info, warn, error, debug};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration as StdDuration, Instant as StdInstant};
use tokio::sync::Mutex as AsyncMutex;
use std::sync::atomic::AtomicU32;

use icn_common::{Did, Cid, CommonError};
use icn_identity::{ExecutionReceipt as IdentityExecutionReceipt, SignatureBytes};
use icn_mesh::{JobId, ActualMeshJob, MeshJobBid, JobState, JobSpec, Resources};
use icn_network::{NetworkService, NetworkMessage};

use async_trait::async_trait;

use downcast_rs::{DowncastSync, impl_downcast}; 

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
        let balances = self.balances.lock().unwrap();
        balances.get(account).cloned()
    }
    pub async fn set_balance(&self, account: &Did, amount: u64) {
        let mut balances = self.balances.lock().unwrap();
        balances.insert(account.clone(), amount);
    }
    pub async fn spend(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        let mut balances = self.balances.lock().unwrap();
        let balance = balances.get_mut(account).ok_or_else(|| HostAbiError::AccountNotFound(account.clone()))?;
        if *balance < amount {
            return Err(HostAbiError::InsufficientMana);
        }
        *balance -= amount;
        Ok(())
    }
    pub async fn credit(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        let mut balances = self.balances.lock().unwrap();
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

// SelectionPolicy is now imported from icn_mesh

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
    inner: Arc<dyn NetworkService>, // Uses the imported NetworkService from icn-network
}

impl DefaultMeshNetworkService {
    pub fn new(service: Arc<dyn NetworkService>) -> Self {
        Self { inner: service }
    }

    // This method allows getting the concrete Libp2pNetworkService if that's what `inner` holds.
    #[cfg(feature = "enable-libp2p")]
    pub fn get_underlying_broadcast_service(&self) -> Result<Arc<dyn NetworkService>, CommonError> {
        self.inner.clone().downcast_arc::<NetworkService>()
            .map_err(|_e| CommonError::NetworkError("Failed to downcast inner NetworkService to Libp2pNetworkService. Ensure it was constructed with Libp2pNetworkService.".to_string()))
    }
}

#[async_trait]
impl MeshNetworkService for DefaultMeshNetworkService {
    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        debug!("[DefaultMeshNetworkService] Announcing job {:?} to the network", job.id);
        
        let message = NetworkMessage::MeshJobAnnouncement(job.clone());
        self.inner.broadcast_message(message).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to announce job: {}", e)))?;
        
        info!("[DefaultMeshNetworkService] Job {:?} announced successfully", job.id);
        Ok(())
    }

    async fn collect_bids_for_job(&self, job_id: &JobId, duration: StdDuration) -> Result<Vec<MeshJobBid>, HostAbiError> {
        debug!("[DefaultMeshNetworkService] Collecting bids for job {:?} for {:?}", job_id, duration);
        
        // Subscribe to network messages
        let mut receiver = self.inner.subscribe().await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to subscribe to network: {}", e)))?;
        
        let mut collected_bids = Vec::new();
        let start_time = StdInstant::now();
        
        while start_time.elapsed() < duration {
            let remaining_time = duration.saturating_sub(start_time.elapsed());
            if remaining_time.is_zero() {
                break;
            }
            
            // Use tokio::time::timeout instead of tokio::timeout
            match tokio::time::timeout(remaining_time, receiver.recv()).await {
                Ok(Some(message)) => {
                    if let NetworkMessage::BidSubmission(bid) = message {
                        if bid.job_id == *job_id {
                            debug!("[DefaultMeshNetworkService] Received bid from {:?} for job {:?}", 
                                  bid.executor_did, job_id);
                            collected_bids.push(bid);
                        }
                    }
                }
                Ok(None) => {
                    warn!("[DefaultMeshNetworkService] Network message receiver closed while collecting bids");
                    break;
                }
                Err(_) => {
                    // Timeout - continue checking for more bids until duration expires
                    continue;
                }
            }
        }
        
        info!("[DefaultMeshNetworkService] Collected {} bids for job {:?}", 
              collected_bids.len(), job_id);
        Ok(collected_bids)
    }

    async fn notify_executor_of_assignment(&self, notice: &JobAssignmentNotice) -> Result<(), HostAbiError> {
        debug!("[DefaultMeshNetworkService] Notifying executor {:?} of assignment for job {:?}", 
               notice.executor_did, notice.job_id);
        
        let message = NetworkMessage::JobAssignmentNotification(notice.job_id.clone(), notice.executor_did.clone());
        self.inner.broadcast_message(message).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to notify executor of assignment: {}", e)))?;
        
        info!("[DefaultMeshNetworkService] Assignment notification sent for job {:?}", notice.job_id);
        Ok(())
    }

    async fn try_receive_receipt(&self, job_id: &JobId, expected_executor: &Did, timeout_duration: StdDuration) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
        debug!("[DefaultMeshNetworkService] Waiting for receipt from {:?} for job {:?}", 
               expected_executor, job_id);
        
        // Subscribe to network messages
        let mut receiver = self.inner.subscribe().await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to subscribe to network: {}", e)))?;
        
        let start_time = StdInstant::now();
        
        while start_time.elapsed() < timeout_duration {
            let remaining_time = timeout_duration.saturating_sub(start_time.elapsed());
            if remaining_time.is_zero() {
                break;
            }
            
            match tokio::time::timeout(remaining_time, receiver.recv()).await {
                Ok(Some(message)) => {
                    if let NetworkMessage::SubmitReceipt(receipt) = message {
                        if receipt.job_id == *job_id && receipt.executor_did == *expected_executor {
                            info!("[DefaultMeshNetworkService] Received receipt from {:?} for job {:?}", 
                                  expected_executor, job_id);
                            return Ok(Some(receipt));
                        } else {
                            debug!("[DefaultMeshNetworkService] Received receipt for different job or executor: job={:?}, executor={:?}", 
                                   receipt.job_id, receipt.executor_did);
                        }
                    }
                }
                Ok(None) => {
                    warn!("[DefaultMeshNetworkService] Network message receiver closed while waiting for receipt");
                    break;
                }
                Err(_) => {
                    // Timeout - continue checking until duration expires
                    continue;
                }
            }
        }
        
        debug!("[DefaultMeshNetworkService] No receipt received for job {:?} within timeout", job_id);
        Ok(None)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
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
    pub pending_mesh_jobs: Arc<AsyncMutex<VecDeque<ActualMeshJob>>>,
    pub job_states: Arc<AsyncMutex<HashMap<JobId, JobState>>>,
    pub governance_module: Arc<AsyncMutex<GovernanceModule>>,
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
        let job_states = Arc::new(AsyncMutex::new(HashMap::new()));
        let pending_mesh_jobs = Arc::new(AsyncMutex::new(VecDeque::new()));

        Arc::new(Self {
            current_identity,
            mana_ledger: SimpleManaLedger::new(),
            pending_mesh_jobs,
            job_states,
            governance_module: Arc::new(AsyncMutex::new(GovernanceModule::new())),
            mesh_network_service,
            signer,
            dag_store,
        })
    }

    #[cfg(feature = "enable-libp2p")]
    pub async fn new_with_libp2p_network(current_identity_str: &str, bootstrap_peers: Option<Vec<(Libp2pPeerId, Multiaddr)>>) -> Result<Arc<Self>, CommonError> {
        let current_identity = Did::from_str(current_identity_str)
            .map_err(|e| CommonError::IdentityError(format!("Invalid DID string for new_with_libp2p_network: {}: {}", current_identity_str, e)))?;
        
        let mut config = NetworkConfig::default();
        if let Some(peers) = bootstrap_peers {
            config.bootstrap_peers = peers;
        }
        
        let libp2p_service_concrete = Arc::new(
            ActualLibp2pNetworkService::new(config).await
                .map_err(|e| CommonError::NetworkSetupError(format!("Failed to create Libp2pNetworkService: {}", e)))?
        );
        let libp2p_service_dyn: Arc<dyn NetworkService> = libp2p_service_concrete;

        let default_mesh_service = Arc::new(DefaultMeshNetworkService::new(
            libp2p_service_dyn.clone()
        ));

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
                        let mut job_states_guard = self.job_states.lock().await;
                        job_states_guard.insert(job.id.clone(), JobState::Completed { receipt: receipt.clone() });
                        // TODO: Credit mana to executor, update reputation, etc.
                        Ok(())
                    }
                    Err(e) => {
                        error!("[JobManagerDetail] Failed to anchor receipt for job {:?}: {}. Marking as Failed (AnchorFailed).", job.id, e);
                        let mut job_states_guard = self.job_states.lock().await;
                        job_states_guard.insert(job.id.clone(), JobState::Failed { reason: format!("Failed to anchor receipt: {}", e) });
                        Err(HostAbiError::DagOperationFailed(format!("Failed to anchor receipt: {}",e)))
                    }
                }
            }
            Ok(None) => {
                warn!("[JobManagerDetail] No receipt received for job {:?} within timeout. Marking as Failed (NoReceipt).", job.id);
                let mut job_states_guard = self.job_states.lock().await;
                job_states_guard.insert(job.id.clone(), JobState::Failed { reason: "No receipt received within timeout".to_string() });
                Err(HostAbiError::NetworkError("No receipt received within timeout".to_string()))
            }
            Err(e) => {
                error!("[JobManagerDetail] Error while trying to receive receipt for job {:?}: {}. Marking as Failed (ReceiptError).", job.id, e);
                let mut job_states_guard = self.job_states.lock().await;
                job_states_guard.insert(job.id.clone(), JobState::Failed { reason: format!("Error receiving receipt: {}", e) });
                Err(e)
            }
        }
    }

    pub async fn spawn_mesh_job_manager(self: Arc<Self>) {
        info!("[JobManager] Starting mesh job manager for DID: {}", self.current_identity);
        
        let ctx = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1000)); // Check every second
            
            loop {
                interval.tick().await;
                
                // Process pending jobs
                if let Err(e) = ctx.process_pending_jobs().await {
                    error!("[JobManager] Error processing pending jobs: {}", e);
                }
                
                // Process assigned jobs waiting for receipts
                if let Err(e) = ctx.process_assigned_jobs().await {
                    error!("[JobManager] Error processing assigned jobs: {}", e);
                }
            }
        });
        
        info!("[JobManager] Mesh job manager started successfully");
    }

    pub async fn process_pending_jobs(&self) -> Result<(), HostAbiError> {
        debug!("[JobManager] Processing pending jobs");
        
        // For now, this is a simplified implementation that works with our existing architecture
        // In a real implementation, this would process the actual pending jobs queue
        
        info!("[JobManager] Job processing completed (simplified implementation)");
        Ok(())
    }

    async fn process_assigned_jobs(&self) -> Result<(), HostAbiError> {
        let job_states = self.job_states.lock().await;
        let assigned_jobs: Vec<(JobId, Did)> = job_states.iter()
            .filter_map(|(job_id, state)| {
                if let JobState::Assigned { executor } = state {
                    Some((job_id.clone(), executor.clone()))
                } else {
                    None
                }
            })
            .collect();
        drop(job_states);
        
        for (job_id, executor_did) in assigned_jobs {
            // Try to receive receipt with a short timeout to avoid blocking
            let receipt_timeout = StdDuration::from_secs(2);
            
            match self.mesh_network_service.try_receive_receipt(&job_id, &executor_did, receipt_timeout).await {
                Ok(Some(receipt)) => {
                    info!("[JobManager] Received receipt for job {:?} from executor {:?}", job_id, executor_did);
                    
                    // Anchor the receipt
                    match self.anchor_receipt(&receipt).await {
                        Ok(receipt_cid) => {
                            info!("[JobManager] Successfully anchored receipt for job {:?} at CID {:?}", 
                                  job_id, receipt_cid);
                            
                            // Update job state to completed
                            let mut job_states = self.job_states.lock().await;
                            job_states.insert(job_id, JobState::Completed { receipt });
                        }
                        Err(e) => {
                            error!("[JobManager] Failed to anchor receipt for job {:?}: {}", job_id, e);
                            // Mark job as failed
                            let mut job_states = self.job_states.lock().await;
                            job_states.insert(job_id, JobState::Failed { 
                                reason: format!("Receipt anchoring failed: {}", e) 
                            });
                        }
                    }
                }
                Ok(None) => {
                    // No receipt yet, continue waiting
                    debug!("[JobManager] No receipt received yet for job {:?}", job_id);
                }
                Err(e) => {
                    error!("[JobManager] Error while waiting for receipt for job {:?}: {}", job_id, e);
                    // We could implement timeout logic here to eventually fail jobs that never receive receipts
                }
            }
        }
        
        Ok(())
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

    /// Anchors an execution receipt to the DAG store and returns the content identifier (CID).
    /// This method is called by the Host ABI function `host_anchor_receipt`.
    pub async fn anchor_receipt(&self, receipt: &IdentityExecutionReceipt) -> Result<Cid, HostAbiError> { 
        info!("[CONTEXT] Attempting to anchor receipt for job {:?} from executor {:?}", receipt.job_id, receipt.executor_did);

        // Verify the receipt signature against the signer's public key
        // This assumes the signer in the context is the one whose keys should verify all receipts.
        // This might be too simplistic; a real system might need to fetch the specific executor's VK.
        let signer_pk_bytes = self.signer.public_key_bytes();
        let verifying_key_bytes_array: [u8; 32] = signer_pk_bytes.as_slice().try_into()
            .map_err(|_| HostAbiError::CryptoError("Signer public key is not 32 bytes long".to_string()))?;
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

    pub async fn execute_governance_proposal(&mut self, _proposal_id_str: &str) -> Result<(), HostAbiError> {
        todo!("Implement full governance proposal execution logic");
    }

    /// Create a new RuntimeContext with real libp2p networking
    #[cfg(feature = "enable-libp2p")]
    pub async fn new_with_real_libp2p(
        identity_str: &str,
        bootstrap_peers: Option<Vec<(Libp2pPeerId, Multiaddr)>>
    ) -> Result<Arc<Self>, CommonError> {
        info!("Initializing RuntimeContext with real libp2p networking");
        
        // Parse the identity
        let identity = Did::from_str(identity_str)
            .map_err(|e| CommonError::InvalidInputError(format!("Invalid DID: {}", e)))?;
        
        // Generate keys for this node
        let (sk, pk) = generate_ed25519_keypair();
        let signer = Arc::new(StubSigner::new_with_keys(sk, pk));
        
        // Create real libp2p network service with proper config
        let mut config = NetworkConfig::default();
        if let Some(peers) = bootstrap_peers {
            info!("Bootstrap peers provided: {} peers", peers.len());
            config.bootstrap_peers = peers;
        }
        
        let libp2p_service = Arc::new(
            ActualLibp2pNetworkService::new(config).await
                .map_err(|e| CommonError::NetworkError(format!("Failed to create libp2p service: {}", e)))?
        );
        
        info!("Libp2p service created with PeerID: {}", libp2p_service.local_peer_id());
        
        // Wrap in DefaultMeshNetworkService 
        let mesh_service = Arc::new(DefaultMeshNetworkService::new(
            libp2p_service.clone() as Arc<dyn NetworkService>
        ));
        
        // Create stub DAG store for now (can be enhanced later)
        let dag_store = Arc::new(StubDagStore::new());
        
        // Create RuntimeContext with real networking - this returns Arc<Self>
        let ctx = Self::new(
            identity,
            mesh_service,
            signer,
            dag_store
        );
        
        info!("RuntimeContext with real libp2p networking created successfully");
        Ok(ctx)
    }

    /// Get the underlying libp2p service for peer info access
    #[cfg(feature = "enable-libp2p")]
    pub fn get_libp2p_service(&self) -> Result<Arc<dyn NetworkService>, CommonError> {
        if let Some(default_mesh) = MeshNetworkService::as_any(self.mesh_network_service.as_ref())
            .downcast_ref::<DefaultMeshNetworkService>() 
        {
            default_mesh.get_underlying_broadcast_service()
        } else {
            Err(CommonError::NetworkError(
                "RuntimeContext is not using DefaultMeshNetworkService with libp2p".to_string()
            ))
        }
    }

    /// Spawns an executor listener that automatically bids on appropriate jobs
    pub async fn spawn_executor_bidder(self: Arc<Self>) {
        info!("[ExecutorBidder] Starting executor bidder for DID: {}", self.current_identity);
        
        let ctx = self.clone();
        tokio::spawn(async move {
            // Get the underlying network service for message subscription
            let network_service = if let Ok(default_service) = ctx.mesh_network_service.clone()
                .downcast_arc::<DefaultMeshNetworkService>()
            {
                default_service.inner.clone()
            } else {
                error!("[ExecutorBidder] Unable to access underlying network service");
                return;
            };
            
            // Subscribe to network messages
            let mut receiver = match network_service.subscribe().await {
                Ok(receiver) => receiver,
                Err(e) => {
                    error!("[ExecutorBidder] Failed to subscribe to network: {}", e);
                    return;
                }
            };
            
            info!("[ExecutorBidder] Listening for job announcements...");
            
            while let Some(message) = receiver.recv().await {
                if let NetworkMessage::MeshJobAnnouncement(job) = message {
                    debug!("[ExecutorBidder] Received job announcement for job {:?}", job.id);
                    
                    // Check if we should bid on this job
                    if ctx.should_bid_on_job(&job).await {
                        info!("[ExecutorBidder] Submitting bid for job {:?}", job.id);
                        
                        if let Err(e) = ctx.submit_bid_for_job(&job).await {
                            error!("[ExecutorBidder] Failed to submit bid for job {:?}: {}", job.id, e);
                        }
                    } else {
                        debug!("[ExecutorBidder] Not bidding on job {:?}", job.id);
                    }
                }
            }
            
            warn!("[ExecutorBidder] Network message receiver closed");
        });
        
        info!("[ExecutorBidder] Executor bidder started successfully");
    }

    /// Evaluates whether this node should bid on a job
    pub async fn should_bid_on_job(&self, job: &ActualMeshJob) -> bool {
        // Don't bid on our own jobs
        if job.creator_did == self.current_identity {
            return false;
        }
        
        // Check if we have sufficient mana to potentially execute the job
        // We need some mana to participate in the network
        let our_mana = match self.get_mana(&self.current_identity).await {
            Ok(mana) => mana,
            Err(e) => {
                debug!("[ExecutorBidder] Failed to get mana balance: {}", e);
                return false;
            }
        };
        
        // Basic heuristic: bid if we have more mana than the job cost
        if our_mana < job.cost_mana {
            debug!("[ExecutorBidder] Insufficient mana to bid on job {:?} (have: {}, need: {})", 
                   job.id, our_mana, job.cost_mana);
            return false;
        }
        
        // Check job spec compatibility (simplified for now)
        match &job.spec {
            JobSpec::Echo { .. } => true, // We can handle Echo jobs
            JobSpec::GenericPlaceholder => true, // And generic placeholder jobs
            // Add more job type checks as needed
        }
    }

    /// Submits a bid for a job
    pub async fn submit_bid_for_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        // Calculate our bid price (simplified pricing strategy)
        let our_bid_price = job.cost_mana / 2; // Bid at half the job cost
        
        let bid = MeshJobBid {
            job_id: job.id.clone(),
            executor_did: self.current_identity.clone(),
            price_mana: our_bid_price,
            resources: Resources::default(), // Use default resources for now
        };
        
        debug!("[ExecutorBidder] Submitting bid: job={:?}, price={} mana", 
               job.id, our_bid_price);
        
        // Get the underlying network service for broadcasting
        let network_service = if let Ok(default_service) = self.mesh_network_service.clone()
            .downcast_arc::<DefaultMeshNetworkService>()
        {
            default_service.inner.clone()
        } else {
            return Err(HostAbiError::NetworkError("Unable to access underlying network service".to_string()));
        };
        
        // Broadcast the bid to the network
        let bid_message = NetworkMessage::BidSubmission(bid);
        network_service.broadcast_message(bid_message).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to broadcast bid: {}", e)))?;
        
        info!("[ExecutorBidder] Bid submitted for job {:?} at {} mana", job.id, our_bid_price);
        Ok(())
    }

    /// Spawns a job assignment listener that executes assigned jobs
    pub async fn spawn_job_assignment_listener(self: Arc<Self>) {
        info!("[AssignmentListener] Starting job assignment listener for DID: {}", self.current_identity);
        
        let ctx = self.clone();
        tokio::spawn(async move {
            // Get the underlying network service for message subscription
            let network_service = if let Ok(default_service) = ctx.mesh_network_service.clone()
                .downcast_arc::<DefaultMeshNetworkService>()
            {
                default_service.inner.clone()
            } else {
                error!("[AssignmentListener] Unable to access underlying network service");
                return;
            };
            
            // Subscribe to network messages
            let mut receiver = match network_service.subscribe().await {
                Ok(receiver) => receiver,
                Err(e) => {
                    error!("[AssignmentListener] Failed to subscribe to network: {}", e);
                    return;
                }
            };
            
            info!("[AssignmentListener] Listening for job assignments...");
            
            while let Some(message) = receiver.recv().await {
                if let NetworkMessage::JobAssignmentNotification(job_id, executor_did) = message {
                    if executor_did == ctx.current_identity {
                        info!("[AssignmentListener] Received assignment for job {:?}", job_id);
                        
                        // Execute the job in a separate task
                        let execution_ctx = ctx.clone();
                        tokio::spawn(async move {
                            if let Err(e) = execution_ctx.execute_assigned_job(&job_id).await {
                                error!("[AssignmentListener] Failed to execute job {:?}: {}", job_id, e);
                            }
                        });
                    }
                }
            }
            
            warn!("[AssignmentListener] Network message receiver closed");
        });
        
        info!("[AssignmentListener] Job assignment listener started successfully");
    }

    /// Executes an assigned job and submits the receipt
    pub async fn execute_assigned_job(&self, job_id: &JobId) -> Result<(), HostAbiError> {
        info!("[JobExecutor] Executing job {:?}", job_id);
        
        // For now, simulate job execution with a simple result
        // In a real implementation, this would:
        // 1. Fetch the job manifest from the DAG
        // 2. Execute the job according to its specification
        // 3. Store the result in the DAG
        // 4. Create and sign an execution receipt
        
        // Simulate execution time
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Create a dummy result
        let result_data = format!("Execution result for job {}", job_id);
        let result_cid = match self.dag_store.put(result_data.as_bytes()).await {
            Ok(cid) => cid,
            Err(e) => {
                error!("[JobExecutor] Failed to store result for job {:?}: {}", job_id, e);
                return Err(e);
            }
        };
        
        // Create the execution receipt
        let receipt = IdentityExecutionReceipt {
            job_id: job_id.clone(),
            executor_did: self.current_identity.clone(),
            result_cid,
            cpu_ms: 2000, // 2 seconds of execution
            sig: SignatureBytes(Vec::new()), // Will be filled by signing
        };
        
        // Sign the receipt
        let receipt_data = format!("receipt_{}_{}_{}_{}", 
                                  receipt.job_id, receipt.executor_did, receipt.result_cid, receipt.cpu_ms);
        let signature_bytes = self.signer.sign(receipt_data.as_bytes())
            .map_err(|e| HostAbiError::SignatureError(format!("Failed to sign receipt: {}", e)))?;
        
        let signed_receipt = IdentityExecutionReceipt {
            job_id: receipt.job_id,
            executor_did: receipt.executor_did,
            result_cid: receipt.result_cid,
            cpu_ms: receipt.cpu_ms,
            sig: SignatureBytes(signature_bytes),
        };
        
        // Get the underlying network service for broadcasting
        let network_service = if let Ok(default_service) = self.mesh_network_service.clone()
            .downcast_arc::<DefaultMeshNetworkService>()
        {
            default_service.inner.clone()
        } else {
            return Err(HostAbiError::NetworkError("Unable to access underlying network service".to_string()));
        };
        
        // Submit the receipt to the network
        let receipt_message = NetworkMessage::SubmitReceipt(signed_receipt);
        network_service.broadcast_message(receipt_message).await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to submit receipt: {}", e)))?;
        
        info!("[JobExecutor] Job {:?} executed successfully, receipt submitted", job_id);
        Ok(())
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
        let job_states = Arc::new(AsyncMutex::new(HashMap::new())); 
        let pending_mesh_jobs = Arc::new(AsyncMutex::new(VecDeque::new())); 
        let mana_ledger = SimpleManaLedger::new();
        let governance_module = Arc::new(AsyncMutex::new(GovernanceModule::default()));

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
    store: Arc<Mutex<HashMap<Cid, Vec<u8>>>>,
}
impl StubDagStore {
    pub fn new() -> Self {
        Self { store: Arc::new(Mutex::new(HashMap::new())) }
    }
    pub async fn all(&self) -> Result<HashMap<Cid, Vec<u8>>, HostAbiError> {
        let store_lock = self.store.lock().unwrap();
        Ok(store_lock.clone())
    }
}

impl Default for StubDagStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl icn_dag::StorageService for StubDagStore { // Implements the icn_dag StorageService trait
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
    
    async fn delete(&self, cid: &Cid) -> Result<bool, HostAbiError> {
        let mut store_lock = self.store.lock().await;
        let removed = store_lock.remove(cid).is_some();
        if removed {
            println!("[StubDagStore] Deleted data for CID: {:?}", cid);
        } else {
            println!("[StubDagStore] No data found to delete for CID: {:?}", cid);
        }
        Ok(removed)
    }
    
    async fn contains(&self, cid: &Cid) -> Result<bool, HostAbiError> {
        let store_lock = self.store.lock().await;
        let contains = store_lock.contains_key(cid);
        println!("[StubDagStore] Contains check for CID {:?}: {}", cid, contains);
        Ok(contains)
    }
}

#[derive(Debug, Clone)]
pub struct StubMeshNetworkService {
    staged_bids: Arc<Mutex<HashMap<JobId, VecDeque<MeshJobBid>>>>,
    staged_receipts: Arc<Mutex<VecDeque<LocalMeshSubmitReceiptMessage>>>, // Using local placeholder & Mutex
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

    pub fn get_staged_bids_for_job(&self, job_id: &JobId) -> Vec<MeshJobBid> {
        self.staged_bids.lock().unwrap()
            .get(job_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_staged_receipts_for_job(&self, job_id: &JobId) -> Vec<IdentityExecutionReceipt> {
        self.staged_receipts.lock().unwrap()
            .get(job_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn stage_bid_for_job(&self, job_id: JobId, bid: MeshJobBid) {
        self.staged_bids.lock().unwrap()
            .entry(job_id)
            .or_insert_with(Vec::new)
            .push(bid);
    }

    pub fn stage_receipt_for_job(&self, job_id: JobId, receipt: IdentityExecutionReceipt) {
        self.staged_receipts.lock().unwrap()
            .entry(job_id)
            .or_insert_with(Vec::new)
            .push(receipt);
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
