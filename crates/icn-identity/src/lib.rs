#![doc = include_str!("../README.md")]

//! # ICN Identity Crate
//! This crate manages decentralized identities (DIDs), verifiable credentials (VCs),
//! and cryptographic operations for the InterCooperative Network (ICN).
//! It focuses on security, interoperability with DID/VC standards, and usability.

use icn_common::{Did, CommonError, ICN_CORE_VERSION, NodeInfo};
use rand::RngCore;
use bs58;

// --- Key Management Placeholder Structs ---
#[derive(Debug, Clone)]
pub struct PublicKey(pub Vec<u8>); // Placeholder for a public key

#[derive(Debug, Clone)]
pub struct PrivateKey(pub Vec<u8>); // Placeholder for a private key, zeroize on drop in real impl

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
}

/// Generates a new cryptographic key pair (placeholder implementation).
/// TODO: Implement with actual cryptographic library (e.g., ed25519).
pub fn generate_key_pair() -> Result<KeyPair, CommonError> {
    let mut public_bytes = [0u8; 32];
    let mut private_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut public_bytes);
    rand::thread_rng().fill_bytes(&mut private_bytes);
    Ok(KeyPair {
        public_key: PublicKey(public_bytes.to_vec()),
        private_key: PrivateKey(private_bytes.to_vec()),
    })
}

/// Creates a `did:key` DID from a public key (very simplified).
/// A real did:key involves multicodec prefix for the key type.
/// See: https://w3c-ccg.github.io/did-method-key/
/// TODO: Implement correctly with multicodec prefix for a supported key type (e.g., Ed25519).
pub fn did_key_from_public_key(public_key: &PublicKey) -> Did {
    // This is NOT a compliant did:key, just a bs58 encoding of raw bytes for now.
    let id_string = bs58::encode(&public_key.0).into_string();
    Did::new("key", &id_string)
}

/// Planned: Resolve a DID string to its corresponding DID Document.
/// For `did:key`, this involves constructing the document from the key itself.
// pub fn resolve_did(did_string: &str) -> Result<icn_common::DidDocument, CommonError> { todo!(); }

// TODO: Implement Verifiable Credential issuance logic.
// TODO: Implement Verifiable Credential verification logic.
// TODO: Define DidDocument, ServiceEndpoint, VerificationMethod structs in icn-common or here.
// TODO: Define VerifiableCredential and Proof structs.

/// Placeholder function demonstrating use of common types for identity.
pub fn register_identity(info: &NodeInfo, did_method: &str) -> Result<String, CommonError> {
    if did_method == "key" {
        let kp = generate_key_pair()?;
        let did = did_key_from_public_key(&kp.public_key);
        Ok(format!("Registered {} for node: {} (v{}). DID: {}", did_method, info.name, info.version, did.to_string()))
    } else {
        Ok(format!("Registered {} identity for node: {} (v{})", did_method, info.name, info.version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation_placeholder() {
        let key_pair = generate_key_pair().unwrap();
        assert_eq!(key_pair.public_key.0.len(), 32);
        assert_eq!(key_pair.private_key.0.len(), 32);
    }

    #[test]
    fn test_did_key_from_public_key_placeholder() {
        let mut public_bytes = [0u8; 32];
        public_bytes[0] = 1; public_bytes[1] = 2; // Make it non-zero for a more distinct bs58
        let public_key = PublicKey(public_bytes.to_vec());
        let did = did_key_from_public_key(&public_key);
        assert_eq!(did.method, "key");
        assert!(!did.id_string.is_empty());
        println!("Generated placeholder did:key: {}", did.to_string());
    }

    #[test]
    fn test_register_identity_with_did_key() {
        let node_info = NodeInfo {
            name: "IdNodeKey".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Identity active with did:key".to_string(),
        };
        let result = register_identity(&node_info, "key");
        assert!(result.is_ok());
        let res_string = result.unwrap();
        assert!(res_string.contains("did:key:"));
        assert!(res_string.contains("IdNodeKey"));
    }

    #[test]
    fn test_register_identity_other_method() {
        let node_info = NodeInfo {
            name: "IdNodeOther".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Identity active other method".to_string(),
        };
        let result = register_identity(&node_info, "did:example");
        assert!(result.is_ok());
        let result_string = result.unwrap();
        assert!(result_string.contains("did:example"));
        assert!(!result_string.contains("did:key:"));
    }
}
