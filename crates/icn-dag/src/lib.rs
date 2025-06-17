#![doc = include_str!("../README.md")]
#![allow(clippy::uninlined_format_args)]

//! # ICN DAG Crate
//! This crate implements or defines interfaces for content-addressed Directed Acyclic Graph (DAG)
//! storage and manipulation, crucial for the InterCooperative Network (ICN) data model.
//! It handles DAG primitives, content addressing, storage abstraction, and serialization formats.

#[cfg(test)]
use icn_common::compute_merkle_cid;
#[cfg(test)]
use icn_common::DagLink;
use icn_common::{Cid, CommonError, DagBlock, NodeInfo, ICN_CORE_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions}; // For FileDagStore
use std::io::{Read, Write}; // Removed Seek, SeekFrom
use std::path::PathBuf; // For FileDagStore
use std::sync::Mutex; // For basic interior mutability for the global store // For FileDagStore block serialization

pub mod index;
#[cfg(feature = "persist-rocksdb")]
pub mod rocksdb_store;
#[cfg(feature = "persist-sled")]
pub mod sled_store;
#[cfg(feature = "persist-sqlite")]
pub mod sqlite_store;

// --- Storage Service Trait ---

/// Defines the interface for a DAG block storage backend.
/// Generic over the block type `B` which must implement `Clone` and be serializable/deserializable.
pub trait StorageService<B: Clone + Serialize + for<'de> Deserialize<'de>> {
    /// Puts a block into the store.
    /// If a block with the same CID already exists, it may be overwritten or an error may be returned,
    /// depending on the implementation.
    fn put(&mut self, block: &B) -> Result<(), CommonError>;

    /// Retrieves a block from the store by its CID.
    /// Returns `Ok(Some(block))` if found, `Ok(None)` if not found, or `Err` on storage failure.
    fn get(&self, cid: &Cid) -> Result<Option<B>, CommonError>;

    /// Deletes a block from the store by its CID.
    /// Returns `Ok(())` if deletion was successful or if the block didn't exist.
    /// Returns `Err` on storage failure.
    fn delete(&mut self, cid: &Cid) -> Result<(), CommonError>;

    /// Checks if a block with the given CID exists in the store.
    /// Returns `Ok(true)` if it exists, `Ok(false)` if not, or `Err` on storage failure.
    fn contains(&self, cid: &Cid) -> Result<bool, CommonError>;
}

// --- In-Memory DAG Store ---

#[derive(Debug, Default)]
pub struct InMemoryDagStore {
    store: HashMap<Cid, DagBlock>,
}

impl InMemoryDagStore {
    pub fn new() -> Self {
        InMemoryDagStore {
            store: HashMap::new(),
        }
    }
}

impl StorageService<DagBlock> for InMemoryDagStore {
    fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        icn_common::verify_block_integrity(block)?;
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

// --- File-based DAG Store (Placeholder) ---

#[derive(Debug)]
pub struct FileDagStore {
    storage_path: PathBuf,
    // Optional: In-memory index for faster lookups, synced with files
    // index: HashMap<Cid, PathBuf>, // Or some offset/length info if storing in a single large file
}

impl FileDagStore {
    pub fn new(storage_path: PathBuf) -> Result<Self, CommonError> {
        if !storage_path.exists() {
            std::fs::create_dir_all(&storage_path).map_err(|e| {
                CommonError::IoError(format!(
                    "Failed to create storage directory {:?}: {}",
                    storage_path, e
                ))
            })?;
        }
        if !storage_path.is_dir() {
            return Err(CommonError::IoError(format!(
                "Storage path {:?} is not a directory",
                storage_path
            )));
        }
        Ok(FileDagStore { storage_path })
    }

    // Renamed from get_block_path in apply model changes, reverting to original name for clarity
    fn block_path(&self, cid: &Cid) -> PathBuf {
        // Consider sharding directories if many blocks are expected (e.g., /ab/cd/efgh...).
        self.storage_path.join(cid.to_string())
    }

    fn put_block_to_file(&self, block: &DagBlock) -> Result<(), CommonError> {
        icn_common::verify_block_integrity(block)?;
        let file_path = self.block_path(&block.cid);
        let serialized_block = serde_json::to_string(block).map_err(|e| {
            CommonError::SerializationError(format!(
                "Failed to serialize block {}: {}",
                block.cid, e
            ))
        })?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true) // Overwrite if exists
            .open(&file_path)
            .map_err(|e| {
                CommonError::IoError(format!(
                    "Failed to open/create file {:?} for writing: {}",
                    file_path, e
                ))
            })?;

