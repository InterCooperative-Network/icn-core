#![doc = include_str!("../README.md")]

//! # ICN Common Crate
//! This crate provides common data structures, types, utilities, and error definitions
//! shared across multiple InterCooperative Network (ICN) core crates. It aims to
//! reduce code duplication, ensure consistency, and simplify dependencies.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub const ICN_CORE_VERSION: &str = "0.1.0-dev-functional";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeInfo {
    pub version: String,
    pub name: String,
    pub status_message: String,
}

/// Represents the operational status of an ICN node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeStatus {
    pub is_online: bool,
    pub peer_count: u32,
    pub current_block_height: u64, // Example field
    pub version: String,
}

/// Represents a generic error that can occur within the ICN network.
#[derive(Debug, Error, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum CommonError {
    #[error("Invalid input: {0}")]
    InvalidInputError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Cryptography error: {0}")]
    CryptoError(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Network setup error: {0}")]
    NetworkSetupError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Message send error: {0}")]
    MessageSendError(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Network unhealthy: {0}")]
    NetworkUnhealthy(String),

    #[error("Identity error: {0}")]
    IdentityError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Operation cancelled: {0}")]
    CancelledError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Feature not implemented: {0}")]
    NotImplementedError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("DAG validation error: {0}")]
    DagValidationError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Node offline: {0}")]
    NodeOffline(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),

    #[error("Deserialization error: {0}")]
    DeserError(String),

    #[error("Serialization error: {0}")]
    SerError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Trait for types that can produce a canonical byte representation for
/// cryptographic signing and provide helper methods for Ed25519 signatures.
pub trait Signable {
    /// Return the bytes that should be signed.
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError>;

    /// Sign `self` with the provided Ed25519 [`SigningKey`].
    fn sign(&self, key: &ed25519_dalek::SigningKey) -> Result<SignatureBytes, CommonError> {
        use ed25519_dalek::Signer;
        let bytes = self.to_signable_bytes()?;
        let sig = key.sign(&bytes);
        Ok(SignatureBytes(sig.to_bytes().to_vec()))
    }

    /// Verify a signature against `self` using the provided [`VerifyingKey`].
    fn verify(
        &self,
        signature: &SignatureBytes,
        key: &ed25519_dalek::VerifyingKey,
    ) -> Result<(), CommonError> {
        use ed25519_dalek::Signature;
        let bytes = self.to_signable_bytes()?;
        let sig: Signature = signature.try_into()?;
        key.verify_strict(&bytes, &sig)
            .map_err(|_| CommonError::CryptoError("Signature verification failed".into()))
    }
}

/// Wrapper for raw Ed25519 signature bytes used by [`Signable`] implementors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureBytes(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl TryFrom<&SignatureBytes> for ed25519_dalek::Signature {
    type Error = CommonError;

    fn try_from(value: &SignatureBytes) -> Result<Self, Self::Error> {
        let arr: [u8; ed25519_dalek::SIGNATURE_LENGTH] = value
            .0
            .clone()
            .try_into()
            .map_err(|_| CommonError::CryptoError("Invalid signature length".into()))?;
        Ok(ed25519_dalek::Signature::from_bytes(&arr))
    }
}

// --- Real Protocol Data Models ---

/// Represents a Decentralized Identifier (DID).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Did {
    pub method: String, // e.g., "key", "web", "ion"
    pub id_string: String, // The method-specific identifier string
                        // TODO: Potentially add DID URL parsing elements like path, query, fragment later.
}

impl Did {
    pub fn new(method: &str, id_string: &str) -> Self {
        Did {
            method: method.to_string(),
            id_string: id_string.to_string(),
        }
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "did:{}:{}", self.method, self.id_string)
    }
}

impl std::str::FromStr for Did {
    type Err = CommonError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.splitn(3, ':').collect();
        if parts.len() == 3 && parts[0] == "did" && !parts[1].is_empty() && !parts[2].is_empty() {
            Ok(Did {
                method: parts[1].to_string(),
                id_string: parts[2].to_string(),
            })
        } else {
            Err(CommonError::InvalidInputError(format!("Invalid DID string format: {s}. Expected 'did:method:id' with non-empty method and id.")))
        }
    }
}

/// Represents a Content Identifier (CID).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Hash is important for CIDs as keys
pub struct Cid {
    pub version: u64,        // CIDv0 or CIDv1
    pub codec: u64,          // Multicodec for the content type (e.g., dag-pb, dag-cbor)
    pub hash_alg: u64,       // Multicodec for the hash algorithm (e.g., sha2-256)
    pub hash_bytes: Vec<u8>, // The raw hash bytes
}

impl Cid {
    // TODO: Implement proper CID creation from data, parsing from string, etc.
    // This is a simplified constructor for now.
    pub fn new_v1_dummy(codec: u64, hash_alg: u64, data: &[u8]) -> Self {
        // In a real scenario, you'd hash the data using the specified hash_alg
        // For now, let's just use a slice of the data as a mock hash
        let hash_bytes = data.iter().take(32).cloned().collect();
        Cid {
            version: 1,
            codec,
            hash_alg,
            hash_bytes,
        }
    }

