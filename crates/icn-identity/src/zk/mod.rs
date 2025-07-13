//! Zero-knowledge proof helpers used by the identity subsystem.
//!
//! The [`Groth16Verifier`] type allows validating Groth16 proofs against a
//! prepared verifying key and known public inputs.

use core::convert::TryInto;
use icn_common::{Cid, ZkCredentialProof, ZkProofType, ZkRevocationProof};
use serde_json;
use serde_json::Value;
use std::any::Any;
use thiserror::Error;

pub mod key_manager;
pub mod vk_cache;
pub use key_manager::Groth16KeyManager;

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

/// Trait for verifying revocation proofs that prove a credential has not been revoked.
pub trait ZkRevocationVerifier: Send + Sync {
    /// Verify the supplied [`ZkRevocationProof`].
    fn verify_revocation(&self, proof: &ZkRevocationProof) -> Result<bool, ZkError>;
}

impl<T: ZkVerifier + ?Sized> ZkRevocationVerifier for T {
    fn verify_revocation(&self, proof: &ZkRevocationProof) -> Result<bool, ZkError> {
        use icn_common::{Cid, ZkCredentialProof};

        let cred_like = ZkCredentialProof {
            issuer: proof.issuer.clone(),
            holder: proof.subject.clone(),
            claim_type: "revocation".into(),
            proof: proof.proof.clone(),
            schema: Cid::new_v1_sha256(0x55, b"revocation"),
            vk_cid: None,
            disclosed_fields: Vec::new(),
            challenge: None,
            backend: proof.backend.clone(),
            verification_key: proof.verification_key.clone(),
            public_inputs: proof.public_inputs.clone(),
        };

        self.verify(&cred_like)
    }
}

/// Trait for generating zero-knowledge proofs for credentials.
pub trait ZkProver: Send + Sync {
    fn prove(
        &self,
        credential: &crate::credential::Credential,
        fields: &[&str],
    ) -> Result<ZkCredentialProof, ZkError>;

    fn as_any(&self) -> &dyn Any;
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Default)]
pub struct BulletproofsProver;

/// Supported circuits for Groth16 credential proofs.
#[derive(Debug, Clone)]
pub enum Groth16Circuit {
    AgeOver18 { current_year: u64 },
    Membership,
    Reputation { threshold: u64 },
}

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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Prover implementation for Groth16 proofs using circuits from `icn-zk`.
#[derive(Debug)]
pub struct Groth16Prover {
    km: Groth16KeyManager,
}

impl Groth16Prover {
    /// Create a new prover wrapping the provided proving key.
    pub fn new(km: Groth16KeyManager) -> Self {
        Self { km }
    }

