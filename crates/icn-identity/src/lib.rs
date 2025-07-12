//! Minimal identity primitives: key-gen, DID:key, signing, verification.
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

// Original imports that might still be needed or can be cleaned up later:
// use icn_common::{NodeInfo, CommonError, Did, ICN_CORE_VERSION, Cid};
// use serde::{Serialize, Deserialize};

use icn_common::{Cid, CommonError, Did, NodeInfo, NodeScope};
use serde::{Deserialize, Serialize}; // Keep serde for ExecutionReceipt

pub use ed25519_dalek::{
    Signature as EdSignature, Signer, SigningKey, VerifyingKey, SIGNATURE_LENGTH,
}; // Made pub, removed unused Verifier initially, then re-added Keys
use multibase::{encode as multibase_encode, Base};
use rand_core::OsRng;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use unsigned_varint::encode as varint_encode;

pub mod zk;
pub use zk::{BulletproofsVerifier, DummyVerifier, ZkError, ZkVerifier};

// --- Core Cryptographic Operations & DID:key generation ---

/// Generate an Ed25519 key-pair using the OS CSPRNG.
pub fn generate_ed25519_keypair() -> (SigningKey, VerifyingKey) {
    let sk = SigningKey::generate(&mut OsRng);
    let pk = sk.verifying_key();
    (sk, pk)
}

/// Return a `did:key` DID for the given Ed25519 public key.
///
/// Steps (did:key v0.7):
/// 1.  multicodec prefix for Ed25519-pub = 0xed (unsigned varint `[0xed, 0x01]`)
/// 2.  concat(prefix || raw_pk_bytes) -> 34 bytes
/// 3.  multibase-encode with base58-btc ('z')
/// 4.  did = "did:key:" + multibase
pub fn did_key_from_verifying_key(pk: &VerifyingKey) -> String {
    // 1. multicodec prefix (0xed for Ed25519-pub)
    let mut code_buf = varint_encode::u16_buffer();
    let prefix_ed25519_pub = varint_encode::u16(0xed, &mut code_buf);

    // 2. concat prefix and public key bytes
    let mut prefixed_pk_bytes: Vec<u8> = prefix_ed25519_pub.to_vec();
    prefixed_pk_bytes.extend_from_slice(pk.as_bytes());

    // 3. multibase encode
    let mb = multibase_encode(Base::Base58Btc, prefixed_pk_bytes);

    // 4. final did string
    format!("did:key:{mb}")
}

/// Parse a `did:key` DID into the associated verifying key.
pub fn verifying_key_from_did_key(did: &Did) -> Result<VerifyingKey, CommonError> {
    if did.method != "key" {
        return Err(CommonError::IdentityError(format!(
            "Unsupported DID method: {}",
            did.method
        )));
    }

    use multibase::Base;
    use unsigned_varint::decode as varint_decode;

    let (base, data) = multibase::decode(&did.id_string)
        .map_err(|e| CommonError::IdentityError(format!("Failed to decode did:key: {e}")))?;
    if base != Base::Base58Btc {
        return Err(CommonError::IdentityError(format!(
            "Unsupported multibase prefix: {base:?}"
        )));
    }
    let (codec, rest) = varint_decode::u16(&data).map_err(|e| {
        CommonError::IdentityError(format!("Failed to decode multicodec prefix: {e}"))
    })?;
    if codec != 0xed {
        return Err(CommonError::IdentityError(format!(
            "Unsupported multicodec code: {codec}"
        )));
    }
    let pk_bytes: [u8; 32] = rest
        .try_into()
        .map_err(|_| CommonError::IdentityError("Invalid Ed25519 key length in did:key".into()))?;
    VerifyingKey::from_bytes(&pk_bytes)
        .map_err(|e| CommonError::IdentityError(format!("Invalid verifying key bytes: {e}")))
}

