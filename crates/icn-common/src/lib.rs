#![doc = include_str!("../README.md")]

//! # ICN Common Crate
//! This crate provides common data structures, types, utilities, and error definitions
//! shared across multiple InterCooperative Network (ICN) core crates. It aims to
//! reduce code duplication, ensure consistency, and simplify dependencies.

use serde::{Serialize, Deserialize};
use std::fmt;
use bs58;

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

impl fmt::Display for CommonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self) // Simple Display implementation using Debug
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommonError {
    PlaceholderError(String),
    ApiError(String),               // General API error
    NodeOffline(String),            // Node is offline
    NetworkUnhealthy(String),       // Network is generally unhealthy
    SerializationError(String),     // Error during serialization
    DeserializationError(String),   // Error during deserialization
    NotFoundError(String),          // Generic not found
    InvalidInputError(String),      // Input validation failed
    StorageError(String),           // General storage error

    // Network specific errors
    NetworkConnectionError(String), // e.g. Cannot connect to peer
    PeerNotFound(String),           // Specific peer not found
    MessageSendError(String),       // Failed to send a message
    MessageReceiveError(String),    // Failed to receive/parse a message

    // Storage specific errors
    BlockNotFound(String),      // Specific to DAG block not found
    DatabaseError(String),      // For underlying database issues if not general StorageError

    // Identity specific errors
    IdentityError(String),              // General identity error
    KeyPairGenerationError(String),
    SignatureError(String),             // For signing or verification failures
    DidResolutionError(String),
    CredentialError(String),            // For Verifiable Credential issues

    // DAG specific errors
    DagValidationError(String),     // e.g. CID mismatch, invalid link
    BlockTooLargeError(String),

    // Governance specific errors
    ProposalExists(String),
    ProposalNotFound(String),
    VotingClosed(String),
    AlreadyVoted(String),
    InvalidVoteOption(String),
    NotEligibleToVote(String),
    NetworkSetupError(String), // For errors during network service initialization (e.g. libp2p setup)

    // TODO: Add more specific error variants as needed
}

// TODO: Define struct for DIDs (e.g., pub struct Did { method: String, id_string: String, ... })
// TODO: Define struct for CIDs (e.g., pub struct Cid { version: u64, codec: u64, hash: Vec<u8> })
// TODO: Define common traits e.g. pub trait Signable { fn sign(&self, key: &Key) -> Signature; fn verify(&self, signature: &Signature, public_key: &PublicKey) -> bool; }

// --- Real Protocol Data Models ---

/// Represents a Decentralized Identifier (DID).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did {
    pub method: String,         // e.g., "key", "web", "ion"
    pub id_string: String,      // The method-specific identifier string
    // TODO: Potentially add DID URL parsing elements like path, query, fragment later.
}

impl Did {
    pub fn new(method: &str, id_string: &str) -> Self {
        Did { method: method.to_string(), id_string: id_string.to_string() }
    }
    pub fn to_string(&self) -> String {
        format!("did:{}:{}", self.method, self.id_string)
    }
}

impl std::str::FromStr for Did {
    type Err = CommonError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.splitn(3, ':').collect();
        if parts.len() == 3 && parts[0] == "did" && !parts[1].is_empty() && !parts[2].is_empty() {
            Ok(Did { method: parts[1].to_string(), id_string: parts[2].to_string() })
        } else {
            Err(CommonError::InvalidInputError(format!("Invalid DID string format: {}. Expected 'did:method:id' with non-empty method and id.", s)))
        }
    }
}

/// Represents a Content Identifier (CID).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Hash is important for CIDs as keys
pub struct Cid {
    pub version: u64,           // CIDv0 or CIDv1
    pub codec: u64,             // Multicodec for the content type (e.g., dag-pb, dag-cbor)
    pub hash_alg: u64,          // Multicodec for the hash algorithm (e.g., sha2-256)
    pub hash_bytes: Vec<u8>,    // The raw hash bytes
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
    pub fn to_string_approx(&self) -> String {
        // This is a highly simplified string representation, not a real Base58BTC or Base32 CID string.
        // Using bs58 encoding of the hash bytes to make it more unique for filenames.
        format!("cidv{}-{}-{}-{}", self.version, self.codec, self.hash_alg, bs58::encode(&self.hash_bytes).into_string())
    }
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
    pub cid: Cid,                   // The CID of this block (calculated from data + links)
    pub data: Vec<u8>,              // The opaque data payload of the block
    pub links: Vec<DagLink>,        // Links to other DagBlocks
    // TODO: Consider adding metadata like timestamp, author DID, signature
}

/// Represents a link within a DagBlock, pointing to another DagBlock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagLink {
    pub cid: Cid,       // CID of the linked block
    pub name: String,   // Optional name for the link (e.g., field name in a CBOR object)
    pub size: u64,      // Total size of the linked block (useful for traversals)
}

/// Represents a generic transaction within the ICN.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String, // Transaction ID (e.g., hash of its content)
    pub timestamp: u64, // Unix timestamp
    pub sender_did: Did, // DID of the sender
    pub recipient_did: Option<Did>, // Optional recipient DID
    pub payload_type: String, // Describes the type of data in payload (e.g., "transfer", "governance_vote")
    pub payload: Vec<u8>, // Serialized transaction-specific data
    pub signature: Option<String>, // Optional signature of the transaction content
    // TODO: Add fields like nonce, gas_limit, gas_price if relevant to economic model
}

// TODO: Define `DidDocument` struct for DID resolution.
// TODO: Define structs for cryptographic keys (`PublicKey`, `PrivateKey`, `KeyPair`) and signatures.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!ICN_CORE_VERSION.is_empty());
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
        assert_eq!(did.to_string(), "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuias7ux1jEZ6KATp8");
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
        assert!(serialized.contains("main data"));
    }

    #[test]
    fn transaction_creation() {
        let sender = Did::new("key", "sender_did_string");
        let transaction = Transaction {
            id: "tx123".to_string(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            sender_did: sender.clone(),
            recipient_did: None,
            payload_type: "test_payload".to_string(),
            payload: b"some test data".to_vec(),
            signature: Some("dummy_signature".to_string()),
        };
        assert_eq!(transaction.sender_did, sender);
        let serialized = serde_json::to_string(&transaction).unwrap();
        assert!(serialized.contains("tx123"));
    }
}
