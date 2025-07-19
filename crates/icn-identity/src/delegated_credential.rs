use crate::{sign_message, verify_signature, DidResolver, SignatureBytes};
use ed25519_dalek::{SigningKey, VerifyingKey};
use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};

/// Credential indicating the issuer has delegated authority to the delegatee.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegatedCredential {
    /// DID of the delegator.
    pub issuer: Did,
    /// DID receiving the delegated authority.
    pub delegatee: Did,
    /// Signature from the issuer over the issuer and delegatee DIDs.
    pub signature: SignatureBytes,
}

impl DelegatedCredential {
    /// Create and sign a new delegated credential.
    pub fn new(issuer: Did, delegatee: Did, key: &SigningKey) -> Self {
        let mut bytes = issuer.to_string().into_bytes();
        bytes.extend_from_slice(delegatee.to_string().as_bytes());
        let sig = sign_message(key, &bytes);
        Self {
            issuer,
            delegatee,
            signature: SignatureBytes::from_ed_signature(sig),
        }
    }

    /// Verify this credential against the provided verifying key.
    pub fn verify(&self, key: &VerifyingKey) -> Result<(), CommonError> {
        let mut bytes = self.issuer.to_string().into_bytes();
        bytes.extend_from_slice(self.delegatee.to_string().as_bytes());
        let ed = self.signature.to_ed_signature()?;
        if verify_signature(key, &bytes, &ed) {
            Ok(())
        } else {
            Err(CommonError::IdentityError(
                "delegation signature invalid".into(),
            ))
        }
    }
}

/// Verify a chain of delegated credentials starting from `root_issuer`.
///
/// Each credential must be issued by the previous delegate and be
/// signed correctly. The function returns the DID of the final delegate
/// in the chain if all signatures verify.
pub fn verify_delegation_chain(
    root_issuer: &Did,
    chain: &[DelegatedCredential],
    resolver: &dyn DidResolver,
) -> Result<Did, CommonError> {
    let mut current_did = root_issuer.clone();
    let mut current_key = resolver.resolve(&current_did)?;

    for cred in chain {
        if cred.issuer != current_did {
            return Err(CommonError::IdentityError(
                "issuer mismatch in delegation chain".into(),
            ));
        }
        cred.verify(&current_key)?;
        current_did = cred.delegatee.clone();
        current_key = resolver.resolve(&current_did)?;
    }
    Ok(current_did)
}
