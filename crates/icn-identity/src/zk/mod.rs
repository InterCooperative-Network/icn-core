use core::convert::TryInto;
use icn_common::{Cid, ZkCredentialProof, ZkProofType};
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
            vk_cid: None,
            disclosed_fields: fields.iter().map(|f| f.to_string()).collect(),
            challenge: None,
            backend: ZkProofType::Groth16,
            verification_key: None,
            public_inputs: None,
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
        let (proof, commit) =
            RangeProof::prove_single(&bp_gens, &pc_gens, &mut transcript, 42u64, &blinding, 64)
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
            vk_cid: None,
            disclosed_fields: fields.iter().map(|f| f.to_string()).collect(),
            challenge: None,
            backend: ZkProofType::Bulletproofs,
            verification_key: None,
            public_inputs: None,
        })
    }
}

/// Prover implementation for Groth16 proofs using circuits from `icn-zk`.
#[derive(Debug)]
pub struct Groth16Prover {
    pk: ark_groth16::ProvingKey<ark_bn254::Bn254>,
}

impl Groth16Prover {
    /// Create a new prover wrapping the provided proving key.
    pub fn new(pk: ark_groth16::ProvingKey<ark_bn254::Bn254>) -> Self {
        Self { pk }
    }
}

impl Default for Groth16Prover {
    fn default() -> Self {
        use ark_std::rand::{rngs::StdRng, SeedableRng};
        use icn_zk::{setup, AgeOver18Circuit};

        // Deterministic setup for demo purposes
        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = StdRng::seed_from_u64(42);
        let pk = setup(circuit, &mut rng).expect("setup");

        Self { pk }
    }
}

impl ZkProver for Groth16Prover {
    fn prove(
        &self,
        credential: &crate::credential::Credential,
        fields: &[&str],
    ) -> Result<ZkCredentialProof, ZkError> {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::{rngs::StdRng, SeedableRng};
        use icn_zk::{prove, AgeOver18Circuit, MembershipCircuit, ReputationCircuit};

        // Determine which circuit to prove based on available claims
        let mut rng = StdRng::seed_from_u64(42);
        let (claim_type, proof_obj) = if let Some(val) = credential.claims.get("birth_year") {
            let birth_year = val
                .parse::<u64>()
                .map_err(|_| ZkError::VerificationFailed)?;
            (
                "age_over_18".to_string(),
                prove(
                    &self.pk,
                    AgeOver18Circuit {
                        birth_year,
                        current_year: 2020,
                    },
                    &mut rng,
                )
                .map_err(|_| ZkError::VerificationFailed)?,
            )
        } else if let Some(val) = credential.claims.get("is_member") {
            let is_member = matches!(val.as_str(), "true" | "1");
            (
                "membership".to_string(),
                prove(&self.pk, MembershipCircuit { is_member }, &mut rng)
                    .map_err(|_| ZkError::VerificationFailed)?,
            )
        } else if let Some(val) = credential.claims.get("reputation") {
            let reputation = val
                .parse::<u64>()
                .map_err(|_| ZkError::VerificationFailed)?;
            (
                "reputation".to_string(),
                prove(
                    &self.pk,
                    ReputationCircuit {
                        reputation,
                        threshold: 5,
                    },
                    &mut rng,
                )
                .map_err(|_| ZkError::VerificationFailed)?,
            )
        } else {
            return Err(ZkError::VerificationFailed);
        };

        let mut bytes = Vec::new();
        proof_obj
            .serialize_compressed(&mut bytes)
            .map_err(|_| ZkError::VerificationFailed)?;

        Ok(ZkCredentialProof {
            issuer: credential.issuer.clone(),
            holder: credential.holder.clone(),
            claim_type,
            proof: bytes,
            schema: credential
                .schema
                .clone()
                .unwrap_or_else(|| Cid::new_v1_sha256(0x55, b"groth16")),
            vk_cid: None,
            disclosed_fields: fields.iter().map(|f| f.to_string()).collect(),
            challenge: None,
            backend: ZkProofType::Groth16,
            verification_key: None,
            public_inputs: None,
        })
    }
}

