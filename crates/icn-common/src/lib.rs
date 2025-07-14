#![doc = include_str!("../README.md")]

//! # ICN Common Crate
//! This crate provides common data structures, types, utilities, and error definitions
//! shared across multiple InterCooperative Network (ICN) core crates. It aims to
//! reduce code duplication, ensure consistency, and simplify dependencies.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
pub mod retry;
pub use retry::retry_with_backoff;
pub mod resilience;
pub use resilience::{CircuitBreaker, CircuitBreakerError, CircuitState};
pub mod resource_token;
pub mod zk;
pub use zk::{ZkCredentialProof, ZkProofType, ZkRevocationProof};

pub const ICN_CORE_VERSION: &str = "0.2.0-beta";

/// Basic metadata about an ICN node used for diagnostics and handshakes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Semantic version of the running software.
    pub version: String,
    /// Human friendly name of the node.
    pub name: String,
    /// Optional status or informational message.
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

/// Indicates whether the local DAG is synchronized with peers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagSyncStatus {
    /// Current root CID of the local DAG.
    pub current_root: Option<Cid>,
    /// True if the node believes it is fully synchronized.
    pub in_sync: bool,
}

/// Identifies a membership scope such as a community, cooperative, or federation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeScope(pub String);

/// Provides the current time for deterministic operations.
pub trait TimeProvider: Send + Sync {
    /// Return the current Unix timestamp in seconds.
    fn unix_seconds(&self) -> u64;
}

/// Uses [`std::time::SystemTime`] as the source of time.
#[derive(Debug, Clone)]
pub struct SystemTimeProvider;

