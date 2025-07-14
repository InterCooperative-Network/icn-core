use icn_common::Did;
use serde::{Deserialize, Serialize};

/// Record describing a resource available for mutual aid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidResource {
    /// Unique identifier for the resource record.
    pub id: String,
    /// Human readable description.
    pub description: String,
    /// DID of the entity offering the resource.
    pub provider: Did,
    /// Quantity available, if applicable.
    pub quantity: u64,
    /// Arbitrary classification tags.
    #[serde(default)]
    pub tags: Vec<String>,
}