    /// Generate a proof for the provided circuit using credential claims.
    pub fn prove_with_circuit(
        &self,
        credential: &crate::credential::Credential,
        fields: &[&str],
        circuit: Groth16Circuit,
    ) -> Result<ZkCredentialProof, ZkError> {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::{rngs::StdRng, SeedableRng};
        use icn_zk::{prove, AgeOver18Circuit, MembershipCircuit, ReputationCircuit};

        let mut rng = StdRng::seed_from_u64(42);

        let (claim_type, proof_obj, vk_bytes) = match circuit {
            Groth16Circuit::AgeOver18 { current_year } => {
                let birth_year = credential
                    .claims
                    .get("birth_year")
                    .ok_or(ZkError::VerificationFailed)?
                    .parse::<u64>()
                    .map_err(|_| ZkError::VerificationFailed)?;
                let circuit = AgeOver18Circuit {
                    birth_year,
                    current_year,
                };
                let proof = prove(self.km.proving_key(), circuit, &mut rng)
                    .map_err(|_| ZkError::VerificationFailed)?;
                let mut vk_bytes = Vec::new();
                self.km
                    .proving_key()
                    .vk
                    .serialize_compressed(&mut vk_bytes)
                    .map_err(|_| ZkError::VerificationFailed)?;
                ("age_over_18".to_string(), proof, vk_bytes)
            }
            Groth16Circuit::Membership => {
                let is_member = credential
                    .claims
                    .get("is_member")
                    .ok_or(ZkError::VerificationFailed)?
                    .as_str()
                    .eq_ignore_ascii_case("true");
                let circuit = MembershipCircuit { is_member };
                let proof = prove(self.km.proving_key(), circuit, &mut rng)
                    .map_err(|_| ZkError::VerificationFailed)?;
                let mut vk_bytes = Vec::new();
                self.km
                    .proving_key()
                    .vk
                    .serialize_compressed(&mut vk_bytes)
                    .map_err(|_| ZkError::VerificationFailed)?;
                ("membership".to_string(), proof, vk_bytes)
            }
            Groth16Circuit::Reputation { threshold } => {
                let reputation = credential
                    .claims
                    .get("reputation")
                    .ok_or(ZkError::VerificationFailed)?
                    .parse::<u64>()
                    .map_err(|_| ZkError::VerificationFailed)?;
                let circuit = ReputationCircuit {
                    reputation,
                    threshold,
                };
                let proof = prove(self.km.proving_key(), circuit, &mut rng)
                    .map_err(|_| ZkError::VerificationFailed)?;
                let mut vk_bytes = Vec::new();
                self.km
                    .proving_key()
                    .vk
                    .serialize_compressed(&mut vk_bytes)
                    .map_err(|_| ZkError::VerificationFailed)?;
                ("reputation".to_string(), proof, vk_bytes)
            }
        };

        let mut proof_bytes = Vec::new();
        proof_obj
            .serialize_compressed(&mut proof_bytes)
            .map_err(|_| ZkError::VerificationFailed)?;

        let vk_cid = Cid::new_v1_sha256(0x55, &vk_bytes);

        Ok(ZkCredentialProof {
            issuer: credential.issuer.clone(),
            holder: credential.holder.clone(),
            claim_type,
            proof: proof_bytes,
            schema: credential
                .schema
                .clone()
                .unwrap_or_else(|| Cid::new_v1_sha256(0x55, b"groth16")),
            vk_cid: Some(vk_cid),
            disclosed_fields: fields.iter().map(|f| f.to_string()).collect(),
            challenge: None,
            backend: ZkProofType::Groth16,
            verification_key: None,
            public_inputs: None,
        })
    }
}

