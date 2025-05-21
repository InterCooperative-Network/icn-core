//! Defines the `RuntimeContext`, `HostEnvironment`, and related types for the ICN runtime.

use icn_common::{Did, Cid, CommonError}; // Removed JobId as CommonJobId
use std::collections::{HashMap, VecDeque}; // Added HashMap for ManaLedger
use std::str::FromStr; // For Did::from_str in new_with_dummy_mana
use std::sync::atomic::{AtomicU32, Ordering}; // Modified import for Ordering
use icn_governance::GovernanceModule; // Assuming this can be imported
use icn_mesh::{ActualMeshJob, MeshJobBid, MeshJobAnnounce, MeshBidSubmit, select_executor, SelectionPolicy, JobId, JobAssignmentNotice, JobState, SubmitReceiptMessage}; // Import from icn-mesh
use icn_economics::{charge_mana, EconError}; // For mana charging
use icn_identity::ExecutionReceipt as IdentityExecutionReceipt; // Import and alias ExecutionReceipt
use tokio::sync::{mpsc, Mutex}; // For channel-based communication if needed and Mutex for shared state
use tokio::time::{sleep, Duration}; // For timeouts
use std::sync::Arc; // For shared state
use async_trait::async_trait; // For async traits

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
pub trait DagStore: Send + Sync + std::fmt::Debug {
    /// Stores a block of data and returns its CID.
    fn put(&self, data: &[u8]) -> Result<Cid, HostAbiError>;
    /// Retrieves a block of data by its CID.
    fn get(&self, cid: &Cid) -> Result<Option<Vec<u8>>, HostAbiError>;
}

/// Trait for a service that can broadcast and receive mesh-specific messages.
#[async_trait]
pub trait MeshNetworkService: Send + Sync + std::fmt::Debug {
    async fn announce_job(&self, announcement: MeshJobAnnounce) -> Result<(), HostAbiError>;
    async fn collect_bids_for_job(&self, job_id: JobId, duration: Duration) -> Result<Vec<MeshJobBid>, HostAbiError>;
    /// Broadcasts the job assignment to the selected executor (and potentially other listeners).
    async fn broadcast_assignment(&self, notice: JobAssignmentNotice) -> Result<(), HostAbiError>;
    /// Attempts to receive a submitted execution receipt (non-blocking).
    async fn try_receive_receipt(&self) -> Result<Option<SubmitReceiptMessage>, HostAbiError>;
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
    InternalError(String),
    Common(CommonError), 
}

impl std::fmt::Display for HostAbiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostAbiError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            HostAbiError::InsufficientMana => write!(f, "Insufficient mana"),
            HostAbiError::AccountNotFound(did) => write!(f, "Account not found: {:?}", did),
            HostAbiError::JobSubmissionFailed(msg) => write!(f, "Job submission failed: {}", msg),
            HostAbiError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
            HostAbiError::DagOperationFailed(msg) => write!(f, "DAG operation failed: {}", msg),
            HostAbiError::SignatureError(msg) => write!(f, "Signature error: {}", msg),
            HostAbiError::InternalError(msg) => write!(f, "Internal runtime error: {}", msg),
            HostAbiError::Common(e) => write!(f, "Common error: {}", e),
        }
    }
}

impl std::error::Error for HostAbiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HostAbiError::Common(_) => None,
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
    balances: HashMap<Did, u64>,
}

impl SimpleManaLedger {
    pub fn new() -> Self {
        Self { balances: HashMap::new() }
    }

    pub fn get_balance(&self, account: &Did) -> Option<u64> {
        self.balances.get(account).cloned()
    }

    pub fn set_balance(&mut self, account: &Did, amount: u64) {
        self.balances.insert(account.clone(), amount);
    }

    pub fn spend(&mut self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        let balance = self.balances.get_mut(account).ok_or_else(|| HostAbiError::AccountNotFound(account.clone()))?;
        if *balance < amount {
            return Err(HostAbiError::InsufficientMana);
        }
        *balance -= amount;
        Ok(())
    }

