// icn-ccl/src/metadata.rs
use serde::{Serialize, Deserialize};
// use icn_common::Cid; // If using the actual Cid type

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub cid: String, // CID of the compiled WASM module itself
    pub exports: Vec<String>, // List of exported function names (policy hooks)
    pub inputs: Vec<String>,  // Description of expected input structures/types for hooks
    pub version: String,      // Version of the CCL contract (from source or compiler)
    pub source_hash: String,  // Hash of the original .ccl source file for auditability
    // Potentially other fields: author, description, linked policies, etc.
} 