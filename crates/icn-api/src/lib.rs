#![doc = include_str!("../README.md")]

//! # ICN API Crate
//! This crate provides the primary API endpoints for interacting with InterCooperative Network (ICN) nodes.
//! It defines service interfaces, data structures for requests and responses, and potentially server/client implementations.
//! The API aims for clarity, modularity, and extensibility, typically using JSON-RPC or gRPC.

// Depending on icn_common crate
use icn_common::{NodeInfo, CommonError, ICN_CORE_VERSION};

/// Retrieves basic information about the ICN node.
/// This function would typically be part of an RPC service.
pub fn get_node_info() -> Result<NodeInfo, CommonError> {
    Ok(NodeInfo {
        version: ICN_CORE_VERSION.to_string(),
        name: "ICN Node (Default Name)".to_string(),
        status_message: "Node is operational".to_string(),
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
}
