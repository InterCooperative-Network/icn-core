use std::collections::HashMap;

/// Stored parameters for a registered zero-knowledge circuit.
#[derive(Clone, Debug)]
pub struct CircuitRecord {
    /// Groth16 proving key bytes.
    pub proving_key: Vec<u8>,
    /// Corresponding verifying key bytes.
    pub verification_key: Vec<u8>,
}

/// In-memory registry mapping circuit slugs and versions to parameters.
#[derive(Default)]
pub struct CircuitRegistry {
    inner: HashMap<String, HashMap<String, CircuitRecord>>, // slug -> version -> record
}

impl CircuitRegistry {
    /// Register a new circuit version.
    pub fn register(
        &mut self,
        slug: &str,
        version: &str,
        proving_key: Vec<u8>,
        verification_key: Vec<u8>,
    ) {
        let record = CircuitRecord { proving_key, verification_key };
        self.inner
            .entry(slug.to_string())
            .or_default()
            .insert(version.to_string(), record);
    }

    /// Fetch a circuit record if present.
    pub fn get(&self, slug: &str, version: &str) -> Option<CircuitRecord> {
        self.inner
            .get(slug)
            .and_then(|m| m.get(version))
            .cloned()
    }

    /// List all known versions for a circuit slug.
    pub fn versions(&self, slug: &str) -> Vec<String> {
        self.inner
            .get(slug)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }
}