        file.write_all(serialized_block.as_bytes()).map_err(|e| {
            CommonError::IoError(format!(
                "Failed to write block {} to file {:?}: {}",
                block.cid, file_path, e
            ))
        })?;
        Ok(())
    }

    fn get_block_from_file(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        let file_path = self.block_path(cid);
        if !file_path.exists() {
            return Ok(None);
        }

        let mut file = File::open(&file_path).map_err(|e| {
            CommonError::IoError(format!(
                "Failed to open file {:?} for reading: {}",
                file_path, e
            ))
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            CommonError::IoError(format!(
                "Failed to read block {} from file {:?}: {}",
                cid, file_path, e
            ))
        })?;

        let block_data: DagBlock = serde_json::from_str(&contents).map_err(|e| {
            CommonError::DeserializationError(format!(
                "Failed to deserialize block {} from file {:?}: {}",
                cid, file_path, e
            ))
        })?;

        if &block_data.cid != cid {
            return Err(CommonError::InvalidInputError(format!(
                "CID mismatch for block read from file {:?}. Expected CID {}, got {}.",
                file_path, cid, block_data.cid
            )));
        }
        Ok(Some(block_data))
    }

    fn delete_block_file(&self, cid: &Cid) -> Result<(), CommonError> {
        let file_path = self.block_path(cid);
        if file_path.exists() {
            std::fs::remove_file(&file_path).map_err(|e| {
                CommonError::IoError(format!("Failed to delete file {:?}: {}", file_path, e))
            })?;
        }
        Ok(())
    }
}

impl StorageService<DagBlock> for FileDagStore {
    fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        self.put_block_to_file(block)
    }

    fn get(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        self.get_block_from_file(cid)
    }

    fn delete(&mut self, cid: &Cid) -> Result<(), CommonError> {
        self.delete_block_file(cid)
    }

    fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        let path = self.block_path(cid);
        Ok(path.exists())
    }
}

// --- Global Store Access (Legacy - To be refactored/removed) ---
// The following functions `put_block` and `get_block` use a global static store.
// This pattern should be replaced by passing `StorageService` instances.
// We keep them for now to minimize disruption to existing code using them,
// but they should be considered deprecated.

lazy_static::lazy_static! {
    /// Global in-memory store used by the deprecated [`put_block`] and [`get_block`] helpers.
    /// New code should create and pass explicit `StorageService` implementations instead.
    static ref DEFAULT_IN_MEMORY_STORE: Mutex<InMemoryDagStore> = Mutex::new(InMemoryDagStore::new());
}

/// Puts a [`DagBlock`] into [`DEFAULT_IN_MEMORY_STORE`].
///
/// # Deprecation
/// This helper exists for legacy code paths. New callers should pass a
/// concrete implementation of [`StorageService`] instead.
#[deprecated(note = "Use an explicit StorageService instance instead")]
pub fn put_block(block: &DagBlock) -> Result<(), CommonError> {
    let mut store = DEFAULT_IN_MEMORY_STORE.lock().map_err(|e| {
        CommonError::InternalError(format!("Failed to acquire lock on default DAG store: {e}"))
    })?;
    store.put(block)
}

/// Retrieves a [`DagBlock`] from [`DEFAULT_IN_MEMORY_STORE`] by CID.
///
/// # Deprecation
/// This helper exists for legacy code paths. New callers should pass a
/// concrete implementation of [`StorageService`] instead.
#[deprecated(note = "Use an explicit StorageService instance instead")]
pub fn get_block(cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
    let store = DEFAULT_IN_MEMORY_STORE.lock().map_err(|e| {
        CommonError::InternalError(format!("Failed to acquire lock on default DAG store: {e}"))
    })?;
    store.get(cid)
}

/// Placeholder function demonstrating use of common types.
pub fn process_dag_related_data(info: &NodeInfo) -> Result<String, CommonError> {
    if info.version == ICN_CORE_VERSION {
        Ok(format!(
            "Processing DAG data for node: {} (version {})",
            info.name, info.version
        ))
    } else {
        Err(CommonError::ConfigError(format!(
            "Version mismatch: Expected {}, got {}. Node: {}",
            ICN_CORE_VERSION, info.version, info.name
        )))
    }
}

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

