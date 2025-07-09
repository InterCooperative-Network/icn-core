//! Signer implementations for the ICN runtime.

use super::errors::HostAbiError;
use icn_common::{CommonError, Did};
use icn_identity::{
    generate_ed25519_keypair, sign_message,
    verify_signature as identity_verify_signature, SigningKey, VerifyingKey,
};
use std::path::Path;

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
        let verifying_key = VerifyingKey::from_bytes(
            public_key_bytes
                .try_into()
                .map_err(|_| HostAbiError::SignatureError("Invalid public key length".to_string()))?,
        )
        .map_err(|e| HostAbiError::SignatureError(format!("Invalid public key: {e}")))?;

        // Convert signature bytes to EdSignature
        let signature = icn_identity::EdSignature::from_bytes(
            signature_bytes
                .try_into()
                .map_err(|_| HostAbiError::SignatureError("Invalid signature length".to_string()))?,
        );

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
}

/// HSM key store trait for hardware security module integration
pub trait HsmKeyStore: Send + Sync {
    /// Fetch the Ed25519 keypair used for signing.
    fn fetch_ed25519_keypair(&self) -> Result<(SigningKey, VerifyingKey), CommonError>;
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
        // This is a placeholder implementation
        // In a real implementation, you'd decrypt the file using the passphrase
        let _path = path.as_ref();
        let _passphrase = passphrase;
        
        // For now, just generate a new keypair
        let (sk, pk) = generate_ed25519_keypair();
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
        let verifying_key = VerifyingKey::from_bytes(
            public_key_bytes
                .try_into()
                .map_err(|_| HostAbiError::SignatureError("Invalid public key length".to_string()))?,
        )
        .map_err(|e| HostAbiError::SignatureError(format!("Invalid public key: {e}")))?;

        // Convert signature bytes to EdSignature
        let signature = icn_identity::EdSignature::from_bytes(
            signature_bytes
                .try_into()
                .map_err(|_| HostAbiError::SignatureError("Invalid signature length".to_string()))?,
        );

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
}

// Add std::str::FromStr import for Did::from_str
use std::str::FromStr; 