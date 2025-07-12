use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Proof data for a zero-knowledge credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkCredentialProof {
    /// Encoded proof bytes.
    #[serde(with = "serde_bytes")]
    pub proof: Vec<u8>,
    /// Compressed commitment bytes.
    #[serde(with = "serde_bytes")]
    pub commitment: Vec<u8>,
    /// Bit size of the committed value.
    pub bit_size: usize,
}

/// Errors that can occur during zero-knowledge verification.
#[derive(Debug, Error)]
pub enum ZkError {
    #[error("invalid proof: {0}")]
    InvalidProof(String),
    #[error("verification failed: {0}")]
    VerificationFailed(String),
}

/// Trait for verifying zero-knowledge credential proofs.
pub trait ZkVerifier {
    /// Verify the supplied proof, returning `true` if valid.
    fn verify(&self, proof: &ZkCredentialProof) -> Result<bool, ZkError>;
}

/// Verifier implementation using Bulletproofs range proofs.
pub struct BulletproofsVerifier;

impl ZkVerifier for BulletproofsVerifier {
    fn verify(&self, proof_data: &ZkCredentialProof) -> Result<bool, ZkError> {
        use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
        use curve25519_dalek::ristretto::CompressedRistretto;
        use merlin::Transcript;

        let proof = RangeProof::from_bytes(&proof_data.proof)
            .map_err(|e| ZkError::InvalidProof(format!("{e:?}")))?;
        if proof_data.commitment.len() != 32 {
            return Err(ZkError::InvalidProof("commitment length".into()));
        }
        let bytes: [u8; 32] = proof_data.commitment[..]
            .try_into()
            .expect("length checked");
        let commitment = CompressedRistretto(bytes);
        let bp_gens = BulletproofGens::new(proof_data.bit_size, 1);
        let pc_gens = PedersenGens::default();
        let mut transcript = Transcript::new(b"ZkCredential");
        proof
            .verify_single(
                &bp_gens,
                &pc_gens,
                &mut transcript,
                &commitment,
                proof_data.bit_size,
            )
            .map_err(|e| ZkError::VerificationFailed(format!("{e:?}")))?;
        Ok(true)
    }
}

/// No-op verifier used for testing and development.
#[derive(Default)]
pub struct DummyVerifier;

impl ZkVerifier for DummyVerifier {
    fn verify(&self, proof: &ZkCredentialProof) -> Result<bool, ZkError> {
        if proof.proof.is_empty() || proof.commitment.len() != 32 {
            return Err(ZkError::InvalidProof("invalid structure".into()));
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_verifier_accepts_well_formed_proof() {
        let proof = ZkCredentialProof {
            proof: vec![1, 2, 3],
            commitment: vec![0; 32],
            bit_size: 32,
        };
        let v = DummyVerifier;
        assert!(v.verify(&proof).unwrap());
    }
}
