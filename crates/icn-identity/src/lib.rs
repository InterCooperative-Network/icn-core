#![doc = include_str!("../README.md")]

//! # ICN Identity Crate
//! This crate manages decentralized identities (DIDs), verifiable credentials (VCs),
//! and cryptographic operations for the InterCooperative Network (ICN).
//! It focuses on security, interoperability with DID/VC standards, and usability.

use icn_common::{NodeInfo, CommonError, ICN_CORE_VERSION};

/// Placeholder function demonstrating use of common types for identity.
pub fn register_identity(info: &NodeInfo, did_method: &str) -> Result<String, CommonError> {
    Ok(format!("Registered {} identity for node: {} (v{})", did_method, info.name, info.version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_identity() {
        let node_info = NodeInfo {
            name: "IdNode".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Identity active".to_string(),
        };
        let result = register_identity(&node_info, "did:example");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("did:example"));
    }
}
