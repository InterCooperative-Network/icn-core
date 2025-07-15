//! Signer implementations for the ICN runtime.

use super::errors::HostAbiError;
use icn_common::{CommonError, Did};
use icn_identity::{
    generate_ed25519_keypair, sign_message, verify_signature as identity_verify_signature,
    SigningKey, VerifyingKey,
};
use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::Aes256Gcm;
use bs58;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use zeroize::Zeroize;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const PBKDF2_ITERS: u32 = 100_000;

/// Updated Signer trait to be synchronous and match new crypto capabilities
pub trait Signer: Send + Sync + std::fmt::Debug {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, HostAbiError>;
    fn verify(
        &self,
        payload: &[u8],
        signature: &[u8],
        public_key_bytes: &[u8],
    ) -> Result<bool, HostAbiError>;
    fn public_key_bytes(&self) -> Vec<u8>;
    fn did(&self) -> Did;
    fn verifying_key_ref(&self) -> &VerifyingKey;
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Helper function to create DID from verifying key
fn create_did_from_verifying_key(vk: &VerifyingKey) -> Did {
    // For now, create a simple DID string - in real implementation this would use proper DID:key format
    let key_bytes = vk.to_bytes();
    let key_hex = hex::encode(key_bytes);
    Did::from_str(&format!("did:key:z{}", key_hex))
        .unwrap_or_else(|_| Did::from_str("did:example:invalid").unwrap())
}

/// Stub signer for testing
pub struct StubSigner {
    sk: SigningKey,
    pk: VerifyingKey,
    did: Did, // Store the actual DID
}

impl std::fmt::Debug for StubSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StubSigner {{ did: {} }}", self.did)
    }
}

impl StubSigner {
    pub fn new() -> Self {
        let (sk, pk) = generate_ed25519_keypair();
        Self::new_with_keys(sk, pk)
    }

    pub fn new_with_keys(sk: SigningKey, pk: VerifyingKey) -> Self {
        let did = create_did_from_verifying_key(&pk);
        Self { sk, pk, did }
    }

    pub fn verifying_key_ref(&self) -> &VerifyingKey {
        &self.pk
    }
}

impl Signer for StubSigner {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, HostAbiError> {
        let signature = sign_message(&self.sk, payload);
        Ok(signature.to_vec())
    }

    fn verify(
        &self,
        payload: &[u8],
        signature_bytes: &[u8],
        public_key_bytes: &[u8],
    ) -> Result<bool, HostAbiError> {
        // Convert bytes to VerifyingKey
        let verifying_key =
            VerifyingKey::from_bytes(public_key_bytes.try_into().map_err(|_| {
                HostAbiError::SignatureError("Invalid public key length".to_string())
            })?)
            .map_err(|e| HostAbiError::SignatureError(format!("Invalid public key: {e}")))?;

        // Convert signature bytes to EdSignature
        let signature =
            icn_identity::EdSignature::from_bytes(signature_bytes.try_into().map_err(|_| {
                HostAbiError::SignatureError("Invalid signature length".to_string())
            })?);

        // Verify the signature
        let is_valid = identity_verify_signature(&verifying_key, payload, &signature);
        Ok(is_valid)
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        self.pk.to_bytes().to_vec()
    }

    fn did(&self) -> Did {
        self.did.clone()
    }

    fn verifying_key_ref(&self) -> &VerifyingKey {
        &self.pk
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// HSM key store trait for hardware security module integration
pub trait HsmKeyStore: Send + Sync {
    /// Fetch the Ed25519 keypair used for signing.
    fn fetch_ed25519_keypair(&self) -> Result<(SigningKey, VerifyingKey), CommonError>;
}

/// Simple file-based HSM implementation used for testing.
/// The `path` is expected to contain a base58 encoded private key.
pub struct ExampleHsm {
    path: std::path::PathBuf,
    _key_id: Option<String>,
}

impl ExampleHsm {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            _key_id: None,
        }
    }

    pub fn with_key<P: AsRef<std::path::Path>>(path: P, key_id: String) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            _key_id: Some(key_id),
        }
    }
}

