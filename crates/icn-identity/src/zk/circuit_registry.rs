use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// Entry describing a Groth16 circuit's verifying key and expected public inputs.
#[derive(Clone)]
pub struct CircuitEntry {
    /// Serialized verifying key bytes.
    pub verifying_key: Vec<u8>,
    /// Public inputs expected by the circuit as `u64` values.
    pub public_inputs: Vec<u64>,
}

/// In-memory registry mapping `(claim_type, version)` pairs to circuit data.
#[derive(Default)]
pub struct CircuitRegistry {
    map: HashMap<(String, Option<String>), CircuitEntry>,
}

impl CircuitRegistry {
    /// Register circuit data for the given claim type and optional version.
    pub fn register(&mut self, claim_type: &str, version: Option<&str>, entry: CircuitEntry) {
        self.map.insert(
            (claim_type.to_string(), version.map(|v| v.to_string())),
            entry,
        );
    }

    /// Fetch circuit data for the claim type and version if present.
    pub fn get(&self, claim_type: &str, version: Option<&str>) -> Option<CircuitEntry> {
        self.map
            .get(&(claim_type.to_string(), version.map(|v| v.to_string())))
            .cloned()
            .or_else(|| self.map.get(&(claim_type.to_string(), None)).cloned())
    }
}

static GLOBAL_REGISTRY: Lazy<Mutex<CircuitRegistry>> =
    Lazy::new(|| Mutex::new(CircuitRegistry::default()));

/// Register circuit data in the global registry.
pub fn register_circuit(claim_type: &str, version: Option<&str>, entry: CircuitEntry) {
    GLOBAL_REGISTRY
        .lock()
        .expect("registry mutex poisoned")
        .register(claim_type, version, entry);
}

/// Lookup circuit data from the global registry.
pub fn get_circuit(claim_type: &str, version: Option<&str>) -> Option<CircuitEntry> {
    GLOBAL_REGISTRY
        .lock()
        .expect("registry mutex poisoned")
        .get(claim_type, version)
}
