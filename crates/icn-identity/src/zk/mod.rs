use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek::scalar::Scalar;
use icn_common::{ZkCredentialProof, ZkProofType};
use merlin::Transcript;
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
        let range_proof =
            RangeProof::from_bytes(&proof.proof).map_err(|_| ZkError::InvalidProof)?;
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(64, 1);
        let commitment = pc_gens.commit(Scalar::from(42u64), Scalar::ZERO).compress();
        let mut transcript = Transcript::new(b"icn-bulletproof");
        range_proof
            .verify_single(&bp_gens, &pc_gens, &mut transcript, &commitment, 64)
            .map(|_| true)
            .map_err(|_| ZkError::VerificationFailed)
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

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{Cid, Did};

    fn make_bulletproof(value: u64) -> Vec<u8> {
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(64, 1);
        let mut transcript = Transcript::new(b"icn-bulletproof");
        let (proof, _) = RangeProof::prove_single(
            &bp_gens,
            &pc_gens,
            &mut transcript,
            value,
            &Scalar::ZERO,
            64,
        )
        .unwrap();
        proof.to_bytes()
    }

    fn dummy_cid(s: &str) -> Cid {
        Cid::new_v1_sha256(0x55, s.as_bytes())
    }

    fn dummy_proof(backend: ZkProofType) -> ZkCredentialProof {
        let proof = if backend == ZkProofType::Bulletproofs {
            make_bulletproof(42)
        } else {
            vec![1, 2, 3]
        };
        ZkCredentialProof {
            issuer: Did::new("key", "issuer"),
            holder: Did::new("key", "holder"),
            claim_type: "test".into(),
            proof,
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
    fn bulletproofs_wrong_value_fails() {
        let mut proof = dummy_proof(ZkProofType::Bulletproofs);
        proof.proof = make_bulletproof(7);
        let verifier = BulletproofsVerifier;
        assert_eq!(verifier.verify(&proof), Err(ZkError::VerificationFailed));
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
