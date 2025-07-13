use std::collections::HashMap;

use ark_bn254::{Bn254, Fr};
use ark_groth16::{PreparedVerifyingKey, Proof, ProvingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use rand::{rngs::StdRng, SeedableRng};
use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_identity::{generate_ed25519_keypair, credential::CredentialIssuer};
use icn_identity::zk::{ZkError, ZkProver, ZkVerifier};
use icn_zk::{self, AgeOver18Circuit};

struct Groth16Prover {
    pk: ProvingKey<Bn254>,
    current_year: u64,
}

impl Groth16Prover {
    fn new(current_year: u64) -> Self {
        let circuit = AgeOver18Circuit { birth_year: 0, current_year };
        let mut rng = StdRng::seed_from_u64(42);
        let pk = icn_zk::setup(circuit, &mut rng).expect("setup");
        Self { pk, current_year }
    }

    fn verifier(&self) -> Groth16Verifier {
        Groth16Verifier {
            vk: icn_zk::prepare_vk(&self.pk),
            current_year: self.current_year,
        }
    }
}

impl ZkProver for Groth16Prover {
    fn prove(
        &self,
        credential: &icn_identity::credential::Credential,
        _fields: &[&str],
    ) -> Result<ZkCredentialProof, ZkError> {
        let birth_year = credential
            .claims
            .get("birth_year")
            .ok_or(ZkError::InvalidProof)?
            .parse::<u64>()
            .map_err(|_| ZkError::InvalidProof)?;
        let circuit = AgeOver18Circuit {
            birth_year,
            current_year: self.current_year,
        };
        let mut rng = StdRng::seed_from_u64(42);
        let proof = icn_zk::prove(&self.pk, circuit, &mut rng).map_err(|_| ZkError::VerificationFailed)?;
        let mut proof_bytes = Vec::new();
        proof.serialize_uncompressed(&mut proof_bytes).unwrap();
        Ok(ZkCredentialProof {
            issuer: credential.issuer.clone(),
            holder: credential.holder.clone(),
            claim_type: "age_over_18".into(),
            proof: proof_bytes,
            schema: credential.schema.clone().unwrap_or_else(|| Cid::new_v1_sha256(0x55, b"age")),
            disclosed_fields: Vec::new(),
            challenge: None,
            backend: ZkProofType::Groth16,
        })
    }
}

struct Groth16Verifier {
    vk: PreparedVerifyingKey<Bn254>,
    current_year: u64,
}

impl ZkVerifier for Groth16Verifier {
    fn verify(&self, proof: &ZkCredentialProof) -> Result<bool, ZkError> {
        let pf = Proof::<Bn254>::deserialize_uncompressed(&*proof.proof).map_err(|_| ZkError::InvalidProof)?;
        icn_zk::verify(&self.vk, &pf, &[Fr::from(self.current_year)])
            .map_err(|_| ZkError::VerificationFailed)
    }
}

#[test]
fn groth16_age_over_18_roundtrip() {
    let (sk, _) = generate_ed25519_keypair();
    let issuer = Did::new("key", "issuer");
    let holder = Did::new("key", "holder");
    let mut claims = HashMap::new();
    claims.insert("birth_year".to_string(), "2000".to_string());

    let prover = Groth16Prover::new(2020);
    let verifier = prover.verifier();
    let issuer = CredentialIssuer::new(issuer, sk).with_prover(Box::new(prover));
    let (_cred, proof_opt) = issuer
        .issue(holder, claims, None, Some(&[]))
        .expect("issuance");
    let proof = proof_opt.expect("proof");
    assert!(verifier.verify(&proof).unwrap());
}
