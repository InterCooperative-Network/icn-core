// icn-dag/src/snapshot.rs
//! DAG snapshot export/import for federation migration and testing

use crate::StorageService;
use icn_common::{Cid, CommonError, DagBlock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::Path;

/// DAG snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Version of the snapshot format
    pub format_version: String,
    /// Timestamp when snapshot was created
    pub created_at: u64,
    /// Node that created the snapshot
    pub created_by: String,
    /// Total number of blocks in snapshot
    pub block_count: usize,
    /// Total size of all blocks in bytes
    pub total_size: u64,
    /// Snapshot description
    pub description: String,
    /// Hash of the snapshot content
    pub content_hash: String,
}

/// Configuration for snapshot creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Include all blocks (true) or only specific roots (false)
    pub include_all: bool,
    /// Root CIDs to include (if include_all is false)
    pub root_cids: Vec<Cid>,
    /// Maximum age of blocks to include (in seconds, None for all)
    pub max_age_seconds: Option<u64>,
    /// Compression to use
    pub compression: CompressionType,
    /// Include metadata with blocks
    pub include_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            include_all: false,
            root_cids: Vec::new(),
            max_age_seconds: None,
            compression: CompressionType::Zstd,
            include_metadata: false,
        }
    }
}

/// A complete DAG snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagSnapshot {
    pub metadata: SnapshotMetadata,
    pub blocks: HashMap<Cid, DagBlock>,
    pub block_metadata: Option<HashMap<Cid, crate::BlockMetadata>>,
}

/// Progress callback for snapshot operations
pub trait SnapshotProgress {
    fn on_progress(&self, processed: usize, total: usize, message: &str);
}

/// No-op progress implementation
pub struct NoProgress;

impl SnapshotProgress for NoProgress {
    fn on_progress(&self, _processed: usize, _total: usize, _message: &str) {}
}

/// DAG snapshot manager
pub struct DagSnapshots<S: StorageService<DagBlock>> {
    store: S,
}