impl HsmKeyStore for ExampleHsm {
    fn fetch_ed25519_keypair(&self) -> Result<(SigningKey, VerifyingKey), CommonError> {
        let sk_bs58 =
            std::fs::read_to_string(&self.path).map_err(|e| CommonError::IoError(e.to_string()))?;
        let sk_bytes = bs58::decode(sk_bs58.trim())
            .into_vec()
            .map_err(|_| CommonError::IdentityError("Invalid base58 key".into()))?;
        let sk_array: [u8; 32] = sk_bytes
            .try_into()
            .map_err(|_| CommonError::IdentityError("Invalid key length".into()))?;
        let sk = SigningKey::from_bytes(&sk_array);
        let pk = sk.verifying_key();
        Ok((sk, pk))
    }
}

/// Production Ed25519 signer
pub struct Ed25519Signer {
    sk: SigningKey,
    pk: VerifyingKey,
    did: Did,
}

impl std::fmt::Debug for Ed25519Signer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ed25519Signer {{ did: {} }}", self.did)
    }
}

impl Ed25519Signer {
    /// Create a new signer with the given signing key.
    pub fn new(sk: SigningKey) -> Self {
        let pk = sk.verifying_key();
        let did = create_did_from_verifying_key(&pk);
        Self { sk, pk, did }
    }

    /// Create a new signer with explicit keys.
    pub fn new_with_keys(sk: SigningKey, pk: VerifyingKey) -> Self {
        let did = create_did_from_verifying_key(&pk);
        Self { sk, pk, did }
    }

    /// Create a signer from an encrypted file.
    pub fn from_encrypted_file<P: AsRef<Path>>(
        path: P,
        passphrase: &[u8],
    ) -> Result<Self, CommonError> {
        use aes_gcm::aead::generic_array::GenericArray;

        let data = std::fs::read(path).map_err(|e| CommonError::IoError(e.to_string()))?;
        if data.len() <= SALT_LEN + NONCE_LEN {
            return Err(CommonError::IoError("encrypted key file truncated".into()));
        }
        let salt = &data[..SALT_LEN];
        let nonce = &data[SALT_LEN..SALT_LEN + NONCE_LEN];
        let ciphertext = &data[SALT_LEN + NONCE_LEN..];

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(passphrase, salt, PBKDF2_ITERS, &mut key);
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
        let plain = cipher
            .decrypt(GenericArray::from_slice(nonce), ciphertext)
            .map_err(|_| CommonError::CryptoError("key decryption failed".into()))?;
        key.zeroize();

        if plain.len() != 32 {
            return Err(CommonError::IdentityError(
                "invalid decrypted key length".into(),
            ));
        }
        let mut sk_bytes = [0u8; 32];
        sk_bytes.copy_from_slice(&plain);
        let sk = SigningKey::from_bytes(&sk_bytes);
        sk_bytes.zeroize();
        let pk = sk.verifying_key();
        Ok(Self::new_with_keys(sk, pk))
    }

    /// Create a signer from an HSM.
    pub fn from_hsm(hsm: &dyn HsmKeyStore) -> Result<Self, CommonError> {
        let (sk, pk) = hsm.fetch_ed25519_keypair()?;
        Ok(Self::new_with_keys(sk, pk))
    }

    pub fn verifying_key_ref(&self) -> &VerifyingKey {
        &self.pk
    }
}

impl Signer for Ed25519Signer {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, HostAbiError> {
        let signature = sign_message(&self.sk, payload);
        Ok(signature.to_vec())
    }

    fn verify(
        &self,
        payload: &[u8],
        signature_bytes: &[u8],
        public_key_bytes: &[u8],
    ) -> Result<bool, HostAbiError> {
        // Convert bytes to VerifyingKey
        let verifying_key =
            VerifyingKey::from_bytes(public_key_bytes.try_into().map_err(|_| {
                HostAbiError::SignatureError("Invalid public key length".to_string())
            })?)
            .map_err(|e| HostAbiError::SignatureError(format!("Invalid public key: {e}")))?;

        // Convert signature bytes to EdSignature
        let signature =
            icn_identity::EdSignature::from_bytes(signature_bytes.try_into().map_err(|_| {
                HostAbiError::SignatureError("Invalid signature length".to_string())
            })?);

        // Verify the signature
        let is_valid = identity_verify_signature(&verifying_key, payload, &signature);
        Ok(is_valid)
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        self.pk.to_bytes().to_vec()
    }

    fn did(&self) -> Did {
        self.did.clone()
    }

    fn verifying_key_ref(&self) -> &VerifyingKey {
        &self.pk
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Add std::str::FromStr import for Did::from_str
use std::str::FromStr;
