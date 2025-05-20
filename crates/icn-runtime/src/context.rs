//! Defines the `RuntimeContext`, `HostEnvironment`, and related types for the ICN runtime.

use icn_common::{Did, Cid, CommonError}; // Assuming Did, Cid might be needed. Adjust as necessary.
use std::collections::{HashMap, VecDeque}; // Added HashMap for ManaLedger
use std::str::FromStr; // For Did::from_str in new_with_dummy_mana
use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering}; // Renamed Ordering to avoid conflict if any, and for clarity

// Counter for generating unique (within this runtime instance) job IDs for stubs
static NEXT_JOB_ID: AtomicU32 = AtomicU32::new(1);

// --- Placeholder Types --- 
// TODO: Replace these with actual types from their respective crates (e.g., icn-mesh, icn-dag)

/// Placeholder for a job specification submitted to the mesh.
#[derive(Debug, Clone)]
pub struct MeshJob {
    pub id: String, // A unique identifier for the job spec itself
    pub data: Vec<u8>, // Serialized job data
    pub owner: Did,    // Submitter of the job
}

/// Placeholder for a Job ID returned by the mesh network.
#[derive(Debug, Clone, PartialEq, Eq, Hash)] // Corrected deriveDebug
pub struct JobId(pub String);

// --- Host ABI Error --- 

/// Errors that can occur during Host ABI function calls via the RuntimeContext.
#[derive(Debug)]
pub enum HostAbiError {
    NotImplemented(String),
    InsufficientMana,
    AccountNotFound(Did),
    JobSubmissionFailed(String),
    InvalidParameters(String),
    DagOperationFailed(String),
    InternalError(String),
    Common(CommonError), // For conversion
}

// Implement std::fmt::Display for HostAbiError
impl std::fmt::Display for HostAbiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostAbiError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            HostAbiError::InsufficientMana => write!(f, "Insufficient mana"),
            HostAbiError::AccountNotFound(did) => write!(f, "Account not found: {:?}", did),
            HostAbiError::JobSubmissionFailed(msg) => write!(f, "Job submission failed: {}", msg),
            HostAbiError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
            HostAbiError::DagOperationFailed(msg) => write!(f, "DAG operation failed: {}", msg),
            HostAbiError::InternalError(msg) => write!(f, "Internal runtime error: {}", msg),
            HostAbiError::Common(e) => write!(f, "Common error: {}", e),
        }
    }
}

impl std::error::Error for HostAbiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HostAbiError::Common(_) => None, // Corrected: CommonError may not be std::error::Error
            // TODO: If CommonError is made to implement std::error::Error, this can be Some(e).
            _ => None, 
        }
    }
}

// Central Error Conversion: From CommonError to HostAbiError
impl From<CommonError> for HostAbiError {
    fn from(err: CommonError) -> Self {
        HostAbiError::Common(err)
    }
}


// --- Mana Ledger (Simple In-Memory Version) ---
// TODO: Replace with `ManaRepositoryAdapter` + `SledManaLedger` integration when ready.
#[derive(Debug)]
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
}

// --- Runtime Context --- 

/// `RuntimeContext` manages the state and capabilities available to an executing
/// ICN script or WASM module. It provides scoped access to identity, mana,
/// mesh job submission, DAG anchoring, etc.
#[derive(Debug)]
pub struct RuntimeContext {
    /// The DID of the identity currently executing within this context.
    pub current_identity: Did,
    /// Simple in-memory mana ledger for now.
    pub mana_ledger: SimpleManaLedger, 
    /// Queue for jobs submitted by this context to the mesh network.
    pub pending_mesh_jobs: VecDeque<MeshJob>,
    // TODO: Placeholder for DAG store access
    // pub dag_store: Arc<dyn DagStoreAccess>,
    // TODO: Add fields for policy enforcers, etc.
}

impl RuntimeContext {
    /// Creates a new `RuntimeContext` for a given identity.
    /// Initializes with an empty mana ledger.
    pub fn new(current_identity: Did) -> Self {
        Self {
            current_identity,
            mana_ledger: SimpleManaLedger::new(),
            pending_mesh_jobs: VecDeque::new(),
        }
    }

    /// Test helper to create a context with some initial mana for the current identity.
    #[cfg(test)]
    pub(crate) fn new_with_initial_mana(current_identity_str: &str, initial_mana: u64) -> Self {
        let current_identity = Did::from_str(current_identity_str).expect("Invalid DID for test context");
        let mut ctx = Self::new(current_identity.clone());
        ctx.mana_ledger.set_balance(&current_identity, initial_mana);
        ctx
    }

