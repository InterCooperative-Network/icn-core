use crate::{
    sign_message, verify_signature, EdSignature, SigningKey, VerifyingKey, SIGNATURE_LENGTH,
};
use ark_bn254::{Bn254, Fr};
use ark_groth16::ProvingKey;
use ark_relations::r1cs::ConstraintSynthesizer;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use rand::rngs::OsRng;
use directories_next as dirs_next;
use icn_common::CommonError;
use icn_zk::CircuitParameters;
use std::fs;
use std::path::PathBuf;

/// Manage Groth16 proving and verifying keys on disk.
#[derive(Debug, Clone)]
pub struct Groth16KeyManager {
    dir: PathBuf,
    pk: ProvingKey<Bn254>,
}

/// Source of Groth16 parameters when generating keys.
pub enum Groth16KeySource<C> {
    /// Run setup for the provided circuit.
    Circuit(C),
    /// Use pre-generated parameters.
    Params(CircuitParameters),
}

impl Groth16KeyManager {
    /// Generate new parameters for `circuit` and store them under
    /// `~/.icn/zk/<name>`.
    pub fn new<C: ConstraintSynthesizer<Fr>>(
        name: &str,
        source: Groth16KeySource<C>,
        signer: &SigningKey,
    ) -> Result<Self, CommonError> {
        use icn_zk::setup;

        let mut rng = OsRng;
        let pk = match source {
            Groth16KeySource::Circuit(c) => setup(c, &mut rng)
                .map_err(|_| CommonError::CryptoError("groth16 setup failed".into()))?,
            Groth16KeySource::Params(p) => p
                .proving_key()
                .map_err(|_| CommonError::DeserializationError("proving key".into()))?,
        };

        let dir = dirs_next::BaseDirs::new()
            .ok_or_else(|| CommonError::IoError("missing home directory".into()))?
            .home_dir()
            .join(".icn/zk")
            .join(name);
        fs::create_dir_all(&dir).map_err(|e| CommonError::IoError(e.to_string()))?;

        let pk_path = dir.join("proving_key.bin");
        let vk_path = dir.join("verifying_key.bin");
        let sig_path = dir.join("verifying_key.sig");

        let mut pk_bytes = Vec::new();
        pk.serialize_compressed(&mut pk_bytes)
            .map_err(|_| CommonError::SerializationError("proving key".into()))?;
        fs::write(&pk_path, &pk_bytes).map_err(|e| CommonError::IoError(e.to_string()))?;

        let mut vk_bytes = Vec::new();
        pk.vk
            .serialize_compressed(&mut vk_bytes)
            .map_err(|_| CommonError::SerializationError("verifying key".into()))?;
        fs::write(&vk_path, &vk_bytes).map_err(|e| CommonError::IoError(e.to_string()))?;

        let sig = sign_message(signer, &vk_bytes);
        fs::write(&sig_path, sig.to_bytes()).map_err(|e| CommonError::IoError(e.to_string()))?;

        Ok(Self { dir, pk })
    }

    /// Load the proving key previously stored on disk.
    pub fn load_proving_key(&self) -> Result<ProvingKey<Bn254>, CommonError> {
        let path = self.dir.join("proving_key.bin");
        let bytes = fs::read(&path).map_err(|e| CommonError::IoError(e.to_string()))?;
        ProvingKey::deserialize_compressed(&*bytes)
            .map_err(|_| CommonError::DeserializationError("proving key".into()))
    }

    /// Verify the stored verifying key signature with the provided public key.
    pub fn verify_key_signature(&self, signer_pk: &VerifyingKey) -> Result<bool, CommonError> {
        let vk_bytes = fs::read(self.dir.join("verifying_key.bin"))
            .map_err(|e| CommonError::IoError(e.to_string()))?;
        let sig_bytes = fs::read(self.dir.join("verifying_key.sig"))
            .map_err(|e| CommonError::IoError(e.to_string()))?;
        if sig_bytes.len() != SIGNATURE_LENGTH {
            return Err(CommonError::DeserializationError("signature length".into()));
        }
        let sig_array: [u8; SIGNATURE_LENGTH] = sig_bytes
            .try_into()
            .map_err(|_| CommonError::DeserializationError("signature length".into()))?;
        let sig = EdSignature::from_bytes(&sig_array);
        Ok(verify_signature(signer_pk, &vk_bytes, &sig))
    }

    /// Access the in-memory proving key.
    pub fn proving_key(&self) -> &ProvingKey<Bn254> {
        &self.pk
    }
}
