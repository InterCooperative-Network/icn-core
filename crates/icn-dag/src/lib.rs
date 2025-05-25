#![doc = include_str!("../README.md")]

//! # ICN DAG Crate
//! This crate implements or defines interfaces for content-addressed Directed Acyclic Graph (DAG)
//! storage and manipulation, crucial for the InterCooperative Network (ICN) data model.
//! It handles DAG primitives, content addressing, storage abstraction, and serialization formats.

use icn_common::{NodeInfo, DagBlock, Cid, CommonError, ICN_CORE_VERSION};
use std::collections::HashMap;
use std::sync::Mutex; // For basic interior mutability for the global store
use std::path::PathBuf; // For FileDagStore
use std::fs::{File, OpenOptions}; // For FileDagStore
use std::io::{Read, Write}; // Removed Seek, SeekFrom
use serde::{Serialize, Deserialize}; // For FileDagStore block serialization

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
        // TODO: Add validation: recalculate block.cid and check it matches content + links.
        // If validation fails, return Err(CommonError::DagValidationError("CID mismatch or invalid block structure".to_string()))
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
            std::fs::create_dir_all(&storage_path)
                .map_err(|e| CommonError::StorageError(format!("Failed to create storage directory {:?}: {}", storage_path, e)))?;
        }
        if !storage_path.is_dir() {
            return Err(CommonError::StorageError(format!("Storage path {:?} is not a directory", storage_path)));
        }
        Ok(FileDagStore { storage_path })
    }

    // Helper to get the file path for a given CID
    fn get_block_path(&self, cid: &Cid) -> PathBuf {
        // Using the string representation of CID as filename.
        // Ensure CID string representation is filesystem-safe.
        // Consider sharding directories if many blocks are expected (e.g., /ab/cd/efgh...).
        self.storage_path.join(cid.to_string())
    }
}

impl StorageService<DagBlock> for FileDagStore {
    fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        let file_path = self.get_block_path(&block.cid);
        let serialized_block = serde_json::to_string(block)
            .map_err(|e| CommonError::SerializationError(format!("Failed to serialize block {}: {}", block.cid, e)))?;
        
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true) // Overwrite if exists
            .open(&file_path)
            .map_err(|e| CommonError::StorageError(format!("Failed to open/create file {:?} for writing: {}", file_path, e)))?;
        
        file.write_all(serialized_block.as_bytes())
            .map_err(|e| CommonError::StorageError(format!("Failed to write block {} to file {:?}: {}", block.cid, file_path, e)))?;
        Ok(())
    }

    fn get(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        let file_path = self.get_block_path(cid);
        if !file_path.exists() {
            return Ok(None);
        }

        let mut file = File::open(&file_path)
            .map_err(|e| CommonError::StorageError(format!("Failed to open file {:?} for reading: {}", file_path, e)))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| CommonError::StorageError(format!("Failed to read block {} from file {:?}: {}", cid, file_path, e)))?;
        
        let block: DagBlock = serde_json::from_str(&contents)
            .map_err(|e| CommonError::DeserializationError(format!("Failed to deserialize block {} from file {:?}: {}", cid, file_path, e)))?;
        
        // Optional: Validate CID matches deserialized block content
        if &block.cid != cid {
            return Err(CommonError::DagValidationError(format!("CID mismatch for block read from {:?}. Expected {}, got {}.", file_path, cid, block.cid)));
        }
        Ok(Some(block))
    }

    fn delete(&mut self, cid: &Cid) -> Result<(), CommonError> {
        let file_path = self.get_block_path(cid);
        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .map_err(|e| CommonError::StorageError(format!("Failed to delete file {:?}: {}", file_path, e)))?;
        }
        Ok(())
    }

    fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        let path = self.get_block_path(cid);
        Ok(path.exists())
    }
}


// --- Global Store Access (Legacy - To be refactored/removed) ---
// The following functions `put_block` and `get_block` use a global static store.
// This pattern should be replaced by passing `StorageService` instances.
// We keep them for now to minimize disruption to existing code using them,
// but they should be considered deprecated.

lazy_static::lazy_static! {
    // This global store is now effectively a default InMemoryDagStore instance.
    // New code should prefer explicit store instances.
    static ref DEFAULT_IN_MEMORY_STORE: Mutex<InMemoryDagStore> = Mutex::new(InMemoryDagStore::new());
}

/// Puts a DagBlock into the default in-memory store.
/// If a block with the same CID already exists, it will be overwritten.
/// DEPRECATED: Use a `StorageService` instance directly.
pub fn put_block(block: &DagBlock) -> Result<(), CommonError> {
    let mut store = DEFAULT_IN_MEMORY_STORE.lock()
        .map_err(|e| CommonError::StorageError(format!("Failed to acquire lock on default DAG store: {}", e)))?;
    store.put(block)
}

/// Retrieves a DagBlock from the default in-memory store by its CID.
/// DEPRECATED: Use a `StorageService` instance directly.
pub fn get_block(cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
    let store = DEFAULT_IN_MEMORY_STORE.lock()
        .map_err(|e| CommonError::StorageError(format!("Failed to acquire lock on default DAG store: {}", e)))?;
    store.get(cid)
}

