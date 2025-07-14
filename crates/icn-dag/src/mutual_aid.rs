use icn_common::{
    compute_merkle_cid, CommonError, DagBlock, Did, SystemTimeProvider, TimeProvider,
};
use crate::StorageService;
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

/// Registry service storing [`AidResource`] records in a DAG store.
pub struct MutualAidRegistry<S: StorageService<DagBlock>> {
    store: S,
}

impl<S: StorageService<DagBlock>> MutualAidRegistry<S> {
    /// Create a new registry using the provided store.
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Register a new aid resource and return its CID.
    pub fn register(
        &mut self,
        resource: &AidResource,
        author: &Did,
    ) -> Result<String, CommonError> {
        let data = serde_json::to_vec(resource).map_err(|e| {
            CommonError::SerializationError(format!("aid resource: {}", e))
        })?;
        let ts = SystemTimeProvider.unix_seconds();
        let cid = compute_merkle_cid(0x71, &data, &[], ts, author, &None, &None);
        let block = DagBlock {
            cid: cid.clone(),
            data,
            links: vec![],
            timestamp: ts,
            author_did: author.clone(),
            signature: None,
            scope: None,
        };
        self.store.put(&block)?;
        Ok(cid.to_string())
    }

    /// List all registered resources.
    pub fn list(&self) -> Result<Vec<AidResource>, CommonError> {
        let blocks = self.store.list_blocks()?;
        Ok(blocks
            .into_iter()
            .filter_map(|b| serde_json::from_slice::<AidResource>(&b.data).ok())
            .collect())
    }

    /// Find resources that contain the specified tag.
    pub fn find_by_tag(&self, tag: &str) -> Result<Vec<AidResource>, CommonError> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|r| r.tags.iter().any(|t| t == tag))
            .collect())
    }
}
