//! Stub implementations for testing the ICN runtime.

use super::errors::HostAbiError;
use super::mesh_network::{JobAssignmentNotice, LocalMeshSubmitReceiptMessage, MeshNetworkService};
use icn_common::{Cid, CommonError, DagBlock, Did};
use icn_identity::ExecutionReceipt as IdentityExecutionReceipt;
use icn_mesh::{ActualMeshJob, JobId, MeshJobBid};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::Mutex as TokioMutex;

/// Stub DAG store for testing.
pub struct StubDagStore {
    store: HashMap<Cid, DagBlock>,
    meta: HashMap<Cid, icn_dag::BlockMetadata>,
}

impl StubDagStore {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            meta: HashMap::new(),
        }
    }

    pub fn get_mut(&mut self, cid: &Cid) -> Option<&mut DagBlock> {
        self.store.get_mut(cid)
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

impl icn_dag::StorageService<DagBlock> for StubDagStore {
    fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        self.store.insert(block.cid.clone(), block.clone());
        self.meta.insert(block.cid.clone(), icn_dag::BlockMetadata::default());
        Ok(())
    }

    fn get(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        Ok(self.store.get(cid).cloned())
    }

    fn delete(&mut self, cid: &Cid) -> Result<(), CommonError> {
        self.store.remove(cid);
        self.meta.remove(cid);
        Ok(())
    }

    fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        Ok(self.store.contains_key(cid))
    }

    fn list_blocks(&self) -> Result<Vec<DagBlock>, CommonError> {
        Ok(self.store.values().cloned().collect())
    }

    fn pin_block(&mut self, cid: &Cid) -> Result<(), CommonError> {
        if let Some(meta) = self.meta.get_mut(cid) {
            meta.pinned = true;
        } else {
            return Err(CommonError::InternalError(format!("Block not found: {}", cid)));
        }
        Ok(())
    }

    fn unpin_block(&mut self, cid: &Cid) -> Result<(), CommonError> {
        if let Some(meta) = self.meta.get_mut(cid) {
            meta.pinned = false;
        } else {
            return Err(CommonError::InternalError(format!("Block not found: {}", cid)));
        }
        Ok(())
    }

    fn prune_expired(&mut self, now: u64) -> Result<Vec<Cid>, CommonError> {
        let mut expired = Vec::new();
        let mut to_remove = Vec::new();
        
        for (cid, meta) in &self.meta {
            if let Some(ttl) = meta.ttl {
                if ttl <= now && !meta.pinned {
                    expired.push(cid.clone());
                    to_remove.push(cid.clone());
                }
            }
        }
        
        for cid in &to_remove {
            self.store.remove(cid);
            self.meta.remove(cid);
        }
        
        Ok(expired)
    }

    fn set_ttl(&mut self, cid: &Cid, ttl: Option<u64>) -> Result<(), CommonError> {
        if let Some(meta) = self.meta.get_mut(cid) {
            meta.ttl = ttl;
        } else {
            return Err(CommonError::InternalError(format!("Block not found: {}", cid)));
        }
        Ok(())
    }

    fn get_metadata(&self, cid: &Cid) -> Result<Option<icn_dag::BlockMetadata>, CommonError> {
        Ok(self.meta.get(cid).cloned())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// AsyncStorageService implementation for StubDagStore
#[async_trait::async_trait]
impl icn_dag::AsyncStorageService<DagBlock> for StubDagStore {
    async fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        self.store.insert(block.cid.clone(), block.clone());
        self.meta.insert(block.cid.clone(), icn_dag::BlockMetadata::default());
        Ok(())
    }

    async fn get(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        Ok(self.store.get(cid).cloned())
    }

    async fn delete(&mut self, cid: &Cid) -> Result<(), CommonError> {
        self.store.remove(cid);
        self.meta.remove(cid);
        Ok(())
    }

    async fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        Ok(self.store.contains_key(cid))
    }

    async fn list_blocks(&self) -> Result<Vec<DagBlock>, CommonError> {
        Ok(self.store.values().cloned().collect())
    }

    async fn pin_block(&mut self, cid: &Cid) -> Result<(), CommonError> {
        if let Some(meta) = self.meta.get_mut(cid) {
            meta.pinned = true;
        } else {
            return Err(CommonError::InternalError(format!("Block not found: {}", cid)));
        }
        Ok(())
    }

    async fn unpin_block(&mut self, cid: &Cid) -> Result<(), CommonError> {
        if let Some(meta) = self.meta.get_mut(cid) {
            meta.pinned = false;
        } else {
            return Err(CommonError::InternalError(format!("Block not found: {}", cid)));
        }
        Ok(())
    }

    async fn prune_expired(&mut self, now: u64) -> Result<Vec<Cid>, CommonError> {
        let mut expired = Vec::new();
        let mut to_remove = Vec::new();
        
        for (cid, meta) in &self.meta {
            if let Some(ttl) = meta.ttl {
                if ttl <= now && !meta.pinned {
                    expired.push(cid.clone());
                    to_remove.push(cid.clone());
                }
            }
        }
        
        for cid in &to_remove {
            self.store.remove(cid);
            self.meta.remove(cid);
        }
        
        Ok(expired)
    }

    async fn set_ttl(&mut self, cid: &Cid, ttl: Option<u64>) -> Result<(), CommonError> {
        if let Some(meta) = self.meta.get_mut(cid) {
            meta.ttl = ttl;
        } else {
            return Err(CommonError::InternalError(format!("Block not found: {}", cid)));
        }
        Ok(())
    }

    async fn get_metadata(&self, cid: &Cid) -> Result<Option<icn_dag::BlockMetadata>, CommonError> {
        Ok(self.meta.get(cid).cloned())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Stub mesh network service for testing.
pub struct StubMeshNetworkService {
    staged_bids: Arc<TokioMutex<HashMap<JobId, VecDeque<MeshJobBid>>>>,
    staged_receipts: Arc<TokioMutex<VecDeque<LocalMeshSubmitReceiptMessage>>>,
}

impl StubMeshNetworkService {
    pub fn new() -> Self {
        Self {
            staged_bids: Arc::new(TokioMutex::new(HashMap::new())),
            staged_receipts: Arc::new(TokioMutex::new(VecDeque::new())),
        }
    }

    pub async fn stage_bid(&self, job_id: JobId, bid: MeshJobBid) {
        let mut staged_bids = self.staged_bids.lock().await;
        staged_bids.entry(job_id).or_insert_with(VecDeque::new).push_back(bid);
    }

    pub async fn stage_receipt(&self, receipt_message: LocalMeshSubmitReceiptMessage) {
        let mut staged_receipts = self.staged_receipts.lock().await;
        staged_receipts.push_back(receipt_message);
    }
}

impl Default for StubMeshNetworkService {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for StubMeshNetworkService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StubMeshNetworkService")
    }
}

