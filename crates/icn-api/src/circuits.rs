use serde::{Deserialize, Serialize};

/// Request body for `POST /circuits/register`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterCircuitRequest {
    /// Circuit identifier slug, e.g. `age_over_18`.
    pub slug: String,
    /// Semantic version string like `1.0.0`.
    pub version: String,
    /// Base64 encoded Groth16 proving key bytes.
    pub proving_key: String,
    /// Base64 encoded verifying key bytes.
    pub verification_key: String,
}

/// Response body for `GET /circuits/{{slug}}/{{version}}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitResponse {
    /// Circuit slug.
    pub slug: String,
    /// Circuit version string.
    pub version: String,
    /// Verifying key bytes.
    #[serde(with = "serde_bytes")]
    pub verification_key: Vec<u8>,
}

/// Response for listing available versions of a circuit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitVersionsResponse {
    /// Circuit slug.
    pub slug: String,
    /// Known versions.
    pub versions: Vec<String>,
}