/// Construct a `did:web` identifier from a domain and optional path segments.
///
/// # Errors
///
/// Returns [`CommonError::IdentityError`] if any path segment contains
/// unsupported characters or exceeds [`MAX_SEGMENT_LEN`].
pub fn did_web_from_parts(domain: &str, path: &[&str]) -> Result<String, CommonError> {
    if !is_valid_domain(domain) {
        return Err(CommonError::IdentityError("invalid did:web domain".into()));
    }
    let mut id = domain.replace(':', "%3A");
    for segment in path {
        if !is_valid_segment(segment) {
            return Err(CommonError::IdentityError(format!(
                "invalid did:web segment: {segment}"
            )));
        }
        id.push(':');
        id.push_str(segment);
    }
    Ok(format!("did:web:{id}"))
}

const MAX_SEGMENT_LEN: usize = 63;
const MAX_DOMAIN_LEN: usize = 253;

fn is_valid_domain(domain: &str) -> bool {
    if domain.is_empty() || domain.len() > MAX_DOMAIN_LEN {
        return false;
    }

    // Handle domains with ports (e.g., "localhost:8080")
    let (hostname, _port) = if let Some(colon_pos) = domain.rfind(':') {
        let hostname = &domain[..colon_pos];
        let port_str = &domain[colon_pos + 1..];

        // Validate port is numeric and in valid range
        if let Ok(port) = port_str.parse::<u16>() {
            if port == 0 {
                return false;
            }
            (hostname, Some(port))
        } else {
            return false;
        }
    } else {
        (domain, None)
    };

    // Validate hostname part
    if hostname.is_empty() {
        return false;
    }

    hostname.split('.').all(|label| {
        let bytes = label.as_bytes();
        !bytes.is_empty()
            && bytes.len() <= MAX_SEGMENT_LEN
            && !bytes.starts_with(b"-")
            && !bytes.ends_with(b"-")
            && bytes
                .iter()
                .all(|c| c.is_ascii_alphanumeric() || *c == b'-')
    })
}

fn is_valid_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment.len() <= MAX_SEGMENT_LEN
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Parse a `did:web` DID into its domain and path segments.
pub fn parse_did_web(did: &Did) -> Result<(String, Vec<String>), CommonError> {
    if did.method != "web" {
        return Err(CommonError::IdentityError(format!(
            "Unsupported DID method: {}",
            did.method
        )));
    }
    let mut parts: Vec<String> = did
        .id_string
        .split(':')
        .map(|s| s.replace("%3A", ":"))
        .collect();
    if parts.is_empty() {
        return Err(CommonError::IdentityError(
            "did:web identifier missing domain".into(),
        ));
    }
    let domain = parts.remove(0);
    Ok((domain, parts))
}

/// Construct a `did:peer` identifier (algorithm 0) from a verifying key.
pub fn did_peer_from_verifying_key(pk: &VerifyingKey) -> String {
    let mut buf = varint_encode::u16_buffer();
    let prefix = varint_encode::u16(0xed, &mut buf);
    let mut data = prefix.to_vec();
    data.extend_from_slice(pk.as_bytes());
    let mb = multibase_encode(Base::Base58Btc, data);
    format!("did:peer:0{mb}")
}

/// Extract the verifying key from a `did:peer` identifier (algorithm 0).
pub fn verifying_key_from_did_peer(did: &Did) -> Result<VerifyingKey, CommonError> {
    if did.method != "peer" {
        return Err(CommonError::IdentityError(format!(
            "Unsupported DID method: {}",
            did.method
        )));
    }
    use unsigned_varint::decode as varint_decode;
    let id = did
        .id_string
        .strip_prefix('0')
        .ok_or_else(|| CommonError::IdentityError("Unsupported did:peer algorithm".into()))?;
    let (base, data) = multibase::decode(id)
        .map_err(|e| CommonError::IdentityError(format!("Failed to decode did:peer: {e}")))?;
    if base != Base::Base58Btc {
        return Err(CommonError::IdentityError(format!(
            "Unsupported multibase prefix: {base:?}"
        )));
    }
    let (codec, rest) = varint_decode::u16(&data).map_err(|e| {
        CommonError::IdentityError(format!("Failed to decode multicodec prefix: {e}"))
    })?;
    if codec != 0xed {
        return Err(CommonError::IdentityError(format!(
            "Unsupported multicodec code: {codec}"
        )));
    }
    let pk_bytes: [u8; 32] = rest
        .try_into()
        .map_err(|_| CommonError::IdentityError("Invalid Ed25519 key length in did:peer".into()))?;
    VerifyingKey::from_bytes(&pk_bytes)
        .map_err(|e| CommonError::IdentityError(format!("Invalid verifying key bytes: {e}")))
}

