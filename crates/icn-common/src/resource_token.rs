use serde::{Deserialize, Serialize};

use crate::{Did, NodeScope};

/// Identifier for a resource token class.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenClassId(pub String);

/// Representation of a token balance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenBalance {
    /// Amount of tokens held.
    pub amount: u64,
}

/// Metadata describing a token class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenClassMetadata {
    /// Optional scope that this token is limited to.
    pub scope: Option<NodeScope>,
    /// DID of the issuer that created the token class.
    pub issuer: Did,
    /// Unit name or symbol (e.g. "kg", "credit").
    pub unit: String,
    /// Whether tokens can be transferred between accounts.
    pub is_transferable: bool,
    /// Indicates if the tokens are fungible or non-fungible.
    pub fungible: bool,
    /// Optional human readable description.
    pub description: Option<String>,
}
