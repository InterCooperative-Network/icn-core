//! Defines the `RuntimeContext`, `HostEnvironment`, and related types for the ICN runtime.

use icn_common::{Cid, CommonError, DagBlock, Did};
use icn_identity::ExecutionReceipt as IdentityExecutionReceipt;
use icn_mesh::{ActualMeshJob, JobId, JobState, MeshJobBid};

use downcast_rs::{impl_downcast, DowncastSync};
#[cfg(feature = "enable-libp2p")]
use icn_network::libp2p_service::Libp2pNetworkService as ActualLibp2pNetworkService;
use icn_network::{NetworkMessage, NetworkService as ActualNetworkService};

#[cfg(not(any(
    feature = "persist-sled",
    feature = "persist-sqlite",
    feature = "persist-rocksdb"
)))]
use icn_economics::FileManaLedger;
#[cfg(all(
    not(feature = "persist-sled"),
    not(feature = "persist-sqlite"),
    feature = "persist-rocksdb"
))]
use icn_economics::RocksdbManaLedger;
#[cfg(feature = "persist-sled")]
use icn_economics::SledManaLedger;
#[cfg(all(not(feature = "persist-sled"), feature = "persist-sqlite"))]
use icn_economics::SqliteManaLedger;
use icn_economics::{EconError, ManaRepositoryAdapter};
use log::{debug, error, info, warn};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant as StdInstant};
#[cfg(test)]
use tempfile;
use tokio::sync::Mutex as TokioMutex;

use async_trait::async_trait;

use std::str::FromStr;

#[cfg(feature = "enable-libp2p")]
use icn_network::libp2p_service::NetworkConfig;
#[cfg(feature = "enable-libp2p")]
use libp2p::{Multiaddr, PeerId as Libp2pPeerId};

use bincode;
use icn_governance::{GovernanceModule, ProposalId, ProposalType, VoteOption};
#[cfg(feature = "enable-libp2p")]
use icn_identity::KeyDidResolver;
use icn_identity::{
    did_key_from_verifying_key, generate_ed25519_keypair, sign_message,
    verify_signature as identity_verify_signature, EdSignature, SigningKey, VerifyingKey,
    SIGNATURE_LENGTH,
};
use serde::{Deserialize, Serialize};

// Counter for generating unique (within this runtime instance) job IDs for stubs
pub static NEXT_JOB_ID: AtomicU32 = AtomicU32::new(1);

// --- Placeholder Local Stubs / Forward Declarations ---

// Updated Signer trait to be synchronous and match new crypto capabilities
// #[async_trait] // No longer async
pub trait Signer: Send + Sync + std::fmt::Debug {
    // async fn sign(&self, did: &Did, data: &[u8]) -> Result<Vec<u8>, HostAbiError>; // Old async version
    // async fn verify(&self, did: &Did, data: &[u8], signature: &[u8]) -> Result<bool, HostAbiError>; // Old async version
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, HostAbiError>;
    fn verify(
        &self,
        payload: &[u8],
        signature: &[u8],
        public_key_bytes: &[u8],
    ) -> Result<bool, HostAbiError>; // Added pk_bytes
    fn public_key_bytes(&self) -> Vec<u8>;
    fn did(&self) -> Did;
    fn verifying_key_ref(&self) -> &VerifyingKey;
}

#[cfg(feature = "persist-rocksdb")]
use icn_dag::rocksdb_store::RocksDagStore;
#[cfg(not(any(
    feature = "persist-rocksdb",
    feature = "persist-sled",
    feature = "persist-sqlite"
)))]
use icn_dag::FileDagStore;
use icn_dag::StorageService as DagStorageService;

// Placeholder for icn_economics::ManaRepository
pub trait ManaRepository: Send + Sync + std::fmt::Debug {
    // Define methods as needed, e.g.:
    // async fn get_balance(&self, account: &Did) -> Result<u64, EconError>;
    // async fn spend_mana(&self, account: &Did, amount: u64) -> Result<(), EconError>;
    // async fn credit_mana(&self, account: &Did, amount: u64) -> Result<(), EconError>;
}

/// Simple wrapper around the selected `ManaLedger` implementation for use inside the runtime.
#[derive(Clone)]
pub struct SimpleManaLedger {
    ledger: Arc<dyn icn_economics::ManaLedger>,
}

impl std::fmt::Debug for SimpleManaLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SimpleManaLedger")
    }
}

impl SimpleManaLedger {
    /// Create a new ledger at the given path. Panics if the ledger cannot be
    /// initialized.
    pub fn new(path: PathBuf) -> Self {
        #[cfg(feature = "persist-sled")]
        let ledger = Arc::new(
            SledManaLedger::new(path)
                .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
        ) as Arc<dyn icn_economics::ManaLedger>;
        #[cfg(all(not(feature = "persist-sled"), feature = "persist-sqlite"))]
        let ledger = Arc::new(
            icn_economics::SqliteManaLedger::new(path)
                .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
        ) as Arc<dyn icn_economics::ManaLedger>;
        #[cfg(all(
            not(feature = "persist-sled"),
            not(feature = "persist-sqlite"),
            feature = "persist-rocksdb"
        ))]
        let ledger = Arc::new(
            icn_economics::RocksdbManaLedger::new(path)
                .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
        ) as Arc<dyn icn_economics::ManaLedger>;
        #[cfg(not(any(
            feature = "persist-sled",
            feature = "persist-sqlite",
            feature = "persist-rocksdb"
        )))]
        let ledger = Arc::new(
            icn_economics::FileManaLedger::new(path)
                .unwrap_or_else(|e| panic!("Failed to create mana ledger: {e}")),
        ) as Arc<dyn icn_economics::ManaLedger>;
        Self { ledger }
    }

    pub fn get_balance(&self, account: &Did) -> u64 {
        self.ledger.get_balance(account)
    }

    pub fn set_balance(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        self.ledger
            .set_balance(account, amount)
            .map_err(HostAbiError::Common)
    }

    pub fn spend(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        match self.ledger.spend(account, amount) {
            Ok(()) => Ok(()),
            Err(EconError::InsufficientBalance(_)) => Err(HostAbiError::InsufficientMana),
            Err(e) => Err(HostAbiError::InternalError(format!("{e:?}"))),
        }
    }

    pub fn credit(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        self.ledger
            .credit(account, amount)
            .map_err(|e| HostAbiError::InternalError(format!("{e:?}")))
    }
}

impl icn_economics::ManaLedger for SimpleManaLedger {
    fn get_balance(&self, did: &Did) -> u64 {
        self.ledger.get_balance(did)
    }

    fn set_balance(&self, did: &Did, amount: u64) -> Result<(), icn_common::CommonError> {
        self.ledger.set_balance(did, amount)
    }

    fn spend(&self, did: &Did, amount: u64) -> Result<(), icn_economics::EconError> {
        self.ledger.spend(did, amount)
    }

    fn credit(&self, did: &Did, amount: u64) -> Result<(), icn_economics::EconError> {
        self.ledger.credit(did, amount)
    }
}

// Placeholder for icn_mesh::MeshJobStateChange
#[derive(Debug, Clone)]
pub struct MeshJobStateChange {/* ... fields ... */}
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
pub struct LocalMeshSubmitReceiptMessage {
    // Renamed to avoid conflict
    pub receipt: IdentityExecutionReceipt,
}

// GovernanceModule is provided by the icn-governance crate

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
pub struct SelectionPolicy {/* ... fields ... */}

// Placeholder for select_executor function (used in Job Manager)
// This would typically belong to the icn-mesh crate or a related module.
// fn select_executor(bids: Vec<MeshJobBid>, _policy: SelectionPolicy) -> Option<Did> {
// Simplistic: return the DID of the first bidder if any
// bids.first().map(|bid| bid.executor_did.clone())
// }
// --- End Placeholder Local Stubs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProposalPayload {
    pub proposal_type_str: String,
    pub type_specific_payload: Vec<u8>,
    pub description: String,
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastVotePayload {
    pub proposal_id_str: String,
    pub vote_option_str: String,
}

