#![doc = include_str!("../README.md")]

//! # ICN Protocol Crate
//! This crate defines core message formats and protocol definitions for the ICN,
//! ensuring interoperability between different components and nodes.

use icn_common::Did;
use icn_common::{CommonError, NodeInfo};
use serde::{Deserialize, Serialize};

/// Placeholder function demonstrating use of common types for protocol messages.
pub fn serialize_protocol_message(
    info: &NodeInfo,
    message_type: u16,
) -> Result<Vec<u8>, CommonError> {
    let msg_string = format!(
        "Msg type {} from node: {} (v{})",
        message_type, info.name, info.version
    );
    Ok(msg_string.into_bytes())
}

/// Message sent when a node requests to join a federation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationJoinRequest {
    /// DID of the requesting node
    pub node: Did,
    /// Identifier of the federation to join
    pub federation_id: String,
}

/// Response to a [`FederationJoinRequest`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationJoinResponse {
    /// DID of the node requesting membership
    pub node: Did,
    /// Whether the request was accepted
    pub accepted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    use icn_common::ICN_CORE_VERSION;
    #[test]
    fn test_serialize_protocol_message() {
        let node_info = NodeInfo {
            name: "ProtoNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Protocol active".to_string(),
        };
        let result = serialize_protocol_message(&node_info, 1);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
        assert!(String::from_utf8(bytes).unwrap().contains("ProtoNode"));
    }
}