/// Trait representing storage for signing keys associated with DIDs.
pub trait KeyStorage: Send + Sync {
    /// Retrieve the signing key for the given DID, if available.
    fn get_signing_key(&self, did: &Did) -> Option<&SigningKey>;
    /// Store the signing key for the given DID.
    fn store_signing_key(&mut self, did: Did, key: SigningKey);
}

/// Trait for rotating signing keys.
pub trait KeyRotation: Send + Sync {
    /// Generate a new key for the DID and return the updated DID string.
    fn rotate_ed25519(&mut self, did: &Did) -> Result<Did, CommonError>;
}

/// Simple in-memory implementation of [`KeyStorage`] and [`KeyRotation`].
#[derive(Default)]
pub struct InMemoryKeyStore {
    keys: HashMap<Did, SigningKey>,
}

impl KeyStorage for InMemoryKeyStore {
    fn get_signing_key(&self, did: &Did) -> Option<&SigningKey> {
        self.keys.get(did)
    }

    fn store_signing_key(&mut self, did: Did, key: SigningKey) {
        self.keys.insert(did, key);
    }
}

impl KeyRotation for InMemoryKeyStore {
    fn rotate_ed25519(&mut self, did: &Did) -> Result<Did, CommonError> {
        let (sk, pk) = generate_ed25519_keypair();
        let new_did_str = did_key_from_verifying_key(&pk);
        let new_did = Did::from_str(&new_did_str)?;
        self.keys.insert(new_did.clone(), sk);
        self.keys.remove(did);
        Ok(new_did)
    }
}

/// Resolve verifying keys for `did:web` identifiers using a provided key map.
pub struct WebDidResolver {
    keys: HashMap<String, VerifyingKey>,
}

impl WebDidResolver {
    /// Create a new resolver with an empty key map.
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    /// Insert a verifying key for the given `did:web` string.
    pub fn insert(&mut self, did: String, key: VerifyingKey) {
        self.keys.insert(did, key);
    }
}

impl Default for WebDidResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl DidResolver for WebDidResolver {
    fn resolve(&self, did: &Did) -> Result<VerifyingKey, CommonError> {
        if did.method != "web" {
            return Err(CommonError::IdentityError(format!(
                "Unsupported DID method: {}",
                did.method
            )));
        }
        if let Some(k) = self.keys.get(&did.to_string()) {
            return Ok(*k);
        }

        let (domain, path) = parse_did_web(did)?;
        let url = if path.is_empty() {
            format!("https://{domain}/.well-known/did.json")
        } else {
            format!("https://{domain}/{}/did.json", path.join("/"))
        };
        let resp = reqwest::blocking::get(&url)
            .map_err(|e| CommonError::IdentityError(format!("HTTP GET failed: {e}")))?;
        if !resp.status().is_success() {
            return Err(CommonError::IdentityError(format!(
                "HTTP error {} for {url}",
                resp.status()
            )));
        }
        let doc: icn_common::DidDocument = resp
            .json()
            .map_err(|e| CommonError::IdentityError(format!("Invalid DID document: {e}")))?;
        let pk_bytes: [u8; 32] = doc
            .public_key
            .as_slice()
            .try_into()
            .map_err(|_| CommonError::IdentityError("Invalid key length".into()))?;
        VerifyingKey::from_bytes(&pk_bytes)
            .map_err(|e| CommonError::IdentityError(format!("Invalid verifying key: {e}")))
    }
}

/// Trait for resolving a [`Did`] to the verifying key used for signature verification.
pub trait DidResolver: Send + Sync {
    /// Resolve the given DID to an Ed25519 verifying key.
    fn resolve(&self, did: &Did) -> Result<VerifyingKey, CommonError>;
}

/// Simple resolver that understands the `did:key` method.
#[derive(Debug, Clone, Default)]
pub struct KeyDidResolver;