/// Trait for a service that can broadcast and receive mesh-specific messages.
/// This is the local definition for icn-runtime.
#[async_trait]
pub trait MeshNetworkService: Send + Sync + std::fmt::Debug + DowncastSync {
    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError>;
    async fn announce_proposal(&self, proposal_bytes: Vec<u8>) -> Result<(), HostAbiError>;
    async fn announce_vote(&self, vote_bytes: Vec<u8>) -> Result<(), HostAbiError>;
    async fn collect_bids_for_job(
        &self,
        job_id: &JobId,
        duration: StdDuration,
    ) -> Result<Vec<MeshJobBid>, HostAbiError>;
    async fn notify_executor_of_assignment(
        &self,
        notice: &JobAssignmentNotice,
    ) -> Result<(), HostAbiError>;
    async fn try_receive_receipt(
        &self,
        job_id: &JobId,
        expected_executor: &Did,
        timeout: StdDuration,
    ) -> Result<Option<IdentityExecutionReceipt>, HostAbiError>;
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
    pub fn get_underlying_broadcast_service(
        &self,
    ) -> Result<Arc<ActualLibp2pNetworkService>, CommonError> {
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
        self.inner
            .broadcast_message(job_message)
            .await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to announce job: {}", e)))
    }

    async fn announce_proposal(&self, proposal_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        let msg = NetworkMessage::ProposalAnnouncement(proposal_bytes);
        self.inner
            .broadcast_message(msg)
            .await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to announce proposal: {}", e)))
    }

    async fn announce_vote(&self, vote_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        let msg = NetworkMessage::VoteAnnouncement(vote_bytes);
        self.inner
            .broadcast_message(msg)
            .await
            .map_err(|e| HostAbiError::NetworkError(format!("Failed to announce vote: {}", e)))
    }

    async fn collect_bids_for_job(
        &self,
        job_id: &JobId,
        duration: StdDuration,
    ) -> Result<Vec<MeshJobBid>, HostAbiError> {
        debug!(
            "[DefaultMeshNetworkService] Collecting bids for job {:?} for {:?}",
            job_id, duration
        );
        let mut bids = Vec::new();
        let mut receiver = self.inner.subscribe().await.map_err(|e| {
            HostAbiError::NetworkError(format!("Failed to subscribe for bids: {}", e))
        })?;

        let end_time = StdInstant::now() + duration;

        loop {
            match tokio::time::timeout_at(tokio::time::Instant::from_std(end_time), receiver.recv())
                .await
            {
                Ok(result) => {
                    // Timeout gives Result<Option<T>, Elapsed>
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
                            debug!(
                                "Received other network message during bid collection: {:?}",
                                other_message
                            );
                        }
                        None => {
                            // Channel closed
                            warn!(
                                "Network channel closed while collecting bids for job {:?}",
                                job_id
                            );
                            break;
                        }
                    }
                }
                Err(_timeout_error) => {
                    // Timeout
                    debug!("Bid collection timeout for job {:?}", job_id);
                    break;
                }
            }
        }
        Ok(bids)
    }

    async fn notify_executor_of_assignment(
        &self,
        notice: &JobAssignmentNotice,
    ) -> Result<(), HostAbiError> {
        debug!(
            "[DefaultMeshNetworkService] Broadcasting assignment for job {:?}",
            notice.job_id
        );
        let assignment_message = NetworkMessage::JobAssignmentNotification(
            notice.job_id.clone(),
            notice.executor_did.clone(),
        );
        self.inner
            .broadcast_message(assignment_message)
            .await
            .map_err(|e| {
                HostAbiError::NetworkError(format!("Failed to broadcast assignment: {}", e))
            })
    }

    async fn try_receive_receipt(
        &self,
        job_id: &JobId,
        expected_executor: &Did,
        timeout_duration: StdDuration,
    ) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
        debug!("[DefaultMeshNetworkService] Waiting for receipt for job {:?} from executor {:?} for {:?}", job_id, expected_executor, timeout_duration);
        let mut receiver = self.inner.subscribe().await.map_err(|e| {
            HostAbiError::NetworkError(format!("Failed to subscribe for receipts: {}", e))
        })?;

        let end_time = StdInstant::now() + timeout_duration;

        loop {
            match tokio::time::timeout_at(tokio::time::Instant::from_std(end_time), receiver.recv())
                .await
            {
                Ok(result) => {
                    match result {
                        Some(NetworkMessage::SubmitReceipt(receipt)) => {
                            if &receipt.job_id == job_id
                                && &receipt.executor_did == expected_executor
                            {
                                debug!("Received relevant receipt: {:?}", receipt);
                                return Ok(Some(receipt));
                            } else {
                                debug!("Received receipt for different job/executor: job_id={:?}, executor={:?}", receipt.job_id, receipt.executor_did);
                            }
                        }
                        Some(other_message) => {
                            debug!(
                                "Received other network message during receipt collection: {:?}",
                                other_message
                            );
                        }
                        None => {
                            // Channel closed
                            warn!(
                                "Network channel closed while waiting for receipt for job {:?}",
                                job_id
                            );
                            break;
                        }
                    }
                }
                Err(_timeout_error) => {
                    // Timeout
                    debug!("Receipt wait timeout for job {:?}", job_id);
                    break;
                }
            }
        }
        Ok(None)
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
            HostAbiError::AccountNotFound(did) => {
                write!(f, "Account not found: {}", did.to_string())
            }
            HostAbiError::JobSubmissionFailed(msg) => write!(f, "Job submission failed: {}", msg),
            HostAbiError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
            HostAbiError::DagOperationFailed(msg) => write!(f, "DAG operation failed: {}", msg),
            HostAbiError::SignatureError(msg) => write!(f, "Signature error: {}", msg),
            HostAbiError::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            HostAbiError::WasmExecutionError(msg) => write!(f, "Wasm execution error: {}", msg),
            HostAbiError::ResourceLimitExceeded(msg) => {
                write!(f, "Resource limit exceeded: {}", msg)
            }
            HostAbiError::InvalidSystemApiCall(msg) => {
                write!(f, "Invalid system API call: {}", msg)
            }
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
pub struct RuntimeContext {
    pub current_identity: Did,
    pub mana_ledger: SimpleManaLedger,
    pub pending_mesh_jobs: Arc<TokioMutex<VecDeque<ActualMeshJob>>>,
    pub job_states: Arc<TokioMutex<HashMap<JobId, JobState>>>,
    pub governance_module: Arc<TokioMutex<GovernanceModule>>,
    pub mesh_network_service: Arc<dyn MeshNetworkService>, // Uses local MeshNetworkService trait
    pub signer: Arc<dyn Signer>,
    pub did_resolver: Arc<dyn icn_identity::DidResolver>,
    pub dag_store: Arc<TokioMutex<dyn DagStorageService<DagBlock> + Send>>, // Uses icn_dag::StorageService
    pub reputation_store: Arc<dyn icn_reputation::ReputationStore>,
    /// Default timeout in milliseconds when waiting for job execution receipts.
    pub default_receipt_wait_ms: u64,
}

