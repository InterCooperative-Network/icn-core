use serde::{Deserialize, Serialize};

use crate::{Did, NodeScope};

/// Unique identifier for a resource token class.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenClassId(pub String);

/// Balance of tokens held in an account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TokenBalance {
    /// Amount of tokens.
    pub amount: u64,
}

/// Metadata describing a class of tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenClassMetadata {
    /// Optional scope that owns or governs this token class.
    pub scope: Option<NodeScope>,
    /// DID of the entity that issued the token class.
    pub issuer: Did,
    /// Display unit or symbol for the token.
    pub unit: String,
    /// Whether tokens can be freely transferred between accounts.
    pub is_transferable: bool,
    /// Indicates if the token is fungible or non-fungible.
    pub fungible: bool,
    /// Human readable name for the token class.
    pub name: String,
    /// Optional longer description.
    pub description: Option<String>,
}
