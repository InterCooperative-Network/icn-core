use icn_common::{Cid, Did, ZkCredentialProof, ZkProofType};
use icn_identity::credential::Credential;
use icn_identity::{
    credential::CredentialIssuer,
    generate_ed25519_keypair,
    zk::{Groth16Circuit, Groth16Verifier, ZkError, ZkProver},
    ZkVerifier,
};

use ark_serialize::CanonicalSerialize;
use icn_zk::{prepare_vk, prove, setup, AgeOver18Circuit};
use icn_identity::{sign_message, verify_signature, SignatureBytes, VerifyingKey};
use rand_core::OsRng;

struct Groth16KeyManager {
    pk: ark_groth16::ProvingKey<ark_bn254::Bn254>,
    vk_bytes: Vec<u8>,
    vk_sig: SignatureBytes,
    signer_pk: VerifyingKey,
}

impl Groth16KeyManager {
    fn new() -> Self {
        let circuit = AgeOver18Circuit {
            birth_year: 2000,
            current_year: 2020,
        };
        let mut rng = OsRng;
        let pk = setup(circuit, &mut rng).expect("setup");
        let (sk, signer_pk) = generate_ed25519_keypair();
        let mut vk_bytes = Vec::new();
        pk.vk
            .serialize_compressed(&mut vk_bytes)
            .expect("serialize");
        let sig = sign_message(&sk, &vk_bytes);
        let vk_sig = SignatureBytes::from_ed_signature(sig);
        Self {
            pk,
            vk_bytes,
            vk_sig,
            signer_pk,
        }
    }

    fn verify_vk_signature(&self) -> bool {
        let sig = self.vk_sig.to_ed_signature().unwrap();
        verify_signature(&self.signer_pk, &self.vk_bytes, &sig)
    }
}

struct Groth16Prover {
    pk: ark_groth16::ProvingKey<ark_bn254::Bn254>,
}

impl Groth16Prover {
    fn from_manager(mgr: &Groth16KeyManager) -> Self {
        Self {
            pk: mgr.pk.clone(),
        }
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
        let mut rng = OsRng;
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
            vk_cid: None,
            disclosed_fields: fields.iter().map(|f| f.to_string()).collect(),
            challenge: None,
            backend: ZkProofType::Groth16,
            verification_key: None,
            public_inputs: Some(serde_json::json!([2020])),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn issue_and_verify_groth16_proof() {
    let manager = Groth16KeyManager::new();
    assert!(manager.verify_vk_signature());

    let (sk, pk) = generate_ed25519_keypair();
    let issuer_did = Did::new("key", "issuer");
    let holder_did = Did::new("key", "holder");
    let issuer = CredentialIssuer::new(issuer_did.clone(), sk)
        .with_prover(Box::new(Groth16Prover::from_manager(&manager)));

    let mut claims = std::collections::HashMap::new();
    claims.insert("birth_year".to_string(), "2000".to_string());

    let (cred, proof_opt) = issuer
        .issue(
            holder_did,
            claims,
            Some(Cid::new_v1_sha256(0x55, b"schema")),
            Some(&[]),
            Some(Groth16Circuit::AgeOver18 { current_year: 2020 }),
        )
        .unwrap();
    let proof = proof_opt.expect("proof");

    let pvk = prepare_vk(&manager.pk);
    let verifier = Groth16Verifier::new(pvk, vec![ark_bn254::Fr::from(2020u64)]);
    assert!(verifier.verify(&proof).unwrap());
    assert!(cred.verify_claim("birth_year", &pk).is_ok());
}