/// Placeholder function demonstrating use of common types.
pub fn process_dag_related_data(info: &NodeInfo) -> Result<String, CommonError> {
    if info.version == ICN_CORE_VERSION {
        Ok(format!("Processing DAG data for node: {} (version {})", info.name, info.version))
    } else {
        Err(CommonError::PlaceholderError("Version mismatch".to_string()))
    }
}

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::DagLink; // For test setup
    use tempfile::tempdir; // For FileDagStore tests

    // Helper function to create a test block
    fn create_test_block(id_str: &str) -> DagBlock {
        let data = format!("data for {}", id_str).into_bytes();
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
        assert_eq!(store.contains(&block1.cid).unwrap(), true);
        assert_eq!(store.contains(&block2.cid).unwrap(), false);

        // Test get
        match store.get(&block1.cid) {
            Ok(Some(retrieved_block)) => assert_eq!(retrieved_block.cid, block1.cid),
            _ => panic!("Failed to get block1"),
        }
        assert!(store.get(&block2.cid).unwrap().is_none());

        // Test put overwrite (assuming implementations overwrite)
        let modified_block1_data = format!("modified data for {}", "block1_service_test").into_bytes();
        let modified_block1 = DagBlock {
            cid: block1.cid.clone(),
            data: modified_block1_data,
            links: vec![],
        };
        assert!(store.put(&modified_block1).is_ok());
        match store.get(&block1.cid) {
            Ok(Some(retrieved_block)) => assert_eq!(retrieved_block.data, modified_block1.data),
            _ => panic!("Failed to get modified block1"),
        }

        // Test delete
        assert!(store.delete(&block1.cid).is_ok());
        assert_eq!(store.contains(&block1.cid).unwrap(), false);
        assert!(store.get(&block1.cid).unwrap().is_none());

        // Test delete non-existent
        assert!(store.delete(&block2.cid).is_ok()); // Should be idempotent

        // Test putting multiple blocks
        assert!(store.put(&block1).is_ok()); // Re-add block1
        assert!(store.put(&block2).is_ok());
        assert_eq!(store.contains(&block1.cid).unwrap(), true);
        assert_eq!(store.contains(&block2.cid).unwrap(), true);
    }
    
    #[test]
    fn test_in_memory_dag_store_service() {
        let mut store = InMemoryDagStore::new(); // Make store mutable
        test_storage_service_suite(&mut store); // Pass as mutable reference
    }

    #[test]
    fn test_file_dag_store_service() {
        let dir = tempdir().expect("Failed to create temp dir for FileDagStore test");
        let store_path = dir.path().to_path_buf();
        {
            let mut store = FileDagStore::new(store_path.clone()).expect("Failed to create FileDagStore");
            test_storage_service_suite(&mut store); // Pass mutable reference for the suite

            // Test persistence: Create a new store instance for the same path
            let block_for_persistence = create_test_block("persistent_block");
            assert!(store.put(&block_for_persistence).is_ok());
        } // store goes out of scope, files should be written

        let store2 = FileDagStore::new(store_path.clone()).expect("Failed to create FileDagStore for persistence test");
        match store2.get(&create_test_block("persistent_block").cid) {
            Ok(Some(retrieved_block)) => {
                assert_eq!(retrieved_block.cid, create_test_block("persistent_block").cid);
                assert_eq!(retrieved_block.data, create_test_block("persistent_block").data);
            }
            _ => panic!("Failed to retrieve persistent block from FileDagStore"),
        }
        
        // Test CID mismatch on read
        let block_cid_mismatch = create_test_block("cid_mismatch_block");
        let another_cid = Cid::new_v1_dummy(0x71, 0x12, b"another_cid_for_file");
        let store_for_mismatch = FileDagStore::new(store_path.clone()).expect("Failed to create FileDagStore");
        
        // Manually create a file with mismatched CID
        let file_path_mismatch = store_for_mismatch.get_block_path(&another_cid);
        let serialized_block = serde_json::to_string(&block_cid_mismatch).unwrap();
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&file_path_mismatch).unwrap();
        file.write_all(serialized_block.as_bytes()).unwrap();
        drop(file);

        match store_for_mismatch.get(&another_cid) {
            Err(CommonError::DagValidationError(_)) => { /* Expected error */ }
            Ok(_) => panic!("Expected DagValidationError for CID mismatch, got Ok"),
            Err(e) => panic!("Expected DagValidationError, got other error: {:?}", e),
        }


        dir.close().expect("Failed to close temp dir");
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
        // Clear the store for a clean test run if tests run in parallel
        DEFAULT_IN_MEMORY_STORE.lock().unwrap().store.clear();

        let data = b"hello dag world".to_vec();
        // Use a more unique CID for testing to avoid clashes if store is not cleared.
        let cid = Cid::new_v1_dummy(0x71, 0x12, b"test_put_and_get_block_data_global"); 
        
        let link_data = b"a link for put_get test global".to_vec();
        let link_cid = Cid::new_v1_dummy(0x71, 0x12, &link_data);
        let link = DagLink {
            cid: link_cid,
            name: "child_link_put_get_global".to_string(),
            size: link_data.len() as u64,
        };

        let block = DagBlock {
            cid: cid.clone(),
            data: data.clone(),
            links: vec![link],
        };

        assert!(put_block(&block).is_ok());

        match get_block(&cid) {
            Ok(Some(retrieved_block)) => {
                assert_eq!(retrieved_block.cid, cid);
                assert_eq!(retrieved_block.data, data);
                assert_eq!(retrieved_block.links.len(), 1);
                assert_eq!(retrieved_block.links[0].name, "child_link_put_get_global");
            }
            Ok(None) => panic!("Block not found after put_block"),
            Err(e) => panic!("get_block returned an error: {:?}", e),
        }

        let non_existent_cid_data = b"non_existent_data_for_dag_test_global".to_vec();
        let non_existent_cid = Cid::new_v1_dummy(0x71, 0x12, &non_existent_cid_data);
        match get_block(&non_existent_cid) {
            Ok(None) => { /* Expected */ }
            Ok(Some(_)) => panic!("Found a block that should not exist"),
            Err(e) => panic!("get_block for non-existent CID returned an error: {:?}", e),
        }
    }
}
