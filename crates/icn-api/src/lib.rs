#![doc = include_str!("../README.md")]

//! # ICN API Crate
//! This crate provides the primary API endpoints for interacting with InterCooperative Network (ICN) nodes.
//! It defines service interfaces, data structures for requests and responses, and potentially server/client implementations.
//! The API aims for clarity, modularity, and extensibility, typically using JSON-RPC or gRPC.

// Depending on icn_common crate
use icn_common::{NodeInfo, NodeStatus, CommonError, ICN_CORE_VERSION};

/// Planned: Define a trait for the ICN API service for RPC implementation.
// pub trait IcnApiService {
//    async fn get_node_info(&self) -> Result<NodeInfo, CommonError>;
//    async fn get_node_status(&self) -> Result<NodeStatus, CommonError>;
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

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }

#[cfg(test)]
mod tests {
    use super::*;

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
}
