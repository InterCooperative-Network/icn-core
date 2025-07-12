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
    /// Fields from the credential that were disclosed in plain text.
    pub disclosed_fields: Vec<String>,
    /// Optional challenge used in the proof generation.
    pub challenge: Option<String>,
    /// Backend proving system used for this proof.
    pub backend: ZkProofType,
}