/// Verifier implementation for Groth16 proofs using circuits from `icn-zk`.
#[derive(Debug)]
pub struct Groth16Verifier {
    vk_age_over_18: ark_groth16::PreparedVerifyingKey<ark_bn254::Bn254>,
}

impl Default for Groth16Verifier {
    fn default() -> Self {
        use ark_std::rand::{rngs::StdRng, SeedableRng};
        use icn_zk::{prepare_vk, setup, AgeOver18Circuit};

        // Deterministic setup for demo purposes
        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = StdRng::seed_from_u64(42);
        let pk = setup(circuit, &mut rng).expect("setup");
        let vk_age_over_18 = prepare_vk(&pk);

        Self { vk_age_over_18 }
    }
}

impl ZkVerifier for Groth16Verifier {
    fn verify(&self, proof: &ZkCredentialProof) -> Result<bool, ZkError> {
        use ark_bn254::Fr;
        use ark_groth16::Proof;
        use ark_serialize::CanonicalDeserialize;

        if proof.backend != ZkProofType::Groth16 {
            return Err(ZkError::UnsupportedBackend(proof.backend.clone()));
        }
        if proof.proof.is_empty() {
            return Err(ZkError::InvalidProof);
        }

        let groth_proof = Proof::<ark_bn254::Bn254>::deserialize_compressed(proof.proof.as_slice())
            .map_err(|_| ZkError::InvalidProof)?;

        let inputs = match proof.claim_type.as_str() {
            "age_over_18" => vec![Fr::from(2020u64)],
            _ => return Err(ZkError::VerificationFailed),
        };

        icn_zk::verify(&self.vk_age_over_18, &groth_proof, &inputs)
            .map_err(|_| ZkError::VerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::{Cid, Did};
    use std::collections::HashMap;

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
        let (proof, commit) =
            RangeProof::prove_single(&bp_gens, &pc_gens, &mut transcript, 42u64, &blinding, 64)
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
            vk_cid: None,
            disclosed_fields: Vec::new(),
            challenge: None,
            backend,
            verification_key: None,
            public_inputs: None,
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
    fn groth16_verifier_ok() {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::{rngs::StdRng, SeedableRng};
        use icn_zk::{prove, setup, AgeOver18Circuit};

        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = StdRng::seed_from_u64(42);
        let pk = setup(circuit.clone(), &mut rng).unwrap();
        let proof_obj = prove(&pk, circuit, &mut rng).unwrap();
        let mut bytes = Vec::new();
        proof_obj.serialize_compressed(&mut bytes).unwrap();

        let proof = ZkCredentialProof {
            issuer: Did::new("key", "issuer"),
            holder: Did::new("key", "holder"),
            claim_type: "age_over_18".into(),
            proof: bytes,
            schema: dummy_cid("schema"),
            vk_cid: None,
            disclosed_fields: Vec::new(),
            challenge: None,
            backend: ZkProofType::Groth16,
            verification_key: None,
            public_inputs: None,
        };

        let verifier = Groth16Verifier::default();
        assert!(verifier.verify(&proof).unwrap());
    }

    #[test]
    fn groth16_proof_roundtrip() {
        use crate::credential::CredentialIssuer;
        use crate::generate_ed25519_keypair;

        let (sk, _) = generate_ed25519_keypair();
        let issuer_did = Did::new("key", "issuer");
        let holder_did = Did::new("key", "holder");
        let mut claims = HashMap::new();
        claims.insert("birth_year".to_string(), "2000".to_string());

        let issuer =
            CredentialIssuer::new(issuer_did, sk).with_prover(Box::new(Groth16Prover::default()));
        let (_, proof_opt) = issuer
            .issue(holder_did, claims, Some(dummy_cid("schema")), Some(&[]))
            .unwrap();
        let proof = proof_opt.expect("proof");

        let verifier = Groth16Verifier::default();
        assert!(verifier.verify(&proof).unwrap());
    }
}
