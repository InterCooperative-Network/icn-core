#![doc = include_str!("../README.md")]

//! # ICN Network Crate
//! This crate manages peer-to-peer (P2P) networking aspects for the InterCooperative Network (ICN),
//! likely using libp2p. It covers P2P communication, transport protocols, peer discovery,
//! message routing, and federation synchronization.

use icn_common::{NodeInfo, CommonError, ICN_CORE_VERSION};

/// Placeholder function demonstrating use of common types for network operations.
pub fn send_network_ping(info: &NodeInfo, target_peer: &str) -> Result<String, CommonError> {
    Ok(format!("Sent ping to {} from node: {} (v{})", target_peer, info.name, info.version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_network_ping() {
        let node_info = NodeInfo {
            name: "NetNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Network active".to_string(),
        };
        let result = send_network_ping(&node_info, "peer-abc");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("peer-abc"));
    }
}
