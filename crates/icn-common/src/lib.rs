#![doc = include_str!("../README.md")]

//! # ICN Common Crate
//! This crate provides common data structures, types, utilities, and error definitions
//! shared across multiple InterCooperative Network (ICN) core crates. It aims to
//! reduce code duplication, ensure consistency, and simplify dependencies.

pub const ICN_CORE_VERSION: &str = "0.1.0-dev-bootstrap";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInfo {
    pub version: String,
    pub name: String,
    pub status_message: String,
}

#[derive(Debug)]
pub enum CommonError {
    PlaceholderError(String),
    ApiError(String),
    // TODO: Add more specific error variants: e.g., SerializationError, IoError, NotFound, PermissionDenied
}

// TODO: Define struct for DIDs (e.g., pub struct Did { method: String, id_string: String, ... })
// TODO: Define struct for CIDs (e.g., pub struct Cid { version: u64, codec: u64, hash: Vec<u8> })
// TODO: Define common traits e.g. pub trait Signable { fn sign(&self, key: &Key) -> Signature; fn verify(&self, signature: &Signature, public_key: &PublicKey) -> bool; }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!ICN_CORE_VERSION.is_empty());
    }

    #[test]
    fn node_info_can_be_created() {
        let info = NodeInfo {
            version: ICN_CORE_VERSION.to_string(),
            name: "Test Node".to_string(),
            status_message: "All good".to_string(),
        };
        assert_eq!(info.name, "Test Node");
    }
}