#[cfg(test)]
mod tests {
    use super::*;
    // For test setup
    use tempfile::tempdir; // For FileDagStore tests

    // Helper function to create a test block
    fn create_test_block(id_str: &str) -> DagBlock {
        let data = format!("data for {id_str}").into_bytes();
        let cid = Cid::new_v1_dummy(0x71, 0x12, id_str.as_bytes());
        DagBlock {
            cid,
            data,
            links: vec![],
        }
    }

    // Generic test suite for any StorageService<DagBlock> implementation
    fn test_storage_service_suite<S: StorageService<DagBlock>>(store: &mut S) {
        let block1 = create_test_block("block1_service_test");
        let block2 = create_test_block("block2_service_test");

        // Test put and contains
        assert!(store.put(&block1).is_ok());
        assert!(store.contains(&block1.cid).unwrap());
        assert!(!store.contains(&block2.cid).unwrap());

        // Test get
        match store.get(&block1.cid) {
            Ok(Some(retrieved_block)) => assert_eq!(retrieved_block.cid, block1.cid),
            _ => panic!("Failed to get block1"),
        }
        assert!(store.get(&block2.cid).unwrap().is_none());

        // Test put overwrite (assuming implementations overwrite)
        let modified_block1_data =
            format!("modified data for {}", "block1_service_test").into_bytes();
        let modified_block1 = DagBlock {
            cid: block1.cid.clone(),
            data: modified_block1_data,
            links: vec![],
        };
        assert!(store.put(&modified_block1).is_ok());
        match store.get(&block1.cid) {
            Ok(Some(retrieved_block)) => {
                assert_eq!(retrieved_block.cid, block1.cid);
                // Ensure data was actually modified
                assert_ne!(retrieved_block.data, block1.data);
                assert_eq!(retrieved_block.data, modified_block1.data);
            }
            _ => panic!("Failed to get modified block1 after overwrite"),
        }

        // Test delete
        assert!(store.delete(&block1.cid).is_ok());
        assert!(!store.contains(&block1.cid).unwrap());
        assert!(store.get(&block1.cid).unwrap().is_none());

        // Test deleting non-existent block (should be Ok)
        assert!(store.delete(&block2.cid).is_ok()); // block2 was never put after block1 deletion test context
                                                    // Or, more robustly, use a fresh CID not in the store
        let non_existent_cid_for_delete = Cid::new_v1_dummy(0x55, 0x12, b"non_existent_for_delete");
        assert!(store.delete(&non_existent_cid_for_delete).is_ok());

        // Put block2 back for further tests if any or ensure clean state for next use of suite
        assert!(store.put(&block2).is_ok());
        assert!(store.contains(&block2.cid).unwrap());
    }

    #[test]
    fn test_in_memory_dag_store_service() {
        let mut store = InMemoryDagStore::new(); // Make store mutable
        test_storage_service_suite(&mut store); // Pass as mutable reference
    }

    #[test]
    fn test_file_dag_store_service() {
        let dir = tempdir().unwrap();
        let mut store = FileDagStore::new(dir.path().to_path_buf()).unwrap();
        test_storage_service_suite(&mut store);

        // Additional FileDagStore specific tests, e.g., persistence across instances
        let block_for_persistence = create_test_block("persistent_block");
        store.put(&block_for_persistence).unwrap();
        let store_path = store.storage_path.clone();
        drop(store); // Drop the store to ensure files are closed

        // Re-open the store
        let mut store2 = FileDagStore::new(store_path).unwrap();
        match store2.get(&block_for_persistence.cid) {
            Ok(Some(retrieved_block)) => {
                assert_eq!(retrieved_block.cid, block_for_persistence.cid);
                assert_eq!(retrieved_block.data, block_for_persistence.data);
            }
            _ => panic!("Failed to retrieve persistent block from FileDagStore"),
        }

        // Test error case: CID mismatch on read
        let block_x = create_test_block("block_x_corruption");
        store2.put(&block_x).unwrap();
        let block_x_path = store2.block_path(&block_x.cid);

        // Manually corrupt the stored block's CID (simulate corruption)
        let mut file_contents = std::fs::read_to_string(&block_x_path).unwrap();
        let mut corrupted_block: DagBlock = serde_json::from_str(&file_contents).unwrap();
        corrupted_block.cid.hash_bytes[0] ^= 0xFF; // Flip some bits in the CID hash
        file_contents = serde_json::to_string(&corrupted_block).unwrap();
        std::fs::write(&block_x_path, file_contents).unwrap();

        match store2.get(&block_x.cid) {
            // Try to get with original CID
            Err(CommonError::InvalidInputError(msg)) => {
                assert!(msg.contains("CID mismatch"));
            }
            Ok(Some(_)) => panic!("Should have failed with CID mismatch"),
            Ok(None) => panic!("Block should exist but be corrupted (CID mismatch), not None"),
            Err(e) => panic!("Expected InvalidInputError for CID mismatch, got other error: {e:?}"),
        }
    }

