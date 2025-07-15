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
#[derive(Debug)]
pub struct StubMeshNetworkService {
    staged_bids: Arc<TokioMutex<HashMap<JobId, VecDeque<MeshJobBid>>>>,
    staged_receipts: Arc<TokioMutex<HashMap<JobId, VecDeque<LocalMeshSubmitReceiptMessage>>>>,
    announced_jobs: Arc<TokioMutex<Vec<ActualMeshJob>>>,
    assignment_notices: Arc<TokioMutex<Vec<JobAssignmentNotice>>>,
    job_announcement_tx: Arc<TokioMutex<Option<tokio::sync::mpsc::UnboundedSender<ActualMeshJob>>>>,
}

impl StubMeshNetworkService {
    pub fn new() -> Self {
        Self {
            staged_bids: Arc::new(TokioMutex::new(HashMap::new())),
            staged_receipts: Arc::new(TokioMutex::new(HashMap::new())),
            announced_jobs: Arc::new(TokioMutex::new(Vec::new())),
            assignment_notices: Arc::new(TokioMutex::new(Vec::new())),
            job_announcement_tx: Arc::new(TokioMutex::new(None)),
        }
    }

    /// Set up the job announcement notification channel
    pub async fn setup_job_announcement_channel(&self) -> tokio::sync::mpsc::UnboundedReceiver<ActualMeshJob> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        *self.job_announcement_tx.lock().await = Some(tx);
        rx
    }

    /// Stage a bid for a specific job (for testing)
    pub async fn stage_bid(&self, job_id: JobId, bid: MeshJobBid) {
        let mut staged_bids = self.staged_bids.lock().await;
        staged_bids.entry(job_id).or_insert_with(VecDeque::new).push_back(bid);
    }

    /// Stage a receipt for a specific job (for testing)
    pub async fn stage_receipt(&self, job_id: JobId, receipt_message: LocalMeshSubmitReceiptMessage) {
        let mut staged_receipts = self.staged_receipts.lock().await;
        staged_receipts.entry(job_id).or_insert_with(VecDeque::new).push_back(receipt_message);
    }

    /// Get announced jobs (for testing verification)
    pub async fn get_announced_jobs(&self) -> Vec<ActualMeshJob> {
        self.announced_jobs.lock().await.clone()
    }

    /// Get assignment notices (for testing verification)
    pub async fn get_assignment_notices(&self) -> Vec<JobAssignmentNotice> {
        self.assignment_notices.lock().await.clone()
    }

    /// Clear all staged data (for test cleanup)
    pub async fn clear_all(&self) {
        self.staged_bids.lock().await.clear();
        self.staged_receipts.lock().await.clear();
        self.announced_jobs.lock().await.clear();
        self.assignment_notices.lock().await.clear();
    }
}

impl Default for StubMeshNetworkService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MeshNetworkService for StubMeshNetworkService {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn announce_job(&self, job: &ActualMeshJob) -> Result<(), HostAbiError> {
        log::info!("[StubMeshNetwork] Announcing job {:?}", job.id);
        
        // Store the announced job for testing verification
        let mut announced_jobs = self.announced_jobs.lock().await;
        announced_jobs.push(job.clone());
        drop(announced_jobs); // Release lock before notifying
        
        // Send immediate notification to executor manager if channel is set up
        if let Some(tx) = &*self.job_announcement_tx.lock().await {
            if let Err(e) = tx.send(job.clone()) {
                log::warn!("[StubMeshNetwork] Failed to send job announcement notification: {}", e);
            } else {
                log::debug!("[StubMeshNetwork] Sent immediate job announcement notification for job {:?}", job.id);
            }
        }
        
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
        log::info!("[StubMeshNetwork] Collecting bids for job {:?}", job_id);
        
        let mut staged_bids = self.staged_bids.lock().await;
        if let Some(bids) = staged_bids.get_mut(job_id) {
            let mut collected_bids = Vec::new();
            while let Some(bid) = bids.pop_front() {
                collected_bids.push(bid);
            }
            log::info!("[StubMeshNetwork] Collected {} bids for job {:?}", collected_bids.len(), job_id);
            Ok(collected_bids)
        } else {
            log::info!("[StubMeshNetwork] No bids found for job {:?}", job_id);
            Ok(Vec::new())
        }
    }

    async fn notify_executor_of_assignment(
        &self,
        notice: &JobAssignmentNotice,
    ) -> Result<(), HostAbiError> {
        log::info!(
            "[StubMeshNetwork] Notifying executor {} of assignment for job {:?}",
            notice.executor_did,
            notice.job_id
        );
        
        // Store the assignment notice for testing verification
        let mut assignment_notices = self.assignment_notices.lock().await;
        assignment_notices.push(notice.clone());
        
        Ok(())
    }

    async fn try_receive_receipt(
        &self,
        job_id: &JobId,
        expected_executor: &Did,
        _timeout: StdDuration,
    ) -> Result<Option<IdentityExecutionReceipt>, HostAbiError> {
        log::info!(
            "[StubMeshNetwork] Looking for receipt from {} for job {:?}",
            expected_executor, job_id
        );
        
        let mut staged_receipts = self.staged_receipts.lock().await;
        if let Some(receipts) = staged_receipts.get_mut(job_id) {
            if let Some(receipt_message) = receipts.pop_front() {
                let receipt = &receipt_message.receipt;
                if receipt.executor_did == *expected_executor {
                    log::info!("[StubMeshNetwork] Found receipt from {} for job {:?}", expected_executor, job_id);
                    return Ok(Some(receipt.clone()));
                } else {
                    // Put it back if it's from a different executor
                    receipts.push_front(receipt_message);
                }
            }
        }
        
        log::info!("[StubMeshNetwork] No receipt found from {} for job {:?}", expected_executor, job_id);
        Ok(None)
    }

    async fn submit_bid_for_job(
        &self,
        bid: &icn_mesh::MeshJobBid,
    ) -> Result<(), HostAbiError> {
        log::info!("[StubMeshNetwork] Submitting bid for job {:?}: {} mana", bid.job_id, bid.price_mana);
        
        // For stub implementation, we'll just stage the bid
        self.stage_bid(bid.job_id.clone(), bid.clone()).await;
        
        Ok(())
    }

    async fn submit_execution_receipt(
        &self,
        receipt: &icn_identity::ExecutionReceipt,
    ) -> Result<(), HostAbiError> {
        log::info!("[StubMeshNetwork] Submitting execution receipt for job {:?}", receipt.job_id);
        
        // For stub implementation, we'll stage the receipt
        let receipt_message = LocalMeshSubmitReceiptMessage {
            receipt: receipt.clone(),
        };
        
        self.stage_receipt(icn_mesh::JobId::from(receipt.job_id.clone()), receipt_message).await;
        
        Ok(())
    }
} 