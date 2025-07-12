use crate::Credential;
use icn_common::{ZkCredentialProof, ZkProofType};
use thiserror::Error;

/// Errors that can occur when verifying zero-knowledge proofs.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ZkError {
    /// The proof backend isn't supported by the verifier.
    #[error("unsupported proof backend: {0:?}")]
    UnsupportedBackend(ZkProofType),
    /// The proof structure is invalid or malformed.
    #[error("invalid proof structure")]
    InvalidProof,
    /// Verification failed due to an unspecified reason.
    #[error("verification failed")]
    VerificationFailed,
}

/// Trait for verifying zero-knowledge credential proofs.
pub trait ZkVerifier: Send + Sync {
    /// Verify the supplied [`ZkCredentialProof`]. Returns `Ok(true)` if the proof
    /// is valid and corresponds to the verifier's backend.
    fn verify(&self, proof: &ZkCredentialProof) -> Result<bool, ZkError>;
}

/// Trait for generating zero-knowledge proofs from credentials.
pub trait ZkProver: Send + Sync {
    /// Produce a [`ZkCredentialProof`] proving selective fields of `credential`.
    fn prove(
        &self,
        credential: &Credential,
        disclosed_fields: &[String],
    ) -> Result<ZkCredentialProof, ZkError>;
}

/// Verifier implementation for the Bulletproofs proving system.
#[derive(Debug, Default)]
pub struct BulletproofsVerifier;

impl ZkVerifier for BulletproofsVerifier {
    fn verify(&self, proof: &ZkCredentialProof) -> Result<bool, ZkError> {
        if proof.backend != ZkProofType::Bulletproofs {
            return Err(ZkError::UnsupportedBackend(proof.backend.clone()));
        }
        if proof.proof.is_empty() {
            return Err(ZkError::InvalidProof);
        }
        // TODO: integrate real Bulletproofs verification once available.
        Ok(true)
    }
}

/// Prover implementation using the Bulletproofs proving system.
#[derive(Debug, Default)]
pub struct BulletproofsProver;

impl ZkProver for BulletproofsProver {
    fn prove(
        &self,
        credential: &Credential,
        disclosed_fields: &[String],
    ) -> Result<ZkCredentialProof, ZkError> {
        // TODO: integrate real Bulletproofs proof generation once available.
        Ok(ZkCredentialProof {
            issuer: credential.issuer.clone(),
            holder: credential.holder.clone(),
            claim_type: credential.claim_type.clone(),
            proof: vec![42],
            schema: credential.schema.clone(),
            disclosed_fields: disclosed_fields.to_vec(),
            challenge: None,
            backend: ZkProofType::Bulletproofs,
        })
    }
}

/// Simple verifier used for testing that always returns `true` when the proof
/// structure is well formed.
#[derive(Debug, Default)]
pub struct DummyVerifier;

impl ZkVerifier for DummyVerifier {
    fn verify(&self, proof: &ZkCredentialProof) -> Result<bool, ZkError> {
        if proof.proof.is_empty() {
            return Err(ZkError::InvalidProof);
        }
        Ok(true)
    }
}

/// Simple prover used for testing that produces placeholder proofs.
#[derive(Debug, Default)]
pub struct DummyProver;

impl ZkProver for DummyProver {
    fn prove(
        &self,
        credential: &Credential,
        disclosed_fields: &[String],
    ) -> Result<ZkCredentialProof, ZkError> {
        Ok(ZkCredentialProof {
            issuer: credential.issuer.clone(),
            holder: credential.holder.clone(),
            claim_type: credential.claim_type.clone(),
            proof: vec![1, 2, 3],
            schema: credential.schema.clone(),
            disclosed_fields: disclosed_fields.to_vec(),
            challenge: None,
            backend: ZkProofType::Groth16,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{Cid, Did};

    fn dummy_cid(s: &str) -> Cid {
        Cid::new_v1_sha256(0x55, s.as_bytes())
    }

    fn dummy_proof(backend: ZkProofType) -> ZkCredentialProof {
        ZkCredentialProof {
            issuer: Did::new("key", "issuer"),
            holder: Did::new("key", "holder"),
            claim_type: "test".into(),
            proof: vec![1, 2, 3],
            schema: dummy_cid("schema"),
            disclosed_fields: Vec::new(),
            challenge: None,
            backend,
        }
    }

    #[test]
    fn dummy_verifier_ok() {
        let proof = dummy_proof(ZkProofType::Groth16);
        let verifier = DummyVerifier;
        assert!(verifier.verify(&proof).unwrap());
    }

    #[test]
    fn bulletproofs_backend_mismatch() {
        let proof = dummy_proof(ZkProofType::Groth16);
        let verifier = BulletproofsVerifier;
        assert!(matches!(
            verifier.verify(&proof),
            Err(ZkError::UnsupportedBackend(_))
        ));
    }

    #[test]
    fn bulletproofs_verifier_ok() {
        let proof = dummy_proof(ZkProofType::Bulletproofs);
        let verifier = BulletproofsVerifier;
        assert!(verifier.verify(&proof).unwrap());
    }

    #[test]
    fn dummy_verifier_rejects_empty_proof() {
        let mut proof = dummy_proof(ZkProofType::Groth16);
        proof.proof.clear();
        let verifier = DummyVerifier;
        assert_eq!(verifier.verify(&proof), Err(ZkError::InvalidProof));
    }

    #[test]
    fn bulletproofs_invalid_proof_error() {
        let mut proof = dummy_proof(ZkProofType::Bulletproofs);
        proof.proof.clear();
        let verifier = BulletproofsVerifier;
        assert_eq!(verifier.verify(&proof), Err(ZkError::InvalidProof));
    }
}
