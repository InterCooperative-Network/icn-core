use ark_bn254::Bn254;
use ark_groth16::{PreparedVerifyingKey, ProvingKey};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use serde::{Deserialize, Serialize};

/// Serialized circuit parameters produced by a trusted setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitParameters {
    /// Compressed Groth16 proving key bytes.
    #[serde(with = "serde_bytes")]
    pub proving_key: Vec<u8>,
}

impl CircuitParameters {
    /// Convert the parameters into a [`ProvingKey`].
    pub fn proving_key(&self) -> Result<ProvingKey<Bn254>, ark_serialize::SerializationError> {
        ProvingKey::<Bn254>::deserialize_compressed(&self.proving_key[..])
    }

    /// Derive a prepared verifying key from the stored proving key.
    pub fn prepared_vk(
        &self,
    ) -> Result<PreparedVerifyingKey<Bn254>, ark_serialize::SerializationError> {
        let pk = self.proving_key()?;
        Ok(crate::prepare_vk(&pk))
    }

    /// Create parameters from an existing proving key.
    pub fn from_proving_key(
        pk: &ProvingKey<Bn254>,
    ) -> Result<Self, ark_serialize::SerializationError> {
        let mut bytes = Vec::new();
        pk.serialize_compressed(&mut bytes)?;
        Ok(Self { proving_key: bytes })
    }
}

/// Storage trait for circuit parameters keyed by circuit name.
pub trait CircuitParametersStorage {
    /// Store parameters for the given circuit name.
    fn put(&mut self, name: &str, params: CircuitParameters);
    /// Fetch parameters for the given circuit name if present.
    fn get(&self, name: &str) -> Option<CircuitParameters>;
}

/// In-memory implementation of [`CircuitParametersStorage`].
#[derive(Default)]
pub struct MemoryParametersStorage {
    inner: std::collections::HashMap<String, CircuitParameters>,
}

impl CircuitParametersStorage for MemoryParametersStorage {
    fn put(&mut self, name: &str, params: CircuitParameters) {
        self.inner.insert(name.to_string(), params);
    }

    fn get(&self, name: &str) -> Option<CircuitParameters> {
        self.inner.get(name).cloned()
    }
}
