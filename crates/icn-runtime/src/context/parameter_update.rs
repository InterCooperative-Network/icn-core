use icn_common::Did;
use serde::{Deserialize, Serialize};

/// Recorded change to a runtime parameter anchored in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterUpdate {
    pub name: String,
    pub value: String,
    pub timestamp: u64,
    pub signer: Did,
}
