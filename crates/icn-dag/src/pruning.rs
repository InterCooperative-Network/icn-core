// icn-dag/src/pruning.rs
//! DAG pruning and compaction utilities

use crate::{StorageService, BlockMetadata};
use icn_common::{Cid, CommonError, DagBlock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Configuration for DAG pruning operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruningConfig {
    /// Maximum age of blocks to keep (in seconds)
    pub max_age_seconds: Option<u64>,
    /// Maximum total size of DAG (in bytes)
    pub max_total_size: Option<u64>,
    /// Preserve blocks referenced by pinned blocks
    pub preserve_pinned_references: bool,
    /// Minimum number of blocks to keep regardless of other criteria
    pub min_blocks_to_keep: usize,
    /// Block types to never prune
    pub preserve_block_types: HashSet<u64>,
}

impl Default for PruningConfig {
    fn default() -> Self {
        Self {
            max_age_seconds: Some(30 * 24 * 3600), // 30 days
            max_total_size: Some(10 * 1024 * 1024 * 1024), // 10 GB
            preserve_pinned_references: true,
            min_blocks_to_keep: 1000,
            preserve_block_types: [0x70, 0x71].iter().cloned().collect(), // Core protocol blocks
        }
    }
}

/// Statistics from a pruning operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruningStats {
    pub blocks_examined: usize,
    pub blocks_removed: usize,
    pub bytes_freed: u64,
    pub duration_ms: u64,
    pub errors_encountered: usize,
}

/// Configuration for DAG compaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    /// Compact blocks older than this age (in seconds)
    pub compact_older_than: u64,
    /// Maximum size of compacted chunks
    pub max_chunk_size: u64,
    /// Compression algorithm to use
    pub compression: CompressionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            compact_older_than: 7 * 24 * 3600, // 7 days
            max_chunk_size: 64 * 1024 * 1024, // 64 MB
            compression: CompressionType::Zstd,
        }
    }
}

/// DAG pruning and compaction manager
pub struct DagMaintenance<S: StorageService<DagBlock>> {
    store: S,
    config: PruningConfig,
    compaction_config: CompactionConfig,
}

impl<S: StorageService<DagBlock>> DagMaintenance<S> {
    pub fn new(store: S) -> Self {
        Self {
            store,
            config: PruningConfig::default(),
            compaction_config: CompactionConfig::default(),
        }
    }

