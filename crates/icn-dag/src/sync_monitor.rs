// icn-dag/src/sync_monitor.rs
//! DAG synchronization monitoring and missing block detection

use crate::StorageService;
use icn_common::{Cid, CommonError, DagBlock, Did};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Information about a missing block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingBlock {
    pub cid: Cid,
    pub referenced_by: HashSet<Cid>,
    pub first_detected: u64,
    pub last_requested: Option<u64>,
    pub request_count: u32,
    pub priority: BlockPriority,
}

/// Priority levels for missing blocks
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Sync monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// How often to check for missing blocks (in seconds)
    pub check_interval: u64,
    /// Maximum age for a missing block before alert (in seconds)
    pub max_missing_age: u64,
    /// Maximum number of missing blocks to track
    pub max_tracked_missing: usize,
    /// Enable automatic re-anchoring of missing blocks
    pub auto_reanchor: bool,
    /// Nodes to request missing blocks from
    pub peer_nodes: Vec<Did>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            check_interval: 300,   // 5 minutes
            max_missing_age: 3600, // 1 hour
            max_tracked_missing: 1000,
            auto_reanchor: false,
            peer_nodes: Vec::new(),
        }
    }
}

/// Sync monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub total_blocks: usize,
    pub missing_blocks: usize,
    pub missing_critical: usize,
    pub missing_high: usize,
    pub missing_normal: usize,
    pub missing_low: usize,
    pub last_check: u64,
    pub sync_health_score: f64, // 0.0 (poor) to 1.0 (perfect)
}

/// Alert for missing blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingBlockAlert {
    pub missing_blocks: Vec<MissingBlock>,
    pub severity: AlertSeverity,
    pub timestamp: u64,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// DAG synchronization monitor
pub struct DagSyncMonitor<S: StorageService<DagBlock>> {
    _store: S,
    config: SyncConfig,
    missing_blocks: HashMap<Cid, MissingBlock>,
    last_check: SystemTime,
}

impl<S: StorageService<DagBlock>> DagSyncMonitor<S> {
    pub fn new(store: S, config: SyncConfig) -> Self {
        Self {
            _store: store,
            config,
            missing_blocks: HashMap::new(),
            last_check: SystemTime::now(),
        }
    }

    /// Check for missing blocks and update tracking
    #[cfg(feature = "async")]
    pub async fn check_missing_blocks(&mut self) -> Result<SyncStats, CommonError> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Get all blocks to check their references
        let all_blocks = self.get_all_blocks().await?;
        let mut all_referenced_cids = HashSet::new();
        let mut existing_cids = HashSet::new();

        // Collect all existing CIDs and referenced CIDs
        for (cid, block) in &all_blocks {
            existing_cids.insert(cid.clone());
            for link in &block.links {
                all_referenced_cids.insert(link.cid.clone());
            }
        }

        // Find missing blocks (referenced but not present)
        let mut new_missing = HashSet::new();
        for referenced_cid in &all_referenced_cids {
            if !existing_cids.contains(referenced_cid) {
                new_missing.insert(referenced_cid.clone());
            }
        }

        // Update missing blocks tracking
        for cid in new_missing {
            if !self.missing_blocks.contains_key(&cid) {
                // Find which blocks reference this missing block
                let mut referenced_by = HashSet::new();
                for (block_cid, block) in &all_blocks {
                    for link in &block.links {
                        if link.cid == cid {
                            referenced_by.insert(block_cid.clone());
                        }
                    }
                }

                let priority = self.calculate_block_priority(&cid, &referenced_by, &all_blocks);

                self.missing_blocks.insert(
                    cid.clone(),
                    MissingBlock {
                        cid,
                        referenced_by,
                        first_detected: current_time,
                        last_requested: None,
                        request_count: 0,
                        priority,
                    },
                );
            }
        }

        // Remove blocks that are no longer missing
        let missing_cids: Vec<Cid> = self.missing_blocks.keys().cloned().collect();
        for cid in missing_cids {
            if existing_cids.contains(&cid) {
                self.missing_blocks.remove(&cid);
            }
        }

        // Limit tracked missing blocks
        if self.missing_blocks.len() > self.config.max_tracked_missing {
            let mut sorted_missing: Vec<_> = self.missing_blocks.iter().collect();
            sorted_missing.sort_by_key(|(_, mb)| (mb.priority, mb.first_detected));

            // Keep highest priority and most recent
            let to_keep: HashSet<Cid> = sorted_missing
                .into_iter()
                .take(self.config.max_tracked_missing)
                .map(|(cid, _)| cid.clone())
                .collect();

            self.missing_blocks.retain(|cid, _| to_keep.contains(cid));
        }

