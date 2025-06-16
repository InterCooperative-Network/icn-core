//! Minimal identity primitives: key-gen, DID:key, signing, verification.
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

// Original imports that might still be needed or can be cleaned up later:
// use icn_common::{NodeInfo, CommonError, Did, ICN_CORE_VERSION, Cid};
// use serde::{Serialize, Deserialize};

use icn_common::{Cid, CommonError, Did, NodeInfo};
use serde::{Deserialize, Serialize}; // Keep serde for ExecutionReceipt

pub use ed25519_dalek::{
    Signature as EdSignature, Signer, SigningKey, VerifyingKey, SIGNATURE_LENGTH,
}; // Made pub, removed unused Verifier initially, then re-added Keys
use multibase::{encode as multibase_encode, Base};
use rand_core::OsRng;
use unsigned_varint::encode as varint_encode;

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

// Wrapper for ed25519_dalek::Signature to allow serde if needed directly on it
// and to be distinct if we were to support multiple signature types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureBytes(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl SignatureBytes {
    pub fn from_ed_signature(ed_sig: ed25519_dalek::Signature) -> Self {
        SignatureBytes(ed_sig.to_bytes().to_vec())
    }
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
    /// Creates the canonical message bytes for signing.
    /// The fields must be serialized in a deterministic way.
    /// IMPORTANT: The order of fields here MUST match the order in `verify_signature`.
    fn to_signable_bytes(&self) -> Result<Vec<u8>, CommonError> {
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
        // Using raw codec (0x55) and SHA2-256 (multihash code 0x12) as example parameters for new_v1_dummy.
        // These values might need adjustment based on specific requirements for dummy CIDs in ICN.
        icn_common::Cid::new_v1_dummy(0x55, 0x12, s.as_bytes())
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
}
