use serde::{Deserialize, Serialize};

use crate::{Cid, Did};

/// Supported zero-knowledge proof backends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZkProofType {
    /// Groth16 zk-SNARK proofs
    Groth16,
    /// Bulletproofs proving system
    Bulletproofs,
    /// Catch-all for custom or future proof systems
    Other(String),
}

/// A verifiable credential proof generated via a zero-knowledge protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZkCredentialProof {
    /// DID of the credential issuer.
    pub issuer: Did,
    /// DID of the credential holder.
    pub holder: Did,
    /// Type or semantic tag of the claim being proven.
    pub claim_type: String,
    /// Raw bytes of the zero-knowledge proof.
    #[serde(with = "serde_bytes")]
    pub proof: Vec<u8>,
    /// CID of the credential schema this proof adheres to.
    pub schema: Cid,
    /// Optional CID referencing the verifying key for this proof.
    pub vk_cid: Option<Cid>,
    /// Fields from the credential that were disclosed in plain text.
    pub disclosed_fields: Vec<String>,
    /// Optional challenge used in the proof generation.
    pub challenge: Option<String>,
    /// Backend proving system used for this proof.
    pub backend: ZkProofType,
    /// Optional verification key bytes used to verify the proof.
    #[serde(with = "serde_bytes", default, skip_serializing_if = "Option::is_none")]
    pub verification_key: Option<Vec<u8>>,
    /// Optional public input values required for verification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_inputs: Option<serde_json::Value>,
}