impl<S: StorageService<DagBlock>> DagSnapshots<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Create a snapshot of the DAG
    #[cfg(feature = "async")]
    pub async fn create_snapshot<P: SnapshotProgress>(
        &self,
        config: SnapshotConfig,
        description: String,
        progress: &P,
    ) -> Result<DagSnapshot, CommonError> {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        progress.on_progress(0, 1, "Collecting blocks to include...");

        // Determine which blocks to include
        let blocks_to_include = if config.include_all {
            self.get_all_blocks().await?
        } else {
            self.get_blocks_from_roots(&config.root_cids).await?
        };

        progress.on_progress(1, blocks_to_include.len() + 1, "Filtering blocks...");

        // Filter by age if specified
        let filtered_blocks = if let Some(max_age) = config.max_age_seconds {
            blocks_to_include
                .into_iter()
                .filter(|(_, block)| start_time - block.timestamp <= max_age)
                .collect()
        } else {
            blocks_to_include
        };

        progress.on_progress(
            filtered_blocks.len(),
            filtered_blocks.len() + 1,
            "Creating snapshot...",
        );

        // Get block metadata if requested
        let block_metadata = if config.include_metadata {
            Some(self.get_blocks_metadata(&filtered_blocks).await?)
        } else {
            None
        };

        // Calculate total size and content hash
        let total_size = filtered_blocks
            .values()
            .map(|block| block.data.len() as u64)
            .sum();

        let content_hash = self.calculate_content_hash(&filtered_blocks);

        let metadata = SnapshotMetadata {
            format_version: "1.0".to_string(),
            created_at: start_time,
            created_by: "icn-dag".to_string(), // TODO: Get actual node ID
            block_count: filtered_blocks.len(),
            total_size,
            description,
            content_hash,
        };

        progress.on_progress(
            filtered_blocks.len() + 1,
            filtered_blocks.len() + 1,
            "Snapshot complete",
        );

        Ok(DagSnapshot {
            metadata,
            blocks: filtered_blocks,
            block_metadata,
        })
    }

    /// Synchronous version of create_snapshot
    pub fn create_snapshot_sync<P: SnapshotProgress>(
        &self,
        config: SnapshotConfig,
        description: String,
        progress: &P,
    ) -> Result<DagSnapshot, CommonError> {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        progress.on_progress(0, 1, "Collecting blocks to include...");

        // Determine which blocks to include
        let blocks_to_include = if config.include_all {
            self.get_all_blocks_sync()?
        } else {
            self.get_blocks_from_roots_sync(&config.root_cids)?
        };

        progress.on_progress(1, blocks_to_include.len() + 1, "Filtering blocks...");

        // Filter by age if specified
        let filtered_blocks = if let Some(max_age) = config.max_age_seconds {
            blocks_to_include
                .into_iter()
                .filter(|(_, block)| start_time - block.timestamp <= max_age)
                .collect()
        } else {
            blocks_to_include
        };

        progress.on_progress(
            filtered_blocks.len(),
            filtered_blocks.len() + 1,
            "Creating snapshot...",
        );

        // Get block metadata if requested
        let block_metadata = if config.include_metadata {
            Some(self.get_blocks_metadata_sync(&filtered_blocks)?)
        } else {
            None
        };

        // Calculate total size and content hash
        let total_size = filtered_blocks
            .values()
            .map(|block| block.data.len() as u64)
            .sum();

        let content_hash = self.calculate_content_hash(&filtered_blocks);

        let metadata = SnapshotMetadata {
            format_version: "1.0".to_string(),
            created_at: start_time,
            created_by: "icn-dag".to_string(), // TODO: Get actual node ID
            block_count: filtered_blocks.len(),
            total_size,
            description,
            content_hash,
        };

        progress.on_progress(
            filtered_blocks.len() + 1,
            filtered_blocks.len() + 1,
            "Snapshot complete",
        );

        Ok(DagSnapshot {
            metadata,
            blocks: filtered_blocks,
            block_metadata,
        })
    }

    /// Export snapshot to file
    pub fn export_snapshot<W: Write>(
        &self,
        snapshot: &DagSnapshot,
        writer: W,
        compression: CompressionType,
    ) -> Result<(), CommonError> {
        let serialized = serde_json::to_vec(snapshot)
            .map_err(|e| CommonError::SerializationError(e.to_string()))?;

        match compression {
            CompressionType::None => {
                let mut writer = writer;
                writer
                    .write_all(&serialized)
                    .map_err(|e| CommonError::IoError(e.to_string()))?;
            }
            CompressionType::Gzip => {
                use flate2::write::GzEncoder;
                use flate2::Compression;

                let mut encoder = GzEncoder::new(writer, Compression::default());
                encoder
                    .write_all(&serialized)
                    .map_err(|e| CommonError::IoError(e.to_string()))?;
                encoder
                    .finish()
                    .map_err(|e| CommonError::IoError(e.to_string()))?;
            }
            CompressionType::Zstd => {
                // TODO: Implement Zstd compression
                return Err(CommonError::NotImplemented(
                    "Zstd compression not yet implemented".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Import snapshot from file
    pub fn import_snapshot<R: Read>(
        &self,
        reader: R,
        compression: CompressionType,
    ) -> Result<DagSnapshot, CommonError> {
        let data = match compression {
            CompressionType::None => {
                let mut reader = reader;
                let mut data = Vec::new();
                reader
                    .read_to_end(&mut data)
                    .map_err(|e| CommonError::IoError(e.to_string()))?;
                data
            }
            CompressionType::Gzip => {
                use flate2::read::GzDecoder;

                let mut decoder = GzDecoder::new(reader);
                let mut data = Vec::new();
                decoder
                    .read_to_end(&mut data)
                    .map_err(|e| CommonError::IoError(e.to_string()))?;
                data
            }
            CompressionType::Zstd => {
                return Err(CommonError::NotImplemented(
                    "Zstd decompression not yet implemented".to_string(),
                ));
            }
        };

        let snapshot: DagSnapshot = serde_json::from_slice(&data)
            .map_err(|e| CommonError::SerializationError(e.to_string()))?;

        Ok(snapshot)
    }

    /// Apply snapshot to the store
    #[cfg(feature = "async")]
    pub async fn apply_snapshot<P: SnapshotProgress>(
        &mut self,
        snapshot: DagSnapshot,
        progress: &P,
    ) -> Result<(), CommonError> {
        let total_blocks = snapshot.blocks.len();
        let mut processed = 0;

        for (cid, block) in snapshot.blocks {
            self.store.put(&block)?;
            processed += 1;
            progress.on_progress(processed, total_blocks, "Applying snapshot...");
        }

        // Apply metadata if included
        if let Some(metadata_map) = snapshot.block_metadata {
            for (cid, metadata) in metadata_map {
                // TODO: Apply block metadata to store
                // This would require extending the DagStore trait
            }
        }

        progress.on_progress(total_blocks, total_blocks, "Snapshot applied");
        Ok(())
    }

    /// Synchronous version of apply_snapshot
    pub fn apply_snapshot_sync<P: SnapshotProgress>(
        &mut self,
        snapshot: DagSnapshot,
        progress: &P,
    ) -> Result<(), CommonError> {
        let total_blocks = snapshot.blocks.len();
        let mut processed = 0;

        for (_cid, block) in snapshot.blocks {
            self.store.put(&block)?;
            processed += 1;
            progress.on_progress(processed, total_blocks, "Applying snapshot...");
        }

        progress.on_progress(total_blocks, total_blocks, "Snapshot applied");
        Ok(())
    }

    /// Verify snapshot integrity
    pub fn verify_snapshot(&self, snapshot: &DagSnapshot) -> Result<bool, CommonError> {
        // Verify content hash
        let calculated_hash = self.calculate_content_hash(&snapshot.blocks);
        if calculated_hash != snapshot.metadata.content_hash {
            return Ok(false);
        }

        // Verify block count
        if snapshot.blocks.len() != snapshot.metadata.block_count {
            return Ok(false);
        }

        // Verify total size
        let calculated_size: u64 = snapshot
            .blocks
            .values()
            .map(|block| block.data.len() as u64)
            .sum();
        if calculated_size != snapshot.metadata.total_size {
            return Ok(false);
        }

        // Verify block integrity
        for (cid, block) in &snapshot.blocks {
            let calculated_cid = block.cid.clone();
            if calculated_cid != *cid {
                return Ok(false);
            }
        }

        Ok(true)
    }

    #[cfg(feature = "async")]
    async fn get_all_blocks(&self) -> Result<HashMap<Cid, DagBlock>, CommonError> {
        // This would need to be implemented by each store type
        Ok(HashMap::new())
    }

    fn get_all_blocks_sync(&self) -> Result<HashMap<Cid, DagBlock>, CommonError> {
        // This would need to be implemented by each store type
        Ok(HashMap::new())
    }

    #[cfg(feature = "async")]
    async fn get_blocks_from_roots(
        &self,
        _roots: &[Cid],
    ) -> Result<HashMap<Cid, DagBlock>, CommonError> {
        // Traverse from roots and collect all reachable blocks
        // This would need to be implemented by each store type
        Ok(HashMap::new())
    }

    fn get_blocks_from_roots_sync(
        &self,
        _roots: &[Cid],
    ) -> Result<HashMap<Cid, DagBlock>, CommonError> {
        // Traverse from roots and collect all reachable blocks
        // This would need to be implemented by each store type
        Ok(HashMap::new())
    }

    #[cfg(feature = "async")]
    async fn get_blocks_metadata(
        &self,
        _blocks: &HashMap<Cid, DagBlock>,
    ) -> Result<HashMap<Cid, crate::BlockMetadata>, CommonError> {
        // Get metadata for all blocks
        Ok(HashMap::new())
    }

    fn get_blocks_metadata_sync(
        &self,
        _blocks: &HashMap<Cid, DagBlock>,
    ) -> Result<HashMap<Cid, crate::BlockMetadata>, CommonError> {
        // Get metadata for all blocks
        Ok(HashMap::new())
    }

    fn calculate_content_hash(&self, blocks: &HashMap<Cid, DagBlock>) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Sort blocks by CID for deterministic hash
        let mut sorted_blocks: Vec<_> = blocks.iter().collect();
        sorted_blocks.sort_by_key(|(cid, _)| cid.to_string());

        for (cid, block) in sorted_blocks {
            hasher.update(cid.to_string().as_bytes());
            hasher.update(&block.data);
        }

        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sled_store::SledDagStore;
    use tempfile::TempDir;

    #[test]
    fn test_snapshot_config_default() {
        let config = SnapshotConfig::default();
        assert!(!config.include_all);
        assert!(config.root_cids.is_empty());
        assert!(matches!(config.compression, CompressionType::Zstd));
    }

    #[test]
    fn test_snapshot_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store = SledDagStore::new(temp_dir.path()).unwrap();
        let snapshots = DagSnapshots::new(store);

        // Just verify it can be created
        // The actual functionality would need a working store implementation
    }

    #[test]
    fn test_no_progress() {
        let progress = NoProgress;
        progress.on_progress(50, 100, "test message");
        // Should not panic or error
    }
}
