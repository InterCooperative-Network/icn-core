use crate::{DagBlock, StorageService};
use icn_common::{
    compute_merkle_cid, CommonError, Did, NodeScope, SystemTimeProvider, TimeProvider,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidResource {
    pub id: String,
    pub description: String,
    pub provider: Did,
    pub quantity: u64,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub fn register_resource(
    store: &mut dyn StorageService<DagBlock>,
    resource: AidResource,
    scope: Option<NodeScope>,
) -> Result<icn_common::Cid, CommonError> {
    let data = serde_json::to_vec(&resource)
        .map_err(|e| CommonError::SerializationError(e.to_string()))?;
    let cid = compute_merkle_cid(
        0x71,
        &data,
        &[],
        SystemTimeProvider.unix_seconds(),
        &resource.provider,
        &None,
        &scope,
    );
    let block = DagBlock {
        cid: cid.clone(),
        data,
        links: vec![],
        timestamp: SystemTimeProvider.unix_seconds(),
        author_did: resource.provider.clone(),
        signature: None,
        scope,
    };
    store.put(&block)?;
    Ok(cid)
}

pub fn get_resource(
    store: &dyn StorageService<DagBlock>,
    cid: &icn_common::Cid,
) -> Result<Option<AidResource>, CommonError> {
    if let Some(block) = store.get(cid)? {
        let res: AidResource = serde_json::from_slice(&block.data)
            .map_err(|e| CommonError::DeserializationError(e.to_string()))?;
        Ok(Some(res))
    } else {
        Ok(None)
    }
}

pub fn list_resources(
    store: &dyn StorageService<DagBlock>,
) -> Result<Vec<AidResource>, CommonError> {
    let blocks = store.list_blocks()?;
    let mut out = Vec::new();
    for b in blocks {
        if let Ok(res) = serde_json::from_slice::<AidResource>(&b.data) {
            out.push(res);
        }
    }
    Ok(out)
}
