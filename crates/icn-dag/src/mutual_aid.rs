use crate::{DagBlock, StorageService};
use icn_common::{
    compute_merkle_cid, Cid, CommonError, Did, NodeScope, SystemTimeProvider, TimeProvider,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidResource {
    pub resource_type: String,
    pub description: String,
    pub quantity: u64,
    pub provider: Did,
    pub timestamp: u64,
}

pub fn register_resource(
    store: &mut dyn StorageService<DagBlock>,
    provider: Did,
    resource_type: String,
    description: String,
    quantity: u64,
    scope: Option<NodeScope>,
) -> Result<Cid, CommonError> {
    let record = AidResource {
        resource_type,
        description,
        quantity,
        provider: provider.clone(),
        timestamp: SystemTimeProvider.unix_seconds(),
    };
    let data =
        serde_json::to_vec(&record).map_err(|e| CommonError::SerializationError(e.to_string()))?;
    let cid = compute_merkle_cid(0x71, &data, &[], record.timestamp, &provider, &None, &scope);
    let block = DagBlock {
        cid: cid.clone(),
        data,
        links: vec![],
        timestamp: record.timestamp,
        author_did: provider,
        signature: None,
        scope,
    };
    store.put(&block)?;
    Ok(cid)
}

pub fn list_resources<S: StorageService<DagBlock>>(
    store: &S,
) -> Result<Vec<(Cid, AidResource)>, CommonError> {
    let mut out = Vec::new();
    for block in store.list_blocks()? {
        if let Ok(rec) = serde_json::from_slice::<AidResource>(&block.data) {
            out.push((block.cid.clone(), rec));
        }
    }
    Ok(out)
}
