#![doc = include_str!("../README.md")]

//! # ICN API Crate
//! This crate provides the primary API endpoints for interacting with InterCooperative Network (ICN) nodes.
//! It defines service interfaces, data structures for requests and responses, and potentially server/client implementations.
//! The API aims for clarity, modularity, and extensibility, typically using JSON-RPC or gRPC.

// Depending on icn_common crate
use icn_common::{NodeInfo, NodeStatus, CommonError, ICN_CORE_VERSION, DagBlock, Cid};
use icn_dag::{put_block as dag_put_block, get_block as dag_get_block};

/// Planned: Define a trait for the ICN API service for RPC implementation.
// pub trait IcnApiService {
//    async fn get_node_info(&self) -> Result<NodeInfo, CommonError>;
//    async fn get_node_status(&self) -> Result<NodeStatus, CommonError>;
//    async fn submit_dag_block(&self, block: DagBlock) -> Result<Cid, CommonError>;
//    async fn retrieve_dag_block(&self, cid: Cid) -> Result<Option<DagBlock>, CommonError>;
    // TODO: Add other API methods: submit_transaction, query_data, etc.
// }

/// Retrieves basic information about the ICN node.
/// This function would typically be part of an RPC service.
pub fn get_node_info() -> Result<NodeInfo, CommonError> {
    Ok(NodeInfo {
        version: ICN_CORE_VERSION.to_string(),
        name: "ICN Node (Default Name)".to_string(),
        status_message: "Node is operational".to_string(),
    })
}

/// Retrieves the current operational status of the ICN node.
/// This function simulates a potential error if the node is considered "offline".
pub fn get_node_status(is_simulated_online: bool) -> Result<NodeStatus, CommonError> {
    if !is_simulated_online {
        return Err(CommonError::NodeOffline("Node is currently simulated offline.".to_string()));
    }

    // In a real scenario, these values would be fetched from the node's internal state.
    Ok(NodeStatus {
        is_online: true,
        peer_count: 5, // Example value
        current_block_height: 1000, // Example value
        version: ICN_CORE_VERSION.to_string(),
    })
}

/// Submits a DagBlock to the DAG store via the `icn-dag` crate.
/// Returns the CID of the stored block upon success.
pub fn submit_dag_block(block_data_json: String) -> Result<Cid, CommonError> {
    // In a real API, block_data would likely be raw bytes or a more structured format.
    // Here we assume JSON for simplicity of the CLI interaction for now.
    let block: DagBlock = serde_json::from_str(&block_data_json)
        .map_err(|e| CommonError::DeserializationError(format!("Failed to parse DagBlock JSON: {}", e)))?;

    // TODO: Validate the block. Especially, recalculate its CID based on data and links
    // and ensure it matches block.cid. For now, we trust the provided CID.

    dag_put_block(&block)?;
    Ok(block.cid.clone())
}

/// Retrieves a DagBlock from the DAG store by its CID via the `icn-dag` crate.
pub fn retrieve_dag_block(cid_json: String) -> Result<Option<DagBlock>, CommonError> {
    let cid: Cid = serde_json::from_str(&cid_json)
        .map_err(|e| CommonError::DeserializationError(format!("Failed to parse Cid JSON: {}", e)))?;
    dag_get_block(&cid)
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
    fn get_node_info_works() {
        match get_node_info() {
            Ok(info) => {
                assert_eq!(info.version, ICN_CORE_VERSION);
                assert_eq!(info.name, "ICN Node (Default Name)");
                assert_eq!(info.status_message, "Node is operational");
            }
            Err(_) => panic!("get_node_info returned an error"),
        }
    }

    #[test]
    fn get_node_status_works_when_online() {
        match get_node_status(true) {
            Ok(status) => {
                assert!(status.is_online);
                assert_eq!(status.version, ICN_CORE_VERSION);
                assert_eq!(status.peer_count, 5);
            }
            Err(e) => panic!("get_node_status returned an error when online: {:?}", e),
        }
    }

    #[test]
    fn get_node_status_errs_when_offline() {
        match get_node_status(false) {
            Ok(_) => panic!("get_node_status should have returned an error when offline"),
            Err(CommonError::NodeOffline(msg)) => {
                assert!(msg.contains("simulated offline"));
            }
            Err(e) => panic!("get_node_status returned an unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_submit_and_retrieve_dag_block_api() {
        let data = b"api test block data".to_vec();
        let cid = Cid::new_v1_dummy(0x71, 0x12, &data);
        let link_cid = Cid::new_v1_dummy(0x71, 0x12, b"api link");
        let link = DagLink { cid: link_cid, name: "apilink".to_string(), size: 8 };
        let block = DagBlock {
            cid: cid.clone(),
            data: data.clone(),
            links: vec![link],
        };

        let block_json = serde_json::to_string(&block).unwrap();
        let submitted_cid = submit_dag_block(block_json.clone()).expect("Failed to submit block via API");
        assert_eq!(submitted_cid, cid);

        // Test retrieval
        let cid_json = serde_json::to_string(&cid).unwrap();
        match retrieve_dag_block(cid_json.clone()) {
            Ok(Some(retrieved_block)) => {
                assert_eq!(retrieved_block.cid, cid);
                assert_eq!(retrieved_block.data, data);
            }
            Ok(None) => panic!("Block submitted via API not found"),
            Err(e) => panic!("retrieve_dag_block via API failed: {:?}", e),
        }

        // Test retrieving non-existent block
        let non_existent_cid = Cid::new_v1_dummy(0x71, 0x12, b"non-existent-api");
        let non_existent_cid_json = serde_json::to_string(&non_existent_cid).unwrap();
        match retrieve_dag_block(non_existent_cid_json) {
            Ok(None) => { /* Expected */ }
            Ok(Some(_)) => panic!("Found a non-existent block via API"),
            Err(e) => panic!("retrieve_dag_block for non-existent CID via API failed: {:?}", e),
        }
    }
}