impl Default for Groth16Prover {
    fn default() -> Self {
        use crate::generate_ed25519_keypair;

        let (sk, _) = generate_ed25519_keypair();
        let km = Groth16KeyManager::new(&sk).expect("key manager setup");
        Self { km }
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
                    self.km.proving_key(),
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
                prove(
                    self.km.proving_key(),
                    MembershipCircuit { is_member },
                    &mut rng,
                )
                .map_err(|_| ZkError::VerificationFailed)?,
            )
        } else if let Some(val) = credential.claims.get("reputation") {
            let reputation = val
                .parse::<u64>()
                .map_err(|_| ZkError::VerificationFailed)?;
            (
                "reputation".to_string(),
                prove(
                    self.km.proving_key(),
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Verifier implementation for Groth16 proofs.
///
/// The verifier stores a prepared verifying key and the public inputs expected
/// by the circuit. Proofs are checked using [`ark_groth16::Groth16::verify_proof`].
#[derive(Debug)]
pub struct Groth16Verifier {
    vk: ark_groth16::PreparedVerifyingKey<ark_bn254::Bn254>,
    public_inputs: Vec<ark_bn254::Fr>,
}

impl Groth16Verifier {
    /// Create a new verifier from the prepared verifying key and public inputs.
    pub fn new(
        vk: ark_groth16::PreparedVerifyingKey<ark_bn254::Bn254>,
        public_inputs: Vec<ark_bn254::Fr>,
    ) -> Self {
        Self { vk, public_inputs }
    }

    /// Verify the supplied [`ZkCredentialProof`] using a cached prepared key if
    /// provided. Public inputs embedded in the proof take precedence over the
    /// verifier's defaults.
    pub fn verify_proof(&self, proof: &ZkCredentialProof) -> Result<bool, ZkError> {
        use ark_groth16::{Groth16, Proof, VerifyingKey};
        use ark_serialize::CanonicalDeserialize;
        use ark_snark::SNARK;

        if proof.backend != ZkProofType::Groth16 {
            return Err(ZkError::UnsupportedBackend(proof.backend.clone()));
        }
        if proof.proof.is_empty() {
            return Err(ZkError::InvalidProof);
        }

        // Deserialize the proof bytes
        let groth_proof = Proof::<ark_bn254::Bn254>::deserialize_compressed(proof.proof.as_slice())
            .map_err(|_| ZkError::InvalidProof)?;

        // Determine public inputs
        let inputs = match &proof.public_inputs {
            Some(v) => parse_public_inputs(v)?,
            None => self.public_inputs.clone(),
        };

        // Fetch or prepare verifying key using global cache
        let pvk = if let Some(vk_bytes) = &proof.verification_key {
            vk_cache::PreparedVkCache::get_or_insert(vk_bytes)?
        } else {
            self.vk.clone()
        };

        Groth16::<ark_bn254::Bn254>::verify_proof(&pvk, &groth_proof, &inputs)
            .map_err(|_| ZkError::VerificationFailed)
    }
}

fn parse_public_inputs(v: &Value) -> Result<Vec<ark_bn254::Fr>, ZkError> {
    let arr = v.as_array().ok_or(ZkError::InvalidProof)?;
    arr.iter()
        .map(|val| {
            val.as_u64()
                .map(ark_bn254::Fr::from)
                .ok_or(ZkError::InvalidProof)
        })
        .collect()
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
        let vk = prepare_vk(&pk);
        let inputs = vec![ark_bn254::Fr::from(2020u64)];

        Self::new(vk, inputs)
    }
}

impl ZkVerifier for Groth16Verifier {
    fn verify(&self, proof: &ZkCredentialProof) -> Result<bool, ZkError> {
        self.verify_proof(proof)
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
        use icn_zk::{prepare_vk, prove, setup, AgeOver18Circuit};

        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = StdRng::seed_from_u64(42);
        let pk = setup(circuit.clone(), &mut rng).unwrap();
        let pvk = prepare_vk(&pk);
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

        let verifier = Groth16Verifier::new(pvk, vec![ark_bn254::Fr::from(2020u64)]);
        assert!(verifier.verify(&proof).unwrap());
    }

    #[test]
    fn groth16_backend_mismatch() {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::{rngs::StdRng, SeedableRng};
        use icn_zk::{prepare_vk, prove, setup, AgeOver18Circuit};

        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = StdRng::seed_from_u64(42);
        let pk = setup(circuit.clone(), &mut rng).unwrap();
        let pvk = prepare_vk(&pk);
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
            backend: ZkProofType::Bulletproofs,
            verification_key: None,
            public_inputs: None,
        };

        let verifier = Groth16Verifier::new(pvk, vec![ark_bn254::Fr::from(2020u64)]);
        assert!(matches!(
            verifier.verify(&proof),
            Err(ZkError::UnsupportedBackend(_))
        ));
    }

    #[test]
    fn groth16_malformed_proof() {
        use ark_std::rand::{rngs::StdRng, SeedableRng};
        use icn_zk::{prepare_vk, setup, AgeOver18Circuit};

        let mut rng = StdRng::seed_from_u64(42);
        let pk = setup(
            AgeOver18Circuit {
                birth_year: 2000,
                current_year: 2020,
            },
            &mut rng,
        )
        .unwrap();
        let pvk = prepare_vk(&pk);

        let proof = ZkCredentialProof {
            issuer: Did::new("key", "issuer"),
            holder: Did::new("key", "holder"),
            claim_type: "age_over_18".into(),
            proof: vec![0u8; 10],
            schema: dummy_cid("schema"),
            vk_cid: None,
            disclosed_fields: Vec::new(),
            challenge: None,
            backend: ZkProofType::Groth16,
            verification_key: None,
            public_inputs: None,
        };

        let verifier = Groth16Verifier::new(pvk, vec![ark_bn254::Fr::from(2020u64)]);
        assert!(matches!(
            verifier.verify(&proof),
            Err(ZkError::InvalidProof)
        ));
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

        let km = Groth16KeyManager::new(&sk).unwrap();
        let verifier_vk = icn_zk::prepare_vk(km.proving_key());
        let issuer = CredentialIssuer::new(issuer_did, sk)
            .with_prover(Box::new(Groth16Prover::new(km.clone())));
        let (_, proof_opt) = issuer
            .issue(
                holder_did,
                claims,
                Some(dummy_cid("schema")),
                Some(&[]),
                Some(Groth16Circuit::AgeOver18 { current_year: 2020 }),
            )
            .unwrap();
        let proof = proof_opt.expect("proof");

        let verifier = Groth16Verifier::new(verifier_vk, vec![ark_bn254::Fr::from(2020u64)]);
        assert!(verifier.verify(&proof).unwrap());
    }

    #[test]
    fn verify_proof_cache_hit() {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::{rngs::StdRng, SeedableRng};
        use icn_zk::{prepare_vk, prove, setup, AgeOver18Circuit};

        let mut rng = StdRng::seed_from_u64(42);
        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let pk = setup(circuit.clone(), &mut rng).unwrap();
        let proof_obj = prove(&pk, circuit, &mut rng).unwrap();
        let mut proof_bytes = Vec::new();
        proof_obj.serialize_compressed(&mut proof_bytes).unwrap();
        let mut vk_bytes = Vec::new();
        pk.vk.serialize_compressed(&mut vk_bytes).unwrap();

        let verifier = Groth16Verifier::new(prepare_vk(&pk), vec![ark_bn254::Fr::from(2020u64)]);
        let proof = ZkCredentialProof {
            issuer: Did::new("key", "issuer"),
            holder: Did::new("key", "holder"),
            claim_type: "age_over_18".into(),
            proof: proof_bytes.clone(),
            schema: dummy_cid("schema"),
            vk_cid: None,
            disclosed_fields: Vec::new(),
            challenge: None,
            backend: ZkProofType::Groth16,
            verification_key: Some(vk_bytes.clone()),
            public_inputs: Some(serde_json::json!([2020])),
        };

        assert!(verifier.verify_proof(&proof).unwrap());
        assert!(verifier.verify_proof(&proof).unwrap());
    }

    #[test]
    fn verify_proof_invalid_key_signature() {
        use crate::generate_ed25519_keypair;

        let (sk, pk1) = generate_ed25519_keypair();
        let km = Groth16KeyManager::new(&sk).unwrap();
        assert!(km.verify_key_signature(&pk1).unwrap());
        // Verification with a different key should fail
        let (_, pk2) = generate_ed25519_keypair();
        assert!(!km.verify_key_signature(&pk2).unwrap());
    }

    #[test]
    fn verify_proof_with_inputs() {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::{rngs::StdRng, SeedableRng};
        use icn_zk::{prepare_vk, prove, setup, AgeOver18Circuit};

        let mut rng = StdRng::seed_from_u64(42);
        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2021,
        };
        let pk = setup(circuit.clone(), &mut rng).unwrap();
        let proof_obj = prove(&pk, circuit, &mut rng).unwrap();
        let mut proof_bytes = Vec::new();
        proof_obj.serialize_compressed(&mut proof_bytes).unwrap();
        let mut vk_bytes = Vec::new();
        pk.vk.serialize_compressed(&mut vk_bytes).unwrap();

        // Verifier has incorrect default inputs
        let verifier = Groth16Verifier::new(prepare_vk(&pk), vec![ark_bn254::Fr::from(9999u64)]);
        let proof = ZkCredentialProof {
            issuer: Did::new("key", "issuer"),
            holder: Did::new("key", "holder"),
            claim_type: "age_over_18".into(),
            proof: proof_bytes,
            schema: dummy_cid("schema"),
            vk_cid: None,
            disclosed_fields: Vec::new(),
            challenge: None,
            backend: ZkProofType::Groth16,
            verification_key: Some(vk_bytes),
            public_inputs: Some(serde_json::json!([2021])),
        };

        assert!(verifier.verify_proof(&proof).unwrap());
    }
}
