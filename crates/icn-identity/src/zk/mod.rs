use icn_common::{Cid, ZkCredentialProof, ZkProofType};
use core::convert::TryInto;
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

/// Trait for generating zero-knowledge proofs for credentials.
pub trait ZkProver: Send + Sync {
    fn prove(
        &self,
        credential: &crate::credential::Credential,
        fields: &[&str],
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
        if proof.proof.len() < 32 {
            return Err(ZkError::InvalidProof);
        }

        use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
        use curve25519_dalek::ristretto::CompressedRistretto;
        use merlin::Transcript;

        let (proof_bytes, commitment_bytes) = proof.proof.split_at(proof.proof.len() - 32);
        let range_proof = RangeProof::from_bytes(proof_bytes).map_err(|_| ZkError::InvalidProof)?;
        let commitment = CompressedRistretto(commitment_bytes.try_into().unwrap());

        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(64, 1);
        let mut transcript = Transcript::new(b"ZkCredentialProof");

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

#[derive(Debug, Default)]
pub struct DummyProver;

impl ZkProver for DummyProver {
    fn prove(
        &self,
        credential: &crate::credential::Credential,
        fields: &[&str],
    ) -> Result<ZkCredentialProof, ZkError> {
        Ok(ZkCredentialProof {
            issuer: credential.issuer.clone(),
            holder: credential.holder.clone(),
            claim_type: "test".into(),
            proof: vec![1, 2, 3],
            schema: credential
                .schema
                .clone()
                .unwrap_or_else(|| Cid::new_v1_sha256(0x55, b"dummy")),
            disclosed_fields: fields.iter().map(|f| f.to_string()).collect(),
            challenge: None,
            backend: ZkProofType::Groth16,
        })
    }
}

#[derive(Debug, Default)]
pub struct BulletproofsProver;

impl ZkProver for BulletproofsProver {
    fn prove(
        &self,
        credential: &crate::credential::Credential,
        fields: &[&str],
    ) -> Result<ZkCredentialProof, ZkError> {
        use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
        use curve25519_dalek::scalar::Scalar;
        use merlin::Transcript;
        use rand_core::OsRng;

        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(64, 1);
        let blinding = Scalar::random(&mut OsRng);
        let mut transcript = Transcript::new(b"ZkCredentialProof");
        let (proof, commit) = RangeProof::prove_single(
            &bp_gens,
            &pc_gens,
            &mut transcript,
            42u64,
            &blinding,
            64,
        )
        .map_err(|_| ZkError::VerificationFailed)?;

        let mut bytes = proof.to_bytes();
        bytes.extend_from_slice(commit.as_bytes());

        Ok(ZkCredentialProof {
            issuer: credential.issuer.clone(),
            holder: credential.holder.clone(),
            claim_type: "test".into(),
            proof: bytes,
            schema: credential
                .schema
                .clone()
                .unwrap_or_else(|| Cid::new_v1_sha256(0x55, b"bp")),
            disclosed_fields: fields.iter().map(|f| f.to_string()).collect(),
            challenge: None,
            backend: ZkProofType::Bulletproofs,
        })
    }
}

/// Prover implementation using the Groth16 proving system from the `icn-zk` crate.
#[derive(Debug, Default)]
pub struct Groth16Prover;

impl ZkProver for Groth16Prover {
    fn prove(
        &self,
        credential: &crate::credential::Credential,
        fields: &[&str],
    ) -> Result<ZkCredentialProof, ZkError> {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::SeedableRng;
        use icn_zk::{setup, prove, AgeOver18Circuit};

        // Extract birth_year claim from credential
        let birth_year = credential
            .claims
            .get("birth_year")
            .ok_or(ZkError::InvalidProof)?
            .parse::<u64>()
            .map_err(|_| ZkError::InvalidProof)?;

        const CURRENT_YEAR: u64 = 2020;
        const RNG_SEED: u64 = 42;

        let circuit = AgeOver18Circuit {
            birth_year,
            current_year: CURRENT_YEAR,
        };
        let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(RNG_SEED);
        let pk = setup(circuit.clone(), &mut rng).map_err(|_| ZkError::VerificationFailed)?;
        let proof = prove(&pk, circuit, &mut rng).map_err(|_| ZkError::VerificationFailed)?;

        let mut proof_bytes = Vec::new();
        proof
            .serialize_compressed(&mut proof_bytes)
            .map_err(|_| ZkError::VerificationFailed)?;

        Ok(ZkCredentialProof {
            issuer: credential.issuer.clone(),
            holder: credential.holder.clone(),
            claim_type: "age_over_18".into(),
            proof: proof_bytes,
            schema: credential
                .schema
                .clone()
                .unwrap_or_else(|| Cid::new_v1_sha256(0x55, b"groth16")),
            disclosed_fields: fields.iter().map(|f| f.to_string()).collect(),
            challenge: None,
            backend: ZkProofType::Groth16,
        })
    }
}

/// Verifier for Groth16 proofs generated by [`Groth16Prover`].
#[derive(Debug, Default)]
pub struct Groth16Verifier;

impl ZkVerifier for Groth16Verifier {
    fn verify(&self, proof: &ZkCredentialProof) -> Result<bool, ZkError> {
        if proof.backend != ZkProofType::Groth16 {
            return Err(ZkError::UnsupportedBackend(proof.backend.clone()));
        }
        use ark_bn254::Fr;
        use ark_groth16::Proof;
        use ark_serialize::CanonicalDeserialize;
        use ark_std::rand::SeedableRng;
        use icn_zk::{setup, verify, AgeOver18Circuit, prepare_vk};

        if proof.proof.is_empty() {
            return Err(ZkError::InvalidProof);
        }

        const CURRENT_YEAR: u64 = 2020;
        const RNG_SEED: u64 = 42;

        let mut reader = &proof.proof[..];
        let groth_proof = Proof::<ark_bn254::Bn254>::deserialize_compressed(&mut reader)
            .map_err(|_| ZkError::InvalidProof)?;

        // Recreate verifying key deterministically
        let circuit = AgeOver18Circuit {
            birth_year: 0,
            current_year: CURRENT_YEAR,
        };
        let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(RNG_SEED);
        let pk = setup(circuit.clone(), &mut rng).map_err(|_| ZkError::VerificationFailed)?;
        let vk = prepare_vk(&pk);

        verify(&vk, &groth_proof, &[Fr::from(CURRENT_YEAR)])
            .map_err(|_| ZkError::VerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{Cid, Did};

    fn dummy_cid(s: &str) -> Cid {
        Cid::new_v1_sha256(0x55, s.as_bytes())
    }

    fn bulletproof_bytes() -> Vec<u8> {
        use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
        use curve25519_dalek::scalar::Scalar;
        use merlin::Transcript;
        use rand_core::OsRng;

        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(64, 1);
        let blinding = Scalar::random(&mut OsRng);
        let mut transcript = Transcript::new(b"ZkCredentialProof");
        let (proof, commit) = RangeProof::prove_single(
            &bp_gens,
            &pc_gens,
            &mut transcript,
            42u64,
            &blinding,
            64,
        )
        .expect("range proof generation should succeed");

        let mut bytes = proof.to_bytes();
        bytes.extend_from_slice(commit.as_bytes());
        bytes
    }

    fn dummy_proof(backend: ZkProofType) -> ZkCredentialProof {
        let proof_bytes = if backend == ZkProofType::Bulletproofs {
            bulletproof_bytes()
        } else {
            vec![1, 2, 3]
        };

        ZkCredentialProof {
            issuer: Did::new("key", "issuer"),
            holder: Did::new("key", "holder"),
            claim_type: "test".into(),
            proof: proof_bytes,
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

    #[test]
    fn bulletproofs_tampered_proof_fails() {
        let mut proof = dummy_proof(ZkProofType::Bulletproofs);
        // Flip a bit in the proof bytes to corrupt it while preserving length
        proof.proof[0] ^= 0x01;
        let verifier = BulletproofsVerifier;
        assert_eq!(verifier.verify(&proof), Err(ZkError::VerificationFailed));
    }

    #[test]
    fn groth16_prover_and_verifier_roundtrip() {
        use crate::credential::CredentialIssuer;
        use crate::generate_ed25519_keypair;

        let (sk, _pk) = generate_ed25519_keypair();
        let issuer_did = Did::new("key", "issuer");
        let holder_did = Did::new("key", "holder");
        let mut claims = std::collections::HashMap::new();
        claims.insert("birth_year".to_string(), "2000".to_string());

        let issuer = CredentialIssuer::new(issuer_did, sk).with_prover(Box::new(Groth16Prover));
        let (_cred, proof_opt) = issuer
            .issue(holder_did, claims, Some(dummy_cid("schema")), Some(&[]))
            .unwrap();
        let proof = proof_opt.expect("proof");
        let verifier = Groth16Verifier;
        assert!(verifier.verify(&proof).unwrap());
    }
}