impl DidResolver for KeyDidResolver {
    fn resolve(&self, did: &Did) -> Result<VerifyingKey, CommonError> {
        verifying_key_from_did_key(did)
    }
}

/// Resolver for the `did:peer` method (algorithm 0).
#[derive(Debug, Clone, Default)]
pub struct PeerDidResolver;

impl DidResolver for PeerDidResolver {
    fn resolve(&self, did: &Did) -> Result<VerifyingKey, CommonError> {
        verifying_key_from_did_peer(did)
    }
}

/// Resolves whether a DID belongs to a particular [`NodeScope`].
pub trait MembershipResolver: Send + Sync {
    /// Returns true if the DID is a member of the given scope.
    fn is_member(&self, did: &Did, scope: &NodeScope) -> bool;
}

/// Simple in-memory [`MembershipResolver`] backed by a map of scopes to members.
#[derive(Default)]
pub struct InMemoryMembershipResolver {
    members: HashMap<NodeScope, HashSet<Did>>,
}

impl InMemoryMembershipResolver {
    /// Create an empty resolver.
    pub fn new() -> Self {
        Self {
            members: HashMap::new(),
        }
    }

    /// Add a DID as a member of the provided scope.
    pub fn add_member(&mut self, scope: NodeScope, did: Did) {
        self.members.entry(scope).or_default().insert(did);
    }

    /// Remove a member from the scope if present.
    pub fn remove_member(&mut self, scope: &NodeScope, did: &Did) {
        if let Some(set) = self.members.get_mut(scope) {
            set.remove(did);
        }
    }
}

impl MembershipResolver for InMemoryMembershipResolver {
    fn is_member(&self, did: &Did, scope: &NodeScope) -> bool {
        self.members
            .get(scope)
            .map(|s| s.contains(did))
            .unwrap_or(false)
    }
}

/// Enforces scoped permissions by consulting a [`MembershipResolver`].
pub struct MembershipPolicyEnforcer<R: MembershipResolver> {
    resolver: R,
}

impl<R: MembershipResolver> MembershipPolicyEnforcer<R> {
    /// Create a new enforcer using the provided membership resolver.
    pub fn new(resolver: R) -> Self {
        Self { resolver }
    }

    /// Check that `actor` is a member of `scope`, returning an error if not.
    pub fn check_permission(&self, actor: &Did, scope: &NodeScope) -> Result<(), CommonError> {
        if self.resolver.is_member(actor, scope) {
            Ok(())
        } else {
            Err(CommonError::PermissionDenied(format!(
                "DID {actor} not a member of scope {scope:?}"
            )))
        }
    }
}

/// Convenience wrapper around signing raw bytes with an Ed25519 SigningKey.
pub fn sign_message(sk: &SigningKey, msg: &[u8]) -> EdSignature {
    sk.sign(msg)
}

/// Verify a message/signature pair with an Ed25519 VerifyingKey.
pub fn verify_signature(pk: &VerifyingKey, msg: &[u8], sig: &EdSignature) -> bool {
    pk.verify_strict(msg, sig).is_ok()
}

// --- Structs for ICN System (Keypair, ExecutionReceipt) ---

// Using ed25519_dalek types directly where possible now.
// PublicKey and SecretKey structs might be reintroduced if we need more abstraction
// or to handle different key types in the future. For now, this simplifies.

/// Wrapper for raw Ed25519 signature bytes with optional Serde support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureBytes(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl SignatureBytes {
    /// Construct from an [`ed25519_dalek::Signature`].
    pub fn from_ed_signature(ed_sig: ed25519_dalek::Signature) -> Self {
        SignatureBytes(ed_sig.to_bytes().to_vec())
    }
    /// Convert back into an [`ed25519_dalek::Signature`].
    pub fn to_ed_signature(&self) -> Result<ed25519_dalek::Signature, CommonError> {
        let bytes_array: [u8; SIGNATURE_LENGTH] = self
            .0
            .clone()
            .try_into()
            .map_err(|_| CommonError::InternalError("Invalid signature length".into()))?;

        // Assuming ed25519_dalek::Signature::from_bytes returns ed25519_dalek::Signature directly
        // based on persistent compiler errors. This implies errors from it would panic.
        let sig = ed25519_dalek::Signature::from_bytes(&bytes_array);
        Ok(sig)
    }
}