    pub fn credit(&mut self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        let balance = self.balances.entry(account.clone()).or_insert(0);
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
        Self {
            current_identity,
            mana_ledger: SimpleManaLedger::new(),
            pending_mesh_jobs: Arc::new(Mutex::new(VecDeque::new())), 
            job_states: Arc::new(Mutex::new(HashMap::new())),
            governance_module: GovernanceModule::new(),
            mesh_network_service,
            signer,
            dag_store,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_stubs(current_identity_str: &str) -> Self {
        let current_identity = Did::from_str(current_identity_str).expect("Invalid DID for test context");
        Self::new(
            current_identity, 
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner),
            Arc::new(StubDagStore::new())
        )
    }

    #[cfg(test)]
    pub(crate) fn new_with_stubs_and_mana(current_identity_str: &str, initial_mana: u64) -> Self {
        let current_identity = Did::from_str(current_identity_str).expect("Invalid DID for test context");
        let mut ctx = Self::new(
            current_identity.clone(), 
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner),
            Arc::new(StubDagStore::new())
        );
        ctx.mana_ledger.set_balance(&current_identity, initial_mana);
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
        let mana_ledger_for_refunds = Arc::new(Mutex::new(self.mana_ledger.clone())); // Clone mana ledger for JobManager refunds

        let mut assigned_jobs: HashMap<JobId, (ActualMeshJob, Did, std::time::Instant)> = HashMap::new();
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
                                assigned_jobs.insert(job_to_assign.id.clone(), (job_to_assign.clone(), selected_executor_did.clone(), std::time::Instant::now()));
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
                            let mut temp_anchor_ctx = RuntimeContext::new(
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
                        let mut ledger = mana_ledger_for_refunds.lock().await;
                        match ledger.credit(&timed_out_job.creator_did, timed_out_job.cost_mana) {
                            Ok(_) => println!("[JobManager] Refunded {} mana to submitter {:?} for timed out job {:?}", 
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

    pub fn get_mana(&self, account: &Did) -> Result<u64, HostAbiError> {
        println!("[CONTEXT] get_mana called for account: {:?}", account);
        self.mana_ledger.get_balance(account).ok_or_else(|| HostAbiError::AccountNotFound(account.clone()))
    }

    pub fn spend_mana(&mut self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        println!("[CONTEXT] spend_mana called for account: {:?} amount: {}", account, amount);
        if account != &self.current_identity {
            return Err(HostAbiError::InvalidParameters(
                "Attempting to spend mana for an account other than the current context identity.".to_string(),
            ));
        }
        self.mana_ledger.spend(account, amount)
    }

    pub fn credit_mana(&mut self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        println!("[CONTEXT] credit_mana called for account: {:?} amount: {}", account, amount);
        // Policy: For now, allow crediting any account, as this might be done by system processes (e.g. refunds)
        // If only self-crediting is allowed, add: if account != &self.current_identity { return Err(...) }
        self.mana_ledger.credit(account, amount)
    }

    pub fn anchor_receipt(&self, receipt: &mut IdentityExecutionReceipt) -> Result<Cid, HostAbiError> { 
        println!("[CONTEXT] anchor_receipt called for job_id: {:?}", receipt.job_id);

        if receipt.executor_did != self.current_identity {
            return Err(HostAbiError::InvalidParameters(
                "Receipt executor_did does not match current context identity.".to_string(),
            ));
        }
        
        let sighash_data = format!("{:?}|{:?}|{:?}|{}", receipt.job_id, receipt.executor_did, receipt.result_cid, receipt.cpu_ms);

        if receipt.sig.is_empty() {
            let signature = self.signer.sign(&self.current_identity, sighash_data.as_bytes())?;
            receipt.sig = signature;
            println!("[CONTEXT] Receipt signed with new signature.");
        } else {
            let is_valid = self.signer.verify(&self.current_identity, sighash_data.as_bytes(), &receipt.sig)?;
            if !is_valid {
                return Err(HostAbiError::SignatureError("Provided signature is invalid.".to_string()));
            }
            println!("[CONTEXT] Existing signature on receipt verified.");
        }

        let receipt_bytes = serde_json::to_vec(receipt)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to serialize receipt: {}", e)))?;
        
        let cid = self.dag_store.put(&receipt_bytes)?;
        println!("[CONTEXT] Receipt anchored to DAG with CID: {:?}", cid);
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
pub struct StubMeshNetworkService {}
impl StubMeshNetworkService { pub fn new() -> Self { Self {} } }

#[async_trait]
impl MeshNetworkService for StubMeshNetworkService {
    async fn announce_job(&self, announcement: MeshJobAnnounce) -> Result<(), HostAbiError> {
        println!("[StubMeshNetworkService] Announced job: {:?}", announcement.job_id);
        Ok(())
    }

    async fn collect_bids_for_job(&self, job_id: JobId, duration: Duration) -> Result<Vec<MeshJobBid>, HostAbiError> {
        println!("[StubMeshNetworkService] Collecting bids for job {:?} for {:?}. STUB: Returning empty vec.", job_id, duration);
        Ok(Vec::new()) 
    }

    async fn broadcast_assignment(&self, notice: JobAssignmentNotice) -> Result<(), HostAbiError> {
        println!("[StubMeshNetworkService] Broadcast assignment for job {:?} to executor {:?}", notice.job_id, notice.executor_did);
        Ok(())
    }

    async fn try_receive_receipt(&self) -> Result<Option<SubmitReceiptMessage>, HostAbiError> {
        println!("[StubMeshNetworkService] try_receive_receipt called. STUB: Returning None.");
        Ok(None)
    }
} 