impl TimeProvider for SystemTimeProvider {
    fn unix_seconds(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Deterministic time provider returning a fixed timestamp.
#[derive(Debug, Clone)]
pub struct FixedTimeProvider(pub u64);

impl FixedTimeProvider {
    /// Create a new [`FixedTimeProvider`] returning `ts`.
    pub fn new(ts: u64) -> Self {
        Self(ts)
    }
}

impl TimeProvider for FixedTimeProvider {
    fn unix_seconds(&self) -> u64 {
        self.0
    }
}

/// Represents a generic error that can occur within the network.
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

    /// Operation was rejected by a policy engine or CCL contract.
    #[error("Policy denied: {0}")]
    PolicyDenied(String),

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

    /// Duplicate or replayed message detected
    #[error("Duplicate message")]
    DuplicateMessage,
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
    /// DID method name, e.g. `"key"` or `"web"`.
    pub method: String,
    /// Method specific identifier string (without path, query, or fragment).
    pub id_string: String,
    /// Optional path component beginning with `/`.
    pub path: Option<String>,
    /// Optional URL query string without the leading `?`.
    pub query: Option<String>,
    /// Optional URL fragment without the leading `#`.
    pub fragment: Option<String>,
}

impl Did {
    /// Construct a new [`Did`] from a method name and identifier string.
    ///
    /// This does not perform validation beyond storing the provided values.
    pub fn new(method: &str, id_string: &str) -> Self {
        Did {
            method: method.to_string(),
            id_string: id_string.to_string(),
            path: None,
            query: None,
            fragment: None,
        }
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "did:{}:{}", self.method, self.id_string)?;
        if let Some(path) = &self.path {
            if !path.starts_with('/') {
                write!(f, "/{path}")?;
            } else {
                write!(f, "{path}")?;
            }
        }
        if let Some(query) = &self.query {
            write!(f, "?{query}")?;
        }
        if let Some(fragment) = &self.fragment {
            write!(f, "#{fragment}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Did {
    type Err = CommonError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("did:") {
            return Err(CommonError::InvalidInputError(
                "DID must start with 'did:'".to_string(),
            ));
        }
        let mut rest = &s[4..];
        let method_end = rest
            .find(':')
            .ok_or_else(|| CommonError::InvalidInputError("Missing method".into()))?;
        let method = &rest[..method_end];
        rest = &rest[method_end + 1..];
        if method.is_empty() || rest.is_empty() {
            return Err(CommonError::InvalidInputError(format!(
                "Invalid DID string format: {s}. Expected 'did:method:id'"
            )));
        }

        let mut id_end = rest.len();
        for c in ['/', '?', '#'] {
            if let Some(pos) = rest.find(c) {
                id_end = id_end.min(pos);
            }
        }
        let id_string = &rest[..id_end];
        rest = &rest[id_end..];

        let mut path = None;
        let mut query = None;
        let mut fragment = None;

        if rest.starts_with('/') {
            let mut end = rest.len();
            for c in ['?', '#'] {
                if let Some(pos) = rest.find(c) {
                    end = end.min(pos);
                }
            }
            path = Some(rest[..end].to_string());
            rest = &rest[end..];
        }

        if rest.starts_with('?') {
            let end = rest.find('#').unwrap_or(rest.len());
            query = Some(rest[1..end].to_string());
            rest = &rest[end..];
        }

        if rest.starts_with('#') {
            fragment = Some(rest[1..].to_string());
            rest = "";
        }

        if !rest.is_empty() {
            return Err(CommonError::InvalidInputError(format!(
                "Unexpected characters in DID URL: {rest}"
            )));
        }

        Ok(Did {
            method: method.to_string(),
            id_string: id_string.to_string(),
            path,
            query,
            fragment,
        })
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
    /// Create a CID using a truncated hash of the provided data.
    ///
    /// This helper is only for testing and should not be used in production
    /// where real multihash computation is required.
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
    fn to_bytes(&self) -> Vec<u8> {
        use unsigned_varint::encode as varint_encode;
        let mut out = Vec::new();
        let mut buf64 = varint_encode::u64_buffer();
        out.extend_from_slice(varint_encode::u64(self.version, &mut buf64));
        let mut buf16 = varint_encode::u16_buffer();
        out.extend_from_slice(varint_encode::u16(self.codec as u16, &mut buf16));
        out.extend_from_slice(varint_encode::u16(self.hash_alg as u16, &mut buf16));
        out.extend_from_slice(&self.hash_bytes);
        out
    }

    /// Create a CID from its raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CommonError> {
        use multicodec::Codec;
        use unsigned_varint::decode as varint_decode;

        let (version, rest) = varint_decode::u64(bytes)
            .map_err(|e| CommonError::DeserError(format!("Invalid version varint: {e}")))?;
        let (codec_u16, rest) = varint_decode::u16(rest)
            .map_err(|e| CommonError::DeserError(format!("Invalid codec varint: {e}")))?;
        let _ = Codec::from_code(codec_u16)
            .map_err(|e| CommonError::InvalidInputError(format!("Unknown codec: {e}")))?;
        let (hash_alg_u16, hash_bytes) = varint_decode::u16(rest)
            .map_err(|e| CommonError::DeserError(format!("Invalid hash alg varint: {e}")))?;
        let _ = Codec::from_code(hash_alg_u16)
            .map_err(|e| CommonError::InvalidInputError(format!("Unknown hash alg: {e}")))?;

        Ok(Cid {
            version,
            codec: codec_u16 as u64,
            hash_alg: hash_alg_u16 as u64,
            hash_bytes: hash_bytes.to_vec(),
        })
    }

    /// Encode this CID to a multibase string.
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        use multibase::{encode, Base};
        encode(Base::Base32Lower, self.to_bytes())
    }
}

/// Parse a CID string produced by [`Cid::to_string`].
pub fn parse_cid_from_string(cid_str: &str) -> Result<Cid, CommonError> {
    if cid_str.is_empty() {
        return Err(CommonError::InvalidInputError(
            "Empty CID string".to_string(),
        ));
    }

    use multibase::decode as multibase_decode;

    let (_base, data) = multibase_decode(cid_str)
        .map_err(|e| CommonError::InvalidInputError(format!("Invalid multibase CID: {e}")))?;

    Cid::from_bytes(&data)
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Represents a generic block in a Content-Addressed DAG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagBlock {
    /// The CID of this block (calculated from the data and metadata).
    pub cid: Cid,
    /// Opaque data payload of the block.
    pub data: Vec<u8>,
    /// Links to other [`DagBlock`]s.
    pub links: Vec<DagLink>,
    /// Unix timestamp when the block was created.
    pub timestamp: u64,
    /// DID of the block author.
    pub author_did: Did,
    /// Optional Ed25519 signature of the block contents.
    pub signature: Option<SignatureBytes>,
    /// Optional scope this block belongs to.
    pub scope: Option<NodeScope>,
}

/// Represents a link within a DagBlock, pointing to another DagBlock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagLink {
    pub cid: Cid,     // CID of the linked block
    pub name: String, // Optional name for the link (e.g., field name in a CBOR object)
    pub size: u64,    // Total size of the linked block (useful for traversals)
}

/// Compute a Merkle-style CID for a block using SHA-256 of its data and link CIDs.
pub fn compute_merkle_cid(
    codec: u64,
    data: &[u8],
    links: &[DagLink],
    timestamp: u64,
    author_did: &Did,
    signature: &Option<SignatureBytes>,
    scope: &Option<NodeScope>,
) -> Cid {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut link_strings: Vec<String> = links.iter().map(|l| l.cid.to_string()).collect();
    link_strings.sort();
    for s in link_strings {
        hasher.update(s.as_bytes());
    }
    hasher.update(timestamp.to_le_bytes());
    hasher.update(author_did.to_string().as_bytes());
    if let Some(sig) = signature {
        hasher.update(&sig.0);
    }
    if let Some(scope) = scope {
        hasher.update(scope.0.as_bytes());
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
    let expected = compute_merkle_cid(
        block.cid.codec,
        &block.data,
        &block.links,
        block.timestamp,
        &block.author_did,
        &block.signature,
        &block.scope,
    );
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
    /// Nonce ensuring transaction uniqueness.
    pub nonce: u64,
    /// Maximum compute units the sender is willing to expend.
    pub mana_limit: u64,
    /// Price per compute unit the sender is willing to pay.
    pub mana_price: u64,
    /// Optional Ed25519 signature of the transaction content.
    pub signature: Option<SignatureBytes>,
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
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.data);
        let mut links: Vec<String> = self.links.iter().map(|l| l.cid.to_string()).collect();
        links.sort();
        for l in links {
            bytes.extend_from_slice(l.as_bytes());
        }
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(self.author_did.to_string().as_bytes());
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
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes.extend_from_slice(&self.mana_limit.to_le_bytes());
        bytes.extend_from_slice(&self.mana_price.to_le_bytes());
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
    #[allow(clippy::const_is_empty)]
    fn version_is_set() {
        assert!(!ICN_CORE_VERSION.is_empty());
        assert!(ICN_CORE_VERSION.contains("0.2.0"));
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
        let cid = Cid::new_v1_sha256(0x71, b"hello world");
        assert_eq!(cid.version, 1);
        assert_eq!(cid.codec, 0x71);
        assert_eq!(cid.hash_alg, 0x12);
        assert_eq!(cid.hash_bytes.len(), 32);
        println!("CID string: {}", cid.to_string());
        assert!(cid.to_string().starts_with('b'));
        let serialized = serde_json::to_string(&cid).unwrap();
        assert!(serialized.contains("hash_bytes"));
    }

    #[test]
    fn dag_block_creation() {
        let link_cid = Cid::new_v1_sha256(0x71, b"linked data");
        let link = DagLink {
            cid: link_cid.clone(),
            name: "child".to_string(),
            size: 100, // Dummy size
        };
        let timestamp = 0u64;
        let author = Did::new("key", "tester");
        let sig = None;
        let block_cid = compute_merkle_cid(
            0x71,
            b"main data",
            std::slice::from_ref(&link),
            timestamp,
            &author,
            &sig,
            &None,
        );
        let block = DagBlock {
            cid: block_cid.clone(),
            data: b"main data".to_vec(),
            links: vec![link],
            timestamp,
            author_did: author,
            signature: sig,
            scope: None,
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
            nonce: 1,
            mana_limit: 10,
            mana_price: 1,
            signature: Some(SignatureBytes(vec![0u8; ed25519_dalek::SIGNATURE_LENGTH])),
        };
        assert_eq!(transaction.sender_did, sender);
        let serialized = serde_json::to_string(&transaction).unwrap();
        assert!(serialized.contains("tx123"));
    }

    #[test]
    fn cid_round_trip_parse_and_to_string() {
        let cid = Cid::new_v1_sha256(0x71, b"round trip test");
        let cid_str = cid.to_string();
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
        let timestamp = 0u64;
        let author = Did::new("key", "tester");
        let sig = None;
        let cid = compute_merkle_cid(
            0x71,
            &data,
            std::slice::from_ref(&link),
            timestamp,
            &author,
            &sig,
            &None,
        );
        let block = DagBlock {
            cid: cid.clone(),
            data,
            links: vec![link],
            timestamp,
            author_did: author,
            signature: sig,
            scope: None,
        };
        assert!(verify_block_integrity(&block).is_ok());
    }
}
