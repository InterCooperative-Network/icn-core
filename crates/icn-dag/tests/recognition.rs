use icn_common::{Cid, CommonError, DagBlock, Did, NodeScope};
use icn_dag::{recognition::log_contribution, StorageService};
use std::collections::HashMap;
use std::str::FromStr;

struct MemStore {
    map: HashMap<Cid, DagBlock>,
}
impl MemStore {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}
impl StorageService<DagBlock> for MemStore {
    fn put(&mut self, block: &DagBlock) -> Result<(), CommonError> {
        self.map.insert(block.cid.clone(), block.clone());
        Ok(())
    }
    fn get(&self, cid: &Cid) -> Result<Option<DagBlock>, CommonError> {
        Ok(self.map.get(cid).cloned())
    }
    fn delete(&mut self, cid: &Cid) -> Result<(), CommonError> {
        self.map.remove(cid);
        Ok(())
    }
    fn contains(&self, cid: &Cid) -> Result<bool, CommonError> {
        Ok(self.map.contains_key(cid))
    }
    fn list_blocks(&self) -> Result<Vec<DagBlock>, CommonError> {
        Ok(self.map.values().cloned().collect())
    }
    fn pin_block(&mut self, _cid: &Cid) -> Result<(), CommonError> {
        Ok(())
    }
    fn unpin_block(&mut self, _cid: &Cid) -> Result<(), CommonError> {
        Ok(())
    }
    fn prune_expired(&mut self, _now: u64) -> Result<Vec<Cid>, CommonError> {
        Ok(vec![])
    }
    fn set_ttl(&mut self, _cid: &Cid, _ttl: Option<u64>) -> Result<(), CommonError> {
        Ok(())
    }
    fn get_metadata(&self, _cid: &Cid) -> Result<Option<icn_dag::BlockMetadata>, CommonError> {
        Ok(None)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[test]
fn log_contribution_stores_block() {
    let mut store = MemStore::new();
    let did = Did::from_str("did:example:alice").unwrap();
    let cid = log_contribution(
        &mut store,
        did,
        "fixed".into(),
        Some(NodeScope("coop".into())),
    )
    .unwrap();
    assert!(store.contains(&cid).unwrap());
}