impl RuntimeContext {
    /// Create a new context using a mana ledger stored at `mana_ledger_path`.
    pub fn new_with_ledger_path(
        current_identity: Did,
        mesh_network_service: Arc<dyn MeshNetworkService>,
        signer: Arc<dyn Signer>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
        dag_store: Arc<TokioMutex<dyn DagStorageService<DagBlock> + Send>>,
        mana_ledger_path: PathBuf,
        reputation_store_path: PathBuf,
    ) -> Arc<Self> {
        let job_states = Arc::new(TokioMutex::new(HashMap::new()));
        let pending_mesh_jobs = Arc::new(TokioMutex::new(VecDeque::new()));

        #[cfg(feature = "persist-sled")]
        let governance_module = Arc::new(TokioMutex::new(
            GovernanceModule::new_sled(std::path::PathBuf::from("./governance_db"))
                .unwrap_or_else(|_| GovernanceModule::new()),
        ));
        #[cfg(not(feature = "persist-sled"))]
        let governance_module = Arc::new(TokioMutex::new(GovernanceModule::new()));

        #[cfg(feature = "persist-sled")]
        let reputation_store: Arc<dyn icn_reputation::ReputationStore> =
            match icn_reputation::SledReputationStore::new(reputation_store_path) {
                Ok(s) => Arc::new(s),
                Err(_) => Arc::new(icn_reputation::InMemoryReputationStore::new()),
            };
        #[cfg(all(not(feature = "persist-sled"), feature = "persist-sqlite"))]
        let reputation_store: Arc<dyn icn_reputation::ReputationStore> =
            match icn_reputation::SqliteReputationStore::new(reputation_store_path) {
                Ok(s) => Arc::new(s),
                Err(_) => Arc::new(icn_reputation::InMemoryReputationStore::new()),
            };
        #[cfg(all(
            not(feature = "persist-sled"),
            not(feature = "persist-sqlite"),
            feature = "persist-rocksdb"
        ))]
        let reputation_store: Arc<dyn icn_reputation::ReputationStore> =
            match icn_reputation::RocksdbReputationStore::new(reputation_store_path) {
                Ok(s) => Arc::new(s),
                Err(_) => Arc::new(icn_reputation::InMemoryReputationStore::new()),
            };
        #[cfg(not(any(
            feature = "persist-sled",
            feature = "persist-sqlite",
            feature = "persist-rocksdb"
        )))]
        let reputation_store: Arc<dyn icn_reputation::ReputationStore> =
            Arc::new(icn_reputation::InMemoryReputationStore::new());

        Arc::new(Self {
            current_identity,
            mana_ledger: SimpleManaLedger::new(mana_ledger_path),
            pending_mesh_jobs,
            job_states,
            governance_module,
            mesh_network_service,
            signer,
            did_resolver,
            dag_store,
            reputation_store,
            default_receipt_wait_ms: 60_000,
        })
    }

    /// Create a new context using the default mana ledger path (`./mana_ledger.sled`).
    pub fn new(
        current_identity: Did,
        mesh_network_service: Arc<dyn MeshNetworkService>,
        signer: Arc<dyn Signer>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
        dag_store: Arc<TokioMutex<dyn DagStorageService<DagBlock> + Send>>,
    ) -> Arc<Self> {
        Self::new_with_ledger_path(
            current_identity,
            mesh_network_service,
            signer,
            did_resolver,
            dag_store,
            PathBuf::from("./mana_ledger.sled"),
            PathBuf::from("./reputation.sled"),
        )
    }

    /// Create a new context using filesystem paths for the DAG store and mana ledger.
    /// The store type is selected based on enabled persistence features.
    pub fn new_with_paths(
        current_identity: Did,
        mesh_network_service: Arc<dyn MeshNetworkService>,
        signer: Arc<dyn Signer>,
        did_resolver: Arc<dyn icn_identity::DidResolver>,
        dag_path: PathBuf,
        mana_ledger_path: PathBuf,
        reputation_store_path: PathBuf,
    ) -> Arc<Self> {
        #[cfg(feature = "persist-rocksdb")]
        let dag_store = Arc::new(TokioMutex::new(
            RocksDagStore::new(dag_path)
                .unwrap_or_else(|e| panic!("Failed to init RocksDagStore: {e}")),
        )) as Arc<TokioMutex<dyn DagStorageService<DagBlock> + Send>>;

        #[cfg(all(not(feature = "persist-rocksdb"), feature = "persist-sled"))]
        let dag_store = Arc::new(TokioMutex::new(
            icn_dag::sled_store::SledDagStore::new(dag_path).expect("Failed to init SledDagStore"),
        )) as Arc<TokioMutex<dyn DagStorageService<DagBlock> + Send>>;

        #[cfg(all(
            not(feature = "persist-rocksdb"),
            not(feature = "persist-sled"),
            feature = "persist-sqlite"
        ))]
        let dag_store = Arc::new(TokioMutex::new(
            icn_dag::sqlite_store::SqliteDagStore::new(dag_path)
                .expect("Failed to init SqliteDagStore"),
        )) as Arc<TokioMutex<dyn DagStorageService<DagBlock> + Send>>;

        #[cfg(not(any(
            feature = "persist-rocksdb",
            feature = "persist-sled",
            feature = "persist-sqlite"
        )))]
        let dag_store = Arc::new(TokioMutex::new(
            FileDagStore::new(dag_path).expect("Failed to init FileDagStore"),
        )) as Arc<TokioMutex<dyn DagStorageService<DagBlock> + Send>>;

        Self::new_with_ledger_path(
            current_identity,
            mesh_network_service,
            signer,
            did_resolver,
            dag_store,
            mana_ledger_path,
            reputation_store_path,
        )
    }

    #[cfg(feature = "enable-libp2p")]
    pub async fn new_with_libp2p_network(
        current_identity_str: &str,
        bootstrap_peers: Option<Vec<(Libp2pPeerId, Multiaddr)>>,
    ) -> Result<Arc<Self>, CommonError> {
        let current_identity = Did::from_str(current_identity_str).map_err(|e| {
            CommonError::IdentityError(format!(
                "Invalid DID string for new_with_libp2p_network: {}: {}",
                current_identity_str, e
            ))
        })?;

        let mut config = NetworkConfig::default();
        if let Some(peers) = bootstrap_peers {
            config.bootstrap_peers = peers;
        }

        let libp2p_service_concrete =
            Arc::new(ActualLibp2pNetworkService::new(config).await.map_err(|e| {
                CommonError::NetworkSetupError(format!(
                    "Failed to create Libp2pNetworkService: {}",
                    e
                ))
            })?);
        let libp2p_service_dyn: Arc<dyn ActualNetworkService> = libp2p_service_concrete;

        let default_mesh_service =
            Arc::new(DefaultMeshNetworkService::new(libp2p_service_dyn.clone()));

        Ok(Self::new(
            current_identity,
            default_mesh_service,
            Arc::new(StubSigner::new()),
            Arc::new(icn_identity::KeyDidResolver),
            Arc::new(TokioMutex::new(StubDagStore::new())),
        ))
    }

    pub fn new_with_stubs(current_identity_str: &str) -> Arc<Self> {
        let current_identity = Did::from_str(current_identity_str)
            .expect("Invalid DID for test context in new_with_stubs");
        #[cfg(feature = "persist-sled")]
        let dag_store = Arc::new(TokioMutex::new(
            icn_dag::sled_store::SledDagStore::new(PathBuf::from("./dag.sled")).unwrap(),
        ));
        #[cfg(not(feature = "persist-sled"))]
        let dag_store = Arc::new(TokioMutex::new(StubDagStore::new()));

        Self::new_with_ledger_path(
            current_identity,
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
            Arc::new(icn_identity::KeyDidResolver),
            dag_store,
            PathBuf::from("./mana_ledger.sled"),
            PathBuf::from("./reputation.sled"),
        )
    }

    pub fn new_with_stubs_and_mana(current_identity_str: &str, initial_mana: u64) -> Arc<Self> {
        let current_identity = Did::from_str(current_identity_str)
            .expect("Invalid DID for test context in new_with_stubs_and_mana");
        #[cfg(feature = "persist-sled")]
        let dag_store = Arc::new(TokioMutex::new(
            icn_dag::sled_store::SledDagStore::new(PathBuf::from("./dag.sled")).unwrap(),
        ));
        #[cfg(not(feature = "persist-sled"))]
        let dag_store = Arc::new(TokioMutex::new(StubDagStore::new()));

        let ctx = Self::new_with_ledger_path(
            current_identity.clone(),
            Arc::new(StubMeshNetworkService::new()),
            Arc::new(StubSigner::new()),
            Arc::new(icn_identity::KeyDidResolver),
            dag_store,
            PathBuf::from("./mana_ledger.sled"),
            PathBuf::from("./reputation.sled"),
        );
        ctx.mana_ledger
            .set_balance(&current_identity, initial_mana)
            .expect("set initial mana");
        ctx
    }

    /// Returns true if the DAG block for the given CID starts with the WASM
    /// magic bytes, indicating a compiled CCL module.
    pub async fn manifest_is_ccl_wasm(&self, cid: &Cid) -> bool {
        if let Ok(Some(block)) = self.dag_store.lock().await.get(cid) {
            return block.data.starts_with(b"\0asm");
        }
        false
    }

    pub async fn internal_queue_mesh_job(
        self: &Arc<Self>,
        job: ActualMeshJob,
    ) -> Result<(), HostAbiError> {
        let mut queue = self.pending_mesh_jobs.lock().await;
        queue.push_back(job.clone());
        let mut states = self.job_states.lock().await;
        states.insert(job.id.clone(), JobState::Pending);
        println!("[CONTEXT] Queued mesh job: id={:?}, state=Pending", job.id);

        if matches!(job.spec.kind, icn_mesh::JobKind::CclWasm)
            || self.manifest_is_ccl_wasm(&job.manifest_cid).await
        {
            let signer = self.signer.clone();
            let ctx_clone = Arc::clone(self);
            let job_clone = job.clone();
            tokio::spawn(async move {
                let executor = crate::executor::WasmExecutor::new(ctx_clone.clone(), signer);
                if let Err(e) = executor.execute_and_anchor_job(&job_clone).await {
                    log::error!("WASM job execution failed: {:?}", e);
                }
            });
        }
        Ok(())
    }

    async fn wait_for_and_process_receipt(
        self: Arc<Self>,
        job: ActualMeshJob,
        assigned_executor_did: Did,
    ) -> Result<(), HostAbiError> {
        info!(
            "[JobManagerDetail] Waiting for receipt for job {:?} from executor {:?}",
            job.id, assigned_executor_did
        );
        // Determine how long to wait for the execution receipt.
        let timeout_ms = job
            .max_execution_wait_ms
            .unwrap_or(self.default_receipt_wait_ms);
        let receipt_timeout = StdDuration::from_millis(timeout_ms);

        match self
            .mesh_network_service
            .try_receive_receipt(&job.id, &assigned_executor_did, receipt_timeout)
            .await
        {
            Ok(Some(receipt)) => {
                info!(
                    "[JobManagerDetail] Received receipt for job {:?}: {:?}",
                    job.id, receipt
                );

                // Resolve executor verifying key and verify the receipt.
                match self.did_resolver.resolve(&receipt.executor_did) {
                    Ok(vk) => {
                        if let Err(e) = receipt.verify_against_key(&vk) {
                            error!(
                                "[JobManagerDetail] Receipt signature VERIFICATION FAILED for job {:?}: {}",
                                job.id, e
                            );
                            return Err(HostAbiError::SignatureError(format!(
                                "Invalid receipt signature: {}",
                                e
                            )));
                        }
                    }
                    Err(e) => {
                        error!(
                            "[JobManagerDetail] Failed to resolve DID {:?}: {}",
                            receipt.executor_did, e
                        );
                        return Err(HostAbiError::Common(e));
                    }
                }

                match self.anchor_receipt(&receipt).await {
                    Ok(receipt_cid) => {
                        info!(
                            "[JobManagerDetail] Receipt for job {:?} anchored successfully: {:?}",
                            job.id, receipt_cid
                        );
                        let mut job_states_guard = self.job_states.lock().await;
                        job_states_guard.insert(
                            job.id.clone(),
                            JobState::Completed {
                                receipt: receipt.clone(),
                            },
                        );
                        self.credit_mana(&receipt.executor_did, job.cost_mana)
                            .await?;
                        self.reputation_store.record_execution(
                            &receipt.executor_did,
                            receipt.success,
                            receipt.cpu_ms,
                        );
                        Ok(())
                    }
                    Err(e) => {
                        error!("[JobManagerDetail] Failed to anchor receipt for job {:?}: {}. Marking as Failed (AnchorFailed).", job.id, e);
                        let mut job_states_guard = self.job_states.lock().await;
                        job_states_guard.insert(
                            job.id.clone(),
                            JobState::Failed {
                                reason: format!("Failed to anchor receipt: {}", e),
                            },
                        );
                        Err(HostAbiError::DagOperationFailed(format!(
                            "Failed to anchor receipt: {}",
                            e
                        )))
                    }
                }
            }
            Ok(None) => {
                warn!("[JobManagerDetail] No receipt received for job {:?} within timeout. Marking as Failed (NoReceipt).", job.id);
                let mut job_states_guard = self.job_states.lock().await;
                job_states_guard.insert(
                    job.id.clone(),
                    JobState::Failed {
                        reason: "No receipt received within timeout".to_string(),
                    },
                );
                Err(HostAbiError::NetworkError(
                    "No receipt received within timeout".to_string(),
                ))
            }
            Err(e) => {
                error!("[JobManagerDetail] Error while trying to receive receipt for job {:?}: {}. Marking as Failed (ReceiptError).", job.id, e);
                let mut job_states_guard = self.job_states.lock().await;
                job_states_guard.insert(
                    job.id.clone(),
                    JobState::Failed {
                        reason: format!("Error receiving receipt: {}", e),
                    },
                );
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
                while let Some(job) = {
                    // job here is ActualMeshJob
                    let mut pending_jobs_guard = self_clone.pending_mesh_jobs.lock().await;
                    let popped_job = pending_jobs_guard.pop_front();
                    drop(pending_jobs_guard);
                    popped_job
                } {
                    jobs_processed_in_cycle += 1;
                    let current_job_id = job.id.clone(); // Clone for use in logs and map keys

                    // Get current state from the central job_states map
                    let job_states_guard = self_clone.job_states.lock().await;
                    let current_job_state = job_states_guard.get(&current_job_id).cloned();
                    drop(job_states_guard); // Release lock quickly

                    info!(
                        "[JobManagerLoop] Processing job: {:?}, current state from map: {:?}",
                        current_job_id, current_job_state
                    );

                    match current_job_state {
                        Some(JobState::Pending) => {
                            info!(
                                "[JobManagerLoop] Job {:?} is Pending. Announcing...",
                                current_job_id
                            );
                            if let Err(e) = self_clone.mesh_network_service.announce_job(&job).await
                            {
                                error!(
                                    "[JobManagerLoop] Failed to announce job {:?}: {}. Re-queuing.",
                                    current_job_id, e
                                );
                                jobs_to_requeue.push_back(job); // Re-queue the ActualMeshJob
                                continue;
                            }
                            info!(
                                "[JobManagerLoop] Job {:?} announced. Collecting bids...",
                                current_job_id
                            );
                            let bid_collection_duration = StdDuration::from_secs(10);

                            let bids_result = self_clone
                                .mesh_network_service
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
                                let mut job_states_guard = self_clone.job_states.lock().await;
                                job_states_guard.insert(
                                    current_job_id.clone(),
                                    JobState::Failed {
                                        reason: "No bids received".to_string(),
                                    },
                                );
                                drop(job_states_guard);
                                if let Err(e) = self_clone
                                    .credit_mana(&job.creator_did, job.cost_mana)
                                    .await
                                {
                                    error!(
                                        "[JobManagerLoop] Failed to refund mana to {:?}: {}",
                                        job.creator_did, e
                                    );
                                }
                                continue;
                            }

                            info!("[JobManagerLoop] Received {} bids for job {:?}. Selecting executor...", bids.len(), current_job_id);
                            let policy = icn_mesh::SelectionPolicy::default();
                            let maybe_exec = icn_mesh::select_executor(
                                &current_job_id,
                                &job.spec,
                                bids.clone(),
                                &policy,
                                self_clone.reputation_store.as_ref(),
                                &self_clone.mana_ledger,
                            );
                            if let Some(selected_exec) = maybe_exec {
                                let new_state = JobState::Assigned {
                                    executor: selected_exec.clone(),
                                };
                                info!("[JobManagerLoop] Job {:?} assigned to executor {:?}. Notifying...", current_job_id, selected_exec);

                                let mut job_states_guard = self_clone.job_states.lock().await;
                                job_states_guard.insert(current_job_id.clone(), new_state);
                                drop(job_states_guard);

                                let notice = JobAssignmentNotice {
                                    job_id: current_job_id.clone(),
                                    executor_did: selected_exec.clone(),
                                };

                                if let Err(e) = self_clone
                                    .mesh_network_service
                                    .notify_executor_of_assignment(&notice)
                                    .await
                                {
                                    error!("[JobManagerLoop] Failed to notify executor for job {:?}: {}. Reverting to Pending.", current_job_id, e);
                                    let mut job_states_guard = self_clone.job_states.lock().await;
                                    job_states_guard
                                        .insert(current_job_id.clone(), JobState::Pending); // Revert state in map
                                    drop(job_states_guard);
                                    jobs_to_requeue.push_back(job); // Re-queue the ActualMeshJob
                                    continue;
                                }

                                info!("[JobManagerLoop] Job {:?} successfully assigned. Spawning receipt monitor.", current_job_id);
                                // self_clone is Arc<RuntimeContext> here
                                let task_ctx = self_clone.clone(); // Clone the Arc for the new task
                                tokio::spawn(async move {
                                    if let Err(e) = task_ctx
                                        .wait_for_and_process_receipt(job, selected_exec)
                                        .await
                                    {
                                        error!("[JobManagerDetail] Error in wait_for_and_process_receipt for job {:?}: {:?}", current_job_id, e);
                                    }
                                });
                            } else {
                                warn!("[JobManagerLoop] No valid bid selected for job {:?}. Marking as Failed.", current_job_id);
                                let mut job_states_guard = self_clone.job_states.lock().await;
                                job_states_guard.insert(
                                    current_job_id.clone(),
                                    JobState::Failed {
                                        reason: "No valid bid selected".to_string(),
                                    },
                                );
                                drop(job_states_guard);
                                if let Err(e) = self_clone
                                    .credit_mana(&job.creator_did, job.cost_mana)
                                    .await
                                {
                                    error!(
                                        "[JobManagerLoop] Failed to refund mana to {:?}: {}",
                                        job.creator_did, e
                                    );
                                }
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
                            info!(
                                "[JobManagerLoop] Job {:?} is in a terminal state. No action.",
                                current_job_id
                            );
                            // Do not re-queue jobs that are completed or failed.
                        }
                        None => {
                            // Job was in pending_mesh_jobs but not in job_states map
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
        Ok(self.mana_ledger.get_balance(account))
    }

    pub async fn spend_mana(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        println!(
            "[CONTEXT] spend_mana called for account: {:?} amount: {}",
            account, amount
        );
        if account != &self.current_identity {
            return Err(HostAbiError::InvalidParameters(
                "Attempting to spend mana for an account other than the current context identity."
                    .to_string(),
            ));
        }
        self.mana_ledger.spend(account, amount)
    }

    pub async fn credit_mana(&self, account: &Did, amount: u64) -> Result<(), HostAbiError> {
        println!(
            "[CONTEXT] credit_mana called for account: {:?} amount: {}",
            account, amount
        );
        let adapter = ManaRepositoryAdapter::new(self.mana_ledger.clone());
        adapter
            .credit_mana(account, amount)
            .map_err(|e| HostAbiError::InternalError(format!("{e:?}")))
    }

    /// Anchors an execution receipt to the DAG store and returns the content identifier (CID).
    /// This method is called by the Host ABI function `host_anchor_receipt`.
    pub async fn anchor_receipt(
        &self,
        receipt: &IdentityExecutionReceipt,
    ) -> Result<Cid, HostAbiError> {
        info!(
            "[CONTEXT] Attempting to anchor receipt for job {:?} from executor {:?}",
            receipt.job_id, receipt.executor_did
        );

        // Resolve the executor's verifying key via the provided DidResolver.
        let verifying_key = self
            .did_resolver
            .resolve(&receipt.executor_did)
            .map_err(HostAbiError::Common)?;

        receipt.verify_against_key(&verifying_key).map_err(|e| {
            HostAbiError::SignatureError(format!(
                "Receipt signature verification failed for job {:?}, executor {:?}: {}",
                receipt.job_id, receipt.executor_did, e
            ))
        })?;

        // If signature is valid, store the receipt in DAG
        let final_receipt_bytes = serde_json::to_vec(receipt).map_err(|e| {
            HostAbiError::InternalError(format!("Failed to serialize final receipt for DAG: {}", e))
        })?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let author = self.current_identity.clone();
        let signature = None;
        let cid = icn_common::compute_merkle_cid(
            0x71,
            &final_receipt_bytes,
            &[],
            timestamp,
            &author,
            &signature,
        );
        let block = DagBlock {
            cid,
            data: final_receipt_bytes,
            links: vec![],
            timestamp,
            author_did: author,
            signature,
        };
        let mut store = self.dag_store.lock().await;
        store.put(&block).map_err(HostAbiError::Common)?;
        let cid = block.cid.clone();
        println!("[CONTEXT] Anchored receipt for job_id {:?} with CID: {:?}. Executor: {:?}. Receipt cost {}ms.", 
                 receipt.job_id, cid, receipt.executor_did, receipt.cpu_ms);

        {
            let mut job_states_guard = self.job_states.lock().await;
            job_states_guard.insert(
                receipt.job_id.clone(),
                JobState::Completed {
                    receipt: receipt.clone(),
                },
            );
            println!(
                "[CONTEXT] Job {:?} state updated to Completed.",
                receipt.job_id
            );
        }
        println!(
            "[CONTEXT] Placeholder: Reputation update needed for executor {:?} for job {:?}.",
            receipt.executor_did, receipt.job_id
        );
        Ok(cid)
    }

    pub async fn create_governance_proposal(
        &self,
        payload: CreateProposalPayload,
    ) -> Result<String, HostAbiError> {
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
        let pid = gov
            .submit_proposal(
                self.current_identity.clone(),
                proposal_type,
                payload.description,
                payload.duration_secs,
            )
            .map_err(HostAbiError::Common)?;
        let proposal = gov
            .get_proposal(&pid)
            .map_err(HostAbiError::Common)?
            .expect("Proposal just inserted should exist");
        drop(gov);
        let encoded = bincode::serialize(&proposal).map_err(|e| {
            HostAbiError::InternalError(format!("Failed to serialize proposal: {}", e))
        })?;
        if let Err(e) = self.mesh_network_service.announce_proposal(encoded).await {
            warn!("Failed to broadcast proposal {:?}: {}", pid, e);
        }
        Ok(pid.0)
    }

    pub async fn cast_governance_vote(&self, payload: CastVotePayload) -> Result<(), HostAbiError> {
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut gov = self.governance_module.lock().await;
        gov.cast_vote(self.current_identity.clone(), &proposal_id, vote_option)
            .map_err(HostAbiError::Common)?;
        let vote = icn_governance::Vote {
            voter: self.current_identity.clone(),
            proposal_id: proposal_id.clone(),
            option: vote_option,
            voted_at: now,
        };
        drop(gov);
        let encoded = bincode::serialize(&vote)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to serialize vote: {}", e)))?;
        if let Err(e) = self.mesh_network_service.announce_vote(encoded).await {
            warn!("Failed to broadcast vote for {:?}: {}", proposal_id, e);
        }
        Ok(())
    }

    pub async fn close_governance_proposal_voting(
        &self,
        proposal_id_str: &str,
    ) -> Result<String, HostAbiError> {
        let proposal_id = ProposalId::from_str(proposal_id_str)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid proposal id: {e}")))?;

        let mut gov = self.governance_module.lock().await;
        let status = gov
            .close_voting_period(&proposal_id)
            .map_err(HostAbiError::Common)?;
        let proposal = gov
            .get_proposal(&proposal_id)
            .map_err(HostAbiError::Common)?
            .expect("Proposal should exist after closing");
        drop(gov);

        let encoded = bincode::serialize(&proposal).map_err(|e| {
            HostAbiError::InternalError(format!("Failed to serialize proposal: {e}"))
        })?;
        if let Err(e) = self.mesh_network_service.announce_proposal(encoded).await {
            warn!("Failed to broadcast proposal {:?}: {}", proposal_id, e);
        }

        Ok(format!("{:?}", status))
    }

    pub async fn execute_governance_proposal(
        &self,
        proposal_id_str: &str,
    ) -> Result<(), HostAbiError> {
        let proposal_id = ProposalId::from_str(proposal_id_str)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid proposal id: {e}")))?;

        let mut gov = self.governance_module.lock().await;
        gov.execute_proposal(&proposal_id)
            .map_err(HostAbiError::Common)?;
        let proposal = gov
            .get_proposal(&proposal_id)
            .map_err(HostAbiError::Common)?
            .expect("Proposal should exist after execution");
        drop(gov);

        let encoded = bincode::serialize(&proposal).map_err(|e| {
            HostAbiError::InternalError(format!("Failed to serialize proposal: {e}"))
        })?;
        if let Err(e) = self.mesh_network_service.announce_proposal(encoded).await {
            warn!("Failed to broadcast proposal {:?}: {}", proposal_id, e);
        }
        Ok(())
    }

    /// Inserts a proposal received from the network into the local GovernanceModule.
    pub async fn ingest_external_proposal(
        &self,
        proposal_bytes: &[u8],
    ) -> Result<(), HostAbiError> {
        let proposal: icn_governance::Proposal =
            bincode::deserialize(proposal_bytes).map_err(|e| {
                HostAbiError::InternalError(format!("Failed to decode proposal: {}", e))
            })?;
        let mut gov = self.governance_module.lock().await;
        if gov
            .get_proposal(&proposal.id)
            .map_err(HostAbiError::Common)?
            .is_some()
        {
            return Ok(());
        }
        gov.insert_external_proposal(proposal)
            .map_err(HostAbiError::Common)
    }

    /// Inserts a vote received from the network into the local GovernanceModule.
    pub async fn ingest_external_vote(&self, vote_bytes: &[u8]) -> Result<(), HostAbiError> {
        let vote: icn_governance::Vote = bincode::deserialize(vote_bytes)
            .map_err(|e| HostAbiError::InternalError(format!("Failed to decode vote: {}", e)))?;
        let mut gov = self.governance_module.lock().await;
        if gov
            .get_proposal(&vote.proposal_id)
            .map_err(HostAbiError::Common)?
            .is_none()
        {
            return Ok(());
        }
        gov.insert_external_vote(vote).map_err(HostAbiError::Common)
    }

    /// Create a new RuntimeContext with real libp2p networking
    #[cfg(feature = "enable-libp2p")]
    pub async fn new_with_real_libp2p(
        identity_str: &str,
        listen_addresses: Vec<Multiaddr>,
        bootstrap_peers: Option<Vec<(Libp2pPeerId, Multiaddr)>>,
        mana_ledger_path: PathBuf,
        reputation_store_path: PathBuf,
    ) -> Result<Arc<Self>, CommonError> {
        info!("Initializing RuntimeContext with real libp2p networking");

        // Parse the identity
        let identity = Did::from_str(identity_str)
            .map_err(|e| CommonError::InvalidInputError(format!("Invalid DID: {}", e)))?;

        // Generate keys for this node
        let (sk, pk) = generate_ed25519_keypair();
        let signer = Arc::new(StubSigner::new_with_keys(sk, pk));

        // Create real libp2p network service with proper config
        let mut config = NetworkConfig {
            listen_addresses,
            ..NetworkConfig::default()
        };
        if let Some(peers) = bootstrap_peers {
            info!("Bootstrap peers provided: {} peers", peers.len());
            config.bootstrap_peers = peers;
        }

        let libp2p_service =
            Arc::new(ActualLibp2pNetworkService::new(config).await.map_err(|e| {
                CommonError::NetworkError(format!("Failed to create libp2p service: {}", e))
            })?);

        info!(
            "Libp2p service created with PeerID: {}",
            libp2p_service.local_peer_id()
        );

        // Wrap in DefaultMeshNetworkService
        let mesh_service = Arc::new(DefaultMeshNetworkService::new(
            libp2p_service.clone() as Arc<dyn ActualNetworkService>
        ));

        // Create stub DAG store for now (can be enhanced later)
        let dag_store = Arc::new(TokioMutex::new(StubDagStore::new()));

        // Create RuntimeContext with real networking - this returns Arc<Self>
        let ctx = Self::new_with_ledger_path(
            identity,
            mesh_service,
            signer,
            Arc::new(KeyDidResolver),
            dag_store,
            mana_ledger_path,
            reputation_store_path,
        );

        info!("RuntimeContext with real libp2p networking created successfully");
        Ok(ctx)
    }

    /// Get the underlying libp2p service for peer info access
    #[cfg(feature = "enable-libp2p")]
    pub fn get_libp2p_service(&self) -> Result<Arc<ActualLibp2pNetworkService>, CommonError> {
        if let Some(default_mesh) = MeshNetworkService::as_any(self.mesh_network_service.as_ref())
            .downcast_ref::<DefaultMeshNetworkService>()
        {
            default_mesh.get_underlying_broadcast_service()
        } else {
            Err(CommonError::NetworkError(
                "RuntimeContext is not using DefaultMeshNetworkService with libp2p".to_string(),
            ))
        }
    }

    /// Shut down the underlying libp2p service if present
    #[cfg(feature = "enable-libp2p")]
    pub async fn shutdown_network(&self) -> Result<(), CommonError> {
        if let Some(default_mesh) = MeshNetworkService::as_any(self.mesh_network_service.as_ref())
            .downcast_ref::<DefaultMeshNetworkService>()
        {
            let service = default_mesh.get_underlying_broadcast_service()?;
            service
                .as_ref()
                .clone()
                .shutdown()
                .await
                .map_err(|e| CommonError::NetworkError(e.to_string()))
        } else {
            Ok(())
        }
    }
}

// --- Supporting: RuntimeContext::new_for_test ---
#[cfg(test)]
impl RuntimeContext {
    pub fn new_for_test(
        current_identity: Did,
        signer: StubSigner,
        mesh_network_service: Arc<StubMeshNetworkService>,
        dag_store: Arc<TokioMutex<StubDagStore>>,
    ) -> Arc<Self> {
        let job_states = Arc::new(TokioMutex::new(HashMap::new()));
        let pending_mesh_jobs = Arc::new(TokioMutex::new(VecDeque::new()));
        let temp_dir = tempfile::tempdir().expect("temp dir for mana ledger");
        let mana_ledger_path = temp_dir.path().join("mana.sled");
        let mana_ledger = SimpleManaLedger::new(mana_ledger_path);
        #[cfg(feature = "persist-sled")]
        let governance_module = Arc::new(TokioMutex::new(
            GovernanceModule::new_sled(std::path::PathBuf::from("./governance_db_test"))
                .unwrap_or_else(|_| GovernanceModule::new()),
        ));
        #[cfg(not(feature = "persist-sled"))]
        let governance_module = Arc::new(TokioMutex::new(GovernanceModule::new()));

        let reputation_store: Arc<dyn icn_reputation::ReputationStore> =
            Arc::new(icn_reputation::InMemoryReputationStore::new());
        let did_resolver: Arc<dyn icn_identity::DidResolver> =
            Arc::new(icn_identity::KeyDidResolver);

        Arc::new(Self {
            current_identity,
            mana_ledger,
            pending_mesh_jobs,
            job_states,
            governance_module,
            mesh_network_service,
            signer: Arc::new(signer),
            did_resolver,
            dag_store,
            reputation_store,
            default_receipt_wait_ms: 60_000,
        })
    }
}
// --- End Supporting: RuntimeContext::new_for_test ---

pub trait HostEnvironment: Send + Sync + std::fmt::Debug {
    fn env_submit_mesh_job(
        &self,
        ctx: &Arc<RuntimeContext>,
        job_data_ptr: u32,
        job_data_len: u32,
    ) -> Result<u32, HostAbiError>;
    fn env_account_get_mana(
        &self,
        ctx: &Arc<RuntimeContext>,
        account_did_ptr: u32,
        account_did_len: u32,
    ) -> Result<u64, HostAbiError>;
    fn env_account_spend_mana(
        &self,
        ctx: &Arc<RuntimeContext>,
        account_did_ptr: u32,
        account_did_len: u32,
        amount: u64,
    ) -> Result<(), HostAbiError>;
}

#[derive(Debug, Default)]
pub struct ConcreteHostEnvironment {
    memory: Vec<u8>,
}

impl ConcreteHostEnvironment {
    pub fn new() -> Self {
        Self { memory: Vec::new() }
    }

    pub fn set_memory(&mut self, data: Vec<u8>) {
        self.memory = data;
    }
}

impl HostEnvironment for ConcreteHostEnvironment {
    fn env_submit_mesh_job(
        &self,
        ctx: &Arc<RuntimeContext>,
        job_data_ptr: u32,
        job_data_len: u32,
    ) -> Result<u32, HostAbiError> {
        let end = job_data_ptr as usize + job_data_len as usize;
        if end > self.memory.len() {
            return Err(HostAbiError::InvalidParameters(
                "Job data out of bounds".into(),
            ));
        }

        let job_slice = &self.memory[job_data_ptr as usize..end];
        let job_json = std::str::from_utf8(job_slice).map_err(|e| {
            HostAbiError::InvalidParameters(format!("Invalid UTF-8 job data: {}", e))
        })?;

        let mut job: ActualMeshJob = serde_json::from_str(job_json).map_err(|e| {
            HostAbiError::InvalidParameters(format!("Failed to deserialize mesh job JSON: {}", e))
        })?;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| HostAbiError::InternalError(format!("Runtime build failed: {e}")))?;

        let ctx_clone = Arc::clone(ctx);
        let cid = rt.block_on(async {
            ctx_clone
                .spend_mana(&ctx_clone.current_identity, job.cost_mana)
                .await?;
            let job_id_val = NEXT_JOB_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let cid = Cid::new_v1_sha256(0x55, format!("job_cid_{job_id_val}").as_bytes());
            job.id = cid.clone();
            job.creator_did = ctx_clone.current_identity.clone();
            ctx_clone.internal_queue_mesh_job(job).await?;
            Ok::<Cid, HostAbiError>(cid)
        })?;

        let mut id_bytes = [0u8; 4];
        if cid.hash_bytes.len() >= 4 {
            id_bytes.copy_from_slice(&cid.hash_bytes[..4]);
        }
        Ok(u32::from_le_bytes(id_bytes))
    }
    fn env_account_get_mana(
        &self,
        ctx: &Arc<RuntimeContext>,
        account_did_ptr: u32,
        account_did_len: u32,
    ) -> Result<u64, HostAbiError> {
        let end = account_did_ptr as usize + account_did_len as usize;
        if end > self.memory.len() {
            return Err(HostAbiError::InvalidParameters(
                "Account DID out of bounds".into(),
            ));
        }
        let did_slice = &self.memory[account_did_ptr as usize..end];
        let did_str = std::str::from_utf8(did_slice).map_err(|e| {
            HostAbiError::InvalidParameters(format!("Invalid UTF-8 account DID: {}", e))
        })?;

        let account_did = Did::from_str(did_str)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid DID format: {}", e)))?;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| HostAbiError::InternalError(format!("Runtime build failed: {e}")))?;

        let ctx_clone = Arc::clone(ctx);
        rt.block_on(ctx_clone.get_mana(&account_did))
    }
    fn env_account_spend_mana(
        &self,
        ctx: &Arc<RuntimeContext>,
        account_did_ptr: u32,
        account_did_len: u32,
        amount: u64,
    ) -> Result<(), HostAbiError> {
        let end = account_did_ptr as usize + account_did_len as usize;
        if end > self.memory.len() {
            return Err(HostAbiError::InvalidParameters(
                "Account DID out of bounds".into(),
            ));
        }
        let did_slice = &self.memory[account_did_ptr as usize..end];
        let did_str = std::str::from_utf8(did_slice).map_err(|e| {
            HostAbiError::InvalidParameters(format!("Invalid UTF-8 account DID: {}", e))
        })?;

        let account_did = Did::from_str(did_str)
            .map_err(|e| HostAbiError::InvalidParameters(format!("Invalid DID format: {}", e)))?;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| HostAbiError::InternalError(format!("Runtime build failed: {e}")))?;

        let ctx_clone = Arc::clone(ctx);
        rt.block_on(ctx_clone.spend_mana(&account_did, amount))
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

    fn verify(
        &self,
        payload: &[u8],
        signature_bytes: &[u8],
        public_key_bytes: &[u8],
    ) -> Result<bool, HostAbiError> {
        let pk_array: [u8; 32] = public_key_bytes.try_into().map_err(|_| {
            HostAbiError::InvalidParameters("Public key bytes not 32 bytes long".to_string())
        })?;
        let verifying_key = VerifyingKey::from_bytes(&pk_array).map_err(|e| {
            HostAbiError::CryptoError(format!("Failed to create verifying key: {}", e))
        })?;

        let signature_array: [u8; SIGNATURE_LENGTH] = signature_bytes.try_into().map_err(|_| {
            HostAbiError::InvalidParameters(format!(
                "Signature not {} bytes long",
                SIGNATURE_LENGTH
            ))
        })?;
        let signature = EdSignature::from_bytes(&signature_array); // ed25519_dalek::Signature::from_bytes
                                                                   // .map_err(|e| HostAbiError::CryptoError(format!("Failed to create signature from bytes: {}", e)))?;

        Ok(identity_verify_signature(
            &verifying_key,
            payload,
            &signature,
        ))
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
pub struct StubDagStore {
    store: HashMap<Cid, DagBlock>,
}
impl StubDagStore {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }
    pub fn all(&self) -> HashMap<Cid, DagBlock> {
        self.store.clone()
    }
}

impl Default for StubDagStore {
    fn default() -> Self {
        Self::new()
    }
}

pub type RuntimeStubDagStore = StubDagStore;
impl DagStorageService<DagBlock> for StubDagStore {
    fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        self.store.insert(block.cid.clone(), block.clone());
        Ok(())
    }

    fn get(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        Ok(self.store.get(cid).cloned())
    }

    fn delete(&mut self, cid: &Cid) -> Result<(), CommonError> {
        self.store.remove(cid);
        Ok(())
    }

    fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        Ok(self.store.contains_key(cid))
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
impl MeshNetworkService for StubMeshNetworkService {
    // Implements local MeshNetworkService trait
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        println!("[StubMeshNetworkService] Announced job: {:?}", job.id);
        Ok(())
    }

    async fn announce_proposal(&self, _proposal_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        println!("[StubMeshNetworkService] Announced proposal (stub)");
        Ok(())
    }

    async fn announce_vote(&self, _vote_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        println!("[StubMeshNetworkService] Announced vote (stub)");
        Ok(())
    }

    async fn collect_bids_for_job(
        &self,
        job_id: &JobId,
        _duration: StdDuration,
    ) -> Result<Vec<MeshJobBid>, HostAbiError> {
        println!(
            "[StubMeshNetworkService] Collecting bids for job {:?}.",
            job_id
        );
        let mut bids_map = self.staged_bids.lock().await;
        if let Some(job_bids_queue) = bids_map.get_mut(job_id) {
            let bids: Vec<MeshJobBid> = job_bids_queue.drain(..).collect();
            println!(
                "[StubMeshNetworkService] Found {} staged bids for job {:?}",
                bids.len(),
                job_id
            );
            Ok(bids)
        } else {
            println!(
                "[StubMeshNetworkService] No staged bids found for job {:?}. Returning empty vec.",
                job_id
            );
            Ok(Vec::new())
        }
    }

    async fn notify_executor_of_assignment(
        &self,
        notice: &JobAssignmentNotice,
    ) -> Result<(), HostAbiError> {
        println!(
            "[StubMeshNetworkService] Broadcast assignment for job {:?} to executor {:?}",
            notice.job_id, notice.executor_did
        );
        Ok(())
    }

    async fn try_receive_receipt(
        &self,
        _job_id: &JobId,
        _expected_executor: &Did,
        _timeout_duration: StdDuration,
    ) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
        let mut receipts_queue = self.staged_receipts.lock().await;
        if let Some(receipt_msg) = receipts_queue.pop_front() {
            println!(
                "[StubMeshNetworkService] try_receive_receipt: Popped staged receipt for job {:?}",
                receipt_msg.receipt.job_id
            );
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

#[cfg(test)]
mod tests {
    use super::*;
    use icn_identity::KeyDidResolver;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Mutex as TokioMutex;

    #[test]
    fn test_env_submit_mesh_job_success() {
        let mut env = ConcreteHostEnvironment::new();
        let _ = std::fs::remove_file("./mana_ledger.sled");
        let ctx_arc = RuntimeContext::new_with_stubs_and_mana("did:icn:test:env_submit", 100);

        let job = ActualMeshJob {
            id: Cid::new_v1_sha256(0x55, b"job"),
            manifest_cid: Cid::new_v1_sha256(0x55, b"manifest"),
            spec: icn_mesh::JobSpec::default(),
            creator_did: ctx_arc.current_identity.clone(),
            cost_mana: 10,
            max_execution_wait_ms: None,
            signature: icn_identity::SignatureBytes(vec![0u8; 64]),
        };
        let job_json = serde_json::to_vec(&job).unwrap();
        env.set_memory(job_json.clone());
        let ptr = 0u32;
        let len = job_json.len() as u32;
        let result = env.env_submit_mesh_job(&ctx_arc, ptr, len);
        assert!(result.is_ok());

        let mana_after =
            futures::executor::block_on(ctx_arc.get_mana(&ctx_arc.current_identity)).unwrap();
        assert_eq!(mana_after, 90);
        let pending_len =
            futures::executor::block_on(async { ctx_arc.pending_mesh_jobs.lock().await.len() });
        assert_eq!(pending_len, 1);
    }

    #[test]
    fn test_env_account_get_mana_success() {
        let mut env = ConcreteHostEnvironment::new();
        let _ = std::fs::remove_file("./mana_ledger.sled");
        let ctx_arc = RuntimeContext::new_with_stubs_and_mana("did:icn:test:env_mana", 50);

        let did_bytes = ctx_arc.current_identity.to_string().into_bytes();
        env.set_memory(did_bytes.clone());
        let ptr = 0u32;
        let len = did_bytes.len() as u32;
        let mana = env.env_account_get_mana(&ctx_arc, ptr, len).unwrap();
        assert_eq!(mana, 50);
    }

    #[test]
    fn test_env_account_spend_mana_success() {
        let mut env = ConcreteHostEnvironment::new();
        let _ = std::fs::remove_file("./mana_ledger.sled");
        let ctx_arc = RuntimeContext::new_with_stubs_and_mana("did:icn:test:env_spend", 20);

        let did_bytes = ctx_arc.current_identity.to_string().into_bytes();
        env.set_memory(did_bytes.clone());
        let ptr = 0u32;
        let len = did_bytes.len() as u32;
        let result = env.env_account_spend_mana(&ctx_arc, ptr, len, 10);
        assert!(result.is_ok());
        let mana =
            futures::executor::block_on(ctx_arc.get_mana(&ctx_arc.current_identity)).unwrap();
        assert_eq!(mana, 10);
    }

    #[test]
    fn test_env_submit_mesh_job_invalid_utf8() {
        let mut env = ConcreteHostEnvironment::new();
        let ctx_arc = RuntimeContext::new_with_stubs_and_mana("did:icn:test:env_submit_utf8", 100);

        // Invalid UTF-8 bytes
        env.set_memory(vec![0xff, 0xfe, 0xfd]);
        let err = env.env_submit_mesh_job(&ctx_arc, 0, 3).unwrap_err();
        assert!(matches!(err, HostAbiError::InvalidParameters(_)));
    }

    #[test]
    fn test_env_account_get_mana_invalid_did() {
        let mut env = ConcreteHostEnvironment::new();
        let ctx_arc = RuntimeContext::new_with_stubs_and_mana("did:icn:test:env_get_invalid", 10);

        env.set_memory(b"not_a_did".to_vec());
        let res = env.env_account_get_mana(&ctx_arc, 0, 9);
        assert!(matches!(res, Err(HostAbiError::InvalidParameters(_))));
    }

    #[test]
    fn test_env_account_spend_mana_insufficient() {
        let mut env = ConcreteHostEnvironment::new();
        let ctx_arc = RuntimeContext::new_with_stubs_and_mana("did:icn:test:env_spend_fail", 5);

        let did_bytes = ctx_arc.current_identity.to_string().into_bytes();
        env.set_memory(did_bytes.clone());
        let err = env
            .env_account_spend_mana(&ctx_arc, 0, did_bytes.len() as u32, 10)
            .unwrap_err();
        assert!(matches!(err, HostAbiError::InsufficientMana));
    }

    #[tokio::test]
    async fn test_wait_for_and_process_receipt_updates_mana_and_reputation() {
        let (sk, vk) = generate_ed25519_keypair();
        let did = did_key_from_verifying_key(&vk);
        let signer = Arc::new(StubSigner::new_with_keys(sk.clone(), vk));
        let ctx = RuntimeContext::new_with_ledger_path(
            Did::from_str(&did).unwrap(),
            Arc::new(StubMeshNetworkService::new()),
            signer.clone(),
            Arc::new(KeyDidResolver),
            Arc::new(TokioMutex::new(StubDagStore::new())),
            PathBuf::from("./mana_ledger.sled"),
            PathBuf::from("./reputation.sled"),
        );

        let stub_net = ctx
            .mesh_network_service
            .clone()
            .downcast_arc::<StubMeshNetworkService>()
            .expect("stub network");

        let job = ActualMeshJob {
            id: Cid::new_v1_sha256(0x55, b"job_update"),
            manifest_cid: Cid::new_v1_sha256(0x55, b"man"),
            spec: icn_mesh::JobSpec::default(),
            creator_did: Did::from_str("did:icn:test:creator").unwrap(),
            cost_mana: 5,
            max_execution_wait_ms: None,
            signature: icn_identity::SignatureBytes(Vec::new()),
        };

        let receipt = IdentityExecutionReceipt {
            job_id: job.id.clone(),
            executor_did: ctx.current_identity.clone(),
            result_cid: Cid::new_v1_sha256(0x55, b"res"),
            cpu_ms: 1,
            success: true,
            sig: icn_identity::SignatureBytes(Vec::new()),
        };

        // Manually sign the receipt using the context signer
        let mut msg = Vec::new();
        msg.extend_from_slice(receipt.job_id.to_string().as_bytes());
        msg.extend_from_slice(ctx.current_identity.to_string().as_bytes());
        msg.extend_from_slice(receipt.result_cid.to_string().as_bytes());
        msg.extend_from_slice(&receipt.cpu_ms.to_le_bytes());
        msg.push(receipt.success as u8);
        let sig_bytes = ctx.signer.sign(&msg).expect("sign");
        let mut signed_receipt = receipt.clone();
        signed_receipt.sig = icn_identity::SignatureBytes(sig_bytes);

        stub_net
            .stage_receipt(LocalMeshSubmitReceiptMessage {
                receipt: signed_receipt.clone(),
            })
            .await;

        ctx.clone()
            .wait_for_and_process_receipt(job, ctx.current_identity.clone())
            .await
            .expect("process receipt");

        assert_eq!(ctx.get_mana(&ctx.current_identity).await.unwrap(), 5);
        assert_eq!(
            ctx.reputation_store.get_reputation(&ctx.current_identity),
            1
        );
    }
}