    /// Create a CID using SHA-256 of the provided data bytes.
    pub fn new_v1_sha256(codec: u64, data: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        let hash_bytes = Sha256::digest(data).to_vec();
        Cid {
            version: 1,
            codec,
            hash_alg: 0x12,
            hash_bytes,
        }
    }
    pub fn to_string_approx(&self) -> String {
        // This is a highly simplified string representation, not a real Base58BTC or Base32 CID string.
        // Using bs58 encoding of the hash bytes to make it more unique for filenames.
        format!(
            "cidv{}-{}-{}-{}",
            self.version,
            self.codec,
            self.hash_alg,
            bs58::encode(&self.hash_bytes).into_string()
        )
    }
}

/// Parse a CID previously produced by [`Cid::to_string_approx`].
///
/// This expects the format `cidv{{version}}-{{codec}}-{{hash_alg}}-{{base58_hash}}`.
pub fn parse_cid_from_string(cid_str: &str) -> Result<Cid, CommonError> {
    if cid_str.is_empty() {
        return Err(CommonError::InvalidInputError(
            "Empty CID string".to_string(),
        ));
    }

    let parts: Vec<&str> = cid_str.split('-').collect();
    if parts.len() != 4 {
        return Err(CommonError::InvalidInputError(format!(
            "Invalid CID format: expected 4 parts separated by '-', got {}",
            parts.len()
        )));
    }

    let version_str = parts[0]
        .strip_prefix("cidv")
        .ok_or_else(|| CommonError::InvalidInputError("Missing 'cidv' prefix".to_string()))?;
    let version: u64 = version_str
        .parse()
        .map_err(|e| CommonError::InvalidInputError(format!("Invalid version: {e}")))?;

    let codec: u64 = parts[1]
        .parse()
        .map_err(|e| CommonError::InvalidInputError(format!("Invalid codec: {e}")))?;

    let hash_alg: u64 = parts[2]
        .parse()
        .map_err(|e| CommonError::InvalidInputError(format!("Invalid hash_alg: {e}")))?;

    let hash_bytes = bs58::decode(parts[3])
        .into_vec()
        .map_err(|e| CommonError::InvalidInputError(format!("Invalid base58 hash: {e}")))?;

    Ok(Cid {
        version,
        codec,
        hash_alg,
        hash_bytes,
    })
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Using the more unique approximate string representation.
        write!(f, "{}", self.to_string_approx())
    }
}

/// Represents a generic block in a Content-Addressed DAG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagBlock {
    pub cid: Cid,      // The CID of this block (calculated from data + links)
    pub data: Vec<u8>, // The opaque data payload of the block
    pub links: Vec<DagLink>, // Links to other DagBlocks
                       // TODO: Consider adding metadata like timestamp, author DID, signature
}

/// Represents a link within a DagBlock, pointing to another DagBlock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagLink {
    pub cid: Cid,     // CID of the linked block
    pub name: String, // Optional name for the link (e.g., field name in a CBOR object)
    pub size: u64,    // Total size of the linked block (useful for traversals)
}

/// Compute a Merkle-style CID for a block using SHA-256 of its data and link CIDs.
pub fn compute_merkle_cid(codec: u64, data: &[u8], links: &[DagLink]) -> Cid {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut link_strings: Vec<String> = links.iter().map(|l| l.cid.to_string()).collect();
    link_strings.sort();
    for s in link_strings {
        hasher.update(s.as_bytes());
    }
    let hash_bytes = hasher.finalize().to_vec();
    Cid {
        version: 1,
        codec,
        hash_alg: 0x12,
        hash_bytes,
    }
}

/// Verify that a block's CID matches the Merkle hash of its contents and links.
pub fn verify_block_integrity(block: &DagBlock) -> Result<(), CommonError> {
    let expected = compute_merkle_cid(block.cid.codec, &block.data, &block.links);
    if expected == block.cid {
        Ok(())
    } else {
        Err(CommonError::DagValidationError("CID mismatch".to_string()))
    }
}

/// Represents a generic transaction within the ICN.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction ID (e.g. hash of the contents).
    pub id: String,
    /// Unix timestamp when the transaction was created.
    pub timestamp: u64,
    /// DID of the sender.
    pub sender_did: Did,
    /// Optional recipient DID.
    pub recipient_did: Option<Did>,
    /// Describes the type of data in [`payload`].
    pub payload_type: String,
    /// Serialized transaction-specific data.
    pub payload: Vec<u8>,
    /// Optional Ed25519 signature of the transaction content.
    pub signature: Option<SignatureBytes>,
    // TODO: Add fields like nonce, gas_limit, gas_price if relevant to economic model
}

/// Minimal DID document containing the public verifying key for a [`Did`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DidDocument {
    /// The identifier this document describes.
    pub id: Did,
    /// Raw public key bytes associated with the DID.
    #[serde(with = "serde_bytes")]
    pub public_key: Vec<u8>,
}

