use serde::{Deserialize, Serialize};

/// Request to register a new zero-knowledge circuit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterCircuitRequest {
    /// Unique circuit slug.
    pub slug: String,
    /// Semantic version string.
    pub version: String,
    /// Base64 encoded Groth16 proving key bytes.
    pub proving_key: String,
    /// Base64 encoded verifying key bytes.
    pub verification_key: String,
}

/// Metadata returned for a specific circuit version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitMetadataResponse {
    pub slug: String,
    pub version: String,
    #[serde(with = "serde_bytes")]
    pub verification_key: Vec<u8>,
}

/// List of available versions for a circuit slug.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitVersionsResponse {
    pub slug: String,
    pub versions: Vec<String>,
}
