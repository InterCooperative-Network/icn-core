use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_identity::{
    credential::CredentialIssuer,
    generate_ed25519_keypair,
    zk::{Groth16Verifier, ZkError, ZkProver},
    ZkVerifier,
};
use icn_identity::credential::Credential;

use ark_serialize::CanonicalSerialize;
use ark_std::rand::{rngs::StdRng, SeedableRng};
use icn_zk::{prove, setup, AgeOver18Circuit};

struct Groth16Prover {
    pk: ark_groth16::ProvingKey<ark_bn254::Bn254>,
}

impl Default for Groth16Prover {
    fn default() -> Self {
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
        credential: &Credential,
        fields: &[&str],
    ) -> Result<ZkCredentialProof, ZkError> {
        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = StdRng::seed_from_u64(42);
        let proof_obj =
            prove(&self.pk, circuit, &mut rng).map_err(|_| ZkError::VerificationFailed)?;
        let mut bytes = Vec::new();
        proof_obj.serialize_compressed(&mut bytes).unwrap();
        Ok(ZkCredentialProof {
            issuer: credential.issuer.clone(),
            holder: credential.holder.clone(),
            claim_type: "age_over_18".into(),
            proof: bytes,
            schema: credential
                .schema
                .clone()
                .unwrap_or_else(|| Cid::new_v1_sha256(0x55, b"age")),
            disclosed_fields: fields.iter().map(|f| f.to_string()).collect(),
            challenge: None,
            backend: ZkProofType::Groth16,
        })
    }
}

#[test]
fn issue_and_verify_groth16_proof() {
    let (sk, pk) = generate_ed25519_keypair();
    let issuer_did = Did::new("key", "issuer");
    let holder_did = Did::new("key", "holder");
    let issuer = CredentialIssuer::new(issuer_did.clone(), sk)
        .with_prover(Box::new(Groth16Prover::default()));

    let mut claims = std::collections::HashMap::new();
    claims.insert("birth_year".to_string(), "2000".to_string());

    let (cred, proof_opt) = issuer
        .issue(
            holder_did,
            claims,
            Some(Cid::new_v1_sha256(0x55, b"schema")),
            Some(&[]),
        )
        .unwrap();
    let proof = proof_opt.expect("proof");

    let verifier = Groth16Verifier::default();
    assert!(verifier.verify(&proof).unwrap());
    assert!(cred.verify_claim("birth_year", &pk).is_ok());
}
