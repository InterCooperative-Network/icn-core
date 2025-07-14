use icn_common::Did;
use serde::{Deserialize, Serialize};

/// A single mutual aid resource entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidResource {
    /// Unique identifier for this resource entry.
    pub id: String,
    /// DID of the entity providing the resource.
    pub provider: Did,
    /// Human readable description of the resource.
    pub description: String,
    /// Optional quantity available.
    pub quantity: Option<u64>,
    /// Optional location information.
    pub location: Option<String>,
    /// Arbitrary tags for matching requests.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Registry of available mutual aid resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidResourceRegistry {
    /// Version timestamp for the registry snapshot.
    pub updated_at: u64,
    /// Collection of resources.
    pub resources: Vec<AidResource>,
}
