#![doc = include_str!("../README.md")]

//! # ICN DAG Crate
//! This crate implements or defines interfaces for content-addressed Directed Acyclic Graph (DAG)
//! storage and manipulation, crucial for the InterCooperative Network (ICN) data model.
//! It handles DAG primitives, content addressing, storage abstraction, and serialization formats.

use icn_common::{NodeInfo, DagBlock, Cid, CommonError, ICN_CORE_VERSION};
use std::collections::HashMap;
use std::sync::Mutex; // For basic interior mutability for the global store

// --- In-Memory DAG Store --- 

// Using a global Mutex-guarded HashMap for a simple in-memory store.
// This is NOT suitable for production but demonstrates the principle.
// A proper implementation would involve a more robust storage solution and likely an async API.
lazy_static::lazy_static! {
    static ref IN_MEMORY_STORE: Mutex<HashMap<Cid, DagBlock>> = Mutex::new(HashMap::new());
}

/// Puts a DagBlock into the in-memory store.
/// If a block with the same CID already exists, it will be overwritten.
/// TODO: Add proper error handling for store failures.
/// TODO: Ensure the block's CID actually matches its content + links (requires hashing).
pub fn put_block(block: &DagBlock) -> Result<(), CommonError> {
    let mut store = IN_MEMORY_STORE.lock().map_err(|_| CommonError::StorageError("Failed to lock DAG store".to_string()))?;
    store.insert(block.cid.clone(), block.clone());
    Ok(())
}

/// Retrieves a DagBlock from the in-memory store by its CID.
/// TODO: Add proper error handling for store failures.
pub fn get_block(cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
    let store = IN_MEMORY_STORE.lock().map_err(|_| CommonError::StorageError("Failed to lock DAG store".to_string()))?;
    Ok(store.get(cid).cloned())
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

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }

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
        // Clear the store for a clean test run if tests run in parallel (Mutex helps but defensive)
        // For more robust tests, consider passing a store instance or using test-specific stores.
        // IN_MEMORY_STORE.lock().unwrap().clear();

        let data = b"hello dag world".to_vec();
        let cid = Cid::new_v1_dummy(0x71, 0x12, &data); // dag-cbor, sha2-256
        
        let link_data = b"a link".to_vec();
        let link_cid = Cid::new_v1_dummy(0x71, 0x12, &link_data);
        let link = DagLink {
            cid: link_cid,
            name: "child_link".to_string(),
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
                assert_eq!(retrieved_block.links[0].name, "child_link");
            }
            Ok(None) => panic!("Block not found after put_block"),
            Err(e) => panic!("get_block returned an error: {:?}", e),
        }

        let non_existent_cid_data = b"non_existent_data".to_vec();
        let non_existent_cid = Cid::new_v1_dummy(0x71, 0x12, &non_existent_cid_data);
        match get_block(&non_existent_cid) {
            Ok(None) => { /* Expected */ }
            Ok(Some(_)) => panic!("Found a block that should not exist"),
            Err(e) => panic!("get_block for non-existent CID returned an error: {:?}", e),
        }
    }
}