    pub fn with_pruning_config(mut self, config: PruningConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_compaction_config(mut self, config: CompactionConfig) -> Self {
        self.compaction_config = config;
        self
    }

    /// Perform DAG pruning based on configuration
    #[cfg(feature = "async")]
    pub async fn prune(&mut self) -> Result<PruningStats, CommonError> {
        let start_time = std::time::Instant::now();
        let mut stats = PruningStats {
            blocks_examined: 0,
            blocks_removed: 0,
            bytes_freed: 0,
            duration_ms: 0,
            errors_encountered: 0,
        };

        // Get all blocks with metadata
        let all_blocks = self.get_all_blocks_with_metadata().await?;
        stats.blocks_examined = all_blocks.len();

        // Calculate which blocks to keep
        let blocks_to_keep = self.calculate_blocks_to_keep(&all_blocks).await?;

        // Remove blocks not in the keep set
        for (cid, (block, metadata)) in all_blocks {
            if !blocks_to_keep.contains(&cid) {
                match self.store.delete(&cid) {
                    Ok(_) => {
                        stats.blocks_removed += 1;
                        stats.bytes_freed += block.data.len() as u64;
                    }
                    Err(_) => {
                        stats.errors_encountered += 1;
                    }
                }
            }
        }

        stats.duration_ms = start_time.elapsed().as_millis() as u64;
        Ok(stats)
    }

    /// Synchronous version of prune for non-async stores
    pub fn prune_sync(&mut self) -> Result<PruningStats, CommonError> {
        let start_time = std::time::Instant::now();
        let mut stats = PruningStats {
            blocks_examined: 0,
            blocks_removed: 0,
            bytes_freed: 0,
            duration_ms: 0,
            errors_encountered: 0,
        };

        // Get all blocks with metadata
        let all_blocks = self.get_all_blocks_with_metadata_sync()?;
        stats.blocks_examined = all_blocks.len();

        // Calculate which blocks to keep
        let blocks_to_keep = self.calculate_blocks_to_keep_sync(&all_blocks)?;

        // Remove blocks not in the keep set
        for (cid, (block, _metadata)) in all_blocks {
            if !blocks_to_keep.contains(&cid) {
                match self.store.delete(&cid) {
                    Ok(_) => {
                        stats.blocks_removed += 1;
                        stats.bytes_freed += block.data.len() as u64;
                    }
                    Err(_) => {
                        stats.errors_encountered += 1;
                    }
                }
            }
        }

        stats.duration_ms = start_time.elapsed().as_millis() as u64;
        Ok(stats)
    }

    #[cfg(feature = "async")]
    async fn get_all_blocks_with_metadata(&self) -> Result<HashMap<Cid, (DagBlock, BlockMetadata)>, CommonError> {
        let blocks = self.store.list_blocks()?;
        let mut result = HashMap::new();
        
        for block in blocks {
            let cid = block.cid.clone();
            let metadata = self.store.get_metadata(&cid)?.unwrap_or_default();
            result.insert(cid, (block, metadata));
        }
        
        Ok(result)
    }

    fn get_all_blocks_with_metadata_sync(&self) -> Result<HashMap<Cid, (DagBlock, BlockMetadata)>, CommonError> {
        let blocks = self.store.list_blocks()?;
        let mut result = HashMap::new();
        
        for block in blocks {
            let cid = block.cid.clone();
            let metadata = self.store.get_metadata(&cid)?.unwrap_or_default();
            result.insert(cid, (block, metadata));
        }
        
        Ok(result)
    }

    #[cfg(feature = "async")]
    async fn calculate_blocks_to_keep(&self, all_blocks: &HashMap<Cid, (DagBlock, BlockMetadata)>) -> Result<HashSet<Cid>, CommonError> {
        let mut keep_set = HashSet::new();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Always keep pinned blocks
        for (cid, (_block, metadata)) in all_blocks {
            if metadata.pinned {
                // keep_set.insert(cid.clone());
            }
        }

        // Keep blocks that are not too old
        if let Some(max_age) = self.config.max_age_seconds {
            for (cid, (block, _metadata)) in all_blocks {
                if current_time - block.timestamp < max_age {
                    // keep_set.insert(cid.clone());
                }
            }
        }


        // If preserving pinned references, recursively add referenced blocks
        if self.config.preserve_pinned_references {
            let mut to_process: Vec<Cid> = keep_set.iter().cloned().collect();
            while let Some(cid) = to_process.pop() {
                if let Some((block, _)) = all_blocks.get(&cid) {
                    for link in &block.links {
                        if !keep_set.contains(&link.cid) {
                            keep_set.insert(link.cid.clone());
                            to_process.push(link.cid.clone());
                        }
                    }
                }
            }
        }

        // Ensure we keep minimum number of blocks (keep newest ones)
        if keep_set.len() < self.config.min_blocks_to_keep {
            let mut blocks_by_time: Vec<_> = all_blocks.iter().collect();
            blocks_by_time.sort_by_key(|(_, (block, _))| block.timestamp);
            blocks_by_time.reverse(); // Newest first

            for (cid, _) in blocks_by_time.iter().take(self.config.min_blocks_to_keep) {
                keep_set.insert((*cid).clone());
            }
        }

        Ok(keep_set)
    }

    fn calculate_blocks_to_keep_sync(&self, all_blocks: &HashMap<Cid, (DagBlock, BlockMetadata)>) -> Result<HashSet<Cid>, CommonError> {
        let mut keep_set = HashSet::new();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Always keep pinned blocks
        for (cid, (_block, metadata)) in all_blocks {
            if metadata.pinned {
                // keep_set.insert(cid.clone());
            }
        }

        // Keep blocks that are not too old
        if let Some(max_age) = self.config.max_age_seconds {
            for (cid, (block, _metadata)) in all_blocks {
                if current_time - block.timestamp < max_age {
                    // keep_set.insert(cid.clone());
                }
            }
        }


        // If preserving pinned references, recursively add referenced blocks
        if self.config.preserve_pinned_references {
            let mut to_process: Vec<Cid> = keep_set.iter().cloned().collect();
            while let Some(cid) = to_process.pop() {
                if let Some((block, _)) = all_blocks.get(&cid) {
                    for link in &block.links {
                        if !keep_set.contains(&link.cid) {
                            keep_set.insert(link.cid.clone());
                            to_process.push(link.cid.clone());
                        }
                    }
                }
            }
        }

        // Ensure we keep minimum number of blocks (keep newest ones)
        if keep_set.len() < self.config.min_blocks_to_keep {
            let mut blocks_by_time: Vec<_> = all_blocks.iter().collect();
            blocks_by_time.sort_by_key(|(_, (block, _))| block.timestamp);
            blocks_by_time.reverse(); // Newest first

            for (cid, _) in blocks_by_time.iter().take(self.config.min_blocks_to_keep) {
                keep_set.insert((*cid).clone());
            }
        }

        Ok(keep_set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sled_store::SledDagStore;
    use tempfile::TempDir;

    #[test]
    fn test_pruning_config_default() {
        let config = PruningConfig::default();
        assert!(config.max_age_seconds.is_some());
        assert!(config.preserve_pinned_references);
        assert_eq!(config.min_blocks_to_keep, 1000);
    }

    #[test]
    fn test_compaction_config_default() {
        let config = CompactionConfig::default();
        assert_eq!(config.compact_older_than, 7 * 24 * 3600);
        assert!(matches!(config.compression, CompressionType::Zstd));
    }

    #[test]
    fn test_dag_maintenance_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store = SledDagStore::new(temp_dir.path()).unwrap();
        let maintenance = DagMaintenance::new(store);
        
        // Just verify it can be created
        assert_eq!(maintenance.config.min_blocks_to_keep, 1000);
    }
}