    #[cfg(feature = "persist-sled")]
    #[test]
    fn test_sled_dag_store_service() {
        let dir = tempdir().unwrap();
        let mut store = sled_store::SledDagStore::new(dir.path().to_path_buf()).unwrap();
        test_storage_service_suite(&mut store);

        let block_persist = create_test_block("persistent_block_sled");
        store.put(&block_persist).unwrap();
        drop(store);

        let store2 = sled_store::SledDagStore::new(dir.path().to_path_buf()).unwrap();
        assert!(store2.get(&block_persist.cid).unwrap().is_some());
    }

    #[cfg(feature = "persist-sqlite")]
    #[test]
    fn test_sqlite_dag_store_service() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("dag.sqlite");
        let mut store = sqlite_store::SqliteDagStore::new(db_path.clone()).unwrap();
        test_storage_service_suite(&mut store);

        let block_persist = create_test_block("persistent_block_sqlite");
        store.put(&block_persist).unwrap();
        drop(store);

        let store2 = sqlite_store::SqliteDagStore::new(db_path).unwrap();
        assert!(store2.get(&block_persist.cid).unwrap().is_some());
    }

    #[cfg(feature = "persist-rocksdb")]
    #[test]
    fn test_rocks_dag_store_service() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("rocks");
        let mut store = rocksdb_store::RocksDagStore::new(db_path.clone()).unwrap();
        test_storage_service_suite(&mut store);

        let block_persist = create_test_block("persistent_block_rocks");
        store.put(&block_persist).unwrap();
        drop(store);

        let store2 = rocksdb_store::RocksDagStore::new(db_path).unwrap();
        assert!(store2.get(&block_persist.cid).unwrap().is_some());
    }

    // The old tests for global put_block/get_block might need adjustment
    // or can be removed if we fully deprecate the global store access.
    // For now, let's ensure they still work with the refactored DEFAULT_IN_MEMORY_STORE.

    #[test]
    fn test_process_dag_data() {
        let node_info = NodeInfo {
            name: "TestDAGNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Testing DAG".to_string(),
        };
        let result = process_dag_related_data(&node_info);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("TestDAGNode"));

        let mismatched_node_info = NodeInfo {
            name: "OldNode".to_string(),
            version: "0.0.1-old".to_string(),
            status_message: "testing old".to_string(),
        };
        let result_err = process_dag_related_data(&mismatched_node_info);
        assert!(result_err.is_err());
    }

    #[test]
    fn test_put_and_get_block() {
        let mut store = InMemoryDagStore::new();
        let block = create_test_block("block_global_store");

        assert!(store.put(&block).is_ok());
        match store.get(&block.cid) {
            Ok(Some(b)) => assert_eq!(b.cid, block.cid),
            Ok(None) => panic!("Block not found in store"),
            Err(e) => panic!("store.get returned an error: {e:?}"),
        }

        let non_existent_cid = Cid::new_v1_dummy(0x55, 0x12, b"non_existent_global");
        match store.get(&non_existent_cid) {
            Ok(None) => { /* Expected */ }
            Ok(Some(_)) => panic!("Found non-existent block in store"),
            Err(e) => panic!("store.get for non-existent CID returned an error: {e:?}"),
        }
    }

    #[test]
    fn test_traversal_index() {
        use crate::index::DagTraversalIndex;
        let mut index = DagTraversalIndex::new();
        let child = create_test_block("child");
        let link = DagLink {
            cid: child.cid.clone(),
            name: "child".into(),
            size: 0,
        };
        let parent_cid = compute_merkle_cid(0x71, b"parent", std::slice::from_ref(&link));
        let parent = DagBlock {
            cid: parent_cid.clone(),
            data: b"parent".to_vec(),
            links: vec![link],
        };
        index.index_block(&child);
        index.index_block(&parent);
        let order = index.traverse(&parent_cid);
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], parent_cid);
        assert!(order.contains(&child.cid));
    }
}
