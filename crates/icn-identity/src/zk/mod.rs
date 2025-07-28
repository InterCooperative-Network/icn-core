//! Zero-knowledge proof helpers used by the identity subsystem.
//!
//! The [`Groth16Verifier`] type allows validating Groth16 proofs against a
//! prepared verifying key and known public inputs.

use core::convert::TryInto;
use directories_next as dirs_next;
use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType, ZkRevocationProof};
use serde_json;
use serde_json::Value;
use std::any::Any;
use std::fs;
use thiserror::Error;

pub mod key_manager;
pub use key_manager::Groth16KeyManager;
pub mod proof_cache;
pub mod vk_cache;

use crate::{
    verify_signature, verifying_key_from_did_key, verifying_key_from_did_peer, SIGNATURE_LENGTH,
};

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
    /// Caller reputation below required threshold.
    #[error("insufficient reputation")]
    InsufficientReputation,
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

fn verify_verification_key(vk_bytes: &[u8], issuer: &Did) -> Result<(), ZkError> {
    use ed25519_dalek::Signature;

    let signer_pk = match issuer.method.as_str() {
        "key" => verifying_key_from_did_key(issuer),
        "peer" => verifying_key_from_did_peer(issuer),
        _ => Err(icn_common::CommonError::IdentityError(
            "unsupported did".into(),
        )),
    }
    .map_err(|_| ZkError::VerificationFailed)?;

    let dirs = dirs_next::BaseDirs::new().ok_or(ZkError::VerificationFailed)?;
    let dir = dirs.home_dir().join(".icn/zk");
    let sig_bytes =
        fs::read(dir.join("verifying_key.sig")).map_err(|_| ZkError::VerificationFailed)?;
    if sig_bytes.len() != SIGNATURE_LENGTH {
        return Err(ZkError::InvalidProof);
    }
    let sig_array: [u8; SIGNATURE_LENGTH] =
        sig_bytes.try_into().map_err(|_| ZkError::InvalidProof)?;
    let sig = Signature::from_bytes(&sig_array);
    if verify_signature(&signer_pk, vk_bytes, &sig) {
        Ok(())
    } else {
        Err(ZkError::VerificationFailed)
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

        // Reputation checks disabled in this build
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

        let result = range_proof
            .verify_single(&bp_gens, &pc_gens, &mut transcript, &commitment, 64)
            .map(|_| true)
            .map_err(|_| ZkError::VerificationFailed);
        if result.is_ok() {
            crate::metrics::PROOFS_VERIFIED.inc();
        } else {
            crate::metrics::PROOF_VERIFICATION_FAILURES.inc();
        }
        result
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
        crate::metrics::PROOFS_VERIFIED.inc();
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
    reputation_store: std::sync::Arc<dyn icn_reputation::ReputationStore>,
    thresholds: icn_zk::ReputationThresholds,
}

impl Groth16Prover {
    /// Create a new prover wrapping the provided proving key and reputation store.
    pub fn new(
        km: Groth16KeyManager,
        reputation_store: std::sync::Arc<dyn icn_reputation::ReputationStore>,
        thresholds: icn_zk::ReputationThresholds,
    ) -> Self {
        Self {
            km,
            reputation_store,
            thresholds,
        }
    }

    /// Generate a proof for the provided circuit using credential claims.
    pub fn prove_with_circuit(
        &self,
        credential: &crate::credential::Credential,
        fields: &[&str],
        circuit: Groth16Circuit,
    ) -> Result<ZkCredentialProof, ZkError> {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::rngs::OsRng;
        use icn_zk::{prove, AgeOver18Circuit, MembershipCircuit, ReputationCircuit};

        let mut rng = OsRng;

        let caller_rep = self.reputation_store.get_reputation(&credential.issuer);
        let required = match circuit {
            Groth16Circuit::AgeOver18 { .. } => self.thresholds.age_over_18,
            Groth16Circuit::Membership => self.thresholds.membership,
            Groth16Circuit::Reputation { .. } => self.thresholds.reputation,
        };
        if caller_rep < required {
            return Err(ZkError::InsufficientReputation);
        }

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

        use crate::zk::key_manager::Groth16KeySource;
        use icn_zk::AgeOver18Circuit;

        let (sk, _) = generate_ed25519_keypair();
        let km = Groth16KeyManager::new(
            "age_over_18",
            Groth16KeySource::Circuit(AgeOver18Circuit {
                birth_year: 2000,
                current_year: 2020,
            }),
            &sk,
        )
        .expect("key manager setup");
        Self {
            km,
            reputation_store: std::sync::Arc::new(icn_reputation::InMemoryReputationStore::new()),
            thresholds: icn_zk::ReputationThresholds::default(),
        }
    }
}

impl ZkProver for Groth16Prover {
    fn prove(
        &self,
        credential: &crate::credential::Credential,
        fields: &[&str],
    ) -> Result<ZkCredentialProof, ZkError> {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::rngs::OsRng;
        use icn_zk::{prove, AgeOver18Circuit, MembershipCircuit, ReputationCircuit};

        // Determine which circuit to prove based on available claims
        let mut rng = OsRng;

        let rep = self.reputation_store.get_reputation(&credential.issuer);
        let (claim_type, proof_obj) = if let Some(val) = credential.claims.get("birth_year") {
            let birth_year = val
                .parse::<u64>()
                .map_err(|_| ZkError::VerificationFailed)?;
            if rep < self.thresholds.age_over_18 {
                return Err(ZkError::InsufficientReputation);
            }
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
            if rep < self.thresholds.membership {
                return Err(ZkError::InsufficientReputation);
            }
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
            if rep < self.thresholds.reputation {
                return Err(ZkError::InsufficientReputation);
            }
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
#[allow(dead_code)]
pub struct Groth16Verifier {
    vk: ark_groth16::PreparedVerifyingKey<ark_bn254::Bn254>,
    public_inputs: Vec<ark_bn254::Fr>,
    #[allow(dead_code)]
    reputation_store: std::sync::Arc<dyn icn_reputation::ReputationStore>,
    #[allow(dead_code)]
    thresholds: icn_zk::ReputationThresholds,
}

impl Groth16Verifier {
    /// Create a new verifier from the prepared verifying key and public inputs.
    pub fn new(
        vk: ark_groth16::PreparedVerifyingKey<ark_bn254::Bn254>,
        public_inputs: Vec<ark_bn254::Fr>,
        reputation_store: std::sync::Arc<dyn icn_reputation::ReputationStore>,
        thresholds: icn_zk::ReputationThresholds,
    ) -> Self {
        Self {
            vk,
            public_inputs,
            reputation_store,
            thresholds,
        }
    }

    /// Verify the supplied [`ZkCredentialProof`] using a cached prepared key if
    /// provided. Public inputs embedded in the proof take precedence over the
    /// verifier's defaults.
    pub fn verify_proof(&self, proof: &ZkCredentialProof) -> Result<bool, ZkError> {
        use ark_groth16::{Groth16, Proof};
        use ark_serialize::CanonicalDeserialize;

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

        // Fetch or prepare verifying key
        let pvk = if let Some(vk_bytes) = &proof.verification_key {
            verify_verification_key(vk_bytes, &proof.issuer)?;
            vk_cache::PreparedVkCache::get_or_insert(vk_bytes)?
        } else {
            self.vk.clone()
        };

        let pvk_clone = pvk.clone();
        let result = proof_cache::ProofCache::get_or_insert(&proof.proof, &pvk, &inputs, || {
            Groth16::<ark_bn254::Bn254>::verify_proof(&pvk_clone, &groth_proof, &inputs)
                .map_err(|_| ZkError::VerificationFailed)
        });
        if result.as_ref().map(|b| *b).unwrap_or(false) {
            crate::metrics::PROOFS_VERIFIED.inc();
        } else if result.is_err() || !result.as_ref().unwrap_or(&false) {
            crate::metrics::PROOF_VERIFICATION_FAILURES.inc();
        }
        result
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
        use ark_std::rand::rngs::OsRng;
        use icn_zk::{prepare_vk, setup, AgeOver18Circuit};

        // Example setup for demo purposes
        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = OsRng;
        let pk = setup(circuit, &mut rng).expect("setup");
        let vk = prepare_vk(&pk);
        let inputs = vec![ark_bn254::Fr::from(2020u64)];
        Self::new(
            vk,
            inputs,
            std::sync::Arc::new(icn_reputation::InMemoryReputationStore::new()),
            icn_zk::ReputationThresholds::default(),
        )
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
    use crate::zk::key_manager::Groth16KeySource;
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
        use ark_std::rand::rngs::OsRng;
        use icn_zk::{prepare_vk, prove, setup, AgeOver18Circuit};

        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = OsRng;
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

        let verifier = Groth16Verifier::new(
            pvk,
            vec![ark_bn254::Fr::from(2020u64)],
            std::sync::Arc::new(icn_reputation::InMemoryReputationStore::new()),
            icn_zk::ReputationThresholds::default(),
        );
        assert!(verifier.verify(&proof).unwrap());
    }

    #[test]
    fn groth16_backend_mismatch() {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::rngs::OsRng;
        use icn_zk::{prepare_vk, prove, setup, AgeOver18Circuit};

        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = OsRng;
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

        let verifier = Groth16Verifier::new(
            pvk,
            vec![ark_bn254::Fr::from(2020u64)],
            std::sync::Arc::new(icn_reputation::InMemoryReputationStore::new()),
            icn_zk::ReputationThresholds::default(),
        );
        assert!(matches!(
            verifier.verify(&proof),
            Err(ZkError::UnsupportedBackend(_))
        ));
    }

    #[test]
    fn groth16_malformed_proof() {
        use ark_std::rand::rngs::OsRng;
        use icn_zk::{prepare_vk, setup, AgeOver18Circuit};

        let mut rng = OsRng;
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

        let verifier = Groth16Verifier::new(
            pvk,
            vec![ark_bn254::Fr::from(2020u64)],
            std::sync::Arc::new(icn_reputation::InMemoryReputationStore::new()),
            icn_zk::ReputationThresholds::default(),
        );
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

        use icn_zk::AgeOver18Circuit;

        let km = Groth16KeyManager::new(
            "age_over_18",
            Groth16KeySource::Circuit(AgeOver18Circuit {
                birth_year: 2000,
                current_year: 2020,
            }),
            &sk,
        )
        .unwrap();
        let verifier_vk = icn_zk::prepare_vk(km.proving_key());
        let reputation_store = std::sync::Arc::new(icn_reputation::InMemoryReputationStore::new());
        reputation_store.set_score(issuer_did.clone(), 100);
        let issuer =
            CredentialIssuer::new(issuer_did, sk).with_prover(Box::new(Groth16Prover::new(
                km.clone(),
                reputation_store.clone(),
                icn_zk::ReputationThresholds::default(),
            )));
        let (_, proof_opt) = issuer
            .issue(
                holder_did,
                claims,
                Some(dummy_cid("schema")),
                Some(&[]),
                Some(Groth16Circuit::AgeOver18 { current_year: 2020 }),
                None,
            )
            .unwrap();
        let proof = proof_opt.expect("proof");

        let verifier = Groth16Verifier::new(
            verifier_vk,
            vec![ark_bn254::Fr::from(2020u64)],
            reputation_store,
            icn_zk::ReputationThresholds::default(),
        );
        assert!(verifier.verify(&proof).unwrap());
    }

    #[test]
    fn verify_proof_cache_hit() {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::rngs::OsRng;
        use icn_zk::{prepare_vk, prove, setup, AgeOver18Circuit};

        let mut rng = OsRng;
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

        let (sk, vk) = crate::generate_ed25519_keypair();
        let issuer_did_string = crate::did_key_from_verifying_key(&vk);
        // Parse "did:key:encoded_part" into Did::new("key", "encoded_part")
        let parts: Vec<&str> = issuer_did_string.splitn(3, ':').collect();
        let issuer_did = Did::new(parts[1], parts[2]);
        let sig = crate::sign_message(&sk, &vk_bytes);
        let dirs = dirs_next::BaseDirs::new().unwrap();
        let dir = dirs.home_dir().join(".icn/zk");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("verifying_key.sig"), sig.to_bytes()).unwrap();

        let verifier = Groth16Verifier::new(
            prepare_vk(&pk),
            vec![ark_bn254::Fr::from(2020u64)],
            std::sync::Arc::new(icn_reputation::InMemoryReputationStore::new()),
            icn_zk::ReputationThresholds::default(),
        );
        let proof = ZkCredentialProof {
            issuer: issuer_did,
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
        let cache_len = vk_cache::PreparedVkCache::len();
        assert_eq!(cache_len, 1);
        assert!(verifier.verify_proof(&proof).unwrap());
        let cache_len2 = vk_cache::PreparedVkCache::len();
        assert_eq!(cache_len2, 1);
    }

    #[test]
    fn verify_proof_invalid_key_signature() {
        use crate::generate_ed25519_keypair;

        use icn_zk::AgeOver18Circuit;

        let (sk, pk1) = generate_ed25519_keypair();
        let km = Groth16KeyManager::new(
            "age_over_18",
            Groth16KeySource::Circuit(AgeOver18Circuit {
                birth_year: 2000,
                current_year: 2020,
            }),
            &sk,
        )
        .unwrap();
        assert!(km.verify_key_signature(&pk1).unwrap());
        // Verification with a different key should fail
        let (_, pk2) = generate_ed25519_keypair();
        assert!(!km.verify_key_signature(&pk2).unwrap());
    }

    #[test]
    fn verify_proof_with_inputs() {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::rngs::OsRng;
        use icn_zk::{prepare_vk, prove, setup, AgeOver18Circuit};

        let mut rng = OsRng;
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

        let (sk, vk) = crate::generate_ed25519_keypair();
        let issuer_did_string = crate::did_key_from_verifying_key(&vk);
        // Parse "did:key:encoded_part" into Did::new("key", "encoded_part")
        let parts: Vec<&str> = issuer_did_string.splitn(3, ':').collect();
        let issuer_did = Did::new(parts[1], parts[2]);
        let sig = crate::sign_message(&sk, &vk_bytes);
        let dirs = dirs_next::BaseDirs::new().unwrap();
        let dir = dirs.home_dir().join(".icn/zk");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("verifying_key.sig"), sig.to_bytes()).unwrap();

        // Verifier has incorrect default inputs
        let verifier = Groth16Verifier::new(
            prepare_vk(&pk),
            vec![ark_bn254::Fr::from(9999u64)],
            std::sync::Arc::new(icn_reputation::InMemoryReputationStore::new()),
            icn_zk::ReputationThresholds::default(),
        );
        let proof = ZkCredentialProof {
            issuer: issuer_did,
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

    #[test]
    fn proof_result_cache_hit() {
        use ark_serialize::CanonicalSerialize;
        use ark_std::rand::rngs::OsRng;
        use icn_zk::{prepare_vk, prove, setup, AgeOver18Circuit};

        let mut rng = OsRng;
        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let pk = setup(circuit.clone(), &mut rng).unwrap();
        let pvk = prepare_vk(&pk);
        let proof_obj = prove(&pk, circuit, &mut rng).unwrap();
        let mut proof_bytes = Vec::new();
        proof_obj.serialize_compressed(&mut proof_bytes).unwrap();
        let mut vk_bytes = Vec::new();
        pk.vk.serialize_compressed(&mut vk_bytes).unwrap();

        let (sk, vk) = crate::generate_ed25519_keypair();
        let issuer_did_string = crate::did_key_from_verifying_key(&vk);
        // Parse "did:key:encoded_part" into Did::new("key", "encoded_part")
        let parts: Vec<&str> = issuer_did_string.splitn(3, ':').collect();
        let issuer_did = Did::new(parts[1], parts[2]);
        let sig = crate::sign_message(&sk, &vk_bytes);
        let dirs = dirs_next::BaseDirs::new().unwrap();
        let dir = dirs.home_dir().join(".icn/zk");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("verifying_key.sig"), sig.to_bytes()).unwrap();

        let verifier = Groth16Verifier::new(
            pvk,
            vec![ark_bn254::Fr::from(2020u64)],
            std::sync::Arc::new(icn_reputation::InMemoryReputationStore::new()),
            icn_zk::ReputationThresholds::default(),
        );
        let proof = ZkCredentialProof {
            issuer: issuer_did,
            holder: Did::new("key", "holder"),
            claim_type: "age_over_18".into(),
            proof: proof_bytes.clone(),
            schema: dummy_cid("schema"),
            vk_cid: None,
            disclosed_fields: Vec::new(),
            challenge: None,
            backend: ZkProofType::Groth16,
            verification_key: Some(vk_bytes),
            public_inputs: Some(serde_json::json!([2020])),
        };

        assert!(verifier.verify_proof(&proof).unwrap());
        assert_eq!(proof_cache::ProofCache::len(), 1);
        assert!(verifier.verify_proof(&proof).unwrap());
        assert_eq!(proof_cache::ProofCache::len(), 1);
    }
}