/// Represents a verifiable proof that a job was executed.
/// This structure is signed by the Executor and anchored to the DAG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    /// Unique identifier of the job that was executed.
    pub job_id: Cid,
    /// DID of the executor node that performed the job.
    pub executor_did: Did, // This DID should correspond to the public key used for signing.
    /// CID of the deterministic result output by the job execution.
    pub result_cid: Cid,
    /// CPU time consumed by the job in milliseconds.
    pub cpu_ms: u64,
    /// Whether the job execution completed successfully.
    pub success: bool,
    /// Cryptographic signature of the receipt fields (job_id, executor_did, result_cid, cpu_ms)
    /// generated by the executor.
    pub sig: SignatureBytes,
}

impl ExecutionReceipt {
    /// Creates the canonical message bytes used when signing an execution receipt.
    ///
    /// The fields are serialized in a deterministic order so both the signer
    /// and verifier produce identical bytes. This method is public so external
    /// components can obtain the exact payload that is covered by the
    /// [`sign_with_key`](Self::sign_with_key) and verification helpers.
    /// IMPORTANT: The order of fields here **must** match the order in
    /// [`verify_against_key`](Self::verify_against_key).
    pub fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
        // Using a simple concatenation of relevant fields.
        // A more robust canonicalization (e.g., IPLD canonical form, or JSON Canonicalization Scheme)
        // would be better for interoperability and to prevent canonicalization attacks.
        // For now, this is kept simple.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.job_id.to_string().as_bytes()); // Using .to_string().as_bytes()
        bytes.extend_from_slice(self.executor_did.to_string().as_bytes()); // DID as string
        bytes.extend_from_slice(self.result_cid.to_string().as_bytes()); // Using .to_string().as_bytes()
        bytes.extend_from_slice(&self.cpu_ms.to_le_bytes());
        bytes.push(self.success as u8);
        Ok(bytes)
    }

    /// Signs this receipt with the provided Ed25519 SigningKey.
    /// The `executor_did` in the receipt should match the DID derived from the public key
    /// corresponding to the `signing_key`.
    pub fn sign_with_key(mut self, signing_key: &SigningKey) -> Result<Self, CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_signature = sign_message(signing_key, &message);
        self.sig = SignatureBytes::from_ed_signature(ed_signature);
        Ok(self)
    }

    /// Verifies the signature of this receipt against the provided Ed25519 VerifyingKey.
    pub fn verify_against_key(&self, verifying_key: &VerifyingKey) -> Result<(), CommonError> {
        let message = self.to_signable_bytes()?;
        let ed_signature = self.sig.to_ed_signature()?;

        if verify_signature(verifying_key, &message, &ed_signature) {
            Ok(())
        } else {
            Err(CommonError::InternalError(
                "ExecutionReceipt signature verification failed".to_string(),
            ))
        }
    }

    /// Verifies the receipt signature against the public key encoded in the provided DID.
    pub fn verify_against_did(&self, did: &Did) -> Result<(), CommonError> {
        if &self.executor_did != did {
            return Err(CommonError::IdentityError("Executor DID mismatch".into()));
        }
        let vk = verifying_key_from_did_key(did)?;
        self.verify_against_key(&vk)
    }

    /// Verify the receipt using a custom [`DidResolver`].
    pub fn verify_with_resolver(&self, resolver: &dyn DidResolver) -> Result<(), CommonError> {
        let vk = resolver.resolve(&self.executor_did)?;
        self.verify_against_key(&vk)
    }
}