        // Calculate statistics
        let stats = self.calculate_sync_stats(current_time, all_blocks.len());
        self.last_check = SystemTime::now();

        Ok(stats)
    }

    /// Synchronous version for non-async stores
    pub fn check_missing_blocks_sync(&mut self) -> Result<SyncStats, CommonError> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Get all blocks to check their references
        let all_blocks = self.get_all_blocks_sync()?;
        let mut all_referenced_cids = HashSet::new();
        let mut existing_cids = HashSet::new();

        // Collect all existing CIDs and referenced CIDs
        for (cid, block) in &all_blocks {
            existing_cids.insert(cid.clone());
            for link in &block.links {
                all_referenced_cids.insert(link.cid.clone());
            }
        }

        // Find missing blocks (referenced but not present)
        let mut new_missing = HashSet::new();
        for referenced_cid in &all_referenced_cids {
            if !existing_cids.contains(referenced_cid) {
                new_missing.insert(referenced_cid.clone());
            }
        }

        // Update missing blocks tracking
        for cid in new_missing {
            if !self.missing_blocks.contains_key(&cid) {
                // Find which blocks reference this missing block
                let mut referenced_by = HashSet::new();
                for (block_cid, block) in &all_blocks {
                    for link in &block.links {
                        if link.cid == cid {
                            referenced_by.insert(block_cid.clone());
                        }
                    }
                }

                let priority = self.calculate_block_priority(&cid, &referenced_by, &all_blocks);

                self.missing_blocks.insert(
                    cid.clone(),
                    MissingBlock {
                        cid,
                        referenced_by,
                        first_detected: current_time,
                        last_requested: None,
                        request_count: 0,
                        priority,
                    },
                );
            }
        }

        // Remove blocks that are no longer missing
        let missing_cids: Vec<Cid> = self.missing_blocks.keys().cloned().collect();
        for cid in missing_cids {
            if existing_cids.contains(&cid) {
                self.missing_blocks.remove(&cid);
            }
        }

        // Calculate statistics
        let stats = self.calculate_sync_stats(current_time, all_blocks.len());
        self.last_check = SystemTime::now();

        Ok(stats)
    }

    /// Generate alerts for missing blocks
    pub fn generate_alerts(&self, current_time: u64) -> Vec<MissingBlockAlert> {
        let mut alerts = Vec::new();

        // Group missing blocks by severity
        let mut critical_blocks = Vec::new();
        let mut high_blocks = Vec::new();
        let mut warning_blocks = Vec::new();

        for missing_block in self.missing_blocks.values() {
            let age = current_time - missing_block.first_detected;

            if missing_block.priority == BlockPriority::Critical {
                critical_blocks.push(missing_block.clone());
            } else if age > self.config.max_missing_age {
                if missing_block.priority == BlockPriority::High {
                    high_blocks.push(missing_block.clone());
                } else {
                    warning_blocks.push(missing_block.clone());
                }
            }
        }

        // Create alerts
        if !critical_blocks.is_empty() {
            alerts.push(MissingBlockAlert {
                missing_blocks: critical_blocks,
                severity: AlertSeverity::Critical,
                timestamp: current_time,
                recommended_actions: vec![
                    "Immediate investigation required".to_string(),
                    "Check network connectivity".to_string(),
                    "Manually request blocks from peers".to_string(),
                ],
            });
        }

        if !high_blocks.is_empty() {
            alerts.push(MissingBlockAlert {
                missing_blocks: high_blocks,
                severity: AlertSeverity::Error,
                timestamp: current_time,
                recommended_actions: vec![
                    "Request missing blocks from peers".to_string(),
                    "Check sync configuration".to_string(),
                ],
            });
        }

        if !warning_blocks.is_empty() {
            alerts.push(MissingBlockAlert {
                missing_blocks: warning_blocks,
                severity: AlertSeverity::Warning,
                timestamp: current_time,
                recommended_actions: vec![
                    "Monitor sync progress".to_string(),
                    "Consider increasing sync frequency".to_string(),
                ],
            });
        }

        alerts
    }

    /// Request a missing block from peers
    #[cfg(feature = "async")]
    pub async fn request_missing_block(&mut self, cid: &Cid) -> Result<bool, CommonError> {
        if let Some(missing_block) = self.missing_blocks.get_mut(cid) {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            missing_block.last_requested = Some(current_time);
            missing_block.request_count += 1;

            // Implement actual peer request logic using federation sync
            // Create a block request message for the missing block
            println!(
                "Requesting missing block {} from {} peers (attempt {})",
                cid,
                self.config.peer_nodes.len(),
                missing_block.request_count
            );

            // In a real implementation, this would:
            // 1. Create a SyncMessage::BlockRequest
            // 2. Send it to available peers via network service
            // 3. Handle the response and update missing block status

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn calculate_block_priority(
        &self,
        _cid: &Cid,
        referenced_by: &HashSet<Cid>,
        all_blocks: &HashMap<Cid, DagBlock>,
    ) -> BlockPriority {
        // Calculate priority based on how many blocks reference this missing block
        // and the importance of those blocks

        let reference_count = referenced_by.len();
        let mut has_recent_referrer = false;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for referrer_cid in referenced_by {
            if let Some(referrer_block) = all_blocks.get(referrer_cid) {
                // If a recent block references this missing block, it's higher priority
                if current_time - referrer_block.timestamp < 3600 {
                    // 1 hour
                    has_recent_referrer = true;
                    break;
                }
            }
        }

        if reference_count > 10 || has_recent_referrer {
            BlockPriority::Critical
        } else if reference_count > 5 {
            BlockPriority::High
        } else if reference_count > 1 {
            BlockPriority::Normal
        } else {
            BlockPriority::Low
        }
    }

    fn calculate_sync_stats(&self, current_time: u64, total_blocks: usize) -> SyncStats {
        let missing_blocks = self.missing_blocks.len();
        let (missing_critical, missing_high, missing_normal, missing_low) = self
            .missing_blocks
            .values()
            .fold((0, 0, 0, 0), |acc, mb| match mb.priority {
                BlockPriority::Critical => (acc.0 + 1, acc.1, acc.2, acc.3),
                BlockPriority::High => (acc.0, acc.1 + 1, acc.2, acc.3),
                BlockPriority::Normal => (acc.0, acc.1, acc.2 + 1, acc.3),
                BlockPriority::Low => (acc.0, acc.1, acc.2, acc.3 + 1),
            });

        // Calculate health score (0.0 to 1.0)
        let health_score = if total_blocks == 0 {
            1.0
        } else {
            let missing_ratio = missing_blocks as f64 / total_blocks as f64;
            let critical_penalty = missing_critical as f64 * 0.1;
            let high_penalty = missing_high as f64 * 0.05;

            (1.0 - missing_ratio - critical_penalty - high_penalty).max(0.0)
        };

        SyncStats {
            total_blocks,
            missing_blocks,
            missing_critical,
            missing_high,
            missing_normal,
            missing_low,
            last_check: current_time,
            sync_health_score: health_score,
        }
    }

    #[cfg(feature = "async")]
    async fn get_all_blocks(&self) -> Result<HashMap<Cid, DagBlock>, CommonError> {
        let blocks = self._store.list_blocks()?;
        let mut block_map = HashMap::new();
        for block in blocks {
            block_map.insert(block.cid.clone(), block);
        }
        Ok(block_map)
    }

    fn get_all_blocks_sync(&self) -> Result<HashMap<Cid, DagBlock>, CommonError> {
        let blocks = self._store.list_blocks()?;
        let mut block_map = HashMap::new();
        for block in blocks {
            block_map.insert(block.cid.clone(), block);
        }
        Ok(block_map)
    }

    /// Get current missing blocks
    pub fn get_missing_blocks(&self) -> &HashMap<Cid, MissingBlock> {
        &self.missing_blocks
    }

    /// Get missing blocks by priority
    pub fn get_missing_blocks_by_priority(&self, priority: BlockPriority) -> Vec<&MissingBlock> {
        self.missing_blocks
            .values()
            .filter(|mb| mb.priority == priority)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sled_store::SledDagStore;
    use tempfile::TempDir;

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert_eq!(config.check_interval, 300);
        assert_eq!(config.max_missing_age, 3600);
        assert!(!config.auto_reanchor);
    }

    #[test]
    fn test_block_priority_ordering() {
        assert!(BlockPriority::Critical > BlockPriority::High);
        assert!(BlockPriority::High > BlockPriority::Normal);
        assert!(BlockPriority::Normal > BlockPriority::Low);
    }

    #[test]
    fn test_sync_monitor_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store = SledDagStore::new(temp_dir.path().to_path_buf()).unwrap();
        let config = SyncConfig::default();
        let monitor = DagSyncMonitor::new(store, config);

        assert_eq!(monitor.missing_blocks.len(), 0);
    }
}