impl Signable for DagBlock {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        let mut bytes = self.cid.to_string().into_bytes();
        bytes.extend_from_slice(&self.data);
        let mut links: Vec<String> = self.links.iter().map(|l| l.cid.to_string()).collect();
        links.sort();
        for l in links {
            bytes.extend_from_slice(l.as_bytes());
        }
        Ok(bytes)
    }
}

impl Signable for Transaction {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.id.as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(self.sender_did.to_string().as_bytes());
        if let Some(ref r) = self.recipient_did {
            bytes.extend_from_slice(r.to_string().as_bytes());
        }
        bytes.extend_from_slice(self.payload_type.as_bytes());
        bytes.extend_from_slice(&self.payload);
        Ok(bytes)
    }
}

impl Signable for DidDocument {
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        let mut bytes = self.id.to_string().into_bytes();
        bytes.extend_from_slice(&self.public_key);
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(ICN_CORE_VERSION.contains("functional"));
    }

    #[test]
    fn node_info_can_be_created() {
        let info = NodeInfo {
            version: ICN_CORE_VERSION.to_string(),
            name: "Test Node".to_string(),
            status_message: "All good".to_string(),
        };
        assert_eq!(info.name, "Test Node");
        let serialized = serde_json::to_string(&info).unwrap();
        assert!(serialized.contains("Test Node"));
    }

    #[test]
    fn node_status_can_be_created() {
        let status = NodeStatus {
            is_online: true,
            peer_count: 10,
            current_block_height: 12345,
            version: ICN_CORE_VERSION.to_string(),
        };
        assert!(status.is_online);
        assert_eq!(status.peer_count, 10);
        let serialized = serde_json::to_string(&status).unwrap();
        assert!(serialized.contains("12345"));
    }

    #[test]
    fn did_creation_and_to_string() {
        let did = Did::new("key", "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8");
        assert_eq!(did.method, "key");
        assert_eq!(
            did.to_string(),
            "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8"
        );
        let serialized = serde_json::to_string(&did).unwrap();
        assert!(serialized.contains("z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8"));
    }

    #[test]
    fn cid_creation_and_to_string() {
        let cid = Cid::new_v1_dummy(0x71, 0x12, b"hello world"); // 0x71 = dag-cbor, 0x12 = sha2-256
        assert_eq!(cid.version, 1);
        assert_eq!(cid.codec, 0x71);
        assert_eq!(cid.hash_alg, 0x12);
        assert_eq!(cid.hash_bytes.len(), 11); // "hello world" is 11 bytes, it takes min(data.len(), 32)
        println!("Dummy CID string: {}", cid.to_string_approx());
        assert!(cid.to_string_approx().starts_with("cidv1-113-18-"));
        let serialized = serde_json::to_string(&cid).unwrap();
        assert!(serialized.contains("hash_bytes"));
    }

    #[test]
    fn dag_block_creation() {
        let link_cid = Cid::new_v1_dummy(0x71, 0x12, b"linked data");
        let link = DagLink {
            cid: link_cid.clone(),
            name: "child".to_string(),
            size: 100, // Dummy size
        };
        let block_cid = Cid::new_v1_dummy(0x71, 0x12, b"main data");
        let block = DagBlock {
            cid: block_cid.clone(),
            data: b"main data".to_vec(),
            links: vec![link],
        };
        assert_eq!(block.cid, block_cid);
        assert_eq!(block.links[0].cid, link_cid);
        let serialized = serde_json::to_string(&block).unwrap();
        let deserialized: DagBlock = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.data, b"main data".to_vec());
    }

    #[test]
    fn transaction_creation() {
        let sender = Did::new("key", "sender_did_string");
        let transaction = Transaction {
            id: "tx123".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            sender_did: sender.clone(),
            recipient_did: None,
            payload_type: "test_payload".to_string(),
            payload: b"some test data".to_vec(),
            signature: Some(SignatureBytes(vec![0u8; ed25519_dalek::SIGNATURE_LENGTH])),
        };
        assert_eq!(transaction.sender_did, sender);
        let serialized = serde_json::to_string(&transaction).unwrap();
        assert!(serialized.contains("tx123"));
    }

    #[test]
    fn cid_round_trip_parse_and_to_string() {
        let cid = Cid::new_v1_dummy(0x71, 0x12, b"round trip test");
        let cid_str = cid.to_string_approx();
        let parsed = parse_cid_from_string(&cid_str).expect("failed to parse cid");
        assert_eq!(cid, parsed);
    }

    #[test]
    fn merkle_cid_and_verify() {
        let child_cid = Cid::new_v1_sha256(0x71, b"child data");
        let link = DagLink {
            cid: child_cid,
            name: "child".to_string(),
            size: 9,
        };
        let data = b"parent".to_vec();
        let cid = compute_merkle_cid(0x71, &data, std::slice::from_ref(&link));
        let block = DagBlock {
            cid: cid.clone(),
            data,
            links: vec![link],
        };
        assert!(verify_block_integrity(&block).is_ok());
    }
}