#[async_trait::async_trait]
impl MeshNetworkService for StubMeshNetworkService {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        log::debug!("StubMeshNetworkService: announce_job called for job {:?}", job.id);
        Ok(())
    }

    async fn announce_proposal(&self, _proposal_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        log::debug!("StubMeshNetworkService: announce_proposal called");
        Ok(())
    }

    async fn announce_vote(&self, _vote_bytes: Vec<u8>) -> Result<(), HostAbiError> {
        log::debug!("StubMeshNetworkService: announce_vote called");
        Ok(())
    }

    async fn collect_bids_for_job(
        &self,
        job_id: &JobId,
        _duration: StdDuration,
    ) -> Result<Vec<MeshJobBid>, HostAbiError> {
        log::debug!("StubMeshNetworkService: collect_bids_for_job called for job {:?}", job_id);
        let mut staged_bids = self.staged_bids.lock().await;
        if let Some(bids) = staged_bids.get_mut(job_id) {
            let mut collected_bids = Vec::new();
            while let Some(bid) = bids.pop_front() {
                collected_bids.push(bid);
            }
            Ok(collected_bids)
        } else {
            Ok(Vec::new())
        }
    }

    async fn notify_executor_of_assignment(
        &self,
        notice: &JobAssignmentNotice,
    ) -> Result<(), HostAbiError> {
        log::debug!(
            "StubMeshNetworkService: notify_executor_of_assignment called for job {:?} -> executor {}",
            notice.job_id,
            notice.executor_did
        );
        Ok(())
    }

    async fn try_receive_receipt(
        &self,
        _job_id: &JobId,
        _expected_executor: &Did,
        _timeout_duration: StdDuration,
    ) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
        log::debug!("StubMeshNetworkService: try_receive_receipt called");
        let mut staged_receipts = self.staged_receipts.lock().await;
        if let Some(receipt_message) = staged_receipts.pop_front() {
            Ok(Some(receipt_message.receipt))
        } else {
            Ok(None)
        }
    }
} 