    /// Submits a mesh job from the current runtime context.
    pub fn submit_mesh_job(&mut self, job_data: Vec<u8>) -> Result<JobId, HostAbiError> {
        let job_id_val = NEXT_JOB_ID.fetch_add(1, AtomicOrdering::SeqC);
        let job_id_str = format!("job_{}", job_id_val);
        let job = MeshJob {
            id: job_id_str.clone(), 
            data: job_data,
            owner: self.current_identity.clone(),
        };
        self.pending_mesh_jobs.push_back(job.clone());
        println!("[CONTEXT_STUB] submit_mesh_job called for job: {:?}, owner: {:?}", job.id, self.current_identity);
        Ok(JobId(job_id_str))
    }

    /// Retrieves the mana for the given account.
    pub fn get_mana(&self, account: &Did) -> Result<u64, HostAbiError> {
        println!("[CONTEXT] get_mana called for account: {:?}", account);
        self.mana_ledger.get_balance(account).ok_or_else(|| HostAbiError::AccountNotFound(account.clone()))
    }

    /// Spends mana from the given account.
    pub fn spend_mana(&mut self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        println!("[CONTEXT] spend_mana called for account: {:?} amount: {}", account, amount);
        if account != &self.current_identity {
            return Err(HostAbiError::InvalidParameters(
                "Attempting to spend mana for an account other than the current context identity.".to_string(),
            ));
        }
        self.mana_ledger.spend(account, amount)
    }

    // TODO: Add other methods for DAG anchoring, policy checks, etc.
}

// --- Host Environment Trait and Implementation ---

/// The `HostEnvironment` trait defines the interface that a WASM runtime (or other execution environment)
/// uses to interact with the host system capabilities, mediated by a `RuntimeContext`.
pub trait HostEnvironment {
    // TODO: Define methods that the WASM runtime will call.
    // These will typically be wrappers around RuntimeContext methods, adapting types as needed.
    // fn call_host_abi(&mut self, context: &mut RuntimeContext, abi_index: u32, args: &[u8]) -> Result<Vec<u8>, HostAbiError>;

    // Example of a more specific function if not using a generic dispatcher:
    fn env_submit_mesh_job(&self, ctx: &mut RuntimeContext, job_data_ptr: u32, job_data_len: u32) -> Result<u32, HostAbiError>; // Returns JobId_ptr
    fn env_account_get_mana(&self, ctx: &RuntimeContext, account_did_ptr: u32, account_did_len: u32) -> Result<u64, HostAbiError>;
    fn env_account_spend_mana(&self, ctx: &mut RuntimeContext, account_did_ptr: u32, account_did_len: u32, amount: u64) -> Result<(), HostAbiError>;
}

/// `ConcreteHostEnvironment` is an example implementation of the `HostEnvironment` trait.
/// It would typically hold references to system services or configurations needed to
/// fulfill the host functions.
pub struct ConcreteHostEnvironment {
    // Example: configuration, shared services, etc.
    // pub mesh_client: Arc<MeshClient>,
    // pub mana_service: Arc<ManaService>,
}

impl ConcreteHostEnvironment {
    pub fn new() -> Self {
        // TODO: Initialize with actual services
        Self {}
    }
}

impl Default for ConcreteHostEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl HostEnvironment for ConcreteHostEnvironment {
    // TODO: Implement the trait methods.
    // These methods will likely involve:
    //  1. Reading/writing to the WASM module's memory (for pointers like job_data_ptr).
    //  2. Deserializing arguments from WASM.
    //  3. Calling the appropriate `RuntimeContext` method.
    //  4. Serializing results back to WASM and writing to its memory.

    fn env_submit_mesh_job(&self, _ctx: &mut RuntimeContext, _job_data_ptr: u32, _job_data_len: u32) -> Result<u32, HostAbiError> {
        println!("[CONCRETE_HOST_ENV_STUB] env_submit_mesh_job called");
        todo!("ConcreteHostEnvironment::env_submit_mesh_job");
    }

    fn env_account_get_mana(&self, _ctx: &RuntimeContext, _account_did_ptr: u32, _account_did_len: u32) -> Result<u64, HostAbiError> {
        println!("[CONCRETE_HOST_ENV_STUB] env_account_get_mana called");
        todo!("ConcreteHostEnvironment::env_account_get_mana");
    }

    fn env_account_spend_mana(&self, _ctx: &mut RuntimeContext, _account_did_ptr: u32, _account_did_len: u32, _amount: u64) -> Result<(), HostAbiError> {
        println!("[CONCRETE_HOST_ENV_STUB] env_account_spend_mana called");
        todo!("ConcreteHostEnvironment::env_account_spend_mana");
    }
} 