/// Placeholder function demonstrating use of common types for identity.
/// This needs to be updated to use the new key generation and DID format.
pub fn register_identity(info: &NodeInfo, did_method: &str) -> Result<String, CommonError> {
    if did_method == "key" {
        let (_sk, pk) = generate_ed25519_keypair();
        let did_string = did_key_from_verifying_key(&pk);
        // The Did struct constructor might need adjustment if it doesn't just take the full string.
        // Assuming Did::new can parse "did:key:z..." or similar.
        // For now, let's just use the string.
        Ok(format!(
            "Registered {} for node: {} (v{}). DID: {}",
            did_method, info.name, info.version, did_string
        ))
    } else {
        Ok(format!(
            "Registered {} identity for node: {} (v{})",
            did_method, info.name, info.version
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::ICN_CORE_VERSION; // Moved import here
                                      // For ExecutionReceipt tests, we might need a way to generate CIDs if not already available.
                                      // use icn_common::generate_cid; // Temporarily commented out due to unresolved import
    use icn_common::Cid;
    use std::str::FromStr; // Ensure FromStr is in scope for Did::from_str // Make sure Cid is in scope

    // Helper to create a dummy Cid for tests
    fn dummy_cid_for_test(s: &str) -> Cid {
        // Using raw codec (0x55) and SHA2-256 (multihash code 0x12).
        icn_common::Cid::new_v1_sha256(0x55, s.as_bytes())
    }

    #[test]
    fn roundtrip_sign_verify_message() {
        let (sk, pk) = generate_ed25519_keypair();
        let msg = b"icn-crypto-smoke-test message";
        let sig = sign_message(&sk, msg);
        assert!(verify_signature(&pk, msg, &sig));
        let bad_msg = b"some other message";
        assert!(!verify_signature(&pk, bad_msg, &sig));
    }

    #[test]
    fn did_key_format_generation() {
        let (_sk, pk) = generate_ed25519_keypair();
        let did = did_key_from_verifying_key(&pk);
        assert!(did.starts_with("did:key:z")); // base58btc multibase prefix for 'z'
                                               // A full Ed25519 public key (32 bytes) + multicodec prefix (0xed01, 2 bytes) = 34 bytes.
                                               // Base58 encoding of 34 bytes should be around 45-48 characters.
                                               // "did:key:z" is 10 chars. So total length ~55-58.
        assert!(
            did.len() > 50 && did.len() < 60,
            "DID length unexpected: {}",
            did.len()
        );
        println!("Generated did:key: {did}");

        // Example from spec (or known value) for cross-checking if available
        // let known_pk_bytes = hex::decode("F132C182A30937309A71732A3A97D353A63DA8B1C60E8FC8A19D8A8308D599DD").unwrap();
        // let known_pk = VerifyingKey::from_bytes(&known_pk_bytes_array).unwrap();
        // let known_did = "did:key:z6MkjL4FwS3np2p2NLiqH57sX99pZtG9x3Fy9bYh3xHqs14z";
        // assert_eq!(did_key_from_verifying_key(&known_pk), known_did);
    }

    #[test]
    fn execution_receipt_signing_and_verification() {
        let (signing_key, verifying_key) = generate_ed25519_keypair();
        let executor_did_string = did_key_from_verifying_key(&verifying_key);
        let executor_did = Did::from_str(&executor_did_string)
            .expect("Failed to parse DID from string for executor_did");

        let job_cid = dummy_cid_for_test("test_job_data_for_cid");
        let result_cid = dummy_cid_for_test("test_result_data_for_cid");

        let unsigned_receipt = ExecutionReceipt {
            job_id: job_cid.clone(),
            executor_did: executor_did.clone(),
            result_cid: result_cid.clone(),
            cpu_ms: 100,
            success: true,
            sig: SignatureBytes(vec![]), // Placeholder
        };

        let signed_receipt = unsigned_receipt
            .clone()
            .sign_with_key(&signing_key)
            .unwrap();
        assert_ne!(signed_receipt.sig.0, Vec::<u8>::new());

        // Verification should pass with the correct public key
        assert!(signed_receipt.verify_against_key(&verifying_key).is_ok());

        // Verification should fail with a different public key
        let (_other_sk, other_pk) = generate_ed25519_keypair();
        assert!(signed_receipt.verify_against_key(&other_pk).is_err());

        // Verification should fail if the receipt data is tampered with
        let mut tampered_receipt = signed_receipt.clone();
        tampered_receipt.cpu_ms = 200; // Modify some data
        assert!(tampered_receipt.verify_against_key(&verifying_key).is_err());

        // Verification should fail with a bad signature
        let mut bad_sig_receipt = signed_receipt.clone();
        bad_sig_receipt.sig.0[0] = bad_sig_receipt.sig.0[0].wrapping_add(1); // Flip a bit
        assert!(bad_sig_receipt.verify_against_key(&verifying_key).is_err());
    }

    // Test for the register_identity function to ensure it uses the new mechanisms
    #[test]
    fn test_register_identity_with_new_did_key() {
        let node_info = NodeInfo {
            name: "TestNodeCrypto".to_string(),
            version: ICN_CORE_VERSION.to_string(),
            status_message: "Testing new crypto".to_string(),
        };
        let result = register_identity(&node_info, "key").unwrap();
        assert!(result.contains("did:key:z"));
        assert!(result.contains("TestNodeCrypto"));
        println!("Registered identity with new crypto: {result}");
    }

    #[test]
    fn did_key_roundtrip() {
        let (_sk, pk) = generate_ed25519_keypair();
        let did_str = did_key_from_verifying_key(&pk);
        let did = Did::from_str(&did_str).unwrap();
        let recovered = verifying_key_from_did_key(&did).unwrap();
        assert_eq!(pk.as_bytes(), recovered.as_bytes());
    }

    #[test]
    fn receipt_verify_against_did() {
        let (sk, pk) = generate_ed25519_keypair();
        let did_str = did_key_from_verifying_key(&pk);
        let did = Did::from_str(&did_str).unwrap();
        let job_cid = dummy_cid_for_test("did_verify_job");
        let result_cid = dummy_cid_for_test("did_verify_res");

        let receipt = ExecutionReceipt {
            job_id: job_cid,
            executor_did: did.clone(),
            result_cid,
            cpu_ms: 1,
            success: true,
            sig: SignatureBytes(vec![]),
        };

        let signed = receipt.sign_with_key(&sk).unwrap();
        assert!(signed.verify_against_did(&did).is_ok());
    }

    #[test]
    fn tampering_success_bit_fails_verification() {
        let (sk, pk) = generate_ed25519_keypair();
        let did_str = did_key_from_verifying_key(&pk);
        let did = Did::from_str(&did_str).unwrap();

        let job_cid = dummy_cid_for_test("toggle_success_job");
        let result_cid = dummy_cid_for_test("toggle_success_result");

        let receipt = ExecutionReceipt {
            job_id: job_cid,
            executor_did: did.clone(),
            result_cid,
            cpu_ms: 1,
            success: true,
            sig: SignatureBytes(vec![]),
        };

        let mut signed = receipt.sign_with_key(&sk).unwrap();
        // Toggle the success field after signing; verification should fail
        signed.success = false;
        assert!(signed.verify_against_did(&did).is_err());
    }

    #[test]
    fn did_web_generation_and_parse() {
        let did_str = did_web_from_parts("example.com", &["user", "alice"]).unwrap();
        assert_eq!(did_str, "did:web:example.com:user:alice");
        let did = Did::from_str(&did_str).unwrap();
        let (domain, path) = parse_did_web(&did).unwrap();
        assert_eq!(domain, "example.com");
        assert_eq!(path, vec!["user".to_string(), "alice".to_string()]);
    }

    #[test]
    fn verify_with_web_did_resolver() {
        let (sk, pk) = generate_ed25519_keypair();
        let did_str = did_web_from_parts("example.com", &[]).unwrap();
        let did = Did::from_str(&did_str).unwrap();

        let job_cid = dummy_cid_for_test("web_job");
        let result_cid = dummy_cid_for_test("web_res");

        let receipt = ExecutionReceipt {
            job_id: job_cid,
            executor_did: did.clone(),
            result_cid,
            cpu_ms: 1,
            success: true,
            sig: SignatureBytes(vec![]),
        };

        let mut resolver = WebDidResolver::new();
        resolver.insert(did_str.clone(), pk);

        let signed = receipt.sign_with_key(&sk).unwrap();
        assert!(signed.verify_with_resolver(&resolver).is_ok());
    }

    #[test]
    fn did_web_rejects_invalid_segment() {
        assert!(did_web_from_parts("example.com", &["bad:seg"]).is_err());
    }

    #[test]
    fn did_web_rejects_long_segment() {
        let long = "a".repeat(70);
        assert!(did_web_from_parts("example.com", &[&long]).is_err());
    }

    #[test]
    fn key_rotation_updates_store() {
        let (sk, pk) = generate_ed25519_keypair();
        let did_str = did_key_from_verifying_key(&pk);
        let did = Did::from_str(&did_str).unwrap();
        let mut store = InMemoryKeyStore::default();
        store.store_signing_key(did.clone(), sk);
        let new_did = store.rotate_ed25519(&did).unwrap();
        assert_ne!(did, new_did);
        assert!(store.get_signing_key(&new_did).is_some());
        assert!(store.get_signing_key(&did).is_none());
    }

    #[test]
    fn did_peer_roundtrip_and_sign() {
        let (sk, pk) = generate_ed25519_keypair();
        let did_str = did_peer_from_verifying_key(&pk);
        let did = Did::from_str(&did_str).unwrap();
        let recovered = verifying_key_from_did_peer(&did).unwrap();
        assert_eq!(pk.as_bytes(), recovered.as_bytes());

        let msg = b"peer did test";
        let sig = sign_message(&sk, msg);
        assert!(verify_signature(&recovered, msg, &sig));

        let resolver = PeerDidResolver;
        assert_eq!(resolver.resolve(&did).unwrap().as_bytes(), pk.as_bytes());
    }

    #[test]
    #[ignore]
    fn web_did_http_resolution_and_verify() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let (sk, pk) = generate_ed25519_keypair();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let domain = format!("localhost:{}", addr.port());
        let did_str = did_web_from_parts(&domain, &[]).unwrap();
        let did = Did::from_str(&did_str).unwrap();

        let doc = icn_common::DidDocument {
            id: did.clone(),
            public_key: pk.as_bytes().to_vec(),
        };
        let doc_json = serde_json::to_string(&doc).unwrap();

        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 512];
                let _ = stream.read(&mut buf);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                    doc_json.len(),
                    doc_json
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });

        let job_cid = dummy_cid_for_test("web_http_job");
        let result_cid = dummy_cid_for_test("web_http_result");
        let receipt = ExecutionReceipt {
            job_id: job_cid,
            executor_did: did.clone(),
            result_cid,
            cpu_ms: 1,
            success: true,
            sig: SignatureBytes(vec![]),
        };

        let resolver = WebDidResolver::new();
        let signed = receipt.sign_with_key(&sk).unwrap();
        assert!(signed.verify_with_resolver(&resolver).is_ok());

        handle.join().unwrap();
    }

    #[test]
    fn receipt_to_signable_bytes_public() {
        let (_sk, pk) = generate_ed25519_keypair();
        let did = Did::from_str(&did_key_from_verifying_key(&pk)).unwrap();
        let job_cid = dummy_cid_for_test("bytes_job");
        let result_cid = dummy_cid_for_test("bytes_res");
        let receipt = ExecutionReceipt {
            job_id: job_cid.clone(),
            executor_did: did.clone(),
            result_cid: result_cid.clone(),
            cpu_ms: 42,
            success: true,
            sig: SignatureBytes(vec![]),
        };
        let bytes = receipt.to_signable_bytes().unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(job_cid.to_string().as_bytes());
        expected.extend_from_slice(did.to_string().as_bytes());
        expected.extend_from_slice(result_cid.to_string().as_bytes());
        expected.extend_from_slice(&42u64.to_le_bytes());
        expected.push(1);
        assert_eq!(bytes, expected);
    }

    #[test]
    fn membership_resolver_and_policy_enforcer() {
        let did = Did::from_str("did:icn:test:alice").unwrap();
        let scope = NodeScope("test_scope".into());
        let mut resolver = InMemoryMembershipResolver::new();
        resolver.add_member(scope.clone(), did.clone());

        assert!(resolver.is_member(&did, &scope));

        let enforcer = MembershipPolicyEnforcer::new(resolver);
        assert!(enforcer.check_permission(&did, &scope).is_ok());

        let outsider = Did::from_str("did:icn:test:bob").unwrap();
        assert!(enforcer.check_permission(&outsider, &scope).is_err());
    